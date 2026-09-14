import { FiTerminal, FiServer, FiClock, FiCpu, FiSettings, FiDroplet } from "react-icons/fi";
import { getTheme, THEMES } from "../theme/themes";

export type Page = "terminal" | "ssh" | "history" | "ai" | "settings";

const NAV: { id: Page; label: string; icon: React.ReactNode }[] = [
  { id: "terminal", label: "终端", icon: <FiTerminal size={17} /> },
  { id: "ssh", label: "SSH", icon: <FiServer size={17} /> },
  { id: "history", label: "历史", icon: <FiClock size={17} /> },
  { id: "ai", label: "AI", icon: <FiCpu size={17} /> },
  { id: "settings", label: "设置", icon: <FiSettings size={17} /> },
];

export default function Sidebar({
  page,
  onNavigate,
  themeId,
  onTheme,
}: {
  page: Page;
  onNavigate: (p: Page) => void;
  themeId: string;
  onTheme: (id: string) => void;
}) {
  return (
    <div className="flex w-[168px] shrink-0 flex-col border-r border-[var(--border)] bg-[var(--bg-panel)] px-2 py-3">
      <div className="mb-4 flex items-center gap-2 px-2">
        <div className="flex h-7 w-7 items-center justify-center rounded-md bg-[var(--accent-dim)] font-mono text-[13px] font-bold text-[var(--accent)]">
          N
        </div>
        <span className="text-[14px] font-semibold tracking-wide">OpenNex</span>
      </div>
      <nav className="flex flex-col gap-0.5">
        {NAV.map((n) => (
          <div
            key={n.id}
            className={`nav-item ${page === n.id ? "active" : ""}`}
            onClick={() => onNavigate(n.id)}
          >
            {n.icon}
            <span>{n.label}</span>
          </div>
        ))}
      </nav>
      <div className="mt-auto">
        <div
          className="nav-item"
          title="切换主题"
          onClick={() => {
            const idx = THEMES.findIndex((t) => t.id === themeId);
            onTheme(THEMES[(idx + 1) % THEMES.length].id);
          }}
        >
          <FiDroplet size={16} />
          <span>{getTheme(themeId).name}</span>
        </div>
        <div className="px-2 pt-1 text-[11px] text-[var(--text-faint)]">v0.1.55-tauri</div>
      </div>
    </div>
  );
}
