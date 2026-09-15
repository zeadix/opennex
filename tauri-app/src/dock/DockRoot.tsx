import { useState } from "react";
import { Layout, Model, Actions, DockLocation, Action } from "flexlayout-react";
// REQUIRED: flexlayout's structural classes (tabsets, tabs, dividers)
// carry the entire layout geometry — without this sheet the dock
// collapses into stacked blocks.
import "flexlayout-react/style/dark.css";
import { FiPlus, FiRadio, FiTrash2, FiUnlock, FiEdit2 } from "react-icons/fi";
import LockOverlay from "./LockOverlay";
import { sha256, LOCK_SALT } from "../workspaces";
import TerminalPane from "../terminal/TerminalPane";
import { getTheme, THEMES } from "../theme/themes";
import SshPage, { SshHost } from "../pages/SshPage";
import HistoryPage from "../pages/HistoryPage";
import AiPage from "../pages/AiPage";
import SettingsPage from "../pages/SettingsPage";
import { NAV_TAB_ID, NAV_TABSET_ID, TERM_TAB_ID, MAIN_TABSET_ID, TERM_TABSET_ID } from "./model";
import { LANGS, t } from "../i18n";
import type { Lang } from "../i18n";
import { broadcastEnabled, broadcastGroup } from "../terminal/registry";

export type Page = "terminal" | "ssh" | "history" | "ai" | "settings";

export const PAGE_TAB_ID: Record<Page, string> = {
  terminal: TERM_TAB_ID,
  ssh: "tab-ssh",
  history: "tab-history",
  ai: "tab-ai",
  settings: "tab-settings",
};

export const PAGE_NAME: Record<Page, string> = {
  terminal: "终端",
  ssh: "SSH",
  history: "历史",
  ai: "AI",
  settings: "设置",
};

export default function DockRoot({
  mainModel,
  termModel,
  onModelChange,
  themeId,
  onTheme,
  lang,
  onLang,
  fontSize,
  shell,
  onFontSize,
  page,
  onOpenPage,
  onAddTerminal,
  onAddTerminalWith,
  shells,
  defaultShell,
  workspaces,
  activeWsId,
  onSwitchWorkspace,
  onCreateWorkspace,
  onDeleteWorkspace,
  onToggleLock,
  onRenameWorkspace,
  onSetLockPassword,
  onUnlock,
  sshHosts,
  onSshHosts,
  onConnectSsh,
  settings,
  onSettings,
}: {
  mainModel: Model;
  termModel: Model;
  onModelChange: () => void;
  themeId: string;
  onTheme: (id: string) => void;
  lang: Lang;
  onLang: (l: Lang) => void;
  fontSize: number;
  shell: string;
  onFontSize?: (size: number) => void;
  page: Page;
  onOpenPage: (p: Page) => void;
  onAddTerminal: () => void;
  onAddTerminalWith: (shell: string) => void;
  shells: string[];
  defaultShell: string;
  workspaces: { id: number; name: string; locked: boolean; lockHash?: string }[];
  activeWsId: number;
  onSwitchWorkspace: (id: number) => void;
  onCreateWorkspace: () => void;
  onDeleteWorkspace: (id: number) => void;
  onToggleLock: (id: number) => void;
  onRenameWorkspace: (id: number, name: string) => void;
  onSetLockPassword: (id: number, hash: string) => void;
  onUnlock: (id: number) => void;
  sshHosts: SshHost[];
  onSshHosts: (h: SshHost[]) => void;
  onConnectSsh: (h: SshHost) => void;
  settings: { fontSize: number; shell: string };
  onSettings: (patch: { fontSize?: number; shell?: string }) => void;
}) {
  // Main dock: the two unique panels (nav + workspace area).
  const [renamingId, setRenamingId] = useState<number | null>(null);
  const [renameBuf, setRenameBuf] = useState("");
  const [menuOpen, setMenuOpen] = useState(false);
  const [broadcast, setBroadcast] = useState(false);

  const mainFactory = (node: any) => {
    const comp = node.getComponent();
    if (comp === "nav") {
      return (
        <div className="flex h-full flex-col overflow-y-auto bg-[var(--bg-panel)] px-2 py-3">
          <div className="mb-3 flex items-center gap-2 px-2">
            <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-[var(--accent-dim)] font-mono text-[13px] font-bold text-[var(--accent)]">N</div>
            <span className="text-[14px] font-semibold tracking-wide">OpenNex</span>
          </div>

          <div className="flex items-center justify-between px-2 pb-1">
            <span className="text-[11px] font-semibold tracking-wider text-[var(--text-faint)]">{t(lang).nav}</span>
            <button className="icon-btn !p-1" title="新建工作空间" onClick={onCreateWorkspace}>
              <FiPlus size={13} />
            </button>
          </div>
          <div className="flex flex-col gap-0.5">
            {workspaces.map((w) => {
              const isActive = w.id === activeWsId && page === "terminal";
              return (
                <div
                  key={w.id}
                  onClick={() => { onSwitchWorkspace(w.id); onOpenPage("terminal"); }}
                  className={`group flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-[12px] transition-colors ${
                    isActive ? "bg-[var(--accent-dim)] text-[var(--text)]" : "text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
                  }`}
                >
                  {w.locked && <FiLock size={12} className="shrink-0 text-[var(--accent)]" />}
                  <span className="min-w-0 flex-1 truncate">{w.name}</span>
                  <button
                    className="icon-btn !p-0.5 shrink-0 opacity-0 group-hover:opacity-100 hover:!text-[var(--danger)]"
                    title="删除工作空间"
                    onClick={(e) => { e.stopPropagation(); onDeleteWorkspace(w.id); }}
                  >
                    <svg width="11" height="11" viewBox="0 0 12 12"><path d="M2 2 L10 10 M10 2 L2 10" stroke="currentColor" strokeWidth="1.6" /></svg>
                  </button>
                </div>
              );
            })}
          </div>

          <div className="my-3 h-px bg-[var(--border)]" />
          <div className="flex flex-col gap-0.5">
            {(["terminal", "ssh", "history", "ai", "settings"] as Page[]).map((p) => (
              <div
                key={p}
                className={`nav-item ${page === p ? "active" : ""}`}
                onClick={() => onOpenPage(p)}
              >
                <span>{PAGE_NAME[p]}</span>
              </div>
            ))}
          </div>

          <div className="mt-auto">
            <div
              className="nav-item"
              title="Switch language"
              onClick={() => onLang(lang === "zh" ? "en" : "zh")}
            >
              <span className="font-mono text-[11px] font-bold text-[var(--accent)]">
                {lang === "zh" ? "中" : "EN"}
              </span>
              <span>{lang === "zh" ? "中文" : "English"}</span>
            </div>
            <div
              className="nav-item"
              title="切换主题"
              onClick={() => {
                const idx = THEMES.findIndex((t) => t.id === themeId);
                onTheme(THEMES[(idx + 1) % THEMES.length].id);
              }}
            >
              <span>{getTheme(themeId).name}</span>
            </div>
            <div className="px-2 pt-1 text-[11px] text-[var(--text-faint)]">v0.1.55-tauri</div>
          </div>
        </div>
      );
    }
    if (comp === "term") {
      // Workspace area: its own dock model for terminals (and opened pages).
      const activeWs = workspaces.find((w: any) => w.id === activeWsId);
      const locked = !!activeWs?.locked;
      const termFactory = (tn: any) => {
        const tcomp = tn.getComponent();
        if (tcomp === "termpane") {
          const cfg = tn.getConfig() ?? {};
          return (
            <TerminalPane
              sessionId={cfg.slot ?? 0}
              themeId={themeId}
              fontSize={fontSize}
              shell={cfg.shell}
              command={cfg.command}
              onFontSize={onFontSize}
            />
          );
        }
        if (tcomp === "ssh") return <SshPage hosts={sshHosts} onHosts={onSshHosts} onConnect={onConnectSsh} />;
        if (tcomp === "history") return <HistoryPage />;
        if (tcomp === "ai") return <AiPage />;
        if (tcomp === "settings") return <SettingsPage settings={settings} onSettings={onSettings} themeId={themeId} onTheme={onTheme} />;
        return null;
      };
      return (
        <div className="relative flex h-full flex-col overflow-hidden">
          <div className="flex h-8 shrink-0 items-center justify-end gap-1 border-b border-[var(--border)] bg-[var(--bg-panel)] px-2">
            <button
              className={`icon-btn ${broadcast ? "!text-[var(--danger)]" : ""}`}
              title={broadcast ? "关闭广播输入（全部终端）" : "开启广播输入（全部终端）"}
              onClick={() => {
                const next = !broadcast;
                setBroadcast(next);
                broadcastEnabled.value = next;
                if (next) {
                  broadcastGroup.clear();
                  // Collect live session slots from the model (defensive
                  // traversal — getRootRow typing varies across versions).
                  const root: any = (termModel as any).getRootRow?.() ?? {};
                  const stack: any[] = [...(root.getChildren?.() ?? [])];
                  while (stack.length) {
                    const n: any = stack.pop();
                    if (!n) continue;
                    if (n.getType?.() === "tab") {
                      const m = /term-(\d+)/.exec(n.getId?.() ?? "");
                      if (m) broadcastGroup.add(Number(m[1]));
                    } else {
                      stack.push(...(n.getChildren?.() ?? []));
                    }
                  }
                } else {
                  broadcastGroup.clear();
                }
              }}
            >
              <FiRadio size={14} />
            </button>
            <button
              className="icon-btn"
              title="新建终端（选择 Shell）"
              onClick={(e) => {
                e.stopPropagation();
                setMenuOpen(!menuOpen);
              }}
            >
              <FiPlus size={14} />
              <svg width="9" height="9" viewBox="0 0 10 10" className="ml-0.5">
                <path d="M1 3 L5 7 L9 3" stroke="currentColor" strokeWidth="1.4" fill="none" />
              </svg>
            </button>
            {menuOpen && (
              <div
                className="absolute right-2 top-9 z-30 w-64 rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] py-1 shadow-xl"
                onClick={(e) => e.stopPropagation()}
              >
                <div
                  className="cursor-pointer px-3 py-1.5 text-[12px] text-[var(--text)] hover:bg-[var(--bg-hover)]"
                  onClick={() => { setMenuOpen(false); onAddTerminal(); }}
                >
                  默认 Shell {defaultShell ? `(${defaultShell.split("/").pop()})` : ""}
                </div>
                <div className="my-1 h-px bg-[var(--border)]" />
                {shells
                  .filter((sh) => sh !== defaultShell)
                  .map((sh) => (
                    <div
                      key={sh}
                      className="cursor-pointer px-3 py-1.5 font-mono text-[11px] text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
                      onClick={() => { setMenuOpen(false); onAddTerminalWith(sh); }}
                    >
                      {sh}
                    </div>
                  ))}
              </div>
            )}
          </div>
          <div className="relative min-h-0 flex-1">
            <Layout
              model={termModel}
              factory={termFactory}
              onModelChange={onModelChange}
              onAction={(a: Action) => a}
              onRenderTabSet={(node, renderValues) => {
                // "+" button on every tabset in the workspace area.
                renderValues.buttons.push(
                  <button
                    key="new-term"
                    className="icon-btn !p-1"
                    title="新建终端"
                    onClick={(e) => {
                      e.stopPropagation();
                      onAddTerminalIn(node.getId());
                    }}
                  >
                    <FiPlus size={13} />
                  </button>,
                );
              }}
            />
          </div>
          {locked && (
            <LockOverlay
              mode={activeWs?.lockHash ? "unlock" : "set"}
              onSubmit={async (pwd) => {
                if (!activeWs) return false;
                if (activeWs.lockHash) {
                  const hash = await sha256(LOCK_SALT + pwd);
                  if (hash === activeWs.lockHash) {
                    onUnlock(activeWs.id);
                    return true;
                  }
                  return false;
                }
                onSetLockPassword(activeWs.id, await sha256(LOCK_SALT + pwd));
                return true;
              }}
            />
          )}
        </div>
      );
    }
    return null;
  };

  /** New terminal tab inside the tabset that owns `tabsetId`. */
  function onAddTerminalIn(tabsetId: string) {
    const slot = nextSlot();
    termModel.doAction(
      Actions.addNode(
        {
          type: "tab",
          id: `term-${slot}`,
          name: `bash ${slot}`,
          component: "termpane",
          enableClose: true,
          enableRenderOnDemand: false,
          config: { slot, shell: settings.shell || null },
        },
        tabsetId,
        DockLocation.CENTER,
        -1,
      ),
    );
  }

  return (
    <Layout
      model={mainModel}
      factory={mainFactory}
      onModelChange={onModelChange}
      onAction={(a: Action) => a}
    />
  );
}

// Slot counter is global; seeds are advanced past restored model tabs by
// the caller (App).
let slotCounter = 1;
export function seedTermSlots(v: number) {
  slotCounter = Math.max(slotCounter, v + 1);
}
function nextSlot() {
  return slotCounter++;
}

function FiLock({ size, className }: { size: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className={className}>
      <rect x="3" y="11" width="18" height="11" rx="2" />
      <path d="M7 11V7a5 5 0 0 1 10 0v4" />
    </svg>
  );
}
