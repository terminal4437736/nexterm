use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

use crate::{RendererError, Result, RgbaColor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsiColors {
    pub black:          String,
    pub red:            String,
    pub green:          String,
    pub yellow:         String,
    pub blue:           String,
    pub magenta:        String,
    pub cyan:           String,
    pub white:          String,
    pub bright_black:   String,
    pub bright_red:     String,
    pub bright_green:   String,
    pub bright_yellow:  String,
    pub bright_blue:    String,
    pub bright_magenta: String,
    pub bright_cyan:    String,
    pub bright_white:   String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiColors {
    pub background:   String,
    pub foreground:   String,
    pub cursor:       String,
    pub cursor_text:  String,
    pub selection_bg: String,
    pub selection_fg: String,
    pub tab_bar_bg:   String,
    pub tab_active:   String,
    pub tab_inactive: String,
    pub border:       String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeToml {
    pub name:   String,
    pub author: Option<String>,
    pub ansi:   AnsiColors,
    pub ui:     UiColors,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name:         String,
    pub ansi:         [RgbaColor; 16],
    pub background:   RgbaColor,
    pub foreground:   RgbaColor,
    pub cursor:       RgbaColor,
    pub cursor_text:  RgbaColor,
    pub selection_bg: RgbaColor,
    pub selection_fg: RgbaColor,
    pub tab_bar_bg:   RgbaColor,
    pub tab_active:   RgbaColor,
    pub tab_inactive: RgbaColor,
    pub border:       RgbaColor,
}

impl Theme {
    pub fn load(path: &Path) -> Result<Self> {
        info!("Loading theme from: {}", path.display());

        let content = std::fs::read_to_string(path)
            .map_err(|e| RendererError::Font(
                format!("Cannot read theme: {}", e)
            ))?;

        let toml: ThemeToml = toml::from_str(&content)
            .map_err(|e| RendererError::Font(
                format!("Invalid theme TOML: {}", e)
            ))?;

        Self::from_toml(toml)
    }

    pub fn from_toml(t: ThemeToml) -> Result<Self> {
        let parse = |hex: &str| -> Result<RgbaColor> {
            RgbaColor::from_hex(hex).ok_or_else(|| {
                RendererError::Font(format!("Invalid color: {}", hex))
            })
        };

        let ansi = [
            parse(&t.ansi.black)?,
            parse(&t.ansi.red)?,
            parse(&t.ansi.green)?,
            parse(&t.ansi.yellow)?,
            parse(&t.ansi.blue)?,
            parse(&t.ansi.magenta)?,
            parse(&t.ansi.cyan)?,
            parse(&t.ansi.white)?,
            parse(&t.ansi.bright_black)?,
            parse(&t.ansi.bright_red)?,
            parse(&t.ansi.bright_green)?,
            parse(&t.ansi.bright_yellow)?,
            parse(&t.ansi.bright_blue)?,
            parse(&t.ansi.bright_magenta)?,
            parse(&t.ansi.bright_cyan)?,
            parse(&t.ansi.bright_white)?,
        ];

        Ok(Self {
            name:         t.name,
            ansi,
            background:   parse(&t.ui.background)?,
            foreground:   parse(&t.ui.foreground)?,
            cursor:       parse(&t.ui.cursor)?,
            cursor_text:  parse(&t.ui.cursor_text)?,
            selection_bg: parse(&t.ui.selection_bg)?,
            selection_fg: parse(&t.ui.selection_fg)?,
            tab_bar_bg:   parse(&t.ui.tab_bar_bg)?,
            tab_active:   parse(&t.ui.tab_active)?,
            tab_inactive: parse(&t.ui.tab_inactive)?,
            border:       parse(&t.ui.border)?,
        })
    }

    pub fn ansi_color(&self, index: u8) -> RgbaColor {
        self.ansi.get(index as usize)
            .copied()
            .unwrap_or(self.foreground)
    }

    pub fn dark() -> Self {
        Self {
            name: "dark".into(),
            ansi: [
                RgbaColor::from_hex("#1d1f21").unwrap(),
                RgbaColor::from_hex("#cc6666").unwrap(),
                RgbaColor::from_hex("#b5bd68").unwrap(),
                RgbaColor::from_hex("#f0c674").unwrap(),
                RgbaColor::from_hex("#81a2be").unwrap(),
                RgbaColor::from_hex("#b294bb").unwrap(),
                RgbaColor::from_hex("#8abeb7").unwrap(),
                RgbaColor::from_hex("#c5c8c6").unwrap(),
                RgbaColor::from_hex("#666666").unwrap(),
                RgbaColor::from_hex("#d54e53").unwrap(),
                RgbaColor::from_hex("#b9ca4a").unwrap(),
                RgbaColor::from_hex("#e7c547").unwrap(),
                RgbaColor::from_hex("#7aa6da").unwrap(),
                RgbaColor::from_hex("#c397d8").unwrap(),
                RgbaColor::from_hex("#70c0b1").unwrap(),
                RgbaColor::from_hex("#eaeaea").unwrap(),
            ],
            background:   RgbaColor::from_hex("#1d1f21").unwrap(),
            foreground:   RgbaColor::from_hex("#c5c8c6").unwrap(),
            cursor:       RgbaColor::from_hex("#c5c8c6").unwrap(),
            cursor_text:  RgbaColor::from_hex("#1d1f21").unwrap(),
            selection_bg: RgbaColor::from_hex("#373b41").unwrap(),
            selection_fg: RgbaColor::from_hex("#c5c8c6").unwrap(),
            tab_bar_bg:   RgbaColor::from_hex("#282a2e").unwrap(),
            tab_active:   RgbaColor::from_hex("#373b41").unwrap(),
            tab_inactive: RgbaColor::from_hex("#282a2e").unwrap(),
            border:       RgbaColor::from_hex("#373b41").unwrap(),
        }
    }
}