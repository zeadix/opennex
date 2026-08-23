//! Detecting spawnable shells, primarily for Windows where multiple
//! terminals coexist (cmd, Windows PowerShell, PowerShell 7, the Visual
//! Studio developer prompt, WSL). Non-Windows platforms report a single
//! entry derived from $SHELL.

use std::path::PathBuf;

/// A shell that can be spawned in a terminal tab.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ShellOption {
    /// Stable settings/scene id, e.g. "cmd", "powershell", "pwsh",
    /// "vs-dev", "wsl".
    pub id: &'static str,
    /// Display name (localized at render time via i18n when available).
    pub name_key: &'static str,
    /// Program to spawn.
    pub program: String,
    /// Extra arguments (vcvars bootstrap for the VS developer prompt).
    pub args: Vec<String>,
}

#[cfg(target_os = "windows")]
mod imp {
    use super::*;

    fn where_exe(name: &str) -> Option<PathBuf> {
        let path = std::env::var_os("PATH")?;
        std::env::split_paths(&path)
            .map(|dir| dir.join(format!("{name}.exe")))
            .find(|candidate| candidate.is_file())
    }

    /// Locate the newest Visual Studio via vswhere and build a cmd /k
    /// bootstrap for its 64-bit developer environment.
    fn vs_developer_prompt() -> Option<ShellOption> {
        let pf = std::env::var_os("ProgramFiles(x86)")
            .or_else(|| std::env::var_os("ProgramFiles"))
            .map(PathBuf::from)?;
        let vswhere = pf
            .join("Microsoft Visual Studio")
            .join("Installer")
            .join("vswhere.exe");
        if !vswhere.is_file() {
            return None;
        }
        let out = std::process::Command::new(&vswhere)
            .args([
                "-latest",
                "-products",
                "*",
                "-requires",
                "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                "-property",
                "installationPath",
            ])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let install_path = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if install_path.is_empty() {
            return None;
        }
        let vcvars = PathBuf::from(&install_path)
            .join("VC")
            .join("Auxiliary")
            .join("Build")
            .join("vcvars64.bat");
        if !vcvars.is_file() {
            return None;
        }
        let comspec = std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".into());
        Some(ShellOption {
            id: "vs-dev",
            name_key: "vs_dev",
            program: comspec,
            args: vec!["/k".into(), format!("\"{}\"", vcvars.to_string_lossy())],
        })
    }

    pub fn detect_shells() -> Vec<ShellOption> {
        let mut shells = Vec::new();
        // cmd.exe (COMSPEC) — always present.
        let comspec = std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".into());
        shells.push(ShellOption {
            id: "cmd",
            name_key: "cmd",
            program: comspec,
            args: vec![],
        });
        // Windows PowerShell 5.1 — always present on supported Windows.
        shells.push(ShellOption {
            id: "powershell",
            name_key: "powershell",
            program: "powershell.exe".into(),
            args: vec![],
        });
        // PowerShell 7 — only when installed.
        let pwsh_on_path = where_exe("pwsh").is_some();
        let pwsh_fixed = PathBuf::from(
            std::env::var_os("ProgramFiles")
                .map(PathBuf::from)
                .unwrap_or_default(),
        )
        .join("PowerShell")
        .join("7")
        .join("pwsh.exe");
        if pwsh_on_path || pwsh_fixed.is_file() {
            shells.push(ShellOption {
                id: "pwsh",
                name_key: "pwsh",
                program: "pwsh.exe".into(),
                args: vec![],
            });
        }
        // Visual Studio developer prompt — only with VS + VC tools.
        if let Some(vs) = vs_developer_prompt() {
            shells.push(vs);
        }
        // WSL — only when the launcher exists.
        if where_exe("wsl").is_some() {
            shells.push(ShellOption {
                id: "wsl",
                name_key: "wsl",
                program: "wsl.exe".into(),
                args: vec![],
            });
        }
        shells
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use super::*;

    pub fn detect_shells() -> Vec<ShellOption> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| {
            #[cfg(target_os = "macos")]
            {
                "/bin/zsh".into()
            }
            #[cfg(not(target_os = "macos"))]
            {
                "/bin/bash".into()
            }
        });
        vec![ShellOption {
            id: "default",
            name_key: "default",
            program: shell,
            args: vec![],
        }]
    }
}

pub fn detect_shells() -> Vec<ShellOption> {
    imp::detect_shells()
}

/// Resolve a shell id (from settings/scene) against the DETECTED list,
/// falling back to the first detected shell.
pub fn resolve_shell<'a>(shells: &'a [ShellOption], id: &str) -> &'a ShellOption {
    shells.iter().find(|s| s.id == id).unwrap_or(&shells[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detected_shells_are_never_empty_and_have_unique_ids() {
        let shells = detect_shells();
        assert!(!shells.is_empty());
        let mut ids: Vec<_> = shells.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count, "duplicate shell ids");
    }

    #[test]
    fn resolve_shell_falls_back_to_first_when_id_unknown() {
        let shells = detect_shells();
        assert_eq!(resolve_shell(&shells, "no-such-shell").id, shells[0].id);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_always_offers_cmd_and_powershell() {
        let shells = detect_shells();
        assert!(shells.iter().any(|s| s.id == "cmd"));
        assert!(shells.iter().any(|s| s.id == "powershell"));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn unix_reports_the_login_shell_only() {
        let shells = detect_shells();
        assert_eq!(shells.len(), 1);
        assert_eq!(shells[0].id, "default");
    }
}
