import { useEffect, useRef, useState } from "react";
import { useI18n } from "../i18n-context";
import { FiX } from "react-icons/fi";

export interface FloatWin {
  id: string;          // unique page id ("settings" | "ssh" | ...)
  title: string;
  x: number;
  y: number;
  w: number;
  h: number;
  z: number;
}

/**
 * Unified floating window chrome: draggable title bar, close button,
 * click-to-front z-order. Purely presentational — content comes via
 * children, position/size controlled by the parent.
 */
export default function FloatingWindow({
  win,
  onFocus,
  onClose,
  onMove,
  children,
}: {
  win: FloatWin;
  onFocus: (id: string) => void;
  onClose: (id: string) => void;
  onMove: (id: string, x: number, y: number) => void;
  children: React.ReactNode;
}) {
  const T = useI18n();
  const dragRef = useRef<{ dx: number; dy: number } | null>(null);

  const onTitleDown = (e: React.MouseEvent) => {
    onFocus(win.id);
    dragRef.current = { dx: e.clientX - win.x, dy: e.clientY - win.y };
    const move = (ev: MouseEvent) => {
      if (!dragRef.current) return;
      onMove(win.id, ev.clientX - dragRef.current.dx, ev.clientY - dragRef.current.dy);
    };
    const up = () => {
      dragRef.current = null;
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  };

  return (
    <div
      className="animate-fade-up absolute flex flex-col overflow-hidden rounded-[14px] border border-[var(--border)] bg-[var(--bg)] shadow-[var(--shadow-pop)]"
      style={{
        left: win.x,
        top: win.y,
        width: win.w,
        // 高度随内容自适应；上限为 开窗高度 与 视口 可用高度中较小者，
        // 超出部分在内容区滚动 —— 短内容不再留大片空白。
        maxHeight: `min(${win.h}px, calc(100vh - 24px))`,
        zIndex: 7000 + win.z,
      }}
      onMouseDown={() => onFocus(win.id)}
    >
      <div
        className="flex h-9 shrink-0 cursor-move items-center justify-between border-b border-[var(--border)] bg-[var(--bg-elevated)] px-3"
        onMouseDown={onTitleDown}
      >
        <span className="flex items-center gap-2 text-[12px] font-semibold text-[var(--text)]">
          <span className="h-1.5 w-1.5 rounded-full bg-[var(--accent)]" />
          {win.title}
        </span>
        <button className="icon-btn !p-1" title={T.cClose} onClick={(e) => { e.stopPropagation(); onClose(win.id); }}>
          <FiX size={14} />
        </button>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto">{children}</div>
    </div>
  );
}
