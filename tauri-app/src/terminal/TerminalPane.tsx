import { useEffect, useRef, useState } from "react";
import { SearchAddon } from "@xterm/addon-search";
import SearchBar from "./SearchBar";
import {
  broadcastEnabled,
  broadcastGroup,
  broadcastInput,
  focusedSlot,
  registerSocket,
  unregisterSocket,
} from "./registry";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { invoke } from "./tauri";
import "@xterm/xterm/css/xterm.css";

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
  onFontSize,
}: {
  sessionId: number;
  themeId: string;
  fontSize: number;
  command?: string[];
  shell?: string;
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

  // Live theme/font updates without recreating the PTY.
  useEffect(() => {
    const t = termRef.current;
    if (!t) return;
    t.options.theme = readTerminalTheme();
    t.options.fontSize = fontSize;
    t.options.fontFamily =
      getComputedStyle(document.documentElement).getPropertyValue("--mono") || "monospace";
  }, [themeId, fontSize]);

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
      let start: StartResult;
      try {
        start = await invoke("start_terminal", {
          cols: term.cols,
          rows: term.rows,
          command: command ?? null,
          shell: shell ?? null,
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
        const data = ev.data as ArrayBuffer;
        // Session-exit sentinel (OSC 777) is shown as plain text.
        term.write(new Uint8Array(data));
      };
      term.attachCustomKeyEventHandler((e) => {
        if (e.ctrlKey && !e.shiftKey && !e.altKey && (e.key === "f" || e.key === "F") && e.type === "keydown") {
          if (!disposed) setSearchOpen(true);
          return false;
        }
        return true;
      });
      term.onData((data) => {
        const bytes = new TextEncoder().encode(data);
        if (ws && ws.readyState === WebSocket.OPEN) {
          ws.send(bytes);
        }
        // Broadcast mode: replicate keystrokes to the group.
        if (broadcastGroup.has(sessionId) || broadcastEnabled.value) {
          broadcastInput(sessionId, bytes);
        }
      });
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
      unregisterSocket(sessionId);
      ws?.close();
      search.dispose();
      term.dispose();
      termRef.current = null;
    };
  }, [sessionId, command, shell]);

  return (
    <div
      ref={hostRef}
      className="absolute inset-0 px-2 py-1"
      style={{ background: "var(--bg)" }}
      onMouseDown={() => {
        focusedSlot.value = sessionId;
      }}
    >
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
        style={{ color: status.state === "connected" ? "var(--success)" : "var(--danger)" }}
      >
        pty:{status.state}
        {status.detail ? ` ${status.detail}` : ""}
      </div>
    </div>
  );
}
