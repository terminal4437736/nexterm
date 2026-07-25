use bytemuck::{Pod, Zeroable};

use nexterm_terminal::screen::Screen;
use nexterm_terminal::Color as TermColor;
use nexterm_terminal::screen::Cell;

use crate::theme::Theme;
use crate::font::FontSystem;
use crate::RgbaColor;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color:    [f32; 4],
    pub uv:       [f32; 2],
}

impl Vertex {
    pub fn new(x: f32, y: f32, color: RgbaColor, u: f32, v: f32) -> Self {
        Self {
            position: [x, y],
            color:    [color.r, color.g, color.b, color.a],
            uv:       [u, v],
        }
    }

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode:    wgpu::VertexStepMode::Vertex,
            attributes:   &[
                wgpu::VertexAttribute {
                    offset:          0,
                    shader_location: 0,
                    format:          wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset:          8,
                    shader_location: 1,
                    format:          wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset:          24,
                    shader_location: 2,
                    format:          wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct GridRenderer {
    pub bg_vertices:   Vec<Vertex>,
    pub bg_indices:    Vec<u32>,
    pub text_vertices: Vec<Vertex>,
    pub text_indices:  Vec<u32>,
}

impl GridRenderer {
    pub fn new() -> Self {
        Self {
            bg_vertices:   Vec::new(),
            bg_indices:    Vec::new(),
            text_vertices: Vec::new(),
            text_indices:  Vec::new(),
        }
    }

    pub fn build(
        &mut self,
        screen:     &Screen,
        font:       &FontSystem,
        theme:      &Theme,
        win_width:  u32,
        win_height: u32,
    ) {
        self.bg_vertices.clear();
        self.bg_indices.clear();
        self.text_vertices.clear();
        self.text_indices.clear();

        let cell_w = font.cell_size().width;
        let cell_h = font.cell_size().height;

        for row in 0..screen.rows {
            let cells = match screen.row(row) {
                Some(c) => c,
                None    => continue,
            };

            for col in 0..screen.cols {
                let cell = &cells[col as usize];
                let px   = col as f32 * cell_w;
                let py   = row as f32 * cell_h;

                let (fg, bg) = resolve_colors(cell, theme);

                self.push_quad_bg(
                    px, py, cell_w, cell_h,
                    win_width, win_height, bg,
                );

                if cell.ch != ' ' {
                    self.push_char(
                        px, py, cell_w, cell_h,
                        win_width, win_height, fg,
                    );
                }
            }
        }

        self.draw_cursor(screen, font, theme, win_width, win_height);
    }

    fn draw_cursor(
        &mut self,
        screen:     &Screen,
        font:       &FontSystem,
        theme:      &Theme,
        win_width:  u32,
        win_height: u32,
    ) {
        if !screen.cursor.visible {
            return;
        }

        let cell_w = font.cell_size().width;
        let cell_h = font.cell_size().height;
        let px     = screen.cursor.col as f32 * cell_w;
        let py     = screen.cursor.row as f32 * cell_h;

        self.push_quad_bg(
            px, py, cell_w, cell_h,
            win_width, win_height,
            theme.cursor,
        );
    }

    fn push_quad_bg(
        &mut self,
        px:         f32,
        py:         f32,
        w:          f32,
        h:          f32,
        win_width:  u32,
        win_height: u32,
        color:      RgbaColor,
    ) {
        let base = self.bg_vertices.len() as u32;

        let (x0, y0) = to_ndc(px,     py,     win_width, win_height);
        let (x1, y1) = to_ndc(px + w, py + h, win_width, win_height);

        self.bg_vertices.extend_from_slice(&[
            Vertex::new(x0, y0, color, 0.0, 0.0),
            Vertex::new(x1, y0, color, 1.0, 0.0),
            Vertex::new(x0, y1, color, 0.0, 1.0),
            Vertex::new(x1, y1, color, 1.0, 1.0),
        ]);

        self.bg_indices.extend_from_slice(&[
            base,     base + 1, base + 2,
            base + 1, base + 3, base + 2,
        ]);
    }

    fn push_char(
        &mut self,
        px:         f32,
        py:         f32,
        w:          f32,
        h:          f32,
        win_width:  u32,
        win_height: u32,
        color:      RgbaColor,
    ) {
        let base = self.text_vertices.len() as u32;

        let (x0, y0) = to_ndc(px + 1.0,     py + 1.0,     win_width, win_height);
        let (x1, y1) = to_ndc(px + w - 1.0, py + h - 1.0, win_width, win_height);

        self.text_vertices.extend_from_slice(&[
            Vertex::new(x0, y0, color, 0.0, 0.0),
            Vertex::new(x1, y0, color, 1.0, 0.0),
            Vertex::new(x0, y1, color, 0.0, 1.0),
            Vertex::new(x1, y1, color, 1.0, 1.0),
        ]);

        self.text_indices.extend_from_slice(&[
            base,     base + 1, base + 2,
            base + 1, base + 3, base + 2,
        ]);
    }
}

impl Default for GridRenderer {
    fn default() -> Self {
        Self::new()
    }
}

fn to_ndc(
    px:         f32,
    py:         f32,
    win_width:  u32,
    win_height: u32,
) -> (f32, f32) {
    let x =  (px / win_width  as f32) * 2.0 - 1.0;
    let y = -(py / win_height as f32) * 2.0 + 1.0;
    (x, y)
}

fn resolve_colors(cell: &Cell, theme: &Theme) -> (RgbaColor, RgbaColor) {
    let style = &cell.style;

    let mut fg = match style.fg {
        TermColor::Default      => theme.foreground,
        TermColor::Indexed(i)   => theme.ansi_color(i),
        TermColor::Palette(i)   => theme.ansi_color(i),
        TermColor::Rgb(r, g, b) => RgbaColor::from((r, g, b)),
    };

    let mut bg = match style.bg {
        TermColor::Default      => theme.background,
        TermColor::Indexed(i)   => theme.ansi_color(i),
        TermColor::Palette(i)   => theme.ansi_color(i),
        TermColor::Rgb(r, g, b) => RgbaColor::from((r, g, b)),
    };

    if style.reverse {
        std::mem::swap(&mut fg, &mut bg);
    }

    (fg, bg)
}