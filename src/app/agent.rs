//! AI terminal agent (v1): a goal-driven loop that lets the model run
//! commands in a terminal and observe the results, one command at a
//! time, behind a safety gate.
//!
//! This module holds the PURE half: command classification, the
//! approval matrix, the model action protocol and the completion
//! detection policy. The state machine and UI live in `agent_ui.rs`.

use serde::Deserialize;

/// How much of the terminal the agent may act on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ApprovalMode {
    /// Every command needs an explicit user confirmation.
    Manual,
    /// Read-only commands run automatically; everything else confirms.
    #[default]
    Allowlist,
    /// Everything except Destructive commands runs automatically.
    FullAuto,
}

impl ApprovalMode {
    /// Settings-file serialization (stable ids, not enum names).
    pub fn as_str(self) -> &'static str {
        match self {
            ApprovalMode::Manual => "manual",
            ApprovalMode::Allowlist => "allowlist",
            ApprovalMode::FullAuto => "full-auto",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "manual" => ApprovalMode::Manual,
            "full-auto" => ApprovalMode::FullAuto,
            _ => ApprovalMode::Allowlist,
        }
    }
}

/// Safety classification of one shell command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdClass {
    /// Read-only observation (ls, cat, grep, git status, ...).
    Safe,
    /// Mutating but legitimate (rm single file, mv, cp, sudo, pipe into
    /// files, installs, service restarts) - confirms unless FullAuto.
    Risky,
    /// Never runs without an explicit confirm (and usually shouldn't
    /// run at all): rm -rf on broad paths, dd to devices, mkfs, fork
    /// bombs, shutdown.
    Destructive,
}

/// Read-only commands allowed to run automatically in Allowlist mode.
const SAFE_COMMANDS: &[&str] = &[
    "ls",
    "pwd",
    "cat",
    "head",
    "tail",
    "wc",
    "grep",
    "rg",
    "find",
    "du",
    "df",
    "free",
    "ps",
    "top",
    "htop",
    "who",
    "whoami",
    "id",
    "uname",
    "uptime",
    "date",
    "env",
    "printenv",
    "echo",
    "which",
    "whereis",
    "type",
    "stat",
    "file",
    "wc",
    "diff",
    "tree",
    "ip",
    "ifconfig",
    "netstat",
    "ss",
    "ping",
    "dig",
    "nslookup",
    "curl",
    "wget",
    "ssh",
    "scp",
    "git",
    "hg",
    "svn",
    "docker",
    "podman",
    "kubectl",
    "helm",
    "systemctl",
    "journalctl",
    "npm",
    "pnpm",
    "yarn",
    "cargo",
    "python",
    "python3",
    "node",
    "go",
    "rustc",
    "make",
    "cmake",
    "jq",
    "awk",
    "sed",
    "cut",
    "sort",
    "uniq",
    "tr",
    "basename",
    "dirname",
    "realpath",
    "readlink",
    "md5sum",
    "sha256sum",
    "history",
    "alias",
    "man",
    "less",
    "more",
];

/// Subcommands that flip an otherwise-safe parent into Risky.
const RISKY_SUBCOMMANDS: &[(&str, &[&str])] = &[
    (
        "git",
        &[
            "reset",
            "clean",
            "push",
            "rebase",
            "checkout",
            "restore",
            "rm",
            "apply",
            "cherry-pick",
        ],
    ),
    (
        "docker",
        &[
            "rm", "rmi", "prune", "stop", "kill", "restart", "system", "volume",
        ],
    ),
    ("podman", &["rm", "rmi", "prune", "stop", "kill", "restart"]),
    (
        "kubectl",
        &[
            "delete", "scale", "apply", "patch", "drain", "cordon", "rollout",
        ],
    ),
    (
        "systemctl",
        &["start", "stop", "restart", "disable", "mask", "kill"],
    ),
    (
        "npm",
        &["install", "uninstall", "update", "link", "publish"],
    ),
    (
        "pnpm",
        &["install", "uninstall", "update", "link", "publish"],
    ),
    ("yarn", &["install", "remove", "upgrade", "link", "publish"]),
];

/// Classify one command line. The first word carries the program; flag
/// analysis and fixed destructive patterns refine the verdict.
pub fn classify_command(line: &str) -> CmdClass {
    let line = line.trim();
    if line.is_empty() {
        return CmdClass::Destructive;
    }
    // Destructive patterns first: they override everything.
    let destructive_patterns = [
        "rm -rf /",
        "rm -fr /",
        "rm -rf ~",
        "rm -rf *",
        "rm -rf .",
        "mkfs",
        "dd of=/dev/",
        ":(){",
        "fork bomb",
        "shutdown",
        "reboot",
        "halt",
        "poweroff",
        "init 0",
        "init 6",
        "> /dev/sd",
        "chmod -R 777 /",
        "chown -R",
        "mkfs.ext",
        "wipefs",
        "blkdiscard",
        "dd if=/dev/zero",
    ];
    for pat in destructive_patterns {
        if line.contains(pat) {
            return CmdClass::Destructive;
        }
    }
    // `rm -rf <anything>` is destructive (broad recursive delete).
    if line.starts_with("rm ")
        && (line.contains("-rf") || line.contains("-fr") || line.contains("-r -f"))
    {
        return CmdClass::Destructive;
    }

    let words: Vec<&str> = line.split_whitespace().collect();
    let Some(first) = words.first().copied() else {
        return CmdClass::Destructive;
    };
    // Strip a leading env/sudo prefix to find the real program.
    let mut program = first;
    while matches!(program, "sudo" | "env" | "nohup" | "time" | "watch") {
        let idx = words.iter().position(|w| *w == program).unwrap_or(0);
        program = words.get(idx + 1).copied().unwrap_or("");
        if program.is_empty() {
            return CmdClass::Risky;
        }
    }
    let base = program.rsplit('/').next().unwrap_or(program);

    // Mutating shell syntax anywhere makes it Risky at minimum.
    let mutates = words.iter().skip(1).any(|w| {
        w.starts_with('>') || *w == "|" || w.contains(">>") || *w == "tee" || *w == "xargs"
    }) || words
        .iter()
        .any(|w| *w == "sudo" || *w == "kill" || *w == "killall" || *w == "pkill")
        || first == "sudo"
        || first == "kill"
        || first == "killall"
        || first == "pkill"
        || words
            .iter()
            .skip(1)
            .any(|w| matches!(*w, "rm" | "mv" | "cp" | "mkdir" | "touch" | "ln"));
    if mutates {
        // Exception: pure read pipelines like `ls -la | grep foo` stay
        // Safe - only PROGRAM positions (first / after a pipe) must be
        // known read-only commands; their arguments are unconstrained.
        let only_read_pipeline = line.contains('|') && !line.contains('>') && {
            let mut expect_program = true;
            let mut ok = true;
            for w in &words {
                if *w == "|" {
                    expect_program = true;
                    continue;
                }
                if expect_program {
                    let base = w.rsplit('/').next().unwrap_or(w);
                    if !SAFE_COMMANDS.contains(&base) {
                        ok = false;
                        break;
                    }
                    expect_program = false;
                }
            }
            ok && !expect_program
        };
        if only_read_pipeline {
            return CmdClass::Safe;
        }
        return CmdClass::Risky;
    }

    // The program itself decides.
    if matches!(
        base,
        "rm" | "mv"
            | "cp"
            | "mkdir"
            | "rmdir"
            | "touch"
            | "ln"
            | "chmod"
            | "chown"
            | "truncate"
            | "tee"
            | "kill"
            | "killall"
            | "pkill"
            | "reboot"
            | "poweroff"
            | "apt"
            | "apt-get"
            | "yum"
            | "dnf"
            | "pacman"
            | "brew"
            | "install"
            | "useradd"
            | "userdel"
            | "usermod"
            | "groupadd"
            | "passwd"
            | "crontab"
            | "iptables"
            | "nft"
            | "mount"
            | "umount"
            | "swapoff"
            | "sync"
    ) {
        return CmdClass::Risky;
    }
    if !SAFE_COMMANDS.contains(&base) {
        // Unknown program: treat as Risky (confirmation), not Safe.
        return CmdClass::Risky;
    }
    // Known-safe program; a risky SUBCOMMAND (git push ...) upgrades it.
    if let Some(risky) = RISKY_SUBCOMMANDS.iter().find(|(parent, _)| *parent == base) {
        let sub = words.get(1).copied().unwrap_or("");
        if risky.1.contains(&sub) {
            return CmdClass::Risky;
        }
    }
    CmdClass::Safe
}

/// What the safety gate decided for one command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateDecision {
    /// Run it now without asking.
    Auto,
    /// Ask the user first.
    Confirm,
    /// Never run: refuse and tell the model why.
    Deny,
}

/// The approval matrix. PROD hosts must pass `prod = true`, which forces
/// at least a confirmation for EVERY class (no auto-execution).
pub fn gate(mode: ApprovalMode, class: CmdClass, prod: bool) -> GateDecision {
    if prod {
        // PROD: never automatic, Destructive is denied outright.
        return match class {
            CmdClass::Destructive => GateDecision::Deny,
            _ => GateDecision::Confirm,
        };
    }
    match (mode, class) {
        (_, CmdClass::Destructive) => GateDecision::Deny,
        (ApprovalMode::Manual, _) => GateDecision::Confirm,
        (ApprovalMode::Allowlist, CmdClass::Safe) => GateDecision::Auto,
        (ApprovalMode::Allowlist, CmdClass::Risky) => GateDecision::Confirm,
        (ApprovalMode::FullAuto, CmdClass::Safe) => GateDecision::Auto,
        (ApprovalMode::FullAuto, CmdClass::Risky) => GateDecision::Auto,
    }
}

/// One step proposed by the model, in its wire format.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentAction {
    pub action: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub reason: String,
}

/// System prompt for the agent: one command per step, strict JSON.
pub const AGENT_SYSTEM: &str = "You are an autonomous terminal agent embedded in the OpenNex \
terminal manager. The user gives you a GOAL; you achieve it by running ONE shell command per \
step and observing the screen after each. Reply with ONLY a JSON object, no markdown fences, \
no prose: {\"action\":\"run\",\"command\":\"...\",\"reason\":\"short why\"} to run a command, \
{\"action\":\"done\",\"reason\":\"...\"} when the goal is achieved, or \
{\"action\":\"say\",\"reason\":\"...\"} when you need the user's input. Prefer read-only \
commands to gather information first. Never run destructive commands. Keep commands short and \
non-interactive (no vim/top/watch; they block the loop).";

/// Parse a model reply into an [`AgentAction`].
///
/// Tolerates the two common failure modes: JSON wrapped in markdown
/// fences, and a bare command line (treated as `run`). Returns None only
/// when nothing usable is found.
pub fn parse_action(reply: &str) -> Option<AgentAction> {
    let trimmed = reply.trim();
    // 1. Direct JSON (possibly fenced).
    let candidates: Vec<&str> = {
        let mut c = vec![trimmed];
        // ```json ... ``` or ``` ... ```
        if let Some(start) = trimmed.find("```") {
            if let Some(end) = trimmed[start + 3..].find("```") {
                let inner = &trimmed[start + 3..start + 3 + end];
                let inner = inner.strip_prefix("json").unwrap_or(inner).trim();
                c.push(inner);
            }
        }
        c
    };
    for candidate in candidates {
        if let Ok(action) = serde_json::from_str::<AgentAction>(candidate) {
            let a = action.action.to_lowercase();
            if matches!(a.as_str(), "run" | "done" | "say") {
                return Some(AgentAction {
                    action: a,
                    ..action
                });
            }
        }
    }
    // 2. Fence fallback: a bare command line in a code fence.
    if let Some(start) = trimmed.find("```") {
        if let Some(end) = trimmed[start + 3..].find("```") {
            let inner = trimmed[start + 3..start + 3 + end]
                .strip_prefix("bash")
                .or_else(|| trimmed[start + 3..start + 3 + end].strip_prefix("sh"))
                .unwrap_or(&trimmed[start + 3..start + 3 + end])
                .trim();
            let first_line = inner.lines().map(str::trim).find(|l| !l.is_empty())?;
            if !first_line.starts_with('{') {
                return Some(AgentAction {
                    action: "run".into(),
                    command: first_line.to_string(),
                    reason: String::new(),
                });
            }
        }
    }
    // 3. A single non-JSON, non-empty line that looks like a command.
    if let Some(first_line) = trimmed.lines().map(str::trim).find(|l| !l.is_empty()) {
        if !first_line.starts_with('{') && first_line.contains(' ') {
            return Some(AgentAction {
                action: "run".into(),
                command: first_line.to_string(),
                reason: String::new(),
            });
        }
    }
    None
}

/// The completion-detection policy for one waiting step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitSignal {
    /// Wait for `prompt_seq` to grow past the snapshot (local bash/zsh).
    PromptSeq,
    /// Wait until the visible screen text stays stable for ~600ms
    /// (SSH / PowerShell / cmd / fish).
    ScreenStable,
    /// Timed out: hand control back to the user.
    Timeout,
}

/// Pick the wait signal for a terminal by whether the shell integration
/// is active (prompt_seq has ever moved) and how long the command has
/// already been running.
pub fn wait_signal(prompt_seq_seen: u64, elapsed_ms: u128, timeout_ms: u128) -> WaitSignal {
    if elapsed_ms >= timeout_ms {
        return WaitSignal::Timeout;
    }
    if prompt_seq_seen > 0 {
        WaitSignal::PromptSeq
    } else {
        WaitSignal::ScreenStable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_commands_are_safe() {
        for cmd in [
            "ls -la",
            "cat /etc/os-release",
            "grep -r TODO .",
            "git status",
            "df -h",
            "ps aux",
            "echo hi",
            "ls | grep foo",
        ] {
            assert_eq!(classify_command(cmd), CmdClass::Safe, "{cmd}");
        }
    }

    #[test]
    fn mutating_commands_are_risky() {
        for cmd in [
            "mv a b",
            "cp a b",
            "mkdir d",
            "touch f",
            "sudo ls",
            "kill 123",
            "echo hi > out.txt",
            "ls > files.txt",
            "apt install curl",
            "git push",
            "docker rm web",
            "kubectl delete pod x",
            "systemctl restart nginx",
        ] {
            assert_eq!(classify_command(cmd), CmdClass::Risky, "{cmd}");
        }
    }

    #[test]
    fn destructive_patterns_are_denied_class() {
        for cmd in [
            "rm -rf /",
            "rm -rf ~",
            "rm -rf .",
            "mkfs /dev/sda",
            "dd of=/dev/sda",
            ":(){ :|:& };:",
            "shutdown now",
            "dd if=/dev/zero of=/dev/sda",
            "rm -rf /tmp/junk",
        ] {
            assert_eq!(classify_command(cmd), CmdClass::Destructive, "{cmd}");
        }
    }

    #[test]
    fn approval_matrix_matches_the_mode() {
        assert_eq!(
            gate(ApprovalMode::Allowlist, CmdClass::Safe, false),
            GateDecision::Auto
        );
        assert_eq!(
            gate(ApprovalMode::Allowlist, CmdClass::Risky, false),
            GateDecision::Confirm
        );
        assert_eq!(
            gate(ApprovalMode::Allowlist, CmdClass::Destructive, false),
            GateDecision::Deny
        );
        assert_eq!(
            gate(ApprovalMode::Manual, CmdClass::Safe, false),
            GateDecision::Confirm
        );
        assert_eq!(
            gate(ApprovalMode::FullAuto, CmdClass::Risky, false),
            GateDecision::Auto
        );
        assert_eq!(
            gate(ApprovalMode::FullAuto, CmdClass::Destructive, false),
            GateDecision::Deny
        );
    }

    #[test]
    fn prod_never_auto_runs() {
        for mode in [
            ApprovalMode::Manual,
            ApprovalMode::Allowlist,
            ApprovalMode::FullAuto,
        ] {
            assert_eq!(
                gate(mode, CmdClass::Safe, true),
                GateDecision::Confirm,
                "{mode:?}"
            );
            assert_eq!(
                gate(mode, CmdClass::Risky, true),
                GateDecision::Confirm,
                "{mode:?}"
            );
        }
        assert_eq!(
            gate(ApprovalMode::FullAuto, CmdClass::Destructive, true),
            GateDecision::Deny
        );
    }

    #[test]
    fn parses_clean_json() {
        let a =
            parse_action(r#"{"action":"run","command":"ls -la","reason":"look around"}"#).unwrap();
        assert_eq!((a.action.as_str(), a.command.as_str()), ("run", "ls -la"));
        let a = parse_action(r#"{"action":"done","reason":"goal met"}"#).unwrap();
        assert_eq!(a.action, "done");
        assert!(a.command.is_empty());
    }

    #[test]
    fn parses_fenced_json_and_bare_commands() {
        let a = parse_action("```json\n{\"action\":\"run\",\"command\":\"pwd\"}\n```").unwrap();
        assert_eq!((a.action.as_str(), a.command.as_str()), ("run", "pwd"));
        let a = parse_action("```\nls -la\n```").unwrap();
        assert_eq!(a.action, "run");
        assert_eq!(a.command, "ls -la");
    }

    #[test]
    fn unknown_action_strings_are_rejected() {
        assert!(parse_action(r#"{"action":"fly","command":"ls"}"#).is_none());
    }

    #[test]
    fn wait_signal_picks_prompt_seq_when_integration_active() {
        assert_eq!(wait_signal(3, 100, 30_000), WaitSignal::PromptSeq);
        assert_eq!(wait_signal(0, 100, 30_000), WaitSignal::ScreenStable);
        assert_eq!(wait_signal(5, 30_000, 30_000), WaitSignal::Timeout);
        assert_eq!(wait_signal(0, 31_000, 30_000), WaitSignal::Timeout);
    }

    #[test]
    fn approval_mode_roundtrips_stable_ids() {
        for mode in [
            ApprovalMode::Manual,
            ApprovalMode::Allowlist,
            ApprovalMode::FullAuto,
        ] {
            assert_eq!(ApprovalMode::from_str(mode.as_str()), mode);
        }
        assert_eq!(
            ApprovalMode::from_str("legacy-unknown"),
            ApprovalMode::Allowlist
        );
        assert_eq!(ApprovalMode::default(), ApprovalMode::Allowlist);
    }
}
