import { useI18n } from '../i18n-context';
import { useEffect, useRef, useState } from "react";
import { FiChevronUp, FiChevronDown, FiX } from "react-icons/fi";

/** Floating search strip anchored to the pane's top-right. Fully
 * controlled by the parent (open/close); navigation callbacks call the
 * SearchAddon. */
export default function SearchBar({
  onSearch,
  onNext,
  onPrev,
  onClose,
}: {
  onSearch: (q: string) => void;
  onNext: () => void;
  onPrev: () => void;
  onClose: () => void;
}) {
  const T = useI18n();
  const [q, setQ] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  return (
    <div
      className="absolute right-3 top-2 z-30 flex items-center gap-1 rounded-md border border-[var(--border)] bg-[var(--bg-elevated)] px-2 py-1 shadow-lg"
      onClick={(e) => e.stopPropagation()}
    >
      <input
        ref={inputRef}
        value={q}
        onChange={(e) => {
          setQ(e.target.value);
          onSearch(e.target.value);
        }}
        onKeyDown={(e) => {
          e.stopPropagation();
          if (e.key === "Enter") {
            e.shiftKey ? onPrev() : onNext();
          }
          if (e.key === "Escape") onClose();
        }}
        placeholder={T.searchPh}
        className="w-44 bg-transparent text-[12px] outline-none"
        style={{ userSelect: "text", fontFamily: "var(--mono)" }}
      />
      <button className="icon-btn !p-1" title={T.uPrevious} onClick={onPrev}>
        <FiChevronUp size={13} />
      </button>
      <button className="icon-btn !p-1" title={T.uNext} onClick={onNext}>
        <FiChevronDown size={13} />
      </button>
      <button className="icon-btn !p-1" title={T.uClose} onClick={onClose}>
        <FiX size={13} />
      </button>
    </div>
  );
}
