// AI 助手的定时任务：向指定终端定时发送命令并回车。
// 任务列表持久化在 localStorage；调度器每秒检查到期任务。
// AI 通过 @@ACTION 指令创建/停止任务（见 AiPage）。

import { focusedSlot, sendTo } from "./terminal/registry";

export interface AiTask {
  id: string;
  /** 展示名，如「每 20s · ls」。 */
  label: string;
  command: string;
  everySec: number;
  /** 目标终端 slot；null = 当前聚焦终端。 */
  slot: number | null;
  createdAt: number;
  enabled: boolean;
  lastRun?: number;
}

const KEY = "opennex-ai-tasks";
let tasks: AiTask[] = load();
const listeners = new Set<() => void>();

function load(): AiTask[] {
  try {
    const raw = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    return Array.isArray(raw)
      ? raw.map((t: any) => ({
          id: String(t.id ?? crypto.randomUUID?.() ?? String(Date.now())),
          label: String(t.label ?? t.command ?? ""),
          command: String(t.command ?? ""),
          everySec: Number(t.everySec) > 0 ? Number(t.everySec) : 10,
          slot: t.slot ?? null,
          createdAt: Number(t.createdAt ?? Date.now()),
          enabled: t.enabled !== false,
          lastRun: Number(t.lastRun ?? 0) || undefined,
        }))
      : [];
  } catch {
    return [];
  }
}

function persist() {
  localStorage.setItem(KEY, JSON.stringify(tasks));
}

function publish() {
  listeners.forEach((l) => l());
}

export function subscribeAiTasks(fn: () => void): () => void {
  listeners.add(fn);
  return () => listeners.delete(fn);
}

export function getAiTasks(): AiTask[] {
  return tasks;
}

export function addAiTask(t: {
  label: string;
  command: string;
  everySec: number;
  slot: number | null;
}): AiTask {
  const task: AiTask = {
    id: crypto.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(36).slice(2)}`,
    label: t.label,
    command: t.command,
    everySec: Math.max(1, Math.round(t.everySec)),
    slot: t.slot,
    createdAt: Date.now(),
    enabled: true,
  };
  tasks = [task, ...tasks];
  persist();
  publish();
  return task;
}

export function removeAiTask(id: string) {
  tasks = tasks.filter((t) => t.id !== id);
  persist();
  publish();
}

export function setTaskEnabled(id: string, enabled: boolean) {
  tasks = tasks.map((t) => (t.id === id ? { ...t, enabled } : t));
  persist();
  publish();
}

/** 到期任务执行：向目标终端发送 command + 回车。 */
function fire(t: AiTask) {
  const slot = t.slot ?? focusedSlot.value;
  sendTo(slot, new TextEncoder().encode(t.command + "\r"));
  t.lastRun = Date.now();
}

let ticker: number | null = null;

/** 每秒检查到期任务；在 App 挂载时启动一次。 */
export function startAiTaskTicker() {
  if (ticker !== null) return;
  ticker = window.setInterval(() => {
    const now = Date.now();
    let fired = false;
    for (const t of tasks) {
      if (!t.enabled) continue;
      if (now - (t.lastRun ?? 0) >= t.everySec * 1000) {
        fire(t);
        fired = true;
      }
    }
    if (fired) publish();
  }, 1000);
}
