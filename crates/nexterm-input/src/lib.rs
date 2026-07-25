pub mod keyboard;
pub mod mouse;

pub use keyboard::{KeyHandler, KeyAction};
pub use mouse::{MouseHandler, MouseAction};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("Unknown key: {0}")]
    UnknownKey(String),

    #[error("Invalid keybind: {0}")]
    InvalidKeybind(String),

    #[error("Input handler error: {0}")]
    Handler(String),
}

pub type Result<T> = std::result::Result<T, InputError>;