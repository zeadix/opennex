import type { Settings } from "../settings";
import { t, type Lang } from "../i18n";

const OPTIONS = [
  { key: "autoMatch", labelKey: "sOptAutoMatch" },
  { key: "followCursor", labelKey: "sOptFollowCursor" },
  { key: "copyOnSelect", labelKey: "sOptCopySelect" },
] as const;

export default function QuickSettingsPage({ settings, onSettings, lang }: {
  settings: Settings;
  onSettings: (patch: Partial<Settings>) => void;
  lang: Lang;
}) {
  const T = t(lang);
  return (
    <div className="h-full min-w-0 overflow-y-auto p-3">
      <h2 className="mb-3 text-[15px] font-semibold">{T.quickSettings}</h2>
      <div className="divide-y divide-[var(--border)]">
        {OPTIONS.map(({ key, labelKey }) => (
          <div key={key}>
            <label className="flex cursor-pointer items-center gap-3 py-3 text-[12px] leading-relaxed text-[var(--text)]">
              <input
                type="checkbox"
                checked={settings[key]}
                onChange={(e) => onSettings({ [key]: e.target.checked })}
                className="switch shrink-0"
              />
              <span className="min-w-0">{T[labelKey as keyof ReturnType<typeof t>]}</span>
            </label>
            {/* 补全指令来源：仅"输入时自动匹配指令"开启时出现。紧凑行,
                与开关行标签对齐;自定义箭头,主题 token 配色 */}
            {key === "autoMatch" && settings.autoMatch && (
              <div className="flex items-center justify-between gap-2 pb-2 pl-[38px] pr-1 text-[11px]">
                <span className="shrink-0 text-[var(--text-dim)]">{T.sSuggestSource}</span>
                <div className="relative shrink-0">
                  <select
                    className="cursor-pointer appearance-none rounded border border-[var(--border)] bg-[var(--bg-elevated)] py-[3px] pl-2 pr-[18px] text-[11px] leading-[14px] text-[var(--text)] outline-none transition-colors hover:border-[var(--text-dim)] focus:border-[var(--accent)]"
                    value={settings.suggestSource ?? "auto"}
                    onChange={(e) =>
                      onSettings({ suggestSource: e.target.value as Settings["suggestSource"] })
                    }
                  >
                    <option value="auto">{T.sSuggestAuto}</option>
                    <option value="system">{T.sSuggestSystem}</option>
                    <option value="history">{T.sSuggestHistory}</option>
                  </select>
                  <span
                    aria-hidden="true"
                    className="pointer-events-none absolute right-[5px] top-1/2 -translate-y-1/2 text-[8px] leading-none text-[var(--text-faint)]"
                  >
                    ▾
                  </span>
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
