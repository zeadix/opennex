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

export interface Theme {
  id: string;
  name: string;
  dark: boolean;
  colors: ThemeColors;
}

export const THEMES: Theme[] = [
  {
    id: "midnight",
    name: "Midnight",
    dark: true,
    colors: {
      bg: "#0b0e14", bgPanel: "#10141b", bgElevated: "#171c26",
      bgHover: "#1c2230", bgActive: "#232b3b", border: "#222a38",
      text: "#d7dce7", textDim: "#8b93a5", textFaint: "#5a6274",
      accent: "#4fc3f7", accentDim: "#23465c",
      danger: "#f07178", success: "#82d99a",
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
    id: "paper",
    name: "Paper",
    dark: false,
    colors: {
      bg: "#f6f7f9", bgPanel: "#eceef2", bgElevated: "#ffffff",
      bgHover: "#e2e6ec", bgActive: "#d4dae3", border: "#d4dae3",
      text: "#2c3340", textDim: "#5f6b7e", textFaint: "#98a2b3",
      accent: "#0f7fae", accentDim: "#cfe6f2",
      danger: "#c73e4a", success: "#2e8b57",
    },
  },
];

const KEY = "opennex-theme";

export function getTheme(id: string): Theme {
  return THEMES.find((t) => t.id === id) ?? THEMES[0];
}

export function loadThemeId(): string {
  return localStorage.getItem(KEY) ?? THEMES[0].id;
}

export function applyTheme(id: string) {
  const theme = getTheme(id);
  const root = document.documentElement;
  for (const [k, v] of Object.entries(theme.colors)) {
    root.style.setProperty(`--${k.replace(/[A-Z]/g, (c) => "-" + c.toLowerCase())}`, v);
  }
  localStorage.setItem(KEY, theme.id);
}
