import { VscTerminalPowershell } from "react-icons/vsc";

export default function StatusBar({
  sessionCount,
  shell,
}: {
  sessionCount: number;
  shell: string;
}) {
  return (
    <div className="flex h-6 shrink-0 items-center gap-4 border-t border-[var(--border)] bg-[var(--bg-panel)] px-3 text-[11px] text-[var(--text-faint)]">
      <span className="flex items-center gap-1.5">
        <VscTerminalPowershell size={12} />
        {shell}
      </span>
      <span>会话 {sessionCount}</span>
      <span className="ml-auto">Tauri</span>
    </div>
  );
}
