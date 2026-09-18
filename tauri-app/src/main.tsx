import ReactDOM from "react-dom/client";
import App from "./App";
import "@fontsource/geist-sans/400.css";
import "@fontsource/geist-sans/500.css";
import "@fontsource/geist-sans/600.css";
import "@fontsource/geist-sans/700.css";
import "@fontsource/jetbrains-mono/400.css";
import "@fontsource/jetbrains-mono/500.css";
import "./styles.css";
import { applyTheme, loadThemeId } from "./theme/themes";

applyTheme(loadThemeId());

// Native-app feel: the webview's default context menu (reload / back /
// forward) must never appear. Only menus we render ourselves (e.g. the
// terminal copy/paste menu) are allowed.
document.addEventListener("contextmenu", (e) => e.preventDefault());

// NO StrictMode: its double-mount in dev spawns each terminal's PTY
// twice (mount -> unmount -> mount), killing the first session for no
// benefit — terminals are effect-heavy, IO-bound components.
ReactDOM.createRoot(document.getElementById("root")!).render(<App />);
