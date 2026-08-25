# OpenNex — Product Context

## What
OpenNex 是一个跨平台（Windows/macOS/Linux）堆叠式终端管理器：多工作区、分屏终端、会话命令记忆、可配置快捷键、主题系统、在线自更新。Rust + egui 0.31 桌面应用，alacritty_terminal 后端。

## Who
- 主画像 Alex：开发者/运维，键盘流，开多终端跑 TUI（vim/opencode/htop），要求效率与低干扰
- 次画像 Jordan：轻度命令行用户，鼠标流，需要可发现的操作路径与温和的引导

## Register
产品型界面（design serves the product）。基调：**专业克制**——信息密度可高但必须有层级，动效仅用于反馈而非装饰，颜色来自主题系统不做临时发挥。对标产品：Alacritty 的性能专注 + Windows Terminal 的管理能力。

## Design Ground Rules
- 所有 UI 颜色走主题 token（app.accent/app.danger/...），禁止硬编码
- 交互控件必须 hover/pressed/focus 三态
- 字号层级：11（辅助）/12（正文）/13（强调）/14（标题），间距 4 的倍数
- 弹窗一律可 Esc 关闭；危险操作有确认；反馈走 toast（可读不依赖时长）
- i18n：9 语言全量覆盖，禁止 UI 层硬编码文案
