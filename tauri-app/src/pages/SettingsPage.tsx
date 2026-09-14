import { useEffect, useState } from "react";
import { THEMES } from "../theme/themes";
import { Settings } from "../settings";

export default function SettingsPage({
  settings,
  onSettings,
  themeId,
  onTheme,
}: {
  settings: Settings;
  onSettings: (patch: Partial<Settings>) => void;
  themeId: string;
  onTheme: (id: string) => void;
}) {
  const [shells, setShells] = useState<string[]>([]);
  useEffect(() => {
    import("@tauri-apps/api/core").then((m) => m.invoke<string[]>("list_shells")).then(setShells).catch(() => {});
  }, []);

  return (
    <div className="h-full overflow-y-auto px-8 py-6">
      <div className="mx-auto max-w-[560px] space-y-8">
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
