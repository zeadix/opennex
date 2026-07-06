use std::collections::VecDeque;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

const MAX_HISTORY: usize = 1000;

#[derive(Serialize, Deserialize)]
struct HistoryFile {
    commands: Vec<String>,
}

pub struct HistoryTracker {
    history: VecDeque<String>,
    history_path: PathBuf,
}

impl HistoryTracker {
    pub fn load() -> Self {
        let history_path = Self::history_path();
        let history = if let Ok(content) = std::fs::read_to_string(&history_path) {
            serde_json::from_str::<HistoryFile>(&content)
                .map(|f| {
                    let mut dq: VecDeque<String> = f.commands.into();
                    while dq.len() > MAX_HISTORY {
                        dq.pop_front();
                    }
                    dq
                })
                .unwrap_or_default()
        } else {
            VecDeque::new()
        };
        Self { history, history_path }
    }

    pub fn save(&self) {
        let file = HistoryFile {
            commands: self.history.iter().cloned().collect(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&file) {
            let _ = std::fs::write(&self.history_path, json);
        }
    }

    pub fn add(&mut self, cmd: String) {
        if self.history.back() == Some(&cmd) {
            return;
        }
        self.history.push_back(cmd);
        while self.history.len() > MAX_HISTORY {
            self.history.pop_front();
        }
        self.save();
    }

    pub fn search(&self, prefix: &str) -> Vec<&str> {
        self.history
            .iter()
            .rev()
            .filter(|c| c.starts_with(prefix))
            .map(|s| s.as_str())
            .collect()
    }

    fn history_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".open_zoo")
            .join("history.json")
    }
}
