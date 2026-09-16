import { useEffect, useState } from "react";
import { Layout, Model, Actions, DockLocation, Action } from "flexlayout-react";
// REQUIRED: flexlayout's structural classes (tabsets, tabs, dividers)
// carry the entire layout geometry — without this sheet the dock
// collapses into stacked blocks.
import "flexlayout-react/style/dark.css";
import {
  FiActivity, FiClock, FiCopy, FiCpu, FiEdit2, FiMessageSquare, FiPlus,
  FiRadio, FiServer, FiSidebar, FiStar, FiTerminal, FiTrash2, FiUnlock,
} from "react-icons/fi";
import LockOverlay from "./LockOverlay";
import { sha256, LOCK_SALT, WsTemplate, jsonTermSlots } from "../workspaces";
import TerminalPane from "../terminal/TerminalPane";
import SshPage, { SshHost } from "../pages/SshPage";
import HistoryPage from "../pages/HistoryPage";
import FavoritesPage from "../pages/FavoritesPage";
import MonitorPage from "../pages/MonitorPage";
import AiPage from "../pages/AiPage";
import SettingsPage from "../pages/SettingsPage";
import RemotePage from "../pages/RemotePage";
import UpdatePage from "../pages/UpdatePage";
import SysmonPage from "../pages/SysmonPage";
import PromptDialog from "../components/PromptDialog";
import type { Settings } from "../settings";
import {
  NAV_TAB_ID, NAV_TABSET_ID, TERM_TAB_ID, MAIN_TABSET_ID, TERM_TABSET_ID,
  collectModelTermSlots, nextSlot,
} from "./model";
import { invoke } from "../terminal/tauri";
import { t } from "../i18n";
import type { Lang } from "../i18n";
import { broadcastEnabled, broadcastGroup } from "../terminal/registry";

export type Page = "terminal" | "ssh" | "history" | "ai" | "settings" | "remote" | "update" | "favorites" | "monitor" | "sysmon";

export const PAGE_TAB_ID: Record<Page, string> = {
  terminal: TERM_TAB_ID,
  ssh: "tab-ssh",
  history: "tab-history",
  ai: "tab-ai",
  settings: "tab-settings",
  remote: "tab-remote",
  update: "tab-update",
  favorites: "tab-favorites",
  monitor: "tab-monitor",
  sysmon: "tab-sysmon",
};

export const PAGE_NAME: Record<Page, string> = {
  terminal: "终端工作区",
  ssh: "SSH",
  history: "历史",
  ai: "AI",
  settings: "设置",
  remote: "远程",
  update: "更新",
  favorites: "收藏",
  monitor: "监控",
  sysmon: "系统资源",
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
  activities,
  activeWsId,
  onSwitchWorkspace,
  onCreateWorkspace,
  onDeleteWorkspace,
  onToggleLock,
  onRenameWorkspace,
  onSetLockPassword,
  onUnlock,
  templates,
  onCreateFromTemplate,
  onSaveAsTemplate,
  onDeleteTemplate,
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
  activities: Record<string, number>;
  activeWsId: number;
  onSwitchWorkspace: (id: number) => void;
  onCreateWorkspace: () => void;
  onDeleteWorkspace: (id: number) => void;
  onToggleLock: (id: number) => void;
  onRenameWorkspace: (id: number, name: string) => void;
  onSetLockPassword: (id: number, hash: string) => void;
  onUnlock: (id: number) => void;
  templates: WsTemplate[];
  onCreateFromTemplate: (tpl: WsTemplate, name: string) => void;
  onSaveAsTemplate: (wsId: number, name: string) => void;
  onDeleteTemplate: (id: string) => void;
  sshHosts: SshHost[];
  onSshHosts: (h: SshHost[]) => void;
  onConnectSsh: (h: SshHost) => void;
  settings: Settings;
  onSettings: (patch: Partial<Settings>) => void;
}) {
  // Main dock: the two unique panels (nav + workspace area).
  const [rowMenu, setRowMenu] = useState<{ x: number; y: number; wsId: number } | null>(null);
  const [prompt, setPrompt] = useState<
    { title: string; value?: string; onOk: (v: string) => void } | null
  >(null);
  const [menuOpen, setMenuOpen] = useState(false);
  const [broadcast, setBroadcast] = useState(false);

  // Right-click workspace menu / prompts close on any outside click.
  useEffect(() => {
    if (!rowMenu && !prompt) return;
    const close = () => setRowMenu(null);
    window.addEventListener("mousedown", close);
    window.addEventListener("blur", close);
    return () => {
      window.removeEventListener("mousedown", close);
      window.removeEventListener("blur", close);
    };
  }, [rowMenu, prompt]);

  const mainFactory = (node: any) => {
    const comp = node.getComponent();
    const now = Date.now();
    if (comp === "nav") {
      const L = t(lang);
      // Per-workspace busy state: any of its terminal sessions active
      // within 10s. The active workspace uses the live model; others use
      // their committed layout JSON.
      const wsBusy = (w: any) => {
        const slots =
          w.id === activeWsId ? collectModelTermSlots(termModel) : jsonTermSlots(w.termJson);
        return slots.some((s) => now - (activities?.[String(s)] ?? 0) < 10_000);
      };
      return (
        <div className="flex h-full flex-col overflow-y-auto bg-[var(--bg-panel)] px-2 py-3">
          <div className="mb-3 flex items-center gap-2 px-2">
            <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-[var(--accent-dim)] font-mono text-[13px] font-bold text-[var(--accent)]">N</div>
            <span className="text-[14px] font-semibold tracking-wide">OpenNex</span>
          </div>

          <div className="flex items-center justify-between px-2 pb-1">
            <span className="text-[11px] font-semibold tracking-wider text-[var(--text-faint)]">{L.nav}</span>
            <button className="icon-btn !p-1" title={L.newWorkspace} onClick={onCreateWorkspace}>
              <FiPlus size={13} />
            </button>
          </div>
          <div className="flex flex-col gap-0.5">
            {workspaces.map((w) => {
              const isActive = w.id === activeWsId && page === "terminal";
              const busy = wsBusy(w);
              return (
                <div
                  key={w.id}
                  onClick={() => { onSwitchWorkspace(w.id); onOpenPage("terminal"); }}
                  onContextMenu={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    setRowMenu({ x: e.clientX, y: e.clientY, wsId: w.id });
                  }}
                  title={busy ? L.busy : L.idle}
                  className={`group flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-[12px] transition-colors ${
                    isActive ? "bg-[var(--accent-dim)] text-[var(--text)]" : "text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
                  }`}
                >
                  {/* 工作状态指示器：行名左侧，红=繁忙 绿=空闲 */}
                  <span
                    className="h-2 w-2 shrink-0 rounded-full"
                    style={{
                      background: busy ? "var(--danger)" : "var(--success)",
                      opacity: busy ? 1 : 0.5,
                      boxShadow: busy ? "0 0 6px var(--danger)" : "none",
                    }}
                  />
                  {w.locked && <FiLock size={12} className="shrink-0 text-[var(--accent)]" />}
                  <span className="min-w-0 flex-1 truncate">{w.name}</span>
                  <button
                    className="icon-btn !p-0.5 shrink-0 opacity-0 group-hover:opacity-100 hover:!text-[var(--danger)]"
                    title={L.deleteWorkspace}
                    onClick={(e) => { e.stopPropagation(); onDeleteWorkspace(w.id); }}
                  >
                    <svg width="11" height="11" viewBox="0 0 12 12"><path d="M2 2 L10 10 M10 2 L2 10" stroke="currentColor" strokeWidth="1.6" /></svg>
                  </button>
                </div>
              );
            })}
          </div>

          <div className="my-3 h-px bg-[var(--border)]" />
          <div className="flex items-center justify-between px-2 pb-1">
            <span className="text-[11px] font-semibold tracking-wider text-[var(--text-faint)]">{L.template}</span>
          </div>
          <div className="flex flex-col gap-0.5">
            {templates.length === 0 && (
              <div className="px-2 py-0.5 text-[11px] leading-relaxed text-[var(--text-faint)]">{L.tplHint}</div>
            )}
            {templates.map((tpl) => (
              <div
                key={tpl.id}
                onClick={() =>
                  setPrompt({
                    title: `${L.newWorkspace} · ${L.template}`,
                    value: tpl.name,
                    onOk: (name) => onCreateFromTemplate(tpl, name),
                  })
                }
                className="group flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-[12px] text-[var(--text-dim)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
              >
                <FiCopy size={12} className="shrink-0 text-[var(--text-faint)]" />
                <span className="min-w-0 flex-1 truncate">{tpl.name}</span>
                <button
                  className="icon-btn !p-0.5 shrink-0 opacity-0 group-hover:opacity-100 hover:!text-[var(--danger)]"
                  title={L.deleteWorkspace}
                  onClick={(e) => { e.stopPropagation(); onDeleteTemplate(tpl.id); }}
                >
                  <svg width="11" height="11" viewBox="0 0 12 12"><path d="M2 2 L10 10 M10 2 L2 10" stroke="currentColor" strokeWidth="1.6" /></svg>
                </button>
              </div>
            ))}
          </div>

        </div>
      );
    }
    // 视图菜单的工具面板：主布局的 dock 面板（与导航/工作区同级），
    // 不进终端区布局 —— 主 dock 与终端 dock 是两个独立 Model。
    if (comp === "sysmon") {
      return <SysmonPage getWsSlots={() => collectModelTermSlots(termModel)} />;
    }
    if (comp === "ai") {
      return <AiPage />;
    }
    if (comp === "history") {
      return <HistoryPage />;
    }
    if (comp === "favorites") {
      return <FavoritesPage />;
    }
    if (comp === "ssh") {
      return <SshPage hosts={sshHosts} onHosts={onSshHosts} onConnect={onConnectSsh} />;
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
              name={tn.getName?.()}
              autoMatch={settings.autoMatch}
              onFontSize={onFontSize}
            />
          );
        }
        // Pages (SSH/history/AI/settings/…) are floating windows now —
        // the workspace dock hosts TERMINALS only.
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
              onAction={(a: Action) => {
                // Closing a terminal tab is the ONLY thing that ends its
                // session — unmounts from workspace switches/drag must
                // not kill the running shell (detach-safe backend).
                if (a.type === Actions.DELETE_TAB) {
                  const m = /term-(\d+)/.exec(String(a.data?.node ?? ""));
                  if (m) invoke("close_session", { sessionId: m[1] }).catch(() => {});
                }
                return a;
              }}
              onRenderTab={(node, renderValues) => {
                if (node.getComponent() !== "termpane" || !broadcast) return;
                const slot = Number((/term-(\d+)/.exec(node.getId() ?? "") ?? [])[1] ?? 0);
                if (!slot) return;
                const inGroup = broadcastGroup.has(slot);
                renderValues.buttons.push(
                  <button
                    key="bcast"
                    className={`icon-btn !p-0.5 ${inGroup ? "!text-[var(--danger)]" : "!text-[var(--text-faint)]"}`}
                    title={inGroup ? "退出广播组" : "加入广播组"}
                    onClick={(e) => {
                      e.stopPropagation();
                      if (inGroup) broadcastGroup.delete(slot);
                      else broadcastGroup.add(slot);
                      setBroadcast(broadcast); // re-render icon state
                    }}
                  >
                    <FiRadio size={11} />
                  </button>,
                );
              }}
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

  // Main-dock tab chips: the terminal workspace gets a solid accent
  // chip, view panels get muted outlined chips with their own icons —
  // the two kinds read differently at a glance.
  const mainTabIcon = (comp: string) => {
    switch (comp) {
      case "term": return <FiTerminal size={11} />;
      case "nav": return <FiSidebar size={11} />;
      case "sysmon": return <FiActivity size={11} />;
      case "ai": return <FiMessageSquare size={11} />;
      case "history": return <FiClock size={11} />;
      case "favorites": return <FiStar size={11} />;
      case "ssh": return <FiServer size={11} />;
      default: return null;
    }
  };

  return (
    <div className="relative h-full w-full">
      <Layout
        model={mainModel}
        factory={mainFactory}
        onModelChange={onModelChange}
        onAction={(a: Action) => a}
        onRenderTab={(node, renderValues) => {
          const comp = node.getComponent() ?? "";
          const icon = mainTabIcon(comp);
          if (!icon) return;
          renderValues.content = (
            <span className={`tabchip ${comp === "term" ? "tabchip-term" : "tabchip-view"}`}>
              {icon}
              {node.getName()}
            </span>
          );
        }}
      />
      {rowMenu && (
        <div
          className="ctx-menu animate-fade-up"
          style={{
            left: Math.max(4, Math.min(rowMenu.x, window.innerWidth - 176)),
            top: Math.max(4, Math.min(rowMenu.y, window.innerHeight - 168)),
          }}
          onMouseDown={(e) => e.stopPropagation()}
          onContextMenu={(e) => e.preventDefault()}
        >
          {(() => {
            const L = t(lang);
            const ws = workspaces.find((w) => w.id === rowMenu.wsId);
            if (!ws) return null;
            return (
              <>
                <button
                  className="ctx-item"
                  onClick={() => {
                    setRowMenu(null);
                    setPrompt({
                      title: L.rename,
                      value: ws.name,
                      onOk: (v) => onRenameWorkspace(ws.id, v),
                    });
                  }}
                >
                  <FiEdit2 size={13} /> {L.rename}
                </button>
                <button
                  className="ctx-item"
                  onClick={() => {
                    setRowMenu(null);
                    onToggleLock(ws.id);
                  }}
                >
                  <FiUnlock size={13} /> {ws.locked ? L.unlock : L.lock}
                </button>
                <button
                  className="ctx-item"
                  onClick={() => {
                    setRowMenu(null);
                    setPrompt({
                      title: L.tplNamePrompt,
                      value: ws.name,
                      onOk: (v) => onSaveAsTemplate(ws.id, v),
                    });
                  }}
                >
                  <FiCopy size={13} /> {L.saveAsTemplate}
                </button>
                <div className="my-1 h-px bg-[var(--border)]" />
                <button
                  className="ctx-item hover:!text-[var(--danger)]"
                  onClick={() => {
                    setRowMenu(null);
                    onDeleteWorkspace(ws.id);
                  }}
                >
                  <FiTrash2 size={13} /> {L.deleteWorkspace}
                </button>
              </>
            );
          })()}
        </div>
      )}
      {prompt && (
        <PromptDialog
          title={prompt.title}
          defaultValue={prompt.value}
          okText={t(lang).ok}
          cancelText={t(lang).cancel}
          onOk={(v) => {
            const fn = prompt.onOk;
            setPrompt(null);
            fn(v);
          }}
          onCancel={() => setPrompt(null)}
        />
      )}
    </div>
  );
}

function FiLock({ size, className }: { size: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className={className}>
      <rect x="3" y="11" width="18" height="11" rx="2" />
      <path d="M7 11V7a5 5 0 0 1 10 0v4" />
    </svg>
  );
}
