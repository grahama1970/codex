use codex_core::ModelProviderInfo;
use codex_core::WireApi;
use codex_core::config::Config;
use codex_core::config::ConfigOverrides;
use codex_core::config::ConfigToml;
use codex_core::config::ContextToml;

#[test]
fn local_only_blocks_external_providers_allows_localhost() {
    let mut base = ConfigToml::default();
    base.local_only = Some(true);
    base.context = Some(ContextToml {
        provider: Some("minimal".into()),
        max_context_tokens: Some(2048),
        budget: None,
        arango: None,
    });

    let cfg = Config::load_from_base_config_with_overrides(
        base,
        ConfigOverrides::default(),
        tempfile::TempDir::new().unwrap().path().to_path_buf(),
    )
    .expect("load config");

    let external = ModelProviderInfo {
        name: "OpenAI".into(),
        base_url: Some("https://api.openai.com/v1".into()),
        env_key: None,
        env_key_instructions: None,
        wire_api: WireApi::Chat,
        query_params: None,
        http_headers: None,
        env_http_headers: None,
        request_max_retries: None,
        stream_max_retries: None,
        stream_idle_timeout_ms: None,
        requires_openai_auth: false,
    };
    assert!(
        !cfg.is_provider_allowed(&external),
        "external provider should be blocked under local_only"
    );

    let local = ModelProviderInfo {
        name: "Ollama".into(),
        base_url: Some("http://127.0.0.1:11434/v1".into()),
        env_key: None,
        env_key_instructions: None,
        wire_api: WireApi::Chat,
        query_params: None,
        http_headers: None,
        env_http_headers: None,
        request_max_retries: None,
        stream_max_retries: None,
        stream_idle_timeout_ms: None,
        requires_openai_auth: false,
    };
    assert!(
        cfg.is_provider_allowed(&local),
        "localhost provider should be allowed under local_only"
    );
}
