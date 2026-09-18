import { useI18n } from '../i18n-context';
import { useEffect, useRef, useState } from "react";

/** Modal single-line prompt (template naming, workspace rename).
 * window.prompt is unavailable inside the Tauri webview. */
export default function PromptDialog({
  title,
  defaultValue,
  placeholder,
  okText,
  cancelText,
  onOk,
  onCancel,
}: {
  title: string;
  defaultValue?: string;
  placeholder?: string;
  okText?: string;
  cancelText?: string;
  onOk: (value: string) => void;
  onCancel: () => void;
}) {
  const T = useI18n();
  const [val, setVal] = useState(defaultValue ?? "");
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => {
    ref.current?.focus();
    ref.current?.select();
  }, []);
  const submit = () => {
    const v = val.trim();
    if (v) onOk(v);
  };
  return (
    <div
      className="fixed inset-0 z-[9000] flex items-center justify-center bg-black/50"
      onMouseDown={onCancel}
    >
      <div
        className="animate-fade-up w-[360px] rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4 shadow-2xl"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <div className="mb-3 text-[13px] font-semibold">{title}</div>
        <input
          ref={ref}
          value={val}
          placeholder={placeholder}
          onChange={(e) => setVal(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") submit();
            else if (e.key === "Escape") onCancel();
          }}
          className="dialog-input"
        />
        <div className="mt-4 flex justify-end gap-2">
          <button
            className="rounded-md border border-[var(--border)] px-3 py-1.5 text-[12px] text-[var(--text-dim)] hover:bg-[var(--bg-hover)]"
            onClick={onCancel}
          >
            {cancelText ?? T.cancel}
          </button>
          <button
            className="rounded-md bg-[var(--accent-dim)] px-3 py-1.5 text-[12px] text-[var(--accent)] hover:brightness-125"
            onClick={submit}
          >
            {okText ?? T.ok}
          </button>
        </div>
      </div>
    </div>
  );
}
