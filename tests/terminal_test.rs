//! Terminal Grid reflow and wrapping tests
//!
//! Run: cargo test --test terminal_test --release

use open_zoo::terminal::grid::{Cell, Grid};

/// Simulate typing characters into the grid
fn type_text(g: &mut Grid, text: &str) {
    for ch in text.chars() {
        g.put_char(ch);
    }
}

/// Simulate pressing Enter
fn press_enter(g: &mut Grid) {
    g.carriage_return();
    g.line_feed();
}

/// Get the text content of a row, trimming trailing spaces and nulls
fn row_text(g: &Grid, row: usize) -> String {
    if row >= g.cells.len() {
        return String::new();
    }
    g.cells[row].iter().map(|c| c.ch).collect::<String>()
        .trim_end_matches(|c| c == ' ' || c == '\0')
        .to_string()
}

/// Get all rows with content
fn dump_grid(g: &Grid) -> Vec<String> {
    let mut result = Vec::new();
    for r in 0..g.rows {
        let text = row_text(g, r);
        let trimmed = text.trim_end_matches(|c| c == ' ' || c == '\0');
        if !trimmed.is_empty() || (r < g.wrapped.len() && g.wrapped[r]) {
            result.push(format!("[{}] w={} len={} '{}'", r, 
                if r < g.wrapped.len() { g.wrapped[r] as u8 } else { 0 },
                text.trim_end_matches('\0').len(),
                text.trim_end_matches(|c| c == ' ' || c == '\0')));
        } else if !text.trim_end_matches('\0').is_empty() {
            result.push(format!("[{}] len={} '{}'", r, text.trim_end_matches('\0').len(), text.trim_end_matches('\0')));
        }
    }
    result
}

#[test]
fn test_typing_100_chars() {
    let mut g = Grid::new(80, 24);
    assert_eq!(g.cols, 80);
    assert_eq!(g.rows, 24);

    type_text(&mut g, &"a".repeat(100));

    // Debug: print grid state
    eprintln!("Grid cols={}, rows={}, cursor=({},{}), wrapped[0]={}, wrapped[1]={}",
        g.cols, g.rows, g.cursor_col, g.cursor_row,
        g.wrapped.get(0).copied().unwrap_or(false),
        g.wrapped.get(1).copied().unwrap_or(false));
    eprintln!("Row 0: len={} '{}'", g.cells[0].len(), row_text(&g, 0));
    eprintln!("Row 1: len={} '{}'", g.cells[1].len(), row_text(&g, 1));

    // After 80 chars, auto-wrap should occur
    // Row 0: 80 chars, wrapped=false
    // Row 1: 20 chars, wrapped=true

    let r0 = row_text(&g, 0).trim_end_matches('\0').to_string();
    let r1 = row_text(&g, 1).trim_end_matches('\0').to_string();

    assert_eq!(r0.len(), 80, "Row 0 should have 80 chars, got {}: '{}'", r0.len(), r0);
    assert_eq!(r1.len(), 20, "Row 1 should have 20 chars, got {}: '{}'", r1.len(), r1);
    assert!(g.wrapped[1], "Row 1 should be wrapped (continuation from row 0)");

    // Verify content is correct
    for (i, ch) in r0.chars().enumerate() {
        assert_eq!(ch, 'a', "Row 0, col {} should be 'a', got '{}'", i, ch);
    }
    for (i, ch) in r1.chars().enumerate() {
        assert_eq!(ch, 'a', "Row 1, col {} should be 'a', got '{}'", i, ch);
    }
}

#[test]
fn test_resize_narrow_100_chars() {
    let mut g = Grid::new(80, 24);
    type_text(&mut g, &"a".repeat(100));

    // Narrow to 50 cols
    g.resize(50, 24);

    // After reflow at 50 cols:
    // Row 0: 50 chars, wrapped=false (first row)
    // Row 1: 50 chars, wrapped=true (continuation)
    // Row 2: 0 chars (empty, the 100 chars fit in 2 rows of 50)

    let r0 = row_text(&g, 0).trim_end_matches('\0').to_string();
    let r1 = row_text(&g, 1).trim_end_matches('\0').to_string();
    let r2 = row_text(&g, 2).trim_end_matches('\0').to_string();

    // 100 chars should fit in 2 rows of 50
    assert_eq!(r0.len(), 50, "Row 0 should have 50 chars, got {}: '{}'", r0.len(), &r0[..r0.len().min(50)]);
    assert_eq!(r1.len(), 50, "Row 1 should have 50 chars, got {}: '{}'", r1.len(), &r1[..r1.len().min(50)]);
    assert!(r2.is_empty() || r2.trim().is_empty(), "Row 2 should be empty");

    // Check wrapped flags
    assert!(!g.wrapped[0], "Row 0 should NOT be wrapped (first row)");
    assert!(g.wrapped[1], "Row 1 should be wrapped (continuation)");

    // Total content should still be 100 chars
    let total = r0.len() + r1.len();
    assert_eq!(total, 100, "Total chars after reflow should be 100, got {}", total);
}

#[test]
fn test_resize_widen_100_chars() {
    let mut g = Grid::new(50, 24);
    type_text(&mut g, &"a".repeat(100));

    // 100 chars at 50 cols: 2 rows of 50
    assert_eq!(row_text(&g, 0).trim_end_matches('\0').len(), 50);
    assert_eq!(row_text(&g, 1).trim_end_matches('\0').len(), 50);

    // Widen to 80 cols
    g.resize(80, 24);

    // After reflow at 80 cols: 100 chars should fit in 2 rows (80 + 20)
    let r0 = row_text(&g, 0).trim_end_matches('\0').to_string();
    let r1 = row_text(&g, 1).trim_end_matches('\0').to_string();

    assert_eq!(r0.len(), 80, "Row 0 should have 80 chars, got {}", r0.len());
    assert_eq!(r1.len(), 20, "Row 1 should have 20 chars, got {}", r1.len());
    assert!(g.wrapped[1], "Row 1 should be wrapped");

    let total = r0.len() + r1.len();
    assert_eq!(total, 100, "Total chars after widening should be 100, got {}", total);
}

#[test]
fn test_resize_back_and_forth() {
    // Start at 80 cols, type 100 chars
    let mut g = Grid::new(80, 24);
    type_text(&mut g, &"a".repeat(100));

    // Narrow to 40
    g.resize(40, 24);
    let r0 = row_text(&g, 0).trim_end_matches('\0').len();
    let r1 = row_text(&g, 1).trim_end_matches('\0').len();
    let r2 = row_text(&g, 2).trim_end_matches('\0').len();
    assert_eq!(r0 + r1 + r2, 100, "After narrowing to 40, total should be 100, got {}", r0 + r1 + r2);

    // Widen to 60
    g.resize(60, 24);
    let r0 = row_text(&g, 0).trim_end_matches('\0').len();
    let r1 = row_text(&g, 1).trim_end_matches('\0').len();
    assert_eq!(r0 + r1, 100, "After widening to 60, total should be 100, got {}", r0 + r1);

    // Narrow to 30
    g.resize(30, 24);
    let r0 = row_text(&g, 0).trim_end_matches('\0').len();
    let r1 = row_text(&g, 1).trim_end_matches('\0').len();
    let r2 = row_text(&g, 2).trim_end_matches('\0').len();
    let r3 = row_text(&g, 3).trim_end_matches('\0').len();
    assert_eq!(r0 + r1 + r2 + r3, 100, "After narrowing to 30, total should be 100, got {}", r0 + r1 + r2 + r3);

    // Widen back to 80
    g.resize(80, 24);
    let r0 = row_text(&g, 0).trim_end_matches('\0').len();
    let r1 = row_text(&g, 1).trim_end_matches('\0').len();
    assert_eq!(r0 + r1, 100, "After widening back to 80, total should be 100, got {}", r0 + r1);
}

#[test]
fn test_resize_narrow_then_wide_multiple_times() {
    let mut g = Grid::new(80, 24);
    type_text(&mut g, &"a".repeat(100));

    // Simulate rapid resize: 80 -> 50 -> 80 -> 50 -> 80
    let expected = 100;
    for _ in 0..3 {
        g.resize(50, 24);
        let total: usize = (0..g.rows).map(|r| row_text(&g, r).trim_end_matches('\0').len()).sum();
        assert_eq!(total, expected, "After narrow, total should be {}, got {}", expected, total);

        g.resize(80, 24);
        let total: usize = (0..g.rows).map(|r| row_text(&g, r).trim_end_matches('\0').len()).sum();
        assert_eq!(total, expected, "After wide, total should be {}, got {}", expected, total);
    }
}

#[test]
fn test_resize_with_echoed_output() {
    // Simulate: shell outputs prompt + user types 100 chars + shell echoes back
    let mut g = Grid::new(80, 24);

    // Simulate shell prompt: "$ "
    prompt(&mut g);

    // User types 100 chars, shell echoes them
    type_text(&mut g, &"b".repeat(100));

    // After 100 chars at 80 cols: row 0 has prompt + 78 chars, row 1 wraps
    // Prompt is 2 chars, so 100+2 = 102 chars
    // At 80 cols: row 0 = 80, row 1 = 22
    let total: usize = (0..g.rows).map(|r| row_text(&g, r).trim_end_matches('\0').len()).sum();
    assert_eq!(total, 102, "Total should be 102 (prompt + 100 chars), got {}", total);

    // Narrow to 50 cols
    g.resize(50, 24);
    let total: usize = (0..g.rows).map(|r| row_text(&g, r).trim_end_matches('\0').len()).sum();
    assert_eq!(total, 102, "After narrow, total should still be 102, got {}", total);

    // Verify first row starts with "$ "
    let r0 = row_text(&g, 0);
    assert!(r0.starts_with("$ "), "Row 0 should start with '$ ', got '{}'", &r0[..r0.len().min(5)]);
}

fn prompt(g: &mut Grid) {
    g.put_char('$');
    g.put_char(' ');
}

#[test]
fn test_backspace_after_wrap() {
    let mut g = Grid::new(80, 24);
    prompt(&mut g);
    type_text(&mut g, &"c".repeat(80));

    // After 80 chars + prompt (2 chars) = 82 chars at 80 cols
    // Row 0: prompt + 78 chars (80 total)
    // Row 1: 2 chars (wrapped=true)
    let r1 = row_text(&g, 1).trim_end_matches('\0').len();
    assert_eq!(r1, 2, "Row 1 should have 2 chars, got {}", r1);

    // Backspace: remove last char
    g.backspace();

    // After backspace, cursor should be at correct position
    // 82 - 1 = 81 chars. At 80 cols: row 0 = 80, row 1 = 1
    let r0 = row_text(&g, 0).trim_end_matches('\0').len();
    let r1 = row_text(&g, 1).trim_end_matches('\0').len();
    assert_eq!(r0, 80, "Row 0 should have 80 chars, got {}", r0);
    assert_eq!(r1, 1, "Row 1 should have 1 char, got {}", r1);
}

#[test]
fn test_carriage_return_to_wrapped() {
    let mut g = Grid::new(80, 24);
    prompt(&mut g);
    type_text(&mut g, &"d".repeat(80));

    // Cursor is on row 1 (wrapped). Send CR.
    g.carriage_return();

    // CR should go to column 0 of the current row (standard behavior)
    assert_eq!(g.cursor_col, 0, "CR should set cursor_col to 0");
    // The cursor stays on the current row (row 1)
    assert_eq!(g.cursor_row, 1, "CR should NOT change cursor_row");
}

#[test]
fn test_rapid_resize_cycle() {
    // Simulate rapid resize during window drag: 80 -> 70 -> 60 -> 50 -> 60 -> 70 -> 80
    let mut g = Grid::new(80, 24);
    type_text(&mut g, &"e".repeat(100));

    // Apply resize cycle
    let sizes = [70, 60, 50, 60, 70, 80, 50, 80, 40, 80];
    for &s in &sizes {
        g.resize(s, 24);
        let total: usize = (0..g.rows).map(|r| row_text(&g, r).trim_end_matches('\0').len()).sum();
        assert_eq!(total, 100, "After resize to {} cols, total should be 100, got {}", s, total);
    }
}

#[test]
fn test_reflow_multiline_output() {
    // Simulate command output: multiple lines of varying lengths
    let mut g = Grid::new(80, 24);

    // Line 1: 60 chars, no wrap
    type_text(&mut g, &"line1_".repeat(10)); // 60 chars
    press_enter(&mut g);

    // Line 2: 120 chars, wraps at 80
    type_text(&mut g, &"line2_".repeat(20)); // 120 chars
    press_enter(&mut g);

    // Line 3: 30 chars, no wrap
    type_text(&mut g, &"line3_".repeat(5)); // 30 chars
    press_enter(&mut g);

    // Total content:
    //   line1: 60 chars
    //   line2: 120 chars (wraps at 80: 80 + 40)
    //   line3: 30 chars
    //   = 210
    let total: usize = (0..g.rows).map(|r| row_text(&g, r).len()).sum();
    assert_eq!(total, 210, "Total should be 210 (60+120+30), got {}", total);

    // Narrow to 50 cols
    g.resize(50, 24);
    let total: usize = (0..g.rows).map(|r| row_text(&g, r).len()).sum();
    assert_eq!(total, 210, "After narrow, total should be 210, got {}", total);

    // Widen to 80 cols
    g.resize(80, 24);
    let total: usize = (0..g.rows).map(|r| row_text(&g, r).len()).sum();
    assert_eq!(total, 210, "After widen, total should be 210, got {}", total);
}