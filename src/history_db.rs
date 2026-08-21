use rusqlite::{params, Connection};
use std::path::Path;

pub struct HistoryDb {
    conn: Connection,
    max_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::HistoryDb;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_db() -> (HistoryDb, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "opennex_history_{}-{}.db",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        (HistoryDb::new(&path, 10), path)
    }

    #[test]
    fn prune_removes_history_of_unknown_terminal_ids() {
        let (db, path) = test_db();
        db.add("terminal-1", "ls");
        db.add("terminal-2", "cd");
        // Only terminal-2 survives the scene; terminal-1's history must go.
        db.prune(&["terminal-2".to_string()]);
        assert_eq!(db.get("terminal-2", 10), vec!["cd".to_string()]);
        assert!(db.get("terminal-1", 10).is_empty());
    }

    #[test]
    fn adding_existing_command_moves_it_to_the_front_without_duplicates() {
        let (db, path) = test_db();

        db.add("terminal", "first");
        db.add("terminal", "second");
        db.add("terminal", "first");

        assert_eq!(db.get("terminal", 10), vec!["first", "second"]);
        let _ = std::fs::remove_file(path);
    }
}

impl HistoryDb {
    pub fn new(path: &Path, max_entries: usize) -> Self {
        let conn = Connection::open(path).expect("Failed to open history database");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS command_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                terminal_id TEXT NOT NULL,
                command TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_terminal ON command_history(terminal_id);",
        )
        .expect("Failed to create history table");
        Self { conn, max_entries }
    }

    pub fn add(&self, terminal_id: &str, command: &str) {
        let trimmed = command.trim();
        if trimmed.is_empty() || trimmed.len() <= 1 {
            return;
        }
        let Ok(tx) = self.conn.unchecked_transaction() else {
            return;
        };
        if tx
            .execute(
                "DELETE FROM command_history WHERE terminal_id = ?1 AND command = ?2",
                params![terminal_id, trimmed],
            )
            .is_err()
            || tx
                .execute(
                    "INSERT INTO command_history (terminal_id, command) VALUES (?1, ?2)",
                    params![terminal_id, trimmed],
                )
                .is_err()
            || tx
                .execute(
                    "DELETE FROM command_history WHERE terminal_id = ?1 AND id NOT IN (
                    SELECT id FROM command_history WHERE terminal_id = ?1 ORDER BY id DESC LIMIT ?2
                )",
                    params![terminal_id, self.max_entries],
                )
                .is_err()
        {
            let _ = tx.rollback();
            return;
        }
        let _ = tx.commit();
    }

    pub fn get(&self, terminal_id: &str, limit: usize) -> Vec<String> {
        let mut stmt = self
            .conn
            .prepare("SELECT command FROM command_history WHERE terminal_id = ?1 ORDER BY id DESC")
            .unwrap();
        let rows = stmt
            .query_map(params![terminal_id], |row| row.get::<_, String>(0))
            .unwrap();
        let mut seen = std::collections::HashSet::new();
        rows.filter_map(|r| r.ok())
            .filter(|command| seen.insert(command.clone()))
            .take(limit)
            .collect()
    }

    pub fn clear(&self, terminal_id: &str) {
        self.conn
            .execute(
                "DELETE FROM command_history WHERE terminal_id = ?1",
                params![terminal_id],
            )
            .ok();
    }

    /// All terminal ids that currently have history rows.
    pub fn terminal_ids(&self) -> Vec<String> {
        let Ok(mut stmt) = self
            .conn
            .prepare("SELECT DISTINCT terminal_id FROM command_history")
        else {
            return Vec::new();
        };
        stmt.query_map([], |row| row.get::<_, String>(0))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    }

    /// Delete history for every terminal id NOT in `keep` (orphan cleanup,
    /// e.g. ids left behind by closed workspaces whose scene was never
    /// re-saved).
    pub fn prune(&self, keep: &[String]) {
        let ids = self.terminal_ids();
        let stale: Vec<&String> = ids.iter().filter(|id| !keep.contains(id)).collect();
        for id in stale {
            self.clear(id);
        }
    }

    pub fn clear_all(&self) {
        self.conn.execute("DELETE FROM command_history", []).ok();
    }

    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
    }
}
