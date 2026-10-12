[简体中文](README.md) | [English](README.en.md) | **繁體中文** | [Deutsch](README.de.md) | [Français](README.fr.md) | [日本語](README.ja.md) | [Italiano](README.it.md) | [한국어](README.ko.md) | [हिन्दी](README.hi.md)

# OpenNex

OpenNex 是一款**多視窗堆疊式 AI 終端管理器**——將任何 AI Coding 或 Agent 工具變為可多開、可遠端操控的智慧工作平台。

![軟體全景](docs/screenshots/panorama.png)

| | |
|---|---|
| **1000+** | 活動視窗並行執行 |
| **20+** | 介面語言 |
| **3** | 原生平台（Linux / Windows / macOS） |

## 介面一覽

**🤖 AI 助理 · 模型設定**

![](docs/screenshots/ai-config.png)

設定模型 API 與上下文限額，即可啟用 AI 助理。

**🤖 AI 助理 · 對話式控制終端**

![](docs/screenshots/ai-chat-terminal.png)

設定好模型 API 後，可直接以對話讓 AI 在終端中執行指令或排查問題，全程可介入。

**🌐 遠端控制助理**

![](docs/screenshots/remote-control.png)

手機掃碼或瀏覽器連線即可遠端操作終端；透過終端啟動的軟體同樣可被遠端使用。

**🔒 工作區鎖定**

![](docs/screenshots/workspace-lock.png)

暫時離開時一鍵鎖定工作區，回來快捷解鎖，不怕被他人誤操作。

**★ 指令收藏夾**

![](docs/screenshots/command-favorites.png)

把常用指令收藏進資料夾，隨手調用。

**↩ 指令自動補全**

![](docs/screenshots/auto-complete.png)

輸入時自動匹配補全：系統指令 + 當前終端輸入過的歷史指令。

**◉ 工作區閒忙指示器**

![](docs/screenshots/idle-busy-indicator.png)

紅色 = 該工作區中還有終端處於活躍狀態；綠色 = 所有終端都已空閒，任務執行完畢。

## 核心特性

**✦ AI 自動執行**
AI 自主規劃並執行終端任務，全程可介入、可暫停。

**🌐 廣域網遠端控制**
突破區域網路限制，手機、平板、電腦均可隨時隨地遠端接管終端會話。

**▦ 彈性視窗布局**
自由排列、堆疊終端視窗，布局可儲存與載入，AI 工具多開井然有序。

**↩ 指令記憶**
無限記錄輸入過的每條指令，快捷鍵一鍵召回，告別重複輸入。

**★ 收藏系統**
常用指令一鍵收藏，高頻命令隨時調用。

**◈ 主題設定**
自訂主題美化，打造專屬終端外觀。

**🔒 工作區加密**
主密碼鎖定與工作區加密，敏感環境安心使用。

**⇄ 區域網路遠端控制**
同一區域網路內，手機、平板、電腦均可遠端接管終端會話，工作現場隨處可達。

**◉ 工作區閒忙偵測**
即時監測各工作區執行狀態，忙閒一目瞭然。

**⇅ SSH 主機連線**
內建 SSH 主機管理，遠端伺服器一鍵直達。

**⚡ 效能監控**
即時掌握視窗與系統資源佔用，執行狀態盡在掌握。

**⟲ 工作區記憶**
工作區狀態自動儲存，重開即恢復上次現場。

**🖥 系統資訊監控**
即時檢視主機系統資訊與資源佔用狀態。

**⤺ 路徑復原**
重新啟動後自動恢復各視窗工作路徑，無縫續接任務。

**🌐 多語言**
內建 20+ 國際化語言，一鍵切換。

## 多平台支援

| 平台 | 最低需求 | 安裝方式 |
|------|---------|---------|
| Windows | Windows 10+ | NSIS 安裝包 / MSI |
| macOS | macOS 12+ (Apple Silicon) | DMG |
| Linux | Ubuntu 22.04+ (glibc 2.35+) | deb / AppImage |

全部安裝套件在 [官網](https://opennex.zeadix.com) 或 [GitHub Releases](https://github.com/zeadix/opennex/releases/latest) 下載；應用內建簽章驗證的一鍵更新。

## 開發與測試

```bash
cd tauri-app

# 安裝前端依賴（首次）
npm install

# 開發模式：vite 熱重載 + Rust 除錯建置（也可直接 ./dev.sh）
npm run tauri dev

# 前端型別檢查 / 生產建置
npx tsc --noEmit
npm run build

# 打包（目前平台）
npm run tauri build
```

### CI/CD 自動發布

推送 `v*.*.*` 格式的 tag 即可觸發 GitHub Actions：多平台打包（NSIS+MSI / DMG / DEB+AppImage）、minisign 簽章更新套件、產生更新源 `tauri/latest.json` 並上傳 R2，同時發布 GitHub Release 並觸發官網重建：

```bash
git tag v0.2.0
git push origin v0.2.0
```

## 專案結構

```
opennex/
├── .github/workflows/
│   ├── release.yml                # 发版：多平台打包 + 签名 + R2 + 更新源 + Release
│   └── rebuild-updater-feed.yml   # 手动工具：从 R2 重组更新源（免发版修复）
├── assets/icon/                   # 应用图标源图
├── tauri-app/                     # Tauri 客户端（唯一迭代线）
│   ├── src/                       # React + TypeScript 前端（xterm.js 终端）
│   ├── src-tauri/                 # Tauri 2 Rust 后端（PTY/历史/AI 代理/更新器）
│   └── dev.sh                     # 一键开发启动（vite + cargo run）
└── research/                      # 技术调研与审计记录
```

## License

MIT