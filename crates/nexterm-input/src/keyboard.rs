use winit::event::{KeyEvent, ElementState};
use winit::keyboard::{KeyCode, PhysicalKey, ModifiersState};
use tracing::debug;

use nexterm_core::event::AppEvent;
use nexterm_core::config::KeybindsConfig;

#[derive(Debug, Clone, PartialEq)]
pub enum KeyAction {
    SendBytes(Vec<u8>),
    AppEvent(AppEvent),
    Ignore,
}

pub struct KeyHandler {
    keybinds:  KeybindsConfig,
    modifiers: ModifiersState,
}

impl KeyHandler {
    pub fn new(keybinds: KeybindsConfig) -> Self {
        Self {
            keybinds,
            modifiers: ModifiersState::default(),
        }
    }

    pub fn update_modifiers(&mut self, modifiers: ModifiersState) {
        self.modifiers = modifiers;
    }

    pub fn handle(&self, event: &KeyEvent) -> KeyAction {
        if event.state != ElementState::Pressed {
            return KeyAction::Ignore;
        }

        let ctrl  = self.modifiers.control_key();
        let shift = self.modifiers.shift_key();
        let alt   = self.modifiers.alt_key();

        debug!(
            "Key: {:?} ctrl={} shift={} alt={}",
            event.physical_key, ctrl, shift, alt
        );

        if let Some(action) = self.check_keybinds(
            &event.physical_key, ctrl, shift, alt
        ) {
            return action;
        }

        if let Some(bytes) = self.handle_special_keys(
            &event.physical_key, ctrl, shift, alt
        ) {
            return KeyAction::SendBytes(bytes);
        }

        if let Some(text) = &event.text {
            return KeyAction::SendBytes(text.as_bytes().to_vec());
        }

        KeyAction::Ignore
    }

    fn check_keybinds(
        &self,
        key:   &PhysicalKey,
        ctrl:  bool,
        shift: bool,
        _alt:  bool,
    ) -> Option<KeyAction> {
        match key {
            PhysicalKey::Code(code) => match code {
                KeyCode::KeyT if ctrl && !shift => Some(KeyAction::AppEvent(
                    AppEvent::TabCreated { tab_id: 0 }
                )),
                KeyCode::KeyW if ctrl && !shift => Some(KeyAction::AppEvent(
                    AppEvent::TabClosed { tab_id: 0 }
                )),
                KeyCode::Tab if ctrl && !shift => Some(KeyAction::AppEvent(
                    AppEvent::TabSwitched { tab_id: 0 }
                )),
                KeyCode::KeyP if ctrl && !shift => Some(KeyAction::AppEvent(
                    AppEvent::PluginEvent {
                        plugin_id: "command-palette".into(),
                        payload:   vec![],
                    }
                )),
                _ => None,
            },
            _ => None,
        }
    }

    fn handle_special_keys(
        &self,
        key:   &PhysicalKey,
        ctrl:  bool,
        _shift: bool,
        alt:   bool,
    ) -> Option<Vec<u8>> {
        match key {
            PhysicalKey::Code(code) => match code {
                KeyCode::Enter     => Some(b"\r".to_vec()),
                KeyCode::Backspace => Some(vec![0x7f]),
                KeyCode::Tab       => Some(b"\t".to_vec()),
                KeyCode::Escape    => Some(b"\x1b".to_vec()),
                KeyCode::ArrowUp    => Some(b"\x1b[A".to_vec()),
                KeyCode::ArrowDown  => Some(b"\x1b[B".to_vec()),
                KeyCode::ArrowRight => Some(b"\x1b[C".to_vec()),
                KeyCode::ArrowLeft  => Some(b"\x1b[D".to_vec()),
                KeyCode::Home      => Some(b"\x1b[H".to_vec()),
                KeyCode::End       => Some(b"\x1b[F".to_vec()),
                KeyCode::PageUp    => Some(b"\x1b[5~".to_vec()),
                KeyCode::PageDown  => Some(b"\x1b[6~".to_vec()),
                KeyCode::Delete    => Some(b"\x1b[3~".to_vec()),
                KeyCode::Insert    => Some(b"\x1b[2~".to_vec()),
                KeyCode::F1  => Some(b"\x1bOP".to_vec()),
                KeyCode::F2  => Some(b"\x1bOQ".to_vec()),
                KeyCode::F3  => Some(b"\x1bOR".to_vec()),
                KeyCode::F4  => Some(b"\x1bOS".to_vec()),
                KeyCode::F5  => Some(b"\x1b[15~".to_vec()),
                KeyCode::F6  => Some(b"\x1b[17~".to_vec()),
                KeyCode::F7  => Some(b"\x1b[18~".to_vec()),
                KeyCode::F8  => Some(b"\x1b[19~".to_vec()),
                KeyCode::F9  => Some(b"\x1b[20~".to_vec()),
                KeyCode::F10 => Some(b"\x1b[21~".to_vec()),
                KeyCode::F11 => Some(b"\x1b[23~".to_vec()),
                KeyCode::F12 => Some(b"\x1b[24~".to_vec()),
                KeyCode::KeyC if ctrl => Some(vec![0x03]),
                KeyCode::KeyD if ctrl => Some(vec![0x04]),
                KeyCode::KeyZ if ctrl => Some(vec![0x1a]),
                KeyCode::KeyL if ctrl => Some(vec![0x0c]),
                KeyCode::KeyA if ctrl => Some(vec![0x01]),
                KeyCode::KeyE if ctrl => Some(vec![0x05]),
                KeyCode::KeyU if ctrl => Some(vec![0x15]),
                KeyCode::KeyK if ctrl => Some(vec![0x0b]),
                code if alt => {
                    let ch = keycode_to_char(code)?;
                    Some(vec![0x1b, ch as u8])
                }
                _ => None,
            },
            _ => None,
        }
    }
}

fn keycode_to_char(code: &KeyCode) -> Option<char> {
    match code {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        _ => None,
    }
}