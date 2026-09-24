import { useI18n } from '../i18n-context';
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { FiFolder, FiPlus, FiStar, FiTrash2, FiEdit2, FiX } from "react-icons/fi";
import { invoke } from "./tauri";
import { createPortal } from "react-dom";
import { cursorBySlot, cursorRefreshers, focusedSlot, sendTo } from "./registry";
import { beginOverlayDrag } from "./TerminalPane";
import { loadSettings } from "../settings";
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
interface HistoryOverlayProps {
  workspaceId: number;
  onClose: () => void;
}

export default function HistoryOverlay(props: HistoryOverlayProps) {
  return <WorkspaceHistoryOverlay key={props.workspaceId} {...props} />;
}

function WorkspaceHistoryOverlay({ workspaceId, onClose }: HistoryOverlayProps) {
  const T = useI18n();
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
  const [q, setQ] = useState("");
  const listRefs = useRef<Record<string, HTMLDivElement | null>>({});
  // 定位模型（设置「指令弹层跟随输入光标」）：
  //   开 = 每次呼出都锚在「聚焦终端」的输入光标处（光标行下方，贴底
  //        翻上方，贴边收进视口），不可拖拽；
  //   关 = 固定位置：顶部拖拽条可拖动，位置记忆到 localStorage，下次
  //        呼出仍在原位，不跟随光标。
  const follow = loadSettings().followCursor;
  const overlayRef = useRef<HTMLDivElement | null>(null);
  const [fixedPos, setFixedPos] = useState<{ x: number; y: number } | null>(() => {
    try {
      return JSON.parse(localStorage.getItem("opennex-palette-pos") ?? "null");
    } catch {
      return null;
    }
  });
  const [anchored, setAnchored] = useState<{ left: number; top: number } | null>(null);
  const anchoredRef = useRef(anchored);
  anchoredRef.current = anchored;
  useLayoutEffect(() => {
    if (!follow) {
      setAnchored(null);
      return;
    }
    // 打开时（及窗口尺寸变化时）刷新光标坐标再定位。
    const compute = () => {
      cursorRefreshers.forEach((fn) => fn());
      const el = overlayRef.current;

      // 多终端：取「聚焦终端」分桶里的光标（pane 本地坐标），用宿主
      // 矩形×缩放映射回视口。聚焦槽位若还没有光标数据（尚未输入），
      // 退回右上角默认位。
      const entry = cursorBySlot.get(focusedSlot.value);
      if (!el || !entry || entry.host.w <= 0) {
        setAnchored(null);
        return;
      }
      const zoom = entry.zoom || 1;
      const curX = entry.host.left + entry.x * zoom;
      const curTop = entry.host.top + entry.y * zoom;
      const lineH = (entry.h || 20) * zoom;
      const r = el.getBoundingClientRect();
      const vw = window.innerWidth;
      const vh = window.innerHeight;
      let left = Math.min(curX, vw - r.width - 4);
      if (left + r.width > vw - 4) left = curX - r.width;
      left = Math.max(4, Math.min(left, vw - r.width - 4));
      let top = curTop + lineH + 4;
      if (top + r.height > vh - 4) top = Math.max(4, curTop - r.height - 6);
      top = Math.max(4, Math.min(top, vh - r.height - 4));
      setAnchored({ left, top });
    };
    compute();
    window.addEventListener("resize", compute);
    // 面板打开时光标分桶可能还没就绪（终端尚未输入过/坐标未刷）：
    // 轮询重定位直到锚到光标，拿到后停止（resize 事件继续维护）。
    let settled = false;
    const poll = window.setInterval(() => {
      if (settled) return;
      compute();
      if (anchoredRef.current) {
        settled = true;
        window.clearInterval(poll);
      }
    }, 250);
    window.setTimeout(() => window.clearInterval(poll), 3000);
    return () => {
      window.removeEventListener("resize", compute);
      window.clearInterval(poll);
    };
  }, [follow]);

  const DEFAULT_POS: React.CSSProperties = { right: 24, top: 56 };
  const posStyle: React.CSSProperties = follow
    ? anchored ?? DEFAULT_POS
    : fixedPos
      ? { left: fixedPos.x, top: fixedPos.y }
      : DEFAULT_POS;

  useEffect(() => {
    let stale = false;
    invoke<HistEntry[]>("get_history", { workspaceId })
      .then((list) => { if (!stale) setHist(list); })
      .catch(() => { if (!stale) setHist([]); });
    return () => { stale = true; };
  }, [workspaceId]);

  // 设计稿（官网预览图）中的搜索：实时过滤历史行。
  const needle = q.trim().toLowerCase();
  const filtered = needle ? hist.filter((h) => h.cmd.toLowerCase().includes(needle)) : hist;

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
  const favSet = new Set(activeFolder()?.items ?? []);

  const addCmdToFolder = (folderId: string, cmd: string) => {
    saveFolders(
      folders.map((f) =>
        f.id === folderId ? { ...f, items: f.items.includes(cmd) ? f.items : [...f.items, cmd] } : f,
      ),
    );
  };

  const deleteHist = (id: number) => {
    invoke("delete_history", { workspaceId, id }).catch(() => {});
    setHist((prev) => prev.filter((e) => e.id !== id));
  };

  useEffect(() => {
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
        if (col === "hist") setSelH((s) => Math.min(filtered.length - 1, s + 1));
        else if (col === "folders") setSelF((s) => Math.min(folders.length - 1, s + 1));
        else setSelI((s) => Math.min((activeFolder()?.items.length ?? 1) - 1, s + 1));
      } else if (e.key === "ArrowRight") {
        e.preventDefault();
        if (col === "hist") {
          setCol(folders.length > 0 ? "folders" : filtered.length > 0 ? "hist" : "folders");
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
          const cmd = filtered[selH]?.cmd;
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
  }, [hist, filtered, q, folders, col, selH, selF, selI, prompt, addingFolder, onClose]);

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

  return createPortal(
    <div
      ref={overlayRef}
      className={`animate-fade-up fixed z-[9500] flex flex-col overflow-hidden rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] shadow-[var(--shadow-lift)] ${
        posStyle ? "" : "right-6 top-14"
      }`}
      style={posStyle}
      onMouseDown={(e) => e.stopPropagation()}
    >
      {/* ── 拖拽条（仅固定模式）：拖动改变面板位置，记忆到下次 ── */}
      {!follow && (
        <div
          className="overlay-grip"
          title={T.uDragPosition}
          onMouseDown={(e) => {
            e.stopPropagation();
            beginOverlayDrag(e, (x, y) => {
              const p = { x: Math.max(4, x), y: Math.max(4, y) };
              setFixedPos(p);
              localStorage.setItem("opennex-palette-pos", JSON.stringify(p));
            });
          }}
        >
          <svg width="22" height="6" aria-hidden="true">
            {[3, 11, 19].map((cx) => (
              <g key={cx}>
                <circle cx={cx} cy="1.5" r="1.2" fill="currentColor" />
                <circle cx={cx} cy="4.5" r="1.2" fill="currentColor" />
              </g>
            ))}
          </svg>
        </div>
      )}

      {/* ── 搜索（设计稿：⌕ 搜索历史指令…）── */}
      <div className="flex items-center gap-2 border-b border-[var(--border)] px-3 py-2">
        <svg viewBox="0 0 12 12" className="h-3 w-3 shrink-0 text-[var(--text-faint)]" aria-hidden="true">
          <circle cx="5" cy="5" r="3.4" fill="none" stroke="currentColor" strokeWidth="1.2" />
          <path d="M7.6 7.6L10.6 10.6" fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
        </svg>
        <input
          value={q}
          onChange={(e) => { setQ(e.target.value); setSelH(0); }}
          placeholder={T.searchPh}
          className="min-w-0 flex-1 bg-transparent text-[12px] text-[var(--text)] outline-none placeholder:text-[var(--text-faint)]"
          style={{ userSelect: "text" }}
          onMouseDown={(e) => e.stopPropagation()}
        />
      </div>

      <div className="flex">
      {/* ── 指令历史 ── */}
      <div className="flex w-[300px] flex-col border-r border-[var(--border)]">
        <div className="flex items-center justify-between border-b border-[var(--border)] px-3 py-1.5">
          <span className={colHeadCls("hist")}>{T.cmdHistory}</span>
          <span className="text-[10px] text-[var(--text-faint)]">{filtered.length}</span>
        </div>
        <div
          ref={(el) => (listRefs.current.hist = el)}
          className="max-h-[320px] min-h-[140px] overflow-y-auto py-1"
        >
          {filtered.length === 0 ? (
            <div className="px-4 py-6 text-center text-[12px] text-[var(--text-faint)]">{T.uNoHistory}</div>
          ) : (
            filtered.map((e, i) => (
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
                  className="shrink-0"
                  title={T.uFavoriteSelected}
                  onClick={(ev) => {
                    ev.stopPropagation();
                    let fid = activeFolder()?.id;
                    if (!fid) {
                      const f: FavFolder = { id: newFolderId(), name: T.uDefaultFolder, items: [] };
                      saveFolders([...folders, f]);
                      fid = f.id;
                    }
                    addCmdToFolder(fid, e.cmd);
                  }}
                >
                  <FiStar
                    size={12}
                    className={
                      favSet.has(e.cmd)
                        ? "text-[var(--accent)]"
                        : "text-[var(--text-faint)] opacity-60 transition-opacity group-hover:opacity-100 hover:text-[var(--accent)]"
                    }
                    style={favSet.has(e.cmd) ? { fill: "currentColor" } : undefined}
                  />
                </button>
                <button
                  className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100 hover:text-[var(--danger)]"
                  title={T.uDeleteRecord}
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
          <span className={colHeadCls("folders")}>{T.uFolders}</span>
          <button
            className="text-[var(--text-faint)] hover:text-[var(--accent)]"
            title={T.uNewFolder}
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
              placeholder={T.uFolderName}
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
              {T.uNewFolderHint}
              <br />
              {T.uDragHistory}
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
                  title={T.rename}
                  onClick={(ev) => {
                    ev.stopPropagation();
                    setPrompt({
                      title: T.uRenameFolder,
                      value: f.name,
                      onOk: (v) => saveFolders(folders.map((x) => (x.id === f.id ? { ...x, name: v } : x))),
                    });
                  }}
                >
                  <FiEdit2 size={11} />
                </button>
                <button
                  className="shrink-0 opacity-0 transition-opacity group-hover:opacity-100 hover:text-[var(--danger)]"
                  title={T.uDeleteFolder}
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
          <span className={colHeadCls("items")}>{activeFolder()?.name ?? T.uCommands}</span>
        </div>
        <div
          ref={(el) => (listRefs.current.items = el)}
          className="max-h-[320px] min-h-[140px] overflow-y-auto py-1"
        >
          {!activeFolder() || items.length === 0 ? (
            <div className="px-3 py-6 text-center text-[11px] leading-relaxed text-[var(--text-faint)]">
              {T.uOpenFolderHint}
              <br />
              {T.uDragToFolderHint}
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
                  title={T.uRemoveFromFolder}
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
      </div>

      {/* ── 底部键位提示（设计稿）── */}
      <div className="flex items-center justify-between border-t border-[var(--border)] bg-[var(--bg-panel)] px-3 py-1.5 text-[10px] text-[var(--text-faint)]">
        <span>{T.uHistoryKeys}</span>
        <span className="flex items-center gap-1.5">
          <FiStar size={9} className="text-[var(--accent)]" />
          {T.uDragHistory}
        </span>
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
    </div>,
    document.body,
  );
}
