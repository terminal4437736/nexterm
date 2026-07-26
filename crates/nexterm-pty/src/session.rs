//! PTY Session — complete session management

use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::pty::{PtyHandle, TermSize};
use crate::shell::{detect_shell, Shell};
use crate::{PtyError, Result};

/// PTY session configuration
#[derive(Debug, Clone)]
pub struct PtyConfig {
    pub shell:      Option<Shell>,
    pub size:       TermSize,
    pub scrollback: usize,
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            shell:      None,
            size:       TermSize::new(24, 80),
            scrollback: 10_000,
        }
    }
}

/// Shell output events
#[derive(Debug, Clone)]
pub enum SessionEvent {
    Data(Vec<u8>),
    Exited,
    Error(String),
}

/// Main PTY session
pub struct PtySession {
    handle: Arc<Mutex<PtyHandle>>,
    config: PtyConfig,
    pub rx: mpsc::Receiver<SessionEvent>,
    tx:     mpsc::Sender<SessionEvent>,
}

impl PtySession {
    pub fn new(config: PtyConfig) -> Result<Self> {
        let shell = match &config.shell {
            Some(s) => s.clone(),
            None    => detect_shell()?,
        };

        info!(
            "Starting PTY session: shell={} size={}x{}",
            shell.name(),
            config.size.cols,
            config.size.rows
        );

        let handle = PtyHandle::open(&shell, config.size)?;
        let handle = Arc::new(Mutex::new(handle));

        let (tx, rx) = mpsc::channel::<SessionEvent>(256);

        let session = Self {
            handle,
            config,
            rx,
            tx,
        };

        session.start_reader();
        Ok(session)
    }

    /// Async write — user input PTY ko bhejo
    pub async fn write(&self, data: &[u8]) -> Result<()> {
        let mut handle = self.handle.lock().await;
        handle.write(data)
    }

    /// Blocking resize — non-async context ke liye
    pub fn resize_blocking(&self, rows: u16, cols: u16) -> Result<()> {
        let size = TermSize::new(rows, cols);
        for _ in 0..10 {
            match self.handle.try_lock() {
                Ok(h)  => return h.resize(size),
                Err(_) => std::thread::sleep(
                    std::time::Duration::from_millis(5)
                ),
            }
        }
        Ok(())
    }

    /// Async resize
    pub async fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        let size   = TermSize::new(rows, cols);
        let handle = self.handle.lock().await;
        handle.resize(size)
    }

    pub async fn send_enter(&self) -> Result<()> {
        self.write(b"\r").await
    }

    pub async fn send_interrupt(&self) -> Result<()> {
        self.write(&[0x03]).await
    }

    pub async fn send_eof(&self) -> Result<()> {
        self.write(&[0x04]).await
    }

    pub fn size(&self) -> TermSize {
        self.config.size
    }

    // ── private ──────────────────────────────────────────

    fn start_reader(&self) {
        let handle = Arc::clone(&self.handle);
        let tx     = self.tx.clone();

        std::thread::spawn(move || {
            let mut buf = vec![0u8; 4096];

            loop {
                let result = match handle.try_lock() {
                    Ok(mut h) => h.read(&mut buf),
                    Err(_)    => {
                        std::thread::sleep(
                            std::time::Duration::from_millis(1)
                        );
                        continue;
                    }
                };

                match result {
                    Ok(0) => {
                        debug!("PTY EOF — shell exited");
                        let _ = tx.blocking_send(SessionEvent::Exited);
                        break;
                    }
                    Ok(n) => {
                        let data = buf[..n].to_vec();
                        if tx.blocking_send(
                            SessionEvent::Data(data)
                        ).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("PTY read error: {}", e);
                        let _ = tx.blocking_send(
                            SessionEvent::Error(e.to_string())
                        );
                        break;
                    }
                }
            }
        });
    }
}