import { useRef, useState } from "react";
import { PaneTree, setRatio } from "./tree";
import TerminalPane from "../terminal/TerminalPane";

/**
 * Recursive pane renderer. Leaves host a TerminalPane; split nodes lay
 * children out along `dir` with draggable dividers between them.
 */
export default function PaneView({
  tree,
  path,
  activePane,
  themeId,
  onActivate,
  onClose,
  onSplit,
  onRatio,
}: {
  tree: PaneTree;
  path: number[];
  activePane: number;
  themeId: string;
  onActivate: (pane: number) => void;
  onClose: (pane: number) => void;
  onSplit: (pane: number, dir: "h" | "v") => void;
  onRatio: (path: number[], ratio: number[]) => void;
}) {
  if (tree.kind === "leaf") {
    const isActive = tree.pane === activePane;
    return (
      <div
        className="relative min-w-0 min-h-0 flex-1"
        style={{ display: "flex", flexDirection: "column" }}
        onMouseDown={() => onActivate(tree.pane)}
      >
        {/* Pane hover toolbar: split + close */}
        <div
          className={`absolute right-1.5 top-1 z-10 flex gap-1 rounded-md border border-[var(--border)] bg-[var(--bg-elevated)] px-1 py-0.5 transition-opacity ${
            isActive ? "opacity-90" : "opacity-0"
          } hover:opacity-100`}
          style={isActive ? { opacity: 0.9 } : undefined}
        >
          <button
            className="icon-btn !p-1"
            title="左右分屏"
            onClick={(e) => {
              e.stopPropagation();
              onSplit(tree.pane, "h");
            }}
          >
            <svg width="12" height="12" viewBox="0 0 12 12">
              <rect x="0" y="0" width="5" height="12" fill="currentColor" opacity="0.5" />
              <rect x="7" y="0" width="5" height="12" fill="currentColor" opacity="0.5" />
            </svg>
          </button>
          <button
            className="icon-btn !p-1"
            title="上下分屏"
            onClick={(e) => {
              e.stopPropagation();
              onSplit(tree.pane, "v");
            }}
          >
            <svg width="12" height="12" viewBox="0 0 12 12">
              <rect x="0" y="0" width="12" height="5" fill="currentColor" opacity="0.5" />
              <rect x="0" y="7" width="12" height="5" fill="currentColor" opacity="0.5" />
            </svg>
          </button>
          <button
            className="icon-btn !p-1 hover:!text-[var(--danger)]"
            title="关闭此窗格"
            onClick={(e) => {
              e.stopPropagation();
              onClose(tree.pane);
            }}
          >
            <svg width="11" height="11" viewBox="0 0 12 12">
              <path d="M2 2 L10 10 M10 2 L2 10" stroke="currentColor" strokeWidth="1.6" />
            </svg>
          </button>
        </div>
        <TerminalPane sessionId={tree.pane} themeId={themeId} />
      </div>
    );
  }

  return (
    <SplitNode
      tree={tree}
      path={path}
      activePane={activePane}
      themeId={themeId}
      onActivate={onActivate}
      onClose={onClose}
      onSplit={onSplit}
      onRatio={onRatio}
    />
  );
}

function SplitNode({
  tree,
  path,
  activePane,
  themeId,
  onActivate,
  onClose,
  onSplit,
  onRatio,
}: {
  tree: Extract<PaneTree, { kind: "split" }>;
  path: number[];
  activePane: number;
  themeId: string;
  onActivate: (pane: number) => void;
  onClose: (pane: number) => void;
  onSplit: (pane: number, dir: "h" | "v") => void;
  onRatio: (path: number[], ratio: number[]) => void;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [dragging, setDragging] = useState<number | null>(null);

  const horizontal = tree.dir === "h";

  const startDrag = (dividerIdx: number, e: React.MouseEvent) => {
    e.preventDefault();
    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect) return;
    const total = horizontal ? rect.width : rect.height;
    // Ratio of the divider's LEFT child (index dividerIdx) changes by
    // dragging; steal/give from the child to its right.
    const startRatio = tree.ratio[dividerIdx];
    const startPos = horizontal ? e.clientX - rect.left : e.clientY - rect.top;
    setDragging(dividerIdx);

    const onMove = (ev: MouseEvent) => {
      const cur = horizontal ? ev.clientX - rect.left : ev.clientY - rect.top;
      const gutter = 5; // matches divider thickness incl. hit area
      const dragSpan = total - (tree.children.length - 1) * gutter;
      let newFirst = startRatio + (cur - startPos) / dragSpan;
      newFirst = Math.min(0.88, Math.max(0.12, newFirst));
      const delta = newFirst - startRatio;
      const ratio = [...tree.ratio];
      ratio[dividerIdx] = newFirst;
      // Transfer the delta from the right neighbor; normalize to avoid
      // float drift.
      ratio[dividerIdx + 1] = tree.ratio[dividerIdx + 1] - delta;
      const sum = ratio.reduce((a, r) => a + r, 0);
      onRatio(path, ratio.map((r) => r / sum));
    };
    const onUp = () => {
      setDragging(null);
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  };

  const children = tree.children.map((c, i) => {
    const parts: React.ReactNode[] = [];
    if (i > 0) {
      parts.push(
        <div
          key={`d${i}`}
          onMouseDown={(e) => startDrag(i - 1, e)}
          className={`relative z-[5] shrink-0 bg-[var(--border)] transition-colors hover:bg-[var(--accent)] ${
            horizontal ? "w-[5px] cursor-col-resize" : "h-[5px] cursor-row-resize"
          } ${dragging === i - 1 ? "bg-[var(--accent)]" : ""}`}
        />,
      );
    }
    parts.push(
      <div
        key={`c${i}`}
        className="min-w-0 min-h-0"
        style={{ flex: `${tree.ratio[i]} 1 0%`, display: "flex" }}
      >
        <PaneView
          tree={c}
          path={[...path, i]}
          activePane={activePane}
          themeId={themeId}
          onActivate={onActivate}
          onClose={onClose}
          onSplit={onSplit}
          onRatio={onRatio}
        />
      </div>,
    );
    return parts;
  });

  return (
    <div
      ref={containerRef}
      className={`flex min-h-0 min-w-0 flex-1 ${horizontal ? "flex-row" : "flex-col"}`}
    >
      {children}
    </div>
  );
}
