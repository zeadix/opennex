import { useI18n } from '../i18n-context';
import { useEffect, useState } from "react";
import { FiCopy, FiTrash2 } from "react-icons/fi";
import { invoke } from "../terminal/tauri";
import { focusedSlot } from "../terminal/registry";

interface HistEntry {
  id: number;
  cmd: string;
  hits: number;
}

/** 历史指令按【终端】隔离:本页展示当前聚焦终端自己的历史。
 *  会话由布局恢复重建(槽位号稳定),历史随终端走;关闭终端即清。 */
export default function HistoryPage() {
  // registry 值变化不触发重渲;挂载时取当前聚焦终端即可(页签打开时刻的终端)。
  const [sessionId] = useState(() => focusedSlot.value ?? 0);
  return <TerminalHistoryPage key={sessionId} sessionId={sessionId} />;
}

function TerminalHistoryPage({ sessionId }: { sessionId: number }) {
  const T = useI18n();
  const [items, setItems] = useState<HistEntry[]>([]);
  const [copied, setCopied] = useState<number | null>(null);

  useEffect(() => {
    let stale = false;
    if (!sessionId) return;
    invoke<HistEntry[]>("get_history", { sessionId: String(sessionId) })
      .then((list) => { if (!stale) setItems(list); })
      .catch(() => {});
    return () => { stale = true; };
  }, [sessionId]);

  const copy = async (e: HistEntry) => {
    await navigator.clipboard.writeText(e.cmd);
    setCopied(e.id);
    setTimeout(() => setCopied(null), 1200);
  };
  const remove = (e: HistEntry) => {
    invoke("delete_history", { sessionId: String(sessionId), id: e.id }).catch(() => {});
    setItems((prev) => prev.filter((x) => x.id !== e.id));
  };

  return (
    <div className="h-full min-w-0 overflow-y-auto p-3">
      <div className="w-full min-w-0">
        <h2 className="mb-4 text-[15px] font-semibold">{T.history}</h2>
        {items.length === 0 ? (
          <div className="rounded-lg border border-dashed border-[var(--border)] py-12 text-center text-[12px] text-[var(--text-faint)]">
            {T.historyEmpty}
          </div>
        ) : (
          <div className="space-y-1">
            {items.map((e) => (
              <div
                key={e.id}
                className="group flex items-center gap-3 rounded-md border border-transparent px-3 py-2 transition-colors hover:border-[var(--border)] hover:bg-[var(--bg-hover)]"
              >
                <span className="min-w-0 flex-1 truncate font-mono text-[12px] text-[var(--text-dim)]">{e.cmd}</span>
                <span
                  className="shrink-0 font-mono text-[10px] text-[var(--text-faint)]"
                  title={T.uRunCount}
                >
                  ×{e.hits}
                </span>
                <button
                  className="icon-btn opacity-0 group-hover:opacity-100"
                  title={T.copy}
                  onClick={() => copy(e)}
                >
                  <FiCopy size={13} />
                </button>
                <button
                  className="icon-btn opacity-0 group-hover:opacity-100 hover:!text-[var(--danger)]"
                  title={T.uDeleteRecord}
                  onClick={() => remove(e)}
                >
                  <FiTrash2 size={13} />
                </button>
                {copied === e.id && (
                  <span className="shrink-0 text-[11px] text-[var(--success)]">{T.uCopied}</span>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
