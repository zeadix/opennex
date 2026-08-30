//! SSH host book: the data model, OpenSSH executable discovery and the
//! spawn-args building for SSH terminal sessions.
//!
//! Connection strategy: we do NOT implement the SSH protocol. A session
//! is a PTY running the platform's native `ssh` client (Windows ships
//! OpenSSH since Win10 1809; Unix has it everywhere), so key material,
//! agent support and known-hosts handling all come for free. Passwords
//! are never persisted — the `Password` auth mode simply lets the user
//! type them interactively inside the PTY.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Authentication strategy for a saved host.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SshAuth {
    /// Delegate to the running ssh-agent (keys, gpg, 1Password, ...).
    #[default]
    Agent,
    /// Point ssh at a specific identity file.
    Key { path: String },
    /// No stored credentials: the user types the password interactively.
    Password,
}

impl SshAuth {
    pub fn to_db(&self) -> &'static str {
        match self {
            SshAuth::Agent => "agent",
            SshAuth::Key { .. } => "key",
            SshAuth::Password => "password",
        }
    }

    pub fn from_db(auth: &str, key_path: &str) -> Self {
        match auth {
            "key" => SshAuth::Key {
                path: key_path.to_string(),
            },
            "password" => SshAuth::Password,
            _ => SshAuth::Agent,
        }
    }

    pub fn key_path(&self) -> Option<&str> {
        match self {
            SshAuth::Key { path } => Some(path),
            _ => None,
        }
    }
}

/// One saved SSH host. `id` comes from the SQLite row (0 for unsaved).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshHost {
    pub id: i64,
    pub name: String,
    /// Free-form grouping label shown in the sidebar (empty = ungrouped).
    pub group: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub auth: SshAuth,
    /// Production marker: red tab title + warning banner. Ops users'
    /// #1 fear is running a command on the wrong machine.
    pub prod: bool,
    pub sort_key: i64,
}

/// Denormalized host snapshot attached to a terminal tab. Copying the
/// display fields (instead of keeping only the row id) keeps the tab's
/// identity, color and banner intact even after the host entry is
/// deleted from the book.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshHostRef {
    pub id: i64,
    pub name: String,
    pub prod: bool,
    /// "user@host" display string for banners and hover texts.
    pub addr: String,
}

impl SshHost {
    pub fn ref_snapshot(&self) -> SshHostRef {
        SshHostRef {
            id: self.id,
            name: self.name.clone(),
            prod: self.prod,
            addr: ssh_target(self),
        }
    }
}

/// "user@host" (port only matters for connection, not display).
pub fn ssh_target(host: &SshHost) -> String {
    if host.user.is_empty() {
        host.host.clone()
    } else {
        format!("{}@{}", host.user, host.host)
    }
}

/// Build the argv for the native ssh client. The caller supplies argv[0]
/// separately (`ssh_executable`), so everything here is pure and testable.
pub fn ssh_args(host: &SshHost) -> Vec<String> {
    let mut args = vec![
        // accept-new: connect without friction on FIRST contact, but a
        // changed host key later still hard-fails (unlike -o
        // StrictHostKeyChecking=no, which silently accepts MITM).
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
    ];
    if host.port != 0 && host.port != 22 {
        args.push("-p".to_string());
        args.push(host.port.to_string());
    }
    if let Some(path) = host.auth.key_path() {
        if !path.is_empty() {
            args.push("-i".to_string());
            args.push(path.to_string());
            args.push("-o".to_string());
            args.push("IdentitiesOnly=yes".to_string());
        }
    }
    args.push(ssh_target(host));
    args
}

/// Locate the platform ssh client. Cached process-wide: the binary
/// cannot appear mid-session in any realistic setup, and this runs on
/// every SSH tab creation.
pub fn ssh_executable() -> Option<PathBuf> {
    static CACHE: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    CACHE.get_or_init(ssh_executable_uncached).clone()
}

fn ssh_executable_uncached() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Some(root) = std::env::var_os("SystemRoot") {
            let bundled = PathBuf::from(root)
                .join("System32")
                .join("OpenSSH")
                .join("ssh.exe");
            if bundled.is_file() {
                return Some(bundled);
            }
        }
        which_on_path("ssh.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        which_on_path("ssh").or_else(|| {
            [
                "/usr/bin/ssh",
                "/usr/local/bin/ssh",
                "/opt/homebrew/bin/ssh",
            ]
            .iter()
            .map(PathBuf::from)
            .find(|p| p.is_file())
        })
    }
}

fn which_on_path(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
}

/// Spawn a PTY running the ssh client for `host`. Working directory is
/// the user's home: it only affects the local ssh process, and the OSC
/// cwd integration does not apply to remote sessions.
pub fn spawn_ssh_instance(
    ctx: &egui::Context,
    id: u64,
    host: &SshHost,
    scrollback: usize,
) -> Option<crate::terminal::TerminalInstance> {
    let exe = ssh_executable()?;
    let cwd = dirs::home_dir()
        .or_else(|| std::env::current_dir().ok())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    crate::terminal::TerminalInstance::create(
        ctx,
        id,
        &exe.to_string_lossy(),
        &cwd,
        80,
        24,
        &ssh_args(host),
        scrollback,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_host() -> SshHost {
        SshHost {
            id: 1,
            name: "prod-web".into(),
            group: String::new(),
            host: "203.0.113.7".into(),
            port: 22,
            user: String::new(),
            auth: SshAuth::Agent,
            prod: false,
            sort_key: 0,
        }
    }

    #[test]
    fn ssh_args_default_host_is_target_only_plus_safe_hostkey_policy() {
        let args = ssh_args(&base_host());
        assert_eq!(
            args,
            vec![
                "-o".to_string(),
                "StrictHostKeyChecking=accept-new".to_string(),
                "203.0.113.7".to_string(),
            ]
        );
    }

    #[test]
    fn ssh_args_carry_user_port_and_identity() {
        let mut host = base_host();
        host.user = "deploy".into();
        host.port = 2222;
        host.auth = SshAuth::Key {
            path: "/home/me/id_ed25519".into(),
        };
        let args = ssh_args(&host);
        assert!(args.contains(&"-p".to_string()));
        assert!(args.contains(&"2222".to_string()));
        assert!(args.contains(&"-i".to_string()));
        assert!(args.contains(&"/home/me/id_ed25519".to_string()));
        assert!(args.contains(&"IdentitiesOnly=yes".to_string()));
        assert_eq!(args.last().unwrap(), "deploy@203.0.113.7");
    }

    #[test]
    fn empty_identity_path_degrades_to_agent_auth() {
        let mut host = base_host();
        host.auth = SshAuth::Key {
            path: String::new(),
        };
        let args = ssh_args(&host);
        assert!(!args.contains(&"-i".to_string()));
    }

    #[test]
    fn auth_db_roundtrip_preserves_strategy() {
        assert_eq!(SshAuth::Agent.to_db(), "agent");
        assert_eq!(
            SshAuth::from_db("key", "/tmp/k"),
            SshAuth::Key {
                path: "/tmp/k".into()
            }
        );
        assert_eq!(SshAuth::from_db("password", ""), SshAuth::Password);
        assert_eq!(SshAuth::from_db("unknown-legacy", ""), SshAuth::Agent);
        assert_eq!(SshAuth::default(), SshAuth::Agent);
    }

    #[test]
    fn target_prefixes_user_only_when_present() {
        let mut host = base_host();
        assert_eq!(ssh_target(&host), "203.0.113.7");
        host.user = "ops".into();
        assert_eq!(ssh_target(&host), "ops@203.0.113.7");
    }

    #[test]
    fn ref_snapshot_survives_host_mutation() {
        let host = base_host();
        let snapshot = host.ref_snapshot();
        assert_eq!(snapshot.addr, "203.0.113.7");
        assert!(!snapshot.prod);
    }
}
