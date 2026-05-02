use super::db::DbConnection;
use crate::content_managers::message_bus::{AppMessage, MessageBus, NetworkClipboardPayload};
use crate::error::{with_error_event, AppError, AppResult};
use chrono::Utc;
use futures::StreamExt;
use libp2p::{
    identity, mdns, noise, request_response, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp,
    yamux, Multiaddr, PeerId, StreamProtocol, Swarm, SwarmBuilder,
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::{hash_map::Entry, HashMap};
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tauri::{async_runtime, async_runtime::JoinHandle, AppHandle, Emitter, State};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

const CLIPPER_PROTOCOL: &str = "/clipper/clipboard/1";
const NETWORK_COMMAND_BUFFER: usize = 100;
const MDNS_QUERY_INTERVAL_SECS: u64 = 10;
const MDNS_RECORD_TTL_SECS: u64 = 60;
const NETWORK_REQUEST_TIMEOUT_SECS: u64 = 10;

#[derive(Clone, Debug)]
struct PeerRecord {
    name: String,
    authorized: bool,
}

struct NetworkManagerState {
    running: bool,
    local_name: String,
    local_otp: Option<String>,
    local_peer_id: Option<String>,
    trusted_peers: HashMap<PeerId, String>,
    peers: HashMap<PeerId, PeerRecord>,
    command_tx: Option<mpsc::Sender<NetworkCommand>>,
    task: Option<JoinHandle<()>>,
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

#[derive(Debug)]
enum NetworkCommand {
    RequestAuth { peer_id: PeerId, otp: String },
    Revoke { peer_id: PeerId },
    Shutdown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum NetworkRequest {
    Hello {
        source_name: String,
    },
    AuthRequest {
        source_name: String,
        otp: String,
    },
    Clipboard {
        source_name: String,
        text: String,
        timestamp: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum NetworkResponse {
    HelloAck { source_name: String },
    AuthResponse { source_name: String, approved: bool },
    Ack,
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ClipperBehaviourEvent")]
struct ClipperBehaviour {
    mdns: mdns::tokio::Behaviour,
    request_response: request_response::json::Behaviour<NetworkRequest, NetworkResponse>,
}

#[derive(Debug)]
enum ClipperBehaviourEvent {
    Mdns(mdns::Event),
    RequestResponse(request_response::Event<NetworkRequest, NetworkResponse>),
}

impl From<mdns::Event> for ClipperBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        Self::Mdns(event)
    }
}

impl From<request_response::Event<NetworkRequest, NetworkResponse>> for ClipperBehaviourEvent {
    fn from(event: request_response::Event<NetworkRequest, NetworkResponse>) -> Self {
        Self::RequestResponse(event)
    }
}

impl ClipperBehaviour {
    fn new(local_peer_id: PeerId) -> std::io::Result<Self> {
        let mdns_config = mdns::Config {
            query_interval: Duration::from_secs(MDNS_QUERY_INTERVAL_SECS),
            ttl: Duration::from_secs(MDNS_RECORD_TTL_SECS),
            ..mdns::Config::default()
        };
        let request_response = request_response::json::Behaviour::new(
            [(
                StreamProtocol::new(CLIPPER_PROTOCOL),
                request_response::ProtocolSupport::Full,
            )],
            request_response::Config::default()
                .with_request_timeout(Duration::from_secs(NETWORK_REQUEST_TIMEOUT_SECS)),
        );

        Ok(Self {
            mdns: mdns::tokio::Behaviour::new(mdns_config, local_peer_id)?,
            request_response,
        })
    }
}

pub struct NetworkManager {
    app_handle: AppHandle,
    bus: MessageBus,
    pool: SqlitePool,
    state: Mutex<NetworkManagerState>,
}

impl NetworkManager {
    pub async fn new(
        db: Arc<DbConnection>,
        bus: MessageBus,
        app_handle: AppHandle,
    ) -> AppResult<Arc<Self>> {
        let manager = Arc::new(Self {
            app_handle,
            bus,
            pool: db.pool.clone(),
            state: Mutex::new(NetworkManagerState {
                running: false,
                local_name: Self::resolve_local_name(),
                local_otp: None,
                local_peer_id: None,
                trusted_peers: HashMap::new(),
                peers: HashMap::new(),
                command_tx: None,
                task: None,
            }),
        });

        manager.start().await?;
        Ok(manager)
    }

    pub async fn start(self: &Arc<Self>) -> AppResult<()> {
        let local_name = {
            let state = self.state.lock().await;
            if state.running {
                return Ok(());
            }
            state.local_name.clone()
        };

        let keypair = self.load_or_create_identity().await?;
        let local_peer_id = keypair.public().to_peer_id();
        let trusted_peers = self.load_trusted_peers().await?;
        let mut swarm = Self::create_swarm(keypair)?;
        swarm
            .listen_on(
                Multiaddr::from_str("/ip4/0.0.0.0/tcp/0")
                    .map_err(|e| AppError::NetworkError(e.to_string()))?,
            )
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        let (command_tx, command_rx) = mpsc::channel(NETWORK_COMMAND_BUFFER);

        {
            let mut state = self.state.lock().await;
            if state.running {
                return Ok(());
            }

            state.running = true;
            state.local_peer_id = Some(local_peer_id.to_string());
            state.trusted_peers = trusted_peers;
            state.peers.clear();
            state.command_tx = Some(command_tx.clone());
        }

        let manager = Arc::clone(self);
        let bus_receiver = self.bus.subscribe();
        let task = async_runtime::spawn(async move {
            Self::run_swarm(manager, swarm, local_name, command_rx, bus_receiver).await;
        });

        {
            let mut state = self.state.lock().await;
            if state.running {
                state.task = Some(task);
            } else {
                task.abort();
            }
        }

        log::info!("Network manager started with local peer {}", local_peer_id);
        if self.app_handle.emit("net_status_changed", true).is_err() {
            log::error!("Unable to emit: net_status_changed");
        }
        self.notify_peers_updated();
        Ok(())
    }

    pub async fn stop(&self) {
        let (command_tx, task) = {
            let mut state = self.state.lock().await;
            if !state.running {
                return;
            }
            state.running = false;
            state.peers.clear();
            state.command_tx.take().zip(state.task.take())
        }
        .unzip();

        if let Some(tx) = command_tx {
            let _ = tx.send(NetworkCommand::Shutdown).await;
        }
        if let Some(task) = task {
            task.abort();
        }

        log::info!("Network manager stopped");
        if self.app_handle.emit("net_status_changed", false).is_err() {
            log::error!("Unable to emit: net_status_changed");
        }
        self.notify_peers_updated();
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
        let peers = self
            .state
            .lock()
            .await
            .peers
            .iter()
            .map(|(peer_id, peer)| NetworkPeerEntry {
                id: peer_id.to_string(),
                name: peer.name.clone(),
                authorized: peer.authorized,
            })
            .collect::<Vec<_>>();

        log::info!("Network manager listing {} peers", peers.len());
        peers
    }

    pub async fn request_auth(&self, peer_id: &str, otp: &str) -> AppResult<()> {
        let peer_id = Self::parse_peer_id(peer_id)?;
        let command_tx = {
            let state = self.state.lock().await;
            if !state.peers.contains_key(&peer_id) {
                return Err(AppError::validation(format!("Peer not found: {peer_id}")));
            }
            state
                .command_tx
                .clone()
                .ok_or_else(|| AppError::runtime("Network manager is not running"))?
        };

        command_tx
            .send(NetworkCommand::RequestAuth {
                peer_id,
                otp: otp.to_string(),
            })
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        log::info!("Network manager queued auth request to {peer_id}");
        Ok(())
    }

    pub async fn revoke_peer(&self, peer_id: &str) -> AppResult<()> {
        let peer_id = Self::parse_peer_id(peer_id)?;
        let command_tx = {
            let mut state = self.state.lock().await;
            state.trusted_peers.remove(&peer_id);
            if let Some(peer) = state.peers.get_mut(&peer_id) {
                peer.authorized = false;
            }
            state.command_tx.clone()
        };

        self.delete_trusted_peer(peer_id).await?;

        if let Some(tx) = command_tx {
            if let Err(e) = tx.send(NetworkCommand::Revoke { peer_id }).await {
                log::debug!("Network manager failed to queue peer revoke: {}", e);
            }
        }

        log::info!("Network manager revoked peer {}", peer_id);
        self.notify_peers_updated();
        Ok(())
    }

    async fn run_swarm(
        manager: Arc<Self>,
        mut swarm: Swarm<ClipperBehaviour>,
        local_name: String,
        mut command_rx: mpsc::Receiver<NetworkCommand>,
        mut bus_rx: tokio::sync::broadcast::Receiver<AppMessage>,
    ) {
        loop {
            tokio::select! {
                command = command_rx.recv() => {
                    match command {
                        Some(NetworkCommand::RequestAuth { peer_id, otp }) => {
                            swarm.behaviour_mut().request_response.send_request(
                                &peer_id,
                                NetworkRequest::AuthRequest {
                                    source_name: local_name.clone(),
                                    otp,
                                },
                            );
                        }
                        Some(NetworkCommand::Revoke { peer_id }) => {
                            manager.mark_peer_unauthorized(peer_id).await;
                        }
                        Some(NetworkCommand::Shutdown) | None => break,
                    }
                }
                message = bus_rx.recv() => {
                    match message {
                        Ok(AppMessage::AddedToClipboard(text)) => {
                            let peers = manager.authorized_discovered_peers().await;
                            for peer_id in peers {
                                swarm.behaviour_mut().request_response.send_request(
                                    &peer_id,
                                    NetworkRequest::Clipboard {
                                        source_name: local_name.clone(),
                                        text: text.clone(),
                                        timestamp: Utc::now().to_rfc3339(),
                                    },
                                );
                            }
                        }
                        Ok(_) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            log::warn!("Network manager lagged and skipped {} messages", n);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
                event = swarm.select_next_some() => {
                    Self::handle_swarm_event(&manager, &mut swarm, &local_name, event).await;
                }
            }
        }

        log::info!("Network manager swarm task stopped");
    }

    async fn handle_swarm_event(
        manager: &Arc<Self>,
        swarm: &mut Swarm<ClipperBehaviour>,
        local_name: &str,
        event: SwarmEvent<ClipperBehaviourEvent>,
    ) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                log::info!("Network manager listening on {}", address);
            }
            SwarmEvent::Behaviour(ClipperBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                log::info!("Network manager mDNS discovered {} addresses", peers.len());

                for (peer_id, address) in peers {
                    if Some(peer_id.to_string()) == manager.local_peer_id().await {
                        continue;
                    }

                    swarm.add_peer_address(peer_id, address.clone());
                    manager.upsert_discovered_peer(peer_id, None).await;
                    swarm.behaviour_mut().request_response.send_request(
                        &peer_id,
                        NetworkRequest::Hello {
                            source_name: local_name.to_string(),
                        },
                    );
                    log::info!("Network manager discovered peer {} at {}", peer_id, address);
                }
            }
            SwarmEvent::Behaviour(ClipperBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                for (peer_id, address) in peers {
                    log::debug!(
                        "Network manager mDNS address expired: {} at {}",
                        peer_id,
                        address
                    );
                }
            }
            SwarmEvent::Behaviour(ClipperBehaviourEvent::RequestResponse(event)) => {
                Self::handle_request_response_event(manager, swarm, local_name, event).await;
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                log::debug!(
                    "Network manager outgoing connection failed for {:?}: {}",
                    peer_id,
                    error
                );
            }
            SwarmEvent::IncomingConnectionError { error, .. } => {
                log::debug!("Network manager incoming connection failed: {}", error);
            }
            _ => {}
        }
    }

    async fn handle_request_response_event(
        manager: &Arc<Self>,
        swarm: &mut Swarm<ClipperBehaviour>,
        local_name: &str,
        event: request_response::Event<NetworkRequest, NetworkResponse>,
    ) {
        match event {
            request_response::Event::Message { peer, message, .. } => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    let response = manager
                        .handle_network_request(peer, request, local_name)
                        .await;
                    if let Err(response) = swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, response)
                    {
                        log::debug!("Network manager failed to send response: {:?}", response);
                    }
                }
                request_response::Message::Response { response, .. } => {
                    manager.handle_network_response(peer, response).await;
                }
            },
            request_response::Event::OutboundFailure { peer, error, .. } => {
                log::debug!(
                    "Network manager outbound request failed for {}: {}",
                    peer,
                    error
                );
            }
            request_response::Event::InboundFailure { peer, error, .. } => {
                log::debug!(
                    "Network manager inbound request failed for {}: {}",
                    peer,
                    error
                );
            }
            request_response::Event::ResponseSent { peer, .. } => {
                log::debug!("Network manager sent response to {}", peer);
            }
        }
    }

    async fn handle_network_request(
        &self,
        peer_id: PeerId,
        request: NetworkRequest,
        local_name: &str,
    ) -> NetworkResponse {
        match request {
            NetworkRequest::Hello { source_name } => {
                self.upsert_discovered_peer(peer_id, Some(source_name))
                    .await;
                NetworkResponse::HelloAck {
                    source_name: local_name.to_string(),
                }
            }
            NetworkRequest::AuthRequest { source_name, otp } => {
                let approved = self.consume_matching_otp(&otp).await;
                if approved {
                    if let Err(e) = self.persist_trusted_peer(peer_id, &source_name).await {
                        log::warn!(
                            "Network manager failed to persist trusted peer {}: {}",
                            peer_id,
                            e
                        );
                    }
                    self.mark_peer_authorized(peer_id, source_name).await;
                } else {
                    log::warn!(
                        "Network manager rejected auth request from {}: invalid or expired OTP",
                        peer_id
                    );
                }

                NetworkResponse::AuthResponse {
                    source_name: local_name.to_string(),
                    approved,
                }
            }
            NetworkRequest::Clipboard {
                source_name, text, ..
            } => {
                if self.is_trusted(peer_id).await {
                    let payload = NetworkClipboardPayload { source_name, text };
                    if self
                        .bus
                        .send(AppMessage::NetworkClipboardReceived(payload))
                        .is_err()
                    {
                        log::error!("Unable to send message: NetworkClipboardReceived");
                    }
                } else {
                    log::warn!(
                        "Network manager ignored clipboard from untrusted peer {}",
                        peer_id
                    );
                }
                NetworkResponse::Ack
            }
        }
    }

    async fn handle_network_response(&self, peer_id: PeerId, response: NetworkResponse) {
        match response {
            NetworkResponse::HelloAck { source_name } => {
                self.upsert_discovered_peer(peer_id, Some(source_name))
                    .await;
            }
            NetworkResponse::AuthResponse {
                source_name,
                approved,
            } => {
                if approved {
                    if let Err(e) = self.persist_trusted_peer(peer_id, &source_name).await {
                        log::warn!(
                            "Network manager failed to persist approved peer {}: {}",
                            peer_id,
                            e
                        );
                    }
                    self.mark_peer_authorized(peer_id, source_name).await;
                } else {
                    log::warn!(
                        "Network manager: peer {} rejected our auth request",
                        peer_id
                    );
                }
            }
            NetworkResponse::Ack => {}
        }
    }

    async fn upsert_discovered_peer(&self, peer_id: PeerId, name: Option<String>) {
        let changed = {
            let mut state = self.state.lock().await;
            let authorized = state.trusted_peers.contains_key(&peer_id);
            let peer_name = name
                .or_else(|| state.trusted_peers.get(&peer_id).cloned())
                .unwrap_or_else(|| peer_id.to_string());

            match state.peers.entry(peer_id) {
                Entry::Vacant(entry) => {
                    entry.insert(PeerRecord {
                        name: peer_name,
                        authorized,
                    });
                    true
                }
                Entry::Occupied(mut entry) => {
                    let peer = entry.get_mut();
                    let changed = peer.name != peer_name || peer.authorized != authorized;
                    peer.name = peer_name;
                    peer.authorized = authorized;
                    changed
                }
            }
        };

        if changed {
            self.notify_peers_updated();
        }
    }

    async fn mark_peer_authorized(&self, peer_id: PeerId, name: String) {
        {
            let mut state = self.state.lock().await;
            state.trusted_peers.insert(peer_id, name.clone());
            match state.peers.entry(peer_id) {
                Entry::Vacant(entry) => {
                    entry.insert(PeerRecord {
                        name,
                        authorized: true,
                    });
                }
                Entry::Occupied(mut entry) => {
                    let peer = entry.get_mut();
                    peer.name = name;
                    peer.authorized = true;
                }
            }
        }
        self.notify_peers_updated();
    }

    async fn mark_peer_unauthorized(&self, peer_id: PeerId) {
        {
            let mut state = self.state.lock().await;
            state.trusted_peers.remove(&peer_id);
            if let Some(peer) = state.peers.get_mut(&peer_id) {
                peer.authorized = false;
            }
        }
        self.notify_peers_updated();
    }

    async fn authorized_discovered_peers(&self) -> Vec<PeerId> {
        self.state
            .lock()
            .await
            .peers
            .iter()
            .filter_map(|(peer_id, peer)| peer.authorized.then_some(*peer_id))
            .collect()
    }

    async fn is_trusted(&self, peer_id: PeerId) -> bool {
        self.state.lock().await.trusted_peers.contains_key(&peer_id)
    }

    async fn consume_matching_otp(&self, otp: &str) -> bool {
        let mut state = self.state.lock().await;
        if state.local_otp.as_deref() == Some(otp) {
            state.local_otp = None;
            true
        } else {
            false
        }
    }

    async fn local_peer_id(&self) -> Option<String> {
        self.state.lock().await.local_peer_id.clone()
    }

    async fn load_or_create_identity(&self) -> AppResult<identity::Keypair> {
        if let Some(row) = sqlx::query("SELECT keypair FROM network_identity WHERE id = 1")
            .fetch_optional(&self.pool)
            .await?
        {
            let bytes: Vec<u8> = row.try_get("keypair")?;
            return identity::Keypair::from_protobuf_encoding(&bytes)
                .map_err(|e| AppError::runtime(format!("Invalid libp2p identity keypair: {e}")));
        }

        let keypair = identity::Keypair::generate_ed25519();
        let encoded = keypair
            .to_protobuf_encoding()
            .map_err(|e| AppError::runtime(format!("Unable to encode libp2p identity: {e}")))?;
        sqlx::query("INSERT INTO network_identity (id, keypair, created_at) VALUES (1, ?1, ?2)")
            .bind(encoded)
            .bind(Utc::now().to_rfc3339())
            .execute(&self.pool)
            .await?;

        Ok(keypair)
    }

    async fn load_trusted_peers(&self) -> AppResult<HashMap<PeerId, String>> {
        let rows = sqlx::query("SELECT peer_id, name FROM network_trusted_peers")
            .fetch_all(&self.pool)
            .await?;
        let mut peers = HashMap::new();

        for row in rows {
            let peer_id: String = row.try_get("peer_id")?;
            let name: String = row.try_get("name")?;
            match PeerId::from_str(&peer_id) {
                Ok(peer_id) => {
                    peers.insert(peer_id, name);
                }
                Err(e) => {
                    log::warn!(
                        "Network manager ignored invalid persisted peer id {}: {}",
                        peer_id,
                        e
                    );
                }
            }
        }

        Ok(peers)
    }

    async fn persist_trusted_peer(&self, peer_id: PeerId, name: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO network_trusted_peers (peer_id, name, authorized_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(peer_id) DO UPDATE SET
                name = excluded.name,
                authorized_at = excluded.authorized_at
            "#,
        )
        .bind(peer_id.to_string())
        .bind(name)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_trusted_peer(&self, peer_id: PeerId) -> AppResult<()> {
        sqlx::query("DELETE FROM network_trusted_peers WHERE peer_id = ?1")
            .bind(peer_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    fn create_swarm(keypair: identity::Keypair) -> AppResult<Swarm<ClipperBehaviour>> {
        SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| AppError::NetworkError(e.to_string()))?
            .with_behaviour(
                |key| -> Result<ClipperBehaviour, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(ClipperBehaviour::new(key.public().to_peer_id())?)
                },
            )
            .map_err(|e| AppError::NetworkError(e.to_string()))
            .map(|builder| builder.build())
    }

    fn notify_peers_updated(&self) {
        if self.app_handle.emit("net_peers_updated", ()).is_err() {
            log::error!("Unable to emit: net_peers_updated");
        }
    }

    fn parse_peer_id(peer_id: &str) -> AppResult<PeerId> {
        PeerId::from_str(peer_id)
            .map_err(|_| AppError::validation(format!("Invalid peer id: {peer_id}")))
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
        manager.start().await
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
