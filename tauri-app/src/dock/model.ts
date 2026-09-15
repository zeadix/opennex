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
          children: [],
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

/** Highest slot number referenced by terminal tabs (bash <n> naming). */
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
