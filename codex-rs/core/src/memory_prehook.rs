use std::time::Duration;

use crate::client_common::Prompt;
use crate::config::Config;
use crate::error::{CodexErr, Result};
use crate::protocol::ConversationId;
use codex_protocol::models::{ContentItem, ResponseItem};

pub fn is_enabled(_cfg: &Config) -> bool {
    std::env::var("MEMORY_PREHOOK_ENABLED").ok().map(|v| v.to_lowercase()).map(|v| v=="1"||v=="true"||v=="yes").unwrap_or(true)
}

pub fn fail_on_error(_cfg: &Config) -> bool {
    std::env::var("MEMORY_PREHOOK_ON_ERROR").ok().map(|v| v.to_lowercase())
        .map(|v| v!="allow").unwrap_or(true)
}

fn extract_last_user_query(prompt: &Prompt) -> String {
    for item in prompt.input.iter().rev() {
        if let ResponseItem::Message { role, content, .. } = item {
            if role == "user" {
                let mut s = String::new();
                for c in content {
                    if let ContentItem::InputText { text } = c { s.push_str(text); s.push_str("\n"); }
                }
                return s.trim().to_string();
            }
        }
    }
    String::new()
}

fn build_system_context_text(items: &serde_json::Value, k: usize, max_bytes: usize) -> Option<String> {
    let mut out = String::from("Memory context (top results):\n");
    let mut count = 0usize;
    if let Some(arr) = items.as_array() {
        for it in arr.iter() {
            if count >= k { break; }
            let title = it.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let why = it.get("why").and_then(|v| v.as_str()).unwrap_or("");
            let key = it.get("_key").and_then(|v| v.as_str()).unwrap_or("");
            let line = format!("• {} — {}; key=lessons/{}\n", title, why, key);
            if out.len() + line.len() > max_bytes { break; }
            out.push_str(&line);
            count += 1;
        }
    }
    if count == 0 { None } else { Some(out) }
}

pub async fn run(cfg: &Config, prompt: &Prompt, conversation_id: &ConversationId) -> Result<Option<Vec<ResponseItem>>> {
    let q = extract_last_user_query(prompt);
    if q.is_empty() { return Ok(None); }

    let scope = std::env::var("MEMORY_PREHOOK_SCOPE").ok().filter(|s|!s.is_empty()).unwrap_or_else(|| "tabbed".to_string());
    let k = std::env::var("MEMORY_PREHOOK_K").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(5usize);
    let timeout_ms = std::env::var("MEMORY_PREHOOK_TIMEOUT_MS").ok().and_then(|s| s.parse::<u64>().ok()).unwrap_or(5_000u64);

    // Prefer rmcp client if available; otherwise shell out to uv run lessons-search as a fallback (dev only)
    let use_rmcp = cfg.use_experimental_use_rmcp_client;
    let items_json = if use_rmcp {
        // Use MCP connection manager to call memory_search
        let payload = serde_json::json!({"q": q, "scope": scope, "k": k});
        let res = crate::mcp_connection_manager::call_tool_json(
            "memory-agent",
            "memory_search",
            payload,
            Some(Duration::from_millis(timeout_ms)),
            Some(vec![("THREAD_ID".to_string(), conversation_id.0.clone())])
        ).await?;
        res
    } else {
        // Dev fallback: shell out (best-effort)
        let text = crate::util::shell_json(
            &vec!["uv","run","lessons-search","--q", &q, "--scope", &scope, "--k", &k.to_string(), "--json"],
            Duration::from_millis(timeout_ms),
            Some(vec![("THREAD_ID".into(), conversation_id.0.clone())])
        ).await.map_err(|e| CodexErr::other(format!("memory cli failed: {e}")))?;
        serde_json::from_str::<serde_json::Value>(&text).unwrap_or(serde_json::json!({"items":[]}))
    };

    let items = items_json.get("items").cloned().unwrap_or(serde_json::json!([]));
    if let Some(text) = build_system_context_text(&items, k, 64_000) {
        let sys = ResponseItem::Message { id: None, role: "system".to_string(), content: vec![ContentItem::OutputText { text } ] };
        let mut augmented = prompt.input.clone();
        augmented.insert(0, sys);
        Ok(Some(augmented))
    } else {
        Ok(None)
    }
}
