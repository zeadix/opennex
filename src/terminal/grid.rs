use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Debug, Default)]
pub struct CellFlags {
    pub bold: bool, pub italic: bool, pub underline: bool,
    pub inverse: bool, pub dim: bool,
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
    pub fn is_empty(&self) -> bool { self.ch == ' ' || self.ch == '\0' }
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
    pub wrapped: Vec<bool>,  // wrapped[row] = true: row is continuation from previous
}

impl Grid {
    pub fn new(cols: usize, rows: usize) -> Self {
        Grid {
            cells: vec![vec![Cell::empty(); cols]; rows],
            cols, rows,
            cursor_col: 0, cursor_row: 0,
            scroll_top: 0, scroll_bottom: rows.saturating_sub(1),
            current_fg: [255, 255, 255], current_bg: [0, 0, 0],
            current_flags: CellFlags::default(),
            cursor_visible: true,
            alt_screen: false,
            alt_grid: vec![vec![Cell::empty(); cols]; rows],
            alt_cursor: (0, 0),
            wrapped: vec![false; rows],
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        if cols == self.cols && rows == self.rows { return; }
        let old_cols = self.cols;

        // --- COLUMN REFLOW ---
        if cols != old_cols {
            let mut new_cells: Vec<Vec<Cell>> = Vec::new();
            let mut new_wrapped: Vec<bool> = Vec::new();
            let mut had_accumulator = false;

            // Find the last row that contains any non-empty cell.
            // Empty rows (pure spaces/nulls) are just padding and must not
            // be reflowed — they would corrupt the content stream with
            // interleaved spaces.
            let last_content_row = self.cells
                .iter()
                .enumerate()
                .rev()
                .find(|(_, row)| row.iter().any(|c| !c.is_empty()))
                .map(|(i, _)| i + 1)      // 1 past the last content row
                .unwrap_or(0);             // 0 means no content at all

            let mut accumulator: Vec<Cell> = Vec::new();

            for r in 0..last_content_row {
                // Drain only up to the last non-empty column in this row.
                // Trailing empty cells (spaces/nulls) are padding and must not
                // be reflowed — they would inflate the content stream.
                let last_col = self.cells[r]
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, c)| !c.is_empty())
                    .map(|(i, _)| i + 1)
                    .unwrap_or(0);

                // Accumulate content from this row (only non-empty part)
                accumulator.extend(self.cells[r].drain(..last_col));

                // Flush complete rows
                while accumulator.len() >= cols {
                    let chunk: Vec<Cell> = accumulator.drain(..cols).collect();
                    new_cells.push(chunk);
                    // wrapped = true means this row is a continuation of the
                    // logical line from the previous row.  The accumulator
                    // is non-empty only if content extends beyond this chunk.
                    new_wrapped.push(had_accumulator);
                    had_accumulator = accumulator.len() > 0;
                }
            }

            // Handle remaining incomplete row
            if !accumulator.is_empty() {
                let mut row = accumulator;
                row.resize(cols, Cell::empty());
                new_cells.push(row);
                new_wrapped.push(had_accumulator);
            }

            // Fallback: ensure at least one row exists so wrapped flags match.
            if new_cells.is_empty() {
                new_cells.push(vec![Cell::empty(); cols]);
                new_wrapped.push(false);
            }

            self.cells = new_cells;
            self.wrapped = new_wrapped;
        }

        // --- ROW COUNT ---
        if self.cells.len() != rows {
            if self.cells.len() > rows {
                self.cells.truncate(rows);
                self.wrapped.truncate(rows);
            } else {
                while self.cells.len() < rows {
                    self.cells.push(vec![Cell::empty(); cols]);
                    self.wrapped.push(false);
                }
            }
            // Also handle alt grid
            if self.alt_grid.len() != rows {
                if self.alt_grid.len() > rows {
                    self.alt_grid.truncate(rows);
                } else {
                    while self.alt_grid.len() < rows {
                        self.alt_grid.push(vec![Cell::empty(); cols]);
                    }
                }
            }
        }

        // Ensure all rows have correct width (may have been resized above)
        for row in &mut self.cells {
            if row.len() != cols {
                if row.len() > cols { row.truncate(cols); }
                else { row.resize(cols, Cell::empty()); }
            }
        }
        for row in &mut self.alt_grid {
            if row.len() != cols {
                if row.len() > cols { row.truncate(cols); }
                else { row.resize(cols, Cell::empty()); }
            }
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
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.line_feed();
            if self.cursor_row < self.wrapped.len() {
                self.wrapped[self.cursor_row] = true;
            }
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
        // Move to the first non-wrapped row (start of the logical line)
        while self.cursor_row > 0 && self.wrapped.get(self.cursor_row).copied().unwrap_or(false) {
            self.cursor_row -= 1;
        }
    }

    pub fn line_feed(&mut self) {
        if self.cursor_row == self.scroll_bottom {
            self.scroll_up(1);
        } else if self.cursor_row < self.rows.saturating_sub(1) {
            self.cursor_row += 1;
        }
        if self.cursor_row < self.wrapped.len() {
            self.wrapped[self.cursor_row] = false;
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            // Clear the cell at the new cursor position
            if let Some(cell) = self.current_cell_mut() {
                *cell = Cell::empty();
            }
        } else if self.cursor_row > 0 {
            // Move to end of previous row
            self.cursor_row -= 1;
            self.cursor_col = self.cols.saturating_sub(1);
            // Clear the cell at the new cursor position
            if let Some(cell) = self.current_cell_mut() {
                *cell = Cell::empty();
            }
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
                self.wrapped.remove(top);
                self.cells.insert(bottom, vec![Cell::empty(); self.cols]);
                self.wrapped.insert(bottom, false);
            }
        }
    }

    pub fn scroll_down(&mut self, n: usize) {
        let top = self.scroll_top;
        let bottom = self.scroll_bottom;
        if bottom < self.rows {
            for _ in 0..n {
                self.cells.remove(bottom);
                self.wrapped.remove(bottom);
                self.cells.insert(top, vec![Cell::empty(); self.cols]);
                self.wrapped.insert(top, false);
            }
        }
    }

    pub fn clear_screen(&mut self, mode: u16) {
        match mode {
            0 => {
                for col in self.cursor_col..self.cols {
                    if let Some(c) = self.cells.get_mut(self.cursor_row).and_then(|r| r.get_mut(col)) {
                        *c = Cell::empty();
                    }
                }
                for row in (self.cursor_row + 1)..self.rows {
                    for col in 0..self.cols { self.cells[row][col] = Cell::empty(); }
                    if row < self.wrapped.len() { self.wrapped[row] = false; }
                }
            }
            1 => {
                for row in 0..self.cursor_row {
                    for col in 0..self.cols { self.cells[row][col] = Cell::empty(); }
                    if row < self.wrapped.len() { self.wrapped[row] = false; }
                }
                for col in 0..=self.cursor_col {
                    if let Some(c) = self.cells.get_mut(self.cursor_row).and_then(|r| r.get_mut(col)) {
                        *c = Cell::empty();
                    }
                }
            }
            2 | 3 => {
                for row in 0..self.rows {
                    for col in 0..self.cols { self.cells[row][col] = Cell::empty(); }
                }
                for w in &mut self.wrapped { *w = false; }
                self.cursor_col = 0; self.cursor_row = 0;
            }
            _ => {}
        }
    }

    pub fn clear_line(&mut self, mode: u16) {
        match mode {
            0 => {
                for col in self.cursor_col..self.cols {
                    if let Some(r) = self.cells.get_mut(self.cursor_row) { r[col] = Cell::empty(); }
                }
            }
            1 => {
                for col in 0..=self.cursor_col.min(self.cols.saturating_sub(1)) {
                    if let Some(r) = self.cells.get_mut(self.cursor_row) { r[col] = Cell::empty(); }
                }
            }
            2 => {
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

    pub fn cursor_up(&mut self, n: usize) { self.cursor_row = self.cursor_row.saturating_sub(n); }
    pub fn cursor_down(&mut self, n: usize) { self.cursor_row = (self.cursor_row + n).min(self.rows.saturating_sub(1)); }
    pub fn cursor_forward(&mut self, n: usize) { self.cursor_col = (self.cursor_col + n).min(self.cols.saturating_sub(1)); }
    pub fn cursor_back(&mut self, n: usize) { self.cursor_col = self.cursor_col.saturating_sub(n); }

    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        self.scroll_top = top.min(self.rows.saturating_sub(1));
        self.scroll_bottom = bottom.min(self.rows.saturating_sub(1)).max(self.scroll_top);
    }

    pub fn save_cursor(&mut self) { self.alt_cursor = (self.cursor_row, self.cursor_col); }
    pub fn restore_cursor(&mut self) {
        self.cursor_row = self.alt_cursor.0.min(self.rows.saturating_sub(1));
        self.cursor_col = self.alt_cursor.1.min(self.cols.saturating_sub(1));
    }

    pub fn switch_screen(&mut self, alt: bool) {
        if alt != self.alt_screen {
            std::mem::swap(&mut self.cells, &mut self.alt_grid);
            std::mem::swap(&mut self.alt_cursor, &mut (self.cursor_row, self.cursor_col));
            if alt {
                // Switching to alt screen: wrapped flags reset
                self.wrapped = vec![false; self.rows];
            }
            self.alt_screen = alt;
        }
    }

    pub fn row_text(&self, row: usize) -> String {
        if row >= self.cells.len() { return String::new(); }
        self.cells[row].iter().map(|c| c.ch).collect()
    }
}

impl Index<usize> for Grid {
    type Output = [Cell];
    fn index(&self, row: usize) -> &[Cell] { &self.cells[row] }
}

impl IndexMut<usize> for Grid {
    fn index_mut(&mut self, row: usize) -> &mut [Cell] { &mut self.cells[row] }
}