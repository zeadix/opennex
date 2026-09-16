import type { Lang } from "../i18n";

/** Help > 关于 — product card with version and links. */
export default function AboutPage({ lang }: { lang: Lang }) {
  const zh = lang === "zh" || lang === "zh-TW";
  return (
    <div className="h-full overflow-y-auto px-8 py-6">
      <div className="mx-auto max-w-[520px]">
        <div className="mb-6 flex items-center gap-3">
          <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-[var(--accent-dim)] font-mono text-[20px] font-bold text-[var(--accent)]">
            N
          </div>
          <div>
            <div className="glow-text text-[18px] font-bold">OpenNex</div>
            <div className="font-mono text-[11px] text-[var(--text-faint)]">v0.1.55-tauri</div>
          </div>
        </div>
        <p className="mb-4 text-[12.5px] leading-relaxed text-[var(--text-dim)]">
          {zh
            ? "现代化的终端管理器：多工作空间、可拖拽分屏、指令收藏与自动补全、SSH 连接、局域网/广域网远程控制、AI 助手。"
            : "A modern terminal manager: multi-workspace, draggable split panes, command favorites & auto-complete, SSH, LAN/WAN remote control and an AI assistant."}
        </p>
        <div className="card-glow rounded-xl border border-[var(--border)] bg-[var(--bg-panel)] p-4 text-[12px]">
          <div className="mb-2 font-semibold">{zh ? "技术栈" : "Tech stack"}</div>
          <div className="grid grid-cols-2 gap-1.5 text-[11px] text-[var(--text-dim)]">
            <span>Tauri v2 · Rust</span>
            <span>React 18 · TypeScript</span>
            <span>xterm.js · portable-pty</span>
            <span>flexlayout-react</span>
          </div>
        </div>
        <div className="mt-4 text-[11px] text-[var(--text-faint)]">
          {zh ? "项目主页：" : "Project home: "}
          <a
            className="text-[var(--accent)] hover:underline"
            href="https://github.com/zeadix/opennex"
            target="_blank"
            rel="noreferrer"
          >
            github.com/zeadix/opennex
          </a>
        </div>
        <div className="mt-1 text-[11px] text-[var(--text-faint)]">
          © 2026 OpenNex · {zh ? "本构建为 Tauri 预览版" : "This build is the Tauri preview"}
        </div>
      </div>
    </div>
  );
}
