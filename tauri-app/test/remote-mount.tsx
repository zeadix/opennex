// RemotePage 目检挂载器(test-remote.html 用):直接渲染组件,
// 不经 App/TopBar 路由;IPC 由 test-remote.html 的假 __TAURI_INTERNALS__ 提供。
import React from "react";
import ReactDOM from "react-dom/client";
import { I18nProvider } from "/src/i18n-context";
import RemotePage from "/src/pages/RemotePage";
import "/src/styles.css";
import { applyTheme, loadThemeId } from "/src/theme/themes";

applyTheme(loadThemeId());
const tab = new URLSearchParams(location.search).get("tab") === "wan" ? "wan" : "lan";
ReactDOM.createRoot(document.getElementById("root")!).render(
  <I18nProvider lang="zh">
    <RemotePage initialTab={tab} />
  </I18nProvider>,
);
