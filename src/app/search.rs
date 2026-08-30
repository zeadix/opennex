//! Terminal scrollback search: whole-history matching, navigation and
//! the floating search bar. (roadmap batch 4)

use super::*;
use alacritty_terminal::grid::{Dimensions, Grid};
use alacritty_terminal::index::Line;
use alacritty_terminal::term::cell::{self, Cell};

// ---- Terminal scrollback search (roadmap batch 4) ---------------------

/// One search hit: absolute grid line (Line(0) = top of the active
/// screen, negative = scrollback), the starting column and the number of
/// grid columns the match spans (wide chars consume two columns).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SearchHit {
    pub line: Line,
    pub col_start: usize,
    pub col_count: usize,
}

/// State of the floating search bar for ONE terminal.
pub(crate) struct TerminalSearch {
    pub tab: String,
    pub query: String,
    /// Query the current `matches` were computed against.
    pub searched: String,
    pub matches: Vec<SearchHit>,
    pub current: usize,
}

impl TerminalSearch {
    pub fn new(tab: String) -> Self {
        Self {
            tab,
            query: String::new(),
            searched: String::new(),
            matches: Vec::new(),
            current: 0,
        }
    }
}

/// Case-insensitive substring search over pre-extracted grid rows. Rows
/// come as (row index, (char, grid column) pairs); wide-char spacers are
/// already skipped by the caller, so a char's column is its cell origin.
fn hits_in_rows(
    rows: impl Iterator<Item = (i32, Vec<(char, usize)>)>,
    query_lower: &str,
) -> Vec<SearchHit> {
    let mut hits = Vec::new();
    if query_lower.is_empty() {
        return hits;
    }
    let qchars: Vec<char> = query_lower.chars().collect();
    for (row_idx, row) in rows {
        // Lowercase char stream WITH its grid column, so matches map back
        // to cell coordinates (row strings would shift on wide chars).
        let mut stream: Vec<(char, usize)> = row
            .into_iter()
            .map(|(c, col)| (c.to_lowercase().next().unwrap_or(c), col))
            .collect();
        // Trailing whitespace rarely matters visually and pollutes hits
        // ("ls" would also match the padded row tail).
        while stream.last().is_some_and(|(c, _)| c.is_whitespace()) {
            stream.pop();
        }
        if stream.len() < qchars.len() {
            continue;
        }
        let mut start = 0;
        while start + qchars.len() <= stream.len() {
            if stream[start..start + qchars.len()]
                .iter()
                .map(|(c, _)| *c)
                .eq(qchars.iter().copied())
            {
                let first = stream[start].1;
                let last = stream[start + qchars.len() - 1].1;
                // A wide char spans its own column PLUS the (skipped)
                // spacer column: the pixel width covers both.
                let hit = SearchHit {
                    line: Line(row_idx),
                    col_start: first,
                    col_count: last - first + 1,
                };
                hits.push(hit);
                start += qchars.len();
            } else {
                start += 1;
            }
        }
    }
    hits
}

/// Search the WHOLE scrollback (history + screen) of a grid. Rows are
/// extracted as (char, column) pairs; wide-char spacer cells are skipped
/// so a wide char maps to its own origin column.
fn find_search_hits(grid: &Grid<Cell>, query_lower: &str) -> Vec<SearchHit> {
    let screen = grid.screen_lines() as i32;
    let history = grid.total_lines() as i32 - screen;
    let rows = (-history..screen).map(|l| {
        let row = &grid[Line(l)];
        let filtered: Vec<(char, usize)> = (0..grid.columns())
            .map(|col| {
                let c = &row[Column(col)];
                (c.c, c.flags, col)
            })
            .filter(|(_, flags, _)| !flags.contains(cell::Flags::WIDE_CHAR_SPACER))
            .map(|(ch, _, col)| (ch, col))
            .collect();
        (l, filtered)
    });
    hits_in_rows(rows, query_lower)
}

impl App {
    /// Jump to the next/previous hit (wraps) and scroll it into view.
    fn search_navigate(&mut self, forward: bool) {
        let Some(search) = self.terminal_search.as_ref() else {
            return;
        };
        if search.matches.is_empty() {
            return;
        }
        let n = search.matches.len();
        let next = if forward {
            (search.current + 1) % n
        } else {
            (search.current + n - 1) % n
        };
        let line = search.matches[next].line;
        let tab = search.tab.clone();
        self.terminal_search.as_mut().unwrap().current = next;
        self.scroll_to_search_hit(&tab, line);
    }
    /// Scroll the terminal so `line` sits ~2 rows below the viewport top.
    fn scroll_to_search_hit(&mut self, tab: &str, line: Line) {
        let Some(td) = self.terminals.get(tab) else {
            return;
        };
        let grid = &td.instance.backend.last_content().grid;
        let history = (grid.total_lines() - grid.screen_lines()) as i32;
        let current = grid.display_offset() as i32;
        let target = (2 - line.0).clamp(0, history);
        let delta = target - current;
        if delta != 0 {
            let td = self.terminals.get_mut(tab).unwrap();
            td.instance
                .backend
                .process_command(egui_term::BackendCommand::Scroll(delta));
            td.instance.backend.set_dirty();
        }
    }
    /// Floating search bar anchored to the top-right of the searched
    /// terminal's viewport. Enter = next, Shift+Enter = previous, Esc
    /// (while the input holds focus) closes.
    pub(crate) fn render_search_bar(&mut self, ctx: &egui::Context) {
        // Recompute matches whenever the query changed (whole-scrollback
        // scan; the sampler cadence does NOT refresh it).
        if let Some(search) = self.terminal_search.as_ref() {
            if search.query != search.searched {
                let tab = search.tab.clone();
                let q = search.query.trim().to_lowercase();
                let hits = if q.is_empty() {
                    Vec::new()
                } else {
                    self.terminals
                        .get(&tab)
                        .map(|td| find_search_hits(&td.instance.backend.last_content().grid, &q))
                        .unwrap_or_default()
                };
                let search = self.terminal_search.as_mut().unwrap();
                search.matches = hits;
                search.searched = search.query.clone();
                search.current = 0;
                if let Some(first) = search.matches.first() {
                    let line = first.line;
                    let tab = search.tab.clone();
                    self.scroll_to_search_hit(&tab, line);
                }
            }
        }
        let Some(search) = self.terminal_search.as_ref() else {
            return;
        };
        let Some(rect) = self.terminal_view_rects.get(&search.tab).copied() else {
            return;
        };
        if !rect.is_positive() {
            return;
        }
        let tab = search.tab.clone();
        let hint = self.texts.terminal.term_search_hint.clone();
        let mut close = false;
        egui::Area::new(egui::Id::new(("term_search", tab.clone())))
            .order(egui::Order::Foreground)
            .current_pos(egui::pos2(rect.right() - 336.0, rect.top() + 4.0))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(ui.visuals().window_fill)
                    .stroke(ui.visuals().window_stroke)
                    .inner_margin(egui::Margin::symmetric(6, 3))
                    .corner_radius(4.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let resp = ui.add(
                                egui::TextEdit::singleline(
                                    &mut self.terminal_search.as_mut().unwrap().query,
                                )
                                .hint_text(&hint)
                                .desired_width(150.0)
                                .font(egui::FontId::monospace(12.0))
                                .id(egui::Id::new(("term_search_input", tab.clone()))),
                            );
                            resp.request_focus();
                            if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                close = true;
                            }
                            let search = self.terminal_search.as_ref().unwrap();
                            let total = search.matches.len();
                            let count = if total > 0 {
                                format!("{}/{}", search.current + 1, total)
                            } else {
                                "0/0".to_string()
                            };
                            ui.monospace(egui::RichText::new(count).size(11.0));
                            if ui
                                .button(
                                    egui::RichText::new(egui_phosphor::regular::CARET_UP)
                                        .size(11.0),
                                )
                                .clicked()
                            {
                                self.search_navigate(false);
                            }
                            if ui
                                .button(
                                    egui::RichText::new(egui_phosphor::regular::CARET_DOWN)
                                        .size(11.0),
                                )
                                .clicked()
                            {
                                self.search_navigate(true);
                            }
                            if ui
                                .button(egui::RichText::new(egui_phosphor::regular::X).size(11.0))
                                .clicked()
                            {
                                close = true;
                            }
                        });
                        // Keyboard nav: Enter / Shift+Enter (input holds
                        // focus while the bar is open — the modal arbiter
                        // keeps the terminal from reclaiming it).
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let shift = ui.input(|i| i.modifiers.shift);
                            self.search_navigate(!shift);
                        }
                    });
            });
        if close {
            self.terminal_search = None;
        }
    }
}

#[cfg(test)]
mod search_tests {
    use super::hits_in_rows;

    fn row(idx: i32, s: &str) -> (i32, Vec<(char, usize)>) {
        (idx, s.chars().enumerate().map(|(i, c)| (c, i)).collect())
    }

    #[test]
    fn finds_all_matches_case_insensitively() {
        let rows = [
            row(0, "Error: not found"),
            row(1, "no ERROR here"),
            row(2, "errors"),
        ];
        let hits = hits_in_rows(rows.into_iter(), "error");
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].line.0, 0);
        assert_eq!(hits[0].col_start, 0);
        assert_eq!(hits[0].col_count, 5);
        assert_eq!(hits[1].line.0, 1);
        assert_eq!(hits[1].col_start, 3);
    }

    #[test]
    fn empty_query_and_no_match_yield_nothing() {
        assert!(hits_in_rows([row(0, "abc")].into_iter(), "").is_empty());
        assert!(hits_in_rows([row(0, "abc")].into_iter(), "zzz").is_empty());
    }

    #[test]
    fn trailing_padding_is_not_part_of_a_match() {
        // Padded row tail: "ls        " — searching "ls" must match at 0.
        let hits = hits_in_rows([row(0, "ls       ")].into_iter(), "ls");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].col_start, 0);
        assert_eq!(hits[0].col_count, 2);
    }
}
