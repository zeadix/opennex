// App settings persisted in localStorage (v1 — SQLite later).
import { useEffect, useState } from "react";

export interface Settings {
  shell: string;
  fontSize: number;
}

const KEY = "opennex-settings";

export const defaultSettings: Settings = {
  shell: "",
  fontSize: 14,
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
