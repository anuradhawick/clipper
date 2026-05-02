use crate::content_managers::message_bus::{AppMessage, MessageBus, NetworkClipboardPayload};
use crate::error::{with_error_event, AppError, AppResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Entry, HashMap};
use std::env;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket as StdUdpSocket};
use std::sync::Arc;
use std::time::Duration;
use tauri::{async_runtime, async_runtime::JoinHandle, AppHandle, Emitter, State};
use tokio::net::UdpSocket;
use tokio::sync::{watch, Mutex};
use uuid::Uuid;

const DISCOVERY_MULTICAST_HOST: Ipv4Addr = Ipv4Addr::new(239, 255, 42, 99);
const DISCOVERY_PORT: u16 = 34254;
const CLIPBOARD_PORT: u16 = 34255;
const DISCOVERY_ANNOUNCE_INTERVAL_SECS: u64 = 10;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DiscoveryPacket {
    name: String,
    clipboard_port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum WirePacket {
    Clipboard {
        source_name: String,
        text: String,
        timestamp: String,
    },
    AuthRequest {
        source_name: String,
        otp: String,
    },
    AuthResponse {
        source_name: String,
        approved: bool,
    },
}

#[derive(Clone, Debug)]
struct PeerRecord {
    name: String,
    addr: SocketAddr,
    authorized: bool,
}

struct NetworkManagerState {
    running: bool,
    local_name: String,
    local_otp: Option<String>,
    peers: HashMap<SocketAddr, PeerRecord>,
    shutdown_tx: Option<watch::Sender<bool>>,
    tasks: Vec<JoinHandle<()>>,
}

#[derive(Debug, Serialize)]
pub struct NetStatus {
    pub running: bool,
    pub local_name: String,
    pub otp: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct NetworkPeerEntry {
    pub id: String,
    pub name: String,
    pub authorized: bool,
}

pub struct NetworkManager {
    app_handle: AppHandle,
    bus: MessageBus,
    state: Mutex<NetworkManagerState>,
}

impl NetworkManager {
    pub async fn new(bus: MessageBus, app_handle: AppHandle) -> Arc<Self> {
        let manager = Arc::new(Self {
            app_handle,
            bus,
            state: Mutex::new(NetworkManagerState {
                running: false,
                local_name: Self::resolve_local_name(),
                local_otp: None,
                peers: HashMap::new(),
                shutdown_tx: None,
                tasks: Vec::new(),
            }),
        });

        manager.start().await;
        manager
    }

    pub async fn start(self: &Arc<Self>) {
        {
            let mut state = self.state.lock().await;
            if state.running {
                return;
            }

            let local_name = state.local_name.clone();
            let (shutdown_tx, shutdown_rx) = watch::channel(false);

            state.running = true;
            state.shutdown_tx = Some(shutdown_tx);
            state.tasks = vec![
                async_runtime::spawn(Self::run_presence_announcer(
                    local_name.clone(),
                    shutdown_rx.clone(),
                )),
                async_runtime::spawn(Self::run_peer_discovery(
                    local_name.clone(),
                    Arc::clone(self),
                    shutdown_rx.clone(),
                )),
                async_runtime::spawn(Self::run_clipboard_transport(
                    local_name,
                    Arc::clone(self),
                    self.bus.clone(),
                    shutdown_rx,
                )),
            ];
        }

        log::info!("Network manager started");
        if self.app_handle.emit("net_status_changed", true).is_err() {
            log::error!("Unable to emit: net_status_changed");
        }
    }

    pub async fn stop(&self) {
        let (shutdown_tx, tasks) = {
            let mut state = self.state.lock().await;
            if !state.running {
                return;
            }
            state.running = false;
            state.peers.clear();
            (state.shutdown_tx.take(), std::mem::take(&mut state.tasks))
        };

        if let Some(tx) = shutdown_tx {
            let _ = tx.send(true);
        }
        for task in tasks {
            task.abort();
        }

        log::info!("Network manager stopped");
        if self.app_handle.emit("net_status_changed", false).is_err() {
            log::error!("Unable to emit: net_status_changed");
        }
    }

    fn generate_otp() -> String {
        loop {
            let bytes = Uuid::new_v4();
            let b = bytes.as_bytes();
            let n = u32::from_le_bytes([b[0], b[1], b[2] & 0x0F, 0]);
            if n < 1_000_000 {
                return format!("{n:06}");
            }
        }
    }

    pub async fn refresh_otp(&self) -> String {
        let otp = Self::generate_otp();
        self.state.lock().await.local_otp = Some(otp.clone());
        otp
    }

    pub async fn get_status(&self) -> NetStatus {
        let s = self.state.lock().await;
        NetStatus {
            running: s.running,
            local_name: s.local_name.clone(),
            otp: s.local_otp.clone(),
        }
    }

    pub async fn list_peers(&self) -> Vec<NetworkPeerEntry> {
        self.state
            .lock()
            .await
            .peers
            .values()
            .map(|p| NetworkPeerEntry {
                id: p.addr.to_string(),
                name: p.name.clone(),
                authorized: p.authorized,
            })
            .collect()
    }

    pub async fn request_auth(&self, peer_id: &str, otp: &str) -> AppResult<()> {
        let (addr, local_name) = {
            let state = self.state.lock().await;
            let addr = Self::parse_peer_id(peer_id)?;
            if !state.peers.contains_key(&addr) {
                return Err(AppError::validation(format!("Peer not found: {peer_id}")));
            }
            (addr, state.local_name.clone())
        };

        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        let packet = WirePacket::AuthRequest {
            source_name: local_name,
            otp: otp.to_string(),
        };

        Self::send_wire_packet(&socket, &packet, addr).await?;

        log::info!("Network manager sent auth request to {peer_id}");
        Ok(())
    }

    pub async fn revoke_peer(&self, peer_id: &str) -> AppResult<()> {
        let addr = Self::parse_peer_id(peer_id)?;
        let revoked_peer = {
            let mut state = self.state.lock().await;
            state.peers.get_mut(&addr).map(|peer| {
                peer.authorized = false;
                peer.name.clone()
            })
        };

        if let Some(name) = revoked_peer {
            log::info!("Network manager revoked peer {}", name);
            self.notify_peers_updated();
        }
        Ok(())
    }

    async fn run_presence_announcer(local_name: String, mut shutdown_rx: watch::Receiver<bool>) {
        let socket = match Self::create_multicast_sender() {
            Ok(s) => s,
            Err(e) => {
                log::error!("Network manager failed to create discovery sender: {}", e);
                return;
            }
        };

        let mut interval =
            tokio::time::interval(Duration::from_secs(DISCOVERY_ANNOUNCE_INTERVAL_SECS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let target = SocketAddr::from((DISCOVERY_MULTICAST_HOST, DISCOVERY_PORT));

        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => break,
                _ = interval.tick() => {
                    let packet = DiscoveryPacket {
                        name: local_name.clone(),
                        clipboard_port: CLIPBOARD_PORT,
                    };
                    match serde_json::to_vec(&packet) {
                        Ok(payload) => {
                            if let Err(e) = socket.send_to(&payload, target).await {
                                log::warn!("Network manager failed to announce presence: {}", e);
                            }
                        }
                        Err(e) => {
                            log::warn!("Network manager failed to serialize discovery packet: {}", e);
                        }
                    }
                }
            }
        }
    }

    async fn run_peer_discovery(
        local_name: String,
        manager: Arc<Self>,
        mut shutdown_rx: watch::Receiver<bool>,
    ) {
        let socket = match Self::bind_discovery_socket() {
            Ok(s) => s,
            Err(e) => {
                log::error!("Network manager failed to bind discovery socket: {}", e);
                return;
            }
        };

        let mut buf = [0_u8; 2048];

        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => break,
                result = socket.recv_from(&mut buf) => {
                    match result {
                        Ok((len, addr)) => {
                            match serde_json::from_slice::<DiscoveryPacket>(&buf[..len]) {
                                Ok(pkt) if pkt.name != local_name => {
                                    let peer_addr = SocketAddr::new(addr.ip(), pkt.clipboard_port);
                                    manager.upsert_discovered_peer(pkt.name, peer_addr).await;
                                }
                                Ok(_) => {}
                                Err(e) => {
                                    log::debug!("Network manager ignored invalid discovery packet: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Network manager failed to receive discovery packet: {}", e);
                        }
                    }
                }
            }
        }
    }

    async fn run_clipboard_transport(
        local_name: String,
        manager: Arc<Self>,
        bus: MessageBus,
        mut shutdown_rx: watch::Receiver<bool>,
    ) {
        let socket = match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, CLIPBOARD_PORT)).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("Network manager failed to bind clipboard socket: {}", e);
                return;
            }
        };

        let mut receiver = bus.subscribe();
        let mut buf = [0_u8; 65_535];

        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => break,
                message = receiver.recv() => {
                    match message {
                        Ok(AppMessage::AddedToClipboard(text)) => {
                            manager
                                .send_clipboard_to_authorized_peers(&socket, &local_name, text)
                                .await;
                        }
                        Ok(_) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            log::warn!("Network manager lagged and skipped {} messages", n);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
                result = socket.recv_from(&mut buf) => {
                    match result {
                        Ok((len, from)) => {
                            match serde_json::from_slice::<WirePacket>(&buf[..len]) {
                                Ok(packet) => manager
                                    .handle_wire_packet(packet, from, &local_name, &bus, &socket)
                                    .await,
                                Err(e) => {
                                    log::debug!("Network manager ignored invalid wire packet: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Network manager failed to receive wire packet: {}", e);
                        }
                    }
                }
            }
        }
    }

    async fn upsert_discovered_peer(&self, name: String, addr: SocketAddr) {
        let is_new = {
            let mut state = self.state.lock().await;
            match state.peers.entry(addr) {
                Entry::Vacant(entry) => {
                    entry.insert(PeerRecord {
                        name: name.clone(),
                        addr,
                        authorized: false,
                    });
                    true
                }
                Entry::Occupied(_) => false,
            }
        };

        if is_new {
            log::info!("Network manager discovered peer {} at {}", name, addr);
            self.notify_peers_updated();
        }
    }

    async fn is_authorized(&self, addr: SocketAddr) -> bool {
        self.state
            .lock()
            .await
            .peers
            .get(&addr)
            .is_some_and(|p| p.authorized)
    }

    async fn authorized_peers(&self) -> Vec<PeerRecord> {
        self.state
            .lock()
            .await
            .peers
            .values()
            .filter(|p| p.authorized)
            .cloned()
            .collect()
    }

    async fn send_clipboard_to_authorized_peers(
        &self,
        socket: &UdpSocket,
        source_name: &str,
        text: String,
    ) {
        let packet = WirePacket::Clipboard {
            source_name: source_name.to_string(),
            text,
            timestamp: Utc::now().to_rfc3339(),
        };

        let payload = match serde_json::to_vec(&packet) {
            Ok(payload) => payload,
            Err(e) => {
                log::warn!(
                    "Network manager failed to serialize clipboard packet: {}",
                    e
                );
                return;
            }
        };

        for peer in self.authorized_peers().await {
            if let Err(e) = socket.send_to(&payload, peer.addr).await {
                log::debug!(
                    "Network manager failed to send clipboard to {} ({}): {}",
                    peer.name,
                    peer.addr,
                    e
                );
            }
        }
    }

    async fn handle_wire_packet(
        &self,
        packet: WirePacket,
        from: SocketAddr,
        local_name: &str,
        bus: &MessageBus,
        socket: &UdpSocket,
    ) {
        match packet {
            WirePacket::Clipboard {
                source_name, text, ..
            } if source_name != local_name => {
                if self.is_authorized(from).await {
                    let payload = NetworkClipboardPayload { source_name, text };
                    if bus
                        .send(AppMessage::NetworkClipboardReceived(payload))
                        .is_err()
                    {
                        log::error!("Unable to send message: NetworkClipboardReceived");
                    }
                }
            }
            WirePacket::AuthRequest { source_name, otp } => {
                self.handle_auth_request(source_name, from, &otp, socket)
                    .await;
            }
            WirePacket::AuthResponse {
                source_name,
                approved,
            } => {
                self.handle_auth_response(source_name, from, approved).await;
            }
            WirePacket::Clipboard { .. } => {}
        }
    }

    async fn handle_auth_request(
        &self,
        source_name: String,
        from: SocketAddr,
        otp: &str,
        socket: &UdpSocket,
    ) {
        let peer_addr = SocketAddr::new(from.ip(), CLIPBOARD_PORT);
        let (local_otp, local_name) = {
            let mut state = self.state.lock().await;
            let consumed = state.local_otp.take();
            (consumed, state.local_name.clone())
        };

        let approved = local_otp.as_deref() == Some(otp);

        if approved {
            self.authorize_peer(source_name.clone(), peer_addr).await;
            log::info!(
                "Network manager authorized peer {} ({})",
                source_name,
                peer_addr
            );
            self.notify_peers_updated();
        } else {
            log::warn!(
                "Network manager rejected auth request from {} ({}): invalid or expired OTP",
                source_name,
                from
            );
        }

        let response = WirePacket::AuthResponse {
            source_name: local_name,
            approved,
        };
        if let Err(e) = Self::send_wire_packet(socket, &response, peer_addr).await {
            log::warn!(
                "Network manager failed to send auth response to {}: {}",
                peer_addr,
                e
            );
        }
    }

    async fn handle_auth_response(&self, source_name: String, from: SocketAddr, approved: bool) {
        if approved {
            self.authorize_peer(source_name.clone(), from).await;
            log::info!(
                "Network manager: peer {} ({}) approved our auth request",
                source_name,
                from
            );
            self.notify_peers_updated();
        } else {
            log::warn!(
                "Network manager: peer {} ({}) rejected our auth request",
                source_name,
                from
            );
        }
    }

    async fn authorize_peer(&self, name: String, addr: SocketAddr) {
        let mut state = self.state.lock().await;
        match state.peers.entry(addr) {
            Entry::Occupied(mut entry) => {
                let peer = entry.get_mut();
                peer.authorized = true;
                peer.name = name;
            }
            Entry::Vacant(entry) => {
                entry.insert(PeerRecord {
                    name,
                    addr,
                    authorized: true,
                });
            }
        }
    }

    fn parse_peer_id(peer_id: &str) -> AppResult<SocketAddr> {
        peer_id
            .parse()
            .map_err(|_| AppError::validation(format!("Invalid peer id: {peer_id}")))
    }

    async fn send_wire_packet(
        socket: &UdpSocket,
        packet: &WirePacket,
        addr: SocketAddr,
    ) -> AppResult<()> {
        let payload =
            serde_json::to_vec(packet).map_err(|e| AppError::RuntimeError(e.to_string()))?;
        socket
            .send_to(&payload, addr)
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;
        Ok(())
    }

    fn notify_peers_updated(&self) {
        if self.app_handle.emit("net_peers_updated", ()).is_err() {
            log::error!("Unable to emit: net_peers_updated");
        }
    }

    fn resolve_local_name() -> String {
        ["CLIPPER_DEVICE_NAME", "HOSTNAME", "COMPUTERNAME"]
            .into_iter()
            .find_map(|key| env::var(key).ok().and_then(Self::normalize_local_name))
            .or_else(|| Self::normalize_local_name(tauri_plugin_os::hostname()))
            .unwrap_or_else(|| "Clipper".to_string())
    }

    fn normalize_local_name(name: impl AsRef<str>) -> Option<String> {
        let trimmed = name.as_ref().trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    fn create_multicast_sender() -> std::io::Result<UdpSocket> {
        let socket = StdUdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
        socket.set_nonblocking(true)?;
        socket.set_multicast_ttl_v4(1)?;
        UdpSocket::from_std(socket)
    }

    fn bind_discovery_socket() -> std::io::Result<UdpSocket> {
        let socket = StdUdpSocket::bind((Ipv4Addr::UNSPECIFIED, DISCOVERY_PORT))?;
        socket.set_nonblocking(true)?;
        socket.join_multicast_v4(&DISCOVERY_MULTICAST_HOST, &Ipv4Addr::UNSPECIFIED)?;
        UdpSocket::from_std(socket)
    }
}

#[cfg(test)]
mod tests {
    use super::NetworkManager;

    #[test]
    fn normalize_local_name_rejects_blank_values() {
        assert_eq!(NetworkManager::normalize_local_name(""), None);
        assert_eq!(NetworkManager::normalize_local_name("   "), None);
    }

    #[test]
    fn normalize_local_name_trims_valid_values() {
        assert_eq!(
            NetworkManager::normalize_local_name("  desk-01  "),
            Some("desk-01".to_string())
        );
    }
}

#[tauri::command]
pub async fn net_get_status(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
) -> AppResult<NetStatus> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Reading network status");
        Ok(state.get_status().await)
    })
    .await
}

#[tauri::command]
pub async fn net_list_peers(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
) -> AppResult<Vec<NetworkPeerEntry>> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Listing network peers");
        Ok(state.list_peers().await)
    })
    .await
}

#[tauri::command]
pub async fn net_generate_otp(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
) -> AppResult<String> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Generating network OTP");
        Ok(state.refresh_otp().await)
    })
    .await
}

#[tauri::command]
pub async fn net_authorize_peer(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
    peer_id: String,
    otp: String,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Authorizing network peer: {:#?}", peer_id);
        state.request_auth(&peer_id, &otp).await
    })
    .await
}

#[tauri::command]
pub async fn net_revoke_peer(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
    peer_id: String,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Revoking network peer: {:#?}", peer_id);
        state.revoke_peer(&peer_id).await
    })
    .await
}

#[tauri::command]
pub async fn net_start(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Starting network manager");
        let manager = Arc::clone(&*state);
        manager.start().await;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn net_stop(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Stopping network manager");
        state.stop().await;
        Ok(())
    })
    .await
}
