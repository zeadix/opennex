import { useEffect, useState } from "react";
import { invoke } from "../terminal/tauri";

interface UpdateResult {
  updateAvailable: boolean;
  latest: string;
  current: string;
  changes: string[];
  changesEn: string[];
}

/** Update checker: compares against the public manifest and lists the
 * new release's bilingual notes. (Download/install comes with the
 * Tauri updater integration.) */
export default function UpdatePage({ lang }: { lang: "zh" | "en" }) {
  const [result, setResult] = useState<UpdateResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [checking, setChecking] = useState(true);

  const check = () => {
    setChecking(true);
    setError(null);
    invoke<UpdateResult>("check_update")
      .then(setResult)
      .catch((e) => setError(String(e)))
      .finally(() => setChecking(false));
  };

  useEffect(check, []);

  const notes = result
    ? lang === "en"
      ? result.changesEn.filter((c) => c.length > 0)
      : result.changes
    : [];

  return (
    <div className="h-full overflow-y-auto px-8 py-6">
      <div className="mx-auto max-w-[560px]">
        <h2 className="mb-4 text-[15px] font-semibold">检查更新</h2>
        {checking && <div className="text-[13px] text-[var(--text-dim)]">检查中…</div>}
        {error && <div className="text-[13px] text-[var(--danger)]">{error}</div>}
        {result && !checking && (
          <div className="space-y-4">
            <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
              <div className="flex items-center gap-3">
                <span className="text-[12px] text-[var(--text-dim)]">当前版本</span>
                <span className="font-mono text-[13px]">v{result.current}</span>
              </div>
              <div className="mt-1 flex items-center gap-3">
                <span className="text-[12px] text-[var(--text-dim)]">最新版本</span>
                <span className="font-mono text-[13px] text-[var(--accent)]">v{result.latest}</span>
                <span
                  className={`rounded-full px-2 py-0.5 text-[11px] font-semibold ${
                    result.updateAvailable
                      ? "bg-[var(--accent-dim)] text-[var(--accent)]"
                      : "bg-[var(--bg-active)] text-[var(--text-dim)]"
                  }`}
                >
                  {result.updateAvailable
                    ? lang === "zh" ? "有新版本" : "Update available"
                    : lang === "zh" ? "已是最新" : "Up to date"}
                </span>
              </div>
            </div>
            {notes.length > 0 && (
              <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
                <div className="mb-2 text-[12px] font-semibold text-[var(--text-dim)]">
                  {lang === "zh" ? "更新内容" : "What's new"}
                </div>
                <ul className="space-y-1.5">
                  {notes.map((c, i) => (
                    <li key={i} className="text-[13px] leading-relaxed text-[var(--text)]">
                      {c}
                    </li>
                  ))}
                </ul>
              </div>
            )}
            <button
              className="rounded-md border border-[var(--border)] px-3 py-1.5 text-[12px] text-[var(--text-dim)] hover:text-[var(--text)]"
              onClick={check}
            >
              重新检查
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
