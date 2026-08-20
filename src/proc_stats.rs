//! Per-terminal CPU / memory accounting, aggregated over each shell's
//! whole process tree (the shell itself is idle; the cost lives in the
//! commands it runs).
//!
//! Sampling model: callers invoke [`sample`] on a fixed cadence (the app
//! already refreshes its status bar every 2 s). CPU% is computed from the
//! delta of consumed CPU time between consecutive samples; memory is the
//! sum of resident sizes at the current sample. Dead PIDs contribute
//! nothing and are simply absent from the platform tables.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// One process row as read from a platform table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcRow {
    pub pid: u32,
    pub ppid: u32,
}

/// Per-sample accumulator fed by platform readers.
#[derive(Debug, Default)]
pub struct ProcSample {
    /// pid -> (ppid, cpu_ticks, rss_bytes)
    pub procs: HashMap<u32, (u32, u64, u64)>,
}

impl ProcSample {
    pub fn insert(&mut self, pid: u32, ppid: u32, cpu_ticks: u64, rss_bytes: u64) {
        self.procs.insert(pid, (ppid, cpu_ticks, rss_bytes));
    }
}

/// Collect the set of PIDs reachable from `roots` via parent->child edges.
/// A root missing from the sample yields an empty tree (dead shell).
pub fn collect_tree(roots: &[u32], sample: &ProcSample) -> HashSet<u32> {
    let mut seen: HashSet<u32> = HashSet::new();
    let mut stack: Vec<u32> = Vec::new();
    for &root in roots {
        if sample.procs.contains_key(&root) {
            stack.push(root);
        }
    }
    while let Some(pid) = stack.pop() {
        if !seen.insert(pid) {
            continue;
        }
        if let Some(&(_, _, _)) = sample.procs.get(&pid) {
            for (&child, &(cppid, _, _)) in sample.procs.iter() {
                if cppid == pid && child != pid {
                    stack.push(child);
                }
            }
        }
    }
    seen
}

/// Sum CPU ticks and RSS bytes over the process trees rooted at `roots`.
pub fn aggregate(roots: &[u32], sample: &ProcSample) -> (u64, u64) {
    let tree = collect_tree(roots, sample);
    let mut ticks = 0u64;
    let mut rss = 0u64;
    for pid in tree {
        if let Some(&(_, t, m)) = sample.procs.get(&pid) {
            ticks += t;
            rss += m;
        }
    }
    (ticks, rss)
}

/// Convert a CPU-ticks delta between two samples into a percentage of one
/// logical core * 100 (i.e. 100.0 == one full core).
pub fn cpu_percent(delta_ticks: u64, elapsed_secs: f32, ticks_per_sec: u64) -> f32 {
    if elapsed_secs <= 0.0 || ticks_per_sec == 0 {
        return 0.0;
    }
    (delta_ticks as f32 / (elapsed_secs * ticks_per_sec as f32)) * 100.0
}

// ---------------------------------------------------------------------------
// Platform readers
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
pub fn ticks_per_sec() -> u64 {
    // Linux USER_HZ is 100 on every mainstream distro; sysconf(_SC_CLK_TCK)
    // would confirm it but libc adds nothing here in practice.
    100
}

#[cfg(target_os = "linux")]
pub fn read_sample() -> Option<ProcSample> {
    let mut sample = ProcSample::default();
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) }.max(1) as u64;
    for entry in std::fs::read_dir("/proc").ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name();
        let name = name.to_str()?;
        if !name.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let pid: u32 = match name.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        // stat: pid (comm) state ppid ... utime stime ...
        let stat = match std::fs::read_to_string(format!("/proc/{name}/stat")) {
            Ok(s) => s,
            Err(_) => continue, // raced with exit
        };
        let close = match stat.rfind(')') {
            Some(i) => i + 1,
            None => continue,
        };
        let mut fields = stat[close..].split_whitespace();
        let _state = fields.next();
        let ppid: u32 = match fields.next().and_then(|f| f.parse().ok()) {
            Some(v) => v,
            None => continue,
        };
        // fields 1-based after comm: state(3) ppid(4) ... utime(14) stime(15)
        let mut utime: Option<u64> = None;
        let mut stime: Option<u64> = None;
        for (i, f) in fields.enumerate() {
            // i==0 was state, 1 ppid; utime is overall field 14 => offset 11
            if i == 11 {
                utime = f.parse().ok();
            }
            if i == 12 {
                stime = f.parse().ok();
                break;
            }
        }
        let (Some(u), Some(s)) = (utime, stime) else {
            continue;
        };
        // statm: size resident shared ... (in pages)
        let rss = std::fs::read_to_string(format!("/proc/{name}/statm"))
            .ok()
            .and_then(|m| {
                let mut it = m.split_whitespace();
                it.next()?;
                let resident: u64 = it.next()?.parse().ok()?;
                Some(resident * page_size)
            })
            .unwrap_or(0);
        sample.insert(pid, ppid, u + s, rss);
    }
    Some(sample)
}

#[cfg(target_os = "macos")]
pub fn ticks_per_sec() -> u64 {
    unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.max(1) as u64
}

#[cfg(target_os = "macos")]
pub fn read_sample() -> Option<ProcSample> {
    use std::mem;
    // Two-call sysctl pattern for the full kinfo_proc table.
    let mut size = 0u8;
    let mut mib = [libc::CTL_KERN, libc::KERN_PROC, libc::KERN_PROC_ALL];
    let name = mib.as_mut_ptr();
    unsafe {
        if libc::sysctl(
            name,
            3,
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null(),
            0,
        ) != 0
        {
            return None;
        }
        if size == 0 {
            return Some(ProcSample::default());
        }
        let mut buf: Vec<libc::c_void> = Vec::with_capacity(size);
        loop {
            if libc::sysctl(name, 3, buf.as_mut_ptr(), &mut size, std::ptr::null(), 0) != 0 {
                return None;
            }
            if size <= buf.capacity() {
                break;
            }
            buf.reserve_exact(size - buf.capacity());
        }
        let count = size / mem::size_of::<libc::kinfo_proc>();
        let procs = std::slice::from_raw_parts(buf.as_ptr() as *const libc::kinfo_proc, count);
        let mut sample = ProcSample::default();
        for kp in procs {
            let info = &kp.kp_proc;
            let pid = info.p_pid as u32;
            let ppid = kp.kp_eproc.e_ppid as u32;
            // p_rusage is only valid for zombies/exited on some versions;
            // live procs carry it too on modern macOS.
            let ru = &info.p_ru;
            if ru.is_null() {
                continue;
            }
            let user = ru.ru_utime.tv_sec as u64 * ticks_per_sec()
                + ru.ru_utime.tv_usec as u64 * ticks_per_sec() / 1_000_000;
            let sys = ru.ru_stime.tv_sec as u64 * ticks_per_sec()
                + ru.ru_stime.tv_usec as u64 * ticks_per_sec() / 1_000_000;
            let rss = ru.ru_maxrss; // bytes on macOS
            sample.insert(pid, ppid, user + sys, rss);
        }
        Some(sample)
    }
}

#[cfg(target_os = "windows")]
pub fn ticks_per_sec() -> u64 {
    100 // GetProcessTimes returns 100ns units; converted at read time.
}

#[cfg(target_os = "windows")]
pub fn read_sample() -> Option<ProcSample> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::OpenProcess;

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return None;
        }
        let mut sample = ProcSample::default();
        let mut row: PROCESSENTRY32W = std::mem::zeroed();
        row.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snap, &mut row) != 0 {
            loop {
                let pid = row.th32ProcessID;
                let ppid = row.th32ParentProcessID;
                if let Some((ticks, rss)) = windows_query_process(pid) {
                    sample.insert(pid, ppid, ticks, rss);
                }
                if Process32NextW(snap, &mut row) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
        Some(sample)
    }
}

#[cfg(target_os = "windows")]
unsafe fn windows_query_process(pid: u32) -> Option<(u64, u64)> {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::Diagnostics::Debug::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
    };
    use windows_sys::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let h: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h == 0 {
            return None;
        }
        let mut create = std::mem::zeroed();
        let mut exit = std::mem::zeroed();
        let mut kernel = std::mem::zeroed();
        let mut user = std::mem::zeroed();
        let mut ticks = 0u64;
        if GetProcessTimes(h, &mut create, &mut exit, &mut kernel, &mut user) != 0 {
            // FILETIME is 100ns units; report in centiseconds (100/s) so
            // ticks_per_sec() == 100 keeps cpu_percent uniform.
            let k = ((*kernel.dwHighDateTime as u64) << 32) | *kernel.dwLowDateTime as u64;
            let u = ((*user.dwHighDateTime as u64) << 32) | *user.dwLowDateTime as u64;
            ticks = (k + u) / 10_000;
        }
        let mut pmc: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
        let mut rss = 0u64;
        if GetProcessMemoryInfo(
            h,
            &mut pmc,
            std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        ) != 0
        {
            rss = pmc.WorkingSetSize as u64;
        }
        CloseHandle(h);
        if ticks == 0 && rss == 0 {
            return None;
        }
        Some((ticks, rss))
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn ticks_per_sec() -> u64 {
    1
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn read_sample() -> Option<ProcSample> {
    None
}

// ---------------------------------------------------------------------------
// Sampler state
// ---------------------------------------------------------------------------

/// Cadence-driven sampler keeping the previous sample for CPU deltas.
pub struct ProcSampler {
    last: Option<(Instant, ProcSample)>, // (when, sample)
    pub last_cpu_percent: Option<f32>,
    pub last_mem_bytes: Option<u64>,
}

// ---------------------------------------------------------------------------
// Network throughput (system-wide RX/TX rates)
// ---------------------------------------------------------------------------

/// Cumulative system-wide network counters in bytes.
#[derive(Debug, Clone, Copy, Default)]
pub struct NetCounters {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

#[cfg(target_os = "linux")]
pub fn read_net_counters() -> Option<NetCounters> {
    let content = std::fs::read_to_string("/proc/net/dev").ok()?;
    let mut c = NetCounters::default();
    for line in content.lines().skip(2) {
        let (iface, data) = line.split_once(':')?;
        let iface = iface.trim();
        // Skip virtual interfaces so we don't double-count.
        if iface.starts_with("lo")
            || iface.starts_with("docker")
            || iface.starts_with("veth")
            || iface.starts_with("br-")
            || iface.starts_with("virbr")
        {
            continue;
        }
        let fields: Vec<&str> = data.split_whitespace().collect();
        if fields.len() >= 9 {
            c.rx_bytes += fields[0].parse().unwrap_or(0);
            c.tx_bytes += fields[8].parse().unwrap_or(0);
        }
    }
    Some(c)
}

#[cfg(target_os = "macos")]
pub fn read_net_counters() -> Option<NetCounters> {
    // netstat -ib -n: summed per-interface counters, first row per iface.
    let out = std::process::Command::new("netstat")
        .args(["-ib", "-n"])
        .output()
        .ok()?;
    let mut seen: std::collections::HashSet<String> = Default::default();
    let mut c = NetCounters::default();
    for line in String::from_utf8_lossy(&out.stdout).lines().skip(1) {
        let f: Vec<&str> = line.split_whitespace().collect();
        // Name Mtu Network Address Ipkts Ierrs Ibytes Opkts Oerrs Obytes ...
        if f.len() >= 10 {
            let name = f[0];
            if name.starts_with("lo") || seen.contains(name) {
                continue;
            }
            seen.insert(name.to_string());
            c.rx_bytes += f[6].parse().unwrap_or(0);
            c.tx_bytes += f[9].parse().unwrap_or(0);
        }
    }
    Some(c)
}

#[cfg(target_os = "windows")]
pub fn read_net_counters() -> Option<NetCounters> {
    // Type-perf-free approach: parse `netstat -e` (works on all Windows).
    let out = std::process::Command::new("netstat")
        .arg("-e")
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut c = NetCounters::default();
    let mut bytes_section = 0; // 0=none,1=received,2=sent
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with("Received") || l.starts_with("已接收") {
            bytes_section = 1;
            continue;
        }
        if l.starts_with("Sent") || l.starts_with("已发送") {
            bytes_section = 2;
            continue;
        }
        if bytes_section > 0 {
            // Counter rows: "Bytes <n>" possibly grouped with spaces.
            if let Some(v) = l.strip_prefix("Bytes").or_else(|| l.strip_prefix("字节")) {
                if let Ok(n) = v.trim().replace(',', "").parse::<u64>() {
                    match bytes_section {
                        1 => {
                            c.rx_bytes = n;
                            bytes_section = 0;
                        }
                        2 => {
                            c.tx_bytes = n;
                            bytes_section = 0;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Some(c)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn read_net_counters() -> Option<NetCounters> {
    None
}

/// Sampler that converts cumulative counters into RX/TX rates in bytes/sec.
pub struct NetSampler {
    last: Option<(Instant, NetCounters)>,
    pub rx_rate: Option<f64>,
    pub tx_rate: Option<f64>,
}

impl Default for NetSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl NetSampler {
    pub fn new() -> Self {
        Self {
            last: None,
            rx_rate: None,
            tx_rate: None,
        }
    }

    pub fn refresh(&mut self) {
        let Some(now) = read_net_counters() else {
            return;
        };
        if let Some((when, prev)) = &self.last {
            let elapsed = when.elapsed().as_secs_f64();
            if elapsed > 0.05 {
                self.rx_rate = Some((now.rx_bytes.saturating_sub(prev.rx_bytes)) as f64 / elapsed);
                self.tx_rate = Some((now.tx_bytes.saturating_sub(prev.tx_bytes)) as f64 / elapsed);
            }
        }
        self.last = Some((Instant::now(), now));
    }
}

/// Human-readable rate, e.g. `12.3 MB/s`.
pub fn format_rate(bytes_per_sec: Option<f64>) -> String {
    match bytes_per_sec {
        Some(r) => {
            const KB: f64 = 1024.0;
            const MB: f64 = KB * 1024.0;
            const GB: f64 = MB * 1024.0;
            if r >= GB {
                format!("{:.2} GB/s", r / GB)
            } else if r >= MB {
                format!("{:.1} MB/s", r / MB)
            } else if r >= KB {
                format!("{:.1} KB/s", r / KB)
            } else {
                format!("{:.0} B/s", r)
            }
        }
        None => "--".into(),
    }
}

impl Default for ProcSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcSampler {
    pub fn new() -> Self {
        Self {
            last: None,
            last_cpu_percent: None,
            last_mem_bytes: None,
        }
    }

    /// Recompute aggregates for the process trees rooted at `roots`.
    /// Call on a fixed cadence; the first call only establishes a baseline
    /// (memory is reported immediately, CPU appears from the second call).
    pub fn refresh(&mut self, roots: &[u32]) {
        let tps = ticks_per_sec();
        let Some(sample) = read_sample() else {
            return;
        };
        let (ticks, rss) = aggregate(roots, &sample);
        self.last_mem_bytes = Some(rss);
        if let Some((when, prev_sample)) = &self.last {
            let (prev_tree_ticks, _) = aggregate(roots, prev_sample);
            let elapsed = when.elapsed().as_secs_f32();
            let delta = ticks.saturating_sub(prev_tree_ticks);
            self.last_cpu_percent = Some(cpu_percent(delta, elapsed, tps));
        }
        self.last = Some((Instant::now(), sample));
    }

    /// Single-table variant: refresh several root groups against one
    /// snapshot, reporting each group's cpu% and memory through the
    /// out-params. Saves re-reading the process table per group.
    pub fn refresh_groups<const N: usize>(
        &mut self,
        groups: [&[u32]; N],
        out_cpu: [&mut Option<f32>; N],
        out_mem: [&mut Option<u64>; N],
    ) {
        let tps = ticks_per_sec();
        let Some(sample) = read_sample() else {
            return;
        };
        for i in 0..N {
            let (ticks, rss) = aggregate(groups[i], &sample);
            *out_mem[i] = Some(rss);
            if let Some((when, prev_sample)) = &self.last {
                let (prev_tree_ticks, _) = aggregate(groups[i], prev_sample);
                let elapsed = when.elapsed().as_secs_f32();
                let delta = ticks.saturating_sub(prev_tree_ticks);
                *out_cpu[i] = Some(cpu_percent(delta, elapsed, tps));
            }
        }
        self.last = Some((Instant::now(), sample));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_from(rows: &[(u32, u32, u64, u64)]) -> ProcSample {
        let mut s = ProcSample::default();
        for &(pid, ppid, t, m) in rows {
            s.insert(pid, ppid, t, m);
        }
        s
    }

    #[test]
    fn tree_collection_follows_children_not_siblings() {
        // bash(1) -> cargo(2) -> rustc(3); unrelated sh(9) under init.
        let s = sample_from(&[(1, 0, 0, 0), (2, 1, 0, 0), (3, 2, 0, 0), (9, 0, 0, 0)]);
        let tree = collect_tree(&[1], &s);
        assert!(tree.contains(&1) && tree.contains(&2) && tree.contains(&3));
        assert!(!tree.contains(&9));
    }

    #[test]
    fn aggregation_sums_only_tree_members() {
        let s = sample_from(&[
            (1, 0, 10, 1_000),
            (2, 1, 20, 2_000),
            (3, 2, 30, 4_000),
            (9, 0, 99, 99_000),
        ]);
        let (ticks, rss) = aggregate(&[1], &s);
        assert_eq!(ticks, 60);
        assert_eq!(rss, 7_000);
    }

    #[test]
    fn multiple_roots_union() {
        let s = sample_from(&[(1, 0, 5, 100), (7, 0, 6, 200), (3, 1, 7, 300)]);
        let (ticks, rss) = aggregate(&[1, 7], &s);
        assert_eq!(ticks, 18);
        assert_eq!(rss, 600);
    }

    #[test]
    fn dead_root_contributes_nothing() {
        let s = sample_from(&[(2, 1, 50, 500)]); // root 1 itself missing
        let tree = collect_tree(&[1], &s);
        assert!(!tree.contains(&1));
        let (ticks, rss) = aggregate(&[1], &s);
        assert_eq!((ticks, rss), (0, 0));
    }

    #[test]
    fn cpu_percent_math() {
        assert!((cpu_percent(200, 2.0, 100) - 100.0).abs() < 1e-4);
        assert_eq!(cpu_percent(0, 2.0, 100), 0.0);
        assert_eq!(cpu_percent(100, 0.0, 100), 0.0);
    }

    #[test]
    fn group_aggregates_are_independent() {
        // Group A roots {1}, group B roots {1,7}: B includes A plus the
        // extra root, so B's totals are a superset of A's.
        let s = sample_from(&[(1, 0, 5, 100), (7, 0, 6, 200), (3, 1, 7, 300)]);
        let ga = [1u32];
        let gb = [1u32, 7u32];
        let (ta, ma) = aggregate(&ga, &s);
        let (tb, mb) = aggregate(&gb, &s);
        assert_eq!((ta, ma), (12, 400));
        assert_eq!((tb, mb), (18, 600));
        assert!(tb >= ta && mb >= ma);
    }

    #[test]
    fn sampler_first_refresh_sets_memory_baseline_only() {
        let mut sampler = ProcSampler::new();
        // Can't portably read the live table in a unit test; drive the
        // pure half instead.
        let s = sample_from(&[(1, 0, 42, 4096)]);
        let (ticks, rss) = aggregate(&[1], &s);
        assert_eq!(ticks, 42);
        assert_eq!(rss, 4096);
        assert!(sampler.last_cpu_percent.is_none());
        assert!(sampler.last_mem_bytes.is_none());
    }
}
