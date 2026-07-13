use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Debug, Default)]
pub struct CellFlags {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
    pub dim: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Cell {
    pub ch: char,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub flags: CellFlags,
    pub wide: bool,
}

impl Cell {
    pub fn empty() -> Self {
        Cell { ch: ' ', fg: [255, 255, 255], bg: [0, 0, 0], flags: CellFlags::default(), wide: false }
    }
}

pub struct Grid {
    pub cells: Vec<Vec<Cell>>,
    pub cols: usize,
    pub rows: usize,
    pub cursor_col: usize,
    pub cursor_row: usize,
    pub scroll_top: usize,
    pub scroll_bottom: usize,
    pub current_fg: [u8; 3],
    pub current_bg: [u8; 3],
    pub current_flags: CellFlags,
    pub cursor_visible: bool,
    pub alt_screen: bool,
    alt_grid: Vec<Vec<Cell>>,
    alt_cursor: (usize, usize),
}

impl Grid {
    pub fn new(cols: usize, rows: usize) -> Self {
        let cells = vec![vec![Cell::empty(); cols]; rows];
        Grid {
            cells, cols, rows,
            cursor_col: 0, cursor_row: 0,
            scroll_top: 0, scroll_bottom: rows.saturating_sub(1),
            current_fg: [255, 255, 255], current_bg: [0, 0, 0],
            current_flags: CellFlags::default(),
            cursor_visible: true,
            alt_screen: false,
            alt_grid: Vec::new(),
            alt_cursor: (0, 0),
        }
    }

    /// Resize without reflow: truncate or extend rows/cols.
    pub fn resize(&mut self, cols: usize, rows: usize) {
        if cols == self.cols && rows == self.rows { return; }

        // Resize each row
        for row in &mut self.cells {
            if cols > row.len() {
                row.resize(cols, Cell::empty());
            } else {
                row.truncate(cols);
            }
        }
        // Resize alt grid too
        for row in &mut self.alt_grid {
            if cols > row.len() {
                row.resize(cols, Cell::empty());
            } else {
                row.truncate(cols);
            }
        }

        // Add or remove rows
        if rows > self.cells.len() {
            self.cells.resize(rows, vec![Cell::empty(); cols]);
        } else {
            self.cells.truncate(rows);
        }
        if rows > self.alt_grid.len() {
            self.alt_grid.resize(rows, vec![Cell::empty(); cols]);
        } else {
            self.alt_grid.truncate(rows);
        }

        self.cols = cols;
        self.rows = rows;
        self.scroll_top = 0;
        self.scroll_bottom = rows.saturating_sub(1);
        self.cursor_col = self.cursor_col.min(cols.saturating_sub(1));
        self.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
    }

    pub fn current_cell_mut(&mut self) -> Option<&mut Cell> {
        self.cells.get_mut(self.cursor_row)?.get_mut(self.cursor_col)
    }

    pub fn put_char(&mut self, ch: char) {
        // Auto-wrap: if cursor at last col with pending wrap
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.line_feed();
        }
        let fg = self.current_fg;
        let bg = self.current_bg;
        let flags = self.current_flags;
        if let Some(cell) = self.current_cell_mut() {
            cell.ch = ch;
            cell.fg = fg;
            cell.bg = bg;
            cell.flags = flags;
            cell.wide = false;
        }
        self.cursor_col += 1;
    }

    pub fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    pub fn line_feed(&mut self) {
        if self.cursor_row == self.scroll_bottom {
            self.scroll_up(1);
        } else if self.cursor_row < self.rows - 1 {
            self.cursor_row += 1;
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    pub fn tab(&mut self) {
        let next = ((self.cursor_col / 8) + 1) * 8;
        self.cursor_col = next.min(self.cols.saturating_sub(1));
    }

    pub fn scroll_up(&mut self, n: usize) {
        let top = self.scroll_top;
        let bottom = self.scroll_bottom;
        if bottom < self.rows {
            for _ in 0..n {
                self.cells.remove(top);
                self.cells.insert(bottom, vec![Cell::empty(); self.cols]);
            }
        }
    }

    pub fn scroll_down(&mut self, n: usize) {
        let top = self.scroll_top;
        let bottom = self.scroll_bottom;
        if bottom < self.rows {
            for _ in 0..n {
                self.cells.remove(bottom);
                self.cells.insert(top, vec![Cell::empty(); self.cols]);
            }
        }
    }

    pub fn clear_screen(&mut self, mode: u16) {
        match mode {
            0 => { // Clear from cursor to end
                for col in self.cursor_col..self.cols {
                    if let Some(c) = self.cells.get_mut(self.cursor_row).and_then(|r| r.get_mut(col)) {
                        *c = Cell::empty();
                    }
                }
                for row in (self.cursor_row + 1)..self.rows {
                    for col in 0..self.cols {
                        self.cells[row][col] = Cell::empty();
                    }
                }
            }
            1 => { // Clear from start to cursor
                for row in 0..self.cursor_row {
                    for col in 0..self.cols {
                        self.cells[row][col] = Cell::empty();
                    }
                }
                for col in 0..=self.cursor_col {
                    if let Some(c) = self.cells.get_mut(self.cursor_row).and_then(|r| r.get_mut(col)) {
                        *c = Cell::empty();
                    }
                }
            }
            2 | 3 => { // Clear entire screen
                for row in 0..self.rows {
                    for col in 0..self.cols {
                        self.cells[row][col] = Cell::empty();
                    }
                }
                self.cursor_col = 0;
                self.cursor_row = 0;
            }
            _ => {}
        }
    }

    pub fn clear_line(&mut self, mode: u16) {
        match mode {
            0 => { // Cursor to end
                for col in self.cursor_col..self.cols {
                    if let Some(r) = self.cells.get_mut(self.cursor_row) {
                        r[col] = Cell::empty();
                    }
                }
            }
            1 => { // Start to cursor
                for col in 0..=self.cursor_col.min(self.cols.saturating_sub(1)) {
                    if let Some(r) = self.cells.get_mut(self.cursor_row) {
                        r[col] = Cell::empty();
                    }
                }
            }
            2 => { // Entire line
                if let Some(r) = self.cells.get_mut(self.cursor_row) {
                    for cell in r.iter_mut() { *cell = Cell::empty(); }
                }
            }
            _ => {}
        }
    }

    pub fn move_cursor(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.rows.saturating_sub(1));
        self.cursor_col = col.min(self.cols.saturating_sub(1));
    }

    pub fn cursor_up(&mut self, n: usize) {
        self.cursor_row = self.cursor_row.saturating_sub(n);
    }

    pub fn cursor_down(&mut self, n: usize) {
        self.cursor_row = (self.cursor_row + n).min(self.rows.saturating_sub(1));
    }

    pub fn cursor_forward(&mut self, n: usize) {
        self.cursor_col = (self.cursor_col + n).min(self.cols.saturating_sub(1));
    }

    pub fn cursor_back(&mut self, n: usize) {
        self.cursor_col = self.cursor_col.saturating_sub(n);
    }

    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        self.scroll_top = top.min(self.rows.saturating_sub(1));
        self.scroll_bottom = bottom.min(self.rows.saturating_sub(1)).max(self.scroll_top);
    }

    pub fn save_cursor(&mut self) {
        self.alt_cursor = (self.cursor_row, self.cursor_col);
    }

    pub fn restore_cursor(&mut self) {
        self.cursor_row = self.alt_cursor.0.min(self.rows.saturating_sub(1));
        self.cursor_col = self.alt_cursor.1.min(self.cols.saturating_sub(1));
    }

    pub fn switch_screen(&mut self, alt: bool) {
        if alt != self.alt_screen {
            std::mem::swap(&mut self.cells, &mut self.alt_grid);
            std::mem::swap(&mut self.alt_cursor, &mut (self.cursor_row, self.cursor_col));
            self.alt_screen = alt;
        }
    }

    pub fn row_text(&self, row: usize) -> String {
        if row >= self.rows { return String::new(); }
        self.cells[row].iter().map(|c| c.ch).collect()
    }
}

impl Index<usize> for Grid {
    type Output = [Cell];
    fn index(&self, row: usize) -> &[Cell] {
        &self.cells[row]
    }
}

impl IndexMut<usize> for Grid {
    fn index_mut(&mut self, row: usize) -> &mut [Cell] {
        &mut self.cells[row]
    }
}