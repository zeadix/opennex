use rusqlite::{params, Connection};
use std::path::Path;

const SCHEMA_SQL: &str = "CREATE TABLE IF NOT EXISTS favorite_folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS command_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                terminal_id TEXT NOT NULL,
                command TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_terminal ON command_history(terminal_id);
            CREATE TABLE IF NOT EXISTS favorite_commands (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                command TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS ssh_hosts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                group_name TEXT NOT NULL DEFAULT '',
                host TEXT NOT NULL,
                port INTEGER NOT NULL DEFAULT 22,
                user TEXT NOT NULL DEFAULT '',
                auth TEXT NOT NULL DEFAULT 'agent',
                key_path TEXT NOT NULL DEFAULT '',
                prod INTEGER NOT NULL DEFAULT 0,
                sort_key INTEGER,
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
        // --- favorite-folder migration (v0.1.46) -------------------------
        // Older schemas had a flat favorite_commands table. Add the
        // folder/position columns idempotently, then move any legacy
        // rows into a default folder on first run.
        let has_folder_col: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('favorite_commands') \
                 WHERE name='folder_id'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n > 0)?;
        if !has_folder_col {
            conn.execute_batch(
                "ALTER TABLE favorite_commands ADD COLUMN folder_id INTEGER \
                   REFERENCES favorite_folders(id) ON DELETE CASCADE;
                 ALTER TABLE favorite_commands ADD COLUMN sort_key INTEGER;
                 UPDATE favorite_commands SET sort_key = id;",
            )?;
        }
        // Drop the legacy UNIQUE(command) constraint: the same command
        // may now exist in several folders AND multiple times in one
        // folder (rowid-addressed item ops). SQLite can't drop a
        // table-level UNIQUE, so rebuild the table once.
        let cmd_unique: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_index_list('favorite_commands') \
                 WHERE [unique] = 1 AND [origin] = 'u'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n > 0)
            .unwrap_or(false);
        if cmd_unique {
            conn.execute_batch(
                "CREATE TABLE favorite_commands_new (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    command TEXT NOT NULL,
                    folder_id INTEGER REFERENCES favorite_folders(id) ON DELETE CASCADE,
                    sort_key INTEGER,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                 );
                 INSERT INTO favorite_commands_new (id, command, folder_id, sort_key, created_at)
                   SELECT id, command, folder_id, sort_key, created_at FROM favorite_commands;
                 DROP TABLE favorite_commands;
                 ALTER TABLE favorite_commands_new RENAME TO favorite_commands;",
            )?;
        }
        let has_position_col: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('favorite_folders') \
                 WHERE name='position'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n > 0)?;
        if !has_position_col {
            // Seed display order from the legacy id-DESC listing, then
            // PIN the default folder to the very top (position 0): it is
            // where legacy favorites land and where the star button
            // sends commands, so it must always be visible — hiding at
            // the list tail looked like "the default folder vanished".
            conn.execute_batch(
                "ALTER TABLE favorite_folders ADD COLUMN position INTEGER;
                 UPDATE favorite_folders SET position =
                   2 + (SELECT COUNT(*) FROM favorite_folders f2
                        WHERE f2.id > favorite_folders.id);
                 UPDATE favorite_folders SET position = 1
                   WHERE name = '默认收藏';",
            )?;
        }
        let default_folder: i64 = conn
            .query_row(
                "SELECT id FROM favorite_folders WHERE name = '默认收藏'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(-1);
        if default_folder < 0 {
            conn.execute(
                "INSERT INTO favorite_folders (name) VALUES ('默认收藏')",
                [],
            )?;
            let id = conn.last_insert_rowid();
            conn.execute(
                "UPDATE favorite_commands SET folder_id = ?1 WHERE folder_id IS NULL",
                [id],
            )?;
        }
        // --- command_history host/frequency columns (v0.1.50) ------------
        // `host` records WHERE a command ran (ssh addr, empty = local);
        // `hits` counts re-runs, feeding future frequency-ranked
        // completion. Both are added idempotently like the columns above.
        let has_host_col: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('command_history') \
                 WHERE name='host'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n > 0)?;
        if !has_host_col {
            conn.execute_batch(
                "ALTER TABLE command_history ADD COLUMN host TEXT NOT NULL DEFAULT '';
                 ALTER TABLE command_history ADD COLUMN hits INTEGER NOT NULL DEFAULT 1;",
            )?;
        }
        // ------------------------------------------------------------------

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

    /// Record a command run. `host` is the ssh address it ran on (empty
    /// = local shell); re-running the same command bumps its `hits`
    /// counter instead of resetting it.
    pub fn add(&self, terminal_id: &str, command: &str, host: &str) {
        let trimmed = command.trim();
        if trimmed.is_empty() || trimmed.len() <= 1 {
            return;
        }
        let Ok(tx) = self.conn.unchecked_transaction() else {
            return;
        };
        // Preserve the re-run count across the delete+insert dedup.
        let prev_hits: i64 = tx
            .query_row(
                "SELECT COALESCE((SELECT hits FROM command_history \
                 WHERE terminal_id = ?1 AND command = ?2 LIMIT 1), 1)",
                params![terminal_id, trimmed],
                |r| r.get(0),
            )
            .unwrap_or(1);
        if tx
            .execute(
                "DELETE FROM command_history WHERE terminal_id = ?1 AND command = ?2",
                params![terminal_id, trimmed],
            )
            .is_err()
            || tx
                .execute(
                    "INSERT INTO command_history (terminal_id, command, host, hits) \
                     VALUES (?1, ?2, ?3, ?4)",
                    params![terminal_id, trimmed, host, prev_hits.saturating_add(1)],
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

    /// Unique commands of a terminal ranked by re-run count (hits DESC,
    /// recency DESC as the tiebreak) — the ranking for command completion.
    pub fn get_ranked(&self, terminal_id: &str, limit: usize) -> Vec<(String, i64)> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT command, MAX(hits) FROM command_history \
             WHERE terminal_id = ?1 GROUP BY command \
             ORDER BY MAX(hits) DESC, MAX(id) DESC LIMIT ?2",
        ) else {
            return Vec::new();
        };
        stmt.query_map(params![terminal_id, limit], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
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

    /// The well-known default folder every legacy favorite lands in.
    pub const DEFAULT_FAVORITE_FOLDER: &str = "默认收藏";

    fn default_folder_id(&self) -> Option<i64> {
        self.conn
            .query_row(
                "SELECT id FROM favorite_folders WHERE name = ?1",
                params![Self::DEFAULT_FAVORITE_FOLDER],
                |r| r.get(0),
            )
            .ok()
    }

    pub fn fav_add(&self, command: &str) {
        let folder = self.default_folder_id().unwrap_or(1);
        self.fav_add_to(folder, command);
    }

    /// Append a command to a folder's END (ascending sort_key); the UI
    /// drag order decides the final assemble order.
    pub fn fav_add_to(&self, folder_id: i64, command: &str) {
        let trimmed = command.trim();
        if trimmed.is_empty() || trimmed.len() <= 1 {
            return;
        }
        let Ok(tx) = self.conn.unchecked_transaction() else {
            return;
        };
        let next_key: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(sort_key), 0) + 1 FROM favorite_commands WHERE folder_id = ?1",
                params![folder_id],
                |r| r.get(0),
            )
            .unwrap_or(1);
        // Duplicates are ALLOWED (same command in several folders and
        // multiple times in one folder); rows are addressed by rowid.
        if tx
            .execute(
                "INSERT INTO favorite_commands (command, folder_id, sort_key) VALUES (?1, ?2, ?3)",
                params![trimmed, folder_id, next_key],
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

    // ---- Favorite folders (v0.1.46) ----

    /// All folders in display order (creation order), newest first to
    /// match the historical favorites list.
    pub fn fav_folders(&self) -> Vec<(i64, String)> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT id, name FROM favorite_folders \
                 ORDER BY position IS NULL, position ASC, id DESC",
        ) else {
            return Vec::new();
        };
        stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    }

    /// Create a folder; returns its id. Name uniqueness is enforced by
    /// the table constraint; a duplicate returns Err(())-style None.
    pub fn fav_folder_create(&self, name: &str) -> Option<i64> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return None;
        }
        let next_pos: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(position), 0) + 1 FROM favorite_folders",
                [],
                |r| r.get(0),
            )
            .unwrap_or(1);
        self.conn
            .execute(
                "INSERT INTO favorite_folders (name, position) VALUES (?1, ?2)",
                params![trimmed, next_pos],
            )
            .ok()?;
        let id = self.conn.last_insert_rowid();
        // Keep the default folder pinned above whatever was just created.
        let _ = self.conn.execute(
            "UPDATE favorite_folders SET position = 0 WHERE name = ?1",
            params![Self::DEFAULT_FAVORITE_FOLDER],
        );
        Some(id)
    }

    pub fn fav_folder_rename(&self, folder_id: i64, new_name: &str) -> bool {
        let trimmed = new_name.trim();
        if trimmed.is_empty() {
            return false;
        }
        self.conn
            .execute(
                "UPDATE favorite_folders SET name = ?2 WHERE id = ?1",
                params![folder_id, trimmed],
            )
            .is_ok()
    }

    /// Remove ALL commands from a folder, keeping the folder itself.
    pub fn fav_folder_clear(&self, folder_id: i64) {
        let _ = self.conn.execute(
            "DELETE FROM favorite_commands WHERE folder_id = ?1",
            params![folder_id],
        );
    }

    /// Delete a folder AND everything inside it (ON DELETE CASCADE).
    /// The default folder is protected — legacy favorites always land
    /// there, so deleting it would orphan future ones.
    pub fn fav_folder_delete(&self, folder_id: i64) -> bool {
        let name: String = self
            .conn
            .query_row(
                "SELECT name FROM favorite_folders WHERE id = ?1",
                params![folder_id],
                |r| r.get(0),
            )
            .unwrap_or_default();
        if name == Self::DEFAULT_FAVORITE_FOLDER {
            return false;
        }
        self.conn
            .execute(
                "DELETE FROM favorite_folders WHERE id = ?1",
                params![folder_id],
            )
            .is_ok()
    }

    /// Persist a new folder order by rewriting ids' positions via a
    /// shadow sort: simplest robust trick is renaming through a temp
    /// mapping, but SQLite rows have no order — the UI keeps Vec order
    /// and we materialize it with a position column-less approach:
    pub fn fav_folder_reorder(&self, ordered_ids: &[i64]) {
        // The default folder stays PINNED to the top regardless of the
        // drag: after applying the user order, force it back to
        // position 0 (favorite-star commands land there; it must never
        // be buried).
        let default_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM favorite_folders WHERE name = ?1",
                params![Self::DEFAULT_FAVORITE_FOLDER],
                |r| r.get(0),
            )
            .ok();
        let Ok(tx) = self.conn.unchecked_transaction() else {
            return;
        };
        for (idx, id) in ordered_ids.iter().enumerate() {
            if tx
                .execute(
                    "UPDATE favorite_folders SET position = ?2 WHERE id = ?1",
                    params![id, (idx + 1) as i64],
                )
                .is_err()
            {
                let _ = tx.rollback();
                return;
            }
        }
        if let Some(def) = default_id {
            let _ = tx.execute(
                "UPDATE favorite_folders SET position = 0 WHERE id = ?1",
                params![def],
            );
        }
        let _ = tx.commit();
    }

    /// Folder rows in drag order WITH their rowids — the stable handle
    /// for delete/move/reorder when duplicate commands exist.
    pub fn fav_items_with_ids(&self, folder_id: i64) -> Vec<(i64, String)> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT id, command FROM favorite_commands \
             WHERE folder_id = ?1 ORDER BY sort_key ASC, id ASC",
        ) else {
            return Vec::new();
        };
        stmt.query_map(params![folder_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    }

    /// Commands of a folder in the user's drag order (ascending
    /// sort_key) — the order "assemble" concatenates.
    pub fn fav_items(&self, folder_id: i64) -> Vec<String> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT command FROM favorite_commands \
             WHERE folder_id = ?1 ORDER BY sort_key ASC",
        ) else {
            return Vec::new();
        };
        stmt.query_map(params![folder_id], |row| row.get::<_, String>(0))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    }

    /// Remove ONE row (rowid-addressed: only that duplicate goes away).
    pub fn fav_item_remove_row(&self, rowid: i64) {
        let _ = self.conn.execute(
            "DELETE FROM favorite_commands WHERE id = ?1",
            params![rowid],
        );
    }

    /// Move ONE row (rowid-addressed) to another folder, appended at
    /// the END of the destination (next sort_key).
    pub fn fav_item_move_row(&self, rowid: i64, to_folder: i64) {
        let Ok(tx) = self.conn.unchecked_transaction() else {
            return;
        };
        let next_key: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(sort_key), 0) + 1 FROM favorite_commands WHERE folder_id = ?1",
                params![to_folder],
                |r| r.get(0),
            )
            .unwrap_or(1);
        let ok = tx
            .execute(
                "UPDATE favorite_commands SET folder_id = ?2, sort_key = ?3 WHERE id = ?1",
                params![rowid, to_folder, next_key],
            )
            .is_ok();
        if ok {
            let _ = tx.commit();
        } else {
            let _ = tx.rollback();
        }
    }

    /// Persist a new drag order by rewriting sort_key to 1..=N over the
    /// given ROWID order (duplicate commands each keep their own slot).
    pub fn fav_item_reorder_rows(&self, folder_id: i64, ordered_rowids: &[i64]) {
        let Ok(tx) = self.conn.unchecked_transaction() else {
            return;
        };
        let mut ok = true;
        for (idx, rid) in ordered_rowids.iter().enumerate() {
            if tx
                .execute(
                    "UPDATE favorite_commands SET sort_key = ?3 \
                     WHERE folder_id = ?1 AND id = ?2",
                    params![folder_id, rid, (idx + 1) as i64],
                )
                .is_err()
            {
                ok = false;
                break;
            }
        }
        if ok {
            let _ = tx.commit();
        } else {
            let _ = tx.rollback();
        }
    }

    /// Commands of a folder in drag order, joined for direct execution:
    /// the assemble feature writes this into the current terminal.
    pub fn fav_folder_assemble(&self, folder_id: i64, separator: &str) -> String {
        self.fav_items(folder_id).join(separator)
    }

    // ---- SSH host book (v0.1.50) ----

    const SSH_HOST_COLUMNS: &str =
        "id, name, group_name, host, port, user, auth, key_path, prod, sort_key";

    fn row_to_ssh_host(row: &rusqlite::Row) -> rusqlite::Result<crate::hosts::SshHost> {
        Ok(crate::hosts::SshHost {
            id: row.get(0)?,
            name: row.get(1)?,
            group: row.get(2)?,
            host: row.get(3)?,
            port: row.get::<_, i64>(4)?.clamp(1, u16::MAX.into()) as u16,
            user: row.get(5)?,
            auth: crate::hosts::SshAuth::from_db(
                &row.get::<_, String>(6)?,
                &row.get::<_, String>(7)?,
            ),
            prod: row.get::<_, i64>(8)? != 0,
            sort_key: row.get::<_, Option<i64>>(9)?.unwrap_or(0),
        })
    }

    /// All saved hosts in display order (ascending sort_key, ties by id).
    pub fn ssh_hosts(&self) -> Vec<crate::hosts::SshHost> {
        let Ok(mut stmt) = self.conn.prepare(&format!(
            "SELECT {} FROM ssh_hosts ORDER BY sort_key IS NULL, sort_key ASC, id ASC",
            Self::SSH_HOST_COLUMNS
        )) else {
            return Vec::new();
        };
        stmt.query_map([], Self::row_to_ssh_host)
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    }

    pub fn ssh_host_get(&self, id: i64) -> Option<crate::hosts::SshHost> {
        let sql = format!(
            "SELECT {} FROM ssh_hosts WHERE id = ?1",
            Self::SSH_HOST_COLUMNS
        );
        self.conn
            .query_row(&sql, params![id], Self::row_to_ssh_host)
            .ok()
    }

    /// Insert a NEW host (the struct's id is ignored); returns the rowid.
    pub fn ssh_host_insert(&self, host: &crate::hosts::SshHost) -> Option<i64> {
        let next_key: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(sort_key), 0) + 1 FROM ssh_hosts",
                [],
                |r| r.get(0),
            )
            .unwrap_or(1);
        self.conn
            .execute(
                "INSERT INTO ssh_hosts \
                 (name, group_name, host, port, user, auth, key_path, prod, sort_key) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    host.name,
                    host.group,
                    host.host,
                    host.port as i64,
                    host.user,
                    host.auth.to_db(),
                    host.auth.key_path().unwrap_or(""),
                    host.prod as i64,
                    next_key,
                ],
            )
            .ok()?;
        Some(self.conn.last_insert_rowid())
    }

    /// Update an EXISTING host row (id-addressed).
    pub fn ssh_host_update(&self, host: &crate::hosts::SshHost) -> bool {
        self.conn
            .execute(
                "UPDATE ssh_hosts SET name = ?2, group_name = ?3, host = ?4, port = ?5, \
                 user = ?6, auth = ?7, key_path = ?8, prod = ?9 WHERE id = ?1",
                params![
                    host.id,
                    host.name,
                    host.group,
                    host.host,
                    host.port as i64,
                    host.user,
                    host.auth.to_db(),
                    host.auth.key_path().unwrap_or(""),
                    host.prod as i64,
                ],
            )
            .is_ok()
    }

    pub fn ssh_host_delete(&self, id: i64) -> bool {
        self.conn
            .execute("DELETE FROM ssh_hosts WHERE id = ?1", params![id])
            .is_ok()
    }

    /// Persist a sidebar drag order by rewriting sort_key to 1..=N.
    pub fn ssh_host_reorder(&self, ordered_ids: &[i64]) {
        let Ok(tx) = self.conn.unchecked_transaction() else {
            return;
        };
        for (idx, id) in ordered_ids.iter().enumerate() {
            if tx
                .execute(
                    "UPDATE ssh_hosts SET sort_key = ?2 WHERE id = ?1",
                    params![id, (idx + 1) as i64],
                )
                .is_err()
            {
                let _ = tx.rollback();
                return;
            }
        }
        let _ = tx.commit();
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
        db.add("terminal-1", "ls", "");
        db.add("terminal-2", "cd", "");
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

        db.add("terminal", "first", "");
        db.add("terminal", "second", "");
        db.add("terminal", "first", "");

        assert_eq!(db.get("terminal", 10), vec!["first", "second"]);
        // Re-running "first" bumped its hits and recorded the host.
        db.add("terminal", "first", "ops@10.0.0.5");
        let (cmd, host, hits): (String, String, i64) = db
            .conn
            .query_row(
                "SELECT command, host, hits FROM command_history WHERE command = 'first'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            (cmd.as_str(), host.as_str(), hits),
            ("first", "ops@10.0.0.5", 4)
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn legacy_favorites_migrate_into_default_folder() {
        // Build a LEGACY database with the pre-folder schema first...
        let path = std::env::temp_dir().join(format!(
            "opennex_hist_legacy_{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        {
            use rusqlite::Connection;
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE command_history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    terminal_id TEXT NOT NULL,
                    command TEXT NOT NULL,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );
                CREATE TABLE favorite_commands (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    command TEXT NOT NULL UNIQUE,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );
                INSERT INTO favorite_commands (command) VALUES ('legacy-cmd'), ('another');",
            )
            .unwrap();
        }
        // ...then open it through the app's entry point: the v0.1.46
        // migration must add the folder columns and move both legacy
        // favorites into the default folder.
        let db = HistoryDb::new(&path, 10);
        let folders = db.fav_folders();
        assert!(
            folders
                .iter()
                .any(|(_, n)| n == HistoryDb::DEFAULT_FAVORITE_FOLDER),
            "default folder must exist after migration: {folders:?}"
        );
        let (default_id, _) = folders
            .iter()
            .find(|(_, n)| *n == HistoryDb::DEFAULT_FAVORITE_FOLDER)
            .unwrap();
        let items = db.fav_items(*default_id);
        assert!(items.contains(&"legacy-cmd".to_string()));
        assert!(items.contains(&"another".to_string()));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn folder_crud_reorder_and_assemble() {
        let (db, path) = test_db();
        let fid = db.fav_folder_create("build").expect("create");
        assert!(db.fav_folder_create("build").is_none(), "dup name rejected");
        db.fav_add_to(fid, "cmake .");
        db.fav_add_to(fid, "make -j8");
        db.fav_add_to(fid, "sudo make install");
        assert_eq!(
            db.fav_items(fid),
            vec!["cmake .", "make -j8", "sudo make install"]
        );

        // Drag reorder (rowid-addressed): last item to the front.
        let ids: Vec<i64> = db
            .fav_items_with_ids(fid)
            .into_iter()
            .map(|(r, _)| r)
            .collect();
        db.fav_item_reorder_rows(fid, &[ids[2], ids[0], ids[1]]);
        assert_eq!(
            db.fav_items(fid),
            vec!["sudo make install", "cmake .", "make -j8"]
        );

        // Assemble joins in drag order.
        assert_eq!(
            db.fav_folder_assemble(fid, " && "),
            "sudo make install && cmake . && make -j8"
        );

        // Item remove (rowid).
        let cmake_id = db
            .fav_items_with_ids(fid)
            .into_iter()
            .find(|(_, c)| c == "cmake .")
            .unwrap()
            .0;
        db.fav_item_remove_row(cmake_id);
        assert_eq!(db.fav_items(fid).len(), 2);

        // Rename + folder reorder persistence: Vec order = display order.
        assert!(db.fav_folder_rename(fid, "build-all"));
        let fid2 = db.fav_folder_create("zzz-other").unwrap();
        db.fav_folder_reorder(&[fid2, fid]);
        let names: Vec<String> = db.fav_folders().into_iter().map(|(_, n)| n).collect();
        // The DEFAULT folder is PINNED to the top; the drag order
        // applies to the rest: 默认收藏, zzz-other, build-all.
        assert_eq!(
            names.first().map(String::as_str),
            Some(HistoryDb::DEFAULT_FAVORITE_FOLDER)
        );
        assert_eq!(names.get(1).map(String::as_str), Some("zzz-other"));
        assert_eq!(names.get(2).map(String::as_str), Some("build-all"));

        // Cross-folder move (rowid) BEFORE the folder delete: move
        // "make -j8" (still in fid) to a fresh folder.
        let fid3 = db.fav_folder_create("third").unwrap();
        let mv_id = db
            .fav_items_with_ids(fid)
            .into_iter()
            .find(|(_, c)| c == "make -j8")
            .unwrap()
            .0;
        db.fav_item_move_row(mv_id, fid3);
        assert!(!db.fav_items(fid).contains(&"make -j8".to_string()));
        let third = db.fav_items(fid3);
        assert_eq!(third.last().map(String::as_str), Some("make -j8"));

        // Delete cascades the remaining items away.
        assert!(db.fav_folder_delete(fid));
        assert!(db.fav_items(fid).is_empty());

        // Duplicates are ALLOWED: same command twice in one folder
        // (fid3 already holds ONE "make -j8" from the move above).
        db.fav_add_to(fid3, "make -j8");
        db.fav_add_to(fid3, "make -j8");
        let with_ids = db.fav_items_with_ids(fid3);
        assert_eq!(
            with_ids.iter().filter(|(_, c)| c == "make -j8").count(),
            3,
            "duplicate rows must all exist"
        );
        // Rowid delete removes exactly ONE of them.
        let first_id = with_ids.iter().find(|(_, c)| c == "make -j8").unwrap().0;
        db.fav_item_remove_row(first_id);
        let after: Vec<String> = db
            .fav_items_with_ids(fid3)
            .into_iter()
            .map(|(_, c)| c)
            .collect();
        assert_eq!(
            after.iter().filter(|c| *c == "make -j8").count(),
            2,
            "rowid delete removes exactly one duplicate"
        );
        // Rowid move takes exactly one copy over to fid.
        let second_id = db
            .fav_items_with_ids(fid3)
            .into_iter()
            .find(|(_, c)| c == "make -j8")
            .unwrap()
            .0;
        db.fav_item_move_row(second_id, fid2);
        assert_eq!(
            db.fav_items(fid3)
                .iter()
                .filter(|c| *c == "make -j8")
                .count(),
            1
        );
        assert!(db.fav_items(fid2).contains(&"make -j8".to_string()));

        // The default folder can never be deleted.
        let (def_id, _) = db
            .fav_folders()
            .into_iter()
            .find(|(_, n)| n == HistoryDb::DEFAULT_FAVORITE_FOLDER)
            .expect("default folder exists");
        assert!(
            !db.fav_folder_delete(def_id),
            "default folder must be protected"
        );
        assert!(db
            .fav_folders()
            .iter()
            .any(|(_, n)| n == HistoryDb::DEFAULT_FAVORITE_FOLDER));
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
        recovered.add("terminal", "after-crash", "");
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

    #[test]
    fn ssh_host_crud_reorder_and_update() {
        let (db, path) = test_db();
        let host = crate::hosts::SshHost {
            id: 0,
            name: "web-1".into(),
            group: "prod".into(),
            host: "203.0.113.7".into(),
            port: 2222,
            user: "deploy".into(),
            auth: crate::hosts::SshAuth::Key {
                path: "/tmp/id_ed25519".into(),
            },
            prod: true,
            sort_key: 0,
        };
        let id = db.ssh_host_insert(&host).expect("insert");
        let mut loaded = db.ssh_host_get(id).expect("loaded");
        assert_eq!(loaded.name, "web-1");
        assert_eq!(loaded.group, "prod");
        assert_eq!(loaded.port, 2222);
        assert_eq!(loaded.auth, host.auth, "key auth roundtrips with path");
        assert!(loaded.prod);

        loaded.name = "web-1-renamed".into();
        loaded.auth = crate::hosts::SshAuth::Password;
        assert!(db.ssh_host_update(&loaded));
        let reloaded = db.ssh_host_get(id).unwrap();
        assert_eq!(reloaded.name, "web-1-renamed");
        assert_eq!(reloaded.auth, crate::hosts::SshAuth::Password);

        let id2 = db.ssh_host_insert(&host).unwrap();
        db.ssh_host_reorder(&[id2, id]);
        let order: Vec<i64> = db.ssh_hosts().into_iter().map(|h| h.id).collect();
        assert_eq!(order, vec![id2, id], "drag order persists");

        assert!(db.ssh_host_delete(id));
        assert!(db.ssh_host_get(id).is_none());
        assert_eq!(db.ssh_hosts().len(), 1);
        let _ = std::fs::remove_file(path);
    }
}
