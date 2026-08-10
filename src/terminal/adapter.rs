use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ShellInfo {
    pub cwd: Arc<Mutex<String>>,
    pub last_command: Arc<Mutex<String>>,
}

impl ShellInfo {
    pub fn new() -> Self {
        ShellInfo {
            cwd: Arc::new(Mutex::new(String::new())),
            last_command: Arc::new(Mutex::new(String::new())),
        }
    }
}
