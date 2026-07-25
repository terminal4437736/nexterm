use winit::event::{MouseButton, MouseScrollDelta, ElementState};
use tracing::debug;

#[derive(Debug, Clone, PartialEq)]
pub enum MouseAction {
    Select {
        start_col: u16,
        start_row: u16,
        end_col:   u16,
        end_row:   u16,
    },
    ScrollUp   { lines: u32 },
    ScrollDown { lines: u32 },
    LeftClick  { col: u16, row: u16 },
    RightClick { col: u16, row: u16 },
    SendBytes(Vec<u8>),
    Ignore,
}

pub struct MouseHandler {
    cursor_x:        f64,
    cursor_y:        f64,
    cell_width:      f64,
    cell_height:     f64,
    left_pressed:    bool,
    select_start:    Option<(u16, u16)>,
    mouse_reporting: bool,
}

impl MouseHandler {
    pub fn new(cell_width: f64, cell_height: f64) -> Self {
        Self {
            cursor_x:        0.0,
            cursor_y:        0.0,
            cell_width,
            cell_height,
            left_pressed:    false,
            select_start:    None,
            mouse_reporting: false,
        }
    }

    pub fn update_cell_size(&mut self, width: f64, height: f64) {
        self.cell_width  = width;
        self.cell_height = height;
    }

    pub fn set_mouse_reporting(&mut self, enabled: bool) {
        self.mouse_reporting = enabled;
        debug!("Mouse reporting: {}", enabled);
    }

    pub fn on_move(&mut self, x: f64, y: f64) -> MouseAction {
        self.cursor_x = x;
        self.cursor_y = y;

        if self.left_pressed {
            if let Some((start_col, start_row)) = self.select_start {
                let (end_col, end_row) = self.pixel_to_cell(x, y);
                return MouseAction::Select {
                    start_col,
                    start_row,
                    end_col,
                    end_row,
                };
            }
        }

        MouseAction::Ignore
    }

    pub fn on_button(
        &mut self,
        button: MouseButton,
        state:  ElementState,
    ) -> MouseAction {
        let (col, row) = self.pixel_to_cell(self.cursor_x, self.cursor_y);

        match (button, state) {
            (MouseButton::Left, ElementState::Pressed) => {
                self.left_pressed = true;
                self.select_start = Some((col, row));

                if self.mouse_reporting {
                    return MouseAction::SendBytes(
                        mouse_report_bytes(0, col, row, true)
                    );
                }

                MouseAction::LeftClick { col, row }
            }
            (MouseButton::Left, ElementState::Released) => {
                self.left_pressed = false;

                if self.mouse_reporting {
                    return MouseAction::SendBytes(
                        mouse_report_bytes(0, col, row, false)
                    );
                }

                MouseAction::Ignore
            }
            (MouseButton::Right, ElementState::Pressed) => {
                MouseAction::RightClick { col, row }
            }
            _ => MouseAction::Ignore,
        }
    }

    pub fn on_scroll(&mut self, delta: MouseScrollDelta) -> MouseAction {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                if y > 0.0 {
                    MouseAction::ScrollUp  { lines: y.abs() as u32 }
                } else {
                    MouseAction::ScrollDown { lines: y.abs() as u32 }
                }
            }
            MouseScrollDelta::PixelDelta(pos) => {
                if pos.y > 0.0 {
                    MouseAction::ScrollUp  { lines: 3 }
                } else {
                    MouseAction::ScrollDown { lines: 3 }
                }
            }
        }
    }

    fn pixel_to_cell(&self, x: f64, y: f64) -> (u16, u16) {
        let col = (x / self.cell_width)  as u16;
        let row = (y / self.cell_height) as u16;
        (col, row)
    }
}

fn mouse_report_bytes(
    button: u8,
    col:    u16,
    row:    u16,
    press:  bool,
) -> Vec<u8> {
    let btn = if press { button } else { 3 };
    vec![
        0x1b, b'[', b'M',
        32 + btn,
        32 + (col + 1) as u8,
        32 + (row + 1) as u8,
    ]
}