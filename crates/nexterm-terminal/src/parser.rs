use vte::{Params, Parser, Perform};
use tracing::debug;

use crate::screen::Screen;
use crate::{CellStyle, Color};

pub struct TerminalParser {
    parser:    Parser,
    performer: TerminalPerformer,
}

impl TerminalParser {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            parser:    Parser::new(),
            performer: TerminalPerformer::new(rows, cols),
        }
    }

    pub fn process(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.parser.advance(&mut self.performer, *byte);
        }
    }

    pub fn screen(&self) -> &Screen {
        &self.performer.screen
    }

    pub fn screen_mut(&mut self) -> &mut Screen {
        &mut self.performer.screen
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.performer.screen.resize(rows, cols);
    }
}

struct TerminalPerformer {
    screen:        Screen,
    current_style: CellStyle,
}

impl TerminalPerformer {
    fn new(rows: u16, cols: u16) -> Self {
        Self {
            screen:        Screen::new(rows, cols),
            current_style: CellStyle::default(),
        }
    }
}

impl Perform for TerminalPerformer {
    fn print(&mut self, c: char) {
        self.screen.put_char(c, self.current_style);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' | 0x0B | 0x0C => self.screen.newline(),
            b'\r'               => self.screen.carriage_return(),
            0x08                => self.screen.backspace(),
            b'\t'               => self.screen.tab(),
            0x07                => debug!("Bell"),
            _                   => debug!("Unhandled control: {:#x}", byte),
        }
    }

    fn csi_dispatch(
        &mut self,
        params:         &Params,
        _intermediates: &[u8],
        _ignore:        bool,
        action:         char,
    ) {
        let ps: Vec<u16> = params
            .iter()
            .map(|p| p[0])
            .collect();

        match action {
            'A' => self.screen.cursor_up(
                ps.first().copied().unwrap_or(1).max(1)
            ),
            'B' => self.screen.cursor_down(
                ps.first().copied().unwrap_or(1).max(1)
            ),
            'C' => self.screen.cursor_forward(
                ps.first().copied().unwrap_or(1).max(1)
            ),
            'D' => self.screen.cursor_back(
                ps.first().copied().unwrap_or(1).max(1)
            ),
            'H' | 'f' => {
                let row = ps.first().copied().unwrap_or(1).saturating_sub(1);
                let col = ps.get(1).copied().unwrap_or(1).saturating_sub(1);
                self.screen.set_cursor(row, col);
            }
            'J' => match ps.first().copied().unwrap_or(0) {
                0     => self.screen.erase_below(),
                1     => self.screen.erase_above(),
                2 | 3 => self.screen.erase_all(),
                _     => {}
            },
            'K' => match ps.first().copied().unwrap_or(0) {
                0 => self.screen.erase_line_right(),
                1 => self.screen.erase_line_left(),
                2 => self.screen.erase_line(),
                _ => {}
            },
            'm' => self.handle_sgr(&ps),
            'S' => self.screen.scroll_up(
                ps.first().copied().unwrap_or(1).max(1)
            ),
            'T' => self.screen.scroll_down(
                ps.first().copied().unwrap_or(1).max(1)
            ),
            _ => debug!("Unhandled CSI: {} {:?}", action, ps),
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.len() >= 2 {
            match params[0] {
                b"0" | b"2" => {
                    if let Ok(title) = std::str::from_utf8(params[1]) {
                        self.screen.set_title(title.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    fn hook(&mut self, _: &Params, _: &[u8], _: bool, _: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn esc_dispatch(&mut self, _: &[u8], _: bool, _byte: u8) {}
}

impl TerminalPerformer {
    fn handle_sgr(&mut self, ps: &[u16]) {
        let mut i = 0;
        while i < ps.len() {
            match ps[i] {
                0  => self.current_style = CellStyle::default(),
                1  => self.current_style.bold      = true,
                3  => self.current_style.italic    = true,
                4  => self.current_style.underline = true,
                5  => self.current_style.blink     = true,
                7  => self.current_style.reverse   = true,
                22 => self.current_style.bold      = false,
                23 => self.current_style.italic    = false,
                24 => self.current_style.underline = false,
                27 => self.current_style.reverse   = false,
                n @ 30..=37 => {
                    self.current_style.fg = Color::Indexed((n - 30) as u8);
                }
                39 => self.current_style.fg = Color::Default,
                n @ 40..=47 => {
                    self.current_style.bg = Color::Indexed((n - 40) as u8);
                }
                49 => self.current_style.bg = Color::Default,
                n @ 90..=97 => {
                    self.current_style.fg = Color::Indexed((n - 90 + 8) as u8);
                }
                n @ 100..=107 => {
                    self.current_style.bg = Color::Indexed((n - 100 + 8) as u8);
                }
                38 if ps.get(i + 1) == Some(&5) => {
                    if let Some(&n) = ps.get(i + 2) {
                        self.current_style.fg = Color::Palette(n as u8);
                        i += 2;
                    }
                }
                38 if ps.get(i + 1) == Some(&2) => {
                    if ps.len() > i + 4 {
                        self.current_style.fg = Color::Rgb(
                            ps[i + 2] as u8,
                            ps[i + 3] as u8,
                            ps[i + 4] as u8,
                        );
                        i += 4;
                    }
                }
                48 if ps.get(i + 1) == Some(&5) => {
                    if let Some(&n) = ps.get(i + 2) {
                        self.current_style.bg = Color::Palette(n as u8);
                        i += 2;
                    }
                }
                48 if ps.get(i + 1) == Some(&2) => {
                    if ps.len() > i + 4 {
                        self.current_style.bg = Color::Rgb(
                            ps[i + 2] as u8,
                            ps[i + 3] as u8,
                            ps[i + 4] as u8,
                        );
                        i += 4;
                    }
                }
                n => debug!("Unhandled SGR: {}", n),
            }
            i += 1;
        }
    }
}