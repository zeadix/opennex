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

const MAX_POINTS = 60;

/** 曲线图（官网监控面板样式）：accent 折线 + 近 60 个采样点，过热转 danger。 */
function Sparkline({ points, hot }: { points: number[]; hot: boolean }) {
  const w = 240;
  const h = 44;
  if (points.length < 2) {
    return (
      <svg viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="none" className="h-12 w-full rounded-md bg-[var(--bg)]" />
    );
  }
  const max = Math.max(100, ...points);
  const step = w / (MAX_POINTS - 1);
  const path = points
    .map((p, i) => `${i === 0 ? "M" : "L"}${(i * step).toFixed(1)},${(h - (p / max) * (h - 8) - 4).toFixed(1)}`)
    .join(" ");
  return (
    <svg
      viewBox={`0 0 ${w} ${h}`}
      preserveAspectRatio="none"
      className="h-12 w-full rounded-md border border-[var(--bg-active)] bg-[var(--bg)]"
    >
      <path
        d={path}
        fill="none"
        stroke={hot ? "var(--danger)" : "var(--accent)"}
        strokeWidth="1.5"
        vectorEffect="non-scaling-stroke"
      />
    </svg>
  );
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
  // CPU 采样历史（每 scope 最近 60 个点），驱动曲线图。
  const histRef = useRef<Record<"focused" | "workspace" | "app", number[]>>({
    focused: [], workspace: [], app: [],
  });

  useEffect(() => {
    let alive = true;
    const poll = () => {
      // Session SLOT ids (strings) — the backend maps them to shell pids.
      const focused = focusedSlot.value ? [String(focusedSlot.value)] : [];
      const workspace = wsRef.current().map(String);
      invoke<Stats>("resource_stats", { focused, workspace })
        .then((s) => {
          if (!alive) return;
          const push = (k: "focused" | "workspace" | "app", v: number | null) => {
            histRef.current[k] = [...(histRef.current[k] ?? []), v ?? 0].slice(-MAX_POINTS);
          };
          push("focused", s.focused?.cpu ?? null);
          push("workspace", s.workspace?.cpu ?? null);
          push("app", s.app?.cpu ?? null);
          setData(s);
        })
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
            const hot = (cpu ?? 0) >= 80;
            const points = histRef.current[r.key as "focused" | "workspace" | "app"] ?? [];
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
                      CPU {cpu!.toFixed(1)}%
                    </span>
                  ) : (
                    <span className="ml-auto font-mono text-[12px] text-[var(--text-faint)]">—</span>
                  )}
                </div>
                <Sparkline points={points} hot={hot} />
                <div className="mt-2 flex justify-between font-mono text-[11px] text-[var(--text-dim)]">
                  <span>{T.uMemory} {r.stat ? fmtMem(r.stat.mem) : "—"}</span>
                  <span>{points.length}/{MAX_POINTS}</span>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
