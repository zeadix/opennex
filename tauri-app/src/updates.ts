import { useSyncExternalStore } from "react";
import { invoke } from "./terminal/tauri";

export interface UpdateResult {
  current: string;
  latest: string;
  updateAvailable: boolean;
  canInstall: boolean;
  channel: string;
  changes: string[];
  changesEn: string[];
  currentChanges: string[];
  currentChangesEn: string[];
  unavailableReason?: string;
}

interface UpdateState {
  current: string | null;
  checking: boolean;
  result: UpdateResult | null;
  error: string | null;
}

export function releaseNotes(zh: string[], en: string[], lang: string): string[] {
  const translated = en.filter((line) => line.trim());
  return lang.startsWith("en") && translated.length ? translated : zh;
}

let state: UpdateState = { current: null, checking: false, result: null, error: null };
const listeners = new Set<() => void>();
let pending: Promise<void> | null = null;
let started = false;
function publish(patch: Partial<UpdateState>) {
  state = { ...state, ...patch };
  listeners.forEach((listener) => listener());
}

export function checkForUpdates(): Promise<void> {
  if (pending) return pending;
  publish({ checking: true, error: null });
  pending = invoke<UpdateResult>("check_update")
    .then((result) => publish({ result, current: result.current }))
    .catch((error) => publish({ error: String(error) }))
    .finally(() => {
      pending = null;
      publish({ checking: false });
    });
  return pending;
}

export function startUpdateCheck() {
  if (started) return;
  started = true;
  void invoke<{ current: string }>("get_app_info")
    .then((info) => publish({ current: info.current }))
    .catch(() => {});
  void checkForUpdates();
}

export function useUpdates() {
  return useSyncExternalStore(
    (listener) => { listeners.add(listener); return () => { listeners.delete(listener); }; },
    () => state,
  );
}
