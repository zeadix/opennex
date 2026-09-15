import { Layout, Model, Actions, DockLocation, Action } from "flexlayout-react";
import { FiPlus } from "react-icons/fi";
import TerminalPane from "../terminal/TerminalPane";
import { getTheme, THEMES } from "../theme/themes";
import SshPage, { SshHost } from "../pages/SshPage";
import HistoryPage from "../pages/HistoryPage";
import AiPage from "../pages/AiPage";
import SettingsPage from "../pages/SettingsPage";
import { NAV_TAB_ID, NAV_TABSET_ID, TERM_TAB_ID, MAIN_TABSET_ID, TERM_TABSET_ID } from "./model";

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
  fontSize,
  shell,
  page,
  onOpenPage,
  workspaces,
  activeWsId,
  onSwitchWorkspace,
  onCreateWorkspace,
  onDeleteWorkspace,
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
  fontSize: number;
  shell: string;
  page: Page;
  onOpenPage: (p: Page) => void;
  workspaces: { id: number; name: string; locked: boolean }[];
  activeWsId: number;
  onSwitchWorkspace: (id: number) => void;
  onCreateWorkspace: () => void;
  onDeleteWorkspace: (id: number) => void;
  sshHosts: SshHost[];
  onSshHosts: (h: SshHost[]) => void;
  onConnectSsh: (h: SshHost) => void;
  settings: { fontSize: number; shell: string };
  onSettings: (patch: { fontSize?: number; shell?: string }) => void;
}) {
  // Main dock: the two unique panels (nav + workspace area).
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
            <span className="text-[11px] font-semibold tracking-wider text-[var(--text-faint)]">工作空间</span>
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
