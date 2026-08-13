# 主窗口极简重设计 Spec

## 目标

把 OpenNex 主窗口（菜单栏、工作区侧栏、底部状态条）从"功能堆叠"改造为"极简现代"风格——视觉接近参考站 `http://localhost:4321/zh/` 控制台示例：
- 大量留白
- 几乎无可见边框（仅 1px 分隔线）
- 配色完全从已选主题读取
- 极简度优先于"功能可见度"

**只改样式，功能不变**——终端多 tab、分屏、拖拽、设置/关于弹窗都保留。

## 范围

**新主题字段**：
- `menu_bg` / `menu_fg`：菜单栏背景/文字
- `button_bg` / `button_fg` / `button_hover_bg`：按钮背景/文字/hover 背景
- `sidebar_border`：侧栏右边缘线颜色

**UI 改动**：
- `TopBottomPanel::top` 菜单栏（极简，无边框）
- `SidePanel::left` 工作区侧栏（可拖拽 120-300px）
- `TopBottomPanel::bottom` 底部状态条（24px 高，显示 3 段小字）
- 中央 `CentralPanel` 终端区（仅去掉外框/边框，逻辑不变）

**保留**：
- 锁屏覆盖层、设置窗口、主题对话框、更新对话框、About 窗口、关闭工作区确认弹窗——所有现有弹窗样式和功能不变
- DockArea 多 tab/分屏/拖拽逻辑不变
- 6 套内置主题不变（仅补充新字段）

## 数据模型

### `AppTheme` 新增 6 字段（`src/theme/model.rs`）

```rust
pub struct AppTheme {
    // ... 现有 16 字段保持不变 ...
    
    #[serde(default = "default_menu_bg")]
    pub menu_bg: ThemeColor,
    #[serde(default = "default_menu_fg")]
    pub menu_fg: ThemeColor,
    #[serde(default = "default_button_bg")]
    pub button_bg: ThemeColor,
    #[serde(default = "default_button_fg")]
    pub button_fg: ThemeColor,
    #[serde(default = "default_button_hover_bg")]
    pub button_hover_bg: ThemeColor,
    #[serde(default = "default_sidebar_border")]
    pub sidebar_border: ThemeColor,
}
```

6 个 `default_*()` 函数返回基于现有 `panel`/`text`/`input_bg`/`hover`/`border` 派生出的合理值——`#[serde(default)]` 让旧主题 JSON 文件自动兼容。

### 6 套主题新字段默认值

| 主题 | menu_bg | menu_fg | button_bg | button_fg | button_hover_bg | sidebar_border |
|---|---|---|---|---|---|---|
| opennex-dark | 略深 panel | 现有 text | 现有 input_bg | 现有 text | 现有 hover | 现有 border |
| opennex-light | 同 | 同 | 同 | 同 | 同 | 同 |
| opennex-noir | 同 | 同 | 同 | 同 | 同 | 同 |
| solarized-dark | 同 | 同 | 同 | 同 | 同 | 同 |
| gruvbox-dark | 同 | 同 | 同 | 同 | 同 | 同 |
| dracula | 同 | 同 | 同 | 同 | 同 | 同 |

**实现**：6 套主题 JSON 各加 6 字段（值由现有字段派生）。`#[serde(default)]` 兼容，所以即使某个文件忘记加也不会 panic——会从 default 拿。

## UI 详细规范

### 菜单栏（`TopBottomPanel::top("menu_bar")`）

```
高度: 30px
背景: ui.theme.app.menu_bg
文字: ui.theme.app.menu_fg
无边框 / 无阴影
菜单项: ui.menu_button("文件") / "视图" / "语言" / "主题"
  内部 ui.selectable_label 或 ui.button
  hover 背景: ui.theme.app.button_hover_bg
下拉:
  - 用 egui::Frame::popup 但去掉 .shadow
  - 内边距 4px
  - 文字按钮（不是图标）
```

**菜单项布局**：
```
文件 → [保存场景] [加载场景] [另存为] [退出]
视图 → [向右分屏] [向下分屏] [工作区侧栏]
语言 → 中文 / English / 日本語 / ...（自动多语言列表）
主题 → [OpenNex Dark] [OpenNex Light] [OpenNex Noir] ... [管理主题...]  (滚动)
```

右侧附加项（不放在下拉中）：
- 当前激活主题名（弱化字号 11pt，紧贴菜单栏右侧）
- "ⓘ" 关于按钮
- "≡" 设置按钮

### 工作区侧栏（`SidePanel::left("navigation")`）

```rust
egui::SidePanel::left("navigation")
    .resizable(true)
    .default_width(192.0)
    .width_range(120.0..=300.0)
    .show(ctx, |ui| { ... });
```

**布局（顶到底）**：

```
30px 顶 [+] [模板▼]
8px 间距
[工作区列表]
  每个项 28px 高：
    12px  ▸/▾ 展开/折叠
    16px  [图标] 名字  ← 整行点击选中
                  右侧 hover 出现 ⋯
                  双击或 ⋯ 菜单触发重命名（行内编辑）
8px 间距
[空态文本"无工作区"]（无工作区时显示）
flex 弹性空间
24px 底 [3 终端] [6% CPU] [8 GB]            [ⓘ]
```

**侧栏视觉**：
```
背景: ui.theme.app.sidebar
右边线: 1px 宽，颜色 ui.theme.app.sidebar_border
顶/底 padding: 8px
行高: 28px (固定)
```

**工作区项**：
- 选中：背景 `ui.theme.app.active`
- hover：背景 `ui.theme.app.hover`
- 图标：Phosphor regular（FOLDER 或 DESKTOP 图标）
- 名字：14pt 常规；选中加粗
- 重命名：行内可编辑 `TextEdit`，Enter 确认、Esc 取消
- 三点菜单项：
  - 重命名（行内编辑）
  - 锁定/解锁
  - 删除（二次确认弹窗保留）

**空态**：无工作区时显示居中弱化文本"无工作区，点 + 创建"。

**模板下拉**：
- 保留现有 `cached_template_files` 机制
- 模板列表项点击加载
- 每项右侧有 `×` 删除按钮

### 底部状态条（`TopBottomPanel::bottom("status_bar")`）

```rust
egui::TopBottomPanel::bottom("status_bar")
    .resizable(false)
    .exact_height(24.0)
    .show(ctx, |ui| { ... });
```

**视觉**：
```
高度: 24px
背景: ui.theme.app.menu_bg（同菜单栏）
文字: 12pt 弱化色 ui.theme.app.weak_text
上边线: 1px, ui.theme.app.sidebar_border
padding: 0 8px
```

**内容（3 段，竖线分隔）**：
```
"3 终端" │ "6% CPU" │ "8 GB"        [ⓘ]
```

`3 终端` = 当前所有工作区终端总数
`6% CPU` = 系统 CPU（用 `sysinfo` crate 或 `psutil` 替代方案——若未引入，固定显示 "--%"）
`8 GB` = 进程内存占用（用 `/proc/self/statm` Linux 或 `mach_task_basic_info` macOS，Windows 用 `GetProcessMemoryInfo`）

`ⓘ` = 打开设置窗口（复用现有 `show_settings = true`）

**自动刷新**：每 2 秒一次（用 `std::time::Instant` + `std::thread` 已经有的 poller）。

### 中央终端区

**仅样式改动**：
- `CentralPanel` 不加 `Frame`（让终端背景直接显示主题 `terminal.background`）
- 不在终端区加 padding/边框
- DockArea 的 tab bar 用 `ui_theme.app.*` 颜色（如果 egui_dock 0.16 允许自定义）

**不动**：
- 终端创建/关闭/分屏/拖拽逻辑
- 锁屏覆盖层（保留在 `is_locked` 中央面板）

## 依赖

**新依赖**（按需）：
- `sysinfo = "0.30"`（仅 Linux/macOS 启用；Windows 用 `windows` feature 或自定义）
- 若 `sysinfo` 与现有 `Cargo.toml` 冲突，回退方案：每 2 秒读取 `/proc/self/statm`（仅 Linux），其他平台显示 `--`

## 错误处理

- 6 套主题新字段缺失 → `#[serde(default)]` 兜底，**不 panic**
- sysinfo 获取失败 → 显示 `--`
- 拖拽侧栏宽度超界 → `width_range` 自动限制

## 测试

- 6 套主题 JSON 都能解析（包括旧版无新字段的）
- 切换主题时菜单栏/侧栏/状态条颜色立即更新
- 主题编辑器 UI 能编辑新 6 字段
- 底部状态条 2 秒刷新
- 拖拽侧栏宽度限制 120-300px

## 文件清单

| 文件 | 改动 |
|---|---|
| `src/theme/model.rs` | `AppTheme` 加 6 字段 + defaults |
| `src/theme/mod.rs` | 重新应用主题时也用新字段 |
| `assets/themes/opennex-dark.json` | 加 6 字段 |
| `assets/themes/opennex-light.json` | 加 6 字段 |
| `assets/themes/opennex-noir.json` | 加 6 字段 |
| `assets/themes/solarized-dark.json` | 加 6 字段 |
| `assets/themes/gruvbox-dark.json` | 加 6 字段 |
| `assets/themes/dracula.json` | 加 6 字段 |
| `src/app.rs` | `show_window_chrome()` 重写：菜单/侧栏/状态条；`App` 加 CPU/内存缓存 |
| `src/theme/ui.rs` | `apply_theme_definition` 用新字段 |
| `Cargo.toml` | 可能加 `sysinfo` 依赖（条件） |

## 风险

1. **新主题字段** → 6 套 JSON 都要补充，漏补不会 panic（`#[serde(default)]`），但需要 CI 验证
2. **侧栏拖拽** → `SidePanel::resizable(true)` 已经支持
3. **sysinfo 跨平台** → Windows 实现复杂；如失败固定显示 `--`
4. **DockArea 自定义** → egui_dock 0.16 API 可能限制 tab bar 样式

## 成功标准

1. 菜单栏/侧栏/状态条都使用主题颜色
2. 6 套主题都能加载，新字段缺失自动用 default
3. 底部状态条 2 秒自动刷新
4. 拖拽侧栏 120-300px
5. 切换主题时所有 UI 区域颜色立即更新
6. 终端功能（多 tab、分屏、拖拽）完全不变
