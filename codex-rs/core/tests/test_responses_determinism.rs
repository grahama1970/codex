use codex_core::responses_payload::apply_determinism;

#[test]
fn responses_payload_sets_seed_and_determinism() {
    let mut payload = serde_json::json!({
        "model": "test-model",
        "input": "Hello world",
    });
    apply_determinism(&mut payload, Some(1234));
    let obj = payload.as_object().expect("object");
    assert_eq!(obj.get("temperature").unwrap(), 0.0);
    assert_eq!(obj.get("top_p").unwrap(), 1.0);
    assert_eq!(obj.get("seed").unwrap(), 1234);
    assert_eq!(obj.get("frequency_penalty").unwrap(), 0.0);
    assert_eq!(obj.get("presence_penalty").unwrap(), 0.0);
    assert_eq!(obj.get("top_k").unwrap(), 0);
    assert_eq!(obj.get("typical_p").unwrap(), 1.0);
    assert!(
        obj.get("logit_bias")
            .unwrap()
            .as_object()
            .unwrap()
            .is_empty()
    );
}
