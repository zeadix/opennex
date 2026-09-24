// AI 权限模式与上下文预算：纯逻辑，供 AiPage 使用（无 React 依赖）。

// ---------------------------------------------------------------- 权限模式

/** AI 全局权限模式（设置持久化 id：consult | ask | free）。 */
export type AiPermissionMode = "consult" | "ask" | "free";

export const AI_MODES: AiPermissionMode[] = ["consult", "ask", "free"];

/** Tab 循环切换的下一个模式。 */
export function nextAiMode(m: AiPermissionMode): AiPermissionMode {
  return AI_MODES[(AI_MODES.indexOf(m) + 1) % AI_MODES.length];
}

/** 从持久化值恢复模式；未知/缺省回退到安全的中间档。 */
export function parseAiMode(v: unknown): AiPermissionMode {
  return AI_MODES.includes(v as AiPermissionMode) ? (v as AiPermissionMode) : "ask";
}

// ------------------------------------------------------------ 上下文预算

/** 模型上下文预算默认值（k token）；即最近约 150k token 的对话随请求上传。 */
export const DEFAULT_MODEL_LIMIT_KTOKENS = 150;

/**
 * 粗略 token 估算：CJK 字符（汉字/假名/谚文等宽字符）≈ 1 token，
 * 其余（拉丁字母、数字、符号）≈ 4 字符/token。混合文本够用作预算。
 */
export function estimateTokens(text: string): number {
  let wide = 0;
  let narrow = 0;
  for (const ch of text) {
    if ((ch.codePointAt(0) ?? 0) >= 0x2e80) wide++;
    else narrow++;
  }
  return wide + Math.ceil(narrow / 4);
}

/** 模型的上下文预算（token）；未设置/非法值回退默认 150k。 */
export function budgetTokens(limitKTokens: number | undefined): number {
  const k = Number(limitKTokens) > 0 ? Number(limitKTokens) : DEFAULT_MODEL_LIMIT_KTOKENS;
  return k * 1000;
}

/**
 * 把对话装配进 token 预算（按 estimateTokens 计）。前 keepFirst 条
 * （system、目标）永久保留；其余预算从最新一条往前整条取用，停在第一
 * 条放不下的消息上，因此保留的是连续的最近尾部。最新一条即使单独超
 * 预算也必发——没有最新轮次的请求没有意义。
 */
export function fitMessages<T extends { content: string }>(
  messages: T[],
  keepFirst: number,
  budget: number,
): T[] {
  const head = messages.slice(0, keepFirst);
  const rest = messages.slice(keepFirst);
  let used = head.reduce((n, m) => n + estimateTokens(m.content), 0);
  let start = rest.length;
  for (let i = rest.length - 1; i >= 0; i--) {
    const len = estimateTokens(rest[i].content);
    if (start < rest.length && used + len > budget) break;
    used += len;
    start = i;
  }
  return [...head, ...rest.slice(start)];
}
