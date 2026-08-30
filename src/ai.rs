//! AI assistant: settings model and an OpenAI-compatible
//! chat-completions client (works with any compatible endpoint, including
//! a local Ollama server at http://localhost:11434/v1).
//!
//! Pure logic lives here; the floating panel UI lives in app.rs. The
//! HTTP call is blocking and always runs on a background thread, with
//! the result handed back through a channel.

use serde::{Deserialize, Serialize};

/// User-supplied AI configuration. The API key is stored in the local
/// settings.json ONLY — it is never sent anywhere except the configured
/// endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
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

/// Build the chat-completions request body. Exposed for tests.
pub fn request_body(model: &str, system: &str, user: &str) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "temperature": 0.2,
        "stream": false,
    })
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

/// Blocking chat completion. NEVER call on the UI thread — spawn it.
pub fn complete(
    cfg: &AiConfig,
    system: &str,
    user: &str,
    timeout_secs: u64,
) -> Result<String, String> {
    if cfg.base_url.trim().is_empty() {
        return Err("base URL is empty".into());
    }
    if cfg.api_key.trim().is_empty() {
        return Err("API key is empty".into());
    }
    let body = request_body(&cfg.model, system, user);
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

#[cfg(test)]
mod tests {
    use super::*;

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
