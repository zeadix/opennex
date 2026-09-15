// Dock layout models (flexlayout-react) + persistence.
// Two top-level panels ONLY: "nav" (workspace navigation) and the
// workspace area ("term" tab = terminals; other pages open as tabs in
// the same tabset). Both panels can be closed and reopened, unique.

import { Actions, Model } from "flexlayout-react";

// v2 keys: layouts saved by flexlayout 0.11 (the brief first attempt)
// are INCOMPATIBLE with 0.8.5 — among other traps 0.11 persists
// `selected: -1` tabsets which render as a fully black empty area.
// Bumping the key quarantines all legacy blobs.
const MAIN_KEY = "opennex-dock-v2-main";
const TERM_KEY = "opennex-dock-v2-term";

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

// LAYOUT PERSISTENCE IS DISABLED for now: layouts saved during the
// flexlayout migration rounds are unreliable (0.11-era blobs, selected:-1
// states, missing seeded terminals). Every launch starts from the clean
// default below so the UI is always correct; re-enable persistence once
// the dock model set is stable.
export function loadMainModel(): Model {
  // Also purge any legacy blobs saved under the old keys.
  try {
    for (const k of ["opennex-dock-main", "opennex-dock-term", MAIN_KEY, TERM_KEY]) {
      localStorage.removeItem(k);
    }
  } catch {
    /* private mode — ignore */
  }
  return Model.fromJson(mainDefault());
}

export function loadTermModel(): Model {
  try {
    localStorage.removeItem(TERM_KEY);
  } catch {
    /* ignore */
  }
  return Model.fromJson(termDefault());
}

export function saveModels(_main: Model, _term: Model) {
  // Persistence disabled alongside load (see loadMainModel) — layouts
  // always start from the clean default until the dock model stabilizes.
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
