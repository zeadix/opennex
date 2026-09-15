// Workspace model (v2): a named, lockable entry in the navigation panel.
// Tab/layout state lives in the flexlayout dock models — workspaces are
// the organizational layer (names, grouping, lock), persisted locally.

export interface Workspace {
  id: number;
  name: string;
  locked: boolean;
}

const KEY = "opennex-workspaces";
let seed = 1;

export function nextId(): number {
  return seed++;
}

export function loadWorkspaces(): Workspace[] {
  try {
    const raw = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    if (Array.isArray(raw) && raw.length > 0) {
      const list = raw.map((w: any) => ({
        id: nextId(),
        name: String(w.name ?? "工作空间"),
        locked: !!w.locked,
      }));
      // Keep the seed clear of restored ids.
      seed = Math.max(seed, ...list.map((w) => w.id + 1));
      return list;
    }
  } catch {
    /* default below */
  }
  return [{ id: nextId(), name: "默认工作空间", locked: false }];
}

export function persistWorkspaces(list: Workspace[]) {
  localStorage.setItem(KEY, JSON.stringify(list));
}

export function makeWorkspace(name: string): Workspace {
  return { id: nextId(), name, locked: false };
}
