// App settings persisted in localStorage (v1 — SQLite later).
import { useEffect, useState } from "react";

export interface Settings {
  shell: string;
  fontSize: number;
  /** Auto-match command suggestions while typing (egui: auto_match_command). */
  autoMatch: boolean;
  /** Command history capacity (synced to the backend via set_history_cap). */
  historyCap: number;
}

const KEY = "opennex-settings";

export const defaultSettings: Settings = {
  shell: "",
  fontSize: 14,
  autoMatch: true,
  historyCap: 500,
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
