// Dock layout models (flexlayout-react) + persistence.
// Two top-level panels ONLY: "nav" (workspace navigation) and the
// workspace area ("term" tab = terminals; other pages open as tabs in
// the same tabset). Both panels can be closed and reopened, unique.

import { Actions, DockLocation, Model } from "flexlayout-react";
import { jsonTermSlots } from "../workspaces";

export const NAV_TAB_ID = "nav";
export const NAV_TABSET_ID = "navset";
export const MAIN_TABSET_ID = "mainset";
export const TERM_TABSET_ID = "termset";
export const TERM_TAB_ID = "tab-term";

export const PAGE_TAB_ID: Record<string, string> = {
  terminal: TERM_TAB_ID,
  ssh: "tab-ssh",
  history: "tab-history",
  ai: "tab-ai",
  settings: "tab-settings",
};

export const PAGE_NAME: Record<string, string> = {
  terminal: "终端",
  ssh: "SSH",
  history: "历史",
  ai: "AI",
  settings: "设置",
};

function mainDefaultJson(): any {
  return {
    global: {
      tabEnableClose: true,
      tabSetEnableDeleteWhenEmpty: true,
      tabSetEnableMaximize: false,
    },
    layout: {
      type: "row",
      weight: 100,
      children: [
        {
          type: "tabset",
          id: NAV_TABSET_ID,
          weight: 20,
          selected: 0,
          children: [
            {
              type: "tab",
              id: NAV_TAB_ID,
              name: "导航",
              component: "nav",
              enableClose: true,
              enableRenderOnDemand: false,
            },
          ],
        },
        {
          type: "tabset",
          id: MAIN_TABSET_ID,
          weight: 80,
          selected: 0,
          children: [
            {
              type: "tab",
              id: TERM_TAB_ID,
              name: "终端工作区",
              component: "term",
              enableClose: true,
              enableRenderOnDemand: false,
            },
          ],
        },
      ],
    },
  };
}

/** Default workspace-area layout with ONE terminal on a fresh slot. */
function termDefaultJson(slot: number): any {
  return {
    global: {
      tabEnableClose: true,
      tabSetEnableDeleteWhenEmpty: true,
      tabSetEnableMaximize: false,
    },
    layout: {
      type: "row",
      weight: 100,
      children: [
        {
          type: "tabset",
          id: TERM_TABSET_ID,
          weight: 100,
          selected: 0,
          children: [
            {
              type: "tab",
              id: `term-${slot}`,
              name: `bash ${slot}`,
              component: "termpane",
              enableClose: true,
              enableRenderOnDemand: false,
              config: { slot },
            },
          ],
        },
      ],
    },
  };
}

function deepCopy(json: any): any {
  return JSON.parse(JSON.stringify(json));
}

function jsonHasTermPane(json: any): boolean {
  return jsonTermSlots(json).length > 0;
}

/** Repair a term-dock JSON before Model.fromJson: guarantee at least
 * one terminal (legacy blobs may carry an empty tabset) and no
 * `selected: -1` tabsets (renders as a black empty area). */
function prepareTermJson(json: any | undefined, fallbackSlot: () => number): any {
  let j = json ? deepCopy(json) : termDefaultJson(fallbackSlot());
  if (!jsonHasTermPane(j)) {
    const slot = fallbackSlot();
    const set = j.layout?.children?.[0];
    if (set) {
      set.children = set.children ?? [];
      set.children.push({
        type: "tab",
        id: `term-${slot}`,
        name: `bash ${slot}`,
        component: "termpane",
        enableClose: true,
        enableRenderOnDemand: false,
        config: { slot },
      });
      set.selected = 0;
    }
  }
  walkJsonTabsets(j, (set: any) => {
    if (typeof set.selected !== "number" || set.selected < 0) set.selected = 0;
  });
  return j;
}

function walkJsonTabsets(node: any, fn: (set: any) => void) {
  const walk = (n: any) => {
    if (!n || typeof n !== "object") return;
    if (Array.isArray(n)) {
      n.forEach(walk);
      return;
    }
    if (n.type === "tabset") fn(n);
    if (n.children) walk(n.children);
    if (n.layout) walk(n.layout);
  };
  walk(node);
}

/** Build a Model from a saved JSON, falling back to a fresh default when
 * absent/corrupt. Deep-copies the input (fromJson takes ownership). */
export function modelFromJson(json: any | undefined, makeDefault: () => any): Model {
  const src = json ? deepCopy(json) : makeDefault();
  try {
    return Model.fromJson(src);
  } catch {
    return Model.fromJson(makeDefault());
  }
}

/** Workspace-area model with repairs applied (≥1 terminal, valid selections). */
export function termModelFrom(json: any | undefined, fallbackSlot: () => number): Model {
  try {
    return Model.fromJson(prepareTermJson(json, fallbackSlot));
  } catch {
    return Model.fromJson(termDefaultJson(fallbackSlot()));
  }
}

export function mainModelFrom(json: any | undefined): Model {
  return modelFromJson(json, mainDefaultJson);
}

/** Fresh single-terminal workspace JSON on a globally unique slot. */
export function newTermJson(): any {
  return termDefaultJson(nextSlot());
}

/** Does the model contain any terminal pane tab? (legacy layouts saved
 * an empty tabset — the app seeds one on startup when this is false.) */
export function hasTermPane(model: Model): boolean {
  let found = false;
  const walk = (n: any) => {
    if (found) return;
    if (n.getType?.() === "tab") {
      if (n.getComponent?.() === "termpane") found = true;
    } else if (n.getChildren) {
      n.getChildren().forEach(walk);
    }
  };
  walk((model as any).getRoot?.() ?? (model as any).getRootRow());
  return found;
}

/** Root row node id (target for re-opening a closed top-level panel). */
export function rootRowId(model: Model): string {
  return ((model as any).getRootRow().getId());
}

/** Is a top-level panel tab currently in the layout? */
export function panelOpen(model: Model, tabId: string): boolean {
  return model.getNodeById(tabId) !== undefined;
}

/** Remove a top-level panel tab; the emptied tabset auto-deletes and the
 * sibling panel expands to fill the space (no blank areas). */
export function closePanel(model: Model, tabId: string) {
  if (panelOpen(model, tabId)) {
    model.doAction(Actions.deleteTab(tabId));
  }
}

/** (Re-)open a top-level panel tab, docking LEFT (nav) or splitting
 * into the root row; if the main tabset exists, dock beside it. */
export function openPanelLeft(model: Model, tabId: string, name: string, component: string) {
  if (panelOpen(model, tabId)) return;
  model.doAction(
    Actions.addNode(
      { type: "tab", id: tabId, name, component, enableClose: true, enableRenderOnDemand: false },
      rootRowId(model),
      DockLocation.LEFT,
      -1,
    ),
  );
}

/** Highest slot number referenced by terminal tabs (bash <n> naming). */
/** Make every tabset select a valid child (0 or first). A tabset whose
 * `selected` is -1 renders NOTHING — the classic "black empty area". */
export function ensureSelection(model: Model) {
  const walk = (n: any) => {
    if (n.getType?.() === "tabset") {
      const count = n.getChildren().length;
      if (count > 0 && (n.getSelected?.() ?? 0) < 0) {
        model.doAction(Actions.selectTab(n.getChildren()[0].getId()));
      }
    }
    n.getChildren?.().forEach((c: any) => walk(c));
  };
  try {
    walk((model as any).getRootRow?.() ?? (model as any).getRoot?.());
  } catch {
    /* best effort */
  }
}

/** Id of the main tabset (the second child of the root row). 0.11
 * regenerates ids for EMPTY tabsets, so callers must resolve this at
 * runtime instead of trusting a stored constant. */
export function mainTabsetId(model: Model): string {
  const children = (model as any).getRootRow().getChildren();
  return children[1]?.getId() ?? children[0]?.getId() ?? "";
}

/** Id of the terminal dock's tabset. */
export function termTabsetId(model: Model): string {
  const children = (model as any).getRootRow().getChildren();
  return children[0]?.getId() ?? "";
}

export function maxTermSlot(term: Model): number {
  return collectModelTermSlots(term).reduce((a, b) => Math.max(a, b), 0);
}

/** All terminal slots referenced by term-N tabs in a LIVE model. */
export function collectModelTermSlots(term: Model): number[] {
  const out: number[] = [];
  const walk = (n: any) => {
    if (n.getType?.() === "tabset") {
      for (const t of n.getChildren()) walk(t);
    } else if (n.getType?.() === "tab") {
      const m = /term-(\d+)/.exec(n.getId() ?? "");
      if (m) out.push(Number(m[1]));
    }
  };
  walk((term as any).getRootRow());
  return out;
}

// ---- global terminal-slot counter ---------------------------------------
// Slot numbers are GLOBAL across workspaces: a term-N id maps 1:1 to a
// backend PTY session, so two workspaces must never share one.
let slotCounter = 1;
export function seedTermSlots(v: number) {
  slotCounter = Math.max(slotCounter, v + 1);
}
export function nextSlot(): number {
  return slotCounter++;
}

/** Deep-copy a term layout JSON, renumbering every term-N tab to fresh
 * slots (used when spawning a workspace copy from a template — the
 * original's sessions must not be shared with the copy). */
export function renumberTermJson(json: any): { json: any; max: number } {
  const j = JSON.parse(JSON.stringify(json));
  let max = 0;
  const walk = (n: any) => {
    if (!n || typeof n !== "object") return;
    if (Array.isArray(n)) {
      n.forEach(walk);
      return;
    }
    const m = /^term-(\d+)$/.exec(String(n.id ?? ""));
    if (m) {
      const slot = nextSlot();
      n.id = `term-${slot}`;
      if (n.config) n.config.slot = slot;
      n.name = `bash ${slot}`;
      max = Math.max(max, slot);
    }
    if (n.children) walk(n.children);
  };
  walk(j);
  return { json: j, max };
}
