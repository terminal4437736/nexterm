//! Tab bar renderer
//! Window ke top pe tabs dikhata hai
//! Active tab highlight hota hai

use crate::theme::Theme;


/// Ek tab ka data
#[derive(Debug, Clone)]
pub struct TabInfo {
    pub id:     usize,
    pub title:  String,
    pub active: bool,
}

/// Tab bar config
pub struct TabBar {
    pub height:    f32,
    pub font_size: f32,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            height:    32.0,
            font_size: 13.0,
        }
    }

    /// Tab bar ki height lo
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Tab rectangles calculate karo
    pub fn tab_rects(
        &self,
        tabs:       &[TabInfo],
        _win_width: u32,
    ) -> Vec<TabRect> {
        let mut rects = Vec::new();
        let tab_width = 160.0f32;
        let mut x     = 0.0f32;

        for tab in tabs {
            rects.push(TabRect {
                x,
                y:      0.0,
                width:  tab_width,
                height: self.height,
                tab_id: tab.id,
                active: tab.active,
                title:  tab.title.clone(),
            });
            x += tab_width;
        }

        // Plus button — naya tab
        rects.push(TabRect {
            x,
            y:      0.0,
            width:  32.0,
            height: self.height,
            tab_id: usize::MAX,
            active: false,
            title:  "+".into(),
        });

        rects
    }

    /// Tab bar vertices banao
    pub fn build_vertices(
        &self,
        tabs:       &[TabInfo],
        theme:      &Theme,
        win_width: u32,
        win_height: u32,
    ) -> (Vec<[f32; 2]>, Vec<[f32; 4]>, Vec<u32>) {
        let mut positions = Vec::new();
        let mut colors    = Vec::new();
        let mut indices   = Vec::new();

        let rects = self.tab_rects(tabs, win_width);

        for rect in &rects {
            let color = if rect.active {
                theme.tab_active
            } else {
                theme.tab_inactive
            };

            let base = positions.len() as u32;

            let (x0, y0) = to_ndc(
                rect.x, rect.y,
                win_width, win_height
            );
            let (x1, y1) = to_ndc(
                rect.x + rect.width,
                rect.y + rect.height,
                win_width, win_height,
            );

            positions.extend_from_slice(&[
                [x0, y0], [x1, y0],
                [x0, y1], [x1, y1],
            ]);

            let c = [color.r, color.g, color.b, color.a];
            colors.extend_from_slice(&[c, c, c, c]);

            indices.extend_from_slice(&[
                base,     base + 1, base + 2,
                base + 1, base + 3, base + 2,
            ]);
        }

        (positions, colors, indices)
    }

    /// Click position se tab_id lo
    pub fn hit_test(
        &self,
        tabs: &[TabInfo],
        x:    f32,
        y:    f32,
        win_width: u32,
    ) -> Option<usize> {
        if y > self.height {
            return None;
        }

        let rects = self.tab_rects(tabs, win_width);

        for rect in &rects {
            if x >= rect.x && x <= rect.x + rect.width {
                return Some(rect.tab_id);
            }
        }

        None
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Tab ka rectangle — position aur size
#[derive(Debug, Clone)]
pub struct TabRect {
    pub x:      f32,
    pub y:      f32,
    pub width:  f32,
    pub height: f32,
    pub tab_id: usize,
    pub active: bool,
    pub title:  String,
}

/// Pixel to NDC
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