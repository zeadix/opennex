use crate::terminal::Tab;
use crossterm::event::{KeyCode, KeyEvent};

pub struct App {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub tab_counter: u32,
}

impl App {
    pub fn new() -> Self {
        let mut app = App {
            tabs: Vec::new(),
            active_tab: 0,
            tab_counter: 0,
        };
        app.new_tab();
        app
    }

    pub fn new_tab(&mut self) {
        self.tab_counter += 1;
        self.tabs.push(Tab::new(&format!("Tab {}", self.tab_counter)));
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    pub fn split_horizontal(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.split_horizontal();
        }
    }

    pub fn split_vertical(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.split_vertical();
        }
    }

    pub fn next_pane(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if tab.panes.len() > 1 {
                tab.active_pane = (tab.active_pane + 1) % tab.panes.len();
            }
        }
    }

    pub fn prev_pane(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if tab.panes.len() > 1 {
                tab.active_pane = if tab.active_pane == 0 {
                    tab.panes.len() - 1
                } else {
                    tab.active_pane - 1
                };
            }
        }
    }

    pub fn select_pane(&mut self, index: usize) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if index < tab.panes.len() {
                tab.active_pane = index;
            }
        }
    }

    pub fn clear_active(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if let Some(pane) = tab.panes.get_mut(tab.active_pane) {
                pane.content.clear();
            }
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if let Some(pane) = tab.panes.get_mut(tab.active_pane) {
                match key.code {
                    KeyCode::Enter => {
                        let cmd = pane.input.clone();
                        pane.input.clear();
                        pane.execute(&cmd);
                    }
                    KeyCode::Char(c) => {
                        pane.input.push(c);
                    }
                    KeyCode::Backspace => {
                        pane.input.pop();
                    }
                    KeyCode::Esc => {
                        pane.input.clear();
                    }
                    KeyCode::Up => {
                        if let Some(last) = pane.history.last() {
                            pane.input = last.clone();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
