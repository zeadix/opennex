import { useEffect, useRef, useState } from "react";
import { FiStar } from "react-icons/fi";
import { invoke } from "./tauri";
import { focusedSlot, sendTo } from "./registry";
import { loadFavorites, saveFavorites } from "../pages/FavoritesPage";

/**
 * The manual command palette (egui parity): two columns — command
 * history (left) and favorite commands (right). ↑↓ navigate the
 * focused column, ←→ switch columns, Enter inserts the selected entry
 * into the focused terminal's input line WITHOUT executing (the user
 * reviews and presses Enter), Esc closes. Rows in the history column
 * toggle a favorite star.
 */
export default function HistoryOverlay({ onClose }: { onClose: () => void }) {
  const [items, setItems] = useState<string[]>([]);
  const [favs, setFavs] = useState<string[]>(() => loadFavorites());
  const [col, setCol] = useState<"hist" | "fav">("hist");
  const [selH, setSelH] = useState(0);
  const [selF, setSelF] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);
  const favRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<string[]>("get_history")
      .then((list) => {
        setItems(list);
        setSelH(0);
      })
      .catch(() => setItems([]));
  }, []);

  const insert = (cmd: string) => {
    // Insert into the input line — no auto-execute (egui semantics).
    sendTo(focusedSlot.value, new TextEncoder().encode(cmd));
    // Sync the pane's local input-line tracker so auto-match resumes
    // from the inserted text instead of a stale buffer.
    window.dispatchEvent(new CustomEvent("opennex-line-set", { detail: { slot: focusedSlot.value, text: cmd } }));
    onClose();
  };

  const toggleFav = (cmd: string) => {
    const next = favs.includes(cmd) ? favs.filter((f) => f !== cmd) : [...favs, cmd];
    setFavs(next);
    saveFavorites(next);
  };

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        if (col === "hist") setSelH((s) => Math.max(0, s - 1));
        else setSelF((s) => Math.max(0, s - 1));
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        if (col === "hist") setSelH((s) => Math.min(items.length - 1, s + 1));
        else setSelF((s) => Math.min(favs.length - 1, s + 1));
      } else if (e.key === "ArrowRight") {
        e.preventDefault();
        if (favs.length > 0) setCol("fav");
      } else if (e.key === "ArrowLeft") {
        e.preventDefault();
        setCol("hist");
      } else if (e.key === "Enter") {
        e.preventDefault();
        const cmd = col === "hist" ? items[selH] : favs[selF];
        if (cmd) insert(cmd);
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [items, favs, col, selH, selF, onClose]);

  useEffect(() => {
    const ref = col === "hist" ? listRef : favRef;
    ref.current?.children[col === "hist" ? selH : selF]?.scrollIntoView({ block: "nearest" });
  }, [selH, selF, col]);

  const rowCls = (active: boolean) =>
    `group flex cursor-pointer items-center gap-2 px-3 py-1.5 font-mono text-[12px] ${
      active ? "bg-[var(--accent-dim)] text-[var(--text)]" : "text-[var(--text-dim)]"
    }`;

  return (
    <div
      className="animate-fade-up fixed right-6 top-14 z-[100] flex overflow-hidden rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] shadow-2xl"
      onMouseDown={(e) => e.stopPropagation()}
    >
      {/* 历史列 */}
      <div className="flex w-[320px] flex-col border-r border-[var(--border)]">
        <div className="flex items-center justify-between border-b border-[var(--border)] px-3 py-1.5 text-[11px] text-[var(--text-faint)]">
          <span className={col === "hist" ? "text-[var(--accent)]" : ""}>指令历史</span>
          <span>↑↓ 选择 · ←→ 切列 · Enter 插入 · Esc 关闭</span>
        </div>
        <div ref={listRef} className="max-h-[320px] min-h-[120px] overflow-y-auto py-1">
          {items.length === 0 ? (
            <div className="px-4 py-6 text-center text-[12px] text-[var(--text-faint)]">暂无历史命令</div>
          ) : (
            items.map((cmd, i) => (
              <div
                key={i}
                onMouseEnter={() => { setCol("hist"); setSelH(i); }}
                onClick={() => insert(cmd)}
                className={rowCls(col === "hist" && i === selH)}
              >
                <span className="w-5 shrink-0 text-right text-[10px] text-[var(--text-faint)]">{i + 1}</span>
                <span className="min-w-0 flex-1 truncate">{cmd}</span>
                <button
                  className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100"
                  title={favs.includes(cmd) ? "取消收藏" : "收藏"}
                  onClick={(e) => { e.stopPropagation(); toggleFav(cmd); }}
                >
                  <FiStar
                    size={12}
                    className={favs.includes(cmd) ? "text-[var(--accent)]" : "text-[var(--text-faint)] hover:text-[var(--accent)]"}
                    fill={favs.includes(cmd) ? "currentColor" : "none"}
                  />
                </button>
              </div>
            ))
          )}
        </div>
      </div>

      {/* 收藏列 */}
      <div className="flex w-[220px] flex-col">
        <div className="border-b border-[var(--border)] px-3 py-1.5 text-[11px] text-[var(--text-faint)]">
          <span className={col === "fav" ? "text-[var(--accent)]" : ""}>收藏指令</span>
        </div>
        <div ref={favRef} className="max-h-[320px] min-h-[120px] overflow-y-auto py-1">
          {favs.length === 0 ? (
            <div className="px-4 py-6 text-center text-[11px] leading-relaxed text-[var(--text-faint)]">
              在左侧历史行上
              <br />
              点击 ☆ 收藏指令
            </div>
          ) : (
            favs.map((cmd, i) => (
              <div
                key={i}
                onMouseEnter={() => { setCol("fav"); setSelF(i); }}
                onClick={() => insert(cmd)}
                className={rowCls(col === "fav" && i === selF)}
              >
                <span className="min-w-0 flex-1 truncate">{cmd}</span>
                <button
                  className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100"
                  title="删除收藏"
                  onClick={(e) => {
                    e.stopPropagation();
                    const next = favs.filter((_, idx) => idx !== i);
                    setFavs(next);
                    saveFavorites(next);
                  }}
                >
                  <svg width="10" height="10" viewBox="0 0 12 12">
                    <path d="M2 2 L10 10 M10 2 L2 10" stroke="currentColor" strokeWidth="1.6" />
                  </svg>
                </button>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
