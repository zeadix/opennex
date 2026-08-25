# OpenNex 架构与功能审查报告

日期：2026-08-25 · 基线：v0.1.33 (cbd55eb)
方法：四路并行审计（架构/性能测试CI/并发IO/安全），全部结论附 file:line 证据。

---

## 一、总体结论

核心链路健康：线程模型简单无死锁、shell 启动无注入面、SQL 全参数化、粘贴仅写 PTY stdin、主题系统有版本校验+原子写+备份。

三大结构性弱点：
1. **数据持久化脆弱** —— 退出不存场景 + 非原子写入 + 损坏静默降级，三者叠加可造成用户布局永久丢失；
2. **自更新信任链薄弱** —— 同源 sha256 无签名、无回滚、macOS 静默失败；
3. **渲染层空转** —— 空闲持续满帧率 + 每帧全 Grid 深拷贝 + 全屏 re-tessellate。

工程最大债务：src/app.rs 8767 行（占主 crate 60%）、App 结构体 ~128 字段、update() 约 2772 行。

---

## 二、问题清单

### 🔴 高危

| # | 问题 | 证据 |
|---|---|---|
| H1 | **退出不保存场景**：eframe::App 未实现 on_close_event，菜单退出直接 ViewportCommand::Close；save_scene 仅 Ctrl+S/菜单/工作区拖拽 3 个触发点；更新重启路径也不保存 | app.rs:4843-4868, 5531, 2760-2783 |
| H2 | **配置写入非原子**：scene.json/settings.json 直接 fs::write；断电截断→解析失败→静默回默认值→autosave 覆盖=永久丢失。theme/store.rs 已有 tmp+bak 正确范式未复用 | app.rs:1474-1481, 1037-1041; theme/store.rs:99-133 |
| H3 | **工作区锁密码明文**存储于 settings.json，解锁为明文比较 | app.rs:18-24, 1037-1041, 6660-6662, 7483 |
| H4 | **自更新无独立信任锚**：sha256 与产物同源同桶同 CI 运行生成（只防传输截断）；替换旧二进制无回滚（坏版本=变砖）；macOS 更新必然失败且无提示（take_last_update_failure cfg 排除 macOS） | updater.rs:7, 165-181, 454, 503-527; release.yml 无 signtool/codesign |
| H5 | **空闲持续满帧率渲染**：聚焦终端每帧无条件 request_repaint（光标闪烁用墙钟取模而非 request_repaint_after）；叠加每帧整 Grid 深拷贝（~万行×80列≈十几 MB）+ 每格 Shape::text 全屏重 tessellate | view.rs:548-552, 573-669, 674; backend/mod.rs:289 |

### 🟠 中危

| # | 问题 | 证据 |
|---|---|---|
| M1 | SQLite 打不开/损坏 .expect panic 循环；无 integrity_check/损坏隔离；双实例 prune 互删对方历史行 | history_db.rs:52, 106-113, 127+; app.rs:1779 |
| M2 | 启动更新检查结果只在第 180 帧（~3s）消费一次，弱网 HTTP>3s 用户永远看不到新版提示 | app.rs:1528-1543, 6422-6429 |
| M3 | Windows UAC helper TOCTOU：固定名 %TEMP% 脚本在写入与 UAC 启动间可被同账户恶意软件替换提权；PS 单引号未转义、bash 双引号内 $/反引号未处理；install_writable 预检测错对象（测 exe 而非目录） | updater.rs:306-386, 408-420, 440-487 |
| M4 | Linux 多用户 /tmp 固定名脚本符号链接攻击（fs::write 跟随 symlink） | updater.rs:297, 437-491 |
| M5 | UI 线程周期性重活：proc_stats 每 2s 全进程表扫描（Windows 可达百 ms）；rebuild_fonts 同步读盘全部字体文件——主题编辑器拖滑块时每帧重建图集 | app.rs:4883-4927, 2706-2710, 3480-3638 |
| M6 | poll_cwd 每 15 帧×每终端强制 set_dirty→持锁全量克隆 Grid 并停摆 PTY 读线程，输出洪泛时帧抖动 | app.rs:4872-4878; terminal/mod.rs:413-416; backend/mod.rs:281-294 |
| M7 | 死设置欺骗用户："滚动回溯"设置从未接入 alacritty term::Config（固定 10000 行），调小不省内存 | app.rs:2923-2937; backend/mod.rs:166 |
| M8 | completion 死子系统 ~203 行零调用，启动却读盘建库 | src/completion/*; app.rs:1298, 7594 |
| M9 | CI 缺陷：仅 tag 触发、三平台只 build 不 test、无 fmt/clippy 门禁、产物未签名；macos-latest(ARM64) 产物误标 x86_64 | release.yml:104, 121, 178-179 |

### 🟡 低危

- 粘贴无 bracketed-paste 包裹（多行剪贴板逐行执行的经典注入面）— view.rs:774-780
- 终端链接打开失败即 panic!（远程文本+一次点击=崩溃）— backend/mod.rs:345-347
- zip 0.6 已停更有 CVE 史（应升 2.x）；未使用依赖 egui-elegance、serde_derive
- 版本比较把 0.2.0-rc1 当正式版推给全量用户 — updater.rs:65-85
- 解压无大小上限（zip 炸弹）；Windows zip 取首个 *.exe 不验名 — updater.rs:136-160, 193-205
- OSC cwd 靠网格文本刮取，可被 echo '9;/x'\a 伪造记录 — terminal/mod.rs:444-464
- save_settings 失败静默吞错 ≥12 处 — app.rs:2099, 2278, 2298 等
- i18n serde(default) 字段缺失时静默变空串（当前 9 语言实测零缺失，但无自动 parity 测试）
- HistoryNav 定义在 app.rs 被 terminal 层引用（唯一反向依赖）— terminal/mod.rs:11
- DEFAULT_SHELL_ID 全局 static RwLock 绕过参数传递 — app.rs:8
- PTY Drop 阻塞 wait 无超时，忽略 SIGHUP 的子进程可致订阅线程泄漏（每关一个漏一个）— backend/mod.rs:664-668
- opennex_update_failed.txt 标记无写入方；perform_update 未被调用（死代码）

---

## 三、可优化项（按性价比排序）

### 快赢（低成本高回报）
1. 光标闪烁改 request_repaint_after(blink_period)，空闲归零 CPU — view.rs:548-552
2. poll_cwd 不再无条件 set_dirty（增量扫描或仅在已有 dirty 时读快照）— terminal/mod.rs:414
3. 启动检查改"≥180 帧且结果就绪"条件 — app.rs:6422 一行
4. scene/settings 写入复用 theme 的 tmp+rename+bak；加载失败备份坏文件并 toast — 半小时工作量
5. on_close_event 保存场景 + 更新重启前保存 — 数据丢失直接消除
6. 删死依赖 egui-elegance/serde_derive；zip 升 2.x
7. 删除或真正接通 completion 子系统
8. scrollback 设置接入 term::Config（约 20 行）
9. save_settings 失败统一走 log+toast helper 替换 12 个调用点
10. 快捷键表每帧双份 clone 改 OnceLock/Arc 预建 — app.rs:4950, 7618

### 中期工程
- proc_stats 移到采样线程（mpsc 回传，与 updater 同模式）— app.rs:4883
- rebuild_fonts 结果缓存（路径扫描+字节缓存），主题编辑器仅在字体相关字段变化时重建 — app.rs:2706, 3480
- SQLite：错误降级日志+内存兜底、启动 integrity_check 失败改名隔离重建、prune 只清 N 天孤儿
- updater：PS `'`→`''` 转义、bash printf %q、随机临时目录、保留 opennex_old 回滚、macOS osascript 流程或明确手动提示
- CI 加 push/PR test+clippy+fmt 门禁；修正 macos runner 命名
- ed25539/minisign 对 latest.json 签名（私钥离线，公钥内嵌）
- 工作区锁密码 argon2id 哈希化
- bracketed paste 支持（200~ 包裹）
- update() 第一刀拆分：render_history_menu（666 行）抽成 history_menu.rs；HistoryNav 移入 terminal 模块

### 架构演进（长期）
- app.rs 按职责拆分：menu_bar / settings_ui / workspace_sidebar / shortcuts / update_flow 五个模块，App 字段分组进子结构体（每步可编译可回归）
- fork 上游同步策略：egui_term_local 记录 diff 清单，评估以 feature 分支 PR 上游减少维护面

---

## 四、后续功能建议

### 短期（补齐现有价值）
1. **终端缓冲区搜索**（Ctrl+F 高亮+跳转）——终端管理器高频刚需，alacritty 有 search 支持基础
2. **命令补全真正接线**——completion 子系统已有 PATH 扫描+历史频率骨架，激活后配合历史菜单形成闭环
3. **监控面板**（project.md 已规划）——先做 proc_stats 线程化再上 UI，状态栏 CPU/MEM 扩展为每终端资源占用
4. **多行粘贴确认对话框**——防剪贴板劫持误执行（比 bracketed paste 更直观的用户保护）
5. **场景自动保存**（防抖 500ms）+ 崩溃恢复提示（检测到 .bak 时询问恢复）

### 中期（差异化竞争力）
6. **SSH 连接管理器**——主机簿（地址/端口/用户/密钥）、标签页直连、复用分屏布局；远程运维是堆叠终端的核心场景，建议用系统 keyring 存凭据
7. **会话恢复增强**——重启后按记录的 cwd 重开终端并可选重放命令（数据已采集：OSC cwd + 历史库）
8. **广播输入**——向同工作区多个终端同时发送输入（运维批量操作刚需）
9. **代码片段库**——收藏指令升级为可分组/参数占位符的片段面板
10. **每终端 Profile**——字体/主题/环境变量/启动命令按终端配置，覆盖开发多环境需求

### 长期（project.md 已规划方向）
11. **插件系统**——建议 Lua/WASM 沙箱跑 UI 侧栏小工具与命令扩展；先定义稳定 IPC 边界再开放
12. **同步功能**——工作区布局/片段/主机簿端到端加密同步（可先做"导出加密档案文件"轻量版）

---

## 五、确认安全项（无需处理）

粘贴路径仅 PTY stdin ✓ · shell argv 数组直传无拼接注入 ✓ · bash init 仅静态 chain-load ✓ · 主题强校验+原子写 ✓ · SQL 参数化 ✓ · 无硬编码密钥、网络出口仅 R2/GitHub ✓ · 运行期无僵尸进程（SIGCHLD+try_wait 收割）✓
