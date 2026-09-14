import { useState } from "react";
import { FiPlus, FiX } from "react-icons/fi";
import TerminalPane from "./terminal/TerminalPane";

interface Tab {
  id: number;
  title: string;
}

let nextId = 1;

export default function App() {
  const [tabs, setTabs] = useState<Tab[]>([{ id: nextId, title: "bash 1" }]);
  const [active, setActive] = useState(nextId);

  const newTab = () => {
    nextId += 1;
    const t = { id: nextId, title: `bash ${nextId}` };
    setTabs((prev) => [...prev, t]);
    setActive(t.id);
  };
  const closeTab = (id: number) => {
    setTabs((prev) => {
      const next = prev.filter((t) => t.id !== id);
      if (next.length === 0) {
        nextId += 1;
        const t = { id: nextId, title: `bash ${nextId}` };
        setActive(t.id);
        return [t];
      }
      if (active === id) setActive(next[next.length - 1].id);
      return next;
    });
  };

  return (
    <div className="flex h-full flex-col overflow-hidden">
      {/* Tab strip */}
      <div className="flex h-9 shrink-0 items-end gap-1 border-b border-[var(--border)] bg-[var(--bg-panel)] px-2 pt-1.5">
        {tabs.map((t) => (
          <div
            key={t.id}
            onClick={() => setActive(t.id)}
            className={`group flex h-8 cursor-pointer items-center gap-2 rounded-t-md border-x border-t px-3 text-[13px] ${
              active === t.id
                ? "border-[var(--border)] bg-[var(--bg)] text-[var(--text)]"
                : "border-transparent bg-transparent text-[var(--text-dim)] hover:text-[var(--text)]"
            }`}
          >
            <span className="font-mono">{t.title}</span>
            <button
              className="rounded p-0.5 opacity-0 hover:bg-[var(--bg-hover)] group-hover:opacity-100"
              onClick={(e) => {
                e.stopPropagation();
                closeTab(t.id);
              }}
            >
              <FiX size={13} />
            </button>
          </div>
        ))}
        <button
          onClick={newTab}
          className="mb-1 rounded p-1.5 text-[var(--text-dim)] hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
          title="新建终端"
        >
          <FiPlus size={15} />
        </button>
      </div>
      {/* Terminal area */}
      <div className="relative min-h-0 flex-1">
        {tabs.map((t) => (
          <div
            key={t.id}
            className="absolute inset-0"
            style={{ display: active === t.id ? "block" : "none" }}
          >
            <TerminalPane sessionId={t.id} />
          </div>
        ))}
      </div>
    </div>
  );
}
