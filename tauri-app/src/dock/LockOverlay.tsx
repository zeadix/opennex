import { useI18n } from '../i18n-context';
import { useState } from "react";
import { FiLock } from "react-icons/fi";

/** Full-cover unlock overlay for a locked workspace. Renders an in-pane
 * password box; errors shake + red. `mode` is "unlock" (verify against
 * stored hash) or "set" (first-time, stores the hash).
 * 视觉对齐官网设计稿：琥珀色锁徽标 + 斜纹遮罩 + 居中密码点。 */
export default function LockOverlay({
  mode,
  onSubmit,
}: {
  mode: "unlock" | "set";
  onSubmit: (password: string) => Promise<boolean>;
}) {
  const T = useI18n();
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
    <div
      className="absolute inset-0 z-30 flex flex-col items-center justify-center gap-4 bg-[var(--bg)]/95"
      style={{
        backgroundImage:
          "radial-gradient(340px 150px at 50% 42%, rgba(217, 164, 65, 0.14), transparent 75%), repeating-linear-gradient(45deg, rgba(255, 255, 255, 0.02) 0 10px, transparent 10px 20px)",
        backdropFilter: "blur(2px)",
      }}
    >
      <div
        className="flex h-14 w-14 items-center justify-center rounded-2xl border animate-[lock-glow_2.4s_ease-in-out_infinite]"
        style={{
          borderColor: "rgba(217, 164, 65, 0.55)",
          background: "rgba(217, 164, 65, 0.14)",
        }}
      >
        <FiLock size={24} style={{ color: "#d9a441" }} />
      </div>
      <div className="text-[15px] font-semibold">
        {mode === "set" ? T.uSetLockPassword : T.uWorkspaceLocked}
      </div>
      <div className={`flex items-center gap-2 ${shake ? "animate-[shake_0.35s]" : ""}`}>
        <input
          autoFocus
          type="password"
          value={pwd}
          onChange={(e) => {
            setPwd(e.target.value);
            setError(false);
          }}
          onKeyDown={(e) => e.key === "Enter" && submit()}
          className="w-60 rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] px-3 py-2 text-center text-[14px] tracking-[0.4em] outline-none transition-colors focus:border-[#d9a441]"
          style={{ userSelect: "text" }}
          placeholder="••••••"
        />
        <button
          className="rounded-md px-4 py-2 text-[12px] font-semibold text-[#17191c] transition-all hover:brightness-110"
          style={{ background: "#d9a441" }}
          onClick={submit}
        >
          {mode === "set" ? T.set : T.uUnlockAction}
        </button>
      </div>
      {error && <div className="text-[12px] text-[var(--danger)]">{T.uPasswordError}</div>}
      <style>{`
        @keyframes shake { 0%,100%{transform:translateX(0)} 25%{transform:translateX(-6px)} 75%{transform:translateX(6px)} }
        @keyframes lock-glow { 0%,100%{ box-shadow: 0 0 14px rgba(217, 164, 65, 0.35); } 50%{ box-shadow: 0 0 30px rgba(217, 164, 65, 0.55); } }
      `}</style>
    </div>
  );
}
