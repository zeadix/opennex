export default function ReleaseNotes({ notes, lang }: { notes: string[]; lang: string }) {
  return notes.length ? (
    <ul className="space-y-2 text-[12px] leading-relaxed text-[var(--text)]">
      {notes.map((note, index) => <li className="break-words" key={index}>{note}</li>)}
    </ul>
  ) : (
    <p className="text-[12px] leading-relaxed text-[var(--text-dim)]">
      {lang.startsWith("en") ? "No release notes are available for this version." : "暂无此版本的更新说明。"}
    </p>
  );
}
