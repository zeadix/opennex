# Project: OpenZoo

## Overview

跨平台的终端管理器，基于 Tauri + xterm.js 重构，用于运行和管理多个 AI 编程工具的终端窗口。

## Requirements

请参阅 [request.md](request.md) 获取详细需求文档。

## Implemented Features (Tauri v2 迁移版)

- [x] 基础终端管理 - xterm.js + portable-pty，支持多独立终端会话
- [x] 多终端切换 - flexlayout-react Tab 切换
- [x] 基础快捷键 - 全局 keydown 绑定，7 个可配置动作
- [x] 分屏布局系统 - flexlayout-react 水平/垂直分屏
- [x] 拖拽调整分屏 - flexlayout-react 原生拖拽
- [x] 嵌套选项卡系统 - flexlayout-react TabSet 嵌套
- [x] 选项卡管理 - 新建、关闭、重命名
- [x] 布局配置持久化 - flexlayout Model → JSON → scene.json
- [x] 命令历史 - Rust SQLite 后端 + 前端 hook
- [x] 布局预设模板 - templates/*.json 模板管理
- [x] 模板系统 - 保存/加载/删除工作区模板
- [x] 工作区管理 - 侧边栏多工作区、拖拽排序
- [x] 设置面板 - 4 Tab（通用/外观/快捷键/锁定）
- [x] 快捷键自定义 - 交互式录制
- [x] 工作区锁定 - 密码保护
- [x] 场景保存/加载 - 启动自动恢复
- [x] PTY 通信 - Tauri IPC 事件驱动
- [x] 终端启动首帧重绘 - PTY 输出可及时刷新到多个终端面板
- [x] 终端工作目录持久化 - 保存 shell 当前 cwd，启动时按 cwd 恢复
- [x] 历史指令菜单 - Alt 呼出、上下导航、Esc 关闭、Enter 写入并自动去重置顶
- [x] 终端焦点键盘隔离 - 上下方向键不会触发 Dock 折叠按钮焦点导航
- [x] Workspace 快捷键说明 - 左侧栏底部显示当前快捷键绑定
- [x] Workspace 操作图标 - 使用 Phosphor 图标并统一关闭/锁定按钮尺寸
- [x] Workspace 顺序拖拽 - 通过左侧手柄调整并自动持久化列表顺序
- [x] Workspace 拖拽反馈 - 拖拽源弱化、目标高亮并显示插入线
- [x] Workspace 拖拽手柄防选中 - 图标使用独立绘制响应区，侧栏默认宽度调整为 192px
- [x] Workspace 重命名确认 - 提供确定/取消按钮并支持 Esc 取消
- [x] Workspace 重命名焦点 - 编辑时输入框独占焦点并优先处理 Esc
- [x] Workspace 单终端保护 - 仅有一个 terminal 时隐藏 tab 关闭按钮
- [x] 多国语言系统 - locales/ YAML 资源、Language 菜单实时切换、UI 文字标签全部国际化
- [x] 主题切换 - 浅色/深色主题菜单和设置持久化
- [x] Workspace 侧栏控制 - Tab 显示/隐藏侧栏、Ctrl+Tab 输入制表符、操作列背景遮罩和菜单宽度优化

## In Progress

- [ ] 命令补全 (PATH 扫描 + 历史频率)
- [x] 历史命令浮层
- [ ] 监控面板
- [ ] 深色主题优化 (flexlayout-react)

## Planned Features

- [ ] 同步功能
- [ ] 插件系统

## Technical Stack

- **窗口框架**: Tauri 2.x
- **前端**: React 18 + TypeScript + Vite
- **终端**: xterm.js + @xterm/addon-fit
- **布局**: flexlayout-react
- **图标**: react-icons
- **状态管理**: zustand
- **PTY 后端**: portable-pty (Rust)
- **数据库**: rusqlite (SQLite, bundled)
- **通信**: @tauri-apps/api (invoke + event)

## Project Structure

```
open-zoo/
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── main.rs             # Tauri 入口
│   │   ├── lib.rs              # Tauri setup + 全部 commands
│   │   ├── pty.rs              # PTY 管理器 (portable-pty)
│   │   ├── history.rs          # SQLite 命令历史
│   │   └── state.rs            # 设置/场景/模板持久化
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                        # React 前端
│   ├── main.tsx                # 入口
│   ├── App.tsx                 # 主布局 (flexlayout + sidebar)
│   ├── components/
│   │   ├── Terminal.tsx        # xterm.js 封装
│   │   ├── WorkspaceSidebar.tsx # 左侧工作区面板
│   │   └── SettingsModal.tsx   # 设置弹窗
│   ├── store/
│   │   ├── workspace.ts        # 工作区状态 (zustand)
│   │   ├── terminal.ts         # 终端状态 + PTY 事件监听
│   │   └── settings.ts         # 设置状态
│   ├── types/index.ts          # TypeScript 类型定义
│   └── vite-env.d.ts
├── package.json
├── tsconfig.json
├── vite.config.ts
└── index.html
```

## Development

```bash
# 开发模式
pnpm tauri dev

# 生产构建
pnpm tauri build
```

## Migration Notes

从 egui/eframe + alacritty_terminal 迁移到 Tauri + xterm.js:
- 移除了 egui, eframe, egui_dock, egui-elegance, vte, alacritty_terminal, uuid
- 保留了 portable-pty, rusqlite, serde
- 新增 tauri, tauri-plugin-dialog, React, xterm.js, flexlayout-react, zustand
- 通信从共享内存 (FairMutex) 改为 Tauri IPC (invoke + event)
- layout 从 egui_dock 手动管理改为 flexlayout-react 自动序列化
