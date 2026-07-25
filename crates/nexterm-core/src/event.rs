use tokio::sync::broadcast;
use tracing::debug;

#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    PtyOutput {
        session_id: usize,
        data:       Vec<u8>,
    },
    PtyInput {
        session_id: usize,
        data:       Vec<u8>,
    },
    PtyExited {
        session_id: usize,
    },
    WindowResized {
        width:  u32,
        height: u32,
    },
    WindowFocused,
    WindowUnfocused,
    Quit,
    TabCreated {
        tab_id: usize,
    },
    TabClosed {
        tab_id: usize,
    },
    TabSwitched {
        tab_id: usize,
    },
    ConfigReloaded,
    ThemeChanged {
        theme_name: String,
    },
    PluginEvent {
        plugin_id: String,
        payload:   Vec<u8>,
    },
}

pub struct EventBus {
    tx: broadcast::Sender<AppEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn send(&self, event: AppEvent) {
        debug!("Event: {:?}", event);
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.tx.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<AppEvent> {
        self.tx.clone()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}