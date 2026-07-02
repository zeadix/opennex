pub mod pty;
pub mod process;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TerminalManager {
    terminals: Arc<RwLock<HashMap<String, TerminalSession>>>,
    next_id: u32,
}

#[derive(Clone)]
pub struct TerminalSession {
    pub id: String,
    pub name: String,
    pub working_directory: String,
    pub process_id: Option<u32>,
}

impl TerminalManager {
    pub fn new() -> Self {
        TerminalManager {
            terminals: Arc::new(RwLock::new(HashMap::new())),
            next_id: 1,
        }
    }

    pub async fn create_terminal(&mut self, name: &str) -> Result<String> {
        let id = format!("terminal-{}", self.next_id);
        self.next_id += 1;

        let session = TerminalSession {
            id: id.clone(),
            name: name.to_string(),
            working_directory: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "/".to_string()),
            process_id: None,
        };

        let mut terminals = self.terminals.write().await;
        terminals.insert(id.clone(), session);

        log::info!("Created terminal: {} ({})", name, id);
        Ok(id)
    }

    pub async fn get_terminal(&self, id: &str) -> Option<TerminalSession> {
        let terminals = self.terminals.read().await;
        terminals.get(id).cloned()
    }

    pub async fn list_terminals(&self) -> Vec<TerminalSession> {
        let terminals = self.terminals.read().await;
        terminals.values().cloned().collect()
    }

    pub async fn remove_terminal(&self, id: &str) -> Result<()> {
        let mut terminals = self.terminals.write().await;
        terminals.remove(id);
        log::info!("Removed terminal: {}", id);
        Ok(())
    }
}
