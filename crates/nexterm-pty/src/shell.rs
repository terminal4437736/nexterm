//! Shell detection — system ka default shell dhundta hai
//! Linux/macOS → $SHELL env variable
//! Windows     → PowerShell ya cmd.exe

use std::path::PathBuf;
use tracing::info;
use crate::PtyError;

/// Supported shell types
#[derive(Debug, Clone, PartialEq)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
    Unknown(String),
}

/// Detected shell info
#[derive(Debug, Clone)]
pub struct Shell {
    pub path: PathBuf,
    pub kind: ShellKind,
    pub args: Vec<String>,
}

impl Shell {
    pub fn new(path: PathBuf) -> Self {
        let kind = detect_kind(&path);
        let args  = default_args(&kind);
        Self { path, kind, args }
    }

    pub fn name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }
}

/// Auto-detect system shell
pub fn detect_shell() -> Result<Shell, PtyError> {
    let path = find_shell_path()?;
    let shell = Shell::new(path.clone());
    info!("Detected shell: {} ({:?})", shell.name(), shell.kind);
    Ok(shell)
}

// ── private helpers ───────────────────────────────────────

fn find_shell_path() -> Result<PathBuf, PtyError> {
    // 1. $SHELL env var (Linux/macOS)
    #[cfg(unix)]
    if let Ok(shell) = std::env::var("SHELL") {
        let path = PathBuf::from(&shell);
        if path.exists() {
            return Ok(path);
        }
    }

    // 2. Windows — PowerShell pehle, phir cmd
    #[cfg(windows)]
    {
        let candidates = [
            r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
            r"C:\Program Files\PowerShell\7\pwsh.exe",
            r"C:\Windows\System32\cmd.exe",
        ];
        for candidate in &candidates {
            let path = PathBuf::from(candidate);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    // 3. Common fallbacks
    let fallbacks = [
        "/bin/bash",
        "/bin/zsh",
        "/bin/sh",
        "/usr/bin/bash",
        "/usr/bin/zsh",
    ];

    for fb in &fallbacks {
        let path = PathBuf::from(fb);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(PtyError::ShellNotFound(
        "No shell found. Install bash or set $SHELL".into(),
    ))
}

fn detect_kind(path: &PathBuf) -> ShellKind {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    match name.as_str() {
        "bash"                       => ShellKind::Bash,
        "zsh"                        => ShellKind::Zsh,
        "fish"                       => ShellKind::Fish,
        "powershell.exe" | "pwsh.exe" => ShellKind::PowerShell,
        "cmd.exe"                    => ShellKind::Cmd,
        other                        => ShellKind::Unknown(other.to_string()),
    }
}

fn default_args(kind: &ShellKind) -> Vec<String> {
    match kind {
        ShellKind::Bash | ShellKind::Zsh => vec![
            "--login".into(),
            "-i".into(),
        ],
        ShellKind::Fish => vec![
            "--login".into(),
            "--interactive".into(),
        ],
        ShellKind::PowerShell => vec![
            "-NoLogo".into(),
            "-NoExit".into(),
        ],
        ShellKind::Cmd => vec![],
        ShellKind::Unknown(_) => vec![],
    }
}