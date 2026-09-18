import type { Lang } from "../i18n";
import { useI18n } from "../i18n-context";

const STEPS: { zh: [string, string]; en: [string, string] }[] = [
  {
    zh: ["工作空间", "左侧导航栏可新建工作空间（自带一个终端），或从模板创建完全相同布局的副本。右键任意工作空间可重命名、锁定、保存为模板或删除。"],
    en: ["Workspaces", "Create workspaces (each starts with one terminal) or duplicate a layout from a template in the left navigation bar. Right-click a workspace to rename, lock, save as template or delete."],
  },
  {
    zh: ["终端分屏", "终端区域内拖拽标签即可分屏，"+" 按钮新建终端，可指定不同的 Shell。调整布局后其余面板自动补位，不会留空白。"],
    en: ["Split panes", "Drag terminal tabs to split the area; the + button opens new terminals with any shell. Panels backfill automatically — no blank gaps."],
  },
  {
    zh: ["菜单栏", "工作空间菜单保存/加载/另存为布局；视图菜单开关导航栏与工作区；远程控制提供局域网与广域网入口。"],
    en: ["Menu bar", "The Workspace menu saves/loads/exports layouts; View toggles the nav and work area; Remote offers LAN and WAN access."],
  },
  {
    zh: ["效率功能", "Ctrl+F 搜索终端内容；输入时自动匹配历史指令，Tab 补全；Alt 呼出历史指令浮层；广播模式可同时向多个终端输入。"],
    en: ["Productivity", "Ctrl+F searches terminal output; history auto-matches while typing (Tab completes); Alt opens the history overlay; broadcast mode types into many terminals at once."],
  },
  {
    zh: ["远程控制", "远程控制页展示局域网地址与二维码，手机扫码即可查看和操作终端；广域网模式通过 Cloudflare 隧道把同一页面暴露到公网。"],
    en: ["Remote control", "The Remote page shows a LAN URL and QR code for phone access; WAN mode exposes the same page via a Cloudflare quick tunnel."],
  },
  {
    zh: ["快捷键与主题", "设置窗口中可录制全局快捷键、切换九种语言与五套主题，终端字号支持 Ctrl+滚轮实时调整。"],
    en: ["Shortcuts & themes", "Record global shortcuts, switch nine languages and five themes in Settings; Ctrl+wheel adjusts the terminal font size live."],
  },
];

/** Help > 教程 — quick-start steps. */
export default function TutorialPage({ lang }: { lang: Lang }) {
  const T = useI18n();
  const zh = lang === "zh" || lang === "zh-TW";
  const titles = [T.sTut1Title, T.sTut2Title, T.sTut3Title, T.sTut4Title, T.sTut5Title, T.sTut6Title];
  return (
    <div className="h-full overflow-y-auto px-8 py-6">
      <div className="mx-auto max-w-[560px]">
        <h2 className="mb-1 text-[15px] font-semibold">{T.sQuickStart}</h2>
        <p className="mb-4 text-[12px] text-[var(--text-dim)]">
          {T.sQuickStartSub}
        </p>
        <div className="space-y-3">
          {STEPS.map((s, i) => {
            const [, body] = zh ? s.zh : s.en;
            const title = titles[i];
            return (
              <div key={i} className="card-glow flex gap-3 rounded-xl border border-[var(--border)] bg-[var(--bg-panel)] p-4">
                <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg bg-[var(--accent-dim)] font-mono text-[12px] font-bold text-[var(--accent)]">
                  {i + 1}
                </div>
                <div>
                  <div className="mb-1 text-[13px] font-semibold">{title}</div>
                  <div className="text-[12px] leading-relaxed text-[var(--text-dim)]">{body}</div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
