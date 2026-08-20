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

    // Stage a durable copy next to the current binary. The downloaded
    // file lives in the system temp dir which may be cleaned (or lost
    // across a reboot) between "download ready" and "user clicks
    // restart"; the update must not depend on that lifetime.
    let staged = parent.join(if cfg!(windows) {
        "opennex_new.exe"
    } else {
        "opennex_new"
    });
    fs::copy(new_binary, &staged).map_err(|e| format!("暂存更新文件失败: {e}"))?;

    #[cfg(target_os = "windows")]
    {
        let script_path = parent.join("opennex_update.bat");
        let script = format!(
            // Wait for the OLD process to actually exit before replacing:
            // a running exe is locked and a blind copy would fail (the
            // silent failure mode that restarted the OLD version).
            // Retry the replacement for up to ~30s, then verify the file
            // was swapped before launching.
            r#"@echo off
set /a tries=0
:wait_exit
tasklist /FI "IMAGENAME eq {cur_name}" 2>nul | find /I "{cur_name}" >nul
if %errorlevel%==0 (
  set /a tries+=1
  if %tries% geq 30 goto fail
  timeout /t 1 /nobreak >nul
  goto wait_exit
)
copy /y "{staged}" "{current}" >nul 2>&1
if not errorlevel 1 (
  del /f /q "{staged}" >nul 2>&1
  start "" "{current}"
  del /f /q "%~f0" >nul 2>&1
  exit /b 0
)
:fail
echo OpenNex update failed to replace the binary. > "{parent}\opennex_update_failed.txt"
del /f /q "%~f0" >nul 2>&1
"#,
            cur_name = current
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or("文件名错误")?,
            staged = staged.to_string_lossy().replace('\\', "/"),
            current = current.to_string_lossy().replace('\\', "/"),
            parent = parent.to_string_lossy().replace('\\', "/"),
        );

        fs::write(&script_path, script).map_err(|e| format!("写入脚本失败: {e}"))?;

        std::process::Command::new("cmd")
            .arg("/C")
            .arg(&script_path)
            .creation_flags_windows()
            .spawn()
            .map_err(|e| format!("启动脚本失败: {e}"))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        // Test write permission to the install dir first: system packages
        // (e.g. /usr/bin from .deb) are root-owned and a plain user run
        // cannot mv over them — report that instead of silently
        // restarting the old binary.
        let probe = parent.join(".opennex_write_probe");
        let writable = fs::write(&probe, b"").is_ok();
        let _ = fs::remove_file(&probe);
        if !writable {
            let _ = fs::remove_file(&staged);
            return Err(format!(
                "无权替换安装目录中的程序（{parent}）。请重新下载安装包更新，或以管理员权限运行。",
                parent = parent.display()
            ));
        }

        let script_path = parent.join("opennex_update.sh");
        let script = format!(
            // Wait for the old PID to exit (binary may be busy briefly),
            // then atomic mv + exec. mktemp log for diagnostics.
            r#"#!/bin/bash
for i in $(seq 1 30); do
  kill -0 {pid} 2>/dev/null || break
  sleep 1
done
if mv -f "{staged}" "{current}" 2>>"{parent}/opennex_update.log"; then
  chmod +x "{current}"
  rm -f -- "$0"
  exec "{current}"
else
  echo "update replace failed" >> "{parent}/opennex_update.log"
  rm -f -- "$0"
fi
"#,
            pid = std::process::id(),
            staged = staged.to_string_lossy(),
            current = current.to_string_lossy(),
            parent = parent.to_string_lossy(),
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
