import { useEffect, useMemo, useRef, useState } from "react";
import { Actions, DockLocation, Model } from "flexlayout-react";
import DockRoot, { Page, PAGE_TAB_ID } from "./dock/DockRoot";
import {
  closePanel,
  NAV_TAB_ID,
  ensureSelection,
  mainModelFrom,
  termModelFrom,
  newTermJson,
  nextSlot,
  seedTermSlots,
  renumberTermJson,
  collectModelTermSlots,
  mainTabsetId,
  panelOpen,
  rootRowId,
  termTabsetId,
} from "./dock/model";
import { PAGE_NAME } from "./dock/DockRoot";
import { useTheme } from "./theme/useTheme";
import { useSettings } from "./settings";
import { loadLang, saveLang, t, Lang } from "./i18n";
import SshPage, { SshHost, loadHosts, saveHosts } from "./pages/SshPage";
import FavoritesPage from "./pages/FavoritesPage";
import RemotePage from "./pages/RemotePage";
import UpdatePage from "./pages/UpdatePage";
import FloatingWindow, { FloatWin } from "./dock/FloatingWindow";
import SettingsPage from "./pages/SettingsPage";
import HistoryPage from "./pages/HistoryPage";
import AiPage from "./pages/AiPage";
import MonitorPage from "./pages/MonitorPage";
import AboutPage from "./pages/AboutPage";
import TutorialPage from "./pages/TutorialPage";
import {
  loadWorkspaces,
  makeWorkspace,
  persistWorkspaces,
  jsonTermSlots,
  maxJsonSlot,
  loadTemplates,
  persistTemplates,
  Workspace,
  WsTemplate,
} from "./workspaces";
import TopBar from "./components/TopBar";
import StatusBar from "./components/StatusBar";
import HistoryOverlay from "./terminal/HistoryOverlay";

async function closeSessions(slots: number[]) {
  if (slots.length === 0) return;
  try {
    const m = await import("@tauri-apps/api/core");
    await Promise.all(
      slots.map((s) => m.invoke("close_session", { sessionId: String(s) }).catch(() => {})),
    );
  } catch {
    /* non-tauri dev — ignore */
  }
}

export default function App() {
  const [themeId, setThemeId] = useTheme();
  const [lang, setLang] = useState<Lang>(loadLang);
  useEffect(() => saveLang(lang), [lang]);
  const [settings, updateSettings] = useSettings();
  const [sshHosts, setSshHosts] = useState<SshHost[]>(loadHosts);
  const [shells, setShells] = useState<string[]>([]);
  const [activities, setActivities] = useState<Record<string, number>>({});
  const [historyOverlay, setHistoryOverlay] = useState(false);
  const [windows, setWindows] = useState<FloatWin[]>([]);
  const winZ = useRef(1);
  const [toast, setToast] = useState<string | null>(null);
  const toastTimer = useRef<number | null>(null);
  const showToast = (msg: string) => {
    setToast(msg);
    if (toastTimer.current) window.clearTimeout(toastTimer.current);
    toastTimer.current = window.setTimeout(() => setToast(null), 2200);
  };

  /** Open (or focus) a floating page window, cascading from center. */
  const openWindow = (id: string, title: string, w = 620, h = 680) => {
    setWindows((prev) => {
      const existing = prev.find((x) => x.id === id);
      const maxZ = prev.reduce((m, x) => Math.max(m, x.z), 0);
      if (existing) return prev.map((x) => (x.id === id ? { ...x, z: maxZ + 1 } : x));
      const vw = window.innerWidth, vh = window.innerHeight;
      const n = prev.length;
      return [
        ...prev,
        {
          id, title, w, h,
          x: Math.max(8, (vw - w) / 2 + n * 28),
          y: Math.max(8, (vh - h) / 2 - 20 + n * 24),
          z: maxZ + 1,
        },
      ];
    });
  };
  const closeWindow = (id: string) => setWindows((prev) => prev.filter((x) => x.id !== id));
  const focusWindow = (id: string) =>
    setWindows((prev) => {
      const maxZ = prev.reduce((m, x) => Math.max(m, x.z), 0);
      const target = prev.find((x) => x.id === id);
      if (!target || target.z === maxZ) return prev;
      return prev.map((x) => (x.id === id ? { ...x, z: maxZ + 1 } : x));
    });
  const moveWindow = (id: string, nx: number, ny: number) =>
    setWindows((prev) => prev.map((x) => (x.id === id ? { ...x, x: nx, y: ny } : x)));

  // ---- workspaces own their layouts -------------------------------------
  const [workspaces, setWorkspaces] = useState<Workspace[]>(loadWorkspaces);
  const [activeWsId, setActiveWsId] = useState(workspaces[0]?.id ?? 1);
  const [page, setPage] = useState<Page>("terminal");

  // One-time: advance the global terminal-slot counter past every saved
  // layout so new terminals can never collide with another workspace's
  // saved sessions.
  useMemo(() => {
    let max = 0;
    for (const w of workspaces) max = Math.max(max, maxJsonSlot(w.termJson));
    seedTermSlots(max);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Live dock models — memory-only until committed (switch / save / unload).
  const [mainModel, setMainModel] = useState<Model>(() => mainModelFrom(workspaces[0]?.mainJson));
  const [termModel, setTermModel] = useState<Model>(() =>
    termModelFrom(workspaces[0]?.termJson, nextSlot),
  );

  useEffect(() => {
    persistWorkspaces(workspaces);
  }, [workspaces]);

  /** Snapshot the live models into the active workspace's committed copy. */
  const commitLayout = () => {
    const snapMain = mainModel.toJson();
    const snapTerm = termModel.toJson();
    setWorkspaces((prev) =>
      prev.map((w) => (w.id === activeWsId ? { ...w, mainJson: snapMain, termJson: snapTerm } : w)),
    );
  };

  // App close without an explicit save: best-effort commit straight to
  // localStorage (setState would not flush in time).
  const commitRef = useRef<() => void>(() => {});
  commitRef.current = () => {
    try {
      const list: Workspace[] = JSON.parse(localStorage.getItem("opennex-workspaces") ?? "[]");
      const i = list.findIndex((w) => w.id === activeWsId);
      if (i >= 0) {
        list[i].mainJson = mainModel.toJson();
        list[i].termJson = termModel.toJson();
        localStorage.setItem("opennex-workspaces", JSON.stringify(list));
      }
    } catch {
      /* ignore */
    }
  };
  useEffect(() => {
    const h = () => commitRef.current();
    window.addEventListener("beforeunload", h);
    return () => window.removeEventListener("beforeunload", h);
  }, []);

  /** Switch workspaces: commit the outgoing layout, load the target's
   * committed copy (live sessions reattach by slot id). */
  const switchWorkspace = (id: number) => {
    if (id === activeWsId) return;
    const target = workspaces.find((w) => w.id === id);
    if (!target) return;
    const snapMain = mainModel.toJson();
    const snapTerm = termModel.toJson();
    setWorkspaces((prev) =>
      prev.map((w) => (w.id === activeWsId ? { ...w, mainJson: snapMain, termJson: snapTerm } : w)),
    );
    setMainModel(mainModelFrom(target.mainJson));
    setTermModel(termModelFrom(target.termJson, nextSlot));
    setActiveWsId(id);
    setPage("terminal");
  };

  const createWorkspace = (name?: string) => {
    commitLayout();
    const w = makeWorkspace(name ?? `工作空间 ${workspaces.length + 1}`);
    w.termJson = newTermJson(); // one fresh terminal on a unique slot
    setWorkspaces((prev) => [...prev, w]);
    setMainModel(mainModelFrom(undefined));
    setTermModel(termModelFrom(w.termJson, nextSlot));
    setActiveWsId(w.id);
    setPage("terminal");
  };

  /** Template copy: identical layout, fresh terminal slots (the copy
   * must not share PTY sessions with the template's source). */
  const createFromTemplate = (tpl: WsTemplate, name: string) => {
    commitLayout();
    const src = tpl.termJson ?? newTermJson();
    const { json: termJson, max } = renumberTermJson(src);
    seedTermSlots(max);
    const w = makeWorkspace(name || tpl.name);
    w.mainJson = tpl.mainJson ? JSON.parse(JSON.stringify(tpl.mainJson)) : undefined;
    w.termJson = termJson;
    setWorkspaces((prev) => [...prev, w]);
    setMainModel(mainModelFrom(w.mainJson));
    setTermModel(termModelFrom(termJson, nextSlot));
    setActiveWsId(w.id);
    setPage("terminal");
  };

  const deleteWorkspace = (id: number) => {
    const w = workspaces.find((x) => x.id === id);
    if (w) {
      const slots =
        id === activeWsId ? collectModelTermSlots(termModel) : jsonTermSlots(w.termJson);
      void closeSessions(slots);
    }
    const rest = workspaces.filter((x) => x.id !== id);
    if (rest.length === 0) {
      const nw = makeWorkspace("默认工作空间");
      nw.termJson = newTermJson();
      setWorkspaces([nw]);
      setMainModel(mainModelFrom(undefined));
      setTermModel(termModelFrom(nw.termJson, nextSlot));
      setActiveWsId(nw.id);
      return;
    }
    setWorkspaces(rest);
    if (activeWsId === id) {
      const next = rest[0];
      setMainModel(mainModelFrom(next.mainJson));
      setTermModel(termModelFrom(next.termJson, nextSlot));
      setActiveWsId(next.id);
    }
  };

  const renameWorkspace = (id: number, name: string) =>
    setWorkspaces((prev) => prev.map((w) => (w.id === id ? { ...w, name } : w)));
  const setLockPassword = (id: number, hash: string) =>
    setWorkspaces((prev) => prev.map((w) => (w.id === id ? { ...w, lockHash: hash } : w)));
  const unlockWorkspace = (id: number) =>
    setWorkspaces((prev) => prev.map((w) => (w.id === id ? { ...w, locked: false } : w)));
  const toggleLock = (id: number) =>
    setWorkspaces((prev) =>
      prev.map((w) =>
        w.id === id ? { ...w, locked: w.lockHash ? !w.locked : true } : w,
      ),
    );

  // ---- templates ---------------------------------------------------------
  const [templates, setTemplates] = useState<WsTemplate[]>(loadTemplates);
  useEffect(() => {
    persistTemplates(templates);
  }, [templates]);

  /** Menu 工作空间 > 布局另存为… / row right-click > 保存为模板. */
  const saveLayoutAsTemplate = (wsId: number, name: string) => {
    const live = wsId === activeWsId;
    const mainJson = live ? mainModel.toJson() : workspaces.find((w) => w.id === wsId)?.mainJson;
    const termJson = live ? termModel.toJson() : workspaces.find((w) => w.id === wsId)?.termJson;
    setTemplates((prev) => [
      ...prev,
      {
        id: crypto.randomUUID?.() ?? `${Date.now()}-${Math.random()}`,
        name,
        mainJson: mainJson ? JSON.parse(JSON.stringify(mainJson)) : undefined,
        termJson: termJson ? JSON.parse(JSON.stringify(termJson)) : undefined,
        createdAt: Date.now(),
      },
    ]);
    showToast(t(lang).tplSaved);
  };

  const saveLayout = () => {
    commitLayout();
    showToast(t(lang).layoutSaved);
  };
  const loadLayout = () => {
    const w = workspaces.find((x) => x.id === activeWsId);
    setMainModel(mainModelFrom(w?.mainJson));
    setTermModel(termModelFrom(w?.termJson, nextSlot));
    showToast(t(lang).layoutLoaded);
  };

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

  // ---- window panel toggles (视图 menu) ----------------------------------
  const navOpen = panelOpen(mainModel, NAV_TAB_ID);
  const termOpen = panelOpen(mainModel, PAGE_TAB_ID.terminal);
  const togglePanel = (panel: "nav" | "term") => {
    const tabId = panel === "nav" ? NAV_TAB_ID : PAGE_TAB_ID.terminal;
    if (panelOpen(mainModel, tabId)) {
      closePanel(mainModel, tabId);
      return;
    }
    const target =
      panel === "nav" ? rootRowId(mainModel) : mainTabsetId(mainModel);
    const location =
      panel === "nav" ? DockLocation.LEFT : DockLocation.RIGHT;
    mainModel.doAction(
      Actions.addNode(
        {
          type: "tab",
          id: tabId,
          name: PAGE_NAME[panel as Page],
          component: panel === "term" ? "term" : panel,
          enableClose: true,
          enableRenderOnDemand: false,
        },
        target,
        location,
        -1,
      ),
    );
    setPage(panel === "term" ? "terminal" : page);
  };

  /** Open a page as a floating window (terminals stay in the dock). */
  const openPage = (p: string) => {
    setPage(p as any);
    if (p !== "terminal") {
      const titles: Record<string, string> = {
        ssh: "SSH", history: "历史", ai: "AI 助手", settings: "设置",
        remote: "手机远程控制", "remote-wan": "远程控制 · 广域网", update: "检查更新",
        favorites: "收藏指令", about: "关于", tutorial: "教程", monitor: "监控",
      };
      openWindow(p, titles[p] ?? p, 640, 700);
    }
  };

  const addTerminal = (command?: string[]) => {
    const slot = nextSlot();
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
    const slot = nextSlot();
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
        onPage={openPage}
        updateAvailable={false}
        currentVersion="0.1.55"
        navOpen={navOpen}
        termOpen={termOpen}
        onTogglePanel={togglePanel}
        onSaveLayout={saveLayout}
        onLoadLayout={loadLayout}
        onSaveLayoutAs={(name) => saveLayoutAsTemplate(activeWsId, name)}
        onCreateWorkspace={(name) => createWorkspace(name)}
      />
      <div className="relative flex min-h-0 flex-1">
      <DockRoot
      key={activeWsId}
      mainModel={mainModel}
      termModel={termModel}
      onModelChange={() => {
        ensureSelection(mainModel);
        ensureSelection(termModel);
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
      onSwitchWorkspace={switchWorkspace}
      onCreateWorkspace={() => createWorkspace()}
      onDeleteWorkspace={deleteWorkspace}
      onRenameWorkspace={renameWorkspace}
      onSetLockPassword={setLockPassword}
      onUnlock={unlockWorkspace}
      onToggleLock={toggleLock}
      templates={templates}
      onCreateFromTemplate={(tpl, name) => createFromTemplate(tpl, name)}
      onSaveAsTemplate={(wsId, name) => saveLayoutAsTemplate(wsId, name)}
      onDeleteTemplate={(id) => setTemplates((prev) => prev.filter((x) => x.id !== id))}
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
      {toast && (
        <div className="animate-fade-up pointer-events-none fixed bottom-10 left-1/2 z-[9500] -translate-x-1/2 rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] px-4 py-2 text-[12px] text-[var(--text)] shadow-2xl">
          {toast}
        </div>
      )}
      {windows.map((w) => (
        <FloatingWindow key={w.id} win={w} onFocus={focusWindow} onClose={closeWindow} onMove={moveWindow}>
          {w.id === "settings" && (
            <SettingsPage settings={settings} onSettings={updateSettings} themeId={themeId} onTheme={setThemeId} lang={lang} />
          )}
          {w.id === "ssh" && (
            <SshPage hosts={sshHosts} onHosts={(h) => { setSshHosts(h); saveHosts(h); }} onConnect={connectSsh} />
          )}
          {w.id === "history" && <HistoryPage />}
          {w.id === "ai" && <AiPage />}
          {w.id === "remote" && <RemotePage />}
          {w.id === "remote-wan" && <RemotePage initialTab="wan" />}
          {w.id === "update" && <UpdatePage lang={lang} />}
          {w.id === "favorites" && <FavoritesPage />}
          {w.id === "about" && <AboutPage lang={lang} />}
          {w.id === "tutorial" && <TutorialPage lang={lang} />}
        </FloatingWindow>
      ))}
      </div>
    </div>
  );
}
