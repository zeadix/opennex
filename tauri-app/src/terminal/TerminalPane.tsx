import { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import { FiCopy, FiClipboard } from "react-icons/fi";
import SearchBar from "./SearchBar";
import {
  broadcastEnabled,
  broadcastGroup,
  broadcastInput,
  focusedSlot,
  registerSocket,
  sockets,
  unregisterSocket,
} from "./registry";
import { invoke } from "./tauri";
import { t, loadLang } from "../i18n";
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
  const v = (name: string) => cs.getPropertyValue(name).trim();
  return {
    background: v("--bg"),
    foreground: v("--text"),
    cursor: v("--accent"),
    cursorAccent: v("--bg"),
    selectionBackground: v("--accent-dim"),
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
export default function TerminalPane({
  sessionId,
  themeId,
  fontSize,
  command,
  shell,
  autoMatch,
  onFontSize,
}: {
  sessionId: number;
  themeId: string;
  fontSize: number;
  command?: string[];
  shell?: string;
  autoMatch?: boolean;
  onFontSize?: (size: number) => void;
  name?: string;
}) {
  const hostRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  // HTML-level status badge: always visible even when the xterm canvas
  // itself fails to draw (webview rendering bugs).
  const [status, setStatus] = useState<{ state: string; detail?: string }>({
    state: "connecting",
  });
  const [searchOpen, setSearchOpen] = useState(false);
  const searchRef = useRef<SearchAddon | null>(null);
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
  const historyRef = useRef<string[]>([]);
  const pathCmdsRef = useRef<string[]>([]);
  const autoMatchRef = useRef(autoMatch);
  autoMatchRef.current = autoMatch;
  // Mirrors `suggest` for the one-time key handler inside the effect.
  const suggestRef = useRef(suggest);
  suggestRef.current = suggest;

  // Live theme/font updates without recreating the PTY.
  useEffect(() => {
    const t = termRef.current;
    if (!t) return;
    t.options.theme = readTerminalTheme();
    t.options.fontSize = fontSize;
    t.options.fontFamily =
      getComputedStyle(document.documentElement).getPropertyValue("--mono") || "monospace";
  }, [themeId, fontSize]);

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
    term.writeln("\x1b[90m[连接 PTY 中…]\x1b[0m");
    try {
      fit.fit();
    } catch {
      /* container not laid out yet */
    }

    let ws: WebSocket | null = null;
    let disposed = false;
    let rows = term.rows;
    let cols = term.cols;

    (async () => {
      invoke<string[]>("get_history")
        .then((h) => (historyRef.current = h))
        .catch(() => {});
      invoke<string[]>("list_path_commands")
        .then((c) => (pathCmdsRef.current = c))
        .catch(() => {});
      let start: StartResult;
      try {
        start = await invoke("start_terminal", {
          cols: term.cols,
          rows: term.rows,
          command: command ?? null,
          shell: shell ?? null,
          name: name ?? null,
          sessionId: String(sessionId),
        });
      } catch (e) {
        if (!disposed) {
          setStatus({ state: "failed", detail: String(e) });
          term.writeln(`\x1b[31m[启动 PTY 失败: ${e}]\x1b[0m`);
        }
        return;
      }
      if (disposed) return;
      setStatus({ state: "attaching" });
      ws = new WebSocket(`ws://127.0.0.1:${start.wsPort}/ws?session=${start.session}`);
      ws.binaryType = "arraybuffer";
      registerSocket(sessionId, ws);
      ws.onerror = () => {
        if (!disposed) {
          setStatus({ state: "ws-error" });
          term.writeln("\x1b[31m[WS 错误]\x1b[0m");
        }
      };
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
            setStatus({ state: "ended" });
            term.write("\r\n\x1b[90m[会话已结束]\x1b[0m\r\n");
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
      const rankedHistory = () => {
        const counts = new Map<string, number>();
        for (const h of historyRef.current) counts.set(h, (counts.get(h) ?? 0) + 1);
        return [...counts.entries()].sort((a, b) => b[1] - a[1]).map(([c]) => c);
      };
      const updateSuggest = () => {
        if (!autoMatchRef.current) {
          setSuggest(null);
          return;
        }
        const word = buf.replace(/^\s+/, "");
        const m = word ? suggestions(word, rankedHistory(), pathCmdsRef.current, 10) : [];
        if (m.length === 0 || (m.length === 1 && m[0] === word)) {
          setSuggest(null);
          return;
        }
        // Keep the selection across edits; every fresh edit re-opens
        // PRISTINE mode (egui: navigation must be redone after typing).
        setSuggest((prev) => ({
          list: m,
          word,
          sel: prev ? Math.min(prev.sel, m.length - 1) : 0,
          navigated: false,
        }));
      };
      term.onData((data) => {
        for (const ch of data) {
          if (ch === "\r") buf = "";
          else if (ch === "\x7f" || ch === "\b") buf = buf.slice(0, -1);
          else if (ch >= " ") buf += ch;
        }
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
          updateSuggest();
        }
      };
      window.addEventListener("opennex-line-set", onLineSet);
      (hostRef.current as any)._lineSetCleanup = () => window.removeEventListener("opennex-line-set", onLineSet);
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
          term.write("\r\n\x1b[90m[会话已结束]\x1b[0m\r\n");
        }
      };
      const ro = new ResizeObserver(() => {
        try {
          fit.fit();
          sendResize();
        } catch {
          /* zero-size during layout */
        }
      });
      ro.observe(hostRef.current!);
      ws.onopen = () => {
        if (!disposed) {
          setStatus({ state: "connected" });
          term.writeln("\x1b[90m[已连接]\x1b[0m");
        }
        sendResize();
      };
      ws.onclose = () => {
        if (!disposed) setStatus({ state: "closed" });
      };
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
      const cleanup = (hostRef.current as any)?._cleanup;
      if (cleanup) cleanup();
      (hostRef.current as any)?._wheelCleanup?.();
      (hostRef.current as any)?._keyCleanup2?.();
      (hostRef.current as any)?._ctxCleanup?.();
      (hostRef.current as any)?._lineSetCleanup?.();
      (hostRef.current as any)?._searchEvtCleanup?.();
      unregisterSocket(sessionId);
      ws?.close();
      search.dispose();
      term.dispose();
      termRef.current = null;
    };
  }, [sessionId, command, shell]);

  const L = t(loadLang());
  return (
    <div
      ref={hostRef}
      className="absolute inset-0 px-2 py-1"
      style={{ background: "var(--bg)" }}
      onMouseDown={() => {
        focusedSlot.value = sessionId;
      }}
    >
      {suggest && (
        <div
          className="animate-fade-up absolute bottom-1 left-2 z-30 w-[min(480px,calc(100%-16px))] overflow-hidden rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] shadow-2xl"
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
          <div className="border-t border-[var(--border)] px-3 py-1 text-[10px] text-[var(--text-faint)]">
            ↑↓ 选择 · Tab 补全 · {suggest.navigated ? "Enter 插入选中指令" : "Enter 直接执行"} · Esc 关闭
          </div>
        </div>
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
      <div
        className="pointer-events-none absolute right-2 top-1 z-20 rounded bg-[var(--bg-elevated)] px-1.5 py-0.5 font-mono text-[10px]"
        style={{
          color:
            status.state === "connected"
              ? "var(--success)"
              : status.state === "ended"
                ? "var(--text-faint)"
                : "var(--danger)",
        }}
      >
        pty:{status.state}
        {status.detail ? ` ${status.detail}` : ""}
      </div>
      {ctx && (
        <div
          className="ctx-menu animate-fade-up"
          style={{
            left: Math.max(4, Math.min(ctx.x, window.innerWidth - 164)),
            top: Math.max(4, Math.min(ctx.y, window.innerHeight - 96)),
          }}
          onMouseDown={(e) => e.stopPropagation()}
          onContextMenu={(e) => e.preventDefault()}
        >
          <button className="ctx-item" disabled={!termRef.current?.hasSelection()} onClick={copySelection}>
            <FiCopy size={13} /> {L.copy}
          </button>
          <button className="ctx-item" onClick={pasteClipboard}>
            <FiClipboard size={13} /> {L.paste}
          </button>
        </div>
      )}
    </div>
  );
}
