// i18n framework: zh/en dictionaries for the chrome copy. Feature
// modules read strings via t(lang); more languages slot into LOCALES
// without call-site changes.

export type Lang = "zh" | "en";
export const LANGS: { id: Lang; label: string }[] = [
  { id: "zh", label: "中文" },
  { id: "en", label: "English" },
];

const zh = {
  nav: "工作空间",
  newWorkspace: "新建工作空间",
  deleteWorkspace: "删除工作空间",
  rename: "重命名",
  lock: "锁定",
  locked: "已锁定",
  unlock: "已解锁",
  terminal: "终端",
  ssh: "SSH",
  history: "历史",
  ai: "AI",
  settings: "设置",
  newTerminal: "新建终端",
  defaultShell: "默认 Shell",
  broadcastOn: "关闭广播输入（全部终端）",
  broadcastOff: "开启广播输入（全部终端）",
  appearance: "外观",
  theme: "主题",
  terminalFontSize: "终端字号",
  defaultShellSection: "默认 Shell",
  about: "关于",
  insertTerminal: "插入终端",
  connected: "已连接",
  connecting: "连接 PTY 中…",
  sessionEnded: "会话已结束",
};

const en: typeof zh = {
  nav: "Workspaces",
  newWorkspace: "New workspace",
  deleteWorkspace: "Delete workspace",
  rename: "Rename",
  lock: "Lock",
  locked: "Locked",
  unlock: "Unlocked",
  terminal: "Terminal",
  ssh: "SSH",
  history: "History",
  ai: "AI",
  settings: "Settings",
  newTerminal: "New terminal",
  defaultShell: "Default shell",
  broadcastOn: "Stop broadcasting input (all terminals)",
  broadcastOff: "Broadcast input to all terminals",
  appearance: "Appearance",
  theme: "Theme",
  terminalFontSize: "Terminal font size",
  defaultShellSection: "Default shell",
  about: "About",
  insertTerminal: "Insert to terminal",
  connected: "connected",
  connecting: "connecting PTY…",
  sessionEnded: "session ended",
};

const DICTS: Record<Lang, typeof zh> = { zh, en };

export function t(lang: Lang): typeof zh {
  return DICTS[lang] ?? zh;
}

const LANG_KEY = "opennex-lang";

export function loadLang(): Lang {
  return (localStorage.getItem(LANG_KEY) as Lang) ?? "zh";
}
export function saveLang(lang: Lang) {
  localStorage.setItem(LANG_KEY, lang);
}
