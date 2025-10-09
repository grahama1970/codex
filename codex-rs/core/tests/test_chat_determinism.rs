use codex_core::client_common::Prompt;
use codex_core::model_family::ModelFamily;
use codex_core::model_provider_info::{ModelProviderInfo, WireApi};

#[test]
fn chat_payload_sets_seed_and_determinism() {
    let prompt = Prompt::new(Some("Hello".into()), vec![], None);
    let mf = ModelFamily { family: "gpt-4o-mini".into(), variant: None };
    let provider = ModelProviderInfo {
        name: "openai".into(),
        base_url: Some("https://api.openai.com".into()),
        env_key: Some("OPENAI_API_KEY".into()),
        env_key_instructions: None,
        wire_api: WireApi::Chat,
        query_params: None,
        http_headers: None,
        env_http_headers: None,
        request_max_retries: Some(0),
        stream_max_retries: Some(0),
        stream_idle_timeout_ms: Some(1000),
        requires_openai_auth: true,
    };
    let payload = codex_core::chat_completions::build_chat_payload_for_test(
        &prompt, &mf, &provider, Some(42),
    );
    let obj = payload.as_object().unwrap();
    assert_eq!(obj.get("temperature").unwrap(), &serde_json::json!(0.0));
    assert_eq!(obj.get("top_p").unwrap(), &serde_json::json!(1.0));
    assert_eq!(obj.get("seed").unwrap(), &serde_json::json!(42));
    assert_eq!(obj.get("frequency_penalty").unwrap(), &serde_json::json!(0.0));
    assert_eq!(obj.get("presence_penalty").unwrap(), &serde_json::json!(0.0));
    assert_eq!(obj.get("top_k").unwrap(), &serde_json::json!(0));
    assert_eq!(obj.get("typical_p").unwrap(), &serde_json::json!(1.0));
    assert!(obj.get("logit_bias").unwrap().as_object().unwrap().is_empty());
}
