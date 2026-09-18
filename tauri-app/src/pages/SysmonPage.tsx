import { useI18n } from '../i18n-context';
import { useEffect, useRef, useState } from "react";
import { FiCpu, FiTerminal, FiGrid, FiWatch } from "react-icons/fi";
import { invoke } from "../terminal/tauri";
import { focusedSlot } from "../terminal/registry";

interface Stat {
  cpu: number;
  mem: number;
}

interface Stats {
  focused: Stat | null;
  workspace: Stat | null;
  app: Stat | null;
}

function fmtMem(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
  if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  return `${(bytes / 1024).toFixed(0)} KB`;
}

/** 视图 > 系统资源 (egui parity): three rows sampled every 2s — the
 * focused terminal's process tree, the active workspace's terminals,
 * and the whole software (the app tree covers every shell + UI). */
export default function SysmonPage({ getWsSlots }: { getWsSlots: () => number[] }) {
  const T = useI18n();
  const [data, setData] = useState<Stats | null>(null);
  const [tick, setTick] = useState(0);
  const wsRef = useRef(getWsSlots);
  wsRef.current = getWsSlots;

  useEffect(() => {
    let alive = true;
    const poll = () => {
      // Session SLOT ids (strings) — the backend maps them to shell pids.
      const focused = focusedSlot.value ? [String(focusedSlot.value)] : [];
      const workspace = wsRef.current().map(String);
      invoke<Stats>("resource_stats", { focused, workspace })
        .then((s) => alive && setData(s))
        .catch(() => {});
    };
    poll();
    const id = window.setInterval(() => {
      poll();
      setTick((t) => t + 1);
    }, 2000);
    return () => {
      alive = false;
      window.clearInterval(id);
    };
  }, []);

  const rows: { key: string; label: string; icon: JSX.Element; stat: Stat | null; dim?: string }[] = [
    { key: "focused", label: T.uCurrentTerminal, icon: <FiTerminal size={14} />, stat: data?.focused ?? null },
    { key: "workspace", label: T.uCurrentWorkspace, icon: <FiGrid size={14} />, stat: data?.workspace ?? null },
    { key: "app", label: T.uGlobal, icon: <FiCpu size={14} />, stat: data?.app ?? null },
  ];

  return (
    <div className="h-full min-w-0 overflow-y-auto p-3" key={tick % 2}>
      <div className="w-full min-w-0">
        <h2 className="mb-1 flex items-center gap-2 text-[15px] font-semibold">
          <FiWatch size={15} className="text-[var(--accent)]" /> {T.sysmon}
        </h2>
        <p className="mb-4 text-[11px] text-[var(--text-faint)]">
          {T.uResourceHint}
        </p>
        <div className="space-y-3">
          {rows.map((r) => {
            const cpu = r.stat?.cpu ?? null;
            const frac = cpu === null ? 0 : Math.min(cpu / 100, 1);
            const hot = (cpu ?? 0) >= 80;
            return (
              <div key={r.key} className="card-glow rounded-xl border border-[var(--border)] bg-[var(--bg-panel)] p-4">
                <div className="mb-2 flex items-center gap-2 text-[12.5px] font-semibold">
                  <span className="text-[var(--text-dim)]">{r.icon}</span>
                  {r.label}
                  {r.stat ? (
                    <span
                      className="ml-auto font-mono text-[13px] font-bold"
                      style={{ color: hot ? "var(--danger)" : "var(--accent)" }}
                    >
                      {cpu!.toFixed(1)}%
                    </span>
                  ) : (
                    <span className="ml-auto font-mono text-[12px] text-[var(--text-faint)]">—</span>
                  )}
                </div>
                <div className="mb-2 h-1.5 overflow-hidden rounded-full bg-[var(--bg-active)]">
                  <div
                    className="h-full rounded-full transition-all duration-500"
                    style={{
                      width: `${frac * 100}%`,
                      background: hot ? "var(--danger)" : "var(--accent)",
                    }}
                  />
                </div>
                <div className="flex justify-between font-mono text-[11px] text-[var(--text-dim)]">
                  <span>CPU {cpu === null ? "—" : `${cpu.toFixed(1)}%`}</span>
                  <span>{T.uMemory} {r.stat ? fmtMem(r.stat.mem) : "—"}</span>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
