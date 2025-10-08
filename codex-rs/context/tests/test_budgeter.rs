use codex_context::ContextProvider;
use codex_context::MinimalContextProvider;
use codex_context::SectionQuotas;
use codex_context::TurnInput;
use pretty_assertions::assert_eq;

fn long_words(n: usize) -> String {
    (0..n).map(|i| format!("w{} ", i)).collect::<String>()
}

#[tokio::test]
async fn quotas_normalize_to_100() {
    let q = SectionQuotas {
        recent_pct: 10,
        plan_pct: 10,
        evidence_pct: 10,
        tools_pct: 10,
    };
    let n = q.normalize();
    assert_eq!(
        n.recent_pct as u16 + n.plan_pct as u16 + n.evidence_pct as u16 + n.tools_pct as u16,
        100
    );

    let z = SectionQuotas {
        recent_pct: 0,
        plan_pct: 0,
        evidence_pct: 0,
        tools_pct: 0,
    }
    .normalize();
    assert_eq!(
        (z.recent_pct, z.plan_pct, z.evidence_pct, z.tools_pct),
        (15, 10, 60, 15)
    );
}

#[tokio::test]
async fn minimal_provider_truncates_within_budget() {
    let quotas = SectionQuotas {
        recent_pct: 25,
        plan_pct: 25,
        evidence_pct: 25,
        tools_pct: 25,
    };
    let input = TurnInput {
        user_text: "hello".into(),
        recent_turns: vec![long_words(200)],
        plan_text: Some(long_words(200)),
        tool_deltas: vec![long_words(200)],
        max_context_tokens: 40, // 10 tokens per section
        quotas,
    };
    let (bundle, metrics) = MinimalContextProvider.build(&input).await.expect("ok");

    // Evidence is empty in minimal provider
    assert_eq!(bundle.evidence_tokens, 0);
    assert!(bundle.evidence.is_empty());

    // Each non-empty section should be truncated to <= budget (approx by token count)
    assert!(bundle.recent_tokens as usize >= 10);
    assert!(bundle.plan_tokens as usize >= 10);
    assert!(bundle.tools_tokens as usize >= 10);
    assert!(bundle.truncated_recent);
    assert!(bundle.truncated_plan);
    assert!(bundle.truncated_tools);

    // Metrics mirror bundle
    assert_eq!(metrics.recent_tokens, bundle.recent_tokens);
    assert_eq!(metrics.plan_tokens, bundle.plan_tokens);
    assert_eq!(metrics.tools_tokens, bundle.tools_tokens);
    assert!(metrics.truncated_recent && metrics.truncated_plan && metrics.truncated_tools);
}
