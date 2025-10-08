use codex_core::config::Config;
use codex_core::config::ConfigOverrides;
use codex_core::config::ContextProviderKind;
use codex_core::config::ContextToml;
use codex_protocol::config_types::SandboxMode;

#[test]
fn arango_provider_triggers_context_prefix_injection_config_wiring() {
    // Build config with provider set to arango.
    let mut base = codex_core::config::ConfigToml::default();
    base.context = Some(ContextToml {
        provider: Some("arango".into()),
        max_context_tokens: Some(2048),
        budget: None,
        arango: None,
    });

    let cfg = Config::load_from_base_config_with_overrides(
        base,
        ConfigOverrides {
            sandbox_mode: Some(SandboxMode::ReadOnly),
            ..Default::default()
        },
        tempfile::TempDir::new().unwrap().path().to_path_buf(),
    )
    .expect("config");

    assert!(matches!(cfg.context_provider, ContextProviderKind::Arango));
    assert_eq!(cfg.context_max_tokens, 2048);

    // Prompt injection path is exercised in runtime; here we only assert cfg wiring.
}
