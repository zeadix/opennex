import { useI18n } from '../i18n-context';
import { useEffect, useRef, useState } from "react";

export interface FontOption {
  /** Display label. */
  label: string;
  /** CSS font-family value; "" = app default. */
  css: string;
  /** Fallback stack used while previewing (sans for UI, mono for terminal). */
  fallback: "sans" | "mono";
}

/** WYSIWYG font picker: every option renders IN its own font so the
 * user sees the actual style while choosing. */
export default function FontSelect({
  value,
  options,
  onChange,
  disabled,
}: {
  value: string;
  options: FontOption[];
  onChange: (css: string) => void;
  disabled?: boolean;
}) {
  const sysDefault = useI18n().sSystemDefault;
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    window.addEventListener("mousedown", onDown);
    return () => window.removeEventListener("mousedown", onDown);
  }, [open]);

  const current = options.find((o) => o.css === value) ?? options[0];

  return (
    <div ref={ref} className="relative w-56 shrink-0">
      <button
        disabled={disabled}
        className={`flex w-full items-center justify-between gap-2 rounded-md border border-[var(--border)] bg-[var(--bg)] px-2.5 py-1.5 text-[12px] text-[var(--text)] ${
          disabled ? "opacity-40" : "hover:border-[var(--text-faint)]"
        }`}
        style={{ fontFamily: current?.css ? `${current.css}, ${current.fallback === "mono" ? "monospace" : "sans-serif"}` : undefined }}
        onClick={() => setOpen((v) => !v)}
      >
        <span className="min-w-0 flex-1 truncate">{current?.label ?? sysDefault}</span>
        <span className="text-[10px] text-[var(--text-faint)]">▾</span>
      </button>
      {open && (
        <div className="animate-fade-up absolute right-0 top-full z-[9500] mt-1 max-h-72 w-[240px] overflow-y-auto rounded-md border border-[var(--border)] bg-[var(--bg-elevated)] py-1 shadow-2xl">
          {options.map((o) => (
            <div
              key={o.label}
              onClick={() => {
                onChange(o.css);
                setOpen(false);
              }}
              className={`flex cursor-pointer items-center justify-between gap-2 px-3 py-1.5 hover:bg-[var(--bg-hover)] ${
                o.css === value ? "text-[var(--accent)]" : "text-[var(--text-dim)]"
              }`}
              style={{ fontFamily: o.css ? `${o.css}, ${o.fallback === "mono" ? "monospace" : "sans-serif"}` : undefined }}
            >
              <span className="min-w-0 flex-1 truncate">
                {o.label}
                <span className="ml-1.5 opacity-70">Aa博 123</span>
              </span>
              {o.css === value && <span className="shrink-0 text-[11px]">✓</span>}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
