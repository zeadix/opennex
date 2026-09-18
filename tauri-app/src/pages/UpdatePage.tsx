import { checkForUpdates, releaseNotes, useUpdates } from "../updates";
import { fmt } from "../i18n";
import { useI18n } from "../i18n-context";
import ReleaseNotes from "../components/ReleaseNotes";

export default function UpdatePage({ lang }: { lang: string }) {
  const T = useI18n();
  const { current, checking, result, error } = useUpdates();
  const available = !!result?.updateAvailable;
  const notes = result ? releaseNotes(
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
              : error ? T.sCheckFailed
              : available ? fmt(T.sNewFound, { v: result!.latest })
              : result ? T.sUpToDate
              : T.sNotChecked}
          </p>
        </header>
        {error && <p role="alert" className="break-words text-[12px] text-[var(--danger)]">{error}</p>}
        <section className="border-t border-[var(--border)] pt-3">
          <h3 className="mb-3 text-[12px] font-semibold">
            {available ? fmt(T.sWhatsNewV, { v: result!.latest })
              : T.sCurrentNotes}
          </h3>
          {checking && !result ? <p className="text-[12px] text-[var(--text-dim)]">{T.sLoadingRelease}</p>
            : <ReleaseNotes lang={lang} notes={notes} />}
        </section>
      </div>
      <footer className="mt-3 flex shrink-0 justify-end border-t border-[var(--border)] pt-3">
        <button
          disabled={checking}
          onClick={() => void checkForUpdates()}
          className="rounded border border-[var(--border)] px-3 py-1.5 text-[12px] hover:bg-[var(--bg-hover)] disabled:cursor-wait disabled:opacity-50"
        >
          {checking ? T.sCheckingShort : T.recheck}
        </button>
      </footer>
    </div>
  );
}
