use rusqlite::{params, Connection};
use std::path::Path;

const SCHEMA_SQL: &str = "CREATE TABLE IF NOT EXISTS command_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                terminal_id TEXT NOT NULL,
                command TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_terminal ON command_history(terminal_id);
            CREATE TABLE IF NOT EXISTS favorite_commands (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                command TEXT NOT NULL UNIQUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );";

/// Rows this old are considered truly orphaned by `prune`; younger rows
/// may still belong to ANOTHER running instance sharing the same db
/// file, so they are never touched.
const PRUNE_MIN_AGE_DAYS: i32 = 7;

pub struct HistoryDb {
    conn: Connection,
    max_entries: usize,
}

impl HistoryDb {
    pub fn new(path: &Path, max_entries: usize) -> Self {
        let conn = Self::open_resilient(path);
        Self { conn, max_entries }
    }

    /// Open the on-disk history db, quarantining a corrupt file and
    /// starting fresh; if even that fails (e.g. unwritable directory),
    /// degrade to an in-memory database so the app keeps running and
    /// only loses persistence, never crashes.
    fn open_resilient(path: &Path) -> Connection {
        match Self::open_healthy(path) {
            Ok(conn) => return conn,
            Err(err) => log::error!("history database unusable ({err}); quarantining"),
        }
        crate::persist::quarantine_corrupt_file(path);
        for suffix in ["-wal", "-shm", "-journal"] {
            let mut side = path.as_os_str().to_owned();
            side.push(suffix);
            let _ = std::fs::remove_file(std::path::PathBuf::from(side));
        }
        match Self::open_healthy(path) {
            Ok(conn) => {
                log::warn!("history database rebuilt after quarantine");
                conn
            }
            Err(err) => {
                log::error!("history database falling back to memory: {err}");
                let conn =
                    Connection::open_in_memory().expect("in-memory sqlite must always be openable");
                conn.execute_batch(SCHEMA_SQL)
                    .expect("in-memory schema creation cannot fail");
                conn
            }
        }
    }

    fn open_healthy(path: &Path) -> rusqlite::Result<Connection> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA_SQL)?;
        // Cheap consistency gate: catches truncated/garbage files that
        // still "open" fine but explode later mid-query.
        let status: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
        if status != "ok" {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CORRUPT),
                Some(status),
            ));
        }
        Ok(conn)
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
        let Ok(mut stmt) = self
            .conn
            .prepare("SELECT command FROM command_history WHERE terminal_id = ?1 ORDER BY id DESC")
        else {
            return Vec::new();
        };
        let rows = match stmt.query_map(params![terminal_id], |row| row.get::<_, String>(0)) {
            Ok(rows) => rows,
            Err(_) => return Vec::new(),
        };
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

    /// Delete history for terminal ids NOT in `keep` — but only rows
    /// older than `PRUNE_MIN_AGE_DAYS`. Recent rows of unknown ids may
    /// belong to a second concurrently running instance sharing this db
    /// file, so they are left alone (the closed-workspace path deletes
    /// its rows immediately and explicitly instead).
    pub fn prune(&self, keep: &[String]) {
        let ids = self.terminal_ids();
        let stale: Vec<&String> = ids.iter().filter(|id| !keep.contains(id)).collect();
        for id in stale {
            let _ = self.conn.execute(
                "DELETE FROM command_history WHERE terminal_id = ?1 AND created_at < datetime('now', ?2)",
                params![id, format!("-{PRUNE_MIN_AGE_DAYS} days")],
            );
        }
    }

    pub fn clear_all(&self) {
        self.conn.execute("DELETE FROM command_history", []).ok();
    }

    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
    }

    // ---- Global favorite commands (shared across ALL terminals) ----

    /// Add a command to the global favorites (idempotent, newest-first on
    /// re-add via delete+insert like the history table).
    pub fn fav_add(&self, command: &str) {
        let trimmed = command.trim();
        if trimmed.is_empty() || trimmed.len() <= 1 {
            return;
        }
        let Ok(tx) = self.conn.unchecked_transaction() else {
            return;
        };
        if tx
            .execute(
                "DELETE FROM favorite_commands WHERE command = ?1",
                params![trimmed],
            )
            .is_err()
            || tx
                .execute(
                    "INSERT INTO favorite_commands (command) VALUES (?1)",
                    params![trimmed],
                )
                .is_err()
        {
            let _ = tx.rollback();
            return;
        }
        let _ = tx.commit();
    }

    /// Remove a command from the global favorites.
    pub fn fav_remove(&self, command: &str) {
        self.conn
            .execute(
                "DELETE FROM favorite_commands WHERE command = ?1",
                params![command],
            )
            .ok();
    }

    /// All favorite commands, newest first.
    pub fn fav_all(&self) -> Vec<String> {
        let Ok(mut stmt) = self
            .conn
            .prepare("SELECT command FROM favorite_commands ORDER BY id DESC")
        else {
            return Vec::new();
        };
        stmt.query_map([], |row| row.get::<_, String>(0))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    }

    pub fn fav_clear(&self) {
        self.conn.execute("DELETE FROM favorite_commands", []).ok();
    }

    /// Delete ONE occurrence of a command from a terminal's history
    /// (the row currently visible at that position in newest-first order).
    pub fn remove_entry(&self, terminal_id: &str, index_from_newest: usize) {
        // Deduplicate in newest-first order, same as `get`, then locate
        // the unique command at the requested index and delete ALL its
        // rows (the list shows unique commands).
        let mut seen = std::collections::HashSet::new();
        let mut target: Option<String> = None;
        let commands: Vec<String> = {
            let Ok(mut stmt) = self.conn.prepare(
                "SELECT command FROM command_history WHERE terminal_id = ?1 ORDER BY id DESC",
            ) else {
                return;
            };
            stmt.query_map(params![terminal_id], |row| row.get::<_, String>(0))
                .map(|rows| rows.flatten().collect())
                .unwrap_or_default()
        };
        for c in commands {
            if seen.insert(c.clone()) && seen.len() - 1 == index_from_newest {
                target = Some(c);
                break;
            }
        }
        if let Some(cmd) = target {
            self.conn
                .execute(
                    "DELETE FROM command_history WHERE terminal_id = ?1 AND command = ?2",
                    params![terminal_id, cmd],
                )
                .ok();
        }
    }
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
    fn prune_removes_aged_history_of_unknown_terminal_ids() {
        let (db, path) = test_db();
        db.add("terminal-1", "ls");
        db.add("terminal-2", "cd");
        // Age terminal-1's rows past the prune cutoff; terminal-2 stays fresh.
        db.conn
            .execute(
                "UPDATE command_history SET created_at = datetime('now', '-30 days')
                 WHERE terminal_id = 'terminal-1'",
                [],
            )
            .unwrap();
        db.prune(&["terminal-1".to_string()]);
        assert_eq!(
            db.get("terminal-1", 10),
            vec!["ls".to_string()],
            "live ids are never pruned regardless of row age"
        );
        db.prune(&["terminal-2".to_string()]);
        assert!(
            db.get("terminal-1", 10).is_empty(),
            "aged orphan rows must be pruned"
        );
        assert_eq!(
            db.get("terminal-2", 10),
            vec!["cd".to_string()],
            "fresh rows of unknown ids must survive (another instance may own them)"
        );
        let _ = std::fs::remove_file(path);
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

    #[test]
    fn corrupt_database_is_quarantined_and_rebuilt() {
        // Drop the FIRST connection before touching the file: Windows
        // refuses renames of files with open handles (the quarantine
        // inside HistoryDb::new would hit a sharing violation).
        let (_db, path) = test_db();
        drop(_db);
        let _ = std::fs::remove_file(&path);
        std::fs::write(&path, b"this is not sqlite").unwrap();

        let recovered = HistoryDb::new(&path, 10);
        recovered.add("terminal", "after-crash");
        assert_eq!(
            recovered.get("terminal", 10),
            vec!["after-crash".to_string()]
        );

        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let quarantined = path.with_file_name(format!("{name}.corrupt"));
        assert!(
            quarantined.exists(),
            "broken db file must be preserved for inspection"
        );
        // Drop before cleanup renames for the same Windows reason.
        drop(recovered);
        let _ = std::fs::remove_file(quarantined);
        let _ = std::fs::remove_file(path);
    }
}
