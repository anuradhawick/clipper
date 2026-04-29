use crate::content_managers::message_bus::{AppMessage, MessageBus, NetworkClipboardPayload};
use crate::error::{with_error_event, AppError, AppResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket as StdUdpSocket};
use std::sync::Arc;
use std::time::Duration;
use tauri::{async_runtime, AppHandle, Emitter, State};
use tokio::net::UdpSocket;
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;
use uuid::Uuid;

const DISCOVERY_MULTICAST_HOST: Ipv4Addr = Ipv4Addr::new(239, 255, 42, 99);
const DISCOVERY_PORT: u16 = 34254;
const CLIPBOARD_PORT: u16 = 34255;
const DISCOVERY_ANNOUNCE_INTERVAL_SECS: u64 = 10;

// ─── Wire types ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DiscoveryPacket {
    name: String,
    clipboard_port: u16,
}

/// All packets exchanged on CLIPBOARD_PORT are tagged with `kind`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum WirePacket {
    Clipboard {
        source_name: String,
        text: String,
        timestamp: String,
    },
    /// Sent by Device B to ask Device A to authorize it.
    /// `otp` is the 6-digit code shown on Device A's UI.
    AuthRequest { source_name: String, otp: String },
    /// Reply from Device A after verifying the OTP.
    AuthResponse { source_name: String, approved: bool },
}

// ─── Internal state ───────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct PeerRecord {
    name: String,
    addr: SocketAddr,
    authorized: bool,
}

struct NetworkManagerState {
    running: bool,
    local_name: String,
    /// 6-digit OTP for the next inbound auth attempt.
    /// `None` means no OTP is currently active — the frontend must call
    /// `net_generate_otp` to issue one.  The value is consumed (set to `None`)
    /// on the first inbound `AuthRequest`, whether it matches or not.
    local_otp: Option<String>,
    peers: HashMap<SocketAddr, PeerRecord>,
    shutdown_tx: Option<watch::Sender<bool>>,
    tasks: Vec<JoinHandle<()>>,
}

// ─── Public DTOs ──────────────────────────────────────────────────────────────

/// Returned by `net_get_status`.
#[derive(Debug, Serialize)]
pub struct NetStatus {
    pub running: bool,
    pub local_name: String,
    /// The active OTP if one has been generated, `None` otherwise.
    /// The frontend should call `net_generate_otp` to obtain a fresh code.
    pub otp: Option<String>,
}

/// One element of the list returned by `net_list_peers`.
#[derive(Clone, Debug, Serialize)]
pub struct NetworkPeerEntry {
    /// Stable peer identity – the stringified socket address (`ip:port`).
    pub id: String,
    pub name: String,
    pub authorized: bool,
}

// ─── Manager ──────────────────────────────────────────────────────────────────

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

    // ─── Lifecycle ──────────────────────────────────────────────────────────

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

    // ─── OTP ────────────────────────────────────────────────────────────────

    fn generate_otp() -> String {
        // Use 6 bytes from a UUID v4 (OS CSPRNG via getrandom) to form a
        // 6-digit code without modulo bias: pick a random value in [0, 999999]
        // using the bottom 20 bits of the first 4 bytes (max value 1_048_575),
        // then reject values >= 1_000_000 and retry. Expected retries < 5%.
        loop {
            let bytes = Uuid::new_v4();
            let b = bytes.as_bytes();
            let n = u32::from_le_bytes([b[0], b[1], b[2], b[3] & 0x0F]);
            if n < 1_000_000 {
                return format!("{n:06}");
            }
        }
    }

    /// Generate a new 6-digit OTP, store it as active, and return it.
    /// Any previously active OTP is discarded.
    pub async fn refresh_otp(&self) -> String {
        let otp = Self::generate_otp();
        self.state.lock().await.local_otp = Some(otp.clone());
        otp
    }

    // ─── Queries used by Tauri commands ─────────────────────────────────────

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

    // ─── Authorization ────────────────────────────────────────────────────────

    /// Send an auth-request packet to `peer_id` carrying the OTP shown on that
    /// peer's UI.  The peer will verify it and respond; the response is handled
    /// inside `run_clipboard_transport`.
    pub async fn request_auth(&self, peer_id: &str, otp: &str) -> AppResult<()> {
        let (addr, local_name) = {
            let state = self.state.lock().await;
            let addr: SocketAddr = peer_id
                .parse()
                .map_err(|_| AppError::validation(format!("Invalid peer id: {peer_id}")))?;
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
        let payload =
            serde_json::to_vec(&packet).map_err(|e| AppError::RuntimeError(e.to_string()))?;

        socket
            .send_to(&payload, addr)
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        log::info!("Network manager sent auth request to {peer_id}");
        Ok(())
    }

    /// Remove authorization from a peer so clipboard sharing stops.
    pub async fn revoke_peer(&self, peer_id: &str) -> AppResult<()> {
        let addr: SocketAddr = peer_id
            .parse()
            .map_err(|_| AppError::validation(format!("Invalid peer id: {peer_id}")))?;

        let mut state = self.state.lock().await;
        if let Some(peer) = state.peers.get_mut(&addr) {
            peer.authorized = false;
            log::info!("Network manager revoked peer {}", peer.name);
        }
        drop(state);

        if self.app_handle.emit("net_peers_updated", ()).is_err() {
            log::error!("Unable to emit: net_peers_updated");
        }
        Ok(())
    }

    // ─── Background tasks ─────────────────────────────────────────────────────

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
                            let packet = WirePacket::Clipboard {
                                source_name: local_name.clone(),
                                text,
                                timestamp: Utc::now().to_rfc3339(),
                            };
                            if let Ok(payload) = serde_json::to_vec(&packet) {
                                for peer in manager.authorized_peers().await {
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
                                Ok(WirePacket::Clipboard { source_name, text, .. })
                                    if source_name != local_name =>
                                {
                                    if manager.is_authorized(from).await {
                                        let payload = NetworkClipboardPayload { source_name, text };
                                        if bus.send(AppMessage::NetworkClipboardReceived(payload)).is_err() {
                                            log::error!("Unable to send message: NetworkClipboardReceived");
                                        }
                                    }
                                }
                                Ok(WirePacket::AuthRequest { source_name, otp }) => {
                                    manager
                                        .handle_auth_request(source_name, from, &otp, &socket)
                                        .await;
                                }
                                Ok(WirePacket::AuthResponse { source_name, approved }) => {
                                    manager.handle_auth_response(source_name, from, approved).await;
                                }
                                Ok(_) => {}
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

    // ─── Internal helpers ─────────────────────────────────────────────────────

    async fn upsert_discovered_peer(&self, name: String, addr: SocketAddr) {
        let is_new = {
            let mut state = self.state.lock().await;
            let was_present = state.peers.contains_key(&addr);
            if !was_present {
                state.peers.insert(
                    addr,
                    PeerRecord {
                        name: name.clone(),
                        addr,
                        authorized: false,
                    },
                );
            }
            !was_present
        };

        if is_new {
            log::info!("Network manager discovered peer {} at {}", name, addr);
            if self.app_handle.emit("net_peers_updated", ()).is_err() {
                log::error!("Unable to emit: net_peers_updated");
            }
        }
    }

    async fn is_authorized(&self, addr: SocketAddr) -> bool {
        self.state
            .lock()
            .await
            .peers
            .get(&addr)
            .map_or(false, |p| p.authorized)
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

    /// Called when we receive an `AuthRequest` on CLIPBOARD_PORT.
    /// Verifies the OTP, updates peer state, and sends an `AuthResponse` back
    /// to the requester's CLIPBOARD_PORT (so their transport loop receives it).
    /// The OTP is consumed (set to `None`) on every attempt, whether it matches
    /// or not, enforcing single-use semantics.
    async fn handle_auth_request(
        &self,
        source_name: String,
        from: SocketAddr,
        otp: &str,
        socket: &UdpSocket,
    ) {
        let (local_otp, local_name) = {
            let mut state = self.state.lock().await;
            // Consume the OTP regardless of whether the request succeeds.
            let consumed = state.local_otp.take();
            (consumed, state.local_name.clone())
        };

        let approved = local_otp.as_deref() == Some(otp);

        if approved {
            let mut state = self.state.lock().await;
            state
                .peers
                .entry(from)
                .and_modify(|p| {
                    p.authorized = true;
                    p.name = source_name.clone();
                })
                .or_insert_with(|| PeerRecord {
                    name: source_name.clone(),
                    addr: from,
                    authorized: true,
                });
            drop(state);
            log::info!("Network manager authorized peer {} ({})", source_name, from);
            if self.app_handle.emit("net_peers_updated", ()).is_err() {
                log::error!("Unable to emit: net_peers_updated");
            }
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
        if let Ok(payload) = serde_json::to_vec(&response) {
            // Respond to the peer's CLIPBOARD_PORT so their transport loop receives it.
            let response_addr = SocketAddr::new(from.ip(), CLIPBOARD_PORT);
            if let Err(e) = socket.send_to(&payload, response_addr).await {
                log::warn!(
                    "Network manager failed to send auth response to {}: {}",
                    response_addr,
                    e
                );
            }
        }
    }

    /// Called when we receive an `AuthResponse` for a request we sent earlier.
    async fn handle_auth_response(&self, source_name: String, from: SocketAddr, approved: bool) {
        if approved {
            let mut state = self.state.lock().await;
            state
                .peers
                .entry(from)
                .and_modify(|p| {
                    p.authorized = true;
                    p.name = source_name.clone();
                })
                .or_insert_with(|| PeerRecord {
                    name: source_name.clone(),
                    addr: from,
                    authorized: true,
                });
            drop(state);
            log::info!(
                "Network manager: peer {} ({}) approved our auth request",
                source_name,
                from
            );
            if self.app_handle.emit("net_peers_updated", ()).is_err() {
                log::error!("Unable to emit: net_peers_updated");
            }
        } else {
            log::warn!(
                "Network manager: peer {} ({}) rejected our auth request",
                source_name,
                from
            );
        }
    }

    fn resolve_local_name() -> String {
        env::var("CLIPPER_DEVICE_NAME")
            .or_else(|_| env::var("HOSTNAME"))
            .or_else(|_| env::var("COMPUTERNAME"))
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| "Clipper".to_string())
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

// ─── Tauri Commands ───────────────────────────────────────────────────────────

/// Returns the current network manager status including the local OTP.
#[tauri::command]
pub async fn net_get_status(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
) -> AppResult<NetStatus> {
    with_error_event(&app_handle, async { Ok(state.get_status().await) }).await
}

/// Returns all discovered peers with their authorization state.
#[tauri::command]
pub async fn net_list_peers(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
) -> AppResult<Vec<NetworkPeerEntry>> {
    with_error_event(&app_handle, async { Ok(state.list_peers().await) }).await
}

/// Generates a fresh 6-digit OTP and stores it as the active one-time code.
/// Any previously active OTP is discarded.  The frontend should call this
/// command before showing the pairing screen, then display the returned code
/// to the user so a remote device can enter it via `net_authorize_peer`.
/// The OTP is automatically consumed (invalidated) after the first inbound
/// auth attempt so it cannot be reused.
#[tauri::command]
pub async fn net_generate_otp(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
) -> AppResult<String> {
    with_error_event(&app_handle, async { Ok(state.refresh_otp().await) }).await
}

/// Sends an authorization request to the peer identified by `peer_id`
/// (an `ip:port` string).  `otp` is the 6-digit code shown on *that peer's* UI.
#[tauri::command]
pub async fn net_authorize_peer(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
    peer_id: String,
    otp: String,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        state.request_auth(&peer_id, &otp).await
    })
    .await
}

/// Removes authorization from a peer, stopping clipboard sharing with it.
#[tauri::command]
pub async fn net_revoke_peer(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
    peer_id: String,
) -> AppResult<()> {
    with_error_event(&app_handle, async { state.revoke_peer(&peer_id).await }).await
}

/// Starts network discovery and clipboard transport workers.
#[tauri::command]
pub async fn net_start(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        let manager = Arc::clone(&*state);
        manager.start().await;
        Ok(())
    })
    .await
}

/// Stops all network workers and clears the peer list.
#[tauri::command]
pub async fn net_stop(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<NetworkManager>>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        state.stop().await;
        Ok(())
    })
    .await
}
