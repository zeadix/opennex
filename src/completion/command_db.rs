use std::env;

pub struct CommandDatabase {
    commands: Vec<String>,
}

impl CommandDatabase {
    pub fn build() -> Self {
        let mut commands = Vec::new();
        let path_var = env::var("PATH").unwrap_or_default();
        let separator = if cfg!(windows) { ';' } else { ':' };
        for dir in path_var.split(separator) {
            if dir.is_empty() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && Self::is_executable(&path) {
                        if let Some(name) = path.file_name() {
                            let name = name.to_string_lossy().to_string();
                            if !commands.contains(&name) {
                                commands.push(name);
                            }
                        }
                    }
                }
            }
        }
        commands.sort();
        Self { commands }
    }

    #[cfg(windows)]
    fn is_executable(path: &std::path::Path) -> bool {
        path.extension()
            .map(|e| matches!(e.to_str(), Some("exe" | "cmd" | "bat" | "ps1")))
            .unwrap_or(false)
    }

    #[cfg(not(windows))]
    fn is_executable(path: &std::path::Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    pub fn search(&self, prefix: &str) -> Vec<&str> {
        self.commands
            .iter()
            .filter(|c| c.starts_with(prefix))
            .map(|s| s.as_str())
            .collect()
    }
}
