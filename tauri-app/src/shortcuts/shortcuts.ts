// User-configurable keyboard shortcuts. Bindings are normalized strings
// like "Ctrl+Shift+N" (Modifier+...+Key, order: Ctrl, Shift, Alt, Meta).
// Persisted in localStorage; defaults match the egui build's spirit.

export type ShortcutAction =
  | "newTerminal"
  | "closeTab"
  | "search"
  | "lockWorkspace"
  | "workspaceNext"
  | "workspacePrev"
  | "historyMenu";

export const SHORTCUT_ACTIONS: { id: ShortcutAction; labelZh: string; labelEn: string }[] = [
  { id: "newTerminal", labelZh: "新建终端", labelEn: "New terminal" },
  { id: "closeTab", labelZh: "关闭当前终端", labelEn: "Close current terminal" },
  { id: "search", labelZh: "终端内搜索", labelEn: "Search in terminal" },
  { id: "lockWorkspace", labelZh: "锁定工作空间", labelEn: "Lock workspace" },
  { id: "workspaceNext", labelZh: "下一个工作空间", labelEn: "Next workspace" },
  { id: "workspacePrev", labelZh: "上一个工作空间", labelEn: "Previous workspace" },
  { id: "historyMenu", labelZh: "呼出指令面板（历史/收藏）", labelEn: "Command palette" },
];

export const DEFAULT_SHORTCUTS: Record<ShortcutAction, string> = {
  newTerminal: "Ctrl+Shift+N",
  closeTab: "Ctrl+Shift+Q",
  search: "Ctrl+Shift+F",
  lockWorkspace: "Ctrl+Shift+L",
  workspaceNext: "Ctrl+PageDown",
  workspacePrev: "Ctrl+PageUp",
  historyMenu: "Alt",
};

const KEY = "opennex-shortcuts";

export function loadShortcuts(): Record<ShortcutAction, string> {
  try {
    return { ...DEFAULT_SHORTCUTS, ...JSON.parse(localStorage.getItem(KEY) ?? "{}") };
  } catch {
    return { ...DEFAULT_SHORTCUTS };
  }
}

export function saveShortcuts(s: Record<ShortcutAction, string>) {
  localStorage.setItem(KEY, JSON.stringify(s));
}

/** Normalize a KeyboardEvent into a binding string ("Ctrl+Shift+N"). */
export function eventToBinding(e: KeyboardEvent): string | null {
  const key = e.key;
  if (["Control", "Shift", "Alt", "Meta"].includes(key)) return null;
  if (!key || key.length === 0) return null;
  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.shiftKey) parts.push("Shift");
  if (e.altKey) parts.push("Alt");
  if (e.metaKey) parts.push("Meta");
  // Displayable key name (letters upper, named keys as-is).
  const k = key.length === 1 ? key.toUpperCase() : key;
  parts.push(k);
  return parts.join("+");
}

/** Does this event match a binding string? */
export function matchesBinding(e: KeyboardEvent, binding: string): boolean {
  const parts = binding.split("+").map((p) => p.trim());
  const key = parts[parts.length - 1];
  const wantCtrl = parts.includes("Ctrl");
  const wantShift = parts.includes("Shift");
  const wantAlt = parts.includes("Alt");
  const wantMeta = parts.includes("Meta");
  if (e.ctrlKey !== wantCtrl || e.shiftKey !== wantShift || e.altKey !== wantAlt || e.metaKey !== wantMeta) {
    return false;
  }
  const k = e.key.length === 1 ? e.key.toUpperCase() : e.key;
  return k === key;
}
