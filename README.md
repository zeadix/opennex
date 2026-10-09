# OpenNex

OpenNex 是一款**多窗口堆叠式 AI 终端管理器**——将任何 AI Coding 或 Agent 工具变为可多开、可远程操控的智能工作平台。

OpenNex is a **multi-window stacked AI terminal manager** — turning any AI Coding or Agent tool into a multi-instance, remotely controllable smart workstation.

| | |
|---|---|
| **1000+** | 活动窗口并行运行 / concurrent active windows |
| **20+** | 国际化语言 / UI languages |
| **3** | 原生跨平台（Linux / Windows / macOS）/ native platforms |

## 核心特性 / Key Features

**✦ AI 自动执行**（即将上线 / Coming soon）
AI 自主规划并执行终端任务，全程可介入、可暂停。*AI plans and runs terminal tasks on its own, with full interrupt and pause control.*

**🌐 广域网远程控制**（即将上线 / Coming soon）
突破局域网限制，手机、平板、电脑均可随时随地远程接管终端会话。*Take over terminal sessions from phones, tablets and PCs beyond the LAN.*

**▦ 灵活窗口布局**
自由排布、堆叠终端窗口，布局可保存与加载，AI 工具多开井然有序。*Freely arrange and stack terminal windows; layouts save & load, keeping multi-instance AI tools tidy.*

**↩ 指令记忆**
无限记录输入过的每条指令，快捷键一键召回，告别重复输入。*Unlimited command history with one-keystroke recall — no more retyping.*

**★ 收藏系统**
常用指令一键收藏，高频命令随时调用。*One-click favorites for your most-used commands.*

**◈ 主题配置**
自定义主题美化，打造专属终端外观。*Custom themes for a personal terminal look.*

**🔒 工作区加密**
主密码锁屏与工作区加密，敏感环境安心使用。*Master-password lock screen and encrypted workspaces for sensitive environments.*

**⇄ 局域网远程控制**
同一局域网内，手机、平板、电脑均可远程接管终端会话，工作现场随处可达。*Control terminal sessions from any device on the same LAN.*

**◉ 工作区闲忙检测**
实时监测各工作区运行状态，忙闲一目了然。*Live idle/busy status for every workspace at a glance.*

**⇅ SSH 主机连接**
内置 SSH 主机管理，远程服务器一键直达。*Built-in SSH host management for one-click access to remote servers.*

**⚡ 性能监控**
实时掌握窗口与系统资源占用，运行状态尽在掌握。*Live window and system resource monitoring.*

**⟲ 工作区记忆**
工作区状态自动保存，重开即恢复上次现场。*Workspace state auto-saves and restores on relaunch.*

**🖥 系统信息监控**
实时查看主机系统信息与资源占用状态。*Real-time host system info and resource usage.*

**⤺ 路径复原**
重启后自动恢复各窗口工作路径，无缝续接任务。*Working directories restore automatically after restart.*

**🌐 多语言**
内置 20+ 国际化语言，一键切换。*20+ UI languages with one-click switching.*

## 多平台支持

| 平台 | 最低要求 | 安装方式 |
|------|---------|---------|
| Windows | Windows 10+ | NSIS 安装包 / MSI |
| macOS | macOS 12+ (Apple Silicon) | DMG |
| Linux | Ubuntu 22.04+ (glibc 2.35+) | deb / AppImage |

全部安装包在 [官网](https://opennex.zeadix.com) 或 [GitHub Releases](https://github.com/zeadix/opennex/releases/latest) 下载;应用内置签名校验的一键更新。

## 开发与测试

```bash
cd tauri-app

# 安装前端依赖(首次)
npm install

# 开发模式:vite 热重载 + Rust 调试构建(也可直接 ./dev.sh)
npm run tauri dev

# 前端类型检查 / 生产构建
npx tsc --noEmit
npm run build

# 打包(当前平台)
npm run tauri build
```

### CI/CD 自动发布

推送 `v*.*.*` 格式的 tag 即可触发 GitHub Actions:三平台打包
(NSIS+MSI / DMG / DEB+AppImage)、minisign 签名更新包、生成更新源
`tauri/latest.json` 并上传 R2,同时发布 GitHub Release 并触发官网重建:

```bash
git tag v0.2.0
git push origin v0.2.0
```

## Project Structure

```
opennex/
├── .github/workflows/
│   ├── release.yml                # 发版:三平台打包 + 签名 + R2 + 更新源 + Release
│   └── rebuild-updater-feed.yml   # 手动工具:从 R2 重组更新源(免发版修复)
├── assets/icon/                   # 应用图标源图
├── tauri-app/                     # Tauri 客户端(唯一迭代线)
│   ├── src/                       # React + TypeScript 前端(xterm.js 终端)
│   ├── src-tauri/                 # Tauri 2 Rust 后端(PTY/历史/AI 代理/更新器)
│   └── dev.sh                     # 一键开发启动(vite + cargo run)
└── research/                      # 技术调研与审计记录
```

## License

MIT
