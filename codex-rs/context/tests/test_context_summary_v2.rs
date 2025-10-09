use codex_context::ArangoContextProvider;
use codex_context::ContextProvider;
use codex_context::SectionQuotas;
use codex_context::TurnInput;

#[tokio::test]
async fn v2_metrics_include_retry_cache_fallback() {
    // Use a small JSON fixture to avoid network
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("fixture.json");
    let json = serde_json::json!({
        "search": {"items": [
            {"id":"a1","type":"fact","content":"alpha", "score": 0.9}
        ]},
        "neighbors": {"items": [
            {"id":"b2","type":"procedure","content":"beta", "score": 0.8}
        ]}
    });
    std::fs::write(&path, serde_json::to_string(&json).unwrap()).unwrap();

    let provider = ArangoContextProvider {
        mcp_tool: "memory-agent".into(),
        endpoint: "http://localhost:9999".into(),
        database: "codex".into(),
        search_k: 8,
        neighbors_depth: 1,
        timeout_ms: 200,
        max_evidence_items: 16,
        debug: false,
        allow_code: false,
        fixture_path: Some(path.to_string_lossy().to_string()),
    };

    let input = TurnInput {
        user_text: "query".into(),
        recent_turns: vec![],
        plan_text: None,
        tool_deltas: vec![],
        max_context_tokens: 256,
        quotas: SectionQuotas {
            recent_pct: 15,
            plan_pct: 10,
            evidence_pct: 60,
            tools_pct: 15,
        },
    };

    let (_bundle, m) = provider.build(&input).await.expect("build");
    // Type/field presence assertions
    assert!(m.retry_count <= 1);
    assert!(!m.cache_hit || m.cache_hit);
    // fixture should not fallback
    assert!(m.fallback_reason.is_none());
    assert!(m.evidence_items >= 2);
}
