import type { Lang } from "../i18n";
import { useI18n } from "../i18n-context";
import { releaseNotes, useUpdates } from "../updates";
import ReleaseNotes from "../components/ReleaseNotes";

export default function AboutPage({ lang, embedded = false }: { lang: Lang; embedded?: boolean }) {
  const T = useI18n();
  const { current, result, checking, error } = useUpdates();
  const version = current ?? "—";
  return (
    <div className={embedded ? "min-w-0" : "h-full overflow-y-auto p-3"}>
      <div className="w-full min-w-0">
        <div className="mb-5 flex items-center gap-3">
          <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-[var(--accent-dim)] font-mono text-[20px] font-bold text-[var(--accent)]">
            N
          </div>
          <div className="min-w-0">
            <div className="glow-text text-[18px] font-bold">OpenNex</div>
            <div className="font-mono text-[11px] text-[var(--text-faint)]">v{version}</div>
          </div>
        </div>
        <p className="mb-4 text-[12.5px] leading-relaxed text-[var(--text-dim)]">
          {T.sAboutIntro}
        </p>
        <section className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4">
          <h3 className="mb-2 text-[12px] font-semibold text-[var(--text-dim)]">
            {T.sCurrentNotes}
          </h3>
          {result ? (
            <ReleaseNotes
              lang={lang}
              notes={releaseNotes(result.currentChanges, result.currentChangesEn, lang)}
            />
          ) : (
            <p className="text-[12px] text-[var(--text-dim)]">
              {checking ? T.sLoadingInfo
                : error ? T.sNotesError
                : T.sNoNotes}
            </p>
          )}
        </section>
        <section className="mt-3 rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-4 text-[12px] leading-relaxed">
          <div className="grid gap-x-6 gap-y-1.5 sm:grid-cols-2">
            <div>
              <span className="text-[var(--text-faint)]">{T.sAuthor}: </span>
              KunPeng.Wang · <a className="break-all text-[var(--accent)] hover:underline" href="mailto:msr.rsm@qq.com">msr.rsm@qq.com</a>
            </div>
            <div>
              <span className="text-[var(--text-faint)]">{T.sLicense}: </span>
              MIT
            </div>
            <div className="sm:col-span-2">
              <span className="text-[var(--text-faint)]">{T.sHome}: </span>
              <a className="text-[var(--accent)] hover:underline" href="https://opennex.zeadix.com" rel="noreferrer" target="_blank">opennex.zeadix.com</a>
            </div>
            <div className="sm:col-span-2">
              <span className="text-[var(--text-faint)]">{T.sSource}: </span>
              <a className="text-[var(--accent)] hover:underline" href="https://github.com/zeadix/opennex" rel="noreferrer" target="_blank">github.com/zeadix/opennex</a>
            </div>
          </div>
          <div className="mt-2 text-[11px] text-[var(--text-faint)]">
            © 2026 KunPeng.Wang · OpenNex · MIT License
          </div>
        </section>
      </div>
    </div>
  );
}
