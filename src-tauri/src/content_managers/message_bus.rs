use tokio::sync::broadcast;

#[derive(Clone)]
pub enum AppMessage {
    AddedToClipboard(String),
}

#[derive(Clone)]
pub struct MessageBus {
    sender: broadcast::Sender<AppMessage>,
}

impl MessageBus {
    /// capacity: ring buffer size (older messages are dropped if lagging).
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { sender: tx }
    }

    /// Subscribe to messages (each subscriber gets all future sends).
    pub fn subscribe(&self) -> broadcast::Receiver<AppMessage> {
        self.sender.subscribe()
    }

    /// Send a message. Returns number of active receivers on success.
    pub fn send(&self, msg: AppMessage) -> Result<usize, broadcast::error::SendError<AppMessage>> {
        self.sender.send(msg)
    }
}
