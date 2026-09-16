import { useEffect, useRef, useState } from "react";
import { FiChevronDown } from "react-icons/fi";
import { getTheme, THEMES } from "../theme/themes";
import { LANGS, t, Lang } from "../i18n";
import PromptDialog from "./PromptDialog";

export default function TopBar({
  lang,
  onLang,
  themeId,
  onTheme,
  onPage,
  updateAvailable,
  currentVersion,
  navOpen,
  termOpen,
  onTogglePanel,
  sysmonOpen,
  onToggleSysmon,
  onSaveLayout,
  onLoadLayout,
  onSaveLayoutAs,
  onCreateWorkspace,
}: {
  lang: Lang;
  onLang: (l: Lang) => void;
  themeId: string;
  onTheme: (id: string) => void;
  onPage: (p: string) => void;
  updateAvailable: boolean;
  currentVersion: string;
  navOpen: boolean;
  termOpen: boolean;
  onTogglePanel: (panel: "nav" | "term") => void;
  sysmonOpen: boolean;
  onToggleSysmon: () => void;
  onSaveLayout: () => void;
  onLoadLayout: () => void;
  onSaveLayoutAs: (name: string) => void;
  onCreateWorkspace: (name?: string) => void;
}) {
  const [open, setOpen] = useState<string | null>(null);
  const [saveAsOpen, setSaveAsOpen] = useState(false);
  const barRef = useRef<HTMLDivElement>(null);
  const T = t(lang);

  useEffect(() => {
    const onDown = (e: MouseEvent) => {
      if (barRef.current && !barRef.current.contains(e.target as Node)) setOpen(null);
    };
    window.addEventListener("mousedown", onDown);
    return () => window.removeEventListener("mousedown", onDown);
  }, []);

  // Hover-switch between menus while one is open (native menu feel).
  const enter = (label: string) => {
    if (open !== null && open !== label) setOpen(label);
  };

  const menus: { label: string; items: { label: string; onClick?: () => void; checked?: boolean }[] }[] = [
    {
      label: T.menuWorkspace,
      items: [
        { label: T.newWorkspace, onClick: () => onCreateWorkspace() },
        { label: T.saveLayout, onClick: onSaveLayout },
        { label: T.loadLayout, onClick: onLoadLayout },
        { label: T.saveLayoutAs, onClick: () => setSaveAsOpen(true) },
      ],
    },
    {
      label: T.menuView,
      items: [
        { label: T.navPanel, checked: navOpen, onClick: () => onTogglePanel("nav") },
        { label: T.workArea, checked: termOpen, onClick: () => onTogglePanel("term") },
        { label: T.sysmon, checked: sysmonOpen, onClick: onToggleSysmon },
        { label: `${T.ai} 助手…`, onClick: () => onPage("ai") },
        { label: `${T.history}…`, onClick: () => onPage("history") },
        { label: `${T.favorites}…`, onClick: () => onPage("favorites") },
      ],
    },
    {
      label: T.menuRemote,
      items: [
        { label: `${T.lan}…`, onClick: () => onPage("remote") },
        { label: `${T.wan}…`, onClick: () => onPage("remote-wan") },
        { label: "SSH 连接…", onClick: () => onPage("ssh") },
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
      label: T.menuHelp,
      items: [
        { label: T.about, onClick: () => onPage("about") },
        { label: T.updateCheck, onClick: () => onPage("update") },
        { label: T.tutorial, onClick: () => onPage("tutorial") },
      ],
    },
  ];

  return (
    <div
      ref={barRef}
      className="relative z-[8000] flex h-10 shrink-0 items-center gap-0.5 border-b border-[var(--border)] bg-[var(--bg-panel)] px-2"
    >
      <span className="glow-text mr-2 font-mono text-[12px] font-bold">OpenNex</span>
      {menus.map((m) => (
        <div key={m.label} className="relative">
          <button
            className={`flex items-center rounded px-2 py-1 text-[12.5px] transition-colors ${
              open === m.label
                ? "bg-[var(--bg-active)] text-[var(--text)]"
                : "text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
            }`}
            onClick={() => setOpen(open === m.label ? null : m.label)}
            onMouseEnter={() => enter(m.label)}
          >
            {m.label}
            <FiChevronDown size={10} className="ml-0.5 opacity-50" />
          </button>
          {open === m.label && (
            <div className="animate-fade-up absolute left-0 top-full z-50 mt-0.5 min-w-[190px] rounded-md border border-[var(--border)] bg-[var(--bg-elevated)] py-1 shadow-xl">
              {m.items.map((it, i) => (
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
              ))}
            </div>
          )}
        </div>
      ))}
      {/* 设置：直接打开设置窗口（无下拉） */}
      <button
        className="rounded px-2 py-1 text-[12.5px] text-[var(--text-dim)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
        onClick={() => onPage("settings")}
      >
        {T.settings}
      </button>
      <div className="flex-1" />
      {updateAvailable && (
        <button
          onClick={() => onPage("update")}
          className="rounded-md bg-[var(--accent-dim)] px-2 py-1 text-[11px] font-semibold text-[var(--accent)] transition-colors hover:brightness-125"
        >
          {lang === "zh" ? "更新至最新版" : "Update available"}
        </button>
      )}
      <span className="ml-1 font-mono text-[11px] text-[var(--text-faint)]">v{currentVersion}</span>

      {saveAsOpen && (
        <PromptDialog
          title={T.saveLayoutAs}
          okText={T.ok}
          cancelText={T.cancel}
          onOk={(name) => {
            onSaveLayoutAs(name);
            setSaveAsOpen(false);
          }}
          onCancel={() => setSaveAsOpen(false)}
        />
      )}
    </div>
  );
}
