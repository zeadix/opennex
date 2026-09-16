// i18n: zh is the base dictionary; other languages are PARTIAL overlays
// merged over it (missing keys fall back to Chinese). Adding a language
// = adding one entry to PARTIALS — no call-site changes.

export type Lang =
  | "zh" | "zh-TW" | "en" | "de" | "fr" | "ja" | "it" | "ko" | "hi";

export const LANGS: { id: Lang; label: string }[] = [
  { id: "zh", label: "中文" },
  { id: "zh-TW", label: "繁體中文" },
  { id: "en", label: "English" },
  { id: "de", label: "Deutsch" },
  { id: "fr", label: "Français" },
  { id: "ja", label: "日本語" },
  { id: "it", label: "Italiano" },
  { id: "ko", label: "한국어" },
  { id: "hi", label: "हिन्दी" },
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
  remote: "远程",
  update: "更新",
  favorites: "收藏",
  monitor: "监控",
  newTerminal: "新建终端",
  newTerminalPick: "新建终端（选择 Shell）",
  defaultShell: "默认 Shell",
  broadcastOn: "关闭广播输入（全部终端）",
  broadcastOff: "开启广播输入（全部终端）",
  appearance: "外观",
  theme: "主题",
  terminalFontSize: "终端字号",
  defaultShellSection: "默认 Shell",
  shortcuts: "快捷键",
  about: "关于",
  insertToTerminal: "插入终端",
  copy: "复制",
  paste: "粘贴",
  template: "模板",
  saveAsTemplate: "保存为模板",
  tplNamePrompt: "模板名称",
  tplHint: "右键工作空间可保存为模板",
  busy: "繁忙",
  idle: "空闲",
  menuWorkspace: "工作空间",
  menuView: "视图",
  menuRemote: "远程控制",
  menuHelp: "帮助",
  saveLayout: "保存布局",
  loadLayout: "加载布局",
  saveLayoutAs: "布局另存为…",
  navPanel: "导航栏",
  workArea: "工作区",
  lan: "局域网",
  wan: "广域网",
  tutorial: "教程",
  layoutSaved: "布局已保存",
  layoutLoaded: "布局已加载",
  tplCreated: "已从模板创建工作空间",
  tplSaved: "已保存为模板",
  ok: "确定",
  cancel: "取消",
  autoMatch: "自动匹配指令",
  wsConnected: "已连接",
  wsConnecting: "连接 PTY 中…",
  sessionEnded: "会话已结束",
  phoneRemote: "手机远程控制",
  phoneRemoteHint: "手机与电脑处于同一局域网时，扫码或访问下方地址即可在手机上查看和操作终端。",
  scanHint: "局域网 IP · 端口 · 同一 Wi-Fi 下可用",
  remoteNote1: "· 远程页面支持查看终端输出、发送命令、切换会话",
  remoteNote2: "· 会话结束或应用退出后远程访问自动失效",
  updateCheck: "检查更新",
  currentVersion: "当前版本",
  latestVersion: "最新版本",
  hasUpdate: "有新版本",
  upToDate: "已是最新",
  whatsNew: "更新内容",
  recheck: "重新检查",
  lockSection: "锁定",
  lockAllHint: "为全部工作空间设置锁定密码",
  set: "设置",
  clear: "清除",
  lockSet: "已为全部工作空间设置锁定密码",
  lockCleared: "已清除全部工作空间的锁定密码",
  pwdShort: "密码至少 4 位",
  sshHosts: "SSH 主机",
  noHosts: "还没有保存的主机",
  name: "名称（可选）",
  hostAddr: "主机地址",
  user: "用户名",
  port: "端口",
  save: "保存",
  cmdHistory: "指令历史",
  historyEmpty: "在终端里执行的命令会出现在这里",
  favEmpty: "点击「添加」保存常用命令，点击条目插入到最近聚焦的终端",
  agentTitle: "Agent · 命令规划执行",
  stableTag: "稳定版",
  searchPh: "搜索…",
};

// Partial overlays: core chrome words per language.
const zhTW: Partial<typeof zh> = {
  nav: "工作空間", newWorkspace: "新建工作空間", deleteWorkspace: "刪除工作空間",
  rename: "重新命名", lock: "鎖定", locked: "已鎖定", terminal: "終端機",
  history: "歷史", settings: "設定", remote: "遠端", update: "更新",
  favorites: "收藏指令", monitor: "監控", newTerminal: "新增終端機",
  defaultShell: "預設 Shell", appearance: "外觀", theme: "主題", about: "關於",
  copy: "複製", paste: "貼上",
  template: "範本", saveAsTemplate: "儲存為範本", tplNamePrompt: "範本名稱",
  tplHint: "右鍵工作空間可儲存為範本", busy: "繁忙", idle: "閒置",
  menuWorkspace: "工作空間", menuView: "檢視", menuRemote: "遠端控制", menuHelp: "說明",
  saveLayout: "儲存佈局", loadLayout: "載入佈局", saveLayoutAs: "佈局另存為…",
  navPanel: "導覽列", workArea: "工作區", lan: "區域網路", wan: "廣域網路",
  tutorial: "教學", layoutSaved: "佈局已儲存", layoutLoaded: "佈局已載入",
  tplCreated: "已從範本建立工作空間", tplSaved: "已儲存為範本",
  ok: "確定", cancel: "取消",
  autoMatch: "自動匹配指令",
};

const en: Partial<typeof zh> = {
  nav: "Workspaces", newWorkspace: "New workspace", deleteWorkspace: "Delete workspace",
  rename: "Rename", lock: "Lock", locked: "Locked", terminal: "Terminal",
  ssh: "SSH", history: "History", ai: "AI", settings: "Settings",
  remote: "Remote", update: "Update", favorites: "Favorites", monitor: "Monitor",
  newTerminal: "New terminal", newTerminalPick: "New terminal (pick shell)",
  defaultShell: "Default shell",
  broadcastOn: "Stop broadcasting input (all terminals)",
  broadcastOff: "Broadcast input to all terminals",
  appearance: "Appearance", theme: "Theme", terminalFontSize: "Terminal font size",
  defaultShellSection: "Default shell", shortcuts: "Shortcuts", about: "About",
  insertToTerminal: "Insert to terminal", wsConnected: "connected",
  copy: "Copy", paste: "Paste",
  template: "Templates", saveAsTemplate: "Save as template", tplNamePrompt: "Template name",
  tplHint: "Right-click a workspace to save it as a template",
  busy: "Busy", idle: "Idle",
  menuWorkspace: "Workspace", menuView: "View", menuRemote: "Remote", menuHelp: "Help",
  saveLayout: "Save layout", loadLayout: "Load layout", saveLayoutAs: "Save layout as…",
  navPanel: "Navigation", workArea: "Work area", lan: "LAN", wan: "WAN",
  tutorial: "Tutorial",
  layoutSaved: "Layout saved", layoutLoaded: "Layout loaded",
  tplCreated: "Workspace created from template", tplSaved: "Saved as template",
  ok: "OK", cancel: "Cancel",
  autoMatch: "Command auto-match",
  wsConnecting: "connecting PTY…", sessionEnded: "session ended",
  phoneRemote: "Phone remote control",
  phoneRemoteHint: "On the same Wi-Fi, scan the code or open the address below to view and drive your terminals from the phone.",
  scanHint: "LAN IP · port · same Wi-Fi required",
  updateCheck: "Check for updates", currentVersion: "Current version",
  latestVersion: "Latest version", hasUpdate: "Update available",
  upToDate: "Up to date", whatsNew: "What's new", recheck: "Re-check",
  lockSection: "Lock", lockAllHint: "Set a lock password for all workspaces",
  set: "Set", clear: "Clear",
  pwdShort: "Password must be at least 4 characters",
  sshHosts: "SSH hosts", noHosts: "No saved hosts yet",
  name: "Name (optional)", hostAddr: "Host address", user: "Username",
  port: "Port", save: "Save", cmdHistory: "Command history",
  historyEmpty: "Commands you run in terminals appear here",
  favEmpty: "Save frequently-used commands, click an entry to insert into the focused terminal",
  agentTitle: "Agent · plan & execute", stableTag: "STABLE", searchPh: "Search…",
};

const de: Partial<typeof zh> = {
  nav: "Arbeitsbereiche", newWorkspace: "Neuer Arbeitsbereich",
  deleteWorkspace: "Arbeitsbereich löschen", rename: "Umbenennen",
  lock: "Sperren", locked: "Gesperrt", terminal: "Terminal",
  history: "Verlauf", settings: "Einstellungen", remote: "Fernzugriff",
  update: "Update", favorites: "Favoriten", monitor: "Monitor",
  newTerminal: "Neues Terminal", defaultShell: "Standard-Shell",
  appearance: "Erscheinungsbild", theme: "Design", about: "Über",
};

const fr: Partial<typeof zh> = {
  nav: "Espaces de travail", newWorkspace: "Nouvel espace",
  deleteWorkspace: "Supprimer l'espace", rename: "Renommer",
  lock: "Verrouiller", locked: "Verrouillé", terminal: "Terminal",
  history: "Historique", settings: "Paramètres", remote: "À distance",
  update: "Mise à jour", favorites: "Favoris", monitor: "Moniteur",
  newTerminal: "Nouveau terminal", defaultShell: "Shell par défaut",
  appearance: "Apparence", theme: "Thème", about: "À propos",
};

const ja: Partial<typeof zh> = {
  nav: "ワークスペース", newWorkspace: "新規ワークスペース",
  deleteWorkspace: "ワークスペースを削除", rename: "名前を変更",
  lock: "ロック", locked: "ロック中", terminal: "ターミナル",
  history: "履歴", settings: "設定", remote: "リモート",
  update: "アップデート", favorites: "お気に入り", monitor: "モニター",
  newTerminal: "新しいターミナル", defaultShell: "既定のシェル",
  appearance: "外観", theme: "テーマ", about: "情報",
};

const it: Partial<typeof zh> = {
  nav: "Aree di lavoro", newWorkspace: "Nuova area",
  deleteWorkspace: "Elimina area", rename: "Rinomina",
  lock: "Blocca", locked: "Bloccato", terminal: "Terminale",
  history: "Cronologia", settings: "Impostazioni", remote: "Remoto",
  update: "Aggiornamento", favorites: "Preferiti", monitor: "Monitor",
  newTerminal: "Nuovo terminale", defaultShell: "Shell predefinita",
  appearance: "Aspetto", theme: "Tema", about: "Informazioni",
};

const ko: Partial<typeof zh> = {
  nav: "작업 공간", newWorkspace: "새 작업 공간",
  deleteWorkspace: "작업 공간 삭제", rename: "이름 변경",
  lock: "잠금", locked: "잠김", terminal: "터미널",
  history: "기록", settings: "설정", remote: "원격",
  update: "업데이트", favorites: "즐겨찾기", monitor: "모니터",
  newTerminal: "새 터미널", defaultShell: "기본 셸",
  appearance: "모양", theme: "테마", about: "정보",
};

const hi: Partial<typeof zh> = {
  nav: "कार्यक्षेत्र", newWorkspace: "नया कार्यक्षेत्र",
  deleteWorkspace: "कार्यक्षेत्र हटाएँ", rename: "नाम बदलें",
  lock: "लॉक", locked: "लॉक्ड", terminal: "टर्मिनल",
  history: "इतिहास", settings: "सेटिंग्स", remote: "रिमोट",
  update: "अपडेट", favorites: "पसंदीदा", monitor: "मॉनिटर",
  newTerminal: "नया टर्मिनल", defaultShell: "डिफ़ॉल्ट शेल",
  appearance: "रूप", theme: "थीम", about: "परिचय",
};

const PARTIALS: Partial<Record<Lang, Partial<typeof zh>>> = {
  "zh-TW": zhTW, en, de, fr, ja, it, ko, hi,
};

export function t(lang: Lang): typeof zh {
  return { ...zh, ...(PARTIALS[lang] ?? {}) };
}

const LANG_KEY = "opennex-lang";

export function loadLang(): Lang {
  const v = localStorage.getItem(LANG_KEY) as Lang | null;
  return v && LANGS.some((l) => l.id === v) ? v : "zh";
}

export function saveLang(lang: Lang) {
  localStorage.setItem(LANG_KEY, lang);
}
