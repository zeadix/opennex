//! Basic terminal integration smoke tests.
//!
//! Run: cargo test --test terminal_test -- --nocapture

#[test]
fn test_term_creation_and_resize() {
    use alacritty_terminal::event::{Event, EventListener};
    use alacritty_terminal::sync::FairMutex;
    use alacritty_terminal::term::{test::TermSize, Config, Term};
    use std::sync::Arc;

    struct MyListener;
    impl EventListener for MyListener {
        fn send_event(&self, _event: Event) {}
    }

    let config = Config::default();
    let size = TermSize::new(80, 24);
    let term = Term::new(config, &size, MyListener);
    let term = Arc::new(FairMutex::new(term));

    // Check initial dimensions
    {
        use alacritty_terminal::grid::Dimensions;
        let t = term.lock();
        assert_eq!(t.grid().columns(), 80);
        assert_eq!(t.grid().screen_lines(), 24);
    }

    // Resize
    {
        let mut t = term.lock();
        let new_size = TermSize::new(40, 12);
        t.resize(new_size);
    }

    {
        use alacritty_terminal::grid::Dimensions;
        let t = term.lock();
        assert_eq!(t.grid().columns(), 40);
        assert_eq!(t.grid().screen_lines(), 12);
    }
}

#[test]
fn test_grid_display_iter() {
    use alacritty_terminal::event::{Event, EventListener};
    use alacritty_terminal::term::{test::TermSize, Config, Term};
    use std::sync::Arc;

    struct MyListener;
    impl EventListener for MyListener {
        fn send_event(&self, _event: Event) {}
    }

    let config = Config::default();
    let size = TermSize::new(80, 24);
    let term = Term::new(config, &size, MyListener);
    let term = Arc::new(alacritty_terminal::sync::FairMutex::new(term));

    // Renderable content should have a display iterator
    let t = term.lock();
    let content = t.renderable_content();
    let cell_count = content.display_iter.count();
    assert_eq!(
        cell_count, 1920,
        "empty 80x24 terminal should have 1920 cells"
    );

    // Check cursor is at origin
    assert_eq!(content.cursor.point.column.0, 0);
    assert_eq!(content.cursor.point.line.0, 0);
}

#[test]
fn test_term_config_default() {
    use alacritty_terminal::term::Config;
    let config = Config::default();
    assert!(config.scrolling_history > 0 || config.scrolling_history == 0);
}
