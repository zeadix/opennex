import { FiPlus, FiX } from "react-icons/fi";
import { VscTerminalPowershell } from "react-icons/vsc";

export interface Tab {
  id: number;
  title: string;
}

export default function TabStrip({
  tabs,
  active,
  onSelect,
  onClose,
  onNew,
}: {
  tabs: Tab[];
  active: number;
  onSelect: (id: number) => void;
  onClose: (id: number) => void;
  onNew: () => void;
}) {
  return (
    <div className="flex h-10 shrink-0 items-center gap-1 border-b border-[var(--border)] bg-[var(--bg-panel)] px-2">
      {tabs.map((t) => (
        <div
          key={t.id}
          onClick={() => onSelect(t.id)}
          className={`tab ${active === t.id ? "active" : ""}`}
        >
          <VscTerminalPowershell size={13} className="opacity-70" />
          <span>{t.title}</span>
          <span
            className="tab-close"
            onClick={(e) => {
              e.stopPropagation();
              onClose(t.id);
            }}
          >
            <FiX size={12} />
          </span>
        </div>
      ))}
      <button className="icon-btn" onClick={onNew} title="新建终端">
        <FiPlus size={15} />
      </button>
    </div>
  );
}

