use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::{CoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general:  GeneralConfig,
    pub terminal: TerminalConfig,
    pub renderer: RendererConfig,
    pub theme:    ThemeConfig,
    pub keybinds: KeybindsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub title:            String,
    pub shell:            Option<String>,
    pub scrollback_lines: usize,
    pub confirm_close:    bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub rows:           u16,
    pub cols:           u16,
    pub cursor_style:   String,
    pub cursor_blink:   bool,
    pub copy_on_select: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererConfig {
    pub font_family: String,
    pub font_size:   f32,
    pub line_height: f32,
    pub opacity:     f32,
    pub vsync:       bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindsConfig {
    pub new_tab:          String,
    pub close_tab:        String,
    pub next_tab:         String,
    pub prev_tab:         String,
    pub command_palette:  String,
    pub split_horizontal: String,
    pub split_vertical:   String,
    pub copy:             String,
    pub paste:            String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                title:            "NexTerm".into(),
                shell:            None,
                scrollback_lines: 10_000,
                confirm_close:    true,
            },
            terminal: TerminalConfig {
                rows:           24,
                cols:           80,
                cursor_style:   "block".into(),
                cursor_blink:   true,
                copy_on_select: false,
            },
            renderer: RendererConfig {
                font_family: "JetBrains Mono".into(),
                font_size:   14.0,
                line_height: 1.2,
                opacity:     1.0,
                vsync:       true,
            },
            theme: ThemeConfig {
                name: "dark".into(),
            },
            keybinds: KeybindsConfig {
                new_tab:          "Ctrl+T".into(),
                close_tab:        "Ctrl+W".into(),
                next_tab:         "Ctrl+Tab".into(),
                prev_tab:         "Ctrl+Shift+Tab".into(),
                command_palette:  "Ctrl+P".into(),
                split_horizontal: "Ctrl+Shift+H".into(),
                split_vertical:   "Ctrl+Shift+V".into(),
                copy:             "Ctrl+Shift+C".into(),
                paste:            "Ctrl+Shift+V".into(),
            },
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        info!("Loading config from: {}", path.display());

        let content = std::fs::read_to_string(path)
            .map_err(|e| CoreError::Config(
                format!("Cannot read config file: {}", e)
            ))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| CoreError::Config(
                format!("Invalid config TOML: {}", e)
            ))?;

        info!("Config loaded successfully");
        Ok(config)
    }

    pub fn default_path() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA")
                .unwrap_or_else(|_| ".".into());
            PathBuf::from(appdata).join("nexterm").join("config.toml")
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")
                .unwrap_or_else(|_| ".".into());
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("nexterm")
                .join("config.toml")
        }

        #[cfg(target_os = "linux")]
        {
            let config_home = std::env::var("XDG_CONFIG_HOME")
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME")
                        .unwrap_or_else(|_| ".".into());
                    format!("{}/.config", home)
                });
            PathBuf::from(config_home)
                .join("nexterm")
                .join("config.toml")
        }
    }

    pub fn load_or_default() -> Self {
        let path = Self::default_path();
        match Self::load(&path) {
            Ok(config) => config,
            Err(e) => {
                warn!("Config load failed: {} — using defaults", e);
                Self::default()
            }
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CoreError::Io(e))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| CoreError::Config(
                format!("Cannot serialize config: {}", e)
            ))?;

        std::fs::write(path, content)
            .map_err(|e| CoreError::Io(e))?;

        info!("Config saved to: {}", path.display());
        Ok(())
    }
}