// AI 助手的定时任务：向指定终端定时发送命令并回车。
// 两种任务：
//   repeat — 每 everySec 秒重复执行（AI 的 addSchedule 指令）；
//   once   — 创建后 runAtSec 秒执行一次，随即自动移除（runAt 指令），
//            用于「5 秒后启动 top、再过 3 秒停止」这类时序编排。
//
// 任务是会话级状态：命令由本进程的调度器发进终端，应用关闭后任务
// 必然失效（终端 PTY 也随之关闭）。因此任务不跨启动存活——启动时
// 清空上一次会话遗留的任务（自动撤销），关闭时同样清空，避免第二次
// 启动看到已失效的任务条、甚至被过期任务立刻轰炸。

import { focusedSlot, sendTo } from "./terminal/registry";

export interface AiTask {
  id: string;
  /** 展示名，如「每 20s · ls」或「5s 后 · top」。 */
  label: string;
  command: string;
  /** 目标终端 slot；null = 当前聚焦终端。 */
  slot: number | null;
  createdAt: number;
  enabled: boolean;
  lastRun?: number;
  /** "repeat" 周期执行；"once" 延时单次执行（执行后自动移除）。 */
  kind: "repeat" | "once";
  /** repeat 模式：间隔秒数。 */
  everySec?: number;
  /** once 模式：自创建起延迟的秒数。 */
  runAtSec?: number;
}

const KEY = "opennex-ai-tasks";
let tasks: AiTask[] = [];
const listeners = new Set<() => void>();

function persist() {
  localStorage.setItem(KEY, JSON.stringify(tasks));
}

/** 撤销全部任务并清空持久化（应用启动 / 关闭时调用）。 */
function revokeAll() {
  tasks = [];
  persist();
}

// 会话级语义：启动即撤销上一个会话遗留的任务；关闭前再清一次。
revokeAll();
window.addEventListener("beforeunload", revokeAll);

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
  command: string;
  slot: number | null;
  kind: "repeat" | "once";
  everySec?: number;
  runAtSec?: number;
  label?: string;
}): AiTask {
  const label =
    t.label ??
    (t.kind === "once"
      ? `${t.runAtSec ?? 0}s 后 · ${t.command}`
      : `每 ${t.everySec ?? 0}s · ${t.command}`);
  const task: AiTask = {
    id: crypto.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(36).slice(2)}`,
    label,
    command: t.command,
    slot: t.slot,
    createdAt: Date.now(),
    enabled: true,
    kind: t.kind,
    everySec: t.kind === "repeat" ? Math.max(1, Math.round(t.everySec ?? 10)) : undefined,
    runAtSec: t.kind === "once" ? Math.max(0, Math.round(t.runAtSec ?? 0)) : undefined,
  };
  // 追加而非置顶：多个延时指令按时间顺序展示，时序一目了然。
  tasks = [...tasks, task];
  persist();
  publish();
  return task;
}

export function removeAiTask(id: string) {
  tasks = tasks.filter((t) => t.id !== id);
  persist();
  publish();
}

/** 停止并撤销全部任务。 */
export function stopAllTasks() {
  revokeAll();
  publish();
}

/** 到期判断：repeat 按距上次执行；once 按创建时刻 + 延迟。 */
function isDue(t: AiTask, now: number): boolean {
  if (!t.enabled) return false;
  return t.kind === "once"
    ? now >= t.createdAt + (t.runAtSec ?? 0) * 1000
    : now - (t.lastRun ?? t.createdAt) >= (t.everySec ?? 1) * 1000;
}

/** 到期任务执行：向目标终端发送 command + 回车。 */
function fire(t: AiTask) {
  const slot = t.slot ?? focusedSlot.value;
  sendTo(slot, new TextEncoder().encode(t.command + "\r"));
  t.lastRun = Date.now();
}

let ticker: number | null = null;

/** 每秒检查到期任务；once 任务执行一次即自动移除。在 App 挂载时启动一次。 */
export function startAiTaskTicker() {
  if (ticker !== null) return;
  ticker = window.setInterval(() => {
    const now = Date.now();
    let fired = false;
    let changed = false;
    for (const t of [...tasks]) {
      if (!isDue(t, now)) continue;
      fire(t);
      fired = true;
      if (t.kind === "once") {
        tasks = tasks.filter((x) => x.id !== t.id);
        changed = true;
      }
    }
    if (changed) persist();
    if (fired || changed) publish();
  }, 1000);
}
