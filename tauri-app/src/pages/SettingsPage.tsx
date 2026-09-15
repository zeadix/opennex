import { useEffect, useState } from "react";
import { FiLock, FiInfo } from "react-icons/fi";
import ShortcutRecorder from "../shortcuts/ShortcutRecorder";
import {
  SHORTCUT_ACTIONS,
  loadShortcuts,
  saveShortcuts,
  ShortcutAction,
} from "../shortcuts/shortcuts";
import { THEMES } from "../theme/themes";
import { Settings } from "../settings";
import { loadWorkspaces, persistWorkspaces, sha256, LOCK_SALT } from "../workspaces";

export default function SettingsPage({
  settings,
  onSettings,
  themeId,
  onTheme,
  lang,
}: {
  settings: Settings;
  onSettings: (patch: Partial<Settings>) => void;
  themeId: string;
  onTheme: (id: string) => void;
  lang: "zh" | "en";
}) {
  const [shells, setShells] = useState<string[]>([]);
  useEffect(() => {
    import("@tauri-apps/api/core").then((m) => m.invoke<string[]>("list_shells")).then(setShells).catch(() => {});
  }, []);

  // ---- shortcuts --------------------------------------------------------
  const [shortcuts, setShortcuts] = useState(loadShortcuts);
  const updateShortcut = (action: ShortcutAction, binding: string) => {
    const next = { ...shortcuts, [action]: binding };
    setShortcuts(next);
    saveShortcuts(next);
  };

  // ---- lock management -------------------------------------------------
  const workspaces = loadWorkspaces();
  const [lockPwd, setLockPwd] = useState("");
  const [lockMsg, setLockMsg] = useState<string | null>(null);
  const setAllLockPasswords = async () => {
    if (lockPwd.length < 4) return setLockMsg("密码至少 4 位");
    const hash = await sha256(LOCK_SALT + lockPwd);
    const list = loadWorkspaces();
    persistWorkspaces(list.map((w) => ({ ...w, lockHash: hash })));
    setLockPwd("");
    setLockMsg(`已为 ${list.length} 个工作空间设置锁定密码`);
  };
  const clearAllLocks = () => {
    persistWorkspaces(loadWorkspaces().map((w) => ({ ...w, locked: false, lockHash: undefined })));
    setLockMsg("已清除全部工作空间的锁定密码");
  };

  return (
    <div className="h-full overflow-y-auto px-8 py-6">
      <div className="mx-auto max-w-[560px] space-y-8">
        <section>
          <h2 className="mb-3 text-[15px] font-semibold">关于</h2>
          <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
            <div className="glow-text animate-fade-up text-[20px] font-bold tracking-wide">OpenNex</div>
            <div className="mt-1 text-[12px] text-[var(--text-dim)]">
              Tauri 版 · v0.1.55 · 终端工作台
            </div>
            <div className="mt-2 flex items-center gap-1.5 text-[11px] text-[var(--text-faint)]">
              <FiInfo size={12} /> Rust 后端 + xterm.js WebGL/Canvas 终端
            </div>
          </div>
        </section>

        <section>
          <h2 className="mb-3 text-[15px] font-semibold">快捷键</h2>
          <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
            <div className="space-y-2">
              {SHORTCUT_ACTIONS.map((a) => (
                <div key={a.id} className="flex items-center justify-between gap-3">
                  <span className="text-[12px] text-[var(--text-dim)]">
                    {lang === "zh" ? a.labelZh : a.labelEn}
                  </span>
                  <ShortcutRecorder
                    binding={shortcuts[a.id]}
                    onChange={(b) => updateShortcut(a.id, b)}
                  />
                </div>
              ))}
            </div>
            <button
              className="mt-3 rounded-md border border-[var(--border)] px-2.5 py-1 text-[11px] text-[var(--text-dim)] hover:text-[var(--text)]"
              onClick={() => setShortcuts(loadShortcuts())}
            >
              恢复默认
            </button>
          </div>
        </section>

        <section>
          <h2 className="mb-3 text-[15px] font-semibold">锁定</h2>
          <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
            <div className="mb-2 flex items-center gap-1.5 text-[12px] text-[var(--text-dim)]">
              <FiLock size={13} /> 为全部工作空间设置锁定密码
            </div>
            <div className="flex items-center gap-2">
              <input
                type="password"
                value={lockPwd}
                onChange={(e) => { setLockPwd(e.target.value); setLockMsg(null); }}
                placeholder="至少 4 位"
                className="dialog-input !w-52"
              />
              <button className="rounded-md bg-[var(--accent-dim)] px-3 py-1.5 text-[12px] text-[var(--accent)] hover:brightness-125"
                onClick={setAllLockPasswords}>设置</button>
              <button className="rounded-md border border-[var(--border)] px-3 py-1.5 text-[12px] text-[var(--text-dim)] hover:text-[var(--danger)]"
                onClick={clearAllLocks}>清除</button>
            </div>
            {lockMsg && <div className="mt-2 text-[11px] text-[var(--success)]">{lockMsg}</div>}
          </div>
        </section>

        <section>
          <h2 className="mb-3 text-[15px] font-semibold">外观</h2>
          <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
            <div className="mb-2 text-[12px] text-[var(--text-dim)]">主题</div>
            <div className="grid grid-cols-3 gap-2">
              {THEMES.map((t) => (
                <button
                  key={t.id}
                  onClick={() => onTheme(t.id)}
                  className={`flex items-center gap-2 rounded-md border px-3 py-2 text-[12px] transition-colors ${
                    themeId === t.id
                      ? "border-[var(--accent)] bg-[var(--bg-hover)]"
                      : "border-[var(--border)] hover:border-[var(--text-faint)]"
                  }`}
                >
                  <span
                    className="h-4 w-4 rounded-full border border-[var(--border)]"
                    style={{ background: t.colors.bg }}
                  />
                  {t.name}
                </button>
              ))}
            </div>
            <div className="mb-2 mt-5 text-[12px] text-[var(--text-dim)]">
              终端字号 · {settings.fontSize}px
            </div>
            <input
              type="range"
              min={10}
              max={24}
              value={settings.fontSize}
              onChange={(e) => onSettings({ fontSize: Number(e.target.value) })}
              className="w-full accent-[var(--accent)]"
            />
          </div>
        </section>

        <section>
          <h2 className="mb-3 text-[15px] font-semibold">默认 Shell</h2>
          <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
            {shells.length === 0 ? (
              <div className="text-[12px] text-[var(--text-faint)]">读取 /etc/shells 失败</div>
            ) : (
              <div className="space-y-1.5">
                {shells.map((s) => (
                  <label key={s} className="flex cursor-pointer items-center gap-2.5 text-[13px]">
                    <input
                      type="radio"
                      name="shell"
                      checked={settings.shell === s}
                      onChange={() => onSettings({ shell: s })}
                      className="accent-[var(--accent)]"
                    />
                    <span className="font-mono text-[12px]">{s}</span>
                  </label>
                ))}
              </div>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
