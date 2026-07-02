use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: String,
    pub title: String,
    pub terminal_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabGroup {
    pub id: String,
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
}

impl TabGroup {
    pub fn new(id: &str) -> Self {
        TabGroup {
            id: id.to_string(),
            tabs: Vec::new(),
            active_tab: 0,
        }
    }

    pub fn add_tab(&mut self, tab: Tab) {
        self.tabs.push(tab);
    }

    pub fn remove_tab(&mut self, tab_id: &str) -> Option<Tab> {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == tab_id) {
            let tab = self.tabs.remove(pos);
            if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
                self.active_tab = self.tabs.len() - 1;
            }
            Some(tab)
        } else {
            None
        }
    }

    pub fn switch_tab(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.active_tab = index;
            true
        } else {
            false
        }
    }

    pub fn get_active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab)
    }

    pub fn rename_tab(&mut self, tab_id: &str, new_title: &str) -> bool {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
            tab.title = new_title.to_string();
            true
        } else {
            false
        }
    }
}

impl Default for TabGroup {
    fn default() -> Self {
        let mut group = TabGroup::new("group-1");
        group.add_tab(Tab {
            id: "tab-1".to_string(),
            title: "终端 1".to_string(),
            terminal_id: "terminal-1".to_string(),
        });
        group
    }
}
