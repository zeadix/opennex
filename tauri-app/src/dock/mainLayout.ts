import { Model } from "flexlayout-react";
import { mainModelFrom } from "./model";

const KEY = "opennex-main-layout";

export function loadMainModel(): Model {
  try {
    const saved = localStorage.getItem(KEY);
    if (saved) return mainModelFrom(JSON.parse(saved));
  } catch {
    /* Fall back to the legacy layout. */
  }
  try {
    const legacy = JSON.parse(localStorage.getItem("opennex-workspaces") ?? "[]");
    if (Array.isArray(legacy)) {
      for (const workspace of legacy) {
        if (!workspace?.mainJson) continue;
        try {
          Model.fromJson(JSON.parse(JSON.stringify(workspace.mainJson)));
          return mainModelFrom(workspace.mainJson);
        } catch {
          /* Try the next legacy workspace. */
        }
      }
    }
  } catch {
    /* Default for a new installation or unreadable legacy data. */
  }
  return mainModelFrom(undefined);
}

export function persistMainModel(model: Model): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(model.toJson()));
  } catch {
    /* Keep the live layout usable when storage is unavailable. */
  }
}
