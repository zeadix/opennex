import { VscTerminalPowershell } from "react-icons/vsc";

export default function StatusBar({
  sessionCount,
  shell,
}: {
  sessionCount: number;
  shell: string;
}) {
  return (
    <div className="flex h-8 shrink-0 items-center gap-4 border-t-2 border-[var(--danger)] bg-[#7a1f1f] px-3 text-[13px] font-bold text-[#ffffff]">
      <span className="flex items-center gap-1.5">
        <VscTerminalPowershell size={12} />
        {shell}
      </span>
      <span>会话 {sessionCount}</span>
      <span className="ml-auto">Tauri</span>
    </div>
  );
}
