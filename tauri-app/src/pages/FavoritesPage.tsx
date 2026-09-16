import { useState } from "react";
import { FiEdit2, FiFolder, FiPlus, FiTrash2 } from "react-icons/fi";
import { focusedSlot, sendTo } from "../terminal/registry";
import { FavFolder, loadFolders, newFolderId, persistFolders } from "../favorites";
import PromptDialog from "../components/PromptDialog";

/** 收藏指令页：收藏夹（增删改名）+ 每个收藏夹内的指令（增删、点击
 * 插入聚焦终端，不自动执行）。 */
export default function FavoritesPage() {
  const [folders, setFolders] = useState<FavFolder[]>(() => loadFolders());
  const [activeId, setActiveId] = useState<string | null>(folders[0]?.id ?? null);
  const [buf, setBuf] = useState("");
  const [prompt, setPrompt] = useState<{ title: string; value?: string; onOk: (v: string) => void } | null>(null);

  const save = (next: FavFolder[]) => {
    setFolders(next);
    persistFolders(next);
  };
  const active = folders.find((f) => f.id === activeId) ?? folders[0] ?? null;

  const addItem = () => {
    const v = buf.trim();
    if (!v || !active) return;
    save(folders.map((f) => (f.id === active.id ? { ...f, items: [...f.items, v] } : f)));
    setBuf("");
  };
  const removeItem = (i: number) => {
    if (!active) return;
    save(folders.map((f) => (f.id === active.id ? { ...f, items: f.items.filter((_, idx) => idx !== i) } : f)));
  };
  const insert = (cmd: string) => {
    sendTo(focusedSlot.value, new TextEncoder().encode(cmd));
    window.dispatchEvent(
      new CustomEvent("opennex-line-set", { detail: { slot: focusedSlot.value, text: cmd } }),
    );
  };

  return (
    <div className="flex h-full">
      {/* 收藏夹列表 */}
      <div className="flex w-44 shrink-0 flex-col border-r border-[var(--border)] bg-[var(--bg-panel)] py-3">
        <div className="flex items-center justify-between px-3 pb-2">
          <span className="text-[11px] font-semibold tracking-wider text-[var(--text-faint)]">收藏夹</span>
          <button
            className="icon-btn !p-1"
            title="新建收藏夹"
            onClick={() =>
              setPrompt({
                title: "新建收藏夹",
                onOk: (v) => {
                  const f: FavFolder = { id: newFolderId(), name: v, items: [] };
                  save([...folders, f]);
                  setActiveId(f.id);
                },
              })
            }
          >
            <FiPlus size={13} />
          </button>
        </div>
        <div className="flex-1 overflow-y-auto px-2">
          {folders.length === 0 && (
            <div className="px-2 py-4 text-[11px] leading-relaxed text-[var(--text-faint)]">
              点击 + 新建收藏夹
            </div>
          )}
          {folders.map((f) => (
            <div
              key={f.id}
              onClick={() => setActiveId(f.id)}
              className={`group flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-[12px] transition-colors ${
                active?.id === f.id
                  ? "bg-[var(--accent-dim)] text-[var(--text)]"
                  : "text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
              }`}
            >
              <FiFolder size={12} className="shrink-0 text-[var(--text-faint)]" />
              <span className="min-w-0 flex-1 truncate">{f.name}</span>
              <span className="shrink-0 font-mono text-[10px] text-[var(--text-faint)]">{f.items.length}</span>
              <button
                className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100 hover:text-[var(--accent)]"
                title="重命名收藏夹"
                onClick={(e) => {
                  e.stopPropagation();
                  setPrompt({
                    title: "重命名收藏夹",
                    value: f.name,
                    onOk: (v) => save(folders.map((x) => (x.id === f.id ? { ...x, name: v } : x))),
                  });
                }}
              >
                <FiEdit2 size={11} />
              </button>
              <button
                className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100 hover:text-[var(--danger)]"
                title="删除收藏夹"
                onClick={(e) => {
                  e.stopPropagation();
                  const rest = folders.filter((x) => x.id !== f.id);
                  save(rest);
                  if (activeId === f.id) setActiveId(rest[0]?.id ?? null);
                }}
              >
                <FiTrash2 size={11} />
              </button>
            </div>
          ))}
        </div>
      </div>

      {/* 指令列表 */}
      <div className="min-w-0 flex-1 overflow-y-auto px-6 py-5">
        <div className="mx-auto max-w-[520px]">
          <h2 className="mb-3 text-[15px] font-semibold">{active ? active.name : "收藏指令"}</h2>
          {active && (
            <>
              <div className="mb-3 flex gap-2">
                <input
                  className="dialog-input font-mono"
                  placeholder="输入命令后回车添加"
                  value={buf}
                  onChange={(e) => setBuf(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && addItem()}
                  style={{ userSelect: "text" }}
                />
                <button
                  className="flex shrink-0 items-center gap-1 rounded-md border border-[var(--border)] px-2.5 text-[12px] hover:border-[var(--accent)] hover:text-[var(--accent)]"
                  onClick={addItem}
                >
                  <FiPlus size={13} /> 添加
                </button>
              </div>
              <div className="space-y-0.5">
                {active.items.length === 0 && (
                  <div className="py-6 text-center text-[12px] text-[var(--text-faint)]">
                    收藏夹为空 · 在 Alt 指令面板中把历史命令拖进来，或在上方添加
                  </div>
                )}
                {active.items.map((cmd, i) => (
                  <div
                    key={`${cmd}-${i}`}
                    onClick={() => insert(cmd)}
                    title="点击插入聚焦终端（不执行）"
                    className="group flex cursor-pointer items-center gap-2 rounded-md border border-transparent px-3 py-1.5 font-mono text-[12px] text-[var(--text-dim)] transition-colors hover:border-[var(--border)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
                  >
                    <span className="min-w-0 flex-1 truncate">{cmd}</span>
                    <button
                      className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100 hover:text-[var(--danger)]"
                      title="删除"
                      onClick={(e) => { e.stopPropagation(); removeItem(i); }}
                    >
                      <FiTrash2 size={12} />
                    </button>
                  </div>
                ))}
              </div>
            </>
          )}
        </div>
      </div>

      {prompt && (
        <PromptDialog
          title={prompt.title}
          defaultValue={prompt.value}
          onOk={(v) => {
            const fn = prompt.onOk;
            setPrompt(null);
            fn(v);
          }}
          onCancel={() => setPrompt(null)}
        />
      )}
    </div>
  );
}
