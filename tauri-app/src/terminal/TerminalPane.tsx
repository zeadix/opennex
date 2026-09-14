import { useEffect, useRef } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebglAddon } from "@xterm/addon-webgl";
import { invoke } from "./tauri";
import "@xterm/xterm/css/xterm.css";

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
export default function TerminalPane({ sessionId }: { sessionId: number }) {
  const hostRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const term = new Terminal({
      fontSize: 14,
      fontFamily: getComputedStyle(document.documentElement).getPropertyValue("--mono") || "monospace",
      cursorBlink: true,
      allowProposedApi: true,
      theme: {
        background: "#0b0e14",
        foreground: "#d6dbe6",
        cursor: "#4fc3f7",
        selectionBackground: "#2b7ba355",
      },
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    const webgl = new WebglAddon();
    term.loadAddon(webgl);
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

    (async () => {
      const start: StartResult = await invoke("start_terminal", {
        cols: term.cols,
        rows: term.rows,
      });
      if (disposed) return;
      ws = new WebSocket(`ws://127.0.0.1:${start.wsPort}/ws?session=${start.session}`);
      ws.binaryType = "arraybuffer";
      ws.onmessage = (ev) => {
        const data = ev.data as ArrayBuffer;
        // Session-exit sentinel (OSC 777) is shown as plain text.
        term.write(new Uint8Array(data));
      };
      term.onData((data) => {
        if (ws && ws.readyState === WebSocket.OPEN) {
          ws.send(new TextEncoder().encode(data));
        }
      });
      const sendResize = () => {
        if (ws && ws.readyState === WebSocket.OPEN && (term.cols !== cols || term.rows !== rows)) {
          cols = term.cols;
          rows = term.rows;
          ws.send(new TextEncoder().encode(JSON.stringify({ type: "resize", cols, rows })));
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
      ws.onopen = sendResize;
      (hostRef.current as any)._cleanup = () => ro.disconnect();
    })();

    return () => {
      disposed = true;
      const cleanup = (hostRef.current as any)?._cleanup;
      if (cleanup) cleanup();
      ws?.close();
      term.dispose();
    };
  }, [sessionId]);

  return (
    <div
      ref={hostRef}
      className="h-full w-full px-2 py-1"
      style={{ background: "var(--bg)" }}
    />
  );
}
