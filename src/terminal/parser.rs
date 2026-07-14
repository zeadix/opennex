use crate::terminal::grid::{Cell, CellFlags, Grid};
use std::io::Write;
use std::sync::{Arc, Mutex};
use vte::{Params, Perform};

pub struct TerminalHandler {
    grid: Arc<Mutex<Grid>>,
    writer: Box<dyn Write + Send>,
}

impl TerminalHandler {
    pub fn new(grid: Arc<Mutex<Grid>>, writer: Box<dyn Write + Send>) -> Self {
        TerminalHandler { grid, writer }
    }

    fn param(params: &Params, idx: usize, default: u16) -> u16 {
        params.into_iter().nth(idx).and_then(|p| p.first().copied()).unwrap_or(default)
    }

    fn color_from_code(code: u16) -> [u8; 3] {
        match code {
            0 => [0, 0, 0],       // Black
            1 => [170, 0, 0],      // Red
            2 => [0, 170, 0],      // Green
            3 => [170, 85, 0],     // Yellow
            4 => [0, 0, 170],      // Blue
            5 => [170, 0, 170],    // Magenta
            6 => [0, 170, 170],    // Cyan
            7 => [170, 170, 170],  // White
            8 => [85, 85, 85],    // Bright Black
            9 => [255, 85, 85],    // Bright Red
            10 => [85, 255, 85],   // Bright Green
            11 => [255, 255, 85],  // Bright Yellow
            12 => [85, 85, 255],   // Bright Blue
            13 => [255, 85, 255],  // Bright Magenta
            14 => [85, 255, 255],  // Bright Cyan
            15 => [255, 255, 255], // Bright White
            _ => [255, 255, 255],
        }
    }

    fn set_sgr(&mut self, params: &Params) {
        let mut iter = params.into_iter();
        while let Some(p) = iter.next() {
            let code = p.first().copied().unwrap_or(0);
            match code {
                0 => {
                    if let Ok(g) = self.grid.lock() {
                        let (fg, bg, flags) = (g.current_fg, g.current_bg, CellFlags::default());
                        drop(g);
                        if let Ok(mut g) = self.grid.lock() {
                            g.current_fg = [255, 255, 255];
                            g.current_bg = [0, 0, 0];
                            g.current_flags = CellFlags::default();
                        }
                    }
                }
                1 => { if let Ok(mut g) = self.grid.lock() { g.current_flags.bold = true; } }
                2 => { if let Ok(mut g) = self.grid.lock() { g.current_flags.dim = true; } }
                3 => { if let Ok(mut g) = self.grid.lock() { g.current_flags.italic = true; } }
                4 => { if let Ok(mut g) = self.grid.lock() { g.current_flags.underline = true; } }
                7 => { if let Ok(mut g) = self.grid.lock() { g.current_flags.inverse = true; } }
                22 => { if let Ok(mut g) = self.grid.lock() { g.current_flags.bold = false; g.current_flags.dim = false; } }
                23 => { if let Ok(mut g) = self.grid.lock() { g.current_flags.italic = false; } }
                24 => { if let Ok(mut g) = self.grid.lock() { g.current_flags.underline = false; } }
                27 => { if let Ok(mut g) = self.grid.lock() { g.current_flags.inverse = false; } }
                30..=37 => {
                    let color = Self::color_from_code(code - 30);
                    if let Ok(mut g) = self.grid.lock() { g.current_fg = color; }
                }
                38 => {
                    // Extended color: 38;5;N (256) or 38;2;R;G;B (truecolor)
                    if let Some(p2) = iter.next() {
                        let mode = p2.first().copied().unwrap_or(0);
                        match mode {
                            5 => {
                                if let Some(p3) = iter.next() {
                                    let idx = p3.first().copied().unwrap_or(0);
                                    let color = color_256(idx);
                                    if let Ok(mut g) = self.grid.lock() { g.current_fg = color; }
                                }
                            }
                            2 => {
                                let r = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                let g_ = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                let b = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                if let Ok(mut g) = self.grid.lock() { g.current_fg = [r, g_, b]; }
                            }
                            _ => {}
                        }
                    }
                }
                39 => { if let Ok(mut g) = self.grid.lock() { g.current_fg = [255, 255, 255]; } }
                40..=47 => {
                    let color = Self::color_from_code(code - 40);
                    if let Ok(mut g) = self.grid.lock() { g.current_bg = color; }
                }
                48 => {
                    if let Some(p2) = iter.next() {
                        let mode = p2.first().copied().unwrap_or(0);
                        match mode {
                            5 => {
                                if let Some(p3) = iter.next() {
                                    let idx = p3.first().copied().unwrap_or(0);
                                    let color = color_256(idx);
                                    if let Ok(mut g) = self.grid.lock() { g.current_bg = color; }
                                }
                            }
                            2 => {
                                let r = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                let g_ = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                let b = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                if let Ok(mut g) = self.grid.lock() { g.current_bg = [r, g_, b]; }
                            }
                            _ => {}
                        }
                    }
                }
                49 => { if let Ok(mut g) = self.grid.lock() { g.current_bg = [0, 0, 0]; } }
                90..=97 => {
                    let color = Self::color_from_code(code - 90 + 8);
                    if let Ok(mut g) = self.grid.lock() { g.current_fg = color; }
                }
                100..=107 => {
                    let color = Self::color_from_code(code - 100 + 8);
                    if let Ok(mut g) = self.grid.lock() { g.current_bg = color; }
                }
                _ => {}
            }
        }
    }
}

fn color_256(idx: u16) -> [u8; 3] {
    if idx < 16 {
        match idx {
            0 => [0, 0, 0], 1 => [170, 0, 0], 2 => [0, 170, 0], 3 => [170, 85, 0],
            4 => [0, 0, 170], 5 => [170, 0, 170], 6 => [0, 170, 170], 7 => [170, 170, 170],
            8 => [85, 85, 85], 9 => [255, 85, 85], 10 => [85, 255, 85], 11 => [255, 255, 85],
            12 => [85, 85, 255], 13 => [255, 85, 255], 14 => [85, 255, 255], 15 => [255, 255, 255],
            _ => [255, 255, 255],
        }
    } else if idx >= 232 {
        let v = 8 + (idx - 232) * 10;
        [v as u8, v as u8, v as u8]
    } else {
        let idx = idx - 16;
        let r = (idx / 36) % 6;
        let g = (idx / 6) % 6;
        let b = idx % 6;
        let to_u8 = |v: u16| if v == 0 { 0u8 } else { (55 + v * 40) as u8 };
        [to_u8(r), to_u8(g), to_u8(b)]
    }
}

impl Perform for TerminalHandler {
    fn print(&mut self, c: char) {
        if let Ok(mut g) = self.grid.lock() {
            g.put_char(c);
        }
    }

    fn execute(&mut self, byte: u8) {
        if let Ok(mut g) = self.grid.lock() {
            match byte {
                b'\r' => g.carriage_return(),
                b'\n' => g.line_feed(),
                b'\x08' | b'\x7f' => g.backspace(),
                b'\x07' => {} // Bell
                b'\t' => g.tab(),
                _ => {}
            }
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        if let Ok(mut g) = self.grid.lock() {
            match action {
                'A' => g.cursor_up(Self::param(params, 0, 1) as usize),
                'B' => g.cursor_down(Self::param(params, 0, 1) as usize),
                'C' => g.cursor_forward(Self::param(params, 0, 1) as usize),
                'D' => g.cursor_back(Self::param(params, 0, 1) as usize),
                'E' => { g.cursor_row = (g.cursor_row + Self::param(params, 0, 1) as usize).min(g.rows.saturating_sub(1)); g.cursor_col = 0; }
                'F' => { g.cursor_row = g.cursor_row.saturating_sub(Self::param(params, 0, 1) as usize); g.cursor_col = 0; }
                'G' | '`' => g.cursor_col = Self::param(params, 0, 1).saturating_sub(1) as usize,
                'd' => g.cursor_row = Self::param(params, 0, 1).saturating_sub(1) as usize,
                'H' | 'f' => {
                    let row = Self::param(params, 0, 1).saturating_sub(1) as usize;
                    let col = Self::param(params, 1, 1).saturating_sub(1) as usize;
                    g.move_cursor(row, col);
                }
                'J' => g.clear_screen(Self::param(params, 0, 0)),
                'K' => g.clear_line(Self::param(params, 0, 0)),
                'S' => g.scroll_up(Self::param(params, 0, 1) as usize),
                'T' => g.scroll_down(Self::param(params, 0, 1) as usize),
                'r' => {
                    let top = Self::param(params, 0, 1).saturating_sub(1) as usize;
                    let bottom = Self::param(params, 1, g.rows as u16) as usize;
                    g.set_scroll_region(top, bottom.saturating_sub(1));
                    g.move_cursor(0, 0);
                }
                'm' => drop(g),
                's' => g.save_cursor(),
                'u' => g.restore_cursor(),
                '?' => {} // Private modes handled below
                _ => {}
            }
        }
        if action == 'm' {
            self.set_sgr(params);
        }
        // Handle private mode sequences (? h / ? l)
        if action == 'h' || action == 'l' {
            let set = action == 'h';
            for p in params.into_iter() {
                let code = p.first().copied().unwrap_or(0);
                match code {
                    25 => { if let Ok(mut g) = self.grid.lock() { g.cursor_visible = set; } }
                    47 | 1047 | 1049 => { if let Ok(mut g) = self.grid.lock() { g.switch_screen(set); } }
                    _ => {}
                }
            }
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        // Handle window title (OSC 0 / OSC 2)
        if let Some(param) = params.first() {
            if !param.is_empty() {
                let _ = param; // Could store title if needed
            }
        }
    }
}