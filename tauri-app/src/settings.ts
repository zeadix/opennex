// App settings persisted in localStorage (v1 — SQLite later).
import { useEffect, useState } from "react";

export interface Settings {
  shell: string;
  fontSize: number;
  /** Auto-match command suggestions while typing (egui: auto_match_command). */
  autoMatch: boolean;
  /** Command history capacity (synced to the backend via set_history_cap). */
  historyCap: number;
  /** Command popups follow the input caret; off = fixed bottom-right
   * corner, draggable, position remembered across restarts. */
  followCursor: boolean;
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
}

const KEY = "opennex-settings";

export const defaultSettings: Settings = {
  shell: "",
  fontSize: 14,
  autoMatch: true,
  historyCap: 500,
  followCursor: false,
  uiFont: "",
  uiFontSize: 13,
  termFont: "",
  copyOnSelect: true,
  useThemeFont: false,
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
