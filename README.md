**简体中文** | [English](README.en.md) | [繁體中文](README.zh-TW.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [日本語](README.ja.md) | [Italiano](README.it.md) | [한국어](README.ko.md) | [हिन्दी](README.hi.md)

# OpenNex

OpenNex 是一款**多窗口堆叠式 AI 终端管理器**——将任何 AI Coding 或 Agent 工具变为可多开、可远程操控的智能工作平台。

![软件全景](docs/screenshots/panorama.png)

| | |
|---|---|
| **1000+** | 活动窗口并行运行 |
| **20+** | 界面语言 |
| **3** | 原生平台（Linux / Windows / macOS） |

## 核心特性

**✦ AI 自动执行**
AI 自主规划并执行终端任务，全程可介入、可暂停。

**🌐 广域网远程控制**
突破局域网限制，手机、平板、电脑均可随时随地远程接管终端会话。

**▦ 灵活窗口布局**
自由排布、堆叠终端窗口，布局可保存与加载，AI 工具多开井然有序。

**↩ 指令记忆**
无限记录输入过的每条指令，快捷键一键召回，告别重复输入。

**★ 收藏系统**
常用指令一键收藏，高频命令随时调用。

**◈ 主题配置**
自定义主题美化，打造专属终端外观。

**🔒 工作区加密**
主密码锁屏与工作区加密，敏感环境安心使用。

**⇄ 局域网远程控制**
同一局域网内，手机、平板、电脑均可远程接管终端会话，工作现场随处可达。

**◉ 工作区闲忙检测**
实时监测各工作区运行状态，忙闲一目了然。

**⇅ SSH 主机连接**
内置 SSH 主机管理，远程服务器一键直达。

**⚡ 性能监控**
实时掌握窗口与系统资源占用，运行状态尽在掌握。

**⟲ 工作区记忆**
工作区状态自动保存，重开即恢复上次现场。

**🖥 系统信息监控**
实时查看主机系统信息与资源占用状态。

**⤺ 路径复原**
重启后自动恢复各窗口工作路径，无缝续接任务。

**🌐 多语言**
内置 20+ 国际化语言，一键切换。

## 界面一览

**🤖 AI 助手 · 模型配置**

配置模型 API 与上下文限额，即可启用 AI 助手。

![AI 助手 · 模型配置](docs/screenshots/ai-config.png)

**🤖 AI 助手 · 对话式控制终端**

配置好模型 API 后，可以直接用对话让 AI 在终端中执行指令或排查问题，全程可介入。

![AI 助手 · 对话式控制终端](docs/screenshots/ai-chat-terminal.png)

**🌐 远程控制助手**

手机扫码或浏览器连接即可远程操作终端；通过终端启动的软件同样可以被远程使用。

![远程控制助手](docs/screenshots/remote-control.png)

**🔒 工作区锁定**

临时离开时一键锁定工作区，回来快捷解锁，不怕被他人误操作。

![工作区锁定](docs/screenshots/workspace-lock.png)

**★ 指令收藏夹**

把常用指令收藏进文件夹，随手调用。

![指令收藏夹](docs/screenshots/command-favorites.png)

**↩ 指令自动补全**

输入时自动匹配补全：系统指令 + 当前终端输入过的历史指令。

![指令自动补全](docs/screenshots/auto-complete.png)

**◉ 工作区闲忙指示器**

红色 = 该工作区中还有终端处于活跃状态；绿色 = 所有终端都已空闲，任务执行完毕。

![工作区闲忙指示器](docs/screenshots/idle-busy-indicator.png)

## 多平台支持

| 平台 | 最低要求 | 安装方式 |
|------|---------|---------|
| Windows | Windows 10+ | NSIS 安装包 / MSI |
| macOS | macOS 12+ (Apple Silicon) | DMG |
| Linux | Ubuntu 22.04+ (glibc 2.35+) | deb / AppImage |

全部安装包在 [官网](https://opennex.zeadix.com) 或 [GitHub Releases](https://github.com/zeadix/opennex/releases/latest) 下载；应用内置签名校验的一键更新。

## 开发与测试

```bash
cd tauri-app

# 安装前端依赖（首次）
npm install

# 开发模式：vite 热重载 + Rust 调试构建（也可直接 ./dev.sh）
npm run tauri dev

# 前端类型检查 / 生产构建
npx tsc --noEmit
npm run build

# 打包（当前平台）
npm run tauri build
```

### CI/CD 自动发布

推送 `v*.*.*` 格式的 tag 即可触发 GitHub Actions：多平台打包（NSIS+MSI / DMG / DEB+AppImage）、minisign 签名更新包、生成更新源 `tauri/latest.json` 并上传 R2，同时发布 GitHub Release 并触发官网重建：

```bash
git tag v0.2.0
git push origin v0.2.0
```

## 项目结构

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
