//! PTY abstraction — portable-pty wrapper
//! Ye trait-based design hai taake future mein
//! alag backends add kar sakein bina core todey

use portable_pty::{CommandBuilder, NativePtySystem, PtyPair, PtySize, PtySystem};
use tracing::{debug, error, info};
use crate::{PtyError, Result};
use crate::shell::Shell;

/// Terminal size — rows aur columns
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TermSize {
    pub rows: u16,
    pub cols: u16,
    pub pixel_width:  u16,
    pub pixel_height: u16,
}

impl TermSize {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            rows,
            cols,
            pixel_width:  0,
            pixel_height: 0,
        }
    }
}

impl Default for TermSize {
    fn default() -> Self {
        Self::new(24, 80)
    }
}

impl From<TermSize> for PtySize {
    fn from(s: TermSize) -> Self {
        PtySize {
            rows:         s.rows,
            cols:         s.cols,
            pixel_width:  s.pixel_width,
            pixel_height: s.pixel_height,
        }
    }
}

/// Raw PTY pair — reader + writer + child process
pub struct PtyHandle {
    pub pair:   PtyPair,
    pub reader: Box<dyn std::io::Read  + Send>,
    pub writer: Box<dyn std::io::Write + Send>,
}

impl PtyHandle {
    /// Yahan se PTY open hota hai aur shell spawn hota hai
    pub fn open(shell: &Shell, size: TermSize) -> Result<Self> {
        info!(
            "Opening PTY: shell={} size={}x{}",
            shell.name(), size.cols, size.rows
        );

        // Native PTY system — Linux=POSIX, Windows=ConPTY
        let pty_system = NativePtySystem::default();

        let pair = pty_system
            .openpty(size.into())
            .map_err(|e| PtyError::OpenFailed(e.to_string()))?;

        // Shell command build karo
        let mut cmd = CommandBuilder::new(&shell.path);
        for arg in &shell.args {
            cmd.arg(arg);
        }

        // Environment variables
        cmd.env("TERM",         "xterm-256color");
        cmd.env("COLORTERM",    "truecolor");
        cmd.env("NEXTERM",      "1");
        cmd.env("NEXTERM_VERSION", env!("CARGO_PKG_VERSION"));

        // Shell spawn karo
        pair.slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        debug!("Shell spawned successfully");

        // Reader + writer banao
        let reader = pair.master
            .try_clone_reader()
            .map_err(|e| PtyError::ReadFailed(e.to_string()))?;

        let writer = pair.master
            .take_writer()
            .map_err(|e| PtyError::WriteFailed(e.to_string()))?;

        Ok(Self { pair, reader, writer })
    }

    /// PTY ko resize karo — window resize pe call hoga
    pub fn resize(&self, size: TermSize) -> Result<()> {
        debug!("Resizing PTY to {}x{}", size.cols, size.rows);

        self.pair.master
            .resize(size.into())
            .map_err(|e| PtyError::ResizeFailed(e.to_string()))
    }

    /// Data PTY mein likhna — user ka input
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        use std::io::Write;

        self.writer
            .write_all(data)
            .map_err(|e| PtyError::WriteFailed(e.to_string()))?;

        self.writer
            .flush()
            .map_err(|e| PtyError::WriteFailed(e.to_string()))
    }

    /// PTY se data padhna — shell ka output
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        use std::io::Read;

        self.reader
            .read(buf)
            .map_err(|e| {
                error!("PTY read error: {}", e);
                PtyError::ReadFailed(e.to_string())
            })
    }
}