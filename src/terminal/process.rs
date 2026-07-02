use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<String, ProcessInfo>>>,
}

#[derive(Clone)]
pub struct ProcessInfo {
    pub id: String,
    pub terminal_id: String,
    pub command: String,
    pub pid: u32,
}

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager {
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn spawn_process(&self, terminal_id: &str, command: &str) -> Result<String> {
        let process_id = format!("process-{}", uuid::Uuid::new_v4());
        
        log::info!("Spawning process for terminal {}: {}", terminal_id, command);
        
        Ok(process_id)
    }

    pub async fn get_process(&self, id: &str) -> Option<ProcessInfo> {
        let processes = self.processes.read().await;
        processes.get(id).cloned()
    }

    pub async fn kill_process(&self, id: &str) -> Result<()> {
        let mut processes = self.processes.write().await;
        processes.remove(id);
        log::info!("Killed process: {}", id);
        Ok(())
    }
}
