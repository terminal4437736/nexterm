pub mod app;
pub mod config;
pub mod event;

pub use app::{App, AppState};
pub use config::Config;
pub use event::{AppEvent, EventBus};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("PTY error: {0}")]
    Pty(#[from] nexterm_pty::PtyError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Event bus error: {0}")]
    EventBus(String),

    #[error("App error: {0}")]
    App(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;