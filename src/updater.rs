use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const UPDATE_URL: &str = "https://opennex.download.zeadix.com/latest.json";

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseInfo {
    pub version: String,
    #[serde(default)]
    pub changelog: Option<String>,
    pub files: ReleaseFiles,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseFiles {
    pub windows: Option<PlatformFile>,
    pub macos: Option<PlatformFile>,
    pub linux: Option<PlatformFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlatformFile {
    pub portable: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub sha256: String,
    pub changelog: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateState {
    Idle,
    Checking,
    Available(UpdateInfo),
    Downloading(f32),
    Verifying,
    Ready(PathBuf),
    Error(String),
    UpToDate,
}

fn current_platform() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
}

pub fn version_is_newer(remote: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.trim_start_matches('v')
            .split('.')
            .filter_map(|n| n.parse::<u64>().ok())
            .collect()
    };
    let r = parse(remote);
    let c = parse(current);
    for i in 0..r.len().max(c.len()) {
        let rv = r.get(i).copied().unwrap_or(0);
        let cv = c.get(i).copied().unwrap_or(0);
        if rv > cv {
            return true;
        }
        if rv < cv {
            return false;
        }
    }
    false
}

pub fn check_for_update() -> Result<Option<UpdateInfo>, String> {
    let resp: ReleaseInfo = ureq::get(UPDATE_URL)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| format!("请求失败: {e}"))?
        .into_json()
        .map_err(|e| format!("解析失败: {e}"))?;

    let platform = current_platform();
    let file = match platform {
        "windows" => resp.files.windows.as_ref(),
        "macos" => resp.files.macos.as_ref(),
        "linux" => resp.files.linux.as_ref(),
        _ => return Err("不支持的平台".into()),
    }
    .ok_or("当前平台没有对应的下载文件")?;

    if version_is_newer(&resp.version, env!("CARGO_PKG_VERSION")) {
        Ok(Some(UpdateInfo {
            version: resp.version,
            download_url: file.portable.clone(),
            sha256: file.sha256.clone(),
            changelog: resp.changelog.unwrap_or_default(),
        }))
    } else {
        Ok(None)
    }
}

fn current_exe_path() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| format!("无法获取当前路径: {e}"))
}

fn temp_dir() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("opennex-update");
    fs::create_dir_all(&dir).map_err(|e| format!("创建临时目录失败: {e}"))?;
    Ok(dir)
}

fn download_file(
    url: &str,
    dest: &Path,
    progress_tx: &std::sync::mpsc::Sender<f32>,
) -> Result<(), String> {
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(300))
        .call()
        .map_err(|e| format!("下载失败: {e}"))?;

    let total = resp
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let mut reader = resp.into_reader();
    let mut file = fs::File::create(dest).map_err(|e| format!("创建文件失败: {e}"))?;
    let mut buf = [0u8; 8192];
    let mut downloaded: u64 = 0;

    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("读取数据失败: {e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .map_err(|e| format!("写入文件失败: {e}"))?;
        downloaded += n as u64;
        if total > 0 {
            let pct = downloaded as f32 / total as f32;
            let _ = progress_tx.send(pct.min(1.0));
        }
    }
    let _ = progress_tx.send(1.0);
    Ok(())
}

fn verify_sha256(path: &Path, expected: &str) -> Result<bool, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("打开文件失败: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("读取文件失败: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let hash = hasher.finalize();
    let hash_hex = format!("{:x}", hash);
    Ok(hash_hex == expected.to_lowercase().trim())
}

fn extract_binary(archive_path: &Path) -> Result<PathBuf, String> {
    let temp = temp_dir()?;
    #[cfg(target_os = "windows")]
    {
        // Windows: extract exe from zip
        let mut zip_archive = zip::ZipArchive::new(
            fs::File::open(archive_path).map_err(|e| format!("打开zip失败: {e}"))?,
        )
        .map_err(|e| format!("解析zip失败: {e}"))?;

        for i in 0..zip_archive.len() {
            let mut entry = zip_archive
                .by_index(i)
                .map_err(|e| format!("读取zip条目失败: {e}"))?;
            let name = entry.name().to_string();
            if name.ends_with(".exe") {
                let out = temp.join("opennex_new.exe");
                let mut f = fs::File::create(&out).map_err(|e| format!("创建文件失败: {e}"))?;
                std::io::copy(&mut entry, &mut f).map_err(|e| format!("解压失败: {e}"))?;
                return Ok(out);
            }
        }
        Err("zip 中未找到 .exe 文件".into())
    }
    #[cfg(not(target_os = "windows"))]
    {
        // macOS/Linux: extract binary from tar.gz
        let tar_gz = fs::File::open(archive_path).map_err(|e| format!("打开tar.gz失败: {e}"))?;
        let gz = flate2::read::GzDecoder::new(tar_gz);
        let mut ar = tar::Archive::new(gz);
        for entry in ar.entries().map_err(|e| format!("解析tar失败: {e}"))? {
            let mut entry = entry.map_err(|e| format!("读取条目失败: {e}"))?;
            let path = entry.path().map_err(|e| format!("路径失败: {e}"))?;
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "opennex" {
                let out = temp.join("opennex_new");
                entry.unpack(&out).map_err(|e| format!("解压失败: {e}"))?;
                return Ok(out);
            }
        }
        Err("tar.gz 中未找到 opennex 二进制".into())
    }
}

pub fn download_and_verify(
    url: &str,
    sha256: &str,
    progress_tx: &std::sync::mpsc::Sender<f32>,
) -> Result<PathBuf, String> {
    let temp = temp_dir()?;
    let archive_path = temp.join("update_download");

    download_file(url, &archive_path, progress_tx)?;

    let valid = verify_sha256(&archive_path, sha256)?;
    if !valid {
        let _ = fs::remove_file(&archive_path);
        return Err("SHA256 校验失败".into());
    }

    let new_binary = extract_binary(&archive_path)?;
    let _ = fs::remove_file(&archive_path);
    Ok(new_binary)
}

pub fn perform_update(download_url: &str, sha256: &str) -> Result<PathBuf, String> {
    let temp = temp_dir()?;
    let archive_path = temp.join("update_download");

    let (progress_tx, _progress_rx) = std::sync::mpsc::channel::<f32>();
    download_file(download_url, &archive_path, &progress_tx)?;

    let valid = verify_sha256(&archive_path, sha256)?;
    if !valid {
        let _ = fs::remove_file(&archive_path);
        return Err("SHA256 校验失败".into());
    }

    let new_binary = extract_binary(&archive_path)?;
    let _ = fs::remove_file(&archive_path);
    Ok(new_binary)
}

pub fn replace_and_restart(new_binary: &Path) -> Result<(), String> {
    let current = current_exe_path()?;
    let parent = current.parent().ok_or("无法获取父目录")?.to_path_buf();

    // Sanity check BEFORE spawning any helper: the staged binary must
    // exist and be non-empty, otherwise we'd "restart" straight into the
    // old version with no diagnostics.
    let staged_len = fs::metadata(new_binary).map(|m| m.len()).unwrap_or(0);
    if staged_len == 0 {
        return Err("更新的二进制文件无效（空文件）".into());
    }

    // The downloaded binary already lives in the user-writable temp dir —
    // do NOT copy it into the install dir from here: the install dir
    // (Program Files, /usr/bin, /Applications) is usually NOT writable by
    // the running user, which failed with Permission denied. The helper
    // script performs the staging+swap with whatever privileges it can
    // obtain (UAC elevation on Windows, pkexec/sudo on Linux).

    #[cfg(target_os = "windows")]
    {
        // (No chmod on Windows: executables extracted from the zip are
        // runnable as-is; the Unix PermissionsExt API doesn't exist here.)

        let script_path = std::env::temp_dir().join("opennex_update.ps1");
        let script = r#"$ErrorActionPreference = 'Stop'
$cur = '@CUR@'
$new = '@NEW@'
$failMarker = '@FAIL@'
$procName = [IO.Path]::GetFileNameWithoutExtension($cur)
for ($i = 0; $i -lt 30; $i++) {
    $p = Get-Process -Name $procName -ErrorAction SilentlyContinue
    if (-not $p) { break }
    Start-Sleep -Seconds 1
}
try {
    Copy-Item -LiteralPath $new -Destination $cur -Force
} catch {
    [IO.File]::WriteAllText($failMarker, 'replace failed: ' + $_.Exception.Message)
    exit 1
}
Remove-Item -LiteralPath $new -Force -ErrorAction SilentlyContinue
Start-Process -FilePath $cur
Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue
"#
        .replace("@CUR@", &current.to_string_lossy())
        .replace("@NEW@", &new_binary.to_string_lossy())
        .replace(
            "@FAIL@",
            &parent
                .join("opennex_update_failed.txt")
                .to_string_lossy()
                .replace('\\', "/"),
        );
        fs::write(&script_path, script).map_err(|e| format!("写入脚本失败: {e}"))?;

        // Launch elevated (UAC): Program Files requires admin to write.
        // '-Verb RunAs' shows the standard elevation prompt.
        let launcher = format!(
            "Start-Process powershell -Verb RunAs -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-WindowStyle','Hidden','-File','{}'",
            script_path.to_string_lossy()
        );
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &launcher])
            .creation_flags_windows()
            .spawn()
            .map_err(|e| format!("启动更新脚本失败: {e}"))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(new_binary, fs::Permissions::from_mode(0o755));

        // Prefer a user-writable staging area: try the install dir first,
        // fall back to the data dir next to the config.
        let staged = parent.join("opennex_new");
        let staged = match fs::copy(new_binary, &staged).and_then(|_| {
            fs::set_permissions(&staged, fs::Permissions::from_mode(0o755)).map(|_| staged.clone())
        }) {
            Ok(p) => p,
            Err(_) => {
                // Cannot stage in the install dir (e.g. /usr/bin): the
                // script will elevate via pkexec/sudo for the swap.
                new_binary.to_path_buf()
            }
        };

        let script_path = std::env::temp_dir().join("opennex_update.sh");
        let script = format!(
            // Try the swap as the user first; if permission denied,
            // escalate via pkexec (GUI) or sudo. Success is verified by
            // comparing file sizes before relaunching.
            r#"#!/bin/bash
for i in $(seq 1 30); do
  kill -0 {pid} 2>/dev/null || break
  sleep 1
done
want_size=$(stat -c %s "{staged}" 2>/dev/null || stat -f %z "{staged}")
if ! mv -f "{staged}" "{current}" 2>>"{log}"; then
  if command -v pkexec >/dev/null 2>&1; then
    pkexec mv -f "{staged}" "{current}" >>"{log}" 2>&1
  elif command -v sudo >/dev/null 2>&1; then
    sudo -n mv -f "{staged}" "{current}" >>"{log}" 2>&1
  fi
fi
got_size=$(stat -c %s "{current}" 2>/dev/null || stat -f %z "{current}")
if [ -n "$want_size" ] && [ "$want_size" = "$got_size" ]; then
  chmod +x "{current}" 2>>"{log}"
  nohup "{current}" >/dev/null 2>&1 &
  rm -f -- "$0"
else
  echo "update replace failed (no permission)" >> "{log}"
  rm -f -- "$0"
fi
"#,
            pid = std::process::id(),
            staged = staged.to_string_lossy(),
            current = current.to_string_lossy(),
            log = std::env::temp_dir()
                .join("opennex_update.log")
                .to_string_lossy(),
        );

        fs::write(&script_path, &script).map_err(|e| format!("写入脚本失败: {e}"))?;
        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("设置权限失败: {e}"))?;

        std::process::Command::new("bash")
            .arg(&script_path)
            .spawn()
            .map_err(|e| format!("启动脚本失败: {e}"))?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
trait CreateFlagsWindows {
    fn creation_flags_windows(&mut self) -> &mut Self;
}
#[cfg(target_os = "windows")]
impl CreateFlagsWindows for std::process::Command {
    fn creation_flags_windows(&mut self) -> &mut Self {
        // DETACHED_PROCESS: the helper must survive our exit and must not
        // flash a console window in GUI-subsystem builds.
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        use std::os::windows::process::CommandExt;
        self.creation_flags(DETACHED_PROCESS);
        self
    }
}
#[cfg(not(target_os = "windows"))]
trait CreateFlagsWindows {
    fn creation_flags_windows(&mut self) -> &mut Self {
        self
    }
}
#[cfg(not(target_os = "windows"))]
impl CreateFlagsWindows for std::process::Command {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison() {
        assert!(version_is_newer("0.2.0", "0.1.0"));
        assert!(version_is_newer("1.0.0", "0.99.99"));
        assert!(!version_is_newer("0.1.0", "0.1.0"));
        assert!(!version_is_newer("0.0.9", "0.1.0"));
        assert!(version_is_newer("v0.2.0", "0.1.0"));
    }
}
