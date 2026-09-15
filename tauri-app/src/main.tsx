import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";
import { applyTheme, loadThemeId } from "./theme/themes";

applyTheme(loadThemeId());

// NO StrictMode: its double-mount in dev spawns each terminal's PTY
// twice (mount -> unmount -> mount), killing the first session for no
// benefit — terminals are effect-heavy, IO-bound components.
ReactDOM.createRoot(document.getElementById("root")!).render(<App />);
