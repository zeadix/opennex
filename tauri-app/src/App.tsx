import { useEffect, useRef, useState } from "react";
import { Actions, DockLocation } from "flexlayout-react";
import DockRoot, { Page, PAGE_TAB_ID, seedTermSlots } from "./dock/DockRoot";
import {
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
import SshPage, { SshHost, loadHosts, saveHosts } from "./pages/SshPage";
import { loadWorkspaces, makeWorkspace, persistWorkspaces, Workspace } from "./workspaces";

export default function App() {
  const [themeId, setThemeId] = useTheme();
  const [settings, updateSettings] = useSettings();
  const [sshHosts, setSshHosts] = useState<SshHost[]>(loadHosts);

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

  const createWorkspace = () => {
    const w = makeWorkspace(`工作空间 ${workspaces.length + 1}`);
    setWorkspaces((prev) => [...prev, w]);
    setActiveWsId(w.id);
    openPage("terminal");
  };
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
            name: p === "terminal" ? "终端" : p.toUpperCase(),
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
    <DockRoot
      mainModel={mainModel}
      termModel={termModel}
      onModelChange={() => saveModels(mainModel, termModel)}
      themeId={themeId}
      onTheme={setThemeId}
      fontSize={settings.fontSize}
      shell={settings.shell}
      page={page}
      onOpenPage={openPage}
      workspaces={workspaces}
      activeWsId={activeWsId}
      onSwitchWorkspace={setActiveWsId}
      onCreateWorkspace={createWorkspace}
      onDeleteWorkspace={deleteWorkspace}
      sshHosts={sshHosts}
      onSshHosts={(h) => {
        setSshHosts(h);
        saveHosts(h);
      }}
      onConnectSsh={connectSsh}
      settings={settings}
      onSettings={updateSettings}
    />
  );
}
