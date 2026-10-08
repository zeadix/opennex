import { checkForUpdates, installTauriUpdate, releaseNotes, useUpdates } from "../updates";
import { fmt } from "../i18n";
import { useI18n } from "../i18n-context";
import ReleaseNotes from "../components/ReleaseNotes";

const DOWNLOAD_PAGE = "https://opennex.zeadix.com/download";

export default function UpdatePage({ lang }: { lang: string }) {
  const T = useI18n();
  const { current, checking, result, error, tauri } = useUpdates();
  const available = tauri.available || !!result?.updateAvailable;
  const notes = tauri.available && tauri.notes
    ? tauri.notes.split("\n")
    : result ? releaseNotes(
      available ? result.changes : result.currentChanges,
      available ? result.changesEn : result.currentChangesEn,
      lang,
    ) : [];

  return (
    <div className="flex h-full min-w-0 flex-col p-3">
      <div className="min-h-0 flex-1 space-y-4 overflow-y-auto">
        <header className="space-y-2">
          <h2 className="text-[15px] font-semibold">{T.sSoftwareUpdate}</h2>
          <p className="text-[12px] text-[var(--text-dim)]">
            {T.currentVersion} <span className="font-mono text-[var(--text)]">v{current ?? "—"}</span>
          </p>
          <p role="status" className="text-[12px] text-[var(--text)]">
            {checking ? T.sChecking
              : tauri.installed ? T.sRestarting
              : tauri.available ? fmt(T.sNewFound, { v: tauri.version })
              : error ? T.sCheckFailed
              : available ? fmt(T.sNewFound, { v: result!.latest })
              : result ? T.sUpToDate
              : T.sNotChecked}
          </p>
        </header>
        {error && !tauri.available && <p role="alert" className="break-words text-[12px] text-[var(--danger)]">{error}</p>}
        {available && (
          <section className="space-y-2 rounded border border-[var(--border)] bg-[var(--bg-elevated)] p-3">
            {tauri.downloading ? (
              <div className="space-y-1.5">
                <p className="text-[12px] text-[var(--text)]">
                  {tauri.progress >= 0 ? fmt(T.sDownloadingPct, { p: tauri.progress }) : T.sDownloading}
                </p>
                <div className="h-1.5 w-full overflow-hidden rounded-full bg-[var(--border)]">
                  {tauri.progress >= 0
                    ? (
                      <div
                        className="h-full rounded-full bg-[var(--accent)] transition-[width] duration-200"
                        style={{ width: `${tauri.progress}%` }}
                      />
                    )
                    : (
                      <div className="h-full w-1/3 animate-pulse rounded-full bg-[var(--accent)]" />
                    )}
                </div>
              </div>
            ) : tauri.installed ? (
              <p className="text-[12px] text-[var(--success, var(--text))]">{T.sRestarting}</p>
            ) : !tauri.available ? (
              // 旧通道报有新版本但当前安装方式不能应用内更新（如 0.1.58 之前的
              // 老版本没带更新器插件）——给出官网下载兜底。
              <div className="space-y-1.5">
                <p className="text-[12px] text-[var(--text-dim)]">{T.sUpdateFallback}</p>
                <a
                  href={DOWNLOAD_PAGE}
                  target="_blank"
                  rel="noreferrer"
                  className="inline-block rounded border border-[var(--border)] px-3 py-1.5 text-[12px] hover:bg-[var(--bg-hover)]"
                >
                  {T.sGoDownload}
                </a>
              </div>
            ) : (
              <div className="flex items-center justify-between gap-2">
                <span className="text-[12px] text-[var(--text-dim)]">{T.sAutoUpdateHint}</span>
                <button
                  onClick={() => void installTauriUpdate()}
                  className="shrink-0 rounded border border-[var(--accent)] bg-[var(--accent)] px-3 py-1.5 text-[12px] font-medium text-white hover:opacity-90"
                >
                  {T.sInstallUpdate}
                </button>
              </div>
            )}
            {tauri.error && (
              <div className="space-y-1.5 border-t border-[var(--border)] pt-2">
                <p role="alert" className="break-words text-[12px] text-[var(--danger)]">{tauri.error}</p>
                <p className="text-[12px] text-[var(--text-dim)]">{T.sUpdateFallback}</p>
                <a
                  href={DOWNLOAD_PAGE}
                  target="_blank"
                  rel="noreferrer"
                  className="inline-block rounded border border-[var(--border)] px-3 py-1.5 text-[12px] hover:bg-[var(--bg-hover)]"
                >
                  {T.sGoDownload}
                </a>
              </div>
            )}
          </section>
        )}
        <section className="border-t border-[var(--border)] pt-3">
          <h3 className="mb-3 text-[12px] font-semibold">
            {tauri.available ? fmt(T.sWhatsNewV, { v: tauri.version })
              : available ? fmt(T.sWhatsNewV, { v: result!.latest })
              : T.sCurrentNotes}
          </h3>
          {checking && !result && !tauri.available ? <p className="text-[12px] text-[var(--text-dim)]">{T.sLoadingRelease}</p>
            : <ReleaseNotes lang={lang} notes={notes} />}
        </section>
      </div>
      <footer className="mt-3 flex shrink-0 justify-end border-t border-[var(--border)] pt-3">
        <button
          disabled={checking || tauri.downloading}
          onClick={() => void checkForUpdates()}
          className="rounded border border-[var(--border)] px-3 py-1.5 text-[12px] hover:bg-[var(--bg-hover)] disabled:cursor-wait disabled:opacity-50"
        >
          {checking ? T.sCheckingShort : T.recheck}
        </button>
      </footer>
    </div>
  );
}
