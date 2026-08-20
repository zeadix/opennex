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
    // Parse `ps -axo pid=,ppid=,%cpu=,rss=`. The libc crate does NOT ship
    // a kinfo_proc type for macOS (only the constants), so a sysctl
    // KERN_PROC_ALL table walk would need hand-declared ABI structs; the
    // ps utility gives the same tree data with zero FFI risk.
    let out = std::process::Command::new("ps")
        .args(["-axo", "pid=,ppid=,%cpu=,rss="])
        .output()
        .ok()?;
    let mut sample = ProcSample::default();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let mut it = line.split_whitespace();
        let (Some(pid), Some(ppid), Some(cpu), Some(rss_kb)) =
            (it.next(), it.next(), it.next(), it.next())
        else {
            continue;
        };
        let (Ok(pid), Ok(ppid), Ok(cpu), Ok(rss_kb)) = (
            pid.parse::<u32>(),
            ppid.parse::<u32>(),
            cpu.parse::<f32>(),
            rss_kb.parse::<u64>(),
        ) else {
            continue;
        };
        // ps %cpu on macOS is percent of one core; store as
        // centisecond-ticks-per-second equivalent (x100) so the delta math
        // stays uniform: ticks = cpu_percent * ticks_per_sec().
        let ticks = (cpu * ticks_per_sec() as f32) as u64;
        sample.insert(pid, ppid, ticks, rss_kb * 1024);
    }
    Some(sample)
}

#[cfg(target_os = "windows")]
pub fn ticks_per_sec() -> u64 {
    // FILETIME deltas are converted to centiseconds in
    // windows_query_process, so ticks-per-second is 100.
    100
}

#[cfg(target_os = "windows")]
pub fn read_sample() -> Option<ProcSample> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return None;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut sample = ProcSample::default();
        let mut more = Process32FirstW(snap, &mut entry);
        while more != 0 {
            let pid = entry.th32ProcessID;
            let ppid = entry.th32ParentProcessID;
            if pid != 0 {
                if let Some((ticks, rss)) = windows_query_process(pid) {
                    sample.insert(pid, ppid, ticks, rss);
                }
            }
            more = Process32NextW(snap, &mut entry);
        }
        CloseHandle(snap);
        Some(sample)
    }
}

#[cfg(target_os = "windows")]
unsafe fn windows_query_process(pid: u32) -> Option<(u64, u64)> {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    // GetProcessMemoryInfo / PROCESS_MEMORY_COUNTERS live in
    // System::ProcessStatus (not Diagnostics::Debug) in windows-sys 0.59.
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
    };
    use windows_sys::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let h: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        // HANDLE is *mut c_void in windows-sys 0.59: null-check via is_null.
        if h.is_null() {
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
            // FILETIME fields are plain u32 (no pointer dereference).
            let k = ((kernel.dwHighDateTime as u64) << 32) | kernel.dwLowDateTime as u64;
            let u = ((user.dwHighDateTime as u64) << 32) | user.dwLowDateTime as u64;
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
