// Pane layout tree: leaves hold terminal sessions; splits are recursive
// flex containers. `h` = children laid out left-to-right, `v` = top-to-
// bottom. `ratio[i]` is the fractional size of children[i] (ratios sum
// to 1 per split node).

export type PaneTree =
  | { kind: "leaf"; pane: number }
  | { kind: "split"; dir: "h" | "v"; children: PaneTree[]; ratio: number[] };

export function leaf(pane: number): PaneTree {
  return { kind: "leaf", pane };
}

/** Replace the leaf holding `pane` with a 2-way split + fresh pane. */
export function splitPane(
  tree: PaneTree,
  pane: number,
  dir: "h" | "v",
  newPane: number,
): PaneTree {
  if (tree.kind === "leaf") {
    if (tree.pane !== pane) return tree;
    return {
      kind: "split",
      dir,
      children: [leaf(pane), leaf(newPane)],
      ratio: [0.5, 0.5],
    };
  }
  return {
    ...tree,
    children: tree.children.map((c) => splitPane(c, pane, dir, newPane)),
  };
}

/** Remove leaf `pane`; a parent with one child left collapses into it. */
export function closePane(tree: PaneTree, pane: number): PaneTree {
  if (tree.kind === "leaf") return tree;
  const children = tree.children.map((c) => closePane(c, pane));
  const alive = children.filter(
    (c) => c.kind === "leaf" || (c.kind === "split" && c.children.length > 0),
  );
  if (alive.length === 0) return leaf(-1); // caller replaces/closes tab
  if (alive.length === 1) return alive[0];
  // Re-normalize ratios over surviving children.
  const keepIdx = children
    .map((c, i) => (alive.includes(c) ? i : -1))
    .filter((i) => i >= 0);
  const total = keepIdx.reduce((acc, i) => acc + tree.ratio[i], 0);
  return {
    ...tree,
    children: alive,
    ratio: keepIdx.map((i) => tree.ratio[i] / (total || 1)),
  };
}

/** Collect every leaf pane id (to know which terminal sessions die). */
export function collectPanes(tree: PaneTree, out: number[] = []): number[] {
  if (tree.kind === "leaf") out.push(tree.pane);
  else tree.children.forEach((c) => collectPanes(c, out));
  return out;
}

/** Does the tree contain `pane`? */
export function hasPane(tree: PaneTree, pane: number): boolean {
  if (tree.kind === "leaf") return tree.pane === pane;
  return tree.children.some((c) => hasPane(c, pane));
}

/** Set the ratio of split node at `path` (indices down the tree). */
export function setRatio(
  tree: PaneTree,
  path: number[],
  ratio: number[],
): PaneTree {
  if (path.length === 0) {
    if (tree.kind !== "split") return tree;
    return { ...tree, ratio };
  }
  if (tree.kind !== "split") return tree;
  const [head, ...rest] = path;
  return {
    ...tree,
    children: tree.children.map((c, i) =>
      i === head ? setRatio(c, rest, ratio) : c,
    ),
  };
}

/** Find the split node at `path`, plus its on-screen size (w or h). */
export function findSplit(
  tree: PaneTree,
  path: number[],
): { dir: "h" | "v"; ratio: number[] } | null {
  let node = tree;
  for (const i of path) {
    if (node.kind !== "split") return null;
    node = node.children[i];
  }
  return node.kind === "split" ? { dir: node.dir, ratio: node.ratio } : null;
}
