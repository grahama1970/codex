use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

#[derive(Clone, Debug, serde::Serialize)]
pub struct ToolCallEntry {
    pub name: String,
    pub kind: String,
    pub call_id: String,
    pub latency_ms: i64,
    pub success: bool,
    pub retries: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

lazy_static! {
    static ref JOURNEY: Mutex<HashMap<String, Vec<ToolCallEntry>>> = Mutex::new(HashMap::new());
}

pub(crate) fn record_mcp_end(
    sub_id: &str,
    name: &str,
    call_id: &str,
    latency_ms: i64,
    success: bool,
) {
    let mut map = JOURNEY.lock().expect("tool journey lock");
    let v = map.entry(sub_id.to_string()).or_default();
    if v.len() < 16 {
        v.push(ToolCallEntry {
            name: name.to_string(),
            kind: "mcp".to_string(),
            call_id: call_id.to_string(),
            latency_ms,
            success,
            retries: 0,
            error_code: None,
        });
    }
}

pub(crate) fn record_exec_end(
    sub_id: &str,
    kind: &str,
    name: &str,
    call_id: &str,
    latency_ms: i64,
    success: bool,
    retries: i32,
    error_code: Option<&str>,
) {
    let mut map = JOURNEY.lock().expect("tool journey lock");
    let v = map.entry(sub_id.to_string()).or_default();
    if v.len() < 16 {
        v.push(ToolCallEntry {
            name: name.to_string(),
            kind: kind.to_string(),
            call_id: call_id.to_string(),
            latency_ms,
            success,
            retries,
            error_code: error_code.map(|s| s.to_string()),
        });
    }
}

pub(crate) fn drain_for_sub_id(sub_id: &str) -> Vec<ToolCallEntry> {
    JOURNEY
        .lock()
        .expect("tool journey lock")
        .remove(sub_id)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_at_16_and_drains() {
        let sid = "sub-test";
        for i in 0..25 {
            record_exec_end(sid, "shell", "cmd", &format!("c{i}"), 10, true, 0, None);
        }
        let v = drain_for_sub_id(sid);
        assert!(v.len() <= 16);
        let v2 = drain_for_sub_id(sid);
        assert!(v2.is_empty());
    }
}
