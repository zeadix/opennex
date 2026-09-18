// Workspaces and templates own only the terminal dock and terminal paths.
// The outer view-panel layout is persisted independently in dock/mainLayout.ts.

export interface Workspace {
  id: number;
  name: string;
  locked: boolean;
  lockHash?: string;
  termJson?: any;
  /** Per-terminal working directory memory: slot -> cwd (captured on
   * switch/save, restored on open/template copy). */
  cwdMap?: Record<string, string>;
}

export interface WsTemplate {
  id: string;
  name: string;
  termJson: any;
  cwdMap?: Record<string, string>;
  createdAt: number;
}

const KEY = "opennex-workspaces";
const TPL_KEY = "opennex-templates";
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
        lockHash: w.lockHash,
        termJson: w.termJson ?? undefined,
        cwdMap: w.cwdMap ?? undefined,
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

// ---- templates ---------------------------------------------------------

export function loadTemplates(): WsTemplate[] {
  try {
    const raw = JSON.parse(localStorage.getItem(TPL_KEY) ?? "[]");
    if (Array.isArray(raw)) {
      return raw.map((t: any) => ({
        id: String(t.id ?? crypto.randomUUID?.() ?? `${Date.now()}`),
        name: String(t.name ?? "模板"),
        termJson: t.termJson ?? undefined,
        cwdMap: t.cwdMap ?? undefined,
        createdAt: Number(t.createdAt ?? Date.now()),
      }));
    }
  } catch {
    /* fallthrough */
  }
  return [];
}

export function persistTemplates(list: WsTemplate[]) {
  localStorage.setItem(TPL_KEY, JSON.stringify(list));
}

// ---- terminal-slot helpers (raw flexlayout JSON) -----------------------

/** All terminal slot numbers referenced by term-N tab ids in a layout JSON. */
export function jsonTermSlots(json: any): number[] {
  const out: number[] = [];
  const walk = (n: any) => {
    if (!n || typeof n !== "object") return;
    if (Array.isArray(n)) {
      n.forEach(walk);
      return;
    }
    const m = /^term-(\d+)$/.exec(String(n.id ?? ""));
    if (m) out.push(Number(m[1]));
    if (n.children) walk(n.children);
    if (n.layout) walk(n.layout);
  };
  walk(json);
  return out;
}

export function maxJsonSlot(json: any): number {
  return jsonTermSlots(json).reduce((a, b) => Math.max(a, b), 0);
}

export const LOCK_SALT = "opennex-ws-lock-v1";

export async function sha256(text: string): Promise<string> {
  const data = new TextEncoder().encode(text);
  const buf = await crypto.subtle.digest("SHA-256", data);
  return [...new Uint8Array(buf)].map((b) => b.toString(16).padStart(2, "0")).join("");
}
