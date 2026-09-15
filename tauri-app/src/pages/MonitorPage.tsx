import { useEffect, useRef, useState } from "react";
import { invoke } from "../terminal/tauri";

interface Stat {
  cpuPct: number;
  memUsedGb: number;
  memTotalGb: number;
  sessions: number;
}

const MAX_POINTS = 60;

/** Hand-rolled sparkline (no chart lib) — CPU over the last 60 samples. */
function Sparkline({ points, color }: { points: number[]; color: string }) {
  const w = 480, h = 96;
  if (points.length < 2) return <svg width={w} height={h} />;
  const max = Math.max(100, ...points);
  const step = w / (MAX_POINTS - 1);
  const path = points
    .map((p, i) => `${i === 0 ? "M" : "L"}${(i * step).toFixed(1)},${(h - (p / max) * (h - 8) - 4).toFixed(1)}`)
    .join(" ");
  return (
    <svg width={w} height={h} className="rounded-md border border-[var(--border)] bg-[var(--bg)]">
      <path d={path} fill="none" stroke={color} strokeWidth="1.5" />
    </svg>
  );
}

export default function MonitorPage() {
  const [cpu, setCpu] = useState<number[]>([]);
  const [latest, setLatest] = useState<Stat | null>(null);
  const timer = useRef<number | null>(null);

  useEffect(() => {
    let stopped = false;
    const poll = async () => {
      try {
        const s = await invoke<Stat>("system_stats");
        if (stopped) return;
        setLatest(s);
        setCpu((prev) => [...prev, s.cpuPct].slice(-MAX_POINTS));
      } catch {
        /* transient */
      }
    };
    poll();
    timer.current = window.setInterval(poll, 2000);
    return () => {
      stopped = true;
      if (timer.current) window.clearInterval(timer.current);
    };
  }, []);

  const memPct = latest ? Math.round((latest.memUsedGb / latest.memTotalGb) * 100) : 0;

  return (
    <div className="h-full overflow-y-auto px-8 py-6">
      <div className="mx-auto max-w-[560px] space-y-6">
        <h2 className="text-[15px] font-semibold">系统监控</h2>

        <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
          <div className="mb-2 flex items-center justify-between text-[12px] text-[var(--text-dim)]">
            <span>CPU 使用率</span>
            <span className="font-mono">{latest ? latest.cpuPct.toFixed(1) : "—"}%</span>
          </div>
          <Sparkline points={cpu} color="var(--accent)" />
        </div>

        <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
          <div className="mb-2 flex items-center justify-between text-[12px] text-[var(--text-dim)]">
            <span>内存</span>
            <span className="font-mono">
              {latest ? latest.memUsedGb.toFixed(2) : "—"} / {latest ? latest.memTotalGb.toFixed(1) : "—"} GB
            </span>
          </div>
          <div className="h-2 w-full overflow-hidden rounded-full bg-[var(--bg-active)]">
            <div
              className="h-full rounded-full bg-[var(--accent)] transition-all"
              style={{ width: `${memPct}%` }}
            />
          </div>
        </div>

        <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4 text-[12px] text-[var(--text-dim)]">
          活跃终端会话：<span className="font-mono text-[var(--text)]">{latest?.sessions ?? 0}</span>
        </div>
      </div>
    </div>
  );
}
