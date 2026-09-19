import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Layout, Model, Actions, DockLocation, Action } from "flexlayout-react";
// REQUIRED: flexlayout's structural classes (tabsets, tabs, dividers)
// carry the entire layout geometry — without this sheet the dock
// collapses into stacked blocks.
import "flexlayout-react/style/dark.css";
import {
  FiCopy, FiEdit2, FiLayers, FiPlus, FiRadio, FiServer,
  FiTrash2, FiUnlock,
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
import QuickSettingsPage from "../pages/QuickSettingsPage";
import PromptDialog from "../components/PromptDialog";
import type { Settings } from "../settings";
import {
  NAV_TAB_ID, NAV_TABSET_ID, TERM_TAB_ID, MAIN_TABSET_ID, TERM_TABSET_ID,
  canCloseTerminal, collectModelTermSlots, nextSlot,
} from "./model";
import { invoke } from "../terminal/tauri";
import { t } from "../i18n";
import type { Lang } from "../i18n";
import { activityStore, broadcastEnabled, broadcastGroup, focusedSlot } from "../terminal/registry";

export type Page = "terminal" | "ssh" | "history" | "ai" | "settings" | "remote" | "update" | "favorites" | "monitor" | "sysmon" | "quick-settings";

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
  "quick-settings": "tab-quick-settings",
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
  workspaces: { id: number; name: string; locked: boolean; lockHash?: string; termJson?: unknown }[];
  uiSnapshot?: () => unknown;
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
  const T = t(lang);
  // Main dock: the two unique panels (nav + workspace area).
  const [rowMenu, setRowMenu] = useState<{ x: number; y: number; wsId: number } | null>(null);
  const [prompt, setPrompt] = useState<
    { title: string; value?: string; onOk: (v: string) => void } | null
  >(null);
  const [tplMenu, setTplMenu] = useState(false);
  const [tplPosition, setTplPosition] = useState({ left: 0, top: 0 });
  const [, refreshBroadcast] = useState(0);
  // Term-tab context menu state (rename / close).
  const [termTabMenu, setTermTabMenu] = useState<{ x: number; y: number; tabId: string; name: string } | null>(null);
  const [plusMenu, setPlusMenu] = useState<{ x: number; y: number; tabsetId: string } | null>(null);
  // Broadcast is PER-WORKSPACE: switching (or unmounting) a workspace
  // clears the group so keystrokes never leak into another workspace's
  // terminals.
  const [broadcast, setBroadcast] = useState(false);
  const toggleBroadcast = () => {
    const next = !broadcast;
    setBroadcast(next);
    broadcastEnabled.value = next;
    broadcastGroup.clear();
    if (next) {
      // Collect live session slots from the model (defensive traversal —
      // getRootRow typing varies across versions).
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
    }
  };
  // Switching workspaces must tear the broadcast group down with the
  // outgoing workspace — otherwise typing in one workspace's terminal
  // lands in the previous workspace's shells too.
  useEffect(() => {
    setBroadcast(false);
    broadcastEnabled.value = false;
    broadcastGroup.clear();
  }, [activeWsId, termModel]);
  useEffect(() => () => {
    broadcastEnabled.value = false;
    broadcastGroup.clear();
  }, []);

  useEffect(() => {
    setRowMenu(null);
    setPrompt(null);
    setTplMenu(false);
    setTermTabMenu(null);
    setPlusMenu(null);
  }, [activeWsId, termModel]);

  useEffect(() => {
    if (!tplMenu) return;
    const close = () => setTplMenu(false);
    window.addEventListener("mousedown", close);
    return () => window.removeEventListener("mousedown", close);
  }, [tplMenu]);

  // Outside click closes the tabset "+" picker.
  useEffect(() => {
    if (!plusMenu) return;
    const close = () => setPlusMenu(null);
    window.addEventListener("mousedown", close);
    return () => window.removeEventListener("mousedown", close);
  }, [plusMenu]);

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
  useEffect(() => {
    if (!termTabMenu) return;
    const close = () => setTermTabMenu(null);
    window.addEventListener("mousedown", close);
    window.addEventListener("blur", close);
    return () => {
      window.removeEventListener("mousedown", close);
      window.removeEventListener("blur", close);
    };
  }, [termTabMenu]);

  const mainFactory = (node: any) => {
    const comp = node.getComponent();
    const now = Date.now();
    if (comp === "nav") {
      const L = t(lang);
      // Per-workspace busy slots: the active workspace uses the live
      // model; others use their committed layout JSON.
      const wsSlots = (w: any) =>
        w.id === activeWsId ? collectModelTermSlots(termModel) : jsonTermSlots(w.termJson);
      return (
        <div className="flex h-full min-h-0 flex-col overflow-hidden bg-[var(--bg-panel)] px-2 py-3">
          <div className="flex shrink-0 items-center justify-between px-2 pb-1">
            <span className="text-[11px] font-semibold tracking-wider text-[var(--text-faint)]">{L.nav}</span>
            <div className="flex items-center gap-0.5">
              {/* Order: 新建工作区 → 模板创建 */}
              <button className="icon-btn !p-1" title={L.newWorkspace} onClick={onCreateWorkspace}>
                <FiPlus size={13} />
              </button>
              {/* 模板按钮：hover 展示模板列表，点击即复制其终端布局 */}
              <div className="relative">
                <button
                  className="icon-btn !p-1"
                  title={templates.length > 0 ? T.cTemplateCreate : T.cTemplateEmpty}
                  onMouseDown={(e) => e.stopPropagation()}
                  onMouseEnter={(e) => {
                    const rect = e.currentTarget.getBoundingClientRect();
                    setTplPosition({ left: Math.max(4, Math.min(rect.right - 192, window.innerWidth - 196)), top: rect.bottom + 4 });
                    setTplMenu(true);
                  }}
                  onClick={(e) => {
                    const rect = e.currentTarget.getBoundingClientRect();
                    setTplPosition({ left: Math.max(4, Math.min(rect.right - 192, window.innerWidth - 196)), top: rect.bottom + 4 });
                    setTplMenu(true);
                  }}
                >
                  <FiLayers size={13} />
                </button>
                {tplMenu && templates.length > 0 && createPortal(
                  <div
                    className="fixed z-50 max-h-80 w-48 overflow-y-auto rounded-md border border-[var(--border)] bg-[var(--bg-elevated)] py-1"
                    style={tplPosition}
                    onMouseDown={(e) => e.stopPropagation()}
                  >
                    {templates.map((tpl) => (
                      <div
                        key={tpl.id}
                        className="flex cursor-pointer items-center gap-2 px-3 py-1.5 text-[12px] text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
                        onClick={() => {
                          setTplMenu(false);
                          onCreateFromTemplate(tpl, tpl.name);
                        }}
                      >
                        <FiLayers size={11} className="shrink-0 text-[var(--text-faint)]" />
                        <span className="min-w-0 flex-1 truncate">{tpl.name}</span>
                        <span className="shrink-0 font-mono text-[10px] text-[var(--text-faint)]">
                          {jsonTermSlots(tpl.termJson).length} {T.cTerminals}
                        </span>
                      </div>
                    ))}
                  </div>,
                  document.body,
                )}
              </div>
            </div>
          </div>
          <div data-testid="workspace-list" className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto">
            {workspaces.map((w) => {
              const isActive = w.id === activeWsId && page === "terminal";
              return (
                <div
                  key={w.id}
                  onClick={() => { onSwitchWorkspace(w.id); onOpenPage("terminal"); }}
                  onContextMenu={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    setRowMenu({ x: e.clientX, y: e.clientY, wsId: w.id });
                  }}
                  className={`workspace-row group flex shrink-0 cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-[12px] transition-colors ${
                    isActive ? "bg-[var(--accent-dim)] text-[var(--text)]" : "text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
                  }`}
                >
                  <WorkspaceActivityDot slots={wsSlots(w)} lang={lang} />
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

        </div>
      );
    }
    // 视图菜单的工具面板：主布局的 dock 面板（与导航/工作区同级），
    // 不进终端区布局 —— 主 dock 与终端 dock 是两个独立 Model。
    if (comp === "quick-settings") {
      return <QuickSettingsPage settings={settings} onSettings={onSettings} lang={lang} />;
    }
    if (comp === "sysmon") {
      return <SysmonPage getWsSlots={() => collectModelTermSlots(termModel)} />;
    }
    if (comp === "ai") {
      return (
        <AiPage
          uiSnapshot={() => ({
            workspaces: workspaces.map((w) => ({
              name: w.name,
              terminals: w.id === activeWsId ? collectModelTermSlots(termModel) : jsonTermSlots(w.termJson),
            })),
            activeWorkspace: workspaces.find((w) => w.id === activeWsId)?.name ?? null,
            focusedSlot: focusedSlot.value,
          })}
        />
      );
    }
    if (comp === "history") {
      return <HistoryPage workspaceId={activeWsId} />;
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
              workspaceId={activeWsId}
              themeId={themeId}
              fontSize={fontSize}
              shell={cfg.shell}
              cwd={cfg.cwd}
              command={cfg.command}
              name={tn.getName?.()}
              autoMatch={settings.autoMatch}
              followCursor={settings.followCursor}
              copyOnSelect={settings.copyOnSelect}
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
          <div className="term-dock relative min-h-0 flex-1">
            <Layout
              key={activeWsId}
              model={termModel}
              factory={termFactory}
              onModelChange={onModelChange}
              onAction={(a: Action) => {
                // Closing a terminal tab is the ONLY thing that ends its
                // session — unmounts from workspace switches/drag must
                // not kill the running shell (detach-safe backend).
                if (a.type === Actions.DELETE_TAB) {
                  if (!canCloseTerminal(termModel, String(a.data?.node ?? ""))) return undefined;
                  const m = /term-(\d+)/.exec(String(a.data?.node ?? ""));
                  if (m) {
                    invoke("close_session", { sessionId: m[1] }).catch(() => {});
                    // A closed terminal must not linger in the broadcast group.
                    broadcastGroup.delete(Number(m[1]));
                  }
                }
                return a;
              }}
              onRenderTab={(node, renderValues) => {
                if (node.getComponent() === "termpane") {
                  // Right-click a terminal tab: rename / close.
                  renderValues.content = (
                    <span
                      onContextMenu={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        setTermTabMenu({ x: e.clientX, y: e.clientY, tabId: node.getId(), name: node.getName() });
                      }}
                    >
                      {node.getName()}
                    </span>
                  );
                }
                if (node.getComponent() !== "termpane" || !broadcast) return;
                const slot = Number((/term-(\d+)/.exec(node.getId() ?? "") ?? [])[1] ?? 0);
                if (!slot) return;
                const inGroup = broadcastGroup.has(slot);
                renderValues.buttons.push(
                  <button
                    key="bcast"
                    className={`icon-btn !p-0.5 ${inGroup ? "!text-[var(--danger)]" : "!text-[var(--text-faint)]"}`}
                    title={inGroup ? T.cBroadcastLeave : T.cBroadcastJoin}
                    onClick={(e) => {
                      e.stopPropagation();
                      if (inGroup) broadcastGroup.delete(slot);
                      else broadcastGroup.add(slot);
                      refreshBroadcast((v) => v + 1);
                    }}
                  >
                    <FiRadio size={11} />
                  </button>,
                );
              }}
              onRenderTabSet={(node, renderValues) => {
                // "+" button on every tabset in the workspace area —
                // opens the shell/SSH picker at the button position.
                renderValues.buttons.push(
                  <button
                    key="new-term"
                    className="icon-btn !p-1"
                    title={T.newTerminal}
                    onClick={(e) => {
                      e.stopPropagation();
                      setPlusMenu({ x: e.clientX, y: e.clientY, tabsetId: node.getId() });
                    }}
                  >
                    <FiPlus size={13} />
                  </button>,
                );
                // 工作区广播（原导航栏头部）——固定在标签栏最右侧。
                renderValues.buttons.push(
                  <button
                    key="bcast-toggle"
                    className={`icon-btn !p-1 ${broadcast ? "!text-[var(--danger)]" : ""}`}
                    title={broadcast ? T.cBroadcastStop : T.cBroadcastStart}
                    onClick={(e) => {
                      e.stopPropagation();
                      toggleBroadcast();
                    }}
                  >
                    <FiRadio size={13} />
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

  /** New terminal tab inside the tabset that owns `tabsetId` — default
   * shell, a picked shell, or an SSH host connection. */
  function onAddTerminalIn(tabsetId: string, sh?: string | null, host?: SshHost) {
    const slot = nextSlot();
    const command = host
      ? ["ssh", "-p", String(host.port), `${host.user}@${host.host}`]
      : sh
        ? [sh, "-l"]
        : null;
    termModel.doAction(
      Actions.addNode(
        {
          type: "tab",
          id: `term-${slot}`,
          name: host?.name ?? (command ? command[0].split("/").pop()! : `bash ${slot}`),
          component: "termpane",
          enableClose: true,
          enableRenderOnDemand: false,
          config: { slot, shell: host ? null : settings.shell || null, command },
        },
        tabsetId,
        DockLocation.CENTER,
        -1,
      ),
    );
  }

  return (
    <div className="relative z-[1] h-full w-full">
      <Layout
        model={mainModel}
        factory={mainFactory}
        onModelChange={onModelChange}
        onAction={(a: Action) => a}
        onRenderTab={(node, renderValues) => {
          // Every main-dock tab is PLAIN text — no button/chip chrome.
          renderValues.content = <span>{node.getName()}</span>;
        }}
      />
      {plusMenu &&
        createPortal(
        <ShellPickerMenu
          lang={lang}
          style={{
            position: "fixed",
            left: Math.max(4, Math.min(plusMenu.x, window.innerWidth - 272)),
            top: Math.max(4, Math.min(plusMenu.y, window.innerHeight - 340)),
          }}
          shells={shells}
          defaultShell={defaultShell}
          sshHosts={sshHosts}
          onShell={(sh) => {
            setPlusMenu(null);
            onAddTerminalIn(plusMenu.tabsetId, sh);
          }}
          onSsh={(h) => {
            setPlusMenu(null);
            onAddTerminalIn(plusMenu.tabsetId, null, h);
          }}
        />,
        document.body,
      )}
      {termTabMenu &&
        createPortal(
        <div
          className="ctx-menu animate-fade-up"
          style={{
            left: Math.max(4, Math.min(termTabMenu.x, window.innerWidth - 176)),
            top: Math.max(4, Math.min(termTabMenu.y, window.innerHeight - 100)),
          }}
          onMouseDown={(e) => e.stopPropagation()}
          onContextMenu={(e) => e.preventDefault()}
        >
          <button
            className="ctx-item"
            onClick={() => {
              const menu = termTabMenu;
              setTermTabMenu(null);
              setPrompt({
                title: T.cRenameTerminal,
                value: menu.name,
                onOk: (v) => termModel.doAction(Actions.renameTab(menu.tabId, v)),
              });
            }}
          >
            <FiEdit2 size={13} /> {T.rename}
          </button>
          <button
            className="ctx-item hover:!text-[var(--danger)] disabled:opacity-40"
            disabled={!canCloseTerminal(termModel, termTabMenu.tabId)}
            onClick={() => {
              if (!canCloseTerminal(termModel, termTabMenu.tabId)) return;
              const slot = /term-(\d+)/.exec(termTabMenu.tabId)?.[1];
              if (slot) {
                invoke("close_session", { sessionId: slot }).catch(() => {});
                broadcastGroup.delete(Number(slot));
              }
              termModel.doAction(Actions.deleteTab(termTabMenu.tabId));
              setTermTabMenu(null);
            }}
          >
            <FiTrash2 size={13} /> {T.cCloseTerminal}
          </button>
        </div>,
        document.body,
      )}
      {rowMenu &&
        createPortal(
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
        </div>,
        document.body,
      )}
      {prompt &&
        createPortal(
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
          />,
          document.body,
        )}
    </div>
  );
}

/** 终端类型选择菜单：默认 Shell + 可选 Shell + 分割线下方的 SSH 主机
 * 列表（过多时滚动）。工具栏 "+" 与每个 tabset 的 "+" 共用。 */
function ShellPickerMenu({
  lang,
  shells,
  defaultShell,
  sshHosts,
  onShell,
  onSsh,
  style,
}: {
  lang: Lang;
  shells: string[];
  defaultShell: string;
  sshHosts: SshHost[];
  onShell: (sh: string | null) => void;
  onSsh: (h: SshHost) => void;
  style?: React.CSSProperties;
}) {
  return (
    <div
      className="absolute z-50 w-64 rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] py-1 shadow-xl"
      style={style}
      onMouseDown={(e) => e.stopPropagation()}
    >
      <div
        className="cursor-pointer px-3 py-1.5 text-[12px] text-[var(--text)] hover:bg-[var(--bg-hover)]"
        onClick={() => onShell(null)}
      >
        {t(lang).defaultShell} {defaultShell ? `(${defaultShell.split("/").pop()})` : ""}
      </div>
      <div className="my-1 h-px bg-[var(--border)]" />
      <div className="max-h-40 overflow-y-auto">
        {shells
          .filter((sh) => sh !== defaultShell)
          .map((sh) => (
            <div
              key={sh}
              className="cursor-pointer px-3 py-1.5 font-mono text-[11px] text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
              onClick={() => onShell(sh)}
            >
              {sh}
            </div>
          ))}
      </div>
      {sshHosts.length > 0 && (
        <>
          <div className="my-1 h-px bg-[var(--border)]" />
          <div className="px-3 py-1 text-[10px] font-semibold tracking-wider text-[var(--text-faint)]">{t(lang).sshHosts}</div>
          <div className="max-h-40 overflow-y-auto">
            {sshHosts.map((h) => (
              <div
                key={h.id}
                className="flex cursor-pointer items-center gap-2 px-3 py-1.5 text-[11px] text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
                onClick={() => onSsh(h)}
              >
                <FiServer size={11} className="shrink-0 text-[var(--text-faint)]" />
                <span className="min-w-0 flex-1 truncate">{h.name}</span>
                <span className="shrink-0 font-mono text-[10px] text-[var(--text-faint)]">{h.port}</span>
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}

const ACTIVITY_IDLE_MS = 10_000;

/** Pure busy decision shared by every workspace indicator: a slot with no
 * recorded activity (startup) is NEVER busy; otherwise busy within the
 * idle window. */
export function isSlotBusy(lastActivityMs: number, now: number): boolean {
  if (lastActivityMs <= 0) return false;
  return now - lastActivityMs < ACTIVITY_IDLE_MS;
}

/** 闲忙指示灯（官网设计稿）：工作区行左缘常显圆点 —
 * 红 = 近 10s 内有终端活动，绿 = 全部空闲，灰 = 无终端。 */
function WorkspaceActivityDot({ slots, lang }: { slots: number[]; lang: Lang }) {
  const T = t(lang);
  const isBusy = () => {
    const now = Date.now();
    return slots.some((slot) => isSlotBusy(activityStore.map[String(slot)] ?? 0, now));
  };
  const [state, setState] = useState<"busy" | "idle" | "none">(() =>
    slots.length === 0 ? "none" : isBusy() ? "busy" : "idle",
  );
  useEffect(() => {
    const tick = () => {
      const now = Date.now();
      setState(
        slots.length === 0
          ? "none"
          : slots.some((slot) => isSlotBusy(activityStore.map[String(slot)] ?? 0, now))
            ? "busy"
            : "idle",
      );
    };
    tick();
    const timer = window.setInterval(tick, 250);
    return () => window.clearInterval(timer);
  }, [slots]);
  const color =
    state === "busy" ? "var(--danger)" : state === "idle" ? "var(--success)" : "var(--text-faint)";
  return (
    <span
      role="img"
      aria-label={state === "busy" ? T.busy : T.idle}
      title={state === "busy" ? T.busy : T.idle}
      className="h-[7px] w-[7px] shrink-0 rounded-full"
      style={{
        background: color,
        boxShadow: state === "busy" ? "0 0 6px color-mix(in srgb, var(--danger) 60%, transparent)" : "none",
      }}
    />
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
