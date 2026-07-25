pub mod parser;
pub mod screen;
pub mod scrollback;

pub use parser::TerminalParser;
pub use screen::{Screen, Cell, CursorState};
pub use scrollback::ScrollbackBuffer;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TerminalError {
    #[error("Parser error: {0}")]
    Parser(String),

    #[error("Screen error: {0}")]
    Screen(String),

    #[error("Invalid UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, TerminalError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Default,
    Indexed(u8),
    Palette(u8),
    Rgb(u8, u8, u8),
}

impl Default for Color {
    fn default() -> Self {
        Color::Default
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CellStyle {
    pub bold:      bool,
    pub italic:    bool,
    pub underline: bool,
    pub blink:     bool,
    pub reverse:   bool,
    pub invisible: bool,
    pub fg:        Color,
    pub bg:        Color,
}