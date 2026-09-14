import { useRef, useState } from "react";
import { FiSend } from "react-icons/fi";
import { invoke } from "../terminal/tauri";

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
            <div key={i} className={`flex ${m.role === "user" ? "justify-end" : "justify-start"}`}>
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
