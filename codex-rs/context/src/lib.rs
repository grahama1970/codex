use serde::Deserialize;
use serde::Serialize;
use std::time::Instant;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct TurnInput {
    pub user_text: String,
    pub recent_turns: Vec<String>,
    pub plan_text: Option<String>,
    pub tool_deltas: Vec<String>,
    pub max_context_tokens: usize,
    pub quotas: SectionQuotas,
}

#[derive(Debug, Clone, Copy)]
pub struct SectionQuotas {
    pub recent_pct: u8,
    pub plan_pct: u8,
    pub evidence_pct: u8,
    pub tools_pct: u8,
}

impl SectionQuotas {
    pub fn normalize(mut self) -> Self {
        let total = (self.recent_pct as u32)
            + (self.plan_pct as u32)
            + (self.evidence_pct as u32)
            + (self.tools_pct as u32);
        if total == 100 {
            return self;
        }
        if total == 0 {
            self.recent_pct = 15;
            self.plan_pct = 10;
            self.evidence_pct = 60;
            self.tools_pct = 15;
            return self;
        }
        let scale = 100f32 / total as f32;
        self.recent_pct = ((self.recent_pct as f32) * scale).round() as u8;
        self.plan_pct = ((self.plan_pct as f32) * scale).round() as u8;
        self.evidence_pct = ((self.evidence_pct as f32) * scale).round() as u8;
        self.tools_pct = ((self.tools_pct as f32) * scale).round() as u8;
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvidenceBundle {
    pub evidence: String,
    pub plan: String,
    pub recent: String,
    pub tools: String,
    pub recent_tokens: u32,
    pub plan_tokens: u32,
    pub evidence_tokens: u32,
    pub tools_tokens: u32,
    pub truncated_recent: bool,
    pub truncated_plan: bool,
    pub truncated_evidence: bool,
    pub truncated_tools: bool,
}

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("mcp call failed: {0}")]
    Mcp(String),
    #[error("internal: {0}")]
    Internal(String),
}

/// Trait for pluggable providers (minimal vs arango)
pub trait ContextProvider: Send + Sync {
    fn build(&self, input: &TurnInput) -> Result<EvidenceBundle, ContextError>;
}

pub struct MinimalContextProvider;

impl ContextProvider for MinimalContextProvider {
    fn build(&self, input: &TurnInput) -> Result<EvidenceBundle, ContextError> {
        let quotas = input.quotas.normalize();
        let budget = input.max_context_tokens.max(256);
        let recent_budget = (budget as f32 * (quotas.recent_pct as f32 / 100.0)) as usize;
        let plan_budget = (budget as f32 * (quotas.plan_pct as f32 / 100.0)) as usize;
        let tools_budget = (budget as f32 * (quotas.tools_pct as f32 / 100.0)) as usize;
        let evidence_budget = (budget as f32 * (quotas.evidence_pct as f32 / 100.0)) as usize;

        let recent_concat = input.recent_turns.join("\n");
        let (recent, trunc_recent) = truncate_tokens(&recent_concat, recent_budget);
        let (plan, trunc_plan) =
            truncate_tokens(input.plan_text.as_deref().unwrap_or(""), plan_budget);
        let deltas = input.tool_deltas.join("\n");
        let (tools, trunc_tools) = truncate_tokens(&deltas, tools_budget);
        let (evidence, trunc_evidence) = truncate_tokens("", evidence_budget);

        Ok(EvidenceBundle {
            evidence,
            plan,
            recent,
            tools,
            recent_tokens: count_tokens(&recent_concat) as u32,
            plan_tokens: count_tokens(input.plan_text.as_deref().unwrap_or("")) as u32,
            evidence_tokens: 0,
            tools_tokens: count_tokens(&deltas) as u32,
            truncated_recent: trunc_recent,
            truncated_plan: trunc_plan,
            truncated_evidence: trunc_evidence,
            truncated_tools: trunc_tools,
        })
    }
}

/// Phase‑0 stub for Arango/memory-agent.
pub struct ArangoContextProvider {
    pub mcp_tool: String,
    pub endpoint: String,
    pub database: String,
    pub search_k: u32,
    pub neighbors_depth: u8,
    pub timeout_ms: u64,
    pub max_evidence_items: u32,
}

impl ContextProvider for ArangoContextProvider {
    fn build(&self, input: &TurnInput) -> Result<EvidenceBundle, ContextError> {
        let t0 = Instant::now();
        let mut minimal = MinimalContextProvider.build(input)?;
        // Retrieval via fixture if provided (Phase‑1 safe path)
        let items = if let Some(fix) = std::env::var("CONTEXT_MCP_FIXTURE").ok() {
            load_fixture_items(&fix).unwrap_or_default()
        } else {
            Vec::new()
        };
        let lines = shape_evidence_lines(items, self.max_evidence_items);
        let evidence_text = if lines.is_empty() {
            String::new()
        } else {
            let mut s = String::from("### Evidence\n");
            for l in &lines { s.push_str(l); s.push('\n'); }
            s
        };
        // Adaptive reflow to evidence
        let quotas = input.quotas.normalize();
        let budget = input.max_context_tokens.max(256);
        let mut ev_budget = (budget as f32 * (quotas.evidence_pct as f32 / 100.0)) as usize;
        if minimal.plan.is_empty() { ev_budget += (budget as f32 * (quotas.plan_pct as f32 / 100.0)) as usize; }
        if minimal.recent.is_empty() { ev_budget += (budget as f32 * (quotas.recent_pct as f32 / 100.0)) as usize; }
        if minimal.tools.is_empty() { ev_budget += (budget as f32 * (quotas.tools_pct as f32 / 100.0)) as usize; }

        let (evidence_trimmed, trunc_evidence) = truncate_tokens(&evidence_text, ev_budget);
        minimal.evidence = evidence_trimmed;
        minimal.evidence_tokens = count_tokens(&minimal.evidence) as u32;
        minimal.truncated_evidence = trunc_evidence;
        let _elapsed = t0.elapsed();
        Ok(minimal)
    }
}

// ---------- Phase‑1 helpers: fixture + shaping ----------

#[derive(Debug, Clone, Deserialize)]
struct FixtureItem {
    id: String,
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    score: f32,
    #[serde(default)]
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureRoot { items: Vec<FixtureItem> }

fn load_fixture_items(path: &str) -> Result<Vec<FixtureItem>, ContextError> {
    let data = std::fs::read(path).map_err(|e| ContextError::Internal(format!("read fixture: {e}")))?;
    let root: FixtureRoot = serde_json::from_slice(&data)
        .map_err(|e| ContextError::Internal(format!("parse fixture: {e}")))?;
    Ok(root.items)
}

fn shape_evidence_lines(mut items: Vec<FixtureItem>, max_items: u32) -> Vec<String> {
    // Dedup by id
    let mut seen = std::collections::HashSet::new();
    items.retain(|it| seen.insert(it.id.clone()));
    // Type priority: fact < procedure < episode (ascending)
    fn type_prio(t: &str) -> u8 {
        match t.to_ascii_lowercase().as_str() { "fact" => 0, "procedure" => 1, "episode" => 2, _ => 3 }
    }
    items.sort_by(|a, b| {
        type_prio(&a.r#type)
            .cmp(&type_prio(&b.r#type))
            .then_with(|| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| a.id.cmp(&b.id))
    });
    let allow_code = std::env::var("CONTEXT_EVIDENCE_ALLOW_CODE").ok().as_deref() == Some("1");
    let mut out = Vec::new();
    for (i, it) in items.into_iter().take(max_items as usize).enumerate() {
        let mut text = it.content.replace('\n', " ");
        if !allow_code { text = text.replace("```", ""); }
        text = redact_secrets(&text);
        let line = format!("- [{}{:03}] {}", type_prefix(&it.r#type), i, one_line(&text));
        out.push(line);
    }
    out
}

fn type_prefix(t: &str) -> &'static str { match t.to_ascii_lowercase().as_str() { "fact" => "F", "procedure" => "P", "episode" => "E", _ => "X" } }

fn redact_secrets(s: &str) -> String {
    let mut out = s.to_string();
    let patterns = [
        (regex_lite::Regex::new("sk-[A-Za-z0-9]{20,}").ok(), "sk-***redacted***"),
        (regex_lite::Regex::new("ghp_[A-Za-z0-9]{20,}").ok(), "ghp_***redacted***"),
        (regex_lite::Regex::new("AKIA[0-9A-Z]{12,}").ok(), "AKIA***redacted***"),
    ];
    for (re, repl) in patterns.iter() { if let Some(re) = re { out = re.replace_all(&out, *repl).to_string(); } }
    out
}

fn count_tokens(s: &str) -> usize {
    s.split_whitespace().count()
}

fn truncate_tokens(s: &str, max_tokens: usize) -> (String, bool) {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.len() <= max_tokens {
        return (s.to_string(), false);
    }
    (tokens[..max_tokens].join(" ") + " …", true)
}

fn one_line(s: &str) -> String {
    let mut out = s.replace('\n', " ");
    if out.len() > 220 {
        out.truncate(220);
        out.push_str(" …");
    }
    out
}

pub struct TokenBudgeter {
    budget: usize,
    quotas: SectionQuotas,
}

impl TokenBudgeter {
    pub fn new(budget: usize, quotas: SectionQuotas) -> Self {
        Self {
            budget: budget.max(1),
            quotas: quotas.normalize(),
        }
    }
    pub fn allocate(&self) -> (usize, usize, usize, usize) {
        let b = self.budget as f32;
        (
            (b * (self.quotas.recent_pct as f32 / 100.0)) as usize,
            (b * (self.quotas.plan_pct as f32 / 100.0)) as usize,
            (b * (self.quotas.evidence_pct as f32 / 100.0)) as usize,
            (b * (self.quotas.tools_pct as f32 / 100.0)) as usize,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotas_normalize_when_off() {
        let q = SectionQuotas {
            recent_pct: 30,
            plan_pct: 30,
            evidence_pct: 30,
            tools_pct: 30,
        }
        .normalize();
        assert_eq!(
            q.recent_pct + q.plan_pct + q.evidence_pct + q.tools_pct,
            100
        );
    }

    #[test]
    fn token_budgeter_alloc_sums() {
        let tb = TokenBudgeter::new(
            1000,
            SectionQuotas {
                recent_pct: 15,
                plan_pct: 10,
                evidence_pct: 60,
                tools_pct: 15,
            },
        );
        let (r, p, e, t) = tb.allocate();
        assert!(r + p + e + t <= 1000 + 3);
    }

    #[test]
    fn truncate_short() {
        let (s, trunc) = truncate_tokens("a b c", 10);
        assert_eq!(s, "a b c");
        assert!(!trunc);
    }

    #[test]
    fn truncate_long() {
        let (s, trunc) = truncate_tokens("a b c d e", 3);
        assert!(trunc);
        assert!(s.starts_with("a b c"));
    }

    #[test]
    fn minimal_provider_populates_sections() {
        let prov = MinimalContextProvider;
        let input = TurnInput {
            user_text: "Ask about system".into(),
            recent_turns: vec!["User: hi".into(), "Assistant: hello".into()],
            plan_text: Some("Step 1\nStep 2".into()),
            tool_deltas: vec!["git diff ...".into()],
            max_context_tokens: 128,
            quotas: SectionQuotas {
                recent_pct: 15,
                plan_pct: 10,
                evidence_pct: 60,
                tools_pct: 15,
            },
        };
        let bundle = prov.build(&input).unwrap();
        assert!(!bundle.recent.is_empty());
        assert!(bundle.evidence.is_empty());
    }
}
