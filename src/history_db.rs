use rusqlite::{Connection, params};
use std::path::Path;

pub struct HistoryDb {
    conn: Connection,
    max_entries: usize,
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
            CREATE INDEX IF NOT EXISTS idx_terminal ON command_history(terminal_id);"
        ).expect("Failed to create history table");
        Self { conn, max_entries }
    }

    pub fn add(&self, terminal_id: &str, command: &str) {
        let trimmed = command.trim();
        if trimmed.is_empty() || trimmed.len() <= 1 {
            return;
        }
        self.conn.execute(
            "INSERT INTO command_history (terminal_id, command) VALUES (?1, ?2)",
            params![terminal_id, trimmed],
        ).ok();
        // Cleanup old entries beyond max
        self.conn.execute(
            "DELETE FROM command_history WHERE terminal_id = ?1 AND id NOT IN (
                SELECT id FROM command_history WHERE terminal_id = ?1 ORDER BY id DESC LIMIT ?2
            )",
            params![terminal_id, self.max_entries],
        ).ok();
    }

    pub fn get(&self, terminal_id: &str, limit: usize) -> Vec<String> {
        let mut stmt = self.conn.prepare(
            "SELECT command FROM command_history WHERE terminal_id = ?1 ORDER BY id DESC LIMIT ?2"
        ).unwrap();
        let rows = stmt.query_map(params![terminal_id, limit], |row| {
            row.get::<_, String>(0)
        }).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn clear(&self, terminal_id: &str) {
        self.conn.execute(
            "DELETE FROM command_history WHERE terminal_id = ?1",
            params![terminal_id],
        ).ok();
    }

    pub fn clear_all(&self) {
        self.conn.execute("DELETE FROM command_history", []).ok();
    }

    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
    }
}
