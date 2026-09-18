// Theme system: every theme is a CSS-variable set applied to :root.
// Terminal (xterm) colors read the same variables so panes always match.

export interface ThemeColors {
  bg: string;
  bgPanel: string;
  bgElevated: string;
  bgHover: string;
  bgActive: string;
  border: string;
  text: string;
  textDim: string;
  textFaint: string;
  accent: string;
  accentDim: string;
  danger: string;
  success: string;
}

/** Terminal-side colors: xterm background/foreground/cursor/selection
 * plus the 16-color ANSI palette (egui theme editor parity). */
export interface TermPalette {
  background: string;
  foreground: string;
  cursor: string;
  selection: string;
  ansi: string[];
}

/** Per-theme font configuration (主题字体): overrides the global font
 * settings when settings.useThemeFont is on. Sizes in px. */
export interface ThemeFonts {
  uiFont: string;
  uiFontSize: number;
  termFont: string;
  termFontSize: number;
}

export interface Theme {
  id: string;
  name: string;
  dark: boolean;
  colors: ThemeColors;
  /** Present on custom themes; presets derive from their UI colors. */
  term?: TermPalette;
  /** Optional font pack (主题编辑器里配置). */
  font?: ThemeFonts;
}

export const THEMES: Theme[] = [
  {
    id: "midnight",
    name: "OpenNex Dark",
    dark: true,
    colors: {
      bg: "#0b0c0f", bgPanel: "#131519", bgElevated: "#1b1e24",
      bgHover: "#20242c", bgActive: "#ff5a2824", border: "#ffffff14",
      text: "#e8eaee", textDim: "#8b95a0", textFaint: "#5f6a76",
      accent: "#ff5a28", accentDim: "#ff5a2838",
      danger: "#e5484d", success: "#22c55e",
    },
  },
  {
    id: "nord",
    name: "Nord",
    dark: true,
    colors: {
      bg: "#2e3440", bgPanel: "#2b303b", bgElevated: "#3b4252",
      bgHover: "#434c5e", bgActive: "#4c566a", border: "#3b4252",
      text: "#eceff4", textDim: "#aab4c6", textFaint: "#6f7d95",
      accent: "#88c0d0", accentDim: "#33506b",
      danger: "#bf616a", success: "#a3be8c",
    },
  },
  {
    id: "gruvbox",
    name: "Gruvbox Dark",
    dark: true,
    colors: {
      bg: "#1d2021", bgPanel: "#22262a", bgElevated: "#282828",
      bgHover: "#32302f", bgActive: "#3c3836", border: "#3c3836",
      text: "#ebdbb2", textDim: "#a89984", textFaint: "#7c6f64",
      accent: "#fabd2f", accentDim: "#5c4a1e",
      danger: "#fb4934", success: "#b8bb26",
    },
  },
  {
    id: "dracula",
    name: "Dracula",
    dark: true,
    colors: {
      bg: "#1a1b26", bgPanel: "#16161e", bgElevated: "#24283b",
      bgHover: "#2f334d", bgActive: "#363b5e", border: "#292e42",
      text: "#c0caf5", textDim: "#8992b8", textFaint: "#565f89",
      accent: "#7aa2f7", accentDim: "#2f3d6b",
      danger: "#f7768e", success: "#9ece6a",
    },
  },
  {
    id: "tokyo-night",
    name: "Tokyo Night",
    dark: true,
    colors: {
      bg: "#1a1b26", bgPanel: "#16161e", bgElevated: "#1f2335",
      bgHover: "#292e42", bgActive: "#2f334d", border: "#292e42",
      text: "#a9b1d6", textDim: "#737aa2", textFaint: "#565f89",
      accent: "#7dcfff", accentDim: "#2a3b56",
      danger: "#f7768e", success: "#9ece6a",
    },
  },
  {
    id: "catppuccin",
    name: "Catppuccin Mocha",
    dark: true,
    colors: {
      bg: "#1e1e2e", bgPanel: "#181825", bgElevated: "#24243a",
      bgHover: "#313244", bgActive: "#3b3b52", border: "#313244",
      text: "#cdd6f4", textDim: "#9399b2", textFaint: "#6c7086",
      accent: "#89b4fa", accentDim: "#31406b",
      danger: "#f38ba8", success: "#a6e3a1",
    },
  },
  {
    id: "one-dark",
    name: "One Dark",
    dark: true,
    colors: {
      bg: "#282c34", bgPanel: "#21252b", bgElevated: "#2f343d",
      bgHover: "#3a3f4b", bgActive: "#454b56", border: "#3a3f4b",
      text: "#dcdfe4", textDim: "#9da5b4", textFaint: "#636d83",
      accent: "#61afef", accentDim: "#2f4a66",
      danger: "#e06c75", success: "#98c379",
    },
  },
  {
    id: "rose-pine",
    name: "Rosé Pine",
    dark: true,
    colors: {
      bg: "#191724", bgPanel: "#1f1d2e", bgElevated: "#26233a",
      bgHover: "#312e48", bgActive: "#3f3a5a", border: "#312e48",
      text: "#e0def4", textDim: "#908caa", textFaint: "#6e6a86",
      accent: "#c4a7e7", accentDim: "#3d2f56",
      danger: "#eb6f92", success: "#31748f",
    },
  },
  {
    id: "everforest",
    name: "Everforest",
    dark: true,
    colors: {
      bg: "#2b3339", bgPanel: "#262d33", bgElevated: "#323c41",
      bgHover: "#3a454b", bgActive: "#414b52", border: "#3a454b",
      text: "#d3c6aa", textDim: "#9da9a0", textFaint: "#7a8478",
      accent: "#a7c080", accentDim: "#42503a",
      danger: "#e67e80", success: "#a7c080",
    },
  },
  {
    id: "paper",
    name: "OpenNex Light",
    dark: false,
    colors: {
      bg: "#ffffff", bgPanel: "#fbfbfa", bgElevated: "#ffffff",
      bgHover: "#f6f6f5", bgActive: "#fdeee7", border: "#10121414",
      text: "#101214", textDim: "#697077", textFaint: "#7a8087",
      accent: "#f04e17", accentDim: "#fdeee7",
      danger: "#c73e4a", success: "#1e9e50",
    },
  },
];

const KEY = "opennex-theme";
const USER_KEY = "opennex-user-themes";

// ---- user-defined themes (主题编辑器) ------------------------------------

export function getUserThemes(): Theme[] {
  try {
    const raw = JSON.parse(localStorage.getItem(USER_KEY) ?? "[]");
    if (Array.isArray(raw)) {
      return raw
        .filter((t: any) => t && typeof t.id === "string")
        .map((t: any) => normalizeTheme(t));
    }
  } catch {
    /* ignore */
  }
  return [];
}

export function persistUserThemes(list: Theme[]) {
  localStorage.setItem(USER_KEY, JSON.stringify(list));
}

function normalizeTheme(t: any): Theme {
  const term = t.term ?? {};
  return {
    id: String(t.id),
    name: String(t.name ?? "自定义主题"),
    dark: t.dark !== false,
    colors: { ...THEMES[0].colors, ...(t.colors ?? {}) },
    term: {
      background: String(term.background ?? ""),
      foreground: String(term.foreground ?? ""),
      cursor: String(term.cursor ?? ""),
      selection: String(term.selection ?? ""),
      ansi: Array.isArray(term.ansi) && term.ansi.length === 16 ? term.ansi.map(String) : [...defaultAnsi(t.dark !== false)],
    },
    font:
      t.font && typeof t.font === "object"
        ? {
            uiFont: String(t.font.uiFont ?? ""),
            uiFontSize: Number(t.font.uiFontSize ?? 13),
            termFont: String(t.font.termFont ?? ""),
            termFontSize: Number(t.font.termFontSize ?? 14),
          }
        : undefined,
  };
}

/** Canonical ANSI-16 sets (standard 8 + bright 8) per light/dark base. */
export function defaultAnsi(dark: boolean): string[] {
  return dark
    ? ["#282c34", "#e06c75", "#98c379", "#e5c07b", "#61afef", "#c678dd", "#56b6c2", "#abb2bf",
       "#5c6370", "#e06c75", "#98c379", "#e5c07b", "#61afef", "#c678dd", "#56b6c2", "#ffffff"]
    : ["#4b505b", "#d43d4f", "#2e8b57", "#c18401", "#0f6fad", "#8a4fbf", "#0e8a86", "#3d4452",
       "#8b93a5", "#d43d4f", "#2e8b57", "#c18401", "#0f6fad", "#8a4fbf", "#0e8a86", "#1d2129"];
}

/** `#RRGGBB` → `rgba(r,g,b,a)`; non-hex input passes through untouched. */
function withAlpha(color: string, alpha: number): string {
  const m = /^#([0-9a-f]{6})$/i.exec(color.trim());
  if (!m) return color;
  const n = parseInt(m[1], 16);
  return `rgba(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}, ${alpha})`;
}

/** Derive a terminal palette from UI colors (preset themes without an
 * explicit term palette). */
export function deriveTerm(t: Theme): TermPalette {
  if (t.term && t.term.ansi.length === 16) return t.term;
  return {
    background: t.colors.bg,
    foreground: t.colors.text,
    cursor: t.colors.accent,
    selection: t.colors.accentDim,
    ansi: defaultAnsi(t.dark),
  };
}

/** Presets + user themes, presets first. */
export function allThemes(): Theme[] {
  return [...THEMES, ...getUserThemes()];
}

export function newThemeId(): string {
  return `user-${crypto.randomUUID?.() ?? Date.now()}`;
}

/** Draft a custom theme from an existing one (编辑副本). */
export function cloneAsCustom(src: Theme): Theme {
  return {
    id: newThemeId(),
    name: `${src.name} 副本`,
    dark: src.dark,
    colors: { ...src.colors },
    term: { ...deriveTerm(src), ansi: [...deriveTerm(src).ansi] },
    font: src.font ? { ...src.font } : undefined,
  };
}

export function getTheme(id: string): Theme {
  return allThemes().find((t) => t.id === id) ?? THEMES[0];
}

export function loadThemeId(): string {
  return localStorage.getItem(KEY) ?? THEMES[0].id;
}

export function applyTheme(id: string) {
  const theme = getTheme(id);
  applyThemeObject(theme);
  localStorage.setItem(KEY, theme.id);
}

/** Apply a theme object's tokens to :root (used by the live editor too).
 * Terminal colors get dedicated --term-* variables so the terminal can
 * diverge from the UI palette. */
export function applyThemeObject(theme: Theme) {
  const root = document.documentElement;
  for (const [k, v] of Object.entries(theme.colors)) {
    root.style.setProperty(`--${k.replace(/[A-Z]/g, (c) => "-" + c.toLowerCase())}`, v);
  }
  // Shape/shadow tokens derive from the accent + dark flag so custom and
  // preset themes stay consistent (website parity: soft two-layer shadows,
  // translucent accent washes instead of solid tints).
  root.style.setProperty("--accent-soft", withAlpha(theme.colors.accent, theme.dark ? 0.14 : 0.1));
  root.style.setProperty("--shadow-pop", theme.dark
    ? "0 1px 2px rgba(0, 0, 0, 0.4), 0 10px 28px rgba(0, 0, 0, 0.38)"
    : "0 1px 2px rgba(16, 18, 20, 0.06), 0 10px 28px rgba(16, 18, 20, 0.10)");
  root.style.setProperty("--shadow-lift", theme.dark
    ? "0 2px 6px rgba(0, 0, 0, 0.45), 0 18px 44px rgba(0, 0, 0, 0.52)"
    : "0 2px 6px rgba(16, 18, 20, 0.08), 0 18px 44px rgba(16, 18, 20, 0.14)");
  // FlexLayout declares its private --fl-* variables on each layout;
  // only the public --flexlayout-* inputs inherit from :root.
  const fl = {
    background: theme.colors.bg,
    base: theme.colors.bg,
    text: theme.colors.text,
    "tabset-background": theme.colors.bgElevated,
    "tabset-background-selected": theme.colors.bgElevated,
    "tabset-divider-line": theme.colors.border,
    "tab-content": theme.colors.bgPanel,
    "tab-selected": theme.colors.text,
    "tab-selected-background": theme.colors.bgPanel,
    "tab-unselected": theme.colors.textDim,
    "border-background": theme.colors.bgElevated,
    "border-divider-line": theme.colors.border,
    "border-tab-content": theme.colors.bgPanel,
    "border-tab-selected": theme.colors.text,
    "border-tab-selected-background": theme.colors.bgPanel,
    "border-tab-unselected": theme.colors.textDim,
    "border-tab-unselected-background": theme.colors.bgElevated,
    icon: theme.colors.textDim,
    overflow: theme.colors.textDim,
    focus: theme.colors.accent,
    splitter: theme.colors.border,
    "splitter-hover": theme.colors.bgHover,
    "splitter-drag": theme.colors.accentDim,
    "toolbar-button-hover": theme.colors.bgHover,
    "1": theme.colors.bgPanel,
    "2": theme.colors.bg,
    "3": theme.colors.bgElevated,
    "4": theme.colors.border,
    "5": theme.colors.bgHover,
    "6": theme.colors.bgActive,
  };
  for (const [k, v] of Object.entries(fl)) {
    root.style.setProperty(`--flexlayout-color-${k}`, v);
  }
  // Terminal-specific tokens (fall back to UI colors when unset).
  const term = deriveTerm(theme);
  root.style.setProperty("--term-background", term.background);
  root.style.setProperty("--term-foreground", term.foreground);
  root.style.setProperty("--term-cursor", term.cursor);
  root.style.setProperty("--term-selection", term.selection);
  term.ansi.forEach((c, i) => root.style.setProperty(`--term-ansi-${i}`, c));
  // Panes re-read the variables live (theme editor preview).
  window.dispatchEvent(new CustomEvent("opennex-terminal-theme"));
}
