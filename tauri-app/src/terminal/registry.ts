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

/** Screen position of the focused terminal's input cursor (px) — lets
 * the auto-match overlay and the Alt palette follow the caret. */
export const lastCursor = { x: 0, y: 0 };

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
