# Project: OpenNex

## Overview

OpenNex 是一个跨平台的多功能堆叠式终端管理器，集成标签布局、会话命令记忆、全局界面缩放与加密工作区，高效管控你的终端环境。

基于纯 Rust 构建，使用 egui/eframe 作为 GUI 框架，alacritty_terminal 作为终端后端。

## Implemented Features

- [x] 基础终端管理 - alacritty_terminal 后端，支持多独立终端会话
- [x] 多终端切换 - egui_dock Tab 切换
- [x] 可配置快捷键 - 11 个可自定义动作，交互式录制
- [x] 分屏布局系统 - egui_dock 水平/垂直分屏
- [x] 拖拽调整分屏 - egui_dock 原生拖拽
- [x] 嵌套选项卡系统 - egui_dock TabSet 嵌套
- [x] 选项卡管理 - 新建、关闭、重命名、关闭确认弹窗
- [x] 布局配置持久化 - DockState → JSON → scene.json
- [x] 命令历史 - Rust SQLite 后端
- [x] 布局预设模板 - templates/*.json 模板管理
- [x] 工作区管理 - 侧边栏多工作区、拖拽排序、新建/模板按钮置顶
- [x] 设置面板 - 4 Tab（通用/外观/快捷键/锁定），按钮靠右
- [x] 快捷键自定义 - 交互式录制 + 恢复默认按键
- [x] 工作区锁定 - 密码保护、遮罩页居中布局
- [x] 场景保存/加载 - 启动自动恢复
- [x] 终端工作目录持久化 - OSC shell 集成 (bash/zsh/powershell) + Linux /proc fallback
- [x] 历史指令菜单 - Alt 呼出、上下导航、Esc 关闭、Enter 写入并自动去重置顶
- [x] 终端焦点键盘隔离 - 上下方向键不会触发 Dock 折叠按钮焦点导航
- [x] Workspace 快捷键说明 - 左侧栏底部显示当前快捷键绑定（keycap 样式）
- [x] Workspace 操作图标 - Phosphor 图标，统一关闭/锁定按钮尺寸，操作列背景遮罩
- [x] Workspace 顺序拖拽 - 通过左侧手柄调整并自动持久化列表顺序
- [x] Workspace 拖拽反馈 - 拖拽源弱化、目标高亮并显示插入线
- [x] Workspace 关闭删除历史 - 关闭终端/工作区时删除对应的 SQLite 指令历史
- [x] 多国语言系统 - 9 种语言（中/繁中/英/德/法/日/意/韩/印地），YAML 资源嵌入二进制
- [x] 主题切换 - 浅色/深色主题菜单和设置持久化，中性选中配色
- [x] Workspace 侧栏控制 - F1 显示/隐藏侧栏，视图菜单勾选指示
- [x] 关于页面 - 版本信息、主页、开源地址、作者、致谢、检查更新
- [x] 在线自更新 - 检查 latest.json → 下载 portable 包 → SHA256 校验 → 平台替换脚本 → 重启
- [x] 跨平台兼容 - Windows/macOS/Linux 三平台，字体打包嵌入，数据目录迁移

## In Progress

- [ ] 命令补全 (PATH 扫描 + 历史频率)
- [ ] 监控面板

## Planned Features

- [ ] 同步功能
- [ ] 插件系统

## Technical Stack

- **语言**: Rust 2021 Edition
- **GUI 框架**: egui 0.31 / eframe 0.31
- **终端后端**: alacritty_terminal 0.25
- **Dock 布局**: egui_dock 0.16
- **图标**: egui-phosphor 0.9
- **数据库**: rusqlite 0.31 (SQLite, bundled)
- **文件对话框**: rfd 0.15
- **HTTP 客户端**: ureq 2 (自更新)
- **校验**: sha2 0.10
- **解压**: tar 0.4 + flate2 1.0 (tar.gz) / zip 0.6 (Windows zip)
- **国际化**: serde_yaml 0.9 + 嵌入式 YAML 资源

## Project Structure

```
opennex/
├── .github/workflows/
│   └── release.yml                # CI/CD：三平台编译 + R2 上传 + latest.json
├── assets/
│   ├── fonts/                     # 打包字体（CJK 19MB + Devanagari + Arabic）
│   └── desktop/                   # Linux .desktop 文件
├── egui_term_local/               # 终端模拟器库（本地 path 依赖）
│   ├── Cargo.toml
│   └── src/
│       ├── backend/               # PTY 后端封装
│       ├── bindings.rs             # 键盘绑定
│       ├── view.rs                 # 终端渲染
│       └── ...
├── locales/                       # 9 种语言 YAML 文件
│   ├── zh.yaml / en.yaml / zh-TW.yaml
│   ├── de.yaml / fr.yaml / ja.yaml
│   └── it.yaml / ko.yaml / hi.yaml
├── src/
│   ├── main.rs                    # 应用入口
│   ├── app.rs                     # 主应用逻辑（UI、事件、状态）
│   ├── updater.rs                 # 在线自更新模块
│   ├── theme.rs                   # 主题系统（浅色/深色）
│   ├── i18n.rs                    # 国际化系统
│   ├── history_db.rs              # SQLite 命令历史
│   ├── terminal/                  # 终端实例封装
│   └── completion/                # 命令补全（开发中）
├── tests/
│   └── terminal_test.rs           # 终端测试
├── Cargo.toml
├── Cargo.lock
└── project.md                     # 本文档
```

## Development

```bash
# 开发模式（debug 构建，快捷键自动用代码默认值覆盖 settings.json）
cargo run

# 运行测试
cargo test --lib

# Release 构建
cargo build --release

# 格式检查
cargo fmt --all -- --check
```

### Dev 模式说明

`cargo run` 使用 debug 构建 (`#[cfg(debug_assertions)]`)，每次启动会强制用代码中的默认快捷键覆盖 `settings.json`。开发者修改快捷键默认值后直接 `cargo run` 即可生效，无需手动删除配置文件。

Release 构建尊重用户已保存的配置，只在缺失时补入默认值。

## CI/CD 发布流程

### 前置条件

| 配置 | 位置 | 说明 |
|------|------|------|
| Cloudflare R2 桶 | `opennex-ci` | 存放版本文件 |
| R2 公开域名 | `opennex.download.zeadix.com` | CDN 下载地址 |
| GitHub Secret `CLOUDFLARE_API_TOKEN` | 仓库 Settings → Secrets | R2 + Pages Edit 权限 |
| GitHub Secret `CLOUDFLARE_ACCOUNT_ID` | 仓库 Settings → Secrets | Cloudflare 账户 ID |

### 发版步骤

```bash
# 1. 更新 Cargo.toml 中的 version（必须与 tag 一致）
#    version = "0.2.0"

# 2. 提交并推送
git add -A
git commit -m "release: v0.2.0"
git push origin dev:main

# 3. 打 tag（格式必须是 v*.*.*）
git tag v0.2.0
git push origin v0.2.0

# 4. CI 自动触发，约 15-20 分钟完成
```

### CI 产出

| 平台 | 便携版（自更新用） | 安装包（首次安装用） |
|------|-------------------|---------------------|
| Windows | `.zip` | `.msi` (WiX) |
| macOS | `.tar.gz` | `.dmg` (cargo-bundle) |
| Linux | `.tar.gz` | `.deb` (cargo-deb) |

### CI 流程

```
Tag v*.*.* push
  ↓
├── build-windows (windows-latest + MSVC)
├── build-macos (macos-latest)
└── build-linux (ubuntu-22.04)
  ↓
publish Job:
  1. 上传 6 个文件到 R2: opennex-ci/vX.Y.Z/
  2. 下载旧 latest.json，合并版本历史（保留最近 3 个）
  3. 删除超出 3 个的旧版本 R2 文件
  4. 上传新的 latest.json 到 R2: opennex-ci/latest.json
```

### R2 文件结构

```
opennex-ci/
├── latest.json                         ← 固定地址，永远是最新的
├── v0.1.2/
│   ├── opennex-v0.1.2-windows-x86_64.zip
│   ├── opennex-v0.1.2-windows-x86_64.msi
│   ├── opennex-v0.1.2-macos-x86_64.tar.gz
│   ├── opennex-v0.1.2-macos.dmg
│   ├── opennex-v0.1.2-linux-x86_64.tar.gz
│   └── opennex-v0.1.2-linux-amd64.deb
├── v0.1.1/
│   └── ...
└── v0.1.0/
    └── ...
```

### latest.json 格式

```json
{
  "version": "0.1.2",
  "date": "2026-08-12",
  "files": {
    "windows": { "portable": "...", "sha256": "...", "installer": "...", "installer_sha256": "..." },
    "macos": { ... },
    "linux": { ... }
  },
  "history": [
    { "version": "0.1.2", "date": "...", "files": { ... } },
    { "version": "0.1.1", "date": "...", "files": { ... } },
    { "version": "0.1.0", "date": "...", "files": { ... } }
  ]
}
```

- `version` / `files`：最新版本信息（应用自更新读这些字段）
- `history`：最近 3 个版本（官网展示历史版本下载）

### 版本历史管理

- 保留最近 **3 个**版本
- 超出自动删除 R2 中对应的旧版本文件
- 每个版本附带 **发布日期**

### 自更新流程

```
应用启动 → 后台请求 https://opennex.download.zeadix.com/latest.json
  ↓
比较 version 与 env!("CARGO_PKG_VERSION")
  ↓ (有新版本)
弹出提示 → 用户点击「更新」
  ↓
下载 portable 包 → SHA256 校验 → 解压
  ↓
平台替换脚本：
  Windows: .bat → 等待退出 → 替换 exe → 重启
  macOS/Linux: .sh → 等待退出 → 替换二进制 → 重启
```

### 验证 URL

| 用途 | URL |
|------|-----|
| 版本清单 | `https://opennex.download.zeadix.com/latest.json` |
| 下载文件 | `https://opennex.download.zeadix.com/v0.1.2/opennex-v0.1.2-linux-x86_64.tar.gz` |
| CI 状态 | `https://github.com/zeadix/opennex/actions` |

## 跨平台注意事项

| 平台 | Runner | 注意事项 |
|------|--------|---------|
| Windows | `windows-latest` + MSVC | ConPTY 需要 Win10 1809+ |
| macOS | `macos-latest` | DMG 未签名，用户需右键打开绕过 Gatekeeper |
| Linux | `ubuntu-22.04` | glibc >= 2.31 (Ubuntu 20.04+) |

### 数据目录

| 平台 | 路径 |
|------|------|
| Linux | `~/.config/opennex/` |
| Windows | `%APPDATA%\opennex\` |
| macOS | `~/Library/Application Support/opennex/` |

首次启动时自动从 CWD 迁移旧数据文件。

### 字体

以下字体通过 `include_bytes!` 打包嵌入二进制：
- `NotoSansCJK-Regular.ttc`（19MB）— 中/日/韩
- `Lohit-Devanagari.ttf`（152KB）— 印地语
- `NotoSansArabic-Regular.ttf`（235KB）— 阿拉伯语

系统字体扫描作为补充，追加到字体族末尾（不覆盖默认字体度量）。
