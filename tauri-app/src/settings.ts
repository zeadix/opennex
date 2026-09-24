// App settings persisted in localStorage (v1 — SQLite later).
import { useEffect, useState } from "react";

export interface Settings {
  shell: string;
  fontSize: number;
  /** Auto-match command suggestions while typing (egui: auto_match_command). */
  autoMatch: boolean;
  /** 补全面板来源："auto" 历史+系统合并；"system" 仅 PATH 系统指令；
   * "history" 仅会话历史。 */
  suggestSource: "auto" | "system" | "history";
  /** 指令面板跟随输入光标；关闭=固定位置（可拖拽、跨重启记忆）。 */
  followCursor: boolean;
  /** Command history capacity (synced to the backend via set_history_cap). */
  historyCap: number;
  /** UI font family ("" = system default). */
  uiFont: string;
  /** UI font size in px at scale 1 (13 = default); drives the UI zoom. */
  uiFontSize: number;
  /** Terminal font family ("" = built-in mono stack via --mono). */
  termFont: string;
  /** Drag-select in the terminal copies to the clipboard on mouse-up. */
  copyOnSelect: boolean;
  /** Use the applied theme's font pack instead of the global settings. */
  useThemeFont: boolean;
  /** 背景图片（data URL；"" = 未设置）。 */
  bgImageData: string;
  /** 背景图片不透明度 %（图片层自身浓度）。 */
  bgImageOpacity: number;
  /** 背景显示模式：填充 / 适应 / 平铺。 */
  bgImageFit: "cover" | "contain" | "tile";
  /** 面板不透明度 %（越低，背景图透出越多）。 */
  bgImagePanelAlpha: number;
}

const KEY = "opennex-settings";

export const defaultSettings: Settings = {
  shell: "",
  fontSize: 13,
  autoMatch: true,
  suggestSource: "auto",
  followCursor: false,
  historyCap: 500,
  uiFont: "",
  uiFontSize: 13,
  termFont: "",
  copyOnSelect: true,
  useThemeFont: false,
  bgImageData: "",
  bgImageOpacity: 40,
  bgImageFit: "cover",
  bgImagePanelAlpha: 75,
};

export function loadSettings(): Settings {
  try {
    return { ...defaultSettings, ...JSON.parse(localStorage.getItem(KEY) ?? "{}") };
  } catch {
    return { ...defaultSettings };
  }
}

export function saveSettings(s: Settings) {
  localStorage.setItem(KEY, JSON.stringify(s));
}

export function useSettings(): [Settings, (patch: Partial<Settings>) => void] {
  const [settings, setSettings] = useState<Settings>(loadSettings());
  useEffect(() => {
    saveSettings(settings);
  }, [settings]);
  const update = (patch: Partial<Settings>) =>
    setSettings((prev) => ({ ...prev, ...patch }));
  return [settings, update];
}
