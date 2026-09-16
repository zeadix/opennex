// Auto-match: given the current input line and the command history,
// return the best matching command (prefix match) or null.
// Pure functions — unit-testable without xterm.

export function bestMatch(line: string, history: string[]): string | null {
  const trimmed = line.trim();
  if (trimmed.length < 2) return null;
  for (const cmd of history) {
    if (cmd !== trimmed && cmd.startsWith(trimmed) && cmd !== line) {
      return cmd;
    }
  }
  return null;
}

/** The remainder the user still needs typed (for the Tab-complete). */
export function remainder(line: string, suggestion: string): string {
  return suggestion.slice(line.length);
}
