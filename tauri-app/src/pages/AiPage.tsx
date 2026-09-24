import { useI18n } from '../i18n-context';
import { useEffect, useRef, useState, useSyncExternalStore, KeyboardEvent } from "react";
import { FiSend, FiCpu, FiPlay, FiSkipForward, FiPlus, FiTrash2, FiRefreshCw, FiClock } from "react-icons/fi";
import { invoke } from "../terminal/tauri";
import { focusedSlot, sendTo } from "../terminal/registry";
import { addAiTask, getAiTasks, removeAiTask, stopAllTasks, subscribeAiTasks } from "../aiTasks";
import {
  AI_MODES, DEFAULT_MODEL_LIMIT_KTOKENS, budgetTokens, estimateTokens, fitMessages,
  nextAiMode, parseAiMode, type AiPermissionMode,
} from "../aiChat";

interface Msg {
  role: "user" | "assistant" | "system";
  content: string;
}

export interface AiSource {
  id: string;
  name: string;
  baseUrl: string;
  apiKey: string;
}

export interface AiModelRef {
  /** `${sourceId}::${model}` */
  id: string;
  sourceId: string;
  model: string;
  /** 该模型的上下文预算（k token）；缺省用 DEFAULT_MODEL_LIMIT_KTOKENS。 */
  limitKTokens?: number;
}

export interface AiConfig {
  baseUrl: string;
  apiKey: string;
  model: string;
  sources: AiSource[];
  models: AiModelRef[];
  activeModelId: string;
  /** AI 全局权限模式。 */
  permissionMode: AiPermissionMode;
}

const CFG_KEY = "opennex-ai";
const DEF_BASE = "https://api.openai.com/v1";

function normalizeAiConfig(raw: any): AiConfig {
  const c: AiConfig = {
    baseUrl: DEF_BASE,
    apiKey: "",
    model: "gpt-4o-mini",
    permissionMode: "ask",
    ...raw,
  };
  c.permissionMode = parseAiMode(c.permissionMode);
  // 迁移：旧的单源字段 → sources/models（保留兼容）。
  if (!Array.isArray(c.sources) || c.sources.length === 0) {
    c.sources = [{ id: "default", name: "默认模型源", baseUrl: c.baseUrl || DEF_BASE, apiKey: c.apiKey || "" }];
  }
  if (!Array.isArray(c.models) || c.models.length === 0) {
    const m = String(c.model || "gpt-4o-mini").trim();
    c.models = m ? [{ id: "default::" + m, sourceId: "default", model: m }] : [];
  }
  // 限额字段消毒：只保留正数；兼容上一版字段名 limitKChars（本就仅
  // 存在数小时，值语义一并迁移为 k token）。
  c.models = c.models.map((m: any) => {
    const raw = m.limitKTokens ?? m.limitKChars;
    return { ...m, limitKTokens: Number(raw) > 0 ? Number(raw) : undefined };
  });
  if (!c.activeModelId || !c.models.some((m) => m.id === c.activeModelId)) {
    c.activeModelId = c.models[0]?.id ?? "";
  }
  return c;
}

export function loadAiConfig(): AiConfig {
  try {
    return normalizeAiConfig(JSON.parse(localStorage.getItem(CFG_KEY) ?? "{}"));
  } catch {
    return normalizeAiConfig({});
  }
}
export function saveAiConfig(c: AiConfig) {
  localStorage.setItem(CFG_KEY, JSON.stringify(c));
}

function normalizeBaseUrl(u: string): string {
  return String(u || "").replace(/\/+$/, "");
}

export default function AiPage({ uiSnapshot }: { uiSnapshot?: () => unknown }) {
  const T = useI18n();
  const [cfg, setCfg] = useState<AiConfig>(loadAiConfig);
  const persistCfg = (c: AiConfig) => {
    setCfg(c);
    saveAiConfig(c);
  };
  const [editing, setEditing] = useState(false);
  const [messages, setMessages] = useState<Msg[]>([]);
  const [input, setInput] = useState("");
  const [busy, setBusy] = useState(false);
  const [inserted, setInserted] = useState(-1);
  const [agentPlan, setAgentPlan] = useState<string[] | null>(null);
  const [agentStep, setAgentStep] = useState(0);
  const [agentLog, setAgentLog] = useState<string[]>([]);
  const bottomRef = useRef<HTMLDivElement>(null);
  const aiTasks = useSyncExternalStore(subscribeAiTasks, getAiTasks);

  // ---- 权限模式（咨询 / 询问 / 自由） ------------------------------------
  // consult: 纯聊天，无任何终端/任务操作；ask: 关键操作需确认；
  // free: 全部直接执行。Tab 在面板内循环切换。
  const mode: AiPermissionMode = cfg.permissionMode;
  const setMode = (m: AiPermissionMode) => persistCfg({ ...cfg, permissionMode: m });
  const modeLabel = (m: AiPermissionMode) =>
    m === "consult" ? T.uModeConsult : m === "ask" ? T.uModeAsk : T.uModeFree;
  const onTabCycle = (e: KeyboardEvent) => {
    if (e.key === "Tab") {
      e.preventDefault();
      setMode(nextAiMode(mode));
    }
  };
  // 询问模式下 @@ACTION 会操作终端的指令（延时/定时）的待确认队列。
  const [pendingSchedules, setPendingSchedules] = useState<PendingAction[]>([]);

  const activeModel = () => cfg.models.find((m) => m.id === cfg.activeModelId) ?? cfg.models[0] ?? null;
  const sourceOf = (m: AiModelRef | null | undefined) =>
    cfg.sources.find((s) => s.id === m?.sourceId) ?? cfg.sources[0] ?? null;

  // ---- 定时任务执行（AI 面板内可见的任务列表） ---------------------------
  const removeTask = (id: string) => removeAiTask(id);

  // ---- AI 控制界面：解析回复中的 @@ACTION 指令并按权限模式执行 ----------
  // consult: 全部阻止（仅提示）；ask: 会操作终端的指令（runAt 延时单次、
  // addSchedule 周期）进待确认队列，其余无害指令直接执行；free: 全部直接执行。
  type PendingAction = {
    kind: "once" | "repeat";
    command: string;
    slot: number | null;
    everySec?: number;
    delaySec?: number;
  };
  const pendingLabel = (p: PendingAction) =>
    p.kind === "once"
      ? `延后 ${p.delaySec ?? 0}s · ${p.command}`
      : `每 ${p.everySec ?? 0}s · ${p.command}`;
  const runActions = (
    content: string,
    mode: AiPermissionMode,
  ): { display: string; executed: string[]; pending: PendingAction[] } => {
    const executed: string[] = [];
    const rest: string[] = [];
    const pending: PendingAction[] = [];
    let blocked = false;
    for (const line of content.split("\n")) {
      const t = line.trim();
      if (!t.startsWith("@@ACTION")) {
        rest.push(line);
        continue;
      }
      if (mode === "consult") {
        blocked = true;
        continue;
      }
      try {
        const a = JSON.parse(t.slice("@@ACTION".length).trim());
        const slot = a.slot === undefined || a.slot === null ? focusedSlot.value : Number(a.slot);
        if (a.action === "runAt" && a.command && Number(a.delaySec) >= 0) {
          const spec: PendingAction = {
            kind: "once",
            command: String(a.command),
            slot,
            delaySec: Number(a.delaySec),
          };
          if (mode === "ask") {
            pending.push(spec);
          } else {
            addAiTask(spec);
            executed.push(`✓ 已安排：${spec.delaySec}s 后向终端 ${spec.slot} 执行「${spec.command}」`);
          }
        } else if (a.action === "addSchedule" && a.command && Number(a.everySec) > 0) {
          const spec: PendingAction = {
            kind: "repeat",
            command: String(a.command),
            slot,
            everySec: Number(a.everySec),
          };
          if (mode === "ask") {
            pending.push(spec);
          } else {
            addAiTask(spec);
            executed.push(`✓ 已创建定时任务：每 ${spec.everySec}s 向终端 ${spec.slot} 执行「${spec.command}」`);
          }
        } else if (a.action === "stopSchedule" && a.taskId) {
          removeAiTask(String(a.taskId));
          executed.push(`✓ 已停止定时任务 ${a.taskId}`);
        } else if (a.action === "switchModel" && a.model) {
          const m = cfg.models.find(
            (x) => x.model === a.model || x.id === a.model || x.id.endsWith("::" + a.model),
          );
          if (m) {
            persistCfg({ ...cfg, activeModelId: m.id });
            executed.push(`✓ 已切换模型 ${m.model}`);
          } else {
            executed.push(`✗ 未找到模型 ${a.model}`);
          }
        }
      } catch {
        /* 忽略无法解析的指令行 */
      }
    }
    let display = rest.join("\n").trimEnd();
    if (blocked) display += (display ? "\n\n" : "") + `⚠ ${T.uBlockedByConsult}`;
    return { display, executed, pending };
  };

  const send = async () => {
    const text = input.trim();
    if (!text || busy) return;
    const next: Msg[] = [...messages, { role: "user" as const, content: text }];
    setMessages(next);
    setInput("");
    setBusy(true);
    try {
      const am = activeModel();
      const src = sourceOf(am);
      if (!am || !src) throw new Error("未配置模型源");
      // 界面快照 + 可用模型/任务/操作规范：让 AI 能理解并控制界面。
      const snap = typeof uiSnapshot === "function" ? uiSnapshot() : null;
      const sys = [
        "你是 OpenNex 终端管理器内置的 AI 助手，可以控制软件界面。",
        snap ? "当前界面状态(JSON)：" + JSON.stringify(snap) : "",
        "已添加模型：" + (cfg.models.map((m) => m.model).join(", ") || "无"),
        "当前任务：" + (aiTasks.length ? JSON.stringify(aiTasks.map((t) => ({ id: t.id, kind: t.kind, everySec: t.everySec, runAtSec: t.runAtSec, command: t.command }))) : "无"),
        "要控制界面（延时执行、定时执行、停止任务、切换模型），在回复中用单独的行输出指令，每行一条，格式：",
        '@@ACTION {"action":"runAt","delaySec":5,"command":"top","slot":null}',
        '@@ACTION {"action":"addSchedule","everySec":20,"command":"ls","slot":null}',
        '@@ACTION {"action":"stopSchedule","taskId":"任务id"}',
        '@@ACTION {"action":"switchModel","model":"模型名"}',
        "runAt：延时单次执行，delaySec 秒后执行一次即自动移除。addSchedule：周期执行，每 everySec 秒一次直到停止。",
        "涉及多个时间点的编排（如「5 秒后启动 top，运行 3 秒停止，再过 3 秒打印当前目录」）时，必须按时间顺序拆成多条 runAt，delaySec 从现在起累加（例：5、8、11，命令依次为 top、q、pwd）。",
        "slot 为终端编号（null/省略 = 当前聚焦终端）。除指令行外，用用户的语言正常简洁回答；不要虚构执行结果。",
      ].filter(Boolean).join("\n");
      const reply = await invoke<string>("ai_chat", {
        baseUrl: src.baseUrl,
        apiKey: src.apiKey,
        model: am.model,
        // 上下文预算（按模型的 k token 限额，默认 150k）：system 永久
        // 保留，历史从最新往前整条装配，只发装得下的最近尾部。
        messages: [
          { role: "system", content: sys },
          ...fitMessages(next, 0, Math.max(budgetTokens(am.limitKTokens) - estimateTokens(sys), 0)),
        ],
      });
      const { display, executed, pending } = runActions(reply, mode);
      if (pending.length) setPendingSchedules((p) => [...p, ...pending]);
      const suffix = executed.length ? "\n\n" + executed.join("\n") : "";
      setMessages([...next, { role: "assistant" as const, content: display + suffix }]);
    } catch (e) {
      setMessages([...next, { role: "assistant" as const, content: `⚠ ${e}` }]);
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
    const am = activeModel();
    const src = sourceOf(am);
    if (!am || !src) return;
    setInput("");
    setBusy(true);
    setAgentPlan(null);
    setAgentStep(0);
    setAgentLog([T.uGoalLog.replace("{goal}", () => goal)]);
    try {
      const reply = await invoke<string>("ai_chat", {
        baseUrl: src.baseUrl,
        apiKey: src.apiKey,
        model: am.model,
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
      setAgentLog((l) => [...l, T.uPlanSteps.replace("{count}", String(cmds.length))]);
    } catch (e) {
      setAgentLog((l) => [...l, `⚠ ${T.uPlanFailed}: ${e}`]);
    } finally {
      setBusy(false);
    }
  };

  const runStep = (cmd: string) => {
    sendTo(focusedSlot.value, new TextEncoder().encode(cmd + "\r"));
    setAgentLog((l) => [...l, `▶ ${cmd}`]);
    setAgentStep((s) => s + 1);
  };

  /** 自由模式专用：把剩余计划一次性全部执行（每条仍是逐条写入并回车）。 */
  const runAll = () => {
    if (!agentPlan) return;
    const remaining = agentPlan.slice(agentStep);
    for (const cmd of remaining) {
      sendTo(focusedSlot.value, new TextEncoder().encode(cmd + "\r"));
    }
    setAgentLog((l) => [...l, ...remaining.map((c) => `▶ ${c}`)]);
    setAgentStep((s) => s + remaining.length);
  };

  // ---- 模型源 / 模型管理（配置页） --------------------------------------
  const [fetching, setFetching] = useState<Record<string, boolean>>({});
  const [fetched, setFetched] = useState<Record<string, string[]>>({});
  const [fetchErr, setFetchErr] = useState<Record<string, string>>({});

  const setSource = (i: number, patch: Partial<AiSource>) =>
    setCfg((c) => ({ ...c, sources: c.sources.map((s, k) => (k === i ? { ...s, ...patch } : s)) }));

  const addSource = () =>
    setCfg((c) => ({
      ...c,
      sources: [
        ...c.sources,
        { id: "src-" + Date.now().toString(36), name: "新模型源", baseUrl: "http://localhost:11434/v1", apiKey: "" },
      ],
    }));

  const removeSource = (i: number) => {
    const sid = cfg.sources[i]?.id;
    setCfg((c) => ({
      ...c,
      sources: c.sources.filter((_, k) => k !== i),
      models: c.models.filter((m) => m.sourceId !== sid),
      activeModelId:
        cfg.activeModelId && cfg.models.some((m) => m.id === cfg.activeModelId && m.sourceId !== sid)
          ? c.activeModelId
          : c.models.filter((m) => m.sourceId !== sid)[0]?.id ?? "",
    }));
  };

  const fetchModelsFor = async (sourceId: string) => {
    const s = cfg.sources.find((x) => x.id === sourceId);
    if (!s) return;
    setFetching((m) => ({ ...m, [sourceId]: true }));
    setFetchErr((m) => ({ ...m, [sourceId]: "" }));
    try {
      const list = await invoke<string[]>("ai_models", { baseUrl: s.baseUrl, apiKey: s.apiKey });
      setFetched((prev) => ({ ...prev, [sourceId]: list }));
    } catch (e) {
      setFetchErr((prev) => ({ ...prev, [sourceId]: String(e) }));
    } finally {
      setFetching((m) => ({ ...m, [sourceId]: false }));
    }
  };

  const toggleModel = (sourceId: string, model: string) => {
    const id = sourceId + "::" + model;
    const has = cfg.models.some((m) => m.id === id);
    const models = has
      ? cfg.models.filter((m) => m.id !== id)
      : [...cfg.models, { id, sourceId, model }];
    const activeModelId =
      cfg.activeModelId && models.some((m) => m.id === cfg.activeModelId)
        ? cfg.activeModelId
        : models[0]?.id ?? "";
    setCfg({ ...cfg, models, activeModelId });
    saveAiConfig({ ...cfg, models, activeModelId });
  };

  const removeModel = (id: string) => {
    const models = cfg.models.filter((m) => m.id !== id);
    const activeModelId = cfg.activeModelId === id ? models[0]?.id ?? "" : cfg.activeModelId;
    persistCfg({ ...cfg, models, activeModelId });
  };

  if (tab === "agent") {
    return (
      <div className="flex h-full flex-col" onKeyDown={onTabCycle}>
        <div className="flex items-center justify-between border-b border-[var(--border)] px-5 py-2.5">
          <span className="flex items-center gap-2 text-[13px] font-medium">
            <FiCpu size={15} className="text-[var(--accent)]" /> {T.agentTitle}
            <span className="rounded-full border border-[var(--border)] px-2 py-0.5 text-[10px] font-normal text-[var(--text-dim)]">
              {modeLabel(mode)}
            </span>
          </span>
          <button className="text-[12px] text-[var(--text-dim)] hover:text-[var(--accent)]" onClick={() => setTab("chat")}>
            {T.uChatMode}
          </button>
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto px-5 py-4">
          <div className="mb-3 text-[12px] text-[var(--text-dim)]">
            {T.uAgentHint}
          </div>
          <div className="flex gap-2">
            <input
              className="dialog-input"
              placeholder={T.uGoalExample}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && !busy && planGoal()}
              style={{ userSelect: "text" }}
            />
            <button className="shrink-0 rounded-md bg-[var(--accent-dim)] px-4 text-[12px] text-[var(--accent)] hover:brightness-125 disabled:opacity-50"
              disabled={busy || !input.trim()} onClick={planGoal}>
              {busy ? T.uPlanning : T.uGeneratePlan}
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
                  {i === agentStep && mode !== "consult" && (
                    <>
                      {/* 询问模式：逐步「执行」即逐条确认；自由模式额外提供全部执行。 */}
                      <button className="flex items-center gap-1 rounded-md bg-[var(--accent-dim)] px-2.5 py-1 text-[11px] text-[var(--accent)]"
                        onClick={() => { runStep(cmd); }}>
                        <FiPlay size={11} /> {T.uExecute}
                      </button>
                      {mode === "free" && (
                        <button className="flex items-center gap-1 rounded-md border border-[var(--border)] px-2.5 py-1 text-[11px] text-[var(--text-dim)] hover:text-[var(--accent)]"
                          onClick={runAll}>
                          {T.uRunAll}
                        </button>
                      )}
                      <button className="flex items-center gap-1 rounded-md border border-[var(--border)] px-2.5 py-1 text-[11px] text-[var(--text-dim)]"
                        onClick={() => { setAgentLog((l) => [...l, `⊘ ${T.uSkip}: ${cmd}`]); setAgentStep((s) => s + 1); }}>
                        <FiSkipForward size={11} /> {T.uSkip}
                      </button>
                    </>
                  )}
                  {i === agentStep && mode === "consult" && (
                    <span className="text-[11px] text-[var(--text-faint)]">{T.uAgentBlocked}</span>
                  )}
                </div>
              ))}
              {agentStep >= agentPlan.length && (
                <div className="rounded-md border border-[var(--success)] px-3 py-2 text-[12px] text-[var(--success)]">
                  {T.uPlanDone}
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
      <div className="h-full overflow-y-auto px-6 py-5">
        <div className="mx-auto max-w-[560px] space-y-4">
          <h2 className="text-[15px] font-semibold">{T.uAiSettings}</h2>

          {/* ── 模型源管理 ── */}
          <div className="space-y-3">
            {cfg.sources.map((s, i) => {
              const list = fetched[s.id] ?? null;
              const err = fetchErr[s.id] ?? "";
              const loading = !!fetching[s.id];
              return (
                <div key={s.id} className="space-y-2 rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-3">
                  <div className="flex items-center gap-2">
                    <input
                      className="dialog-input !w-28"
                      placeholder={T.uModelHint}
                      value={s.name}
                      onChange={(e) => setSource(i, { ...s, name: e.target.value })}
                    />
                    <button
                      className="flex shrink-0 items-center gap-1 rounded-md border border-[var(--border)] px-2.5 py-1 text-[11px] text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:opacity-50"
                      disabled={loading}
                      title="从 /models 接口获取可用模型"
                      onClick={() => fetchModelsFor(s.id)}
                    >
                      <FiRefreshCw size={11} className={loading ? "animate-spin" : ""} /> 获取模型列表
                    </button>
                    {cfg.sources.length > 1 && (
                      <button
                        className="ml-auto text-[var(--text-faint)] transition-colors hover:text-[var(--danger)]"
                        title="删除此模型源"
                        onClick={() => removeSource(i)}
                      >
                        <FiTrash2 size={13} />
                      </button>
                    )}
                  </div>
                  <input
                    className="dialog-input"
                    placeholder="API Base URL（如 https://api.openai.com/v1）"
                    value={s.baseUrl}
                    onChange={(e) => setSource(i, { ...s, baseUrl: e.target.value })}
                  />
                  <input
                    className="dialog-input"
                    type="password"
                    placeholder="API Key（本地服务可留空）"
                    value={s.apiKey}
                    onChange={(e) => setSource(i, { ...s, apiKey: e.target.value })}
                  />
                  {err && <div className="text-[11px] text-[var(--danger)]">⚠ {err}</div>}
                  {list && (
                    <div className="max-h-40 space-y-0.5 overflow-y-auto rounded-md border border-[var(--border)] bg-[var(--bg)] p-1.5">
                      {list.length === 0 && <div className="px-2 py-1 text-[11px] text-[var(--text-faint)]">（空）</div>}
                      {list.map((m) => {
                        const id = s.id + "::" + m;
                        const added = cfg.models.some((x) => x.id === id);
                        return (
                          <label
                            key={m}
                            className="flex cursor-pointer items-center gap-2 rounded px-2 py-1 font-mono text-[11px] text-[var(--text-dim)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--text)]"
                          >
                            <input
                              type="checkbox"
                              checked={added}
                              onChange={() => toggleModel(s.id, m)}
                              className="accent-[var(--accent)]"
                            />
                            <span className="min-w-0 flex-1 truncate">{m}</span>
                          </label>
                        );
                      })}
                    </div>
                  )}
                </div>
              );
            })}
            <button
              className="flex w-full items-center justify-center gap-1.5 rounded-md border border-dashed border-[var(--border)] px-3 py-1.5 text-[12px] text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]"
              onClick={addSource}
            >
              <FiPlus size={12} /> 添加模型源
            </button>
          </div>

          {/* ── 已添加模型（对话中可切换）── */}
          <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] p-3">
            <div className="mb-2 text-[11px] font-semibold tracking-wider text-[var(--text-faint)]">
              已添加模型（对话中可切换）
            </div>
            <div className="flex flex-wrap gap-2">
              {cfg.models.length === 0 && (
                <span className="text-[11px] text-[var(--text-faint)]">尚未添加模型</span>
              )}
              {cfg.models.map((m) => (
                <span
                  key={m.id}
                  className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 font-mono text-[11px] ${
                    cfg.activeModelId === m.id
                      ? "border-[var(--accent)] text-[var(--accent)]"
                      : "border-[var(--border)] text-[var(--text-dim)]"
                  }`}
                >
                  {m.model}
                  <label
                    className="flex items-center gap-1 font-sans text-[10px] text-[var(--text-faint)]"
                    title={T.uModelLimitHint}
                  >
                    {T.uModelLimit}
                    <input
                      type="number"
                      min={4}
                      max={2000}
                      className="w-14 rounded border border-[var(--border)] bg-[var(--bg-panel)] px-1 text-[11px] text-[var(--text)] outline-none"
                      value={m.limitKTokens ?? ""}
                      placeholder={String(DEFAULT_MODEL_LIMIT_KTOKENS)}
                      onChange={(e) => {
                        const v = parseInt(e.target.value, 10);
                        const models = cfg.models.map((x) =>
                          x.id === m.id
                            ? { ...x, limitKTokens: Number.isFinite(v) && v > 0 ? v : undefined }
                            : x,
                        );
                        persistCfg({ ...cfg, models });
                      }}
                    />
                    {/* 单位注明：填 150 即 150k token，不是 150 个字符 */}
                    <span className="font-mono">k</span>
                  </label>
                  <i
                    className="not-italic text-[var(--text-faint)] transition-colors hover:text-[var(--danger)]"
                    title="移除"
                    onClick={() => removeModel(m.id)}
                  >
                    ✕
                  </i>
                </span>
              ))}
            </div>
            <p className="mt-2 text-[11px] leading-relaxed text-[var(--text-faint)]">
              勾选获取的模型列表即可添加；对话中可随时切换。
            </p>
          </div>

          <div className="flex justify-end gap-2">
            <button
              className="rounded-md px-3 py-1.5 text-[12px] text-[var(--text-dim)] hover:text-[var(--text)]"
              onClick={() => setEditing(false)}
            >
              {T.uBack}
            </button>
            <button
              className="rounded-md bg-[var(--accent-dim)] px-3 py-1.5 text-[12px] text-[var(--accent)] hover:brightness-125"
              onClick={() => { saveAiConfig(cfg); setEditing(false); }}
            >
              {T.save}
            </button>
          </div>
        </div>
      </div>
    );
  }

  // ---- 对话页 ------------------------------------------------------------
  return (
    <div className="flex h-full flex-col" onKeyDown={onTabCycle}>
      <div className="flex items-center justify-between gap-2 border-b border-[var(--border)] px-5 py-2.5">
        <select
          className="min-w-0 max-w-[60%] rounded-md border border-[var(--border)] bg-[var(--bg-panel)] px-2 py-1 text-[12px] text-[var(--text)] outline-none"
          value={cfg.activeModelId}
          onChange={(e) => persistCfg({ ...cfg, activeModelId: e.target.value })}
        >
          {cfg.models.map((m) => {
            const src = sourceOf(m);
            return (
              <option key={m.id} value={m.id}>
                {m.model} · {src?.name ?? ""}
              </option>
            );
          })}
        </select>
        <button className="shrink-0 text-[12px] text-[var(--text-dim)] hover:text-[var(--accent)]"
          onClick={() => setEditing(true)}>{T.uConfigure}</button>
      </div>

      {/* 定时任务列表（AI 创建的延时/定时执行命令），按时间顺序展示 */}
      {aiTasks.length > 0 && (
        <div className="border-b border-[var(--border)] px-5 py-2">
          <div className="mb-1 flex items-center gap-1.5 text-[11px] font-semibold text-[var(--text-dim)]">
            <FiClock size={11} /> 定时任务（{aiTasks.length}）
            <button
              className="ml-auto font-normal text-[var(--danger)] hover:underline"
              onClick={() => stopAllTasks()}
            >
              全部停止
            </button>
          </div>
          <div className="space-y-1">
            {aiTasks.map((t) => (
              <div key={t.id} className="flex items-center gap-2 rounded-md border border-[var(--border)] bg-[var(--bg-panel)] px-2.5 py-1.5 text-[11px]">
                <span className={`h-1.5 w-1.5 shrink-0 rounded-full ${t.enabled ? "bg-[var(--success)]" : "bg-[var(--text-faint)]"}`} />
                <span className="min-w-0 flex-1 truncate">{t.label}</span>
                <span className="shrink-0 rounded border border-[var(--border)] px-1 text-[10px] text-[var(--text-faint)]">
                  {t.kind === "once" ? `单次 · ${t.slot ?? focusedSlot.value}` : `周期 · ${t.everySec}s`}
                </span>
                <button className="shrink-0 text-[var(--danger)] hover:underline" onClick={() => removeAiTask(t.id)}>停止</button>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="min-h-0 flex-1 overflow-y-auto px-5 py-4">
        {messages.length === 0 && (
          <div className="pt-16 text-center text-[12px] text-[var(--text-faint)]">
            {T.uChatEmpty}
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
                  className="mt-1 rounded-md border border-[var(--border)] px-2 py-1 text-[11px] text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-40"
                  disabled={mode === "consult"}
                  title={mode === "consult" ? T.uBlockedByConsult : undefined}
                  onClick={() => {
                    const firstLine = m.content.split("\n").find((l) => l.trim()) ?? "";
                    const ok = sendTo(focusedSlot.value, new TextEncoder().encode(firstLine));
                    setInserted(ok ? i : -1);
                  }}
                >
                  ⤓ {T.insertToTerminal}{inserted === i ? " ✓" : ""}
                </button>
              )}
            </div>
          ))}
          {busy && (
            <div className="text-[12px] text-[var(--text-faint)]">{T.uThinking}</div>
          )}
          <div ref={bottomRef} />
        </div>
      </div>
      {/* 询问模式：AI 请求创建定时任务时的确认条 */}
      {pendingSchedules.length > 0 && (
        <div className="border-t border-[var(--border)] bg-[var(--bg-panel)] px-5 py-2">
          <div className="mb-1 text-[11px] text-[var(--text-dim)]">{T.uConfirmSchedule}</div>
          <div className="space-y-0.5">
            {pendingSchedules.map((p, i) => (
              <div key={i} className="truncate font-mono text-[11px] text-[var(--text-dim)]">
                {pendingLabel(p)} → 终端 {p.slot ?? focusedSlot.value}
              </div>
            ))}
          </div>
          <div className="mt-1.5 flex gap-2">
            <button
              className="rounded-md bg-[var(--accent-dim)] px-3 py-1 text-[11px] text-[var(--accent)] hover:brightness-125"
              onClick={() => {
                for (const p of pendingSchedules) {
                  addAiTask(p);
                }
                setPendingSchedules([]);
              }}
            >
              {T.uAllow}
            </button>
            <button
              className="rounded-md border border-[var(--border)] px-3 py-1 text-[11px] text-[var(--text-dim)] hover:text-[var(--danger)]"
              onClick={() => setPendingSchedules([])}
            >
              {T.uDeny}
            </button>
          </div>
        </div>
      )}

      <div className="border-t border-[var(--border)] p-3">
        <div className="flex items-center gap-2 rounded-lg border border-[var(--border)] bg-[var(--bg-panel)] px-3 py-2 focus-within:border-[var(--accent)]">
          {/* 权限模式下拉：与输入框同高，左侧；Tab 在面板聚焦时循环切换 */}
          <select
            className="shrink-0 self-stretch rounded-md border border-[var(--border)] bg-[var(--bg)] px-1.5 text-[11px] text-[var(--text)] outline-none"
            value={mode}
            onChange={(e) => setMode(e.target.value as AiPermissionMode)}
          >
            {AI_MODES.map((m) => (
              <option key={m} value={m}>
                {modeLabel(m)}
              </option>
            ))}
          </select>
          <input
            className="min-w-0 flex-1 bg-transparent text-[13px] outline-none"
            placeholder={T.uAskHint}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && !e.shiftKey && send()}
            style={{ userSelect: "text" }}
          />
          <button className="icon-btn" title={T.uSend} onClick={send} disabled={busy}>
            <FiSend size={15} />
          </button>
        </div>
      </div>
    </div>
  );
}
