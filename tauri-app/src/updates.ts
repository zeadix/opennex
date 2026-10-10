import { useSyncExternalStore } from "react";
import { invoke } from "./terminal/tauri";
import { check as pluginCheck, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

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

/** 应用内一键更新（tauri-plugin-updater）通道的状态。 */
export interface TauriUpdateState {
  available: boolean;
  version: string;
  notes: string;
  downloading: boolean;
  /** 0-100；-1 = 总大小未知，进度不确定 */
  progress: number;
  installed: boolean;
  error: string | null;
  /** 安装方式不支持应用内更新（如 Windows 便携版：无安装器注册，
   *  更新器找不到匹配平台键）——UI 显示下载引导而非原始错误。 */
  unsupported: boolean;
}

interface UpdateState {
  current: string | null;
  checking: boolean;
  result: UpdateResult | null;
  error: string | null;
  tauri: TauriUpdateState;
}

export function releaseNotes(zh: string[], en: string[], lang: string): string[] {
  const translated = en.filter((line) => line.trim());
  return lang.startsWith("en") && translated.length ? translated : zh;
}

const TAURI_IDLE: TauriUpdateState = {
  available: false, version: "", notes: "",
  downloading: false, progress: 0, installed: false, error: null,
  unsupported: false,
};

let state: UpdateState = { current: null, checking: false, result: null, error: null, tauri: TAURI_IDLE };
const listeners = new Set<() => void>();
let pending: Promise<void> | null = null;
let started = false;
/** plugin-updater 的 Update 实例必须持有才能 downloadAndInstall */
let pluginUpdate: Update | null = null;

function publish(patch: Partial<UpdateState>) {
  state = { ...state, ...patch };
  listeners.forEach((listener) => listener());
}

function publishTauri(patch: Partial<TauriUpdateState>) {
  publish({ tauri: { ...state.tauri, ...patch } });
}

/** 应用内更新通道：查 tauri 更新源（签名校验），结果进 state.tauri。
 *  更新源暂不可用（如首个未带 tauri 更新源的版本）不报错，回退展示旧通道结果。 */
async function tauriCheck(): Promise<void> {
  try {
    pluginUpdate = await pluginCheck();
    publishTauri({
      available: !!pluginUpdate,
      version: pluginUpdate?.version ?? "",
      notes: pluginUpdate?.body ?? "",
      error: null,
      installed: false,
    });
  } catch (e) {
    pluginUpdate = null;
    // 便携版等无安装器注册的场景:更新器报 "none of the fallback
    // platforms ... were found"。这是环境限制而非故障——UI 只给
    // 下载引导,不抛原始错误。
    const msg = String(e);
    const unsupported =
      msg.includes("none of the fallback platforms") ||
      msg.includes("were found in the response") ||
      msg.includes("TargetsNotFound");
    publishTauri({
      available: false,
      error: unsupported ? null : String(e),
      unsupported,
    });
  }
}

/** 一键更新：下载（带进度）→ 安装 → 自动重启。失败进 state.tauri.error。 */
export async function installTauriUpdate(): Promise<void> {
  if (!pluginUpdate) return;
  publishTauri({ downloading: true, progress: 0, error: null, installed: false });
  try {
    let total = 0;
    let received = 0;
    await pluginUpdate.downloadAndInstall((event) => {
      if (event.event === "Started") {
        total = event.data.contentLength ?? 0;
        received = 0;
        publishTauri({ progress: total > 0 ? 0 : -1 });
      } else if (event.event === "Progress") {
        received += event.data.chunkLength;
        publishTauri({
          progress: total > 0 ? Math.min(99, Math.round((received / total) * 100)) : -1,
        });
      } else if (event.event === "Finished") {
        publishTauri({ progress: 100 });
      }
    });
    publishTauri({ downloading: false, installed: true });
    await relaunch();
  } catch (e) {
    publishTauri({ downloading: false, error: String(e) });
  }
}

export function checkForUpdates(): Promise<void> {
  if (pending) return pending;
  publish({ checking: true, error: null, tauri: { ...TAURI_IDLE } });
  pending = (async () => {
    // 旧通道（egui 更新源）负责版本说明展示；tauri 通道负责一键更新。
    const legacy = invoke<UpdateResult>("check_update")
      .then((result) => publish({ result, current: result.current }))
      .catch((error) => publish({ error: String(error) }));
    await Promise.all([legacy, tauriCheck()]);
  })()
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
