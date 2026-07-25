use std::sync::Arc;
use tracing::{error, info};

use nexterm_pty::{PtyConfig, PtySession};
use nexterm_pty::pty::TermSize;

use crate::config::Config;
use crate::event::{AppEvent, EventBus};
use crate::{CoreError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Initializing,
    Running,
    ShuttingDown,
    Stopped,
}

pub struct Tab {
    pub id:      usize,
    pub title:   String,
    pub session: PtySession,
}

pub struct App {
    pub state:      AppState,
    pub config:     Config,
    pub event_bus:  Arc<EventBus>,
    pub tabs:       Vec<Tab>,
    pub active_tab: usize,
    next_tab_id:    usize,
}

impl App {
    pub fn new(config: Config) -> Self {
        info!("Initializing NexTerm v{}", env!("CARGO_PKG_VERSION"));

        Self {
            state:       AppState::Initializing,
            config,
            event_bus:   Arc::new(EventBus::default()),
            tabs:        Vec::new(),
            active_tab:  0,
            next_tab_id: 0,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        info!("Starting NexTerm");
        self.new_tab()?;
        self.state = AppState::Running;
        info!("NexTerm running");
        Ok(())
    }

    pub fn new_tab(&mut self) -> Result<usize> {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        info!("Creating tab {}", tab_id);

        let pty_config = PtyConfig {
            shell: None,
            size:  TermSize::new(
                self.config.terminal.rows,
                self.config.terminal.cols,
            ),
            scrollback: self.config.general.scrollback_lines,
        };

        let session = PtySession::new(pty_config)?;

        let tab = Tab {
            id:      tab_id,
            title:   format!("Terminal {}", tab_id + 1),
            session,
        };

        self.tabs.push(tab);
        self.active_tab = tab_id;
        self.event_bus.send(AppEvent::TabCreated { tab_id });

        Ok(tab_id)
    }

    pub fn close_tab(&mut self, tab_id: usize) -> Result<()> {
        info!("Closing tab {}", tab_id);

        let pos = self.tabs
            .iter()
            .position(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::App(
                format!("Tab {} not found", tab_id)
            ))?;

        self.tabs.remove(pos);

        if self.tabs.is_empty() {
            info!("No tabs left — shutting down");
            self.shutdown();
            return Ok(());
        }

        self.active_tab = self.tabs
            .last()
            .map(|t| t.id)
            .unwrap_or(0);

        self.event_bus.send(AppEvent::TabClosed { tab_id });
        Ok(())
    }

    pub fn switch_tab(&mut self, tab_id: usize) -> Result<()> {
        let exists = self.tabs.iter().any(|t| t.id == tab_id);

        if !exists {
            return Err(CoreError::App(
                format!("Tab {} not found", tab_id)
            ));
        }

        self.active_tab = tab_id;
        self.event_bus.send(AppEvent::TabSwitched { tab_id });
        Ok(())
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == self.active_tab)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        let active = self.active_tab;
        self.tabs.iter_mut().find(|t| t.id == active)
    }

    pub fn on_resize(&mut self, width: u32, height: u32) {
        let cols = (width  / 8)  as u16;
        let rows = (height / 16) as u16;

        for tab in &mut self.tabs {
            if let Err(e) = tab.session.resize_blocking(rows, cols) {
                error!("Resize failed for tab {}: {}", tab.id, e);
            }
        }

        self.event_bus.send(AppEvent::WindowResized { width, height });
    }

    pub fn shutdown(&mut self) {
        info!("NexTerm shutting down");
        self.state = AppState::ShuttingDown;
        self.event_bus.send(AppEvent::Quit);
        self.tabs.clear();
        self.state = AppState::Stopped;
    }

    pub fn is_running(&self) -> bool {
        self.state == AppState::Running
    }
}