import { useEffect, useRef, useState } from "react";
import { invoke } from "./tauri";
import { focusedSlot, sendTo } from "./registry";

/** Alt-invoked floating command history (egui parity): ↑↓ select,
 * Enter inserts into the focused terminal, Esc closes. */
export default function HistoryOverlay({ onClose }: { onClose: () => void }) {
  const [items, setItems] = useState<string[]>([]);
  const [sel, setSel] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<string[]>("get_history")
      .then((list) => {
        setItems(list);
        setSel(0);
      })
      .catch(() => setItems([]));
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSel((s) => Math.max(0, s - 1));
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        setSel((s) => Math.min(items.length - 1, s + 1));
      } else if (e.key === "Enter") {
        e.preventDefault();
        const cmd = items[sel];
        if (cmd) {
          // Insert without executing — user reviews then presses Enter.
          sendTo(focusedSlot.value, new TextEncoder().encode(cmd));
          onClose();
        }
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [items, sel, onClose]);

  useEffect(() => {
    listRef.current?.children[sel]?.scrollIntoView({ block: "nearest" });
  }, [sel]);

  return (
    <div
      className="animate-fade-up fixed right-6 top-14 z-[100] w-[420px] overflow-hidden rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] shadow-2xl"
      onClick={onClose}
    >
      <div className="flex items-center justify-between border-b border-[var(--border)] px-3 py-1.5 text-[11px] text-[var(--text-faint)]">
        <span>指令历史 · Alt 呼出</span>
        <span>↑↓ 选择 · Enter 插入 · Esc 关闭</span>
      </div>
      <div ref={listRef} className="max-h-[320px] overflow-y-auto py-1">
        {items.length === 0 ? (
          <div className="px-4 py-6 text-center text-[12px] text-[var(--text-faint)]">
            暂无历史命令
          </div>
        ) : (
          items.map((cmd, i) => (
            <div
              key={i}
              onMouseEnter={() => setSel(i)}
              onClick={() => {
                sendTo(focusedSlot.value, new TextEncoder().encode(cmd));
                onClose();
              }}
              className={`cursor-pointer px-3 py-1.5 font-mono text-[12px] ${
                i === sel ? "bg-[var(--accent-dim)] text-[var(--text)]" : "text-[var(--text-dim)]"
              }`}
            >
              {cmd}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
