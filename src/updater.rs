use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const UPDATE_URL: &str = "https://opennex.download.zeadix.com/latest.json";
const UPDATE_SIG_URL: &str = "https://opennex.download.zeadix.com/latest.json.sig";

/// Ed25519 public key whose private half lives ONLY in the release CI
/// (`OPENNEX_UPDATE_SIGNING_KEY` GitHub secret). The manifest is signed
/// over its raw bytes; a matching `latest.json.sig` must be present and
/// valid before any update is offered. Rotate by replacing this const
/// together with the secret.
const MANIFEST_SIGNING_PUBKEY: [u8; 32] = [
    0x51, 0xe8, 0xb1, 0xe5, 0x21, 0x93, 0x42, 0xd2, 0x10, 0x17, 0x94, 0x38, 0x49, 0xae, 0xad, 0x3e,
    0x4c, 0x28, 0x92, 0xa8, 0x73, 0x8e, 0x9a, 0xb6, 0x8c, 0xf1, 0x0b, 0x76, 0x3d, 0xac, 0x1d, 0x46,
];

/// Fetch the detached manifest signature and verify it against the raw
/// manifest bytes. Unsigned or forged manifests are rejected — sha256
/// alone only protects against transfer corruption, not against a
/// compromised CDN/bucket/CI. Set OPENNEX_ALLOW_UNSIGNED_UPDATES=1 to
/// bypass for local testing (logged loudly).
fn verify_manifest_signature(manifest_bytes: &[u8]) -> Result<(), String> {
    if std::env::var("OPENNEX_ALLOW_UNSIGNED_UPDATES").as_deref() == Ok("1") {
        log::warn!("update signature check bypassed via OPENNEX_ALLOW_UNSIGNED_UPDATES");
        return Ok(());
    }
    let sig_b64 = ureq::get(UPDATE_SIG_URL)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|_| {
            "更新清单缺少签名文件 (latest.json.sig)，为防投毒已拒绝本次更新。请到官网手动下载。"
                .to_string()
        })?
        .into_string()
        .map_err(|_| "签名文件读取失败".to_string())?;
    let sig_b64_clean: String = sig_b64.chars().filter(|c| !c.is_whitespace()).collect();
    verify_signature_with(manifest_bytes, &sig_b64_clean, &MANIFEST_SIGNING_PUBKEY)
}

/// Pure verification core (key injected so tests can exercise the logic
/// without holding the release private key).
fn verify_signature_with(
    manifest_bytes: &[u8],
    sig_b64: &str,
    pubkey: &[u8; 32],
) -> Result<(), String> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    let sig_bytes = base64_decode(sig_b64).ok_or("签名 base64 解码失败")?;
    let sig = Signature::from_slice(&sig_bytes).map_err(|_| "签名格式非法".to_string())?;
    let key = VerifyingKey::from_bytes(pubkey).expect("public key bytes are valid");
    key.verify(manifest_bytes, &sig)
        .map_err(|_| "更新清单签名校验失败，可能被篡改。请到官网手动下载。".to_string())
}

#[cfg(test)]
mod signing_tests {
    use super::*;

    #[test]
    fn signature_roundtrip_and_tamper_rejection() {
        use base64::Engine;
        // Local throwaway keypair: proves the verifier accepts a valid
        // detached signature over the exact manifest bytes and rejects
        // any tampering. The RELEASE key is the embedded const.
        use ed25519_dalek::Signer;
        let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
        let vk = sk.verifying_key().to_bytes();
        let manifest = br#"{"version":"9.9.9"}"#;
        let sig = sk.sign(manifest);
        let sig_b64 = base64::engine::general_purpose::STANDARD.encode(sig.to_bytes());
        assert!(verify_signature_with(manifest, &sig_b64, &vk).is_ok());
        let tampered = br#"{"version":"0.0.1"}"#;
        assert!(verify_signature_with(tampered, &sig_b64, &vk).is_err());
        // Wrong key must also fail.
        assert!(verify_signature_with(manifest, &sig_b64, &[0u8; 32]).is_err());
    }

    #[test]
    fn embedded_pubkey_matches_committed_public_pem() {
        // research/formalization/audit/update-signing-pub.pem holds the
        // matching SPKI PEM; keep them in sync on rotation.
        assert_eq!(
            MANIFEST_SIGNING_PUBKEY,
            [
                0x51, 0xe8, 0xb1, 0xe5, 0x21, 0x93, 0x42, 0xd2, 0x10, 0x17, 0x94, 0x38, 0x49, 0xae,
                0xad, 0x3e, 0x4c, 0x28, 0x92, 0xa8, 0x73, 0x8e, 0x9a, 0xb6, 0x8c, 0xf1, 0x0b, 0x76,
                0x3d, 0xac, 0x1d, 0x46,
            ]
        );
    }
}

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
    /// Legacy single-arch fields (historically x86_64); still required so
    /// old manifests keep parsing.
    pub portable: String,
    pub sha256: String,
    /// Arch-keyed entries (macOS universal manifests). Optional so the
    /// Windows/Linux variants and old manifests deserialize unchanged.
    #[serde(default)]
    pub portable_aarch64: Option<String>,
    #[serde(default)]
    pub sha256_aarch64: Option<String>,
    #[serde(default)]
    pub portable_x86_64: Option<String>,
    #[serde(default)]
    pub sha256_x86_64: Option<String>,
}

impl PlatformFile {
    /// Pick the artifact matching the RUNNING architecture (macOS ships
    /// both builds; the legacy `portable` field is the x86_64 fallback).
    pub fn pick_for_running_arch(&self) -> (&str, &str) {
        match std::env::consts::ARCH {
            "aarch64" => {
                if let (Some(url), Some(sha)) = (
                    self.portable_aarch64.as_deref(),
                    self.sha256_aarch64.as_deref(),
                ) {
                    return (url, sha);
                }
            }
            "x86_64" => {
                if let (Some(url), Some(sha)) = (
                    self.portable_x86_64.as_deref(),
                    self.sha256_x86_64.as_deref(),
                ) {
                    return (url, sha);
                }
            }
            _ => {}
        }
        (&self.portable, &self.sha256)
    }
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
    // Pre-releases (0.2.0-rc1, 1.0.0+build) never go out to the stable
    // update channel.
    if remote.contains('-') || remote.contains('+') {
        return false;
    }
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
    let raw = ureq::get(UPDATE_URL)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| format!("请求失败: {e}"))?
        .into_string()
        .map_err(|e| format!("读取更新清单失败: {e}"))?;
    verify_manifest_signature(raw.as_bytes())?;
    let resp: ReleaseInfo = serde_json::from_str(&raw).map_err(|e| format!("解析失败: {e}"))?;

    let platform = current_platform();
    let file = match platform {
        "windows" => resp.files.windows.as_ref(),
        "macos" => resp.files.macos.as_ref(),
        "linux" => resp.files.linux.as_ref(),
        _ => return Err("不支持的平台".into()),
    }
    .ok_or("当前平台没有对应的下载文件")?;
    let (download_url, sha256) = file.pick_for_running_arch();

    if version_is_newer(&resp.version, env!("CARGO_PKG_VERSION")) {
        Ok(Some(UpdateInfo {
            version: resp.version,
            download_url: download_url.to_string(),
            sha256: sha256.to_string(),
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
    // Unique per attempt: fixed-name files under a shared /tmp would be
    // symlink-attackable by other local users.
    let unique = uuid::Uuid::new_v4().simple().to_string();
    let dir = std::env::temp_dir().join(format!("opennex-dl-{unique}"));
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
    /// Hard ceiling on any single extracted file (512 MiB): a malicious
    /// or corrupted archive must not be able to exhaust the disk.
    const MAX_ENTRY_BYTES: u64 = 512 * 1024 * 1024;

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
            // Only the app binary is expected — no path traversal, no
            // arbitrary first-.exe pick from a forged archive.
            if !name.eq_ignore_ascii_case("opennex.exe") && !name.ends_with("/opennex.exe") {
                continue;
            }
            if entry.size() > MAX_ENTRY_BYTES {
                return Err("压缩包内文件过大，已拒绝解压".into());
            }
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
                if entry.size() > MAX_ENTRY_BYTES {
                    return Err("压缩包内文件过大，已拒绝解压".into());
                }
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

/// Per-update scratch directory with an unpredictable name. Scripts and
/// the staged binary live here so a same-user attacker cannot pre-place
/// a predictable path that later gets executed or elevated (Windows UAC
/// TOCTOU / Linux /tmp symlink games). The dir is removed by the helper
/// on completion; a leftover from a crashed run is swept at startup.
fn update_work_dir() -> Result<PathBuf, String> {
    let unique = uuid::Uuid::new_v4().simple().to_string();
    let dir = std::env::temp_dir().join(format!("opennex-update-{unique}"));
    fs::create_dir_all(&dir).map_err(|e| format!("创建更新临时目录失败: {e}"))?;
    Ok(dir)
}

#[allow(dead_code)] // Windows-only helper; compiled everywhere for tests
/// PowerShell single-quoted literal escaping ('' doubles a quote).
fn ps_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// POSIX shell single-quote escaping ('\'' splice).
fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Result marker lives in the USER-PRIVATE data dir, not world-writable
/// /tmp: fixed-name files under /tmp are symlink-attackable by other
/// local users; the data dir has the same per-user trust level as the
/// app config itself.
pub fn update_status_path() -> PathBuf {
    crate::app::app_data_dir().join("update.status")
}

/// Read (and clear) the result marker left by the last update helper.
/// Returns the failure reason if the previous in-app update failed to
/// replace the binary — the user would otherwise keep running the old
/// version with no indication anything went wrong.
pub fn take_last_update_failure() -> Option<String> {
    let path = update_status_path();
    let content = fs::read_to_string(&path).ok()?;
    let _ = fs::remove_file(&path);
    let content = content.trim();
    if let Some(reason) = content.strip_prefix("fail:") {
        return Some(reason.trim().to_string());
    }
    // "ok" (or garbage) — consumed and ignored.
    None
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

    // Fresh scratch dir for this update attempt: scripts + staged copy
    // live here under an unguessable name.
    let work_dir = update_work_dir()?;

    #[cfg(target_os = "windows")]
    {
        // Two-stage helper:
        //   launcher (unelevated): waits for exit → keeps a rollback
        //     copy of the running binary → tries the copy as the user
        //     (portable/zip installs) → only on denial elevates a
        //     minimal copier via UAC (-Wait, cancellation is catchable)
        //     → verifies the swap by hash → relaunches UNELEVATED via
        //     explorer.exe → writes the status marker → sweeps the
        //     scratch dir.
        //   copier (elevated): backup current + copy staged + hash only.
        let script_path = work_dir.join("update.ps1");
        let copier_path = work_dir.join("copier.ps1");
        let status_file = update_status_path();

        let script = r#"$ErrorActionPreference = 'Stop'
$cur = @CUR@
$new = '@NEW@'
$status = @STATUS@
$backup = "$cur.old"
$procName = [IO.Path]::GetFileNameWithoutExtension($cur)
"fail: helper crashed" | Out-File -FilePath $status -Encoding ascii
for ($i = 0; $i -lt 30; $i++) {
    $p = Get-Process -Name $procName -ErrorAction SilentlyContinue
    if (-not $p) { break }
    Start-Sleep -Seconds 1
}
function Test-SameFile($a, $b) {
    $ha = (Get-FileHash -LiteralPath $a -Algorithm SHA256).Hash
    $hb = (Get-FileHash -LiteralPath $b -Algorithm SHA256).Hash
    return ($ha -eq $hb)
}
$replaced = $false
$reason = ''
try {
    Copy-Item -LiteralPath $new -Destination $cur -Force
    if (Test-SameFile $new $cur) { $replaced = $true } else { $reason = 'user copy hash mismatch' }
} catch {
    $reason = $_.Exception.Message
}
if (-not $replaced) {
    # Needs admin (Program Files). -Wait lets us catch a DECLINED UAC
    # prompt as an exception instead of dying silently.
    try {
        $p = Start-Process powershell -Verb RunAs -WindowStyle Hidden -PassThru -Wait -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-WindowStyle','Hidden','-File','@COPIER@'
        if ($p.ExitCode -eq 0 -and (Test-SameFile $new $cur)) { $replaced = $true }
        else { $reason = "elevated copy failed (exit=$($p.ExitCode))" }
    } catch {
        $reason = 'UAC declined: ' + $_.Exception.Message
    }
}
if ($replaced) {
    # Relaunch UNELEVATED: explorer.exe spawns the target with the
    # shell's normal token (running the new version as admin caused
    # drag-drop and inherited-shell issues).
    Start-Process -FilePath explorer.exe -ArgumentList $cur
    'ok' | Out-File -FilePath $status -Encoding ascii
} else {
    ("fail: " + $reason) | Out-File -FilePath $status -Encoding ascii
}
Remove-Item -LiteralPath $new -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath '@COPIER@' -Force -ErrorAction SilentlyContinue
"#
        .replace("@CUR@", &ps_quote(&current.to_string_lossy()))
        .replace("@NEW@", &ps_quote(&new_binary.to_string_lossy()))
        .replace("@STATUS@", &ps_quote(&status_file.to_string_lossy()))
        .replace("@COPIER@", &ps_quote(&copier_path.to_string_lossy()));

        // Elevated stage: keep a one-generation rollback copy before the
        // swap so a bad release can be recovered manually by renaming
        // <exe>.old back over <exe>.
        let copier = r#"$ErrorActionPreference = 'Stop'
$cur = @CUR@
$new = '@NEW@'
$backup = "$cur.old"
Copy-Item -LiteralPath $cur -Destination $backup -Force
Copy-Item -LiteralPath $new -Destination $cur -Force
exit 0
"#
        .replace("@CUR@", &ps_quote(&current.to_string_lossy()))
        .replace("@NEW@", &ps_quote(&new_binary.to_string_lossy()));

        // The launcher itself performs the user-level rollback copy too
        // (portable installs are user-writable): insert it right before
        // its first Copy-Item attempt.
        let script = script.replace(
            "try {\r\n    Copy-Item -LiteralPath $new",
            "try {\r\n    Copy-Item -LiteralPath $cur -Destination $backup -Force\r\n    Copy-Item -LiteralPath $new",
        );
        let script = script.replace(
            "try {\n    Copy-Item -LiteralPath $new",
            "try {\n    Copy-Item -LiteralPath $cur -Destination $backup -Force\n    Copy-Item -LiteralPath $new",
        );

        fs::write(&script_path, script).map_err(|e| format!("写入脚本失败: {e}"))?;
        fs::write(&copier_path, copier).map_err(|e| format!("写入脚本失败: {e}"))?;

        // Clear any stale/symlinked status marker before the helper runs.
        let _ = fs::remove_file(&status_file);

        // The launcher itself must stay unelevated and windowless.
        std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-WindowStyle",
                "Hidden",
                "-File",
            ])
            .arg(&script_path)
            .creation_flags_windows()
            .spawn()
            .map_err(|e| format!("启动更新脚本失败: {e}"))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(new_binary, fs::Permissions::from_mode(0o755));

        // Stage INSIDE the private scratch dir — never in the (possibly
        // root-owned) install dir.
        let staged = work_dir.join("opennex_new");
        fs::copy(new_binary, &staged).map_err(|e| format!("暂存新版本失败: {e}"))?;
        fs::set_permissions(&staged, fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("设置权限失败: {e}"))?;

        // Pre-flight: probe DIRECTORY writability by creating a throwaway
        // file inside it (opening the exe itself answered the wrong
        // question and broke portable installs).
        let install_writable = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(parent.join(".opennex-write-probe"))
            .is_ok();
        if install_writable {
            let _ = fs::remove_file(parent.join(".opennex-write-probe"));
        } else {
            let has_helper = which("pkexec").is_some()
                || which("sudo").is_some()
                || (which("osascript").is_some());
            if !has_helper {
                return Err(format!(
                    "无权替换 {current}，且系统没有 pkexec/sudo/osascript 可提权。请下载安装包手动更新。",
                    current = current.display()
                ));
            }
        }

        let status_file = update_status_path();
        let log_file = work_dir.join("update.log");
        let script_path = work_dir.join("update.sh");

        // Rollback copy of the running binary happens BEFORE any swap
        // attempt; on total failure the message points at <exe>.old.
        let script = r#"#!/bin/bash
status_file=QSTATUS
log=QLOG
echo "fail: helper crashed" > "$status_file"
for i in $(seq 1 30); do
  kill -0 QPID 2>/dev/null || break
  sleep 1
done
want_hash=$(sha256sum QSTAGED 2>/dev/null | awk '{print $1}')
[ -z "$want_hash" ] && want_hash=$(shasum -a 256 QSTAGED 2>/dev/null | awk '{print $1}')
cp -p QCURRENT QCURRENT.old 2>>"$log"
swapped=0
if mv -f QSTAGED QCURRENT 2>>"$log"; then
  swapped=1
else
  if command -v pkexec >/dev/null 2>&1; then
    pkexec mv -f QSTAGED QCURRENT >>"$log" 2>&1 && swapped=1
  elif [ "$(uname)" = "Darwin" ] && command -v osascript >/dev/null 2>&1; then
    osascript -e "do shell script \"mv -f 'QSTAGED_ESC' 'QCURRENT_ESC'\" with administrator privileges" >>"$log" 2>&1 && swapped=1
  elif command -v sudo >/dev/null 2>&1; then
    sudo -n mv -f QSTAGED QCURRENT >>"$log" 2>&1 && swapped=1
  fi
fi
got_hash=$(sha256sum QCURRENT 2>/dev/null | awk '{print $1}')
[ -z "$got_hash" ] && got_hash=$(shasum -a 256 QCURRENT 2>/dev/null | awk '{print $1}')
if [ "$swapped" = "1" ] && [ -n "$want_hash" ] && [ "$want_hash" = "$got_hash" ]; then
  chmod +x QCURRENT 2>>"$log"
  echo "ok" > "$status_file"
  nohup QCURRENT >/dev/null 2>&1 &
  rm -f -- "$0"
else
  echo "fail: 替换失败（提权被取消或目录不可写）。旧版本已保留在 QCURRENT.old，可直接改回；或用安装包手动更新。" > "$status_file"
  echo "update replace failed" >> "$log"
  rm -f -- "$0"
fi
"#
        .replace("QSTATUS", &sh_quote(&status_file.to_string_lossy()))
        .replace("QLOG", &sh_quote(&log_file.to_string_lossy()))
        .replace("QPID", &std::process::id().to_string())
        .replace("QSTAGED_ESC", &staged.to_string_lossy())
        .replace("QCURRENT_ESC", &current.to_string_lossy())
        .replace("QSTAGED", &sh_quote(&staged.to_string_lossy()))
        .replace("QCURRENT", &sh_quote(&current.to_string_lossy()));

        fs::write(&script_path, script).map_err(|e| format!("写入脚本失败: {e}"))?;
        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("设置权限失败: {e}"))?;

        // Clear any stale/symlinked status marker before the helper runs.
        let _ = fs::remove_file(&status_file);

        std::process::Command::new("bash")
            .arg(&script_path)
            .spawn()
            .map_err(|e| format!("启动脚本失败: {e}"))?;
    }

    Ok(())
}

fn which(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
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
#[allow(dead_code)]
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

/// Minimal standard-base64 decoder (padding required) — avoids pulling
/// another crate for one call site.
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes: Vec<u8> = input.bytes().collect();
    if bytes.is_empty() || !bytes.len().is_multiple_of(4) {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let mut vals = [0u32; 4];
        for (i, &b) in chunk.iter().enumerate() {
            vals[i] = match b {
                b'=' => 0,
                _ => TABLE.iter().position(|&t| t == b)? as u32,
            };
        }
        let n = (vals[0] << 18) | (vals[1] << 12) | (vals[2] << 6) | vals[3];
        out.push((n >> 16) as u8);
        out.push((n >> 8) as u8);
        out.push(n as u8);
    }
    // Strip bytes introduced by '=' padding.
    let pad = input.bytes().rev().take_while(|&b| b == b'=').count();
    out.truncate(out.len() - pad);
    Some(out)
}

#[cfg(test)]
mod base64_tests {
    use super::base64_decode;

    #[test]
    fn decodes_standard_vectors() {
        assert_eq!(base64_decode("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(base64_decode("aGVsbG8h").unwrap(), b"hello!");
        assert_eq!(
            base64_decode("Ueix5SGTQtIQF5Q4Sa6tPkwokqhzjpq2jPELdj2sHUY=")
                .unwrap()
                .len(),
            32
        );
        assert!(base64_decode("!!!").is_none());
    }
}
