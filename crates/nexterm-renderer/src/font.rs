use cosmic_text::{
    Attrs, Buffer, Color, Family, FontSystem as CosmicFontSystem,
    Metrics, Shaping, SwashCache,
};
use tracing::info;

use crate::{Result, RgbaColor};

#[derive(Debug, Clone, Copy)]
pub struct CellSize {
    pub width:  f32,
    pub height: f32,
}

pub struct FontSystem {
    pub font_system: CosmicFontSystem,
    pub swash_cache: SwashCache,
    pub cell_size:   CellSize,
    font_name:       String,
    font_size:       f32,
    line_height:     f32,
}

impl FontSystem {
    pub fn new(
        font_name:   &str,
        font_size:   f32,
        line_height: f32,
    ) -> Result<Self> {
        info!("Initializing font: {} {}pt", font_name, font_size);

        let mut font_system = CosmicFontSystem::new();
        let swash_cache     = SwashCache::new();

        let cell_size = calculate_cell_size(
            &mut font_system,
            font_name,
            font_size,
            line_height,
        )?;

        info!("Cell size: {}x{}px", cell_size.width, cell_size.height);

        Ok(Self {
            font_system,
            swash_cache,
            cell_size,
            font_name: font_name.to_string(),
            font_size,
            line_height,
        })
    }

    pub fn set_font(
        &mut self,
        font_name: &str,
        font_size: f32,
    ) -> Result<()> {
        self.font_name = font_name.to_string();
        self.font_size = font_size;

        self.cell_size = calculate_cell_size(
            &mut self.font_system,
            font_name,
            font_size,
            self.line_height,
        )?;

        Ok(())
    }

    pub fn create_buffer(
        &mut self,
        text:  &str,
        color: RgbaColor,
        width: f32,
    ) -> Buffer {
        let metrics = Metrics::new(
            self.font_size,
            self.font_size * self.line_height,
        );

        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        buffer.set_size(&mut self.font_system, width, f32::MAX);

        let attrs = Attrs::new()
            .family(Family::Name(&self.font_name))
            .color(rgba_to_cosmic(color));

        buffer.set_text(
            &mut self.font_system,
            text,
            attrs,
            Shaping::Advanced,
        );

        buffer.shape_until_scroll(&mut self.font_system, false);

        buffer
    }

    pub fn cell_size(&self) -> CellSize {
        self.cell_size
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn calculate_grid_size(
        &self,
        width:  u32,
        height: u32,
    ) -> (u16, u16) {
        let cols = (width  as f32 / self.cell_size.width)  as u16;
        let rows = (height as f32 / self.cell_size.height) as u16;
        (rows.max(1), cols.max(1))
    }
}

fn calculate_cell_size(
    font_system: &mut CosmicFontSystem,
    font_name:   &str,
    font_size:   f32,
    line_height: f32,
) -> Result<CellSize> {
    let metrics = Metrics::new(font_size, font_size * line_height);
    let mut buffer = Buffer::new(font_system, metrics);

    buffer.set_size(font_system, 1000.0, f32::MAX);

    let attrs = Attrs::new().family(Family::Name(font_name));

    buffer.set_text(font_system, "M", attrs, Shaping::Basic);
    buffer.shape_until_scroll(font_system, false);

    let width = buffer
        .layout_runs()
        .next()
        .map(|run| run.line_w)
        .unwrap_or(font_size * 0.6);

    Ok(CellSize {
        width,
        height: font_size * line_height,
    })
}

fn rgba_to_cosmic(color: RgbaColor) -> Color {
    Color::rgba(
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8,
        (color.a * 255.0) as u8,
    )
}