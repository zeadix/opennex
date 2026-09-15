import { useState } from "react";
import { FiLock } from "react-icons/fi";

/** Full-cover unlock overlay for a locked workspace. Renders an in-pane
 * password box; errors shake + red. `mode` is "unlock" (verify against
 * stored hash) or "set" (first-time, stores the hash). */
export default function LockOverlay({
  mode,
  onSubmit,
}: {
  mode: "unlock" | "set";
  onSubmit: (password: string) => Promise<boolean>;
}) {
  const [pwd, setPwd] = useState("");
  const [error, setError] = useState(false);
  const [shake, setShake] = useState(false);

  const fail = () => {
    setError(true);
    setShake(true);
    setTimeout(() => setShake(false), 400);
  };

  const submit = async () => {
    if (mode === "set" && pwd.length < 4) return fail();
    const ok = await onSubmit(pwd);
    if (!ok) fail();
  };

  return (
    <div className="absolute inset-0 z-30 flex flex-col items-center justify-center gap-4 bg-[var(--bg)]/95 backdrop-blur-sm">
      <div
        className="glow-pulse flex h-14 w-14 items-center justify-center rounded-2xl border border-[var(--border)] bg-[var(--bg-elevated)]"
        style={{ boxShadow: "0 0 24px var(--accent-dim)" }}
      >
        <FiLock size={24} className="text-[var(--accent)]" />
      </div>
      <div className="text-[15px] font-semibold">
        {mode === "set" ? "设置锁定密码（至少 4 位）" : "工作空间已锁定"}
      </div>
      <div
        className={`flex items-center gap-2 ${shake ? "animate-[shake_0.35s]" : ""}`}
      >
        <input
          autoFocus
          type="password"
          value={pwd}
          onChange={(e) => {
            setPwd(e.target.value);
            setError(false);
          }}
          onKeyDown={(e) => e.key === "Enter" && submit()}
          className="w-60 rounded-md border border-[var(--border)] bg-[var(--bg-panel)] px-3 py-2 text-[13px] outline-none transition-colors focus:border-[var(--accent)]"
          style={{ userSelect: "text" }}
          placeholder="密码"
        />
        <button
          className="rounded-md bg-[var(--accent-dim)] px-3 py-2 text-[12px] text-[var(--accent)] transition-all hover:brightness-125"
          onClick={submit}
        >
          {mode === "set" ? "设置" : "解锁"}
        </button>
      </div>
      {error && <div className="text-[12px] text-[var(--danger)]">密码错误或过短</div>}
      <style>{`@keyframes shake { 0%,100%{transform:translateX(0)} 25%{transform:translateX(-6px)} 75%{transform:translateX(6px)} }`}</style>
    </div>
  );
}
