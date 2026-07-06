pub mod command_db;
pub mod history;
pub mod overlay;

use command_db::CommandDatabase;
use history::HistoryTracker;

pub struct CompletionEngine {
    db: CommandDatabase,
    history: HistoryTracker,
}

impl CompletionEngine {
    pub fn new() -> Self {
        Self {
            db: CommandDatabase::build(),
            history: HistoryTracker::load(),
        }
    }

    pub fn suggest(&self, input: &str) -> Vec<String> {
        if input.is_empty() {
            return Vec::new();
        }
        let mut results: Vec<String> = Vec::new();
        for cmd in self.history.search(input) {
            if !results.contains(&cmd.to_string()) {
                results.push(cmd.to_string());
            }
        }
        for cmd in self.db.search(input) {
            if !results.contains(&cmd.to_string()) {
                results.push(cmd.to_string());
            }
        }
        results.truncate(10);
        results
    }

    pub fn record_command(&mut self, cmd: &str) {
        let trimmed = cmd.trim();
        if !trimmed.is_empty() && trimmed.len() > 1 {
            self.history.add(trimmed.to_string());
        }
    }

    pub fn save_history(&self) {
        self.history.save();
    }
}
