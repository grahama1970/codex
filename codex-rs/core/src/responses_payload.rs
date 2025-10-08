use serde_json::Value;

/// Apply determinism controls to an OpenAI Responses API payload.
///
/// When a deterministic seed is provided, force low-variance sampling and include the seed.
pub fn apply_determinism(payload: &mut Value, deterministic_seed: Option<u64>) {
    if let Some(seed) = deterministic_seed
        && let Some(obj) = payload.as_object_mut()
    {
        obj.insert("temperature".to_string(), serde_json::json!(0.0));
        obj.insert("top_p".to_string(), serde_json::json!(1.0));
        obj.insert("seed".to_string(), serde_json::json!(seed));
        // Determinism hardening: neutralize other knobs that can introduce variance.
        obj.insert("frequency_penalty".to_string(), serde_json::json!(0.0));
        obj.insert("presence_penalty".to_string(), serde_json::json!(0.0));
        obj.insert("top_k".to_string(), serde_json::json!(0));
        obj.insert("typical_p".to_string(), serde_json::json!(1.0));
        obj.insert("logit_bias".to_string(), serde_json::json!({}));
        if !obj.contains_key("stream") {
            obj.insert("stream".to_string(), serde_json::json!(true));
        }
    }
}
