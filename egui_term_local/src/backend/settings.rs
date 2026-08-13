use std::path::PathBuf;

#[cfg(target_os = "windows")]
const DEFAULT_SHELL: &str = "powershell.exe";
#[cfg(target_os = "macos")]
const DEFAULT_SHELL: &str = "/bin/zsh";
#[cfg(all(unix, not(target_os = "macos")))]
const DEFAULT_SHELL: &str = "/bin/bash";

#[derive(Debug, Clone)]
pub struct BackendSettings {
    pub shell: String,
    pub args: Vec<String>,
    pub working_directory: Option<PathBuf>,
    /// Extra environment variables passed to the spawned shell, as
    /// `(name, value)` pairs. Existing variables with the same name are
    /// replaced.
    pub env: Vec<(String, String)>,
}

impl Default for BackendSettings {
    fn default() -> Self {
        Self {
            shell: DEFAULT_SHELL.to_string(),
            args: vec![],
            working_directory: None,
            env: vec![],
        }
    }
}
