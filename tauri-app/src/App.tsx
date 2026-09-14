import { useEffect, useState } from "react";
import { FiChevronDown, FiLock } from "react-icons/fi";
import Sidebar, { Page } from "./components/Sidebar";
import StatusBar from "./components/StatusBar";
import TabStrip from "./components/TabStrip";
import { FiTool } from "react-icons/fi";
import TerminalPane from "./terminal/TerminalPane";
import PaneView from "./panes/PaneView";
import {
  PaneTree,
  closePane as treeClosePane,
  collectPanes,
  hasPane,
  setRatio as treeSetRatio,
  splitPane as treeSplitPane,
} from "./panes/tree";
import { useTheme } from "./theme/useTheme";
import SettingsPage from "./pages/SettingsPage";
import HistoryPage from "./pages/HistoryPage";
import AiPage from "./pages/AiPage";
import SshPage, { SshHost, loadHosts, saveHosts } from "./pages/SshPage";
import { useSettings } from "./settings";
import {
  LOCK_SALT,
  Workspace,
  makeTab,
  makeWorkspace,
  persist,
  restore,
  sha256,
} from "./workspaces";

let nextPaneId = 1;
let nextTabId = 1;
let nextWsId = 1;

interface TermTab {
  id: number;
  title: string;
  tree: PaneTree;
  activePane: number;
  command?: string[];
}

function seedFromTabs() {
  // Keep the global id seeds above every stored id (restore() bumps the
  // workspace id seed; pane/tab seeds must clear stored ids too).
}

function Placeholder({ label }: { label: string }) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2 text-[var(--text-faint)]">
      <FiTool size={28} />
      <div className="text-sm">{label} · 开发中</div>
    </div>
  );
}

function freshWorkspace(name: string): Workspace {
  const tab = makeTab();
  const ws: Workspace = {
    id: nextWsId++,
    name,
    tabs: [tab],
    activeTabId: tab.id,
    locked: false,
  };
  nextTabId = Math.max(nextTabId, tab.id + 1);
  return ws;
}

/** Unlock overlay shown over a locked workspace's content. */
function LockOverlay({
  hasHash,
  onUnlock,
  onSetPassword,
}: {
  hasHash: boolean;
  onUnlock: (password: string) => Promise<boolean>;
  onSetPassword: (password: string) => void;
}) {
  const [pwd, setPwd] = useState("");
  const [error, setError] = useState(false);

  const submit = async () => {
    if (hasHash) {
      const ok = await onUnlock(pwd);
      if (!ok) setError(true);
    } else if (pwd.length >= 4) {
      onSetPassword(pwd);
    } else {
      setError(true);
    }
  };

  return (
    <div className="absolute inset-0 z-20 flex flex-col items-center justify-center gap-4 bg-[var(--bg)]/95">
      <FiLock size={30} className="text-[var(--accent)]" />
      <div className="text-[15px] font-semibold">
        {hasHash ? "工作空间已锁定" : "设置锁定密码（至少 4 位）"}
      </div>
      <div className="flex items-center gap-2">
        <input
          autoFocus
          type="password"
          value={pwd}
          onChange={(e) => {
            setPwd(e.target.value);
            setError(false);
          }}
          onKeyDown={(e) => e.key === "Enter" && submit()}
          className="w-56 rounded-md border border-[var(--border)] bg-[var(--bg-panel)] px-3 py-2 text-[13px] outline-none focus:border-[var(--accent)]"
          style={{ userSelect: "text" }}
          placeholder="密码"
        />
        <button
          className="rounded-md bg-[var(--accent-dim)] px-3 py-2 text-[12px] text-[var(--accent)] hover:brightness-125"
          onClick={submit}
        >
          {hasHash ? "解锁" : "设置"}
        </button>
      </div>
      {error && <div className="text-[12px] text-[var(--danger)]">密码错误或过短</div>}
    </div>
  );
}

/** New-tab dropdown: default shell + every shell from /etc/shells. */
function NewTabMenu({
  shells,
  defaultShell,
  onPick,
}: {
  shells: string[];
  defaultShell: string;
  onPick: (shell: string | undefined) => void;
}) {
  return (
    <div className="absolute right-0 top-full z-30 mt-1 w-56 rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] py-1 shadow-lg">
      <div
        className="cursor-pointer px-3 py-1.5 text-[12px] hover:bg-[var(--bg-hover)]"
        onClick={() => onPick(undefined)}
      >
        默认 Shell {defaultShell ? `(${defaultShell})` : ""}
      </div>
      {shells
        .filter((s) => s !== defaultShell)
        .map((s) => (
          <div
            key={s}
            className="cursor-pointer px-3 py-1.5 font-mono text-[11px] text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
            onClick={() => onPick(s)}
          >
            {s}
          </div>
        ))}
    </div>
  );
}

export default function App() {
  const [themeId, setThemeId] = useTheme();
  const [settings, updateSettings] = useSettings();
  const [sshHosts, setSshHosts] = useState<SshHost[]>(loadHosts);
  const [page, setPage] = useState<Page>("terminal");
  const [shells, setShells] = useState<string[]>([]);
  const [newMenuOpen, setNewMenuOpen] = useState(false);

  const [workspaces, setWorkspaces] = useState<Workspace[]>(() => {
    const restored = restore();
    if (restored.length) {
      // Advance global seeds past every restored id.
      for (const w of restored) {
        nextWsId = Math.max(nextWsId, w.id + 1);
        for (const t of w.tabs) nextTabId = Math.max(nextTabId, t.id + 1);
      }
      return restored;
    }
    return [freshWorkspace("默认工作空间")];
  });
  const [activeWsId, setActiveWsId] = useState<number>(() => 1);

  const activeWs = workspaces.find((w) => w.id === activeWsId) ?? workspaces[0];

  useEffect(() => {
    persist(workspaces);
  }, [workspaces]);
  useEffect(() => {
    import("@tauri-apps/api/core")
      .then((m) => m.invoke<string[]>("list_shells"))
      .then(setShells)
      .catch(() => {});
  }, []);

  // ---- workspace ops -----------------------------------------------------
  const createWorkspace = () => {
    const w = freshWorkspace(`工作空间 ${nextWsId}`);
    setWorkspaces((prev) => [...prev, w]);
    setActiveWsId(w.id);
    setPage("terminal");
  };
  const renameWorkspace = (id: number, name: string) =>
    setWorkspaces((prev) => prev.map((w) => (w.id === id ? { ...w, name } : w)));
  const deleteWorkspace = (id: number) => {
    setWorkspaces((prev) => {
      const rest = prev.filter((w) => w.id !== id);
      if (rest.length === 0) {
        const w = freshWorkspace("默认工作空间");
        setActiveWsId(w.id);
        return [w];
      }
      if (activeWsId === id) setActiveWsId(rest[0].id);
      return rest;
    });
  };
  const toggleLock = async (id: number) => {
    const w = workspaces.find((x) => x.id === id);
    if (!w) return;
    if (w.locked) {
      // Unlocking happens via the overlay; the sidebar button only locks.
      return;
    }
    if (!w.lockHash) {
      // First lock: the overlay collects + sets the password.
      setWorkspaces((prev) => prev.map((x) => (x.id === id ? { ...x, locked: true } : x)));
    } else {
      setWorkspaces((prev) => prev.map((x) => (x.id === id ? { ...x, locked: true } : x)));
    }
  };
  const setLockPassword = async (id: number, password: string) => {
    const hash = await sha256(LOCK_SALT + password);
    setWorkspaces((prev) => prev.map((x) => (x.id === id ? { ...x, lockHash: hash } : x)));
  };
  const tryUnlock = async (id: number, password: string): Promise<boolean> => {
    const w = workspaces.find((x) => x.id === id);
    if (!w?.lockHash) return true;
    const hash = await sha256(LOCK_SALT + password);
    if (hash === w.lockHash) {
      setWorkspaces((prev) => prev.map((x) => (x.id === id ? { ...x, locked: false } : x)));
      return true;
    }
    return false;
  };

  // ---- tab ops (on the ACTIVE workspace) ---------------------------------
  const patchTab = (id: number, patch: (t: TermTab) => TermTab) =>
    setWorkspaces((prev) =>
      prev.map((w) =>
        w.id === activeWsId
          ? { ...w, tabs: w.tabs.map((t) => (t.id === id ? patch(t) : t)) }
          : w,
      ),
    );

  const newTab = (shell?: string) => {
    const t = makeTab();
    // Command-less tab with the chosen shell: pass shell via command[0].
    const withShell = shell ? { ...t, command: [shell, "-l"] } : t;
    setWorkspaces((prev) =>
      prev.map((w) =>
        w.id === activeWsId
          ? {
              ...w,
              tabs: [...w.tabs, withShell],
              activeTabId: withShell.id,
            }
          : w,
      ),
    );
    setPage("terminal");
  };
  const closeTab = (id: number) => {
    setWorkspaces((prev) =>
      prev.map((w) => {
        if (w.id !== activeWsId) return w;
        const rest = w.tabs.filter((t) => t.id !== id);
        if (rest.length === 0) {
          const t = makeTab();
          return { ...w, tabs: [t], activeTabId: t.id };
        }
        const activeTabId =
          w.activeTabId === id ? rest[rest.length - 1].id : w.activeTabId;
        return { ...w, tabs: rest, activeTabId };
      }),
    );
  };
  const selectTab = (id: number) =>
    setWorkspaces((prev) =>
      prev.map((w) => (w.id === activeWsId ? { ...w, activeTabId: id } : w)),
    );

  const splitPane = (tabId: number, pane: number, dir: "h" | "v") => {
    const newPane = nextPaneId++;
    nextTabId = Math.max(nextTabId, newPane);
    patchTab(tabId, (t) => ({
      ...t,
      tree: treeSplitPane(t.tree, pane, dir, newPane),
      activePane: newPane,
    }));
  };
  const closePane = (tabId: number, pane: number) => {
    patchTab(tabId, (t) => {
      const next = treeClosePane(t.tree, pane);
      const alive = collectPanes(next).filter((p) => p > 0);
      const activePane = hasPane(next, t.activePane)
        ? t.activePane
        : alive[alive.length - 1] ?? 0;
      return { ...t, tree: next, activePane };
    });
  };
  const setRatio = (tabId: number, path: number[], ratio: number[]) =>
    patchTab(tabId, (t) => ({ ...t, tree: treeSetRatio(t.tree, path, ratio) }));
  const activatePane = (tabId: number, pane: number) =>
    patchTab(tabId, (t) => (t.activePane === pane ? t : { ...t, activePane: pane }));

  const connectSsh = (h: SshHost) => {
    saveHosts(sshHosts);
    newTab();
    // Pass the ssh command through the newest tab by patching it in.
    setWorkspaces((prev) =>
      prev.map((w) => {
        if (w.id !== activeWsId) return w;
        const tabs = [...w.tabs];
        const last = tabs[tabs.length - 1];
        if (last) last.command = ["ssh", "-p", String(h.port), `${h.user}@${h.host}`];
        return { ...w, tabs };
      }),
    );
    setPage("terminal");
  };

  const activeTermTab = activeWs.tabs.find((t) => t.id === activeWs.activeTabId);
  const defaultShell = settings.shell || shells[0] || "";

  return (
    <div className="flex h-full">
      <Sidebar
        page={page}
        onNavigate={setPage}
        themeId={themeId}
        onTheme={setThemeId}
        workspaces={workspaces}
        activeWsId={page === "terminal" ? activeWsId : -1}
        onSwitchWorkspace={setActiveWsId}
        onCreateWorkspace={createWorkspace}
        onRenameWorkspace={renameWorkspace}
        onDeleteWorkspace={deleteWorkspace}
        onToggleLock={(id) => {
          void toggleLock(id);
        }}
      />
      <div className="flex min-w-0 flex-1 flex-col">
        {page === "terminal" ? (
          <>
            <div className="relative">
              <TabStrip
                tabs={activeWs.tabs}
                active={activeWs.activeTabId}
                onSelect={selectTab}
                onClose={closeTab}
                onNew={() => setNewMenuOpen(!newMenuOpen)}
              />
              {newMenuOpen && (
                <div className="absolute right-3 top-9">
                  <NewTabMenu
                    shells={shells}
                    defaultShell={defaultShell}
                    onPick={(shell) => {
                      setNewMenuOpen(false);
                      if (shell === undefined) newTab(undefined);
                      else newTabWithShell(shell);
                    }}
                  />
                </div>
              )}
            </div>
            <div className="relative min-h-0 flex-1">
              {activeWs.locked ? (
                <LockOverlay
                  hasHash={!!activeWs.lockHash}
                  onUnlock={(pwd) => tryUnlock(activeWs.id, pwd)}
                  onSetPassword={(pwd) => setLockPassword(activeWs.id, pwd)}
                />
              ) : (
                activeWs.tabs.map((t) => (
                  <div
                    key={t.id}
                    className="absolute inset-0 flex"
                    style={{ display: activeWs.activeTabId === t.id ? "flex" : "none" }}
                  >
                    {hasPane(t.tree, t.activePane) || t.tree.kind === "split" ? (
                      <PaneView
                        tree={t.tree}
                        path={[]}
                        activePane={t.activePane}
                        themeId={themeId}
                        fontSize={settings.fontSize}
                        shell={t.command ? undefined : t.command?.[0] || settings.shell}
                        command={t.command}
                        onActivate={(p) => activatePane(t.id, p)}
                        onClose={(p) => closePane(t.id, p)}
                        onSplit={(p, d) => splitPane(t.id, p, d)}
                        onRatio={(path, ratio) => setRatio(t.id, path, ratio)}
                      />
                    ) : (
                      <TerminalPane
                        sessionId={t.activePane}
                        themeId={themeId}
                        fontSize={settings.fontSize}
                        shell={t.command ? undefined : settings.shell}
                        command={t.command}
                      />
                    )}
                  </div>
                ))
              )}
            </div>
            <StatusBar sessionCount={activeWs.tabs.length} shell={defaultShell || "bash"} />
          </>
        ) : page === "ssh" ? (
          <SshPage
            hosts={sshHosts}
            onHosts={(h) => {
              setSshHosts(h);
              saveHosts(h);
            }}
            onConnect={connectSsh}
          />
        ) : page === "history" ? (
          <HistoryPage />
        ) : page === "ai" ? (
          <AiPage />
        ) : (
          <SettingsPage
            settings={settings}
            onSettings={updateSettings}
            themeId={themeId}
            onTheme={setThemeId}
          />
        )}
      </div>
    </div>
  );

  // Helper kept last: opens a tab with an explicitly chosen shell.
  function newTabWithShell(shell: string) {
    const wsId = activeWsId;
    const t = makeTab();
    t.command = [shell, "-l"];
    setWorkspaces((prev) =>
      prev.map((w) =>
        w.id === wsId ? { ...w, tabs: [...w.tabs, t], activeTabId: t.id } : w,
      ),
    );
    setPage("terminal");
  }
}
