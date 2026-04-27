use crate::content_managers::message_bus::{AppMessage, MessageBus, NetworkClipboardPayload};
use chrono::Utc;
use std::collections::HashMap;
use std::env;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket as StdUdpSocket};
use std::sync::Arc;
use std::time::Duration;
use tauri::async_runtime;
use tokio::net::UdpSocket;
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;

const DISCOVERY_MULTICAST_HOST: Ipv4Addr = Ipv4Addr::new(239, 255, 42, 99);
const DISCOVERY_PORT: u16 = 34254;
const CLIPBOARD_PORT: u16 = 34255;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct DiscoveryPacket {
    name: String,
    clipboard_port: u16,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct ClipboardWirePacket {
    source_name: String,
    text: String,
    timestamp: String,
}

#[derive(Clone, Debug)]
struct NetworkPeer {
    name: String,
    addr: SocketAddr,
}

#[derive(Clone)]
struct ConnectedPeer {
    name: String,
    addr: SocketAddr,
    socket: Arc<UdpSocket>,
}

struct NetworkManagerState {
    running: bool,
    local_name: String,
    peers: HashMap<SocketAddr, NetworkPeer>,
    connected_peers: HashMap<SocketAddr, ConnectedPeer>,
    shutdown_tx: Option<watch::Sender<bool>>,
    tasks: Vec<JoinHandle<()>>,
}

pub struct NetworkManager {
    bus: MessageBus,
    state: Mutex<NetworkManagerState>,
}

impl NetworkManager {
    pub async fn new(bus: MessageBus) -> Arc<Self> {
        let manager = Arc::new(Self {
            bus,
            state: Mutex::new(NetworkManagerState {
                running: false,
                local_name: Self::local_name(),
                peers: HashMap::new(),
                connected_peers: HashMap::new(),
                shutdown_tx: None,
                tasks: Vec::new(),
            }),
        });

        manager.start().await;
        manager
    }

    pub async fn start(self: &Arc<Self>) {
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

    pub async fn stop(&self) {
        let (shutdown_tx, tasks) = {
            let mut state = self.state.lock().await;
            if !state.running {
                return;
            }

            state.running = false;
            state.peers.clear();
            state.connected_peers.clear();
            (state.shutdown_tx.take(), std::mem::take(&mut state.tasks))
        };

        if let Some(shutdown_tx) = shutdown_tx {
            let _ = shutdown_tx.send(true);
        }

        for task in tasks {
            task.abort();
        }
    }

    fn local_name() -> String {
        env::var("CLIPPER_DEVICE_NAME")
            .or_else(|_| env::var("HOSTNAME"))
            .or_else(|_| env::var("COMPUTERNAME"))
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "Clipper".to_string())
    }

    async fn run_presence_announcer(local_name: String, mut shutdown_rx: watch::Receiver<bool>) {
        let socket = match Self::create_multicast_sender() {
            Ok(socket) => socket,
            Err(error) => {
                log::error!(
                    "Network manager failed to create discovery sender: {}",
                    error
                );
                return;
            }
        };

        let mut interval = tokio::time::interval(Duration::from_secs(10));
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
                            if let Err(error) = socket.send_to(&payload, target).await {
                                log::warn!("Network manager failed to announce presence: {}", error);
                            }
                        }
                        Err(error) => {
                            log::warn!("Network manager failed to serialize discovery packet: {}", error);
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
            Ok(socket) => socket,
            Err(error) => {
                log::error!("Network manager failed to bind discovery socket: {}", error);
                return;
            }
        };

        let mut buffer = [0_u8; 2048];

        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => break,
                result = socket.recv_from(&mut buffer) => {
                    match result {
                        Ok((length, address)) => {
                            match serde_json::from_slice::<DiscoveryPacket>(&buffer[..length]) {
                                Ok(packet) if packet.name != local_name => {
                                    let peer = NetworkPeer {
                                        name: packet.name,
                                        addr: SocketAddr::new(address.ip(), packet.clipboard_port),
                                    };
                                    manager.upsert_peer(peer.clone()).await;

                                    if let Err(error) = manager.connect_peer(peer.clone()).await {
                                        log::debug!(
                                            "Network manager failed to prepare peer transport to {} ({}): {}",
                                            peer.name,
                                            peer.addr,
                                            error
                                        );
                                    }
                                }
                                Ok(_) => {}
                                Err(error) => {
                                    log::debug!("Network manager ignored invalid discovery packet: {}", error);
                                }
                            }
                        }
                        Err(error) => {
                            log::warn!("Network manager failed to receive discovery packet: {}", error);
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
            Ok(socket) => socket,
            Err(error) => {
                log::error!("Network manager failed to bind clipboard socket: {}", error);
                return;
            }
        };

        let mut receiver = bus.subscribe();
        let mut buffer = [0_u8; 65_535];

        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => break,
                message = receiver.recv() => {
                    match message {
                        Ok(AppMessage::AddedToClipboard(text)) => {
                            let packet = ClipboardWirePacket {
                                source_name: local_name.clone(),
                                text,
                                timestamp: Utc::now().to_rfc3339(),
                            };

                            match serde_json::to_vec(&packet) {
                                Ok(payload) => {
                                    for peer in manager.peer_connections().await {
                                        if let Err(error) = peer.socket.send(&payload).await {
                                            log::debug!(
                                                "Network manager failed to send clipboard payload to {} ({}): {}",
                                                peer.name,
                                                peer.addr,
                                                error
                                            );
                                        }
                                    }
                                }
                                Err(error) => {
                                    log::warn!("Network manager failed to serialize clipboard payload: {}", error);
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            log::warn!("Network manager lagged and skipped {} clipboard messages", skipped);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
                result = socket.recv_from(&mut buffer) => {
                    match result {
                        Ok((length, _)) => {
                            match serde_json::from_slice::<ClipboardWirePacket>(&buffer[..length]) {
                                Ok(packet) if packet.source_name != local_name => {
                                    let payload = NetworkClipboardPayload {
                                        source_name: packet.source_name,
                                        text: packet.text,
                                    };
                                    if bus.send(AppMessage::NetworkClipboardReceived(payload)).is_err() {
                                        log::error!("Unable to send message: NetworkClipboardReceived");
                                    }
                                }
                                Ok(_) => {}
                                Err(error) => {
                                    log::debug!("Network manager ignored invalid clipboard payload: {}", error);
                                }
                            }
                        }
                        Err(error) => {
                            log::warn!("Network manager failed to receive clipboard payload: {}", error);
                        }
                    }
                }
            }
        }
    }

    async fn upsert_peer(&self, peer: NetworkPeer) {
        let mut state = self.state.lock().await;
        let is_new_peer = !state.peers.contains_key(&peer.addr);
        let peer_name = peer.name.clone();
        let peer_addr = peer.addr;
        state.peers.insert(peer.addr, peer);

        if is_new_peer {
            log::info!(
                "Network manager discovered peer {} at {}",
                peer_name,
                peer_addr
            );
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

    async fn connect_peer(&self, peer: NetworkPeer) -> std::io::Result<()> {
        {
            let state = self.state.lock().await;
            if state.connected_peers.contains_key(&peer.addr) {
                return Ok(());
            }
        }

        let socket = Arc::new(UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?);
        socket.connect(peer.addr).await?;

        let mut state = self.state.lock().await;
        if state.connected_peers.contains_key(&peer.addr) {
            return Ok(());
        }

        state.connected_peers.insert(
            peer.addr,
            ConnectedPeer {
                name: peer.name.clone(),
                addr: peer.addr,
                socket,
            },
        );

        log::info!(
            "Network manager prepared clipboard transport to {} at {}",
            peer.name,
            peer.addr
        );

        Ok(())
    }

    async fn peer_connections(&self) -> Vec<ConnectedPeer> {
        self.state
            .lock()
            .await
            .connected_peers
            .values()
            .cloned()
            .collect()
    }
}
