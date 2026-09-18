import type { Settings } from "../settings";
import { t, type Lang } from "../i18n";

const OPTIONS = [
  { key: "autoMatch", labelKey: "sOptAutoMatch" },
  { key: "copyOnSelect", labelKey: "sOptCopySelect" },
  { key: "followCursor", labelKey: "sOptFollowCursor" },
] as const;

export default function QuickSettingsPage({ settings, onSettings, lang }: {
  settings: Settings;
  onSettings: (patch: Partial<Settings>) => void;
  lang: Lang;
}) {
  return (
    <div className="settings-controls h-full min-w-0 overflow-y-auto p-3">
      <h2 className="mb-3 text-[15px] font-semibold">{t(lang).quickSettings}</h2>
      <div className="divide-y divide-[var(--border)]">
        {OPTIONS.map(({ key, labelKey }) => (
          <label key={key} className="flex cursor-pointer items-start gap-3 py-3 text-[12px] leading-relaxed text-[var(--text)]">
            <input
              type="checkbox"
              checked={settings[key]}
              onChange={(e) => onSettings({ [key]: e.target.checked })}
              className="mt-0.5 shrink-0 accent-[var(--accent)]"
            />
            <span className="min-w-0">{t(lang)[labelKey as keyof ReturnType<typeof t>]}</span>
          </label>
        ))}
      </div>
    </div>
  );
}
