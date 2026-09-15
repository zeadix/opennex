// Dock layout models (flexlayout-react) + persistence.
// Two top-level panels ONLY: "nav" (workspace navigation) and the
// workspace area ("term" tab = terminals; other pages open as tabs in
// the same tabset). Both panels can be closed and reopened, unique.

import { Model } from "flexlayout-react";

const MAIN_KEY = "opennex-dock-main";
const TERM_KEY = "opennex-dock-term";

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

function mainDefault() {
  return {
    global: {
      tabEnableClose: true,
      tabSetEnableDeleteWhenEmpty: false,
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
              name: "终端",
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

function termDefault() {
  return {
    global: {
      tabEnableClose: true,
      tabSetEnableDeleteWhenEmpty: false,
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
              id: "term-1",
              name: "bash 1",
              component: "termpane",
              enableClose: true,
              enableRenderOnDemand: false,
              config: { slot: 1 },
            },
          ],
        },
      ],
    },
  };
}

export function loadMainModel(): Model {
  try {
    const raw = localStorage.getItem(MAIN_KEY);
    if (raw) return Model.fromJson(JSON.parse(raw));
  } catch {
    /* default below */
  }
  return Model.fromJson(mainDefault());
}

export function loadTermModel(): Model {
  try {
    const raw = localStorage.getItem(TERM_KEY);
    if (raw) return Model.fromJson(JSON.parse(raw));
  } catch {
    /* default below */
  }
  return Model.fromJson(termDefault());
}

export function saveModels(main: Model, term: Model) {
  localStorage.setItem(MAIN_KEY, JSON.stringify(main.toJson()));
  localStorage.setItem(TERM_KEY, JSON.stringify(term.toJson()));
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

/** Highest slot number referenced by terminal tabs (bash <n> naming). */
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
  let max = 0;
  const walk = (n: any) => {
    if (n.getType?.() === "tabset") {
      for (const t of n.getChildren()) walk(t);
    } else if (n.getType?.() === "tab") {
      const m = /term-(\d+)/.exec(n.getId() ?? "");
      if (m) max = Math.max(max, Number(m[1]));
    }
  };
  walk((term as any).getRootRow());
  return max;
}
