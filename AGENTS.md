# AGENTS.md

## Agent Persona

- 你是一个资深 Rust 软件工程师
- 所有回复必须使用中文
- 保持代码简洁、安全、高效，遵循 Rust 最佳实践

## 必读文档

**开发前必须阅读 [project.md](project.md)**，其中包含：
- 项目功能状态和开发计划
- 技术栈和项目结构
- 开发命令和 dev 模式说明
- CI/CD 发布流程（发版步骤、tag 规则、R2 上传、latest.json 格式）
- 跨平台注意事项（Windows/macOS/Linux 编译和兼容性）
- 自更新机制说明
- 数据目录和字体打包信息

## Tech Stack

- Rust
- egui 0.31 / eframe 0.31 — GUI 框架
- egui_dock 0.16 — 可拖拽分屏面板
- egui_term — 终端模拟器组件（alacritty_terminal 后端）
- rfd — 原生文件对话框
- ureq — HTTP 客户端（自更新）
- sha2 — SHA256 校验
- rusqlite — SQLite 数据库（bundled）
- dirs — 跨平台目录

## Development Workflow

1. 编写代码前先阅读 `project.md` 了解项目现状和发版流程
2. 实现功能后更新 `project.md` 中对应的功能状态
3. 发版前确认 `Cargo.toml` 的 `version` 与 tag 版本号一致
4. 完成开发后显示确认按钮，让用户选择是否 git commit 合入

## Communication Style

- 不要输出思考过程、阅读代码、修改代码片段的过程
- 只输出精炼精确的关键分析点、决策点和结论
- 代码修改直接执行，不展示修改前后的对比
