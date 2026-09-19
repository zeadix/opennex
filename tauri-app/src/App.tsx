import { useEffect, useMemo, useRef, useState } from "react";
import { Actions, DockLocation, Model } from "flexlayout-react";
import DockRoot, { Page, PAGE_TAB_ID } from "./dock/DockRoot";
import {
  closePanel,
  NAV_TAB_ID,
  ensureSelection,
  termModelFrom,
  newTermJson,
  nextSlot,
  seedTermSlots,
  renumberTermJson,
  collectModelTermSlots,
  withCwd,
  cwdMapFromJson,
  mainTabsetId,
  panelOpen,
  rootRowId,
  termTabsetId,
  canCloseTerminal,
} from "./dock/model";
import { loadMainModel, persistMainModel } from "./dock/mainLayout";
import { panelTitles, translatePanelTitles } from "./dock/titles";
import { I18nProvider } from "./i18n-context";
import { useTheme } from "./theme/useTheme";
import { applyBackgroundImage, getTheme } from "./theme/themes";
import { useSettings } from "./settings";
import { loadLang, saveLang, t, Lang } from "./i18n";
import { loadShortcuts, matchesBinding } from "./shortcuts/shortcuts";
import { activityStore, broadcastGroup } from "./terminal/registry";
import { checkForUpdates, startUpdateCheck, useUpdates } from "./updates";
import SshPage, { SshHost, loadHosts, saveHosts } from "./pages/SshPage";
import RemotePage from "./pages/RemotePage";
import UpdatePage from "./pages/UpdatePage";
import FloatingWindow, { FloatWin } from "./dock/FloatingWindow";
import SettingsPage from "./pages/SettingsPage";
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
  const titles = panelTitles(lang);
  const [settings, updateSettings] = useSettings();
  const [sshHosts, setSshHosts] = useState<SshHost[]>(loadHosts);
  const [shells, setShells] = useState<string[]>([]);
  const [activities, setActivities] = useState<Record<string, number>>({});
  const [historyOverlay, setHistoryOverlay] = useState(false);
  const [windows, setWindows] = useState<FloatWin[]>([]);
  const winZ = useRef(1);
  const updates = useUpdates();
  useEffect(startUpdateCheck, []);
  const [toast, setToast] = useState<string | null>(null);
  const toastTimer = useRef<number | null>(null);
  const showToast = (msg: string, ms = 2200) => {
    setToast(msg);
    if (toastTimer.current) window.clearTimeout(toastTimer.current);
    toastTimer.current = window.setTimeout(() => setToast(null), ms);
  };
  useEffect(
    () => () => {
      if (toastTimer.current) window.clearTimeout(toastTimer.current);
    },
    [],
  );

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

  // ---- workspaces own only terminal layouts -------------------------------------
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

  // The outer dock survives every workspace operation.
  const [mainModel] = useState<Model>(loadMainModel);
  useEffect(() => {
    const save = () => persistMainModel(mainModel);
    save();
    mainModel.addChangeListener(save);
    return () => mainModel.removeChangeListener(save);
  }, [mainModel]);
  const [termModel, setTermModel] = useState<Model>(() =>
    termModelFrom(withCwd(workspaces[0]?.termJson, workspaces[0]?.cwdMap), nextSlot),
  );

  useEffect(() => { translatePanelTitles(mainModel, lang); }, [mainModel, lang]);

  // ---- terminal path memory ----------------------------------------------
  const termModelRef = useRef(termModel);
  termModelRef.current = termModel;
  const cwdRef = useRef<Record<string, string>>({});
  const captureCwdMap = async (): Promise<Record<string, string>> => {
    const slots = collectModelTermSlots(termModelRef.current);
    if (slots.length === 0) return {};
    try {
      const m = await import("@tauri-apps/api/core");
      const map: Record<string, string> = {};
      await Promise.all(
        slots.map(async (slot) => {
          try {
            const cwd = await m.invoke<string | null>("session_cwd", { sessionId: String(slot) });
            if (cwd) map[String(slot)] = cwd;
          } catch {
            /* session gone */
          }
        }),
      );
      return map;
    } catch {
      return {};
    }
  };
  /** Persist the live cwd map onto a workspace record (fire-and-forget). */
  const storeCwdMap = (id: number) => {
    void captureCwdMap().then((cwdMap) => {
      cwdRef.current = cwdMap;
      if (Object.keys(cwdMap).length === 0) return;
      setWorkspaces((prev) => prev.map((w) => (w.id === id ? { ...w, cwdMap } : w)));
    });
  };

  useEffect(() => {
    persistWorkspaces(workspaces);
  }, [workspaces]);

  /** Snapshot the terminal layout into the active workspace's committed copy. */
  const commitLayout = () => {
    const snapTerm = termModel.toJson();
    setWorkspaces((prev) =>
      prev.map((w) => (w.id === activeWsId ? { ...w, termJson: snapTerm } : w)),
    );
    storeCwdMap(activeWsId);
  };

  // App close without an explicit save: best-effort commit straight to
  // localStorage (setState would not flush in time).
  const commitRef = useRef<() => void>(() => {});
  commitRef.current = () => {
    persistMainModel(mainModel);
    try {
      const list: Workspace[] = JSON.parse(localStorage.getItem("opennex-workspaces") ?? "[]");
      const i = list.findIndex((w) => w.id === activeWsId);
      if (i >= 0) {
        list[i].termJson = termModel.toJson();
        if (Object.keys(cwdRef.current).length > 0) list[i].cwdMap = cwdRef.current;
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
    const snapTerm = termModel.toJson();
    setWorkspaces((prev) =>
      prev.map((w) => (w.id === activeWsId ? { ...w, termJson: snapTerm } : w)),
    );
    storeCwdMap(activeWsId);
    setTermModel(termModelFrom(withCwd(target.termJson, target.cwdMap), nextSlot));
    setActiveWsId(id);
    setPage("terminal");
  };

  const createWorkspace = (name?: string) => {
    commitLayout();
    const w = makeWorkspace(name ?? `${t(lang).cWorkspace} ${workspaces.length + 1}`);
    w.termJson = newTermJson(); // one fresh terminal on a unique slot
    setWorkspaces((prev) => [...prev, w]);
    void captureCwdMap().then((cwdMap) => cwdRef.current = cwdMap);
    setTermModel(termModelFrom(w.termJson, nextSlot));
    setActiveWsId(w.id);
    setPage("terminal");
  };

  /** Template copy: identical layout, fresh terminal slots (the copy
   * must not share PTY sessions with the template's source). */
  const createFromTemplate = (tpl: WsTemplate, name: string) => {
    commitLayout();
    // Inject the template's per-terminal paths BEFORE renumbering — the
    // cwd travels with each tab config to its fresh slot.
    const src = withCwd(tpl.termJson ?? newTermJson(), tpl.cwdMap);
    const { json: termJson, max } = renumberTermJson(src);
    seedTermSlots(max);
    const w = makeWorkspace(name || tpl.name);
    w.termJson = termJson;
    w.cwdMap = cwdMapFromJson(termJson);
    setWorkspaces((prev) => [...prev, w]);
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
      const nw = makeWorkspace(t(lang).cDefaultWorkspace);
      nw.termJson = newTermJson();
      setWorkspaces([nw]);
      setTermModel(termModelFrom(nw.termJson, nextSlot));
      setActiveWsId(nw.id);
      return;
    }
    setWorkspaces(rest);
    if (activeWsId === id) {
      const next = rest[0];
      setTermModel(termModelFrom(withCwd(next.termJson, next.cwdMap), nextSlot));
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
    const termJson = live ? termModel.toJson() : workspaces.find((w) => w.id === wsId)?.termJson;
    // Live save captures each terminal's current cwd so the template
    // restores paths on copy.
    void (async () => {
      const cwdMap = live ? await captureCwdMap() : (workspaces.find((w) => w.id === wsId)?.cwdMap ?? {});
      setTemplates((prev) => [
        ...prev,
        {
          id: crypto.randomUUID?.() ?? `${Date.now()}-${Math.random()}`,
          name,
          termJson: termJson ? JSON.parse(JSON.stringify(termJson)) : undefined,
          cwdMap,
          createdAt: Date.now(),
        },
      ]);
      showToast(t(lang).tplSaved);
    })();
  };

  const saveLayout = () => {
    commitLayout();
    showToast(t(lang).layoutSaved);
  };
  const loadLayout = () => {
    const w = workspaces.find((x) => x.id === activeWsId);
    setTermModel(termModelFrom(withCwd(w?.termJson, w?.cwdMap), nextSlot));
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
        .then((m) => {
          activityStore.map = m;
          setActivities(m);
        })
        .catch(() => {});
      void captureCwdMap().then((cwdMap) => (cwdRef.current = cwdMap));
    };
    poll();
    const id = window.setInterval(poll, 3000);
    return () => window.clearInterval(id);
  }, []);

  // ---- main-dock panel toggles (视图 menu) -------------------------------
  // View-menu panels are DOCK PANES of the main layout — same level as
  // the nav panel and the workspace area, freely splittable by drag.
  // The main dock and the terminal dock are separate Models, so these
  // panels can never be dragged into the terminal area (and terminals
  // never leave it).
  const MAIN_PANELS: Record<string, { tabId: string; name: string; location: DockLocation }> = {
    "quick-settings": { tabId: PAGE_TAB_ID["quick-settings"], name: t(lang).quickSettings, location: DockLocation.RIGHT },
    nav: { tabId: NAV_TAB_ID, name: titles.nav, location: DockLocation.LEFT },
    term: { tabId: PAGE_TAB_ID.terminal, name: titles.term, location: DockLocation.RIGHT },
    sysmon: { tabId: PAGE_TAB_ID.sysmon, name: titles.sysmon, location: DockLocation.RIGHT },
    ai: { tabId: PAGE_TAB_ID.ai, name: titles.ai, location: DockLocation.RIGHT },
    history: { tabId: PAGE_TAB_ID.history, name: titles.history, location: DockLocation.RIGHT },
    favorites: { tabId: PAGE_TAB_ID.favorites, name: titles.favorites, location: DockLocation.RIGHT },
    ssh: { tabId: PAGE_TAB_ID.ssh, name: titles.ssh, location: DockLocation.RIGHT },
  };
  const panelChecks: Record<string, boolean> = {
    "quick-settings": panelOpen(mainModel, PAGE_TAB_ID["quick-settings"]),
    nav: panelOpen(mainModel, NAV_TAB_ID),
    term: panelOpen(mainModel, PAGE_TAB_ID.terminal),
    sysmon: panelOpen(mainModel, PAGE_TAB_ID.sysmon),
    ai: panelOpen(mainModel, PAGE_TAB_ID.ai),
    history: panelOpen(mainModel, PAGE_TAB_ID.history),
    favorites: panelOpen(mainModel, PAGE_TAB_ID.favorites),
    ssh: panelOpen(mainModel, PAGE_TAB_ID.ssh),
  };
  const togglePanel = (panel: string) => {
    const def = MAIN_PANELS[panel];
    if (!def) return;
    if (panelOpen(mainModel, def.tabId)) {
      closePanel(mainModel, def.tabId);
      return;
    }
    const target = panel === "nav" ? rootRowId(mainModel) : mainTabsetId(mainModel);
    mainModel.doAction(
      Actions.addNode(
        {
          type: "tab",
          id: def.tabId,
          name: def.name,
          component: panel === "term" ? "term" : panel,
          enableClose: true,
          enableRenderOnDemand: false,
        },
        target,
        def.location,
        -1,
      ),
    );
    setPage(panel === "term" ? "terminal" : page);
  };

  // ---- global shortcuts (recordable in Settings; dispatched here) -------
  // Refs mirror the latest closures; the capture listener installs once.
  const addTerminalRef = useRef(() => {});
  addTerminalRef.current = () => addTerminal();
  const toggleLockRef = useRef(() => {});
  toggleLockRef.current = () => toggleLock(activeWsId);
  const cycleWsRef = useRef((_d: number) => {});
  cycleWsRef.current = (d: number) => {
    const idx = workspaces.findIndex((w) => w.id === activeWsId);
    if (idx < 0 || workspaces.length === 0) return;
    const next = workspaces[(idx + d + workspaces.length) % workspaces.length];
    switchWorkspace(next.id);
  };
  const closeTabRef = useRef(() => {});
  closeTabRef.current = () => {
    try {
      const ts: any = termModel.getNodeById(termTabsetId(termModel));
      const children = ts?.getChildren?.() ?? [];
      const sel = ts?.getSelectedNode?.() ?? children[ts.getSelected?.() ?? 0];
      if (sel && canCloseTerminal(termModel, sel.getId())) {
        const slot = /term-(\d+)/.exec(sel.getId())?.[1];
        if (slot) {
          void closeSessions([Number(slot)]);
          broadcastGroup.delete(Number(slot));
        }
        termModel.doAction(Actions.deleteTab(sel.getId()));
      }
    } catch {
      /* no tabset */
    }
  };
  /** Cycle the selected terminal tab inside the terminal dock. */
  const nextTerminalRef = useRef(() => {});
  nextTerminalRef.current = () => {
    try {
      const set: any = termModel.getNodeById(termTabsetId(termModel));
      const kids: any[] = set?.getChildren?.() ?? [];
      if (kids.length < 1) return;
      const cur = set.getSelectedNode?.() ?? kids[set.getSelected?.() ?? 0];
      const idx = kids.findIndex((k) => k.getId?.() === cur?.getId?.());
      termModel.doAction(Actions.selectTab(kids[(idx + 1) % kids.length].getId()));
    } catch {
      /* ignore */
    }
  };
  /** Cycle the selected tab of the MAIN dock (nav / workspace area / view panels). */
  const nextPanelRef = useRef(() => {});
  nextPanelRef.current = () => {
    try {
      const set: any = mainModel.getNodeById(mainTabsetId(mainModel));
      const kids: any[] = set?.getChildren?.() ?? [];
      if (kids.length < 1) return;
      const cur = set.getSelectedNode?.() ?? kids[set.getSelected?.() ?? 0];
      const idx = kids.findIndex((k) => k.getId?.() === cur?.getId?.());
      mainModel.doAction(Actions.selectTab(kids[(idx + 1) % kids.length].getId()));
    } catch {
      /* ignore */
    }
  };
  const saveLayoutRef = useRef(() => {});
  saveLayoutRef.current = () => saveLayout();
  const togglePanelRef = useRef<(panel: string) => void>(() => {});
  togglePanelRef.current = (panel) => togglePanel(panel);
  const uiFontSizeRef = useRef(settings.uiFontSize);
  uiFontSizeRef.current = settings.uiFontSize;
  const updateSettingsRef = useRef(updateSettings);
  updateSettingsRef.current = updateSettings;
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const bs = loadShortcuts();
      if (matchesBinding(e, bs.historyMenu)) {
        e.preventDefault();
        // Exclusive pair: the palette closes the auto-match list.
        window.dispatchEvent(new CustomEvent("opennex-close-suggest"));
        setHistoryOverlay((v) => !v);
      } else if (matchesBinding(e, bs.newTerminal)) {
        e.preventDefault();
        addTerminalRef.current();
      } else if (matchesBinding(e, bs.closeTerminal)) {
        e.preventDefault();
        closeTabRef.current();
      } else if (matchesBinding(e, bs.search)) {
        e.preventDefault();
        window.dispatchEvent(new CustomEvent("opennex-search"));
      } else if (matchesBinding(e, bs.lockWorkspace)) {
        e.preventDefault();
        toggleLockRef.current();
      } else if (matchesBinding(e, bs.workspaceNext)) {
        e.preventDefault();
        cycleWsRef.current(1);
      } else if (matchesBinding(e, bs.workspacePrev)) {
        e.preventDefault();
        cycleWsRef.current(-1);
      } else if (matchesBinding(e, bs.workspaceUp)) {
        e.preventDefault();
        cycleWsRef.current(-1);
      } else if (matchesBinding(e, bs.workspaceDown)) {
        e.preventDefault();
        cycleWsRef.current(1);
      } else if (matchesBinding(e, bs.nextTerminal)) {
        e.preventDefault();
        nextTerminalRef.current();
      } else if (matchesBinding(e, bs.nextPanel)) {
        e.preventDefault();
        nextPanelRef.current();
      } else if (matchesBinding(e, bs.saveLayout)) {
        e.preventDefault();
        saveLayoutRef.current();
      } else if (matchesBinding(e, bs.toggleSidebar)) {
        e.preventDefault();
        togglePanelRef.current("nav");
      } else if (matchesBinding(e, bs.zoomIn) || matchesBinding(e, bs.zoomOut)) {
        e.preventDefault();
        const step = matchesBinding(e, bs.zoomIn) ? 1 : -1;
        const next = Math.min(18, Math.max(11, uiFontSizeRef.current + step));
        if (next !== uiFontSizeRef.current) updateSettingsRef.current({ uiFontSize: next });
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, []);

  // The auto-match overlay closes the Alt palette (exclusive pair).
  useEffect(() => {
    const close = () => setHistoryOverlay(false);
    window.addEventListener("opennex-close-palette", close);
    return () => window.removeEventListener("opennex-close-palette", close);
  }, []);

  // Panes surface lightweight hints (e.g. copy-on-select) via this event.
  // detail: string, or { text, ms } for a custom toast duration.
  useEffect(() => {
    const onToast = (e: Event) => {
      const msg = (e as CustomEvent).detail;
      if (typeof msg === "string") showToast(msg);
      else if (msg && typeof msg === "object") showToast(String(msg.text ?? ""), Number(msg.ms) || 2200);
    };
    window.addEventListener("opennex-toast", onToast);
    return () => window.removeEventListener("opennex-toast", onToast);
  }, []);

  // Sync the history cap to the backend whenever it changes.
  useEffect(() => {
    import("@tauri-apps/api/core")
      .then((m) => m.invoke("set_history_cap", { workspaceId: activeWsId, cap: settings.historyCap }))
      .catch(() => {});
  }, [settings.historyCap, activeWsId]);

  // User theme packs may change without a themeId change (re-save).
  const [themesV, setThemesV] = useState(0);
  useEffect(() => {
    const bump = () => setThemesV((v) => v + 1);
    window.addEventListener("opennex-themes-changed", bump);
    return () => window.removeEventListener("opennex-themes-changed", bump);
  }, []);

  // Effective font pack: the theme's fonts win when 使用主题字体 is on
  // and the theme defines one; otherwise the global settings apply.
  const effFonts = useMemo(() => {
    if (settings.useThemeFont) {
      const th = getTheme(themeId);
      if (th.font) return th.font;
    }
    return {
      uiFont: settings.uiFont,
      uiFontSize: settings.uiFontSize,
      termFont: settings.termFont,
      termFontSize: settings.fontSize,
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [settings.useThemeFont, settings.uiFont, settings.uiFontSize, settings.termFont, settings.fontSize, themeId, themesV]);

  // Font settings: UI family via --ui-font, terminal family via --mono
  // (panes re-read both on the terminal-style event).
  useEffect(() => {
    const root = document.documentElement;
    if (effFonts.uiFont) root.style.setProperty("--ui-font", effFonts.uiFont);
    else root.style.removeProperty("--ui-font");
    root.style.setProperty(
      "--mono",
      effFonts.termFont ||
        '"JetBrains Mono", "Cascadia Code", "Fira Code", ui-monospace, monospace',
    );
    window.dispatchEvent(new CustomEvent("opennex-terminal-theme"));
  }, [effFonts]);

  /** UI scale from the effective 界面字号 (13px = 1.0). */
  const uiScale = effFonts.uiFontSize / 13;

  // 全局背景图片：图层 + 半透明面板。主题切换/编辑器预览后需重新应用。
  const bgCfg = settings.bgImageData
    ? {
        data: settings.bgImageData,
        opacity: settings.bgImageOpacity,
        fit: settings.bgImageFit,
        panelAlpha: settings.bgImagePanelAlpha,
      }
    : null;
  useEffect(() => {
    const reapply = () => applyBackgroundImage(bgCfg, getTheme(themeId));
    reapply();
    window.addEventListener("opennex-terminal-theme", reapply);
    return () => window.removeEventListener("opennex-terminal-theme", reapply);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [bgCfg?.data, bgCfg?.opacity, bgCfg?.fit, bgCfg?.panelAlpha, themeId]);

  /** Open a page as a floating window (terminals stay in the dock). */
  const openPage = (p: string) => {
    if (p === "update") void checkForUpdates();
    setPage(p as any);
    if (p !== "terminal") {
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
    <I18nProvider lang={lang}>
    <div className="flex h-full flex-col">
      <div className="shrink-0" style={{ zoom: uiScale }}>
      <TopBar
        lang={lang}
        onLang={setLang}
        themeId={themeId}
        onTheme={setThemeId}
        onPage={openPage}
        updateAvailable={!!updates.result?.updateAvailable}
        currentVersion={updates.current ?? "—"}
        panelChecks={panelChecks}
        onTogglePanel={togglePanel}
        onSaveLayout={saveLayout}
        onLoadLayout={loadLayout}
        onSaveLayoutAs={(name) => saveLayoutAsTemplate(activeWsId, name)}
        onCreateWorkspace={(name) => createWorkspace(name)}
      />
      </div>
      <div className="relative flex min-h-0 flex-1">
      <div className="min-h-0 flex-1" style={{ zoom: uiScale }}>
      <DockRoot
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
      fontSize={effFonts.termFontSize}
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
      </div>
      {historyOverlay && <HistoryOverlay workspaceId={activeWsId} onClose={() => setHistoryOverlay(false)} />}
      {toast && (
        <div className="animate-fade-up pointer-events-none fixed left-1/2 top-10 z-[9500] -translate-x-1/2 rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] px-4 py-2 text-[12px] text-[var(--text)] shadow-2xl">
          {toast}
        </div>
      )}
      {windows.map((w) => (
        <FloatingWindow key={w.id} win={{ ...w, title: titles[w.id] ?? w.title }} onFocus={focusWindow} onClose={closeWindow} onMove={moveWindow}>
          {w.id === "settings" && (
            <SettingsPage settings={settings} onSettings={updateSettings} themeId={themeId} onTheme={setThemeId} lang={lang} />
          )}
          {w.id === "ssh" && (
            <SshPage hosts={sshHosts} onHosts={(h) => { setSshHosts(h); saveHosts(h); }} onConnect={connectSsh} />
          )}
          {w.id === "remote" && <RemotePage />}
          {w.id === "remote-wan" && <RemotePage initialTab="wan" />}
          {w.id === "update" && <UpdatePage lang={lang} />}
          {w.id === "about" && <AboutPage lang={lang} />}
          {w.id === "tutorial" && <TutorialPage lang={lang} />}
        </FloatingWindow>
      ))}
      </div>
    </div>
    </I18nProvider>
  );
}
