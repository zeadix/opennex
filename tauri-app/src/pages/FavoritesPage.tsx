import { useState } from "react";
import { FiPlay, FiPlus, FiStar, FiTrash2 } from "react-icons/fi";
import { focusedSlot, sendTo } from "../terminal/registry";

const KEY = "opennex-favorites";

export function loadFavorites(): string[] {
  try {
    return JSON.parse(localStorage.getItem(KEY) ?? "[]");
  } catch {
    return [];
  }
}
export function saveFavorites(list: string[]) {
  localStorage.setItem(KEY, JSON.stringify(list));
}

/** Snippets/favorites: stored commands, one click inserts into the
 * focused terminal (no auto-exec). */
export default function FavoritesPage() {
  const [items, setItems] = useState<string[]>(loadFavorites);
  const [adding, setAdding] = useState(false);
  const [buf, setBuf] = useState("");

  const add = () => {
    const v = buf.trim();
    if (!v) return;
    const next = [...items, v];
    setItems(next);
    saveFavorites(next);
    setBuf("");
    setAdding(false);
  };
  const remove = (i: number) => {
    const next = items.filter((_, idx) => idx !== i);
    setItems(next);
    saveFavorites(next);
  };
  const insert = (cmd: string) => {
    sendTo(focusedSlot.value, new TextEncoder().encode(cmd));
  };

  return (
    <div className="h-full overflow-y-auto px-8 py-6">
      <div className="mx-auto max-w-[560px]">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="flex items-center gap-2 text-[15px] font-semibold">
            <FiStar size={16} className="text-[var(--accent)]" /> 收藏指令
          </h2>
          <button
            onClick={() => setAdding(!adding)}
            className="flex items-center gap-1.5 rounded-md border border-[var(--border)] px-2.5 py-1.5 text-[12px] hover:border-[var(--accent)] hover:text-[var(--accent)]"
          >
            <FiPlus size={13} /> 添加
          </button>
        </div>

        {adding && (
          <div className="mb-4 space-y-2 rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
            <input
              autoFocus
              className="dialog-input font-mono"
              placeholder="命令内容"
              value={buf}
              onChange={(e) => setBuf(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && add()}
              style={{ userSelect: "text" }}
            />
            <div className="flex justify-end gap-2">
              <button className="rounded-md px-3 py-1.5 text-[12px] text-[var(--text-dim)] hover:text-[var(--text)]" onClick={() => setAdding(false)}>取消</button>
              <button className="rounded-md bg-[var(--accent-dim)] px-3 py-1.5 text-[12px] text-[var(--accent)] hover:brightness-125" onClick={add}>保存</button>
            </div>
          </div>
        )}

        <div className="space-y-1.5">
          {items.length === 0 && (
            <div className="rounded-lg border border-dashed border-[var(--border)] py-10 text-center text-[12px] text-[var(--text-faint)]">
              点击「添加」保存常用命令，点击条目插入到最近聚焦的终端
            </div>
          )}
          {items.map((cmd, i) => (
            <div
              key={i}
              className="group flex items-center gap-3 rounded-md border border-transparent px-3 py-2 transition-colors hover:border-[var(--border)] hover:bg-[var(--bg-hover)]"
            >
              <span className="min-w-0 flex-1 truncate font-mono text-[12px]">{cmd}</span>
              <button className="icon-btn opacity-0 group-hover:opacity-100" title="插入终端" onClick={() => insert(cmd)}>
                <FiPlay size={13} />
              </button>
              <button className="icon-btn opacity-0 group-hover:opacity-100 hover:!text-[var(--danger)]" title="删除" onClick={() => remove(i)}>
                <FiTrash2 size={13} />
              </button>
            </div>
          ))}
        </div>
        <div className="mt-3 text-[11px] text-[var(--text-faint)]">
          插入只写入命令行，按回车执行（可在终端内先修改参数）
        </div>
      </div>
    </div>
  );
}
