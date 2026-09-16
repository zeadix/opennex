import { useEffect, useRef, useState } from "react";
import { Actions, DockLocation } from "flexlayout-react";
import DockRoot, { Page, PAGE_TAB_ID, seedTermSlots } from "./dock/DockRoot";
import {
  ensureSelection,
  hasTermPane,
  loadMainModel,
  loadTermModel,
  mainTabsetId,
  maxTermSlot,
  saveModels,
  termTabsetId,
} from "./dock/model";
import { useTheme } from "./theme/useTheme";
import { useSettings } from "./settings";
import { loadLang, saveLang, Lang } from "./i18n";
import SshPage, { SshHost, loadHosts, saveHosts } from "./pages/SshPage";
import { loadWorkspaces, makeWorkspace, persistWorkspaces, Workspace } from "./workspaces";
import TopBar from "./components/TopBar";
import StatusBar from "./components/StatusBar";
import HistoryOverlay from "./terminal/HistoryOverlay";

export default function App() {
  const [themeId, setThemeId] = useTheme();
  const [lang, setLang] = useState<Lang>(loadLang);
  useEffect(() => saveLang(lang), [lang]);
  const [settings, updateSettings] = useSettings();
  const [sshHosts, setSshHosts] = useState<SshHost[]>(loadHosts);
  const [shells, setShells] = useState<string[]>([]);
  const [activities, setActivities] = useState<Record<string, number>>({});
  const [sideBarVisible, setSideBarVisible] = useState(true);
  const [historyOverlay, setHistoryOverlay] = useState(false);

  const [mainModel] = useState(() => loadMainModel());
  const [termModel] = useState(() => loadTermModel());
  const seeded = useRef(false);
  useEffect(() => {
    if (!seeded.current) {
      seedTermSlots(maxTermSlot(termModel));
      seeded.current = true;
      // Legacy saved layouts may carry an EMPTY terminal tabset (from the
      // era before terminals were seeded) — give it a live terminal.
      if (!hasTermPane(termModel)) {
        const slot = maxTermSlot(termModel) + 1;
        termModel.doAction(
          Actions.addNode(
            {
              type: "tab",
              id: `term-${slot}`,
              name: `bash ${slot}`,
              component: "termpane",
              enableClose: true,
              enableRenderOnDemand: false,
              config: { slot },
            },
            termTabsetId(termModel),
            DockLocation.CENTER,
            -1,
          ),
        );
      }
    }
  }, [termModel]);

  const [workspaces, setWorkspaces] = useState<Workspace[]>(loadWorkspaces);
  const [activeWsId, setActiveWsId] = useState(workspaces[0]?.id ?? 1);
  const [page, setPage] = useState<Page>("terminal");

  useEffect(() => {
    persistWorkspaces(workspaces);
  }, [workspaces]);
  useEffect(() => {
    import("@tauri-apps/api/core")
      .then((m) => m.invoke<string[]>("list_shells"))
      .then(setShells)
      .catch(() => {});
  }, []);
  // Workspace busy indicator: poll per-session last activity every 3s.
  useEffect(() => {
    const poll = () => {
      import("@tauri-apps/api/core")
        .then((m) => m.invoke<Record<string, number>>("session_activities"))
        .then(setActivities)
        .catch(() => {});
    };
    poll();
    const id = window.setInterval(poll, 3000);
    return () => window.clearInterval(id);
  }, []);

  const createWorkspace = () => {
    const w = makeWorkspace(`工作空间 ${workspaces.length + 1}`);
    setWorkspaces((prev) => [...prev, w]);
    setActiveWsId(w.id);
    openPage("terminal");
  };
  const renameWorkspace = (id: number, name: string) =>
    setWorkspaces((prev) => prev.map((w) => (w.id === id ? { ...w, name } : w)));
  const setLockPassword = (id: number, hash: string) =>
    setWorkspaces((prev) => prev.map((w) => (w.id === id ? { ...w, lockHash: hash } : w)));
  const unlockWorkspace = (id: number) =>
    setWorkspaces((prev) => prev.map((w) => (w.id === id ? { ...w, locked: false } : w)));
  /** Sidebar lock toggle: unlock directly; locking an unlocked ws needs a
   * password — the overlay collects it (locked = true, no hash yet → the
   * overlay runs in "set" mode). */
  const toggleLock = (id: number) =>
    setWorkspaces((prev) =>
      prev.map((w) =>
        w.id === id ? { ...w, locked: w.lockHash ? !w.locked : true } : w,
      ),
    );

  const deleteWorkspace = (id: number) => {
    setWorkspaces((prev) => {
      const rest = prev.filter((w) => w.id !== id);
      if (rest.length === 0) {
        const w = makeWorkspace("默认工作空间");
        setActiveWsId(w.id);
        return [w];
      }
      if (activeWsId === id) setActiveWsId(rest[0].id);
      return rest;
    });
  };

  /** Open a page tab in the main tabset (recreate if it was closed). */
  const openPage = (p: string) => {
    setPage(p as any);
    const tabId = PAGE_TAB_ID[p as Page];
    if (mainModel.getNodeById(tabId)) {
      mainModel.doAction(Actions.selectTab(tabId));
    } else {
      mainModel.doAction(
        Actions.addNode(
          {
            type: "tab",
            id: tabId,
            name: p === "remote" ? "远程" : p === "terminal" ? "终端" : p.toUpperCase(),
            component: p,
            enableClose: true,
            enableRenderOnDemand: false,
          },
          mainTabsetId(mainModel),
          DockLocation.CENTER,
          -1,
        ),
      );
    }
  };

  const addTerminal = (command?: string[]) => {
    const slot = maxTermSlot(termModel) + 1;
    seedTermSlots(slot);
    termModel.doAction(
      Actions.addNode(
        {
          type: "tab",
          id: `term-${slot}`,
          name: command?.[0]?.split("/").pop() ?? `bash ${slot}`,
          component: "termpane",
          enableClose: true,
          enableRenderOnDemand: false,
          config: { slot, command: command ?? null, shell: settings.shell || null },
        },
        termTabsetId(termModel),
        DockLocation.CENTER,
        -1,
      ),
    );
    setPage("terminal");
    openPage("terminal");
  };

  const connectSsh = (h: SshHost) => {
    saveHosts(sshHosts);
    const slot = maxTermSlot(termModel) + 1;
    seedTermSlots(slot);
    termModel.doAction(
      Actions.addNode(
        {
          type: "tab",
          id: `term-${slot}`,
          name: h.name,
          component: "termpane",
          enableClose: true,
          enableRenderOnDemand: false,
          config: {
            slot,
            command: ["ssh", "-p", String(h.port), `${h.user}@${h.host}`],
            shell: settings.shell || null,
          },
        },
        termTabsetId(termModel),
        DockLocation.CENTER,
        -1,
      ),
    );
    openPage("terminal");
  };

  return (
    <div className="flex h-full flex-col">
      <TopBar
        lang={lang}
        onLang={setLang}
        themeId={themeId}
        onTheme={setThemeId}
        onNewTerminal={() => addTerminal()}
        onLockWorkspace={() => toggleLock(activeWsId)}
        onCycleWorkspace={() => {
          const idx = workspaces.findIndex((w) => w.id === activeWsId);
          const next = workspaces[(idx + 1) % workspaces.length];
          if (next) setActiveWsId(next.id);
        }}
        onPage={openPage}
        updateAvailable={false}
        currentVersion="0.1.55"
        sideBarVisible={sideBarVisible}
        onToggleSidebar={() => setSideBarVisible(!sideBarVisible)}
      />
      <div className="relative flex min-h-0 flex-1">
      <DockRoot
      mainModel={mainModel}
      termModel={termModel}
      onModelChange={() => {
        ensureSelection(mainModel);
        ensureSelection(termModel);
        saveModels(mainModel, termModel);
      }}
      themeId={themeId}
      onTheme={setThemeId}
      lang={lang}
      onLang={(l: Lang) => { setLang(l); }}
      fontSize={settings.fontSize}
      onFontSize={(size) => updateSettings({ fontSize: size })}
      shell={settings.shell}
      page={page}
      onOpenPage={openPage}
      onAddTerminal={() => addTerminal()}
      onAddTerminalWith={(sh) => addTerminal([sh, "-l"])}
      shells={shells}
      defaultShell={settings.shell || shells[0] || ""}
      workspaces={workspaces}
      activeWsId={activeWsId}
      onSwitchWorkspace={setActiveWsId}
      onCreateWorkspace={createWorkspace}
      onDeleteWorkspace={deleteWorkspace}
      onRenameWorkspace={renameWorkspace}
      onSetLockPassword={setLockPassword}
      onUnlock={unlockWorkspace}
      onToggleLock={toggleLock}
      sshHosts={sshHosts}
      onSshHosts={(h) => {
        setSshHosts(h);
        saveHosts(h);
      }}
      onConnectSsh={connectSsh}
      settings={settings}
      onSettings={updateSettings}
      activities={activities}
    />
      <StatusBar sessionCount={Object.keys(activities).length} shell={settings.shell || "bash"} />
      {historyOverlay && <HistoryOverlay onClose={() => setHistoryOverlay(false)} />}
      </div>
    </div>
  );
}
