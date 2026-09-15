import { useRef, useState } from "react";
import { FiSend, FiCpu, FiPlay, FiSkipForward } from "react-icons/fi";
import { invoke } from "../terminal/tauri";
import { focusedSlot, sendTo } from "../terminal/registry";

interface Msg {
  role: "user" | "assistant";
  content: string;
}

interface AiConfig {
  baseUrl: string;
  apiKey: string;
  model: string;
}

const CFG_KEY = "opennex-ai";

export function loadAiConfig(): AiConfig {
  try {
    return {
      baseUrl: "https://api.openai.com/v1",
      apiKey: "",
      model: "gpt-4o-mini",
      ...JSON.parse(localStorage.getItem(CFG_KEY) ?? "{}"),
    };
  } catch {
    return { baseUrl: "https://api.openai.com/v1", apiKey: "", model: "gpt-4o-mini" };
  }
}
export function saveAiConfig(c: AiConfig) {
  localStorage.setItem(CFG_KEY, JSON.stringify(c));
}

export default function AiPage() {
  const [cfg, setCfg] = useState<AiConfig>(loadAiConfig);
  const [editing, setEditing] = useState(false);
  const [messages, setMessages] = useState<Msg[]>([]);
  const [input, setInput] = useState("");
  const [busy, setBusy] = useState(false);
  const [inserted, setInserted] = useState(-1);
  const [agentPlan, setAgentPlan] = useState<string[] | null>(null);
  const [agentStep, setAgentStep] = useState(0);
  const [agentLog, setAgentLog] = useState<string[]>([]);
  const bottomRef = useRef<HTMLDivElement>(null);

  const send = async () => {
    const text = input.trim();
    if (!text || busy) return;
    const next = [...messages, { role: "user" as const, content: text }];
    setMessages(next);
    setInput("");
    setBusy(true);
    try {
      const reply = await invoke<string>("ai_chat", {
        baseUrl: cfg.baseUrl,
        apiKey: cfg.apiKey,
        model: cfg.model,
        messages: next,
      });
      setMessages([...next, { role: "assistant", content: reply }]);
    } catch (e) {
      setMessages([...next, { role: "assistant", content: `⚠ ${e}` }]);
    } finally {
      setBusy(false);
      bottomRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  };

  // ---- agent mode -------------------------------------------------------
  const [tab, setTab] = useState<"chat" | "agent">("chat");

  const planGoal = async () => {
    const goal = input.trim();
    if (!goal || busy) return;
    setInput("");
    setBusy(true);
    setAgentPlan(null);
    setAgentStep(0);
    setAgentLog([`目标: ${goal}`]);
    try {
      const reply = await invoke<string>("ai_chat", {
        baseUrl: cfg.baseUrl,
        apiKey: cfg.apiKey,
        model: cfg.model,
        messages: [
          {
            role: "system",
            content:
              '你是终端命令规划器。根据用户目标输出一个 JSON 对象：{"commands": ["cmd1", "cmd2", ...]}。每个命令是单行 shell 命令，按执行顺序排列。只输出 JSON，不要其他内容。',
          },
          { role: "user", content: goal },
        ],
      });
      const parsed = JSON.parse(reply.replace(/^[\s\S]*?```json\s*|```[\s\S]*$/g, "").trim() || reply.slice(reply.indexOf("{")));
      const cmds: string[] = Array.isArray(parsed.commands) ? parsed.commands : [String(parsed)];
      setAgentPlan(cmds);
      setAgentStep(0);
      setAgentLog((l) => [...l, `计划 ${cmds.length} 步`]);
    } catch (e) {
      setAgentLog((l) => [...l, `⚠ 计划失败: ${e}`]);
    } finally {
      setBusy(false);
    }
  };

  const runStep = (cmd: string) => {
    sendTo(focusedSlot.value, new TextEncoder().encode(cmd + "\r"));
    setAgentLog((l) => [...l, `▶ ${cmd}`]);
    setAgentStep((s) => s + 1);
  };

  if (tab === "agent") {
    return (
      <div className="flex h-full flex-col">
        <div className="flex items-center justify-between border-b border-[var(--border)] px-5 py-2.5">
          <span className="flex items-center gap-2 text-[13px] font-medium">
            <FiCpu size={15} className="text-[var(--accent)]" /> Agent · 命令规划执行
          </span>
          <button className="text-[12px] text-[var(--text-dim)] hover:text-[var(--accent)]" onClick={() => setTab("chat")}>
            ← 对话模式
          </button>
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto px-5 py-4">
          <div className="mb-3 text-[12px] text-[var(--text-dim)]">
            输入目标 → AI 生成命令计划 → 你逐步批准执行（写入最近聚焦的终端并回车）
          </div>
          <div className="flex gap-2">
            <input
              className="dialog-input"
              placeholder="例如: 在 /tmp 搭建一个 python venv 并安装 requests"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && !busy && planGoal()}
              style={{ userSelect: "text" }}
            />
            <button className="shrink-0 rounded-md bg-[var(--accent-dim)] px-4 text-[12px] text-[var(--accent)] hover:brightness-125 disabled:opacity-50"
              disabled={busy || !input.trim()} onClick={planGoal}>
              {busy ? "规划中…" : "生成计划"}
            </button>
          </div>
          {agentPlan && (
            <div className="mt-4 space-y-2">
              {agentPlan.map((cmd, i) => (
                <div key={i} className={`flex items-center gap-2 rounded-md border px-3 py-2 ${
                  i < agentStep ? "border-[var(--border)] opacity-50" : "border-[var(--accent)]"
                }`}>
                  <span className="font-mono text-[11px] text-[var(--text-faint)]">#{i + 1}</span>
                  <span className="min-w-0 flex-1 truncate font-mono text-[12px]">{cmd}</span>
                  {i === agentStep && (
                    <>
                      <button className="flex items-center gap-1 rounded-md bg-[var(--accent-dim)] px-2.5 py-1 text-[11px] text-[var(--accent)]"
                        onClick={() => { runStep(cmd); }}>
                        <FiPlay size={11} /> 执行
                      </button>
                      <button className="flex items-center gap-1 rounded-md border border-[var(--border)] px-2.5 py-1 text-[11px] text-[var(--text-dim)]"
                        onClick={() => { setAgentLog((l) => [...l, `⊘ 跳过: ${cmd}`]); setAgentStep((s) => s + 1); }}>
                        <FiSkipForward size={11} /> 跳过
                      </button>
                    </>
                  )}
                </div>
              ))}
              {agentStep >= agentPlan.length && (
                <div className="rounded-md border border-[var(--success)] px-3 py-2 text-[12px] text-[var(--success)]">
                  计划执行完毕 ✓
                </div>
              )}
            </div>
          )}
          {agentLog.length > 0 && (
            <div className="mt-4 rounded-md border border-[var(--border)] bg-[var(--bg-panel)] p-3 font-mono text-[11px] text-[var(--text-dim)]">
              {agentLog.map((l, i) => <div key={i}>{l}</div>)}
            </div>
          )}
        </div>
      </div>
    );
  }

  if (editing) {
    return (
      <div className="h-full overflow-y-auto px-8 py-6">
        <div className="mx-auto max-w-[520px] space-y-3">
          <h2 className="text-[15px] font-semibold">AI 设置</h2>
          <input className="dialog-input" placeholder="API Base URL（OpenAI 兼容）"
            value={cfg.baseUrl}
            onChange={(e) => setCfg({ ...cfg, baseUrl: e.target.value })} />
          <input className="dialog-input" type="password" placeholder="API Key"
            value={cfg.apiKey}
            onChange={(e) => setCfg({ ...cfg, apiKey: e.target.value })} />
          <input className="dialog-input" placeholder="模型名（如 gpt-4o-mini）"
            value={cfg.model}
            onChange={(e) => setCfg({ ...cfg, model: e.target.value })} />
          <div className="flex justify-end gap-2">
            <button className="rounded-md px-3 py-1.5 text-[12px] text-[var(--text-dim)] hover:text-[var(--text)]"
              onClick={() => setEditing(false)}>返回</button>
            <button className="rounded-md bg-[var(--accent-dim)] px-3 py-1.5 text-[12px] text-[var(--accent)] hover:brightness-125"
              onClick={() => { saveAiConfig(cfg); setEditing(false); }}>保存</button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between border-b border-[var(--border)] px-5 py-2.5">
        <span className="text-[13px] font-medium">AI 助手 · {cfg.model}</span>
        <button className="text-[12px] text-[var(--text-dim)] hover:text-[var(--accent)]"
          onClick={() => setEditing(true)}>配置</button>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto px-5 py-4">
        {messages.length === 0 && (
          <div className="pt-16 text-center text-[12px] text-[var(--text-faint)]">
            输入问题开始对话 · 需要 OpenAI 兼容 API
          </div>
        )}
        <div className="space-y-3">
          {messages.map((m, i) => (
            <div key={i} className={`flex flex-col ${m.role === "user" ? "items-end" : "items-start"}`}>
              <div
                className={`max-w-[80%] whitespace-pre-wrap rounded-lg px-3.5 py-2.5 text-[13px] leading-relaxed ${
                  m.role === "user"
                    ? "bg-[var(--accent-dim)] text-[var(--text)]"
                    : "border border-[var(--border)] bg-[var(--bg-panel)]"
                }`}
                style={m.role === "assistant" && m.content.startsWith("⚠") ? { color: "var(--danger)" } : undefined}
              >
                {m.content}
              </div>
              {m.role === "assistant" && !m.content.startsWith("⚠") && (
                <button
                  className="mt-1 rounded-md border border-[var(--border)] px-2 py-1 text-[11px] text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]"
                  onClick={() => {
                    const firstLine = m.content.split("\n").find((l) => l.trim()) ?? "";
                    const ok = sendTo(focusedSlot.value, new TextEncoder().encode(firstLine));
                    setInserted(ok ? i : -1);
                  }}
                >
                  ⤓ 插入终端{inserted === i ? " ✓" : ""}
                </button>
              )}
            </div>
          ))}
          {busy && (
            <div className="text-[12px] text-[var(--text-faint)]">思考中…</div>
          )}
          <div ref={bottomRef} />
        </div>
      </div>
      <div className="border-t border-[var(--border)] p-3">
        <div className="flex items-center gap-2 rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] px-3 py-2 focus-within:border-[var(--accent)]">
          <input
            className="min-w-0 flex-1 bg-transparent text-[13px] outline-none"
            placeholder="问点什么…"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && !e.shiftKey && send()}
            style={{ userSelect: "text" }}
          />
          <button className="icon-btn" onClick={send} disabled={busy}>
            <FiSend size={15} />
          </button>
        </div>
      </div>
    </div>
  );
}
