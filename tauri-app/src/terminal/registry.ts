// Process-wide registry of live terminal sockets. Enables cross-cutting
// features (broadcast input, AI insert-to-terminal) without prop
// drilling through the dock layout.

export const sockets = new Map<number, WebSocket>();

/** Slots subscribed to broadcast input. */
export const broadcastGroup = new Set<number>();
export const broadcastEnabled = { value: false };

/** The pane the user last clicked/focused — target for "insert". */
export const focusedSlot = { value: 0 };

/** Session slot -> last activity unix ms, mirrored from App's poll so
 * components outside the dock models (busy dots) can read it freely. */
export const activityStore: { map: Record<string, number> } = { map: {} };

/** Per-pane cursor-position refreshers — the Alt palette runs them all
 * right before mounting so it positions against the LIVE caret, not a
 * coordinate cached from the last keystroke/render. */
export const cursorRefreshers = new Set<() => void>();

/** 每个终端自己的光标位置：pane 本地 px（与终端同一缩放上下文，绝无
 * 跨空间换算误差）+ 宿主元素可视矩形 + 缩放系数。自动补全面板直接在
 * pane 内渲染（天然跟随）；历史面板用宿主矩形把本地坐标映射回视口。
 * 各终端写各自的桶，互不覆盖 —— 多终端下不可能锚错终端。 */
export const cursorBySlot = new Map<
  number,
  {
    x: number;
    y: number;
    h: number;
    host: { left: number; top: number; w: number; h: number };
    zoom: number;
  }
>();

// DEV 调试出口：浏览器控制台可直接检查光标跟随的实际状态（生产构建剔除）。
const isDev = Boolean((import.meta as any).env?.DEV) || location.hostname === "localhost";
if (isDev) {
  (window as any).__opennexCursorDebug = { cursorBySlot, focusedSlot };
}

export function registerSocket(slot: number, ws: WebSocket) {
  sockets.set(slot, ws);
}

export function unregisterSocket(slot: number) {
  sockets.delete(slot);
}

export function sendTo(slot: number, data: Uint8Array): boolean {
  const ws = sockets.get(slot);
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(data);
    return true;
  }
  return false;
}

/** Broadcast input to every OTHER session in the broadcast group. */
export function broadcastInput(fromSlot: number, data: Uint8Array) {
  if (!broadcastEnabled.value || !broadcastGroup.has(fromSlot)) return false;
  let sent = false;
  for (const slot of broadcastGroup) {
    if (slot === fromSlot) continue;
    if (sendTo(slot, data)) sent = true;
  }
  return sent;
}
