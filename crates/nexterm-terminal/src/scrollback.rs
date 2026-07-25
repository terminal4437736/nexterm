use crate::screen::Cell;

#[derive(Debug, Clone)]
pub struct ScrollbackLine {
    pub cells: Vec<Cell>,
}

impl ScrollbackLine {
    pub fn new(cells: Vec<Cell>) -> Self {
        Self { cells }
    }

    pub fn blank(cols: u16) -> Self {
        Self {
            cells: vec![Cell::blank(); cols as usize],
        }
    }
}

pub struct ScrollbackBuffer {
    lines:             Vec<ScrollbackLine>,
    max_lines:         usize,
    pub scroll_offset: usize,
}

impl ScrollbackBuffer {
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines:         Vec::with_capacity(max_lines),
            max_lines,
            scroll_offset: 0,
        }
    }

    pub fn push_line(&mut self, cells: Vec<Cell>) {
        if self.lines.len() >= self.max_lines {
            self.lines.remove(0);
        }
        self.lines.push(ScrollbackLine::new(cells));
    }

    pub fn push_lines(&mut self, lines: Vec<Vec<Cell>>) {
        for cells in lines {
            self.push_line(cells);
        }
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = (self.scroll_offset + n)
            .min(self.lines.len());
    }

    pub fn scroll_down(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset
            .saturating_sub(n);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = self.lines.len();
    }

    pub fn is_at_bottom(&self) -> bool {
        self.scroll_offset == 0
    }

    pub fn visible_lines(&self, n: usize) -> Vec<&ScrollbackLine> {
        if self.lines.is_empty() || self.scroll_offset == 0 {
            return vec![];
        }

        let end   = self.lines.len().saturating_sub(self.scroll_offset);
        let start = end.saturating_sub(n);

        self.lines[start..end].iter().collect()
    }

    pub fn search(&self, query: &str) -> Vec<(usize, usize)> {
        let mut results = Vec::new();

        for (line_idx, line) in self.lines.iter().enumerate() {
            let text: String = line.cells
                .iter()
                .map(|c| c.ch)
                .collect();

            if let Some(col) = text.find(query) {
                results.push((line_idx, col));
            }
        }

        results
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll_offset = 0;
    }

    pub fn to_text(&self) -> String {
        self.lines
            .iter()
            .map(|line| {
                let text: String = line.cells
                    .iter()
                    .map(|c| c.ch)
                    .collect();
                text.trim_end().to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}