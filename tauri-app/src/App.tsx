import { useState } from "react";
import Sidebar, { Page } from "./components/Sidebar";
import StatusBar from "./components/StatusBar";
import TabStrip, { Tab } from "./components/TabStrip";
import TerminalPane from "./terminal/TerminalPane";
import { FiTool } from "react-icons/fi";

let nextId = 1;

function Placeholder({ label }: { label: string }) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2 text-[var(--text-faint)]">
      <FiTool size={28} />
      <div className="text-sm">{label} · 开发中</div>
    </div>
  );
}

export default function App() {
  const [page, setPage] = useState<Page>("terminal");
  const [tabs, setTabs] = useState<Tab[]>([{ id: nextId, title: "bash 1" }]);
  const [active, setActive] = useState(nextId);
  const shell = (() => {
    try {
      return (window as any).__OPENNEX_SHELL__ ?? "bash";
    } catch {
      return "bash";
    }
  })();

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
    <div className="flex h-full">
      <Sidebar page={page} onNavigate={setPage} />
      <div className="flex min-w-0 flex-1 flex-col">
        {page === "terminal" ? (
          <>
            <TabStrip
              tabs={tabs}
              active={active}
              onSelect={setActive}
              onClose={closeTab}
              onNew={newTab}
            />
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
            <StatusBar sessionCount={tabs.length} shell={shell} />
          </>
        ) : page === "ssh" ? (
          <Placeholder label="SSH 主机管理" />
        ) : page === "history" ? (
          <Placeholder label="指令历史" />
        ) : page === "ai" ? (
          <Placeholder label="AI 助手" />
        ) : (
          <Placeholder label="设置" />
        )}
      </div>
    </div>
  );
}
