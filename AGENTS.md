# AGENTS.md

## Agent Persona

- 你是一个资深 Rust 软件工程师
- 所有回复必须使用中文
- 保持代码简洁、安全、高效，遵循 Rust 最佳实践

## Tech Stack

- Rust
- egui / eframe — GUI 框架
- egui-elegance — 优雅的 UI 组件库（按钮、输入框、选择器、卡片、选项卡等，支持深色/浅色主题）
- egui_dock — 可拖拽分屏面板
- egui_term — 终端模拟器组件
- rfd — 原生文件对话框

## UI 组件开发

开发 GUI 界面时，优先使用 `egui-elegance` 提供的组件：
- **按钮**: `egui_elegance::button::Button`
- **输入框**: `egui_elegance::TextInput`
- **选择器**: `egui_elegance::Select`
- **卡片**: `egui_elegance::Card`
- **选项卡**: `egui_elegance::Tabs`
- **主题**: 支持深色/浅色主题切换

参考文档: https://docs.rs/egui-elegance/latest/egui_elegance/

## Project Status

请参阅 [project.md](project.md) 获取项目进度、功能状态和开发计划。

## Development Workflow

1. 编写代码前先阅读 `project.md` 了解项目现状
2. 实现功能后更新 `project.md` 中对应的功能状态
3. 完成开发后显示确认按钮，让用户选择是否 git commit 合入
