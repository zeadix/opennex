// Workspaces and templates own only the terminal dock and terminal paths.
// The outer view-panel layout is persisted independently in dock/mainLayout.ts.

export interface Workspace {
  id: number;
  name: string;
  locked: boolean;
  lockHash?: string;
  termJson?: any;
  /** Per-terminal working directory memory: slot -> cwd (captured on
   * switch/save, restored on open/template copy). */
  cwdMap?: Record<string, string>;
}

export interface WsTemplate {
  id: string;
  name: string;
  termJson: any;
  cwdMap?: Record<string, string>;
  createdAt: number;
}

const KEY = "opennex-workspaces";
const TPL_KEY = "opennex-templates";
let seed = 1;

export function nextId(): number {
  return seed++;
}

export function loadWorkspaces(): Workspace[] {
  try {
    const raw = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    if (Array.isArray(raw) && raw.length > 0) {
      const list = raw.map((w: any) => ({
        id: nextId(),
        name: String(w.name ?? "工作空间"),
        locked: !!w.locked,
        lockHash: w.lockHash,
        termJson: w.termJson ?? undefined,
        cwdMap: w.cwdMap ?? undefined,
      }));
      // Keep the seed clear of restored ids.
      seed = Math.max(seed, ...list.map((w) => w.id + 1));
      return list;
    }
  } catch {
    /* default below */
  }
  return [{ id: nextId(), name: "默认工作空间", locked: false }];
}

export function persistWorkspaces(list: Workspace[]) {
  localStorage.setItem(KEY, JSON.stringify(list));
}

export function makeWorkspace(name: string): Workspace {
  return { id: nextId(), name, locked: false };
}

// ---- templates ---------------------------------------------------------

export function loadTemplates(): WsTemplate[] {
  try {
    const raw = JSON.parse(localStorage.getItem(TPL_KEY) ?? "[]");
    if (Array.isArray(raw)) {
      return raw.map((t: any) => ({
        id: String(t.id ?? crypto.randomUUID?.() ?? `${Date.now()}`),
        name: String(t.name ?? "模板"),
        termJson: t.termJson ?? undefined,
        cwdMap: t.cwdMap ?? undefined,
        createdAt: Number(t.createdAt ?? Date.now()),
      }));
    }
  } catch {
    /* fallthrough */
  }
  return [];
}

export function persistTemplates(list: WsTemplate[]) {
  localStorage.setItem(TPL_KEY, JSON.stringify(list));
}

// ---- terminal-slot helpers (raw flexlayout JSON) -----------------------

/** All terminal slot numbers referenced by term-N tab ids in a layout JSON. */
export function jsonTermSlots(json: any): number[] {
  const out: number[] = [];
  const walk = (n: any) => {
    if (!n || typeof n !== "object") return;
    if (Array.isArray(n)) {
      n.forEach(walk);
      return;
    }
    const m = /^term-(\d+)$/.exec(String(n.id ?? ""));
    if (m) out.push(Number(m[1]));
    if (n.children) walk(n.children);
    if (n.layout) walk(n.layout);
  };
  walk(json);
  return out;
}

export function maxJsonSlot(json: any): number {
  return jsonTermSlots(json).reduce((a, b) => Math.max(a, b), 0);
}

export const LOCK_SALT = "opennex-ws-lock-v1";

const SHA_K = [
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/** Sync SHA-256 (hex). crypto.subtle is unavailable in non-secure
 * contexts — Tauri's WebKitGTK custom protocol is not treated as secure
 * on some Linux stacks, which silently broke lock/unlock. Pure JS keeps
 * the exact same digests as subtle.digest("SHA-256") without that
 * dependency. */
export function sha256(text: string): string {
  const msg = new TextEncoder().encode(text);
  const len = msg.length;
  const total = ((len + 8) >> 6 << 6) + 64;
  const buf = new Uint8Array(total);
  buf.set(msg);
  buf[len] = 0x80;
  const dv = new DataView(buf.buffer);
  dv.setUint32(total - 4, len * 8);
  const H = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
  const rotr = (x: number, n: number) => (x >>> n) | (x << (32 - n));
  for (let off = 0; off < total; off += 64) {
    const w = new Array<number>(64);
    for (let i = 0; i < 16; i++) w[i] = dv.getUint32(off + i * 4);
    for (let i = 16; i < 64; i++) {
      const s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >>> 3);
      const s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >>> 10);
      w[i] = (w[i - 16] + s0 + w[i - 7] + s1) | 0;
    }
    let a = H[0], b = H[1], c = H[2], d = H[3], e = H[4], f = H[5], g = H[6], hh = H[7];
    for (let i = 0; i < 64; i++) {
      const S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
      const ch = (e & f) ^ (~e & g);
      const t1 = (hh + S1 + ch + SHA_K[i] + w[i]) | 0;
      const S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
      const maj = (a & b) ^ (a & c) ^ (b & c);
      const t2 = (S0 + maj) | 0;
      hh = g; g = f; f = e;
      e = (d + t1) | 0;
      d = c; c = b; b = a;
      a = (t1 + t2) | 0;
    }
    H[0] = (H[0] + a) | 0; H[1] = (H[1] + b) | 0; H[2] = (H[2] + c) | 0; H[3] = (H[3] + d) | 0;
    H[4] = (H[4] + e) | 0; H[5] = (H[5] + f) | 0; H[6] = (H[6] + g) | 0; H[7] = (H[7] + hh) | 0;
  }
  return H.map((x) => (x >>> 0).toString(16).padStart(8, "0")).join("");
}
