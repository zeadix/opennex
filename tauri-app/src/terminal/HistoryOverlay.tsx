import { useEffect, useRef, useState } from "react";
import { FiFolder, FiPlus, FiStar, FiTrash2, FiEdit2, FiX } from "react-icons/fi";
import { invoke } from "./tauri";
import { focusedSlot, sendTo } from "./registry";
import { FavFolder, loadFolders, newFolderId, persistFolders } from "../favorites";
import PromptDialog from "../components/PromptDialog";

interface HistEntry {
  id: number;
  cmd: string;
  hits: number;
}

type Col = "hist" | "folders" | "items";

/**
 * The manual command palette (egui parity), three columns:
 * 1. 指令历史 — click/Enter inserts, ☆ adds to the selected folder,
 *    × deletes the record, rows DRAG into a folder.
 * 2. 收藏夹 — create/rename/delete folders; a drop target for history
 *    rows; Enter opens the folder.
 * 3. 指令 — the selected folder's commands; click/Enter inserts, × removes.
 * Insert puts the command on the terminal's input line WITHOUT executing.
 */
export default function HistoryOverlay({ onClose }: { onClose: () => void }) {
  const [hist, setHist] = useState<HistEntry[]>([]);
  const [folders, setFolders] = useState<FavFolder[]>(() => loadFolders());
  const [col, setCol] = useState<Col>("hist");
  const [selH, setSelH] = useState(0);
  const [selF, setSelF] = useState(0);
  const [selI, setSelI] = useState(0);
  const [dragOver, setDragOver] = useState<string | null>(null);
  const [prompt, setPrompt] = useState<{ title: string; value?: string; onOk: (v: string) => void } | null>(null);
  const [addingFolder, setAddingFolder] = useState(false);
  const [newFolderName, setNewFolderName] = useState("");
  const listRefs = useRef<Record<string, HTMLDivElement | null>>({});

  useEffect(() => {
    invoke<HistEntry[]>("get_history")
      .then((list) => setHist(list))
      .catch(() => setHist([]));
  }, []);

  const saveFolders = (next: FavFolder[]) => {
    setFolders(next);
    persistFolders(next);
  };

  const insert = (cmd: string) => {
    sendTo(focusedSlot.value, new TextEncoder().encode(cmd));
    window.dispatchEvent(
      new CustomEvent("opennex-line-set", { detail: { slot: focusedSlot.value, text: cmd } }),
    );
    onClose();
  };

  const activeFolder = () => folders[Math.min(selF, folders.length - 1)] ?? null;

  const addCmdToFolder = (folderId: string, cmd: string) => {
    saveFolders(
      folders.map((f) =>
        f.id === folderId ? { ...f, items: f.items.includes(cmd) ? f.items : [...f.items, cmd] } : f,
      ),
    );
  };

  const deleteHist = (id: number) => {
    invoke("delete_history", { id }).catch(() => {});
    setHist((prev) => prev.filter((e) => e.id !== id));
  };

  useEffect(() => {
    const count = col === "hist" ? hist.length : col === "folders" ? folders.length : activeFolder()?.items.length ?? 0;
    if (count === 0) return;
    const onKey = (e: KeyboardEvent) => {
      // Capture-phase interception: handled keys must not also reach the
      // terminal (arrow keys would drive the shell history under us).
      const handled = ["Escape", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Enter"].includes(e.key);
      if (handled) e.stopPropagation();
      if (e.key === "Escape" && !prompt && !addingFolder) {
        e.preventDefault();
        onClose();
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        if (col === "hist") setSelH((s) => Math.max(0, s - 1));
        else if (col === "folders") setSelF((s) => Math.max(0, s - 1));
        else setSelI((s) => Math.max(0, s - 1));
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        if (col === "hist") setSelH((s) => Math.min(hist.length - 1, s + 1));
        else if (col === "folders") setSelF((s) => Math.min(folders.length - 1, s + 1));
        else setSelI((s) => Math.min((activeFolder()?.items.length ?? 1) - 1, s + 1));
      } else if (e.key === "ArrowRight") {
        e.preventDefault();
        if (col === "hist") {
          setCol(folders.length > 0 ? "folders" : hist.length > 0 ? "hist" : "folders");
        } else if (col === "folders" && (activeFolder()?.items.length ?? 0) > 0) {
          setCol("items");
        }
      } else if (e.key === "ArrowLeft") {
        e.preventDefault();
        if (col === "items") setCol("folders");
        else if (col === "folders") setCol("hist");
      } else if (e.key === "Enter") {
        e.preventDefault();
        if (col === "hist") {
          const cmd = hist[selH]?.cmd;
          if (cmd) insert(cmd);
        } else if (col === "folders") {
          if (folders[selF] && (folders[selF].items.length > 0 || true)) setCol("items");
        } else {
          const cmd = activeFolder()?.items[selI];
          if (cmd) insert(cmd);
        }
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hist, folders, col, selH, selF, selI, prompt, addingFolder, onClose]);

  useEffect(() => {
    const ref = col === "hist" ? listRefs.current.hist : col === "folders" ? listRefs.current.folders : listRefs.current.items;
    const sel = col === "hist" ? selH : col === "folders" ? selF : selI;
    ref?.children[sel]?.scrollIntoView({ block: "nearest" });
  }, [selH, selF, selI, col]);

  const colHeadCls = (c: Col) => `text-[11px] ${col === c ? "text-[var(--accent)]" : "text-[var(--text-faint)]"}`;
  const rowCls = (c: Col, active: boolean) =>
    `group flex cursor-pointer items-center gap-2 px-3 py-1.5 font-mono text-[12px] ${
      active ? "bg-[var(--accent-dim)] text-[var(--text)]" : "text-[var(--text-dim)]"
    }`;

  const items = activeFolder()?.items ?? [];

  return (
    <div
      className="animate-fade-up fixed right-6 top-14 z-[6000] flex overflow-hidden rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] shadow-2xl"
      onMouseDown={(e) => e.stopPropagation()}
    >
      {/* ── 指令历史 ── */}
      <div className="flex w-[300px] flex-col border-r border-[var(--border)]">
        <div className="flex items-center justify-between border-b border-[var(--border)] px-3 py-1.5">
          <span className={colHeadCls("hist")}>指令历史</span>
          <span className="text-[10px] text-[var(--text-faint)]">↑↓ 选择 · Enter 插入 · Esc 关闭</span>
        </div>
        <div
          ref={(el) => (listRefs.current.hist = el)}
          className="max-h-[320px] min-h-[140px] overflow-y-auto py-1"
        >
          {hist.length === 0 ? (
            <div className="px-4 py-6 text-center text-[12px] text-[var(--text-faint)]">暂无历史命令</div>
          ) : (
            hist.map((e, i) => (
              <div
                key={e.id}
                draggable
                onDragStart={(ev) => ev.dataTransfer.setData("text/opennex-cmd", e.cmd)}
                onMouseEnter={() => { setCol("hist"); setSelH(i); }}
                onClick={() => insert(e.cmd)}
                className={rowCls("hist", col === "hist" && i === selH)}
              >
                <span className="w-5 shrink-0 text-right text-[10px] text-[var(--text-faint)]">{i + 1}</span>
                <span className="min-w-0 flex-1 truncate">{e.cmd}</span>
                <button
                  className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100"
                  title="收藏到选中收藏夹"
                  onClick={(ev) => {
                    ev.stopPropagation();
                    let fid = activeFolder()?.id;
                    if (!fid) {
                      const f: FavFolder = { id: newFolderId(), name: "收藏", items: [] };
                      saveFolders([...folders, f]);
                      fid = f.id;
                    }
                    addCmdToFolder(fid, e.cmd);
                  }}
                >
                  <FiStar size={12} className="text-[var(--text-faint)] hover:text-[var(--accent)]" />
                </button>
                <button
                  className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100 hover:text-[var(--danger)]"
                  title="删除该记录"
                  onClick={(ev) => { ev.stopPropagation(); deleteHist(e.id); }}
                >
                  <FiX size={12} />
                </button>
              </div>
            ))
          )}
        </div>
      </div>

      {/* ── 收藏夹 ── */}
      <div className="flex w-[170px] flex-col border-r border-[var(--border)]">
        <div className="flex items-center justify-between border-b border-[var(--border)] px-3 py-1.5">
          <span className={colHeadCls("folders")}>收藏夹</span>
          <button
            className="text-[var(--text-faint)] hover:text-[var(--accent)]"
            title="新建收藏夹"
            onClick={() => { setAddingFolder(true); setNewFolderName(""); }}
          >
            <FiPlus size={12} />
          </button>
        </div>
        {addingFolder && (
          <div className="border-b border-[var(--border)] p-2" onMouseDown={(e) => e.stopPropagation()}>
            <input
              autoFocus
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && newFolderName.trim()) {
                  const f: FavFolder = { id: newFolderId(), name: newFolderName.trim(), items: [] };
                  saveFolders([...folders, f]);
                  setAddingFolder(false);
                } else if (e.key === "Escape") setAddingFolder(false);
              }}
              placeholder="收藏夹名称"
              className="dialog-input !py-1 !text-[11px]"
            />
          </div>
        )}
        <div
          ref={(el) => (listRefs.current.folders = el)}
          className="max-h-[320px] min-h-[140px] overflow-y-auto py-1"
        >
          {folders.length === 0 ? (
            <div className="px-3 py-6 text-center text-[11px] leading-relaxed text-[var(--text-faint)]">
              点击 + 新建收藏夹
              <br />
              或把历史命令拖进来
            </div>
          ) : (
            folders.map((f, i) => (
              <div
                key={f.id}
                onClick={() => { setCol("folders"); setSelF(i); }}
                onDoubleClick={() => setCol("items")}
                onDragOver={(ev) => { ev.preventDefault(); setDragOver(f.id); }}
                onDragLeave={() => setDragOver((d) => (d === f.id ? null : d))}
                onDrop={(ev) => {
                  ev.preventDefault();
                  setDragOver(null);
                  const cmd = ev.dataTransfer.getData("text/opennex-cmd");
                  if (cmd) addCmdToFolder(f.id, cmd);
                }}
                onMouseEnter={() => { setCol("folders"); setSelF(i); }}
                className={`group flex cursor-pointer items-center gap-2 px-3 py-1.5 text-[12px] ${
                  dragOver === f.id
                    ? "bg-[var(--accent-dim)] text-[var(--accent)]"
                    : col === "folders" && i === selF
                      ? "bg-[var(--accent-dim)] text-[var(--text)]"
                      : "text-[var(--text-dim)]"
                }`}
              >
                <FiFolder size={12} className="shrink-0 text-[var(--text-faint)]" />
                <span className="min-w-0 flex-1 truncate">{f.name}</span>
                <span className="shrink-0 font-mono text-[10px] text-[var(--text-faint)]">{f.items.length}</span>
                <button
                  className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100 hover:text-[var(--accent)]"
                  title="重命名"
                  onClick={(ev) => {
                    ev.stopPropagation();
                    setPrompt({
                      title: "重命名收藏夹",
                      value: f.name,
                      onOk: (v) => saveFolders(folders.map((x) => (x.id === f.id ? { ...x, name: v } : x))),
                    });
                  }}
                >
                  <FiEdit2 size={11} />
                </button>
                <button
                  className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100 hover:text-[var(--danger)]"
                  title="删除收藏夹"
                  onClick={(ev) => {
                    ev.stopPropagation();
                    saveFolders(folders.filter((x) => x.id !== f.id));
                  }}
                >
                  <FiTrash2 size={11} />
                </button>
              </div>
            ))
          )}
        </div>
      </div>

      {/* ── 指令 ── */}
      <div className="flex w-[220px] flex-col">
        <div className="border-b border-[var(--border)] px-3 py-1.5">
          <span className={colHeadCls("items")}>{activeFolder()?.name ?? "指令"}</span>
        </div>
        <div
          ref={(el) => (listRefs.current.items = el)}
          className="max-h-[320px] min-h-[140px] overflow-y-auto py-1"
        >
          {!activeFolder() || items.length === 0 ? (
            <div className="px-3 py-6 text-center text-[11px] leading-relaxed text-[var(--text-faint)]">
              双击收藏夹查看指令
              <br />
              历史命令可拖拽进收藏夹
            </div>
          ) : (
            items.map((cmd, i) => (
              <div
                key={`${cmd}-${i}`}
                onMouseEnter={() => { setCol("items"); setSelI(i); }}
                onClick={() => insert(cmd)}
                className={rowCls("items", col === "items" && i === selI)}
              >
                <span className="min-w-0 flex-1 truncate">{cmd}</span>
                <button
                  className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100 hover:text-[var(--danger)]"
                  title="从收藏夹移除"
                  onClick={(ev) => {
                    ev.stopPropagation();
                    const fid = activeFolder()!.id;
                    saveFolders(
                      folders.map((f) =>
                        f.id === fid ? { ...f, items: f.items.filter((_, idx) => idx !== i) } : f,
                      ),
                    );
                  }}
                >
                  <FiX size={12} />
                </button>
              </div>
            ))
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
