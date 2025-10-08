//! Phase-1 retrieval logic (JSON-RPC calls to memory-agent / Arango).
//! - Provides search + optional neighbors with a single timeout budget.
//! - Falls back silently (debug log if CONTEXT_DEBUG=1) on errors.
//! - No raw evidence lines ever leave this module; shaping occurs in lib.rs.

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use sha1::Digest;
use sha1::Sha1;
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub struct RetrievalConfig {
    pub endpoint: String,
    pub search_k: u32,
    pub neighbors_depth: u32,
    pub timeout_ms: u64,
    pub debug: bool,
    pub allow_code: bool,
    pub max_items: usize,
    pub fixture_path: Option<String>,
}

/// Internal representation of a node returned by memory.* calls.
#[derive(Debug, Clone, Deserialize)]
pub struct MemoryNode {
    pub id: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub tags: Option<Value>,
    #[serde(default)]
    pub meta: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct RetrievalBundle {
    pub nodes: Vec<MemoryNode>,
    pub retrieval_ms: u64,
    pub evidence_items: usize,
    pub cache_hit: bool,
    pub retry_count: Option<u8>,
    pub fallback_reason: Option<String>,
}

/// Lightweight JSON-RPC client (single-method POST).
struct JsonRpcClient {
    base: String,
    debug: bool,
    http: reqwest::Client,
}

impl JsonRpcClient {
    fn new(base: String, debug: bool) -> Self {
        Self {
            base,
            debug,
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": "1",
            "method": method,
            "params": params,
        });
        let res = self
            .http
            .post(&self.base)
            .json(&payload)
            .send()
            .await
            .context("json-rpc post failed")?;
        let status = res.status();
        if status.as_u16() == 429 {
            return Err(anyhow!("429"));
        }
        if !status.is_success() {
            return Err(anyhow!("http status {}", status));
        }
        let val: Value = res.json().await.context("json-rpc parse")?;
        Ok(val)
    }
}

/// Derive a per-turn cache key.
pub fn compute_cache_key(query_norm: &str, search_k: u32, neighbors_depth: u32) -> String {
    let mut h = Sha1::new();
    h.update(query_norm.as_bytes());
    h.update(b"|");
    h.update(search_k.to_string().as_bytes());
    h.update(b"|");
    h.update(neighbors_depth.to_string().as_bytes());
    format!("{:x}", h.finalize())
}

/// Execute retrieval or return fallback empty bundle.
pub async fn perform_retrieval(
    cfg: &RetrievalConfig,
    query_raw: &str,
    cache: &mut HashMap<String, RetrievalBundle>,
) -> RetrievalBundle {
    // Fixture path short-circuit (deterministic integration tests).
    if let Some(fixture) = &cfg.fixture_path {
        return load_fixture(fixture, cfg);
    }

    let t0 = Instant::now();

    let query_norm = normalize_query(query_raw);
    let key = compute_cache_key(&query_norm, cfg.search_k, cfg.neighbors_depth);

    if let Some(existing) = cache.get(&key).cloned() {
        return RetrievalBundle {
            cache_hit: true,
            retry_count: existing.retry_count,
            fallback_reason: existing.fallback_reason.clone(),
            ..existing
        };
    }

    let mut nodes: Vec<MemoryNode> = vec![];
    let deadline = Duration::from_millis(cfg.timeout_ms);
    let client = JsonRpcClient::new(cfg.endpoint.clone(), cfg.debug);

    let mut retry_attempts: u8 = 0;
    let fut = async {
        // memory.search
        let search_params = json!({
          "q": query_norm,
          "k": cfg.search_k,
          "types": ["facts","procedures","episodes"]
        });

        let mut search_val = client.call("memory.search", search_params).await?;
        if let Some(arr) = search_val
            .get_mut("result")
            .and_then(|v| v.get_mut("items"))
            .and_then(|v| v.as_array_mut())
        {
            for v in arr {
                if let Ok(node) = serde_json::from_value::<MemoryNode>(v.clone()) {
                    nodes.push(node);
                }
            }
        } else {
            return Err(anyhow!("schema mismatch: search.result.items"));
        }

        // neighbors
        if cfg.neighbors_depth > 0 {
            let neighbor_ids: Vec<String> = nodes.iter().take(5).map(|n| n.id.clone()).collect();
            for id in neighbor_ids {
                if nodes.len() >= cfg.max_items * 2 {
                    break;
                }
                let params = json!({
                  "id": id,
                  "depth": cfg.neighbors_depth,
                  "types": ["facts","procedures"]
                });
                match client.call("memory.neighbors", params).await {
                    Ok(val) => {
                        if let Some(items) = val
                            .get("result")
                            .and_then(|r| r.get("items"))
                            .and_then(|i| i.as_array())
                        {
                            for it in items {
                                if let Ok(node) = serde_json::from_value::<MemoryNode>(it.clone()) {
                                    nodes.push(node);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if cfg.debug {
                            eprintln!("[context] neighbors error: {e}");
                        }
                    }
                }
            }
        }

        Ok::<(), anyhow::Error>(())
    };

    // First attempt with overall deadline
    let first = timeout(deadline, fut).await;
    if first.is_err() {
        return fallback_bundle(t0, cfg, true);
    }
    if let Ok(Err(e)) = &first {
        if e.to_string().contains("429") {
            // Retry once with small backoff if time remains
            let spent = t0.elapsed();
            if spent >= deadline {
                return fallback_bundle(t0, cfg, false);
            }
            let remain = deadline - spent;
            if cfg.debug {
                eprintln!(
                    "[context] search 429, retrying after 100ms (remain {:?})",
                    remain
                );
            }
            if remain > Duration::from_millis(150) {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            // Re-run full flow with fresh client and local buffers
            let mut nodes_retry: Vec<MemoryNode> = vec![];
            let retry_client = JsonRpcClient::new(cfg.endpoint.clone(), cfg.debug);
            let query_norm = normalize_query(query_raw);
            let retry = async {
                let search_params = json!({
                  "q": query_norm,
                  "k": cfg.search_k,
                  "types": ["facts","procedures","episodes"]
                });
                let search_val = retry_client.call("memory.search", search_params).await?;
                if let Some(arr) = search_val
                    .get("result")
                    .and_then(|v| v.get("items"))
                    .and_then(|v| v.as_array())
                {
                    for v in arr {
                        if let Ok(node) = serde_json::from_value::<MemoryNode>(v.clone()) {
                            nodes_retry.push(node);
                        }
                    }
                } else {
                    return Err(anyhow!("schema mismatch: search.result.items"));
                }
                if cfg.neighbors_depth > 0 {
                    let ids: Vec<String> =
                        nodes_retry.iter().take(5).map(|n| n.id.clone()).collect();
                    for id in ids {
                        let params = json!({
                          "id": id,
                          "depth": cfg.neighbors_depth,
                          "types": ["facts","procedures"]
                        });
                        if let Ok(val) = retry_client.call("memory.neighbors", params).await {
                            if let Some(items) = val
                                .get("result")
                                .and_then(|r| r.get("items"))
                                .and_then(|i| i.as_array())
                            {
                                for it in items {
                                    if let Ok(node) =
                                        serde_json::from_value::<MemoryNode>(it.clone())
                                    {
                                        nodes_retry.push(node);
                                    }
                                }
                            }
                        }
                        if nodes_retry.len() >= cfg.max_items * 2 {
                            break;
                        }
                    }
                }
                Ok::<(), anyhow::Error>(())
            };
            let remain_after = deadline.saturating_sub(t0.elapsed());
            let retry_res = timeout(remain_after, retry).await;
            match retry_res {
                Err(_) => return fallback_bundle(t0, cfg, true),
                Ok(Err(_)) => return fallback_bundle(t0, cfg, false),
                Ok(Ok(())) => {
                    retry_attempts = 1;
                    if nodes.is_empty() {
                        nodes = nodes_retry;
                    }
                }
            }
        } else {
            return fallback_bundle(t0, cfg, false);
        }
    }

    let retrieval_ms = t0.elapsed().as_millis() as u64;
    let mut seen = std::collections::HashSet::new();
    nodes.retain(|n| seen.insert(n.id.clone()));
    let final_nodes = nodes;
    let bundle = RetrievalBundle {
        retrieval_ms,
        evidence_items: final_nodes.len(),
        nodes: final_nodes,
        cache_hit: false,
        retry_count: Some(retry_attempts),
        fallback_reason: None,
    };
    cache.insert(key, bundle.clone());
    bundle
}

fn fallback_bundle(start: Instant, cfg: &RetrievalConfig, timeout_hit: bool) -> RetrievalBundle {
    if cfg.debug {
        eprintln!(
            "[context] retrieval fallback: {}",
            if timeout_hit { "timeout" } else { "error" }
        );
    }
    RetrievalBundle {
        nodes: vec![],
        retrieval_ms: start.elapsed().as_millis() as u64,
        evidence_items: 0,
        cache_hit: false,
        retry_count: None,
        fallback_reason: Some(if timeout_hit { "timeout" } else { "error" }.to_string()),
    }
}

fn normalize_query(raw: &str) -> String {
    let trimmed = raw.trim().to_lowercase();
    if trimmed.len() <= 256 {
        trimmed
    } else {
        trimmed.chars().take(256).collect()
    }
}

fn load_fixture(path: &str, _cfg: &RetrievalConfig) -> RetrievalBundle {
    match std::fs::read_to_string(path) {
        Ok(s) => {
            let mut nodes: Vec<MemoryNode> = vec![];
            if let Ok(val) = serde_json::from_str::<Value>(&s) {
                if let Some(items) = val
                    .get("search")
                    .and_then(|v| v.get("items"))
                    .and_then(Value::as_array)
                {
                    for it in items {
                        if let Ok(node) = serde_json::from_value::<MemoryNode>(it.clone()) {
                            nodes.push(node);
                        }
                    }
                }
                if let Some(neigh) = val
                    .get("neighbors")
                    .and_then(|v| v.get("items"))
                    .and_then(Value::as_array)
                {
                    for it in neigh {
                        if let Ok(node) = serde_json::from_value::<MemoryNode>(it.clone()) {
                            nodes.push(node);
                        }
                    }
                }
            }
            RetrievalBundle {
                nodes: nodes.clone(),
                retrieval_ms: 5,
                evidence_items: nodes.len(),
                cache_hit: false,
                retry_count: Some(0),
                fallback_reason: None,
            }
        }
        Err(_) => RetrievalBundle {
            nodes: vec![],
            retrieval_ms: 0,
            evidence_items: 0,
            cache_hit: false,
            retry_count: None,
            fallback_reason: Some("error".to_string()),
        },
    }
}
