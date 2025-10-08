use codex_context::ArangoContextProvider;
use codex_context::ContextProvider;
use codex_context::SectionQuotas;
use codex_context::TurnInput;
use pretty_assertions::assert_eq;
use std::fs;

#[test]
#[ignore]
fn arango_provider_uses_fixture() {
    // Create a small fixture file with search + neighbors items
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("fixture.json");
    let json = serde_json::json!({
        "search": {"items": [
            {"id":"a1","type":"fact","content":"one", "score": 0.9},
            {"id":"b2","type":"procedure","content":"two", "score": 0.8}
        ]},
        "neighbors": {"items": [
            {"id":"c3","type":"episode","content":"three", "score": 0.7}
        ]}
    });
    fs::write(&path, serde_json::to_string(&json).unwrap()).unwrap();

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

    let quotas = SectionQuotas {
        recent_pct: 10,
        plan_pct: 10,
        evidence_pct: 70,
        tools_pct: 10,
    };
    let input = TurnInput {
        user_text: "hello".into(),
        recent_turns: vec!["recent one".into()],
        plan_text: Some("plan".into()),
        tool_deltas: vec![],
        max_context_tokens: 128,
        quotas,
    };

    let (_bundle, metrics) = provider.build(&input).expect("ok");
    // We expect 3 evidence items from the fixture and non-zero token accounting
    assert_eq!(metrics.evidence_items, 3);
    assert!(metrics.total_tokens > 0);
}
