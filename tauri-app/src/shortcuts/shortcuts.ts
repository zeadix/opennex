// User-configurable keyboard shortcuts. Bindings are normalized strings
// like "Ctrl+Shift+N" (Modifier+...+Key, order: Ctrl, Shift, Alt, Meta).
// Persisted in localStorage; defaults match the egui build's spirit.

export type ShortcutAction =
  | "newTerminal"
  | "nextTerminal"
  | "nextPanel"
  | "closeTerminal"
  | "search"
  | "terminalInterrupt"
  | "terminalCopy"
  | "terminalPaste"
  | "lockWorkspace"
  | "workspaceNext"
  | "workspacePrev"
  | "workspaceUp"
  | "workspaceDown"
  | "toggleSidebar"
  | "zoomIn"
  | "zoomOut"
  | "saveLayout"
  | "historyMenu";

export const SHORTCUT_ACTIONS: { id: ShortcutAction; labelZh: string; labelEn: string }[] = [
  { id: "newTerminal", labelZh: "新建终端", labelEn: "New terminal" },
  { id: "nextTerminal", labelZh: "下一个终端标签", labelEn: "Next terminal tab" },
  { id: "nextPanel", labelZh: "切换主面板标签", labelEn: "Cycle main panels" },
  { id: "closeTerminal", labelZh: "关闭当前终端", labelEn: "Close current terminal" },
  { id: "search", labelZh: "终端内搜索", labelEn: "Search in terminal" },
  { id: "terminalInterrupt", labelZh: "终端中断 (^C)", labelEn: "Terminal interrupt (^C)" },
  { id: "terminalCopy", labelZh: "复制终端选中文本", labelEn: "Copy terminal selection" },
  { id: "terminalPaste", labelZh: "粘贴到终端", labelEn: "Paste into terminal" },
  { id: "saveLayout", labelZh: "保存布局", labelEn: "Save layout" },
  { id: "lockWorkspace", labelZh: "锁定工作空间", labelEn: "Lock workspace" },
  { id: "workspaceNext", labelZh: "下一个工作空间", labelEn: "Next workspace" },
  { id: "workspacePrev", labelZh: "上一个工作空间", labelEn: "Previous workspace" },
  { id: "workspaceUp", labelZh: "上移工作空间", labelEn: "Workspace up" },
  { id: "workspaceDown", labelZh: "下移工作空间", labelEn: "Workspace down" },
  { id: "toggleSidebar", labelZh: "显示/隐藏侧栏", labelEn: "Toggle sidebar" },
  { id: "zoomIn", labelZh: "界面放大", labelEn: "Zoom in" },
  { id: "zoomOut", labelZh: "界面缩小", labelEn: "Zoom out" },
  { id: "historyMenu", labelZh: "呼出指令面板（历史/收藏）", labelEn: "Command palette" },
];

// Defaults mirror the egui build's keymap.
export const DEFAULT_SHORTCUTS: Record<ShortcutAction, string> = {
  newTerminal: "Ctrl+N",
  nextTerminal: "Ctrl+Tab",
  nextPanel: "Ctrl+Q",
  closeTerminal: "Ctrl+E",
  search: "Ctrl+F",
  terminalInterrupt: "Ctrl+Shift+C",
  terminalCopy: "Ctrl+C",
  terminalPaste: "Ctrl+V",
  saveLayout: "Ctrl+S",
  lockWorkspace: "Ctrl+L",
  workspaceNext: "Ctrl+W",
  workspacePrev: "Ctrl+Shift+W",
  workspaceUp: "Ctrl+ArrowUp",
  workspaceDown: "Ctrl+ArrowDown",
  toggleSidebar: "F1",
  zoomIn: "Ctrl+=",
  zoomOut: "Ctrl+-",
  historyMenu: "Alt",
};

const KEY = "opennex-shortcuts-v2";

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
