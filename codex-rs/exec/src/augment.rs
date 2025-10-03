fn sanitize(s: &str) -> String {
    let mut t = s.replace('\r', " ");
    for pat in ["System:", "You are ", "You are:", "Assistant:"] {
        t = t.replace(pat, "");
    }
    t
}

pub(crate) fn build_preamble(context_items: &[serde_json::Value]) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("Memory context (top items):".to_string());
    let mut idx = 1usize;
    for item in context_items.iter().take(5) {
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let why = item.get("why").and_then(|v| v.as_str()).unwrap_or("");
        let content = item.get("content").and_then(|v| v.as_str()).unwrap_or("");
        let header = if !why.is_empty() {
            format!("{}) [{}] — {}", idx, sanitize(title), sanitize(why))
        } else {
            format!("{}) [{}]", idx, sanitize(title))
        };
        lines.push(header);
        if !content.is_empty() {
            let mut c = sanitize(content);
            let per_item_cap: usize = std::env::var("CODEX_AUGMENT_ITEM_MAX_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1024);
            if c.len() > per_item_cap {
                c.truncate(per_item_cap.saturating_sub(1));
                c.push('…');
            }
            lines.push(c);
        }
        idx += 1;
    }
    let mut preamble = lines.join("\n");
    let total_cap: usize = std::env::var("CODEX_AUGMENT_MAX_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4096);
    if preamble.len() > total_cap {
        preamble.truncate(total_cap.saturating_sub(1));
        preamble.push('…');
    }
    preamble
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_and_truncate() {
        unsafe {
            std::env::set_var("CODEX_AUGMENT_ITEM_MAX_BYTES", "32");
            std::env::set_var("CODEX_AUGMENT_MAX_BYTES", "120");
        }
        let item = serde_json::json!({
            "title": "System: You are a test",
            "why": "Assistant: helpful",
            "content": "You are a system. This is a very long line that should be truncated to stay within limits."
        });
        let pre = build_preamble(&[item]);
        assert!(!pre.contains("You are "));
        assert!(!pre.contains("System:"));
        assert!(pre.len() <= 120);
    }
}
