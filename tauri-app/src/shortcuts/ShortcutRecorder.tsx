import { useI18n } from '../i18n-context';
import { useEffect, useState } from "react";
import { eventToBinding } from "./shortcuts";

/** Click-to-record key capture button: click, press a combo, done. */
export default function ShortcutRecorder({
  binding,
  onChange,
}: {
  binding: string;
  onChange: (binding: string) => void;
}) {
  const T = useI18n();
  const [recording, setRecording] = useState(false);
  const [captured, setCaptured] = useState<string | null>(null);

  useEffect(() => {
    if (!recording) return;
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const b = eventToBinding(e);
      if (b) {
        onChange(b);
        setCaptured(b);
        setRecording(false);
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [recording, onChange]);

  return (
    <button
      onClick={() => setRecording(true)}
      className={`rounded-md border px-2.5 py-1 font-mono text-[11px] transition-colors ${
        recording
          ? "border-[var(--accent)] bg-[var(--accent-dim)] text-[var(--accent)]"
          : "border-[var(--border)] text-[var(--text-dim)] hover:text-[var(--text)]"
      }`}
    >
      {recording ? T.sPressKeys : captured ?? binding}
    </button>
  );
}
