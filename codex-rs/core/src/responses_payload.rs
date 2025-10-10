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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_determinism_sets_seed_and_neutralizes() {
        let mut payload = serde_json::json!({
            "model": "gpt-5",
            "input": [{"role": "user", "content": [{"type":"input_text","text":"hi"}]}]
        });
        apply_determinism(&mut payload, Some(42));
        let obj = payload.as_object().unwrap();
        assert_eq!(obj.get("temperature"), Some(&serde_json::json!(0.0)));
        assert_eq!(obj.get("top_p"), Some(&serde_json::json!(1.0)));
        assert_eq!(obj.get("seed"), Some(&serde_json::json!(42)));
        assert_eq!(obj.get("frequency_penalty"), Some(&serde_json::json!(0.0)));
        assert_eq!(obj.get("presence_penalty"), Some(&serde_json::json!(0.0)));
        assert_eq!(obj.get("top_k"), Some(&serde_json::json!(0)));
        assert_eq!(obj.get("typical_p"), Some(&serde_json::json!(1.0)));
        assert!(
            obj.get("logit_bias")
                .unwrap()
                .as_object()
                .unwrap()
                .is_empty()
        );
    }
}
