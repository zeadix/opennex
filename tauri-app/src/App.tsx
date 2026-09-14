import { useState } from "react";
import Sidebar, { Page } from "./components/Sidebar";
import StatusBar from "./components/StatusBar";
import TabStrip, { Tab } from "./components/TabStrip";
import TerminalPane from "./terminal/TerminalPane";
import PaneView from "./panes/PaneView";
import {
  PaneTree,
  closePane as treeClosePane,
  collectPanes,
  hasPane,
  leaf,
  setRatio as treeSetRatio,
  splitPane as treeSplitPane,
} from "./panes/tree";
import { FiTool } from "react-icons/fi";
import { useTheme } from "./theme/useTheme";
import SettingsPage from "./pages/SettingsPage";
import SshPage, { SshHost, loadHosts, saveHosts } from "./pages/SshPage";
import { useSettings } from "./settings";

let nextPaneId = 1;

/** A terminal tab owns one pane tree; every leaf is a live PTY session. */
interface TermTab extends Tab {
  tree: PaneTree;
  activePane: number;
  command?: string[];
}

function newTab(): TermTab {
  const pane = nextPaneId++;
  return { id: nextPaneId++, title: `bash ${pane}`, tree: leaf(pane), activePane: pane };
}

function Placeholder({ label }: { label: string }) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2 text-[var(--text-faint)]">
      <FiTool size={28} />
      <div className="text-sm">{label} · 开发中</div>
    </div>
  );
}

function makeTab(): TermTab {
  const pane = nextPaneId++;
  return { id: nextPaneId++, title: `bash ${pane}`, tree: leaf(pane), activePane: pane };
}

export default function App() {
  const [themeId, setThemeId] = useTheme();
  const [settings, updateSettings] = useSettings();
  const [sshHosts, setSshHosts] = useState<SshHost[]>(loadHosts);
  const [page, setPage] = useState<Page>("terminal");
  const [tabs, setTabs] = useState<TermTab[]>([makeTab()]);
  const [activeTab, setActiveTab] = useState(tabs[0].id);

  const patchTab = (id: number, patch: (t: TermTab) => TermTab) =>
    setTabs((prev) => prev.map((t) => (t.id === id ? patch(t) : t)));

  const newTab = (command?: string[]) => {
    const t = makeTab();
    t.command = command;
    setTabs((prev) => [...prev, t]);
    setActiveTab(t.id);
    setPage("terminal");
  };
  const connectSsh = (h: SshHost) => {
    saveHosts(sshHosts);
    newTab(["ssh", "-p", String(h.port), `${h.user}@${h.host}`]);
  };
  const closeTab = (id: number) => {
    setTabs((prev) => {
      const victim = prev.find((t) => t.id === id);
      if (victim) {
        // Closing the tab kills every session in its pane tree.
        collectPanes(victim.tree).forEach(() => {
          /* WS close in TerminalPane unmount triggers backend cleanup */
        });
      }
      const rest = prev.filter((t) => t.id !== id);
      if (rest.length === 0) {
        const t = makeTab();
        setActiveTab(t.id);
        return [t];
      }
      if (activeTab === id) setActiveTab(rest[rest.length - 1].id);
      return rest;
    });
  };

  const splitPane = (tabId: number, pane: number, dir: "h" | "v") => {
    const newPane = nextPaneId++;
    patchTab(tabId, (t) => ({
      ...t,
      tree: treeSplitPane(t.tree, pane, dir, newPane),
      activePane: newPane,
      title: t.title,
    }));
  };
  const closePane = (tabId: number, pane: number) => {
    patchTab(tabId, (t) => {
      const next = treeClosePane(t.tree, pane);
      const alivePanes = collectPanes(next).filter((p) => p > 0);
      const activePane = hasPane(next, t.activePane)
        ? t.activePane
        : alivePanes[alivePanes.length - 1] ?? 0;
      return { ...t, tree: next, activePane };
    });
  };
  const setRatio = (tabId: number, path: number[], ratio: number[]) => {
    patchTab(tabId, (t) => ({ ...t, tree: treeSetRatio(t.tree, path, ratio) }));
  };
  const activatePane = (tabId: number, pane: number) => {
    patchTab(tabId, (t) => (t.activePane === pane ? t : { ...t, activePane: pane }));
  };

  const activeTermTab = tabs.find((t) => t.id === activeTab);

  return (
    <div className="flex h-full">
      <Sidebar
        page={page}
        onNavigate={setPage}
        themeId={themeId}
        onTheme={setThemeId}
      />
      <div className="flex min-w-0 flex-1 flex-col">
        {page === "terminal" && activeTermTab ? (
          <>
            <TabStrip
              tabs={tabs}
              active={activeTab}
              onSelect={setActiveTab}
              onClose={closeTab}
              onNew={newTab}
            />
            <div className="relative min-h-0 flex-1">
              {tabs.map((t) => (
                <div
                  key={t.id}
                  className="absolute inset-0 flex"
                  style={{ display: activeTab === t.id ? "flex" : "none" }}
                >
                  {hasPane(t.tree, t.activePane) || t.tree.kind === "split" ? (
                    <PaneView
                      tree={t.tree}
                      path={[]}
                      activePane={t.activePane}
                      themeId={themeId}
                      fontSize={settings.fontSize}
                      shell={settings.shell}
                      command={t.command}
                      onActivate={(p) => activatePane(t.id, p)}
                      onClose={(p) => closePane(t.id, p)}
                      onSplit={(p, d) => splitPane(t.id, p, d)}
                      onRatio={(path, ratio) => setRatio(t.id, path, ratio)}
                    />
                  ) : (
                    <TerminalPane sessionId={t.activePane} themeId={themeId} fontSize={settings.fontSize} shell={settings.shell} />
                  )}
                </div>
              ))}
            </div>
            <StatusBar sessionCount={tabs.length} shell="bash" />
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
          <Placeholder label="指令历史" />
        ) : page === "ai" ? (
          <Placeholder label="AI 助手" />
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
}
