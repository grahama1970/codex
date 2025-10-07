use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;

mod retrieval;
use retrieval::{perform_retrieval, MemoryNode, RetrievalBundle, RetrievalConfig};

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

#[derive(Debug, Clone, Default, Serialize)]
pub struct ContextMetrics {
    pub retrieval_ms: u64,
    pub evidence_items: u32,
    pub search_k: u32,
    pub neighbors_depth: u32,
    pub reflowed_from_plan: u32,
    pub reflowed_from_recent: u32,
    pub reflowed_from_tools: u32,
    pub total_tokens: u32,
    pub evidence_tokens: u32,
    pub plan_tokens: u32,
    pub recent_tokens: u32,
    pub tools_tokens: u32,
    pub truncated_evidence: bool,
    pub truncated_plan: bool,
    pub truncated_recent: bool,
    pub truncated_tools: bool,
}

/// Trait for pluggable providers (minimal vs arango)
pub trait ContextProvider: Send + Sync {
    fn build(&self, input: &TurnInput) -> Result<(EvidenceBundle, ContextMetrics), ContextError>;
}

pub struct MinimalContextProvider;

impl ContextProvider for MinimalContextProvider {
    fn build(&self, input: &TurnInput) -> Result<(EvidenceBundle, ContextMetrics), ContextError> {
        let quotas = input.quotas.normalize();
        let budget = input.max_context_tokens.max(256);
        let recent_budget = (budget as f32 * (quotas.recent_pct as f32 / 100.0)) as usize;
        let plan_budget = (budget as f32 * (quotas.plan_pct as f32 / 100.0)) as usize;
        let tools_budget = (budget as f32 * (quotas.tools_pct as f32 / 100.0)) as usize;
        let evidence_budget = (budget as f32 * (quotas.evidence_pct as f32 / 100.0)) as usize;

        let recent_concat = input.recent_turns.join("\n");
        let (recent, trunc_recent) = truncate_tokens(&recent_concat, recent_budget);
        let (plan, trunc_plan) = truncate_tokens(input.plan_text.as_deref().unwrap_or(""), plan_budget);
        let deltas = input.tool_deltas.join("\n");
        let (tools, trunc_tools) = truncate_tokens(&deltas, tools_budget);
        let (evidence, trunc_evidence) = truncate_tokens("", evidence_budget);

        let bundle = EvidenceBundle {
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
        };
        let metrics = ContextMetrics {
            retrieval_ms: 0,
            evidence_items: 0,
            search_k: 0,
            neighbors_depth: 0,
            reflowed_from_plan: 0,
            reflowed_from_recent: 0,
            reflowed_from_tools: 0,
            total_tokens: (bundle.recent_tokens
                + bundle.plan_tokens
                + bundle.evidence_tokens
                + bundle.tools_tokens) as u32,
            evidence_tokens: bundle.evidence_tokens,
            plan_tokens: bundle.plan_tokens,
            recent_tokens: bundle.recent_tokens,
            tools_tokens: bundle.tools_tokens,
            truncated_evidence: bundle.truncated_evidence,
            truncated_plan: bundle.truncated_plan,
            truncated_recent: bundle.truncated_recent,
            truncated_tools: bundle.truncated_tools,
        };
        Ok((bundle, metrics))
    }
}

/// Phase‑1 Arango/memory-agent.
pub struct ArangoContextProvider {
    pub mcp_tool: String,
    pub endpoint: String,
    pub database: String,
    pub search_k: u32,
    pub neighbors_depth: u8,
    pub timeout_ms: u64,
    pub max_evidence_items: u32,
    pub debug: bool,
    pub allow_code: bool,
    pub fixture_path: Option<String>,
}

impl ContextProvider for ArangoContextProvider {
    fn build(&self, input: &TurnInput) -> Result<(EvidenceBundle, ContextMetrics), ContextError> {
        // Retrieval (fixture or JSON‑RPC)
        let cfg = RetrievalConfig {
            endpoint: self.endpoint.clone(),
            search_k: self.search_k,
            neighbors_depth: self.neighbors_depth as u32,
            timeout_ms: self.timeout_ms,
            debug: self.debug,
            allow_code: self.allow_code,
            max_items: self.max_evidence_items as usize,
            fixture_path: self
                .fixture_path
                .clone()
                .or_else(|| std::env::var("CONTEXT_MCP_FIXTURE").ok()),
        };
        // Build a tiny runtime for the async call (keeps trait sync for now)
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| ContextError::Internal(format!("rt: {e}")))?;
        let mut cache = HashMap::new();
        let retrieval = rt.block_on(async { perform_retrieval(&cfg, &input.user_text, &mut cache).await });

        // Shape evidence
        let lines = shape_evidence(&retrieval.nodes, self.max_evidence_items as usize, self.allow_code);
        let evidence_text = if lines.is_empty() { String::new() } else { lines.join("\n") };

        // Build minimal and merge
        let (mut out, mut metrics) = MinimalContextProvider.build(input)?;
        out.evidence = evidence_text;
        out.evidence_tokens = count_tokens(&out.evidence) as u32;

        // Adaptive reflow (heuristic) and metrics
        let quotas = input.quotas.normalize();
        if out.plan_tokens == 0 {
            metrics.reflowed_from_plan = (quotas.plan_pct as u32) * input.max_context_tokens as u32 / 100;
        }
        if out.recent_tokens == 0 {
            metrics.reflowed_from_recent = (quotas.recent_pct as u32) * input.max_context_tokens as u32 / 100;
        }
        if out.tools_tokens == 0 {
            metrics.reflowed_from_tools = (quotas.tools_pct as u32) * input.max_context_tokens as u32 / 100;
        }

        metrics.retrieval_ms = retrieval.retrieval_ms;
        metrics.evidence_items = retrieval.evidence_items as u32;
        metrics.search_k = self.search_k;
        metrics.neighbors_depth = self.neighbors_depth as u32;
        metrics.total_tokens = (out.recent_tokens + out.plan_tokens + out.evidence_tokens + out.tools_tokens) as u32;
        metrics.evidence_tokens = out.evidence_tokens;
        metrics.truncated_evidence = out.truncated_evidence;

        Ok((out, metrics))
    }
}

// ---------- Evidence shaping helpers ----------

fn shape_evidence(nodes: &[MemoryNode], max_items: usize, allow_code: bool) -> Vec<String> {
    let mut vec: Vec<_> = nodes
        .iter()
        .filter(|n| matches!(n.r#type.as_str(), "fact" | "procedure" | "episode"))
        .cloned()
        .collect();
    vec.sort_by(|a, b| {
        let pa = type_prio(&a.r#type);
        let pb = type_prio(&b.r#type);
        pa.cmp(&pb)
            .then_with(|| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for n in vec.into_iter() {
        if !seen.insert(n.id.clone()) { continue; }
        if out.len() >= max_items { break; }
        let mut text = n.content.replace('\n', " ");
        if !allow_code { text = strip_code_fences(&text); }
        text = redact_secrets(&text);
        if text.len() > 220 { text = text.chars().take(220).collect::<String>() + " …"; }
        out.push(format!("- [{}{}] {}", type_prefix(&n.r#type), short_hash(&n.id), text));
    }
    out
}

fn type_prio(t: &str) -> u8 {
    match t {
        "fact" => 0,
        "procedure" => 1,
        "episode" => 2,
        _ => 3,
    }
}

fn type_prefix(t: &str) -> &'static str {
    match t.to_ascii_lowercase().as_str() {
        "fact" => "F",
        "procedure" => "P",
        "episode" => "E",
        _ => "X",
    }
}

fn redact_secrets(s: &str) -> String {
    let mut out = s.to_string();
    let patterns: &[&str] = &[
        r"sk-[A-Za-z0-9]{16,}",
        r"ghp_[A-Za-z0-9]{36}",
        r"gh_[A-Za-z0-9]{36}",
        r"AKIA[0-9A-Z]{16}",
        r"xox[abp]-[A-Za-z0-9\-]{10,}",
        r"ya29\.[A-Za-z0-9._\-]+",
        r"AIzaSy[A-Za-z0-9_\-]{20,}",
        r"sk_live_[A-Za-z0-9]{16,}",
        r"sk_test_[A-Za-z0-9]{16,}",
        r"AC[0-9A-F]{32}",
        r"sbp_[A-Za-z0-9]{20,}",
    ];
    for pat in patterns {
        if let Ok(re) = regex::Regex::new(pat) {
            out = re.replace_all(&out, "REDACTED").into_owned();
        }
    }
    out
}

fn count_tokens(s: &str) -> usize { s.split_whitespace().count() }

fn truncate_tokens(s: &str, max_tokens: usize) -> (String, bool) {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.len() <= max_tokens { return (s.to_string(), false); }
    (tokens[..max_tokens].join(" ") + " …", true)
}

fn strip_code_fences(s: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    for line in s.split('\n') {
        let l = line.trim();
        if l.starts_with("```") { in_fence = !in_fence; continue; }
        if !in_fence { out.push_str(line); out.push(' '); }
    }
    out.trim().to_string()
}

fn short_hash(id: &str) -> String {
    let mut h = Sha1::new();
    h.update(id.as_bytes());
    let s = format!("{:x}", h.finalize());
    s.chars().rev().take(4).collect::<String>().chars().rev().collect()
}

fn one_line(s: &str) -> String {
    let mut out = s.replace('\n', " ");
    if out.len() > 220 {
        out.truncate(220);
        out.push_str(" …");
    }
    out
}

