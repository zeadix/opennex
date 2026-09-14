import { useState } from "react";
import {
  FiTerminal, FiServer, FiClock, FiCpu, FiSettings, FiDroplet,
  FiPlus, FiTrash2, FiLock, FiUnlock, FiEdit2,
} from "react-icons/fi";
import { getTheme, THEMES } from "../theme/themes";
import { Workspace } from "../workspaces";

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
  workspaces,
  activeWsId,
  onSwitchWorkspace,
  onCreateWorkspace,
  onRenameWorkspace,
  onDeleteWorkspace,
  onToggleLock,
}: {
  page: Page;
  onNavigate: (p: Page) => void;
  themeId: string;
  onTheme: (id: string) => void;
  workspaces: Workspace[];
  activeWsId: number;
  onSwitchWorkspace: (id: number) => void;
  onCreateWorkspace: () => void;
  onRenameWorkspace: (id: number, name: string) => void;
  onDeleteWorkspace: (id: number) => void;
  onToggleLock: (id: number) => void;
}) {
  const [renaming, setRenaming] = useState<number | null>(null);
  const [renameBuf, setRenameBuf] = useState("");

  const commitRename = (id: number) => {
    if (renameBuf.trim()) onRenameWorkspace(id, renameBuf.trim());
    setRenaming(null);
  };

  return (
    <div className="flex w-[184px] shrink-0 flex-col border-r border-[var(--border)] bg-[var(--bg-panel)] px-2 py-3">
      <div className="mb-3 flex items-center gap-2 px-2">
        <div className="flex h-7 w-7 items-center justify-center rounded-md bg-[var(--accent-dim)] font-mono text-[13px] font-bold text-[var(--accent)]">
          N
        </div>
        <span className="text-[14px] font-semibold tracking-wide">OpenNex</span>
      </div>

      <nav className="mb-3 flex flex-col gap-0.5">
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

      {/* Workspace list */}
      <div className="flex items-center justify-between px-2 pb-1 pt-1">
        <span className="text-[11px] font-semibold tracking-wider text-[var(--text-faint)]">
          工作空间
        </span>
        <button className="icon-btn !p-1" title="新建工作空间" onClick={onCreateWorkspace}>
          <FiPlus size={13} />
        </button>
      </div>
      <div className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto">
        {workspaces.map((w) => {
          const isActive = w.id === activeWsId && page === "terminal";
          return (
            <div
              key={w.id}
              onClick={() => {
                onSwitchWorkspace(w.id);
                onNavigate("terminal");
              }}
              className={`group flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-[12px] transition-colors ${
                isActive
                  ? "bg-[var(--accent-dim)] text-[var(--text)]"
                  : "text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
              }`}
            >
              {w.locked ? (
                <FiLock size={12} className="shrink-0 text-[var(--accent)]" />
              ) : (
                <FiTerminal size={12} className="shrink-0 opacity-60" />
              )}
              {renaming === w.id ? (
                <input
                  autoFocus
                  className="min-w-0 flex-1 rounded border border-[var(--accent)] bg-[var(--bg)] px-1 text-[12px] outline-none"
                  style={{ userSelect: "text" }}
                  value={renameBuf}
                  onClick={(e) => e.stopPropagation()}
                  onChange={(e) => setRenameBuf(e.target.value)}
                  onBlur={() => commitRename(w.id)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") commitRename(w.id);
                    if (e.key === "Escape") setRenaming(null);
                  }}
                />
              ) : (
                <span className="min-w-0 flex-1 truncate">{w.name}</span>
              )}
              <span className="flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100">
                <button
                  className="icon-btn !p-0.5"
                  title="重命名"
                  onClick={(e) => {
                    e.stopPropagation();
                    setRenameBuf(w.name);
                    setRenaming(w.id);
                  }}
                >
                  <FiEdit2 size={11} />
                </button>
                <button
                  className="icon-btn !p-0.5"
                  title={w.locked ? "已锁定" : "锁定"}
                  onClick={(e) => {
                    e.stopPropagation();
                    onToggleLock(w.id);
                  }}
                >
                  {w.locked ? <FiUnlock size={11} /> : <FiLock size={11} />}
                </button>
                <button
                  className="icon-btn !p-0.5 hover:!text-[var(--danger)]"
                  title="删除工作空间"
                  onClick={(e) => {
                    e.stopPropagation();
                    onDeleteWorkspace(w.id);
                  }}
                >
                  <FiTrash2 size={11} />
                </button>
              </span>
            </div>
          );
        })}
      </div>

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
