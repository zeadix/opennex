import { useEffect, useState } from "react";
import { FiCopy } from "react-icons/fi";
import { invoke } from "../terminal/tauri";

export default function HistoryPage() {
  const [items, setItems] = useState<string[]>([]);
  const [copied, setCopied] = useState<string | null>(null);

  useEffect(() => {
    invoke<string[]>("get_history").then(setItems).catch(() => {});
  }, []);

  const copy = async (cmd: string) => {
    await navigator.clipboard.writeText(cmd);
    setCopied(cmd);
    setTimeout(() => setCopied(null), 1200);
  };

  return (
    <div className="h-full overflow-y-auto px-8 py-6">
      <div className="mx-auto max-w-[640px]">
        <h2 className="mb-4 text-[15px] font-semibold">指令历史</h2>
        {items.length === 0 ? (
          <div className="rounded-lg border border-dashed border-[var(--border)] py-12 text-center text-[12px] text-[var(--text-faint)]">
            在终端里执行的命令会出现在这里
          </div>
        ) : (
          <div className="space-y-1">
            {items.map((cmd, i) => (
              <div
                key={i}
                className="group flex items-center gap-3 rounded-md border border-transparent px-3 py-2 transition-colors hover:border-[var(--border)] hover:bg-[var(--bg-hover)]"
              >
                <span className="font-mono text-[12px] text-[var(--text-dim)]">{cmd}</span>
                <button
                  className="icon-btn ml-auto opacity-0 group-hover:opacity-100"
                  title="复制"
                  onClick={() => copy(cmd)}
                >
                  <FiCopy size={13} />
                </button>
                {copied === cmd && (
                  <span className="text-[11px] text-[var(--success)]">已复制</span>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
