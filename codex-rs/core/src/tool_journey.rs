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
