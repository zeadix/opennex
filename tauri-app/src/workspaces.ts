// Workspace model + persistence. A workspace owns an independent set of
// terminal tabs (each with its own pane tree). Terminal sessions are NOT
// persisted — tabs are recreated (title + command kept) on restore.

import { PaneTree, leaf } from "./panes/tree";

export interface WsTab {
  id: number;
  title: string;
  tree: PaneTree;
  activePane: number;
  command?: string[];
}

export interface Workspace {
  id: number;
  name: string;
  tabs: WsTab[];
  activeTabId: number;
  locked: boolean;
  lockHash?: string;
}

const KEY = "opennex-workspaces";

let idSeed = 1;
export function nextId(): number {
  return idSeed++;
}
export function bumpIdSeed(v: number) {
  idSeed = Math.max(idSeed, v + 1);
}

interface StoredTab {
  title: string;
  command?: string[];
  tree: PaneTree;
  activePane?: number;
}
interface StoredWs {
  name: string;
  tabs: StoredTab[];
  activeTabIdx: number;
  locked: boolean;
  lockHash?: string;
}

export function persist(workspaces: Workspace[]) {
  const data: StoredWs[] = workspaces.map((w) => ({
    name: w.name,
    tabs: w.tabs.map((t) => ({
      title: t.title,
      command: t.command,
      tree: t.tree,
      activePane: t.activePane,
    })),
    activeTabIdx: Math.max(
      0,
      w.tabs.findIndex((t) => t.id === w.activeTabId),
    ),
    locked: w.locked,
    lockHash: w.lockHash,
  }));
  localStorage.setItem(KEY, JSON.stringify(data));
}

export function restore(): Workspace[] {
  try {
    const data: StoredWs[] = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    if (!Array.isArray(data) || data.length === 0) return [];
    return data.map((w) => {
      const tabs: WsTab[] = w.tabs.map((t) => {
        const tabId = nextId();
        // Remap stored pane ids to FRESH ids so restored trees can never
        // collide with newly created panes.
        const map = new Map<number, number>();
        const tree = remapTreePanes(t.tree, (oldPane) => {
          const fresh = nextId();
          bumpIdSeed(fresh);
          map.set(oldPane, fresh);
          return fresh;
        });
        const activePane = map.get(t.activePane ?? 0) ?? [...map.values()][0] ?? 0;
        return { id: tabId, title: t.title, command: t.command, tree, activePane };
      });
      const activeTab = tabs[Math.min(w.activeTabIdx, tabs.length - 1)] ?? tabs[0];
      return {
        id: nextId(),
        name: w.name,
        tabs: tabs.length ? tabs : [makeTab()],
        activeTabId: activeTab?.id ?? tabs[0]?.id ?? 0,
        locked: w.locked,
        lockHash: w.lockHash,
      };
    });
  } catch {
    return [];
  }
}

export function makeTab(command?: string[]): WsTab {
  const pane = nextId();
  bumpIdSeed(pane);
  const id = nextId();
  return { id, title: `bash ${pane}`, tree: leaf(pane), activePane: pane, command };
}

export function makeWorkspace(name: string): Workspace {
  const tab = makeTab();
  return {
    id: nextId(),
    name,
    tabs: [tab],
    activeTabId: tab.id,
    locked: false,
  };
}

/** Remap every leaf pane id in a stored tree to fresh session ids. */
export function remapTreePanes(tree: PaneTree, map: (old: number) => number): PaneTree {
  if (tree.kind === "leaf") return { kind: "leaf", pane: map(tree.pane) };
  return {
    kind: "split",
    dir: tree.dir,
    children: tree.children.map((c) => remapTreePanes(c, map)),
    ratio: tree.ratio,
  };
}

export async function sha256(text: string): Promise<string> {
  const data = new TextEncoder().encode(text);
  const buf = await crypto.subtle.digest("SHA-256", data);
  return [...new Uint8Array(buf)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

export const LOCK_SALT = "opennex-ws-lock-v1";
