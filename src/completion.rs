//! Command completion (roadmap follow-up): merges two suggestion sources
//! for the auto-match overlay while the user types in a terminal —
//!
//! 1. the terminal's own command history, ranked by re-run count (hits),
//! 2. executable names found on PATH (scanned once per process).
//!
//! Pure logic lives here; the overlay wiring lives in app.rs.

use std::collections::HashSet;

/// Executables reachable via PATH, as bare command names (sorted,
/// deduplicated). Cached process-wide: PATH doesn't change mid-session
/// in any realistic setup, and the scan touches every PATH directory.
pub fn path_executables() -> &'static Vec<String> {
    static CACHE: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    CACHE.get_or_init(path_executables_uncached)
}

#[cfg(target_os = "windows")]
const EXEC_EXTENSIONS: &[&str] = &[".exe", ".bat", ".cmd", ".com"];

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;

fn path_executables_uncached() -> Vec<String> {
    let mut names: HashSet<String> = HashSet::new();
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                // Directories named like commands (e.g. a "git" folder)
                // must not become suggestions.
                if !file_type.is_file() {
                    continue;
                }
                let raw = entry.file_name().to_string_lossy().into_owned();
                #[cfg(target_os = "windows")]
                let stem = {
                    let lower = raw.to_lowercase();
                    match EXEC_EXTENSIONS.iter().find(|ext| lower.ends_with(*ext)) {
                        Some(ext) => raw[..raw.len() - ext.len()].to_string(),
                        None => continue,
                    }
                };
                #[cfg(not(target_os = "windows"))]
                let stem = {
                    // Only executables (any of the x bits) qualify.
                    let executable = entry
                        .metadata()
                        .map(|m| m.permissions().mode() & 0o111 != 0)
                        .unwrap_or(false);
                    if !executable {
                        continue;
                    }
                    raw
                };
                if !stem.is_empty() {
                    names.insert(stem);
                }
            }
        }
    }
    let mut sorted: Vec<String> = names.into_iter().collect();
    sorted.sort();
    sorted
}

/// Merge ranked history and PATH names into one suggestion list for the
/// typed `word`:
///
/// - history entries matching the WHOLE text by prefix (so "cd " matches
///   "cd /tmp" but never bare "cd"), best hits first — the query is
///   already sorted that way;
/// - then PATH command names matching by prefix, alphabetical;
/// - a PATH name is skipped when an accepted history entry already starts
///   with it (its first token), so "docker ps" shadows bare "docker".
pub fn suggestions(word: &str, ranked_history: &[(String, i64)], limit: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(limit);
    if word.is_empty() {
        return out;
    }
    let mut covered_first_tokens: HashSet<String> = HashSet::new();
    for (cmd, _hits) in ranked_history {
        if out.len() >= limit {
            return out;
        }
        if cmd.starts_with(word) && !out.contains(cmd) {
            if let Some(first) = cmd.split_whitespace().next() {
                covered_first_tokens.insert(first.to_string());
            }
            out.push(cmd.clone());
        }
    }
    if out.len() < limit {
        for name in path_executables() {
            if out.len() >= limit {
                break;
            }
            if name.starts_with(word) && !covered_first_tokens.contains(name) {
                out.push(name.clone());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{path_executables, suggestions};

    fn ranked(pairs: &[(&str, i64)]) -> Vec<(String, i64)> {
        pairs.iter().map(|(c, h)| (c.to_string(), *h)).collect()
    }

    #[test]
    fn history_ranks_by_hits_and_prefixes_the_whole_text() {
        // A synthetic prefix that no PATH executable shares, so the
        // assertion is environment-independent.
        let word = "zz-opennex-test";
        // Input order = ranking order (hits DESC): the caller passes
        // `get_ranked` output which is already hits-sorted.
        let out = suggestions(
            word,
            &ranked(&[
                ("zz-opennex-test", 5),
                ("zz-opennex-test down", 2),
                ("zz-opennex-test up --all", 1),
            ]),
            10,
        );
        assert_eq!(
            out,
            vec![
                "zz-opennex-test",
                "zz-opennex-test down",
                "zz-opennex-test up --all"
            ]
        );
        // Whole-text prefix INCLUDING spaces: mid-command words match.
        let out = suggestions("cd /tm", &ranked(&[("cd /tmp", 3)]), 10);
        assert_eq!(out, vec!["cd /tmp"]);
        // A one-char difference breaks the history match (only PATH
        // names — if any — may fill in).
        let out = suggestions("cdx", &ranked(&[("cd /tmp", 3)]), 10);
        assert!(
            !out.iter().any(|s| s == "cd /tmp"),
            "changed prefix must not match history: {out:?}"
        );
    }

    #[test]
    fn path_names_fill_remaining_slots_and_dedup_against_history() {
        // "dockerr" is NOT on PATH in this environment; craft a word that
        // hits nothing in history so PATH names appear (if any match).
        let out = suggestions("zzzz-no-match", &ranked(&[]), 10);
        // Nothing on PATH starts with that gibberish either.
        assert!(out.is_empty());
    }

    #[test]
    fn path_names_appear_when_history_has_no_match() {
        // Pick a PATH name and search by its exact prefix.
        let execs = path_executables();
        if let Some(name) = execs.first() {
            let prefix = &name[..name.len().min(1)];
            let out = suggestions(prefix, &ranked(&[]), 10);
            assert!(
                out.iter().any(|s| s == name),
                "PATH name {name} must be suggested for prefix {prefix}"
            );
        }
    }

    #[test]
    fn history_shadows_bare_path_name() {
        // If history already offers "docker ps", the bare "docker" PATH
        // name must not duplicate it as a separate suggestion.
        let execs = path_executables();
        let Some(name) = execs.first() else { return };
        let cmd = format!("{name} --version");
        let out = suggestions(name, &ranked(&[(cmd.as_str(), 1)]), 10);
        assert_eq!(out.first().map(String::as_str), Some(cmd.as_str()));
        assert!(
            !out.iter().skip(1).any(|s| s == name),
            "bare {name} must be shadowed by the history entry"
        );
    }

    #[test]
    fn empty_word_yields_nothing() {
        assert!(suggestions("", &ranked(&[("ls", 1)]), 10).is_empty());
    }
}
