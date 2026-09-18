import { useI18n } from '../i18n-context';
import { useEffect, useRef, useState } from "react";
import { FiFolder, FiPlus, FiStar, FiTrash2, FiEdit2, FiX } from "react-icons/fi";
import { invoke } from "./tauri";
import { focusedSlot, lastCursor, sendTo } from "./registry";
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
  const listRefs = useRef<Record<string, HTMLDivElement | null>>({});
  // Positioning: remembered > caret-follow (setting) > default top-right.
  const follow = loadSettings().followCursor;
  const [pos, setPos] = useState<{ x: number; y: number } | null>(() => {
    try {
      return JSON.parse(localStorage.getItem("opennex-palette-pos") ?? "null");
    } catch {
      return null;
    }
  });
  const dragPos = (e: React.MouseEvent) => {
    beginOverlayDrag(e, (x, y) => {
      const p = { x: Math.max(0, x), y: Math.max(0, y) };
      setPos(p);
      localStorage.setItem("opennex-palette-pos", JSON.stringify(p));
    });
  };
  const posStyle: React.CSSProperties | undefined =
    pos
      ? { left: pos.x, top: pos.y }
      : follow && lastCursor.x > 0
        ? {
            left: Math.max(4, Math.min(lastCursor.x, window.innerWidth - 700)),
            top:
              lastCursor.y + 240 > window.innerHeight
                ? Math.max(4, lastCursor.y - 240)
                : lastCursor.y + 24,
          }
        : undefined;

  useEffect(() => {
    let stale = false;
    invoke<HistEntry[]>("get_history", { workspaceId })
      .then((list) => { if (!stale) setHist(list); })
      .catch(() => { if (!stale) setHist([]); });
    return () => { stale = true; };
  }, [workspaceId]);

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
      className={`animate-fade-up fixed z-[6000] flex overflow-hidden rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] shadow-2xl ${
        posStyle ? "" : "right-6 top-14"
      }`}
      style={posStyle}
      onMouseDown={(e) => e.stopPropagation()}
    >
      {/* ── 指令历史 ── */}
      <div className="flex w-[300px] flex-col border-r border-[var(--border)]">
        <div
          className={`flex items-center justify-between border-b border-[var(--border)] px-3 py-1.5 ${
            pos || follow ? "" : "cursor-move"
          }`}
          title={!follow ? T.uDragPosition : undefined}
          onMouseDown={(e) => {
            if (follow) return;
            e.stopPropagation();
            dragPos(e);
          }}
        >
          <span className={colHeadCls("hist")}>{T.cmdHistory}</span>
          <span className="text-[10px] text-[var(--text-faint)]">{T.uHistoryKeys}</span>
        </div>
        <div
          ref={(el) => (listRefs.current.hist = el)}
          className="max-h-[320px] min-h-[140px] overflow-y-auto py-1"
        >
          {hist.length === 0 ? (
            <div className="px-4 py-6 text-center text-[12px] text-[var(--text-faint)]">{T.uNoHistory}</div>
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
                  <FiStar size={12} className="text-[var(--text-faint)] hover:text-[var(--accent)]" />
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
