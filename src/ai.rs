//! AI assistant: settings model and an OpenAI-compatible
//! chat-completions client (works with any compatible endpoint, including
//! a local Ollama server at http://localhost:11434/v1).
//!
//! Pure logic lives here; the floating panel UI lives in app.rs. The
//! HTTP call is blocking and always runs on a background thread, with
//! the result handed back through a channel.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User-supplied AI configuration. The API key is stored in the local
/// settings.json ONLY — it is never sent anywhere except the configured
/// endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

/// Context budget for models WITHOUT a per-model entry in the settings
/// map, in kilo-tokens: the budget answers "how many tokens of RECENT
/// conversation are sent with the request". 150 means 150k tokens.
pub const DEFAULT_MODEL_LIMIT_KTOKENS: usize = 150;

/// Rough token estimate: CJK characters (han/kana/hangul and other wide
/// codepoints) ≈ 1 token each, everything else ≈ 4 chars per token.
/// Good enough for a budget; no real tokenizer.
pub fn estimate_tokens(text: &str) -> usize {
    let wide = text.chars().filter(|c| (*c as u32) >= 0x2e80).count();
    let narrow = text.chars().count() - wide;
    wide + narrow.div_ceil(4)
}

/// Context budget in TOKENS for `model`: the per-model override
/// (kilo-tokens, 0 = unset) or [`DEFAULT_MODEL_LIMIT_KTOKENS`].
pub fn context_budget_tokens(limits: &HashMap<String, usize>, model: &str) -> usize {
    limits
        .get(model)
        .copied()
        .filter(|k| *k > 0)
        .unwrap_or(DEFAULT_MODEL_LIMIT_KTOKENS)
        .saturating_mul(1000)
}

/// Fit a conversation into a token budget (measured with
/// [`estimate_tokens`]). The FIRST `keep_first` messages (system prompt,
/// agent goal) are always kept; the rest of the budget is filled from
/// the NEWEST message backwards, taking whole messages only and stopping
/// at the first one that no longer fits, so the kept tail stays
/// contiguous. The newest message is always taken even when it alone
/// exceeds the budget — a request without its latest turn is useless.
pub fn fit_messages(
    messages: &[ChatMessage],
    keep_first: usize,
    budget_tokens: usize,
) -> Vec<ChatMessage> {
    let keep_first = keep_first.min(messages.len());
    let mut out: Vec<ChatMessage> = messages[..keep_first].to_vec();
    let mut used: usize = out.iter().map(|m| estimate_tokens(&m.content)).sum();
    let rest = &messages[keep_first..];
    let mut start = rest.len();
    for i in (0..rest.len()).rev() {
        let len = estimate_tokens(&rest[i].content);
        if start < rest.len() && used + len > budget_tokens {
            break;
        }
        used += len;
        start = i;
    }
    out.extend(rest[start..].iter().cloned());
    out
}

/// System prompt for the free-form chat box.
pub const CHAT_SYSTEM: &str = "You are a senior terminal and ops assistant embedded in the \
OpenNex terminal manager. The user works in a shell. Answer concisely and practically; prefer \
copy-pasteable commands over prose. Reply in the language the user writes in.";

/// System prompt for the "explain terminal output" action.
pub const EXPLAIN_SYSTEM: &str = "You are a senior terminal and ops assistant embedded in the \
OpenNex terminal manager. The user will paste terminal output or an error inside a code block. \
Explain in 1-3 short sentences what happened, then — if it is an error — give the concrete fix \
as a copy-pasteable command. Skip preamble. Reply in the language of the pasted content.";

/// System prompt for the "fix selected output" action: the answer IS the
/// fix, ready to paste into the shell.
pub const FIX_SYSTEM: &str = "You are a senior terminal and ops assistant embedded in the \
OpenNex terminal manager. The user pastes failing terminal output inside a code block. Reply \
with ONLY the sequence of shell commands that fixes it — no explanations, no code fences, no \
comments. If it cannot be fixed by commands, reply with the single most informative sentence.";

/// One conversation message (multi-turn panel history). `role` is one of
/// "system" / "user" / "assistant".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: &'static str,
    pub content: String,
}

/// Join the configured base URL with the chat-completions path,
/// tolerating a trailing slash.
pub fn endpoint(base_url: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    if base.ends_with("/chat/completions") {
        base.to_string()
    } else {
        format!("{base}/chat/completions")
    }
}

/// Build the chat-completions request body from a full conversation.
pub fn request_body_messages(model: &str, messages: &[ChatMessage]) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "messages": messages
            .iter()
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect::<Vec<_>>(),
        "temperature": 0.2,
        "stream": false,
    })
}

/// Single-turn convenience wrapper around [`request_body_messages`].
pub fn request_body(model: &str, system: &str, user: &str) -> serde_json::Value {
    request_body_messages(
        model,
        &[
            ChatMessage {
                role: "system",
                content: system.to_string(),
            },
            ChatMessage {
                role: "user",
                content: user.to_string(),
            },
        ],
    )
}

/// Extract the assistant message from a (non-streaming) response.
pub fn parse_content(json: &serde_json::Value) -> Result<String, String> {
    let content = json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or_else(|| "unexpected response shape (no choices[0].message.content)".to_string())?;
    Ok(content.trim().to_string())
}

/// Blocking multi-turn chat completion. NEVER call on the UI thread.
/// `messages` carries the FULL conversation (system first, then alternating
/// user/assistant turns); token bloat is the caller's responsibility.
pub fn complete_messages(
    cfg: &AiConfig,
    messages: &[ChatMessage],
    timeout_secs: u64,
) -> Result<String, String> {
    if cfg.base_url.trim().is_empty() {
        return Err("base URL is empty".into());
    }
    if cfg.api_key.trim().is_empty() {
        return Err("API key is empty".into());
    }
    let body = request_body_messages(&cfg.model, messages);
    let response = ureq::post(&endpoint(&cfg.base_url))
        .timeout(std::time::Duration::from_secs(timeout_secs.max(1)))
        .set("Authorization", &format!("Bearer {}", cfg.api_key.trim()))
        .send_json(body)
        .map_err(|err| match err {
            ureq::Error::Status(code, resp) => {
                let detail = resp.into_string().unwrap_or_default();
                let detail: String = detail.chars().take(400).collect();
                if detail.is_empty() {
                    format!("HTTP {code}")
                } else {
                    format!("HTTP {code}: {detail}")
                }
            }
            ureq::Error::Transport(t) => t.to_string(),
        })?;
    let json: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("invalid JSON response: {e}"))?;
    parse_content(&json)
}

/// Blocking single-turn chat completion (system + user). NEVER call on
/// the UI thread — spawn it.
pub fn complete(
    cfg: &AiConfig,
    system: &str,
    user: &str,
    timeout_secs: u64,
) -> Result<String, String> {
    complete_messages(
        cfg,
        &[
            ChatMessage {
                role: "system",
                content: system.to_string(),
            },
            ChatMessage {
                role: "user",
                content: user.to_string(),
            },
        ],
        timeout_secs,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(content: &str) -> ChatMessage {
        ChatMessage {
            role: "user",
            content: content.to_string(),
        }
    }

    #[test]
    fn budget_uses_per_model_override_or_default() {
        let mut limits = HashMap::new();
        assert_eq!(context_budget_tokens(&limits, "m"), 150_000);
        limits.insert("m".into(), 8);
        assert_eq!(context_budget_tokens(&limits, "m"), 8_000);
        // An explicit 0 means "unset", not "no budget at all".
        limits.insert("m".into(), 0);
        assert_eq!(context_budget_tokens(&limits, "m"), 150_000);
    }

    #[test]
    fn token_estimate_counts_cjk_and_latin_differently() {
        // CJK chars count ~1 token each.
        assert_eq!(estimate_tokens("你好世界"), 4);
        // Latin runs at ~4 chars per token.
        assert_eq!(estimate_tokens("abcdefgh"), 2);
        // Mixed.
        assert_eq!(estimate_tokens("你好world"), 2 + 2);
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn fit_keeps_everything_when_under_budget() {
        let msgs = [msg("a"), msg("bb"), msg("ccc")];
        let fit = fit_messages(&msgs, 0, 100);
        assert_eq!(fit.len(), 3);
    }

    #[test]
    fn fit_keeps_head_and_drops_oldest_middle() {
        let msgs = [
            msg("head1"),
            msg("head2"),
            msg("old1"),
            msg("old2"),
            msg("new"),
        ];
        // Budget only fits head (always kept) + the last two messages:
        // 2×2 head tokens ("head1" = 5 latin chars → 2 tokens) + 1 + 1.
        let fit = fit_messages(&msgs, 2, 6);
        assert_eq!(fit.len(), 4);
        assert_eq!(fit[0].content, "head1");
        assert_eq!(fit[1].content, "head2");
        assert_eq!(fit[2].content, "old2");
        assert_eq!(fit[3].content, "new");
    }

    #[test]
    fn fit_tail_is_contiguous_and_whole() {
        let msgs = [msg("one"), msg("two"), msg("three"), msg("four")];
        // Budget fits only the last message: nothing partial in between.
        let fit = fit_messages(&msgs, 0, 2);
        assert_eq!(fit.len(), 1);
        assert_eq!(fit[0].content, "four");
    }

    #[test]
    fn fit_always_sends_the_newest_message() {
        let msgs = [msg("short"), msg("a-very-long-overflowing-message")];
        let fit = fit_messages(&msgs, 0, 5);
        assert_eq!(fit.len(), 1);
        assert_eq!(fit[0].content, "a-very-long-overflowing-message");
    }

    #[test]
    fn fit_keep_first_survives_tiny_budgets() {
        let msgs = [msg("sys"), msg("goal"), msg("obs")];
        let fit = fit_messages(&msgs, 2, 1);
        assert_eq!(fit.len(), 3);
        assert_eq!(fit[0].content, "sys");
        assert_eq!(fit[2].content, "obs");
    }

    #[test]
    fn endpoint_joins_paths_tolerantly() {
        assert_eq!(
            endpoint("https://api.openai.com/v1"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            endpoint("https://api.openai.com/v1/"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            endpoint("http://localhost:11434/v1"),
            "http://localhost:11434/v1/chat/completions"
        );
        // A user pasting the FULL endpoint must not get it doubled.
        assert_eq!(
            endpoint("https://x.example/v1/chat/completions"),
            "https://x.example/v1/chat/completions"
        );
    }

    #[test]
    fn multi_turn_body_preserves_conversation_order() {
        let messages = vec![
            ChatMessage {
                role: "system",
                content: "sys".into(),
            },
            ChatMessage {
                role: "user",
                content: "q1".into(),
            },
            ChatMessage {
                role: "assistant",
                content: "a1".into(),
            },
            ChatMessage {
                role: "user",
                content: "q2".into(),
            },
        ];
        let body = request_body_messages("m", &messages);
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 4);
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(msgs[1]["content"], "q1");
        assert_eq!(msgs[2]["role"], "assistant");
        assert_eq!(msgs[3]["content"], "q2");
        assert_eq!(body["stream"], false);
    }

    #[test]
    fn single_turn_body_is_the_two_message_special_case() {
        let body = request_body("m", "sys", "hi");
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(
            (
                msgs[0]["role"].as_str().unwrap(),
                msgs[0]["content"].as_str().unwrap()
            ),
            ("system", "sys")
        );
        assert_eq!(
            (
                msgs[1]["role"].as_str().unwrap(),
                msgs[1]["content"].as_str().unwrap()
            ),
            ("user", "hi")
        );
    }

    #[test]
    fn request_body_carries_messages_and_model() {
        let body = request_body("m1", "sys", "user text");
        assert_eq!(body["model"], "m1");
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][0]["content"], "sys");
        assert_eq!(body["messages"][1]["content"], "user text");
        assert_eq!(body["stream"], false);
    }

    #[test]
    fn parse_content_reads_assistant_message() {
        let good = serde_json::json!({
            "choices": [{"message": {"role": "assistant", "content": "  hello  "}}]
        });
        assert_eq!(parse_content(&good).unwrap(), "hello");
        assert!(parse_content(&serde_json::json!({})).is_err());
        assert!(parse_content(&serde_json::json!({"choices": []})).is_err());
    }

    #[test]
    fn complete_rejects_incomplete_config_without_network() {
        let cfg = AiConfig {
            base_url: "https://api.example/v1".into(),
            api_key: String::new(),
            model: "m".into(),
        };
        assert!(complete(&cfg, "s", "u", 5).is_err());
        let cfg = AiConfig {
            base_url: String::new(),
            api_key: "k".into(),
            model: "m".into(),
        };
        assert!(complete(&cfg, "s", "u", 5).is_err());
    }
}
