//! Crash-safe persistence primitives shared by settings, scene and
//! history storage: atomic temp+backup+rename writes and quarantine of
//! unreadable files.

use std::io::Write;
use std::path::{Path, PathBuf};

/// Atomically replace `path` with `data`.
///
/// Writes `<name>.tmp` next to the target, syncs it, moves the existing
/// target to `<name>.bak`, renames the temp into place, then removes the
/// backup. On failure the previous file is restored from the backup, so
/// a crash mid-write can never leave a truncated document behind.
pub fn atomic_write(path: &Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "path has no file name")
        })?;
    let tmp = path.with_file_name(format!("{name}.tmp"));
    let backup = path.with_file_name(format!("{name}.bak"));

    {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(data)?;
        file.sync_all()?;
    }

    let had_existing = path.exists();
    if had_existing {
        let _ = std::fs::remove_file(&backup);
        std::fs::rename(path, &backup)?;
    }

    match std::fs::rename(&tmp, path) {
        Ok(()) => {
            let _ = std::fs::remove_file(&backup);
            Ok(())
        }
        Err(err) => {
            if had_existing {
                let _ = std::fs::rename(&backup, path);
            }
            let _ = std::fs::remove_file(&tmp);
            Err(err)
        }
    }
}

/// Serialize `value` as pretty JSON and atomically write it to `path`.
pub fn atomic_write_json<T: serde::Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    Ok(atomic_write(path, json.as_bytes())?)
}

/// Move an unreadable file aside as `<name>.corrupt` so a fresh default
/// can take its place while preserving the broken content for inspection.
/// Returns the quarantine path on success.
pub fn quarantine_corrupt_file(path: &Path) -> Option<PathBuf> {
    if !path.exists() {
        return None;
    }
    let name = path.file_name()?.to_string_lossy().into_owned();
    let bad = path.with_file_name(format!("{name}.corrupt"));
    let _ = std::fs::remove_file(&bad);
    std::fs::rename(path, &bad).ok()?;
    Some(bad)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "opennex_persist_{}-{}-{tag}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn atomic_write_roundtrip_and_overwrite() {
        let path = temp_file("rw");
        atomic_write(&path, b"v1").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"v1");
        atomic_write(&path, b"v2-longer").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"v2-longer");
        assert!(!path.with_file_name("opennex_persist_rw.tmp").exists());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn quarantine_moves_broken_file_aside() {
        let path = temp_file("q");
        std::fs::write(&path, b"garbage").unwrap();
        let quarantined = quarantine_corrupt_file(&path).unwrap();
        assert!(!path.exists());
        assert_eq!(std::fs::read(&quarantined).unwrap(), b"garbage");
        let _ = std::fs::remove_file(quarantined);
    }
}
