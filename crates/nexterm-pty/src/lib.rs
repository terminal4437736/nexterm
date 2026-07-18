//! nexterm-pty — cross-platform PTY management
//!
//! Linux/macOS → POSIX PTY via portable-pty
//! Windows     → ConPTY via portable-pty
//!
//! Usage:
//!   let pty = PtySession::new(PtyConfig::default())?;
//!   pty.write(b"ls\n")?;
//!   let output = pty.read().await?;

pub mod pty;
pub mod session;
pub mod shell;

pub use session::{PtySession, PtyConfig};
pub use shell::detect_shell;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PtyError {
    #[error("Failed to open PTY: {0}")]
    OpenFailed(String),

    #[error("Failed to spawn shell: {0}")]
    SpawnFailed(String),

    #[error("Failed to read from PTY: {0}")]
    ReadFailed(String),

    #[error("Failed to write to PTY: {0}")]
    WriteFailed(String),

    #[error("Failed to resize PTY: {0}")]
    ResizeFailed(String),

    #[error("Shell not found: {0}")]
    ShellNotFound(String),

    #[error("PTY session closed")]
    SessionClosed,
}

pub type Result<T> = std::result::Result<T, PtyError>;