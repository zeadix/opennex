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
| Windows | Windows 10 1809+ | `.msi` 安装包 / `.zip` 便携版 |
| macOS | macOS 11+ | `.dmg` 安装包 / `.tar.gz` 便携版 |
| Linux | Ubuntu 20.04+ (glibc 2.31+) | `.deb` 安装包 / `.tar.gz` 便携版 |

## 开发与测试

```bash
# 开发模式（debug 构建，快捷键自动用代码默认值覆盖配置）
cargo run

# 运行测试
cargo test --lib

# Release 构建
cargo build --release

# 代码格式化检查
cargo fmt --all -- --check
```

### 多平台打包

```bash
# 需要在对应平台上执行，或通过 GitHub Actions 自动编译

# Linux (.deb + .tar.gz)
cargo install cargo-deb
cargo deb --output opennex-<version>-linux-amd64.deb
tar czf opennex-<version>-linux-x86_64.tar.gz -C target/release opennex

# macOS (.dmg + .tar.gz)
cargo install cargo-bundle
cargo bundle --release
tar czf opennex-<version>-macos-x86_64.tar.gz -C target/release opennex

# Windows (.msi + .zip)
cargo install cargo-wix
cargo wix --output opennex-<version>-windows-x86_64.msi
# 使用 7z 打包便携版
7z a opennex-<version>-windows-x86_64.zip target/release/opennex.exe
```

### CI/CD 自动发布

推送 `v*.*.*` 格式的 tag 即可触发 GitHub Actions 自动编译三平台并上传到 Cloudflare R2：

```bash
git tag v0.2.0
git push origin v0.2.0
```

## Project Structure

```
opennex/
├── .github/workflows/
│   └── release.yml                # CI/CD: 3-platform build + R2 upload + latest.json
├── assets/
│   ├── fonts/                     # Bundled fonts (CJK, Devanagari, Arabic)
│   └── desktop/                   # Linux .desktop entry
├── egui_term_local/               # Terminal emulator library (local path dependency)
│   └── src/
│       ├── backend/               # PTY backend (alacritty_terminal)
│       ├── bindings.rs            # Keyboard bindings
│       └── view.rs                # Terminal rendering
├── locales/                       # 9 language YAML files (embedded in binary)
├── src/
│   ├── main.rs                    # Application entry point
│   ├── app.rs                     # Main app logic (UI, events, state)
│   ├── updater.rs                 # Auto-update module
│   ├── theme.rs                   # Theme system (light/dark)
│   ├── i18n.rs                    # Internationalization
│   ├── history_db.rs              # SQLite command history
│   ├── terminal/                  # Terminal instance wrapper
│   └── completion/                # Command completion (WIP)
├── tests/
│   └── terminal_test.rs
├── Cargo.toml
└── Cargo.lock
```

## License

MIT
