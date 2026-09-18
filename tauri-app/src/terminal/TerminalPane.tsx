import { useI18n } from '../i18n-context';
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import { FiCopy, FiClipboard } from "react-icons/fi";
import SearchBar from "./SearchBar";
import {
  broadcastEnabled,
  broadcastGroup,
  broadcastInput,
  cursorRefreshers,
  focusedSlot,
  lastCursor,
  registerSocket,
  sockets,
  unregisterSocket,
} from "./registry";

/** Drag an overlay by a handle; `apply` receives the new viewport position. */
export function beginOverlayDrag(e: React.MouseEvent, apply: (x: number, y: number) => void) {
  const el = (e.currentTarget as HTMLElement).parentElement;
  if (!el) return;
  const rect = el.getBoundingClientRect();
  const dx = e.clientX - rect.left;
  const dy = e.clientY - rect.top;
  const move = (ev: MouseEvent) => apply(ev.clientX - dx, ev.clientY - dy);
  const up = () => {
    window.removeEventListener("mousemove", move);
    window.removeEventListener("mouseup", up);
  };
  window.addEventListener("mousemove", move);
  window.addEventListener("mouseup", up);
}
import { invoke } from "./tauri";

import "@xterm/xterm/css/xterm.css";

/**
 * Command suggestions for the typed `word` (port of egui's
 * completion.rs): history entries matching the WHOLE text by prefix,
 * ranked by re-run count; then PATH executables by prefix, skipping
 * names already covered by a history entry's first token.
 */
function suggestions(word: string, rankedHistory: string[], pathCmds: string[], limit = 10): string[] {
  if (!word) return [];
  const out: string[] = [];
  const covered = new Set<string>();
  for (const cmd of rankedHistory) {
    if (out.length >= limit) break;
    if (cmd.startsWith(word) && !out.includes(cmd)) {
      const first = cmd.split(/\s+/)[0];
      if (first) covered.add(first);
      out.push(cmd);
    }
  }
  for (const name of pathCmds) {
    if (out.length >= limit) break;
    if (name.startsWith(word) && !covered.has(name)) out.push(name);
  }
  return out;
}

function readTerminalTheme() {
  const cs = getComputedStyle(document.documentElement);
  const v = (name: string, fallback: string) => cs.getPropertyValue(name).trim() || fallback;
  // Dedicated --term-* tokens (theme editor) fall back to the UI colors.
  const ansi = Array.from({ length: 16 }, (_, i) => v(`--term-ansi-${i}`, "#000000"));
  return {
    background: v("--term-background", v("--bg", "#0b0e14")),
    foreground: v("--term-foreground", v("--text", "#d7dce7")),
    cursor: v("--term-cursor", v("--accent", "#4fc3f7")),
    cursorAccent: v("--bg", "#0b0e14"),
    selectionBackground: v("--term-selection", v("--accent-dim", "#23465c")),
    black: ansi[0], red: ansi[1], green: ansi[2], yellow: ansi[3],
    blue: ansi[4], magenta: ansi[5], cyan: ansi[6], white: ansi[7],
    brightBlack: ansi[8], brightRed: ansi[9], brightGreen: ansi[10],
    brightYellow: ansi[11], brightBlue: ansi[12], brightMagenta: ansi[13],
    brightCyan: ansi[14], brightWhite: ansi[15],
  };
}

interface StartResult {
  session: string;
  wsPort: number;
}

const SENTINEL = new TextEncoder().encode("\x1b]777;session-exit\x07");

/** Byte offset of the session-exit sentinel in a WS chunk (-1 if absent). */
function findSentinel(hay: Uint8Array): number {
  outer: for (let i = 0; i + SENTINEL.length <= hay.length; i++) {
    for (let j = 0; j < SENTINEL.length; j++) {
      if (hay[i + j] !== SENTINEL[j]) continue outer;
    }
    return i;
  }
  return -1;
}

/**
 * One PTY-backed terminal. Raw bytes flow over a localhost WebSocket:
 * PTY(Rust) -> WS -> xterm.write, and xterm.onData -> WS -> PTY.
 * Resize: ResizeObserver -> fit -> resize message -> Rust resizes the
 * PTY winsize.
 */
interface TerminalPaneProps {
  workspaceId: number;
  sessionId: number;
  themeId: string;
  fontSize: number;
  command?: string[];
  shell?: string;
  cwd?: string;
  autoMatch?: boolean;
  followCursor?: boolean;
  copyOnSelect?: boolean;
  onFontSize?: (size: number) => void;
  name?: string;
}

export default function TerminalPane(props: TerminalPaneProps) {
  // Reset local caches and overlays before another workspace can render them.
  return <WorkspaceTerminalPane key={`${props.workspaceId}:${props.sessionId}`} {...props} />;
}

function WorkspaceTerminalPane({
  workspaceId,
  sessionId,
  themeId,
  fontSize,
  command,
  shell,
  cwd,
  autoMatch,
  followCursor,
  copyOnSelect,
  onFontSize,
  name,
}: TerminalPaneProps) {
  const T = useI18n();
  // Keep long-lived PTY callbacks current without restarting the session.
  const Tref = useRef(T);
  Tref.current = T;
  const hostRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const [searchOpen, setSearchOpen] = useState(false);
  const searchRef = useRef<SearchAddon | null>(null);
  /** 字号/字体变化后的重适配入口（big effect 内赋值）。 */
  const fitRef = useRef<{ fitAndSync: () => void } | null>(null);
  // Custom right-click menu (copy/paste) — the webview's own context
  // menu is suppressed app-wide, this is the only menu terminals show.
  const [ctx, setCtx] = useState<{ x: number; y: number } | null>(null);
  // Auto-match state: current input line + suggestion list + history
  // cache. The overlay mirrors egui's HistoryNav: pristine (no arrows
  // used) Enter stays a plain terminal Enter and Tab completes; after
  // navigation Enter confirms the highlighted entry.
  const [suggest, setSuggest] = useState<{
    list: string[];
    word: string;
    sel: number;
    navigated: boolean;
  } | null>(null);
  const historyRef = useRef<{ id: number; cmd: string; hits: number }[]>([]);
  const pathCmdsRef = useRef<string[]>([]);
  const autoMatchRef = useRef(autoMatch);
  autoMatchRef.current = autoMatch;
  const copyOnSelectRef = useRef(copyOnSelect);
  copyOnSelectRef.current = copyOnSelect;
  // Latch: a palette/history insert rewrote the input line — auto-match
  // must stay closed until the next REAL keystroke (egui's
  // history_menu_just_closed latch).
  const suppressMatchRef = useRef(false);
  // Mirrors `suggest` for the one-time key handler inside the effect.
  const suggestRef = useRef(suggest);
  suggestRef.current = suggest;
  // Remembered overlay position (followCursor off): draggable, restored
  // across restarts.
  const [suggestPos, setSuggestPos] = useState<{ x: number; y: number } | null>(() => {
    try {
      return JSON.parse(localStorage.getItem("opennex-suggest-pos") ?? "null");
    } catch {
      return null;
    }
  });
  const dragSuggest = (e: React.MouseEvent) => {
    beginOverlayDrag(e, (x, y) => {
      const p = { x: Math.max(0, x), y: Math.max(0, y) };
      setSuggestPos(p);
      localStorage.setItem("opennex-suggest-pos", JSON.stringify(p));
    });
  };

  // Live theme/font updates without recreating the PTY.
  useEffect(() => {
    const t = termRef.current;
    if (!t) return;
    t.options.theme = readTerminalTheme();
    t.options.fontSize = fontSize;
    t.options.fontFamily =
      getComputedStyle(document.documentElement).getPropertyValue("--mono") || "monospace";
    // 字号/字体变化后必须重新 fit（重算列数）并同步 PTY winsize，
    // 否则换行停留 在旧列数 —— 小字号时内容右侧留白一大块。
    // xterm 异步重测字形 → 立即一次 + 80ms 后补一次。
    fitRef.current?.fitAndSync();
    const tid = window.setTimeout(() => fitRef.current?.fitAndSync(), 80);
    return () => window.clearTimeout(tid);
  }, [themeId, fontSize]);

  // Theme editor live preview + font changes: tokens changed under the
  // same theme id.
  useEffect(() => {
    const onTheme = () => {
      const t = termRef.current;
      if (!t) return;
      t.options.theme = readTerminalTheme();
      t.options.fontFamily =
        getComputedStyle(document.documentElement).getPropertyValue("--mono") || "monospace";
      // 字体族变化会改变字形宽度 → 重新 fit 并同步 PTY。
      fitRef.current?.fitAndSync();
    };
    window.addEventListener("opennex-terminal-theme", onTheme);
    return () => window.removeEventListener("opennex-terminal-theme", onTheme);
  }, []);

  // Context menu closes on any outside click or window blur.
  useEffect(() => {
    if (!ctx) return;
    const close = () => setCtx(null);
    window.addEventListener("mousedown", close);
    window.addEventListener("blur", close);
    return () => {
      window.removeEventListener("mousedown", close);
      window.removeEventListener("blur", close);
    };
  }, [ctx]);

  const copySelection = async () => {
    const sel = termRef.current?.getSelection() ?? "";
    if (sel) await navigator.clipboard.writeText(sel);
    setCtx(null);
    termRef.current?.focus();
  };
  const pasteClipboard = async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (text) termRef.current?.paste(text);
    } catch {
      /* clipboard read denied by the webview */
    }
    setCtx(null);
    termRef.current?.focus();
  };

  useEffect(() => {
    const term = new Terminal({
      fontSize: fontSize ?? 14,
      fontFamily: getComputedStyle(document.documentElement).getPropertyValue("--mono") || "monospace",
      cursorBlink: true,
      allowProposedApi: true,
      theme: readTerminalTheme(),
    });
    termRef.current = term;
    const fit = new FitAddon();
    term.loadAddon(fit);
    const search = new SearchAddon();
    searchRef.current = search;
    term.loadAddon(search);
    // NOTE: no WebGL addon by default — WebKitGTK's WebGL regularly
    // renders a fully BLACK viewport on Linux GPU stacks, which read as
    // "the terminal is empty and inputs go nowhere". The 2D canvas
    // renderer is the stable path here; add WebGL behind a setting once
    // verified per-machine.
    term.open(hostRef.current!);
    try {
      fit.fit();
    } catch {
      /* container not laid out yet */
    }

    let ws: WebSocket | null = null;
    let disposed = false;
    let rows = term.rows;
    let cols = term.cols;

    // 字号/字体变化（font effect 与 Ctrl+滚轮）后的重适配入口：
    // fit 重算列数 → 把新 cols/rows 同步给 PTY（经 registry socket）。
    fitRef.current = {
      fitAndSync: () => {
        try {
          fit.fit();
          const sock = sockets.get(sessionId);
          if (sock && sock.readyState === WebSocket.OPEN && (term.cols !== cols || term.rows !== rows)) {
            cols = term.cols;
            rows = term.rows;
            sock.send(JSON.stringify({ type: "resize", cols, rows }));
          }
        } catch {
          /* not laid out yet */
        }
      },
    };

    (async () => {
      invoke<Array<{ id: number; cmd: string; hits: number }>>("get_history", { workspaceId })
        .then((h) => { if (!disposed) historyRef.current = h; })
        .catch(() => {});
      invoke<string[]>("list_path_commands")
        .then((c) => { if (!disposed) pathCmdsRef.current = c; })
        .catch(() => {});
      // Wait for a REAL layout before spawning the shell: a PTY born at
      // 1–2 columns makes the prompt wrap into garbage that the
      // scrollback then replays forever (the "flooded characters" bug).
      for (let i = 0; i < 100; i++) {
        if (disposed) return;
        const el = hostRef.current;
        if (el && el.clientWidth > 60 && el.clientHeight > 40) break;
        await new Promise((r) => setTimeout(r, 50));
      }
      if (disposed) return;
      try {
        fit.fit();
      } catch {
        /* still not laid out */
      }
      let start: StartResult;
      try {
        start = await invoke("start_terminal", {
          cols: term.cols,
          rows: term.rows,
          command: command ?? null,
          shell: shell ?? null,
          name: name ?? null,
          sessionId: String(sessionId),
          workspaceId,
          cwd: cwd ?? null,
        });
      } catch (e) {
        if (!disposed) {
          term.writeln(`\x1b[31m[${Tref.current.uPtyFailed}: ${e}]\x1b[0m`);
        }
        return;
      }
      // A delayed response can belong to a workspace that was just detached.
      // Do not kill its session: another mount may already have reattached it.
      // Explicit tab/workspace deletion owns close_session.
      if (disposed) return;
      ws = new WebSocket(`ws://127.0.0.1:${start.wsPort}/ws?session=${start.session}`);
      ws.binaryType = "arraybuffer";
      registerSocket(sessionId, ws);
      ws.onerror = () => {};
      ws.onmessage = (ev) => {
        const bytes = new Uint8Array(ev.data as ArrayBuffer);
        // Session-exit sentinel (OSC 777) rides in its own chunk from the
        // pump — swap it for a visible end-of-session line and flip the
        // status badge (sessions now survive detaches, so the socket
        // stays open and onclose no longer signals this).
        const idx = findSentinel(bytes);
        if (idx >= 0) {
          if (idx > 0) term.write(bytes.subarray(0, idx));
          if (!disposed) {
            term.write(`\r\n\x1b[90m[${Tref.current.sessionEnded}]\x1b[0m\r\n`);
          }
          return;
        }
        term.write(bytes);
      };
      // Auto-match: track the input line locally; the typed text (after
      // the prompt) is matched against history (whole-text prefix,
      // ranked by re-run count) and PATH executables. The overlay is
      // PRISTINE until an arrow key is used: Enter stays a plain
      // terminal Enter, Tab completes the highlighted suggestion.
      let buf = "";
      // Backend already ranks newest-first with re-run counts — sort by hits.
      const rankedHistory = () =>
        [...historyRef.current].sort((a, b) => b.hits - a.hits).map((e) => e.cmd);
      const updateSuggest = () => {
        if (!autoMatchRef.current || suppressMatchRef.current) {
          setSuggest(null);
          return;
        }
        const word = buf.replace(/^\s+/, "");
        const m = word ? suggestions(word, rankedHistory(), pathCmdsRef.current, 10) : [];
        if (m.length === 0 || (m.length === 1 && m[0] === word)) {
          setSuggest(null);
          return;
        }
        // Exclusive with the Alt palette: opening the auto-match list
        // closes the palette.
        window.dispatchEvent(new CustomEvent("opennex-close-palette"));
        // Keep the selection across edits; every fresh edit re-opens
        // PRISTINE mode (egui: navigation must be redone after typing).
        setSuggest((prev) => ({
          list: m,
          word,
          sel: prev ? Math.min(prev.sel, m.length - 1) : 0,
          navigated: false,
        }));
      };
      // Track the input caret in SCREEN px (palette/overlay positioning).
      const updateCursorPos = () => {
        try {
          const host = hostRef.current;
          if (!host) return;
          const rect = host.getBoundingClientRect();
          const dims = (term as any)?._core?._renderService?.dimensions?.css;
          const cw = dims?.cell?.width ?? 9;
          const ch = dims?.cell?.height ?? 20;
          lastCursor.x = rect.left + 8 + (term.buffer.active.cursorX ?? 0) * cw;
          lastCursor.y = rect.top + 4 + (term.buffer.active.cursorY ?? 0) * ch;
        } catch {
          /* ignore */
        }
      };
      term.onData((data) => {
        // A real keystroke re-arms auto-match after a palette insert.
        suppressMatchRef.current = false;
        for (const ch of data) {
          if (ch === "\r") buf = "";
          else if (ch === "\x7f" || ch === "\b") buf = buf.slice(0, -1);
          else if (ch >= " ") buf += ch;
        }
        updateCursorPos();
        updateSuggest();
        const bytes = new TextEncoder().encode(data);
        if (ws && ws.readyState === WebSocket.OPEN) {
          ws.send(bytes);
        }
        // Broadcast mode: replicate keystrokes to the group.
        if (broadcastGroup.has(sessionId) || broadcastEnabled.value) {
          broadcastInput(sessionId, bytes);
        }
      });
      // Keep the tracked caret fresh WITHOUT keystrokes too (shell output
      // redraws, scrolls, pane resizes) — the Alt palette reads it at open.
      term.onRender(() => updateCursorPos());
      term.onScroll(() => updateCursorPos());
      // Alt 面板打开前由 registry 逐一调用，强制刷新光标屏幕坐标。
      cursorRefreshers.add(updateCursorPos);
      (hostRef.current as any)._cursorCleanup = () => cursorRefreshers.delete(updateCursorPos);
      // Overlay + search key handling (single dispatcher; suggestRef
      // mirrors the latest overlay state for this one-time handler).
      term.attachCustomKeyEventHandler((e) => {
        if (e.type !== "keydown") return true;
        if (e.ctrlKey && !e.shiftKey && !e.altKey && (e.key === "f" || e.key === "F")) {
          if (!disposed) setSearchOpen(true);
          return false;
        }
        const sg = suggestRef.current;
        if (!sg) return true;
        if (e.key === "ArrowDown" && !e.ctrlKey && !e.altKey) {
          e.preventDefault();
          setSuggest((s) => (s ? { ...s, sel: Math.min(s.sel + 1, s.list.length - 1), navigated: true } : s));
          return false;
        }
        if (e.key === "ArrowUp" && !e.ctrlKey && !e.altKey) {
          e.preventDefault();
          setSuggest((s) => (s ? { ...s, sel: Math.max(0, s.sel - 1), navigated: true } : s));
          return false;
        }
        if (e.key === "Escape") {
          e.preventDefault();
          setSuggest(null);
          return false;
        }
        if (e.key === "Enter" && !e.ctrlKey && !e.altKey) {
          if (sg.navigated) {
            // Confirm the highlighted entry: delete the typed word
            // from the line, then insert the full command — NOT
            // executed (egui confirm_history_entry semantics).
            e.preventDefault();
            const del = "\x7f".repeat([...sg.word].length);
            const bytes = new TextEncoder().encode(del + sg.list[sg.sel]);
            if (ws && ws.readyState === WebSocket.OPEN) ws.send(bytes);
            buf = sg.list[sg.sel];
            setSuggest(null);
            return false;
          }
          // Pristine: the Enter executes the typed line; the onData
          // handler clears the buffer and the overlay with it.
          setSuggest(null);
          return true;
        }
        if (e.key === "Tab" && !e.ctrlKey && !e.altKey) {
          // Tab accepts the highlighted suggestion: send only the
          // remainder beyond the typed word.
          e.preventDefault();
          const full = sg.list[sg.sel];
          const rest = full.startsWith(sg.word) ? full.slice(sg.word.length) : full;
          if (rest) {
            const bytes = new TextEncoder().encode(rest);
            if (ws && ws.readyState === WebSocket.OPEN) ws.send(bytes);
          }
          buf = full;
          setSuggest(null);
          return false;
        }
        return true;
      });
      // History palette inserts bypass onData — resync the tracked line.
      const onLineSet = (ev: Event) => {
        const d = (ev as CustomEvent).detail ?? {};
        if (d.slot === sessionId) {
          buf = String(d.text ?? "");
          // Inserted text must not re-open the auto-match overlay.
          suppressMatchRef.current = true;
          setSuggest(null);
        }
      };
      window.addEventListener("opennex-line-set", onLineSet);
      (hostRef.current as any)._lineSetCleanup = () => window.removeEventListener("opennex-line-set", onLineSet);
      // Alt palette opening closes the auto-match list (exclusive pair).
      const onCloseSuggest = () => setSuggest(null);
      window.addEventListener("opennex-close-suggest", onCloseSuggest);
      (hostRef.current as any)._closeSuggestCleanup = () =>
        window.removeEventListener("opennex-close-suggest", onCloseSuggest);
      const sendResize = () => {
        if (ws && ws.readyState === WebSocket.OPEN && (term.cols !== cols || term.rows !== rows)) {
          cols = term.cols;
          rows = term.rows;
          // MUST be a TEXT frame: the backend routes Text frames to the
          // control channel and Binary frames to the PTY input. Encoding
          // this JSON as binary wrote the literal `{"type":"resize"...}`
          // into the shell — polluting the input line (terminals showed
          // the JSON and Enter stopped working).
          ws.send(JSON.stringify({ type: "resize", cols, rows }));
        }
      };
      // A dead socket must never look like a live terminal: show the
      // session end so keystrokes are visibly going nowhere.
      ws.onclose = () => {
        if (!disposed) {
          term.write(`\r\n\x1b[90m[${Tref.current.sessionEnded}]\x1b[0m\r\n`);
        }
      };
      // Debounced resize: the dock settles over several frames — resizing
      // the PTY at every intermediate width makes readline repaint the
      // prompt each time.
      let roTimer: number | null = null;
      const ro = new ResizeObserver(() => {
        if (roTimer !== null) window.clearTimeout(roTimer);
        roTimer = window.setTimeout(() => {
          roTimer = null;
          const el = hostRef.current;
          if (!el || el.clientWidth < 60 || el.clientHeight < 40) return;
          try {
            fit.fit();
            sendResize();
          } catch {
            /* zero-size during layout */
          }
        }, 120);
      });
      ro.observe(hostRef.current!);
      ws.onopen = () => sendResize();
      // Ctrl+wheel = font size (matches the egui build's behavior).
      const host = hostRef.current!;
      const onWheel = (e: WheelEvent) => {
        if (!e.ctrlKey || !onFontSize) return;
        e.preventDefault();
        const next = Math.min(28, Math.max(8, fontSize + (e.deltaY < 0 ? 1 : -1)));
        if (next !== fontSize) onFontSize(next);
      };
      host.addEventListener("wheel", onWheel, { passive: false });
      (host as any)._wheelCleanup = () => host.removeEventListener("wheel", onWheel);
      // 拖选即复制（设置可关）：松开鼠标时把选中文本送入剪贴板。
      const onMouseUp = () => {
        if (!copyOnSelectRef.current) return;
        const sel = term.getSelection();
        if (!sel) return;
        navigator.clipboard
          .writeText(sel)
          .then(() => window.dispatchEvent(new CustomEvent("opennex-toast", { detail: Tref.current.uCopiedClipboard })))
          .catch(() => {});
      };
      host.addEventListener("mouseup", onMouseUp);
      (host as any)._mouseupCleanup = () => host.removeEventListener("mouseup", onMouseUp);
      // Right-click opens OUR menu (copy/paste), never the webview's.
      const onCtxMenu = (e: MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setCtx({ x: e.clientX, y: e.clientY });
      };
      host.addEventListener("contextmenu", onCtxMenu);
      (host as any)._ctxCleanup = () => host.removeEventListener("contextmenu", onCtxMenu);
      // External search request (global shortcut routes to the focused pane).
      const onSearchEvent = () => {
        if (focusedSlot.value === sessionId && !disposed) setSearchOpen(true);
      };
      window.addEventListener("opennex-search", onSearchEvent);
      (host as any)._searchEvtCleanup = () => window.removeEventListener("opennex-search", onSearchEvent);
      // Ctrl+F opens the search strip.
      const onKey = (e: KeyboardEvent) => {
        if (e.ctrlKey && !e.shiftKey && !e.altKey && (e.key === "f" || e.key === "F")) {
          e.preventDefault();
          if (!disposed) setSearchOpen(true);
        }
      };
      host.addEventListener("keydown", onKey);
      (host as any)._keyCleanup2 = () => host.removeEventListener("keydown", onKey);
      (hostRef.current as any)._cleanup = () => ro.disconnect();
    })();

    return () => {
      disposed = true;
      (hostRef.current as any)?._cursorCleanup?.();
      const cleanup = (hostRef.current as any)?._cleanup;
      if (cleanup) cleanup();
      (hostRef.current as any)?._wheelCleanup?.();
      (hostRef.current as any)?._keyCleanup2?.();
      (hostRef.current as any)?._ctxCleanup?.();
      (hostRef.current as any)?._mouseupCleanup?.();
      (hostRef.current as any)?._lineSetCleanup?.();
      (hostRef.current as any)?._closeSuggestCleanup?.();
      (hostRef.current as any)?._searchEvtCleanup?.();
      unregisterSocket(sessionId);
      ws?.close();
      search.dispose();
      term.dispose();
      termRef.current = null;
    };
  }, [sessionId, workspaceId, command, shell]);

  return (
    <div
      ref={hostRef}
      className="absolute inset-0 px-2 py-1"
      style={{ background: "var(--bg)" }}
      onMouseDown={() => {
        focusedSlot.value = sessionId;
      }}
    >
      {suggest &&
        createPortal(
        <div
          className="animate-fade-up fixed z-[6000] w-[min(480px,calc(100vw-16px))] overflow-hidden rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] shadow-2xl"
          style={
            suggestPos
              ? { left: suggestPos.x, top: suggestPos.y }
              : followCursor && lastCursor.x > 0
                ? {
                    left: Math.max(4, Math.min(lastCursor.x, window.innerWidth - 496)),
                    top:
                      lastCursor.y + 190 > window.innerHeight
                        ? Math.max(4, lastCursor.y - 190)
                        : lastCursor.y + 24,
                  }
                : { right: 12, bottom: 44 }
          }
          onMouseDown={(e) => e.stopPropagation()}
        >
          <div className="max-h-[220px] overflow-y-auto py-1">
            {suggest.list.map((cmd, i) => (
              <div
                key={i}
                onMouseEnter={() => setSuggest((s) => (s ? { ...s, sel: i, navigated: true } : s))}
                onClick={() => {
                  const del = "\x7f".repeat([...suggest.word].length);
                  const bytes = new TextEncoder().encode(del + cmd);
                  const socket = sockets.get(sessionId);
                  if (socket && socket.readyState === WebSocket.OPEN) socket.send(bytes);
                  setSuggest(null);
                }}
                className={`flex cursor-pointer items-center gap-2 px-3 py-1 font-mono text-[12px] ${
                  i === suggest.sel
                    ? "bg-[var(--accent-dim)] text-[var(--text)]"
                    : "text-[var(--text-dim)]"
                }`}
              >
                <span className="w-4 shrink-0 text-right text-[10px] text-[var(--text-faint)]">{i + 1}</span>
                <span className="min-w-0 flex-1 truncate">{cmd}</span>
              </div>
            ))}
          </div>
          <div
            className="flex cursor-move items-center justify-between border-t border-[var(--border)] px-3 py-1 text-[10px] text-[var(--text-faint)]"
            title={T.uDragPosition}
            onMouseDown={(e) => {
              if (followCursor) return;
              e.stopPropagation();
              dragSuggest(e);
            }}
          >
            <span>
              {suggest.navigated ? T.uSuggestNavigated : T.uSuggestPristine}
            </span>
            {!followCursor && <span className="ml-2 opacity-60">⣿</span>}
          </div>
        </div>,
        document.body,
      )}
      {searchOpen && (
        <SearchBar
          onSearch={(q) => (q ? searchRef.current?.findNext(q) : searchRef.current?.clearDecorations())}
          onNext={() => searchRef.current?.findNext("")}
          onPrev={() => searchRef.current?.findPrevious("")}
          onClose={() => {
            searchRef.current?.clearDecorations();
            setSearchOpen(false);
            termRef.current?.focus();
          }}
        />
      )}
      {ctx &&
        createPortal(
        <div
          className="ctx-menu animate-fade-up fixed"
          style={{
            left: Math.max(4, Math.min(ctx.x, window.innerWidth - 164)),
            top: Math.max(4, Math.min(ctx.y, window.innerHeight - 96)),
          }}
          onMouseDown={(e) => e.stopPropagation()}
          onContextMenu={(e) => e.preventDefault()}
        >
          <button className="ctx-item" disabled={!termRef.current?.hasSelection()} onClick={copySelection}>
            <FiCopy size={13} /> {T.copy}
          </button>
          <button className="ctx-item" onClick={pasteClipboard}>
            <FiClipboard size={13} /> {T.paste}
          </button>
        </div>,
        document.body,
      )}
    </div>
  );
}
