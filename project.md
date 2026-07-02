# Project: OpenZoo

## Overview

跨平台的 Rust 控制台管理器，专门用于运行和管理多个 AI 编程工具的终端窗口。

## Requirements

请参阅 [request.md](request.md) 获取详细需求文档。

## Implemented Features

- [x] 基础终端管理 - 状态: 已实现
- [x] 多终端切换 - 状态: 已实现
- [x] 基础快捷键 - 状态: 已实现
- [x] 简单布局 - 状态: 已实现
- [x] 分屏布局系统 - 状态: 已实现
- [x] 拖拽调整分屏 - 状态: 已实现
- [x] 嵌套选项卡系统 - 状态: 已实现
- [x] 选项卡管理 - 状态: 已实现
- [x] 布局配置持久化 - 状态: 已实现
- [x] 选项卡配置持久化 - 状态: 已实现
- [x] 命令历史 - 状态: 已实现
- [x] 布局预设模板 - 状态: 已实现
- [x] 插件系统 - 状态: 已实现
- [x] 模板系统 - 状态: 已实现

## In Progress

- [ ] 监控面板 - 进度: 0%

## Planned Features

- [ ] 同步功能

## Technical Stack

- Language: Rust
- TUI Framework: ratatui
- Terminal Backend: crossterm
- Async Runtime: tokio
- Serialization: serde

## Development Notes

所有主要功能已完成，包括：
- 项目基础结构
- 终端管理模块
- 快捷键系统
- UI 渲染基础
- 配置管理
- 状态管理
- 分屏布局系统
- 嵌套选项卡系统
- 拖拽调整分屏
- 布局预设模板
- 插件系统
- 模板系统
- 持久化功能
