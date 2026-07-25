use crate::CellStyle;

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub ch:    char,
    pub style: CellStyle,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch:    ' ',
            style: CellStyle::default(),
        }
    }
}

impl Cell {
    pub fn new(ch: char, style: CellStyle) -> Self {
        Self { ch, style }
    }

    pub fn blank() -> Self {
        Self::default()
    }

    pub fn is_blank(&self) -> bool {
        self.ch == ' ' && self.style == CellStyle::default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CursorState {
    pub row:     u16,
    pub col:     u16,
    pub visible: bool,
    pub blink:   bool,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            row:     0,
            col:     0,
            visible: true,
            blink:   true,
        }
    }
}

pub struct Screen {
    pub rows:          u16,
    pub cols:          u16,
    cells:             Vec<Vec<Cell>>,
    pub cursor:        CursorState,
    pub title:         String,
    scroll_top:        u16,
    scroll_bottom:     u16,
    pub dirty:         bool,
}

impl Screen {
    pub fn new(rows: u16, cols: u16) -> Self {
        let cells = vec![
            vec![Cell::blank(); cols as usize];
            rows as usize
        ];

        Self {
            rows,
            cols,
            cells,
            cursor:        CursorState::default(),
            title:         "NexTerm".into(),
            scroll_top:    0,
            scroll_bottom: rows.saturating_sub(1),
            dirty:         true,
        }
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        let mut new_cells = vec![
            vec![Cell::blank(); cols as usize];
            rows as usize
        ];

        let copy_rows = self.rows.min(rows) as usize;
        let copy_cols = self.cols.min(cols) as usize;

        for r in 0..copy_rows {
            for c in 0..copy_cols {
                new_cells[r][c] = self.cells[r][c].clone();
            }
        }

        self.rows          = rows;
        self.cols          = cols;
        self.cells         = new_cells;
        self.scroll_top    = 0;
        self.scroll_bottom = rows.saturating_sub(1);
        self.cursor.row    = self.cursor.row.min(rows.saturating_sub(1));
        self.cursor.col    = self.cursor.col.min(cols.saturating_sub(1));
        self.dirty         = true;
    }

    pub fn put_char(&mut self, ch: char, style: CellStyle) {
        if self.cursor.col >= self.cols {
            self.carriage_return();
            self.newline();
        }

        let row = self.cursor.row as usize;
        let col = self.cursor.col as usize;

        if row < self.rows as usize && col < self.cols as usize {
            self.cells[row][col] = Cell::new(ch, style);
            self.cursor.col += 1;
        }

        self.dirty = true;
    }

    pub fn newline(&mut self) {
        if self.cursor.row >= self.scroll_bottom {
            self.scroll_up(1);
        } else {
            self.cursor.row += 1;
        }
        self.dirty = true;
    }

    pub fn carriage_return(&mut self) {
        self.cursor.col = 0;
    }

    pub fn backspace(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        }
        self.dirty = true;
    }

    pub fn tab(&mut self) {
        let next_tab = ((self.cursor.col / 8) + 1) * 8;
        self.cursor.col = next_tab.min(self.cols.saturating_sub(1));
        self.dirty = true;
    }

    pub fn set_cursor(&mut self, row: u16, col: u16) {
        self.cursor.row = row.min(self.rows.saturating_sub(1));
        self.cursor.col = col.min(self.cols.saturating_sub(1));
    }

    pub fn cursor_up(&mut self, n: u16) {
        self.cursor.row = self.cursor.row.saturating_sub(n);
    }

    pub fn cursor_down(&mut self, n: u16) {
        self.cursor.row = (self.cursor.row + n)
            .min(self.rows.saturating_sub(1));
    }

    pub fn cursor_forward(&mut self, n: u16) {
        self.cursor.col = (self.cursor.col + n)
            .min(self.cols.saturating_sub(1));
    }

    pub fn cursor_back(&mut self, n: u16) {
        self.cursor.col = self.cursor.col.saturating_sub(n);
    }

    pub fn scroll_up(&mut self, n: u16) {
        let top    = self.scroll_top    as usize;
        let bottom = self.scroll_bottom as usize;

        for _ in 0..n {
            self.cells.remove(top);
            self.cells.insert(
                bottom,
                vec![Cell::blank(); self.cols as usize],
            );
        }
        self.dirty = true;
    }

    pub fn scroll_down(&mut self, n: u16) {
        let top    = self.scroll_top    as usize;
        let bottom = self.scroll_bottom as usize;

        for _ in 0..n {
            self.cells.remove(bottom);
            self.cells.insert(
                top,
                vec![Cell::blank(); self.cols as usize],
            );
        }
        self.dirty = true;
    }

    pub fn erase_below(&mut self) {
        let row = self.cursor.row as usize;
        let col = self.cursor.col as usize;

        for c in col..self.cols as usize {
            self.cells[row][c] = Cell::blank();
        }
        for r in (row + 1)..self.rows as usize {
            for c in 0..self.cols as usize {
                self.cells[r][c] = Cell::blank();
            }
        }
        self.dirty = true;
    }

    pub fn erase_above(&mut self) {
        let row = self.cursor.row as usize;
        let col = self.cursor.col as usize;

        for r in 0..row {
            for c in 0..self.cols as usize {
                self.cells[r][c] = Cell::blank();
            }
        }
        for c in 0..=col {
            self.cells[row][c] = Cell::blank();
        }
        self.dirty = true;
    }

    pub fn erase_all(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::blank();
            }
        }
        self.cursor = CursorState::default();
        self.dirty  = true;
    }

    pub fn erase_line_right(&mut self) {
        let row = self.cursor.row as usize;
        let col = self.cursor.col as usize;

        for c in col..self.cols as usize {
            self.cells[row][c] = Cell::blank();
        }
        self.dirty = true;
    }

    pub fn erase_line_left(&mut self) {
        let row = self.cursor.row as usize;
        let col = self.cursor.col as usize;

        for c in 0..=col {
            self.cells[row][c] = Cell::blank();
        }
        self.dirty = true;
    }

    pub fn erase_line(&mut self) {
        let row = self.cursor.row as usize;

        for c in 0..self.cols as usize {
            self.cells[row][c] = Cell::blank();
        }
        self.dirty = true;
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn row(&self, row: u16) -> Option<&Vec<Cell>> {
        self.cells.get(row as usize)
    }

    pub fn cell(&self, row: u16, col: u16) -> Option<&Cell> {
        self.cells
            .get(row as usize)
            .and_then(|r| r.get(col as usize))
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}