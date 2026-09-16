import { useEffect, useRef, useState } from "react";
import { FiChevronDown, FiPlus } from "react-icons/fi";
import { getTheme, THEMES } from "../theme/themes";
import { LANGS, t, Lang } from "../i18n";
import type { Page } from "../dock/DockRoot";

interface MenuItem {
  label: string;
  items: { label: string; onClick?: () => void; checked?: boolean; sep?: boolean }[];
}

export default function TopBar({
  lang,
  onLang,
  themeId,
  onTheme,
  onNewTerminal,
  onLockWorkspace,
  onCycleWorkspace,
  onPage,
  updateAvailable,
  currentVersion,
  sideBarVisible,
  onToggleSidebar,
}: {
  lang: Lang;
  onLang: (l: Lang) => void;
  themeId: string;
  onTheme: (id: string) => void;
  onNewTerminal: () => void;
  onLockWorkspace: () => void;
  onCycleWorkspace: () => void;
  onPage: (p: Page) => void;
  updateAvailable: boolean;
  currentVersion: string;
  sideBarVisible: boolean;
  onToggleSidebar: () => void;
}) {
  const [open, setOpen] = useState<string | null>(null);
  const barRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onDown = (e: MouseEvent) => {
      if (barRef.current && !barRef.current.contains(e.target as Node)) setOpen(null);
    };
    window.addEventListener("mousedown", onDown);
    return () => window.removeEventListener("mousedown", onDown);
  }, []);

  const T = t(lang);
  const menus: MenuItem[] = [
    {
      label: T.terminal,
      items: [
        { label: T.newTerminal, onClick: onNewTerminal },
        { label: lang === "zh" ? "关闭当前终端" : "Close current terminal", onClick: () => window.dispatchEvent(new CustomEvent("opennex-close-tab")), sep: false },
      ],
    },
    {
      label: lang === "zh" ? "视图" : "View",
      items: [
        { label: T.monitor, onClick: () => onPage("monitor") },
        { label: T.remote, onClick: () => onPage("remote") },
        { label: sideBarVisible ? (lang === "zh" ? "隐藏侧栏" : "Hide sidebar") : (lang === "zh" ? "显示侧栏" : "Show sidebar"), onClick: onToggleSidebar },
      ],
    },
    {
      label: T.theme,
      items: THEMES.map((th) => ({
        label: th.name,
        checked: th.id === themeId,
        onClick: () => onTheme(th.id),
      })),
    },
    {
      label: lang === "zh" ? "语言" : "Language",
      items: LANGS.map((l) => ({
        label: l.label,
        checked: l.id === lang,
        onClick: () => onLang(l.id),
      })),
    },
    {
      label: lang === "zh" ? "工作空间" : "Workspace",
      items: [
        { label: T.lock, onClick: onLockWorkspace },
        { label: T.nav + " ↻", onClick: onCycleWorkspace },
      ],
    },
  ];

  return (
    <div
      ref={barRef}
      className="flex h-10 shrink-0 items-center gap-0.5 border-b-2 border-[var(--danger)] px-2 text-[13px] font-bold text-white"
      style={{ minHeight: 40, background: "#7a1f1f", color: "#ffffff" }}
    >
      <span className="glow-text mr-2 font-mono text-[12px] font-bold">OpenNex</span>
      {menus.map((m) => (
        <div key={m.label} className="relative">
          <button
            className={`flex items-center gap-1 rounded px-2 py-1 transition-colors ${
              open === m.label ? "bg-[var(--bg-active)] text-[var(--text)]" : "text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
            }`}
            onClick={() => setOpen(open === m.label ? null : m.label)}
          >
            {m.label}
            <FiChevronDown size={10} className="opacity-60" />
          </button>
          {open === m.label && (
            <div className="animate-fade-up absolute left-0 top-full z-50 mt-0.5 min-w-[180px] rounded-md border border-[var(--border)] bg-[var(--bg-elevated)] py-1 shadow-xl">
              {m.items.map((it, i) =>
                it.sep ? (
                  <div key={i} className="my-1 h-px bg-[var(--border)]" />
                ) : (
                  <div
                    key={i}
                    onClick={() => {
                      it.onClick?.();
                      setOpen(null);
                    }}
                    className="flex cursor-pointer items-center justify-between gap-4 px-3 py-1.5 text-[12px] text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
                  >
                    <span>{it.label}</span>
                    {it.checked && <span className="text-[var(--accent)]">✓</span>}
                  </div>
                ),
              )}
            </div>
          )}
        </div>
      ))}
      <div className="flex-1" />
      {/* 右侧：更新按钮 + 版本号 */}
      <button
        onClick={() => onPage("update")}
        className={`flex items-center gap-1.5 rounded-md px-2 py-1 text-[11px] transition-colors ${
          updateAvailable
            ? "bg-[var(--accent-dim)] font-semibold text-[var(--accent)]"
            : "text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
        }`}
      >
        {updateAvailable ? (
          <>
            <FiPlus size={11} className="rotate-45" />
            {lang === "zh" ? `更新至最新版` : "Update available"}
          </>
        ) : (
          lang === "zh" ? "检查更新" : "Check updates"
        )}
      </button>
      <span className="ml-1 font-mono text-[11px] text-[var(--text-faint)]">v{currentVersion}</span>
    </div>
  );
}
