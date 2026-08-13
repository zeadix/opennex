# OpenNex

多功能堆叠式终端管理器，集成标签布局、会话命令记忆、全局界面缩放与加密工作区，高效管控你的终端环境。

A multi-functional stacked terminal manager with tabbed layouts, unlimited session command memory, global interface scaling, and encrypted workspaces for efficient terminal management.

## 核心特性 / Key Features

### 多窗口与可堆叠布局 / Multi-Window & Stackable Layouts

支持多终端会话同时运行，每个终端独立管理。水平/垂直分屏，可自由拖拽调整布局。嵌套选项卡系统，支持多层级 TabSet。

Run multiple terminal sessions simultaneously with independent management. Split panes horizontally/vertically with free drag-to-resize. Nested tab system with multi-level TabSets.

### 可保存的终端布局 / Persistent Terminal Layouts

工作区布局自动持久化，重启后完整恢复。支持保存/加载自定义布局模板。多工作区管理，侧边栏快速切换。

Workspace layouts auto-persist and fully restore after restart. Save/load custom layout templates. Multi-workspace management with quick sidebar switching.

### 无限指令记忆菜单 / Unlimited Command History Menu

记录当前终端窗口的所有指令，支持快捷键召回。按 `Alt` 命出历史菜单，上下键快速导航，`Enter` 一键召回，自动去重置顶。

Records all commands per terminal session with quick recall via shortcuts. Press `Alt` to open history menu, navigate with arrow keys, `Enter` to recall, with automatic deduplication and most-recent-first ordering.

### 加密终端工作区 / Encrypted Terminal Workspaces

密码保护工作区，防止未授权访问。一键锁定/解锁，支持快捷键操作。遮罩层覆盖，锁定状态下隐藏所有终端内容。

Password-protect workspaces against unauthorized access. One-click lock/unlock with keyboard shortcuts. Overlay mask hides all terminal content when locked.

### 便捷快捷键 / Convenient Shortcuts

11 个可自定义快捷键动作，支持交互式录制。`F1` 显示/隐藏工作区侧栏。`Alt` 呼出历史指令菜单。支持恢复默认快捷键。

11 customizable shortcut actions with interactive recording. `F1` to toggle workspace sidebar. `Alt` for command history menu. Reset to defaults supported.

### 更多功能 / More Features

- **命令历史**：SQLite 存储，自动去重置顶，按终端会话隔离
- **多语言支持**：中文、繁体中文、英语、德语、法语、日语、意大利语、韩语、印地语
- **视觉主题系统**：5 套内嵌主题（OpenNex Dark/Light、Solarized、Gruvbox、Dracula），统一应用界面与终端 ANSI 配色，支持自定义、导入导出 `.opennex-theme.json`，跨平台通用
- **跨平台 CWD 跟踪**：OSC shell 集成（bash/zsh/powershell），重启后恢复终端工作目录
- **在线自动更新**：检测新版本 → 一键下载 → SHA256 校验 → 自动替换重启

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
