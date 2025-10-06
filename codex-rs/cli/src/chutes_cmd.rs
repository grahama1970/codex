use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use clap::Parser;
use codex_common::CliConfigOverrides;
use regex_lite::Regex;
use reqwest::Url;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Parser)]
pub struct ChutesCli {
    #[clap(flatten)]
    pub config_overrides: CliConfigOverrides,

    #[command(subcommand)]
    pub subcommand: ChutesSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum ChutesSubcommand {
    /// Print the best multi‑modal (>=2 modalities incl. text) Chutes model >= min‑params, cheapest output USD/1M.
    Recommend(RecommendArgs),
    /// Run `exec` using the recommended Chutes model.
    Exec(ChutesExecArgs),
    /// Perform a lightweight warm-up request against a Chutes model (max_tokens=1) and print a result line.
    Warmup(ChutesWarmupArgs),
}

#[derive(Debug, Parser, Clone)]
pub struct RecommendArgs {
    /// Minimum effective parameter size (default: 70_000_000_000).
    #[arg(long, default_value_t = 70_000_000_000_i64)]
    pub min_params: i64,

    /// Require listed modalities (comma separated). Default: text plus at least one more modality.
    #[arg(long)]
    pub require_modalities: Option<String>,

    /// Require capability keys (comma separated) present in the catalog item's `capabilities` map (case‑insensitive).
    /// Example: `--require-capabilities coding,code`.
    #[arg(long)]
    pub require_capabilities: Option<String>,

    /// Maximum effective parameter size (optional). Use to avoid SOTA scale.
    #[arg(long)]
    pub max_params: Option<i64>,

    /// Maximum output price (USD per 1M tokens). Filters out expensive models.
    #[arg(long)]
    pub max_output_ppm: Option<f64>,

    /// Print full JSON for the selected catalog item instead of just the model id.
    #[arg(long)]
    pub json: bool,
    /// Include derived base URL (if any) alongside model id in plain output.
    #[arg(long)]
    pub show_base: bool,
}

#[derive(Debug, Parser)]
pub struct ChutesExecArgs {
    /// Prompt to run; pass '-' to read from stdin.
    pub prompt: Option<String>,

    /// Minimum effective parameter size (default: 70_000_000_000).
    #[arg(long, default_value_t = 70_000_000_000_i64)]
    pub min_params: i64,

    /// Require listed modalities (comma separated). Default: text plus at least one more modality.
    #[arg(long)]
    pub require_modalities: Option<String>,

    /// Require capability keys (comma separated) present in the catalog item's `capabilities` map (case‑insensitive).
    #[arg(long)]
    pub require_capabilities: Option<String>,

    /// Additional images (comma‑separated paths) to include (forwarded to exec).
    #[arg(long)]
    pub images: Option<String>,

    /// Force JSON output from exec.
    #[arg(long)]
    pub json: bool,

    /// Wire API to use with the Chutes provider (chat|responses). Default: chat.
    #[arg(long = "wire-api", value_name = "MODE", default_value = "chat")]
    pub wire_api: String,

    /// Optional warm-up seconds: perform a tiny chat completion to warm the model before exec.
    /// You can also set CHUTES_WARMUP=1 and CHUTES_WARMUP_SECS (default 8) via env vars.
    #[arg(long = "warmup-secs")]
    pub warmup_secs: Option<u64>,
}

#[derive(Debug, Parser, Clone)]
pub struct ChutesWarmupArgs {
    /// Seconds to spend warming up (with brief retries), default 8
    #[arg(long = "secs")]
    pub secs: Option<u64>,
    /// Optional explicit model id (openai/<name>), otherwise uses CODEX_MODEL or discovery fallback.
    #[arg(long)]
    pub model: Option<String>,
    /// Dry-run: print success without making any network calls. Also enabled by CHUTES_WARMUP_DRYRUN=1
    #[arg(long)]
    pub dry_run: bool,
}

impl ChutesCli {
    pub async fn run(self, codex_linux_sandbox_exe: Option<PathBuf>) -> Result<()> {
        let Self {
            config_overrides,
            subcommand,
        } = self;

        match subcommand {
            ChutesSubcommand::Recommend(args) => {
                let (_model_id, item) = select_best(&args).await?;
                if args.json {
                    println!("{}", serde_json::to_string_pretty(&item)?);
                } else {
                    let name = item
                        .get("name")
                        .and_then(Value::as_str)
                        .ok_or_else(|| anyhow!("missing name"))?;
                    if args.show_base {
                        if let Some(base) = derive_base_url(&item) {
                            println!("openai/{name} base_url={base}");
                        } else {
                            println!("openai/{name}");
                        }
                    } else {
                        println!("openai/{name}");
                    }
                }
            }
            ChutesSubcommand::Exec(args) => {
                let (model_id, item) = select_best(&RecommendArgs {
                    min_params: args.min_params,
                    require_modalities: args.require_modalities.clone(),
                    require_capabilities: args.require_capabilities.clone(),
                    max_params: None,
                    max_output_ppm: None,
                    json: false,
                    show_base: false,
                })
                .await?;
                // Build argv for ExecCli::parse_from
                let mut argv: Vec<String> = vec![
                    "codex-exec".to_string(),
                    "-c".to_string(),
                    "model_provider=\"chutes\"".to_string(),
                    "-m".to_string(),
                    model_id.clone(),
                ];
                if args.json {
                    argv.push("--json".to_string());
                }
                // Wire API override (default chat)
                let wire = if matches!(args.wire_api.as_str(), "chat" | "responses") {
                    args.wire_api.clone()
                } else {
                    "chat".to_string()
                };
                argv.push("-c".to_string());
                argv.push(format!("model_providers.chutes.wire_api=\"{wire}\""));

                // Base URL override: env beats derived, else keep provider default
                if let Ok(base) = std::env::var("CHUTES_API_BASE") {
                    if !base.is_empty() {
                        argv.push("-c".to_string());
                        argv.push(format!("model_providers.chutes.base_url=\"{base}\""));
                    }
                } else if let Some(derived) = derive_base_url(&item) {
                    argv.push("-c".to_string());
                    argv.push(format!("model_providers.chutes.base_url=\"{derived}\""));
                }

                // Optional warm-up: perform a tiny chat completion to wake the target.
                // Gate behind flag or env CHUTES_WARMUP=1; budget defaults to 8s.
                let do_warmup = args.warmup_secs.is_some()
                    || std::env::var("CHUTES_WARMUP")
                        .map(|v| v == "1")
                        .unwrap_or(false);
                if do_warmup {
                    let base = std::env::var("CHUTES_API_BASE")
                        .or_else(|_| std::env::var("CHUTES_BASE_URL"))
                        .ok()
                        .or_else(|| derive_base_url(&item));
                    let secs = args
                        .warmup_secs
                        .or_else(|| {
                            std::env::var("CHUTES_WARMUP_SECS")
                                .ok()
                                .and_then(|v| v.parse::<u64>().ok())
                        })
                        .unwrap_or(8);
                    if let Some(base_url) = base {
                        let _ = warmup_chat_completion(&base_url, &model_id, secs).await;
                    }
                }
                if let Some(images) = args.images.as_deref()
                    && !images.trim().is_empty()
                {
                    argv.push("-i".to_string());
                    argv.push(images.to_string());
                }
                if let Some(prompt) = args.prompt.clone() {
                    argv.push(prompt);
                }
                let exec_cli = codex_exec::Cli::parse_from(argv);
                codex_exec::run_main(exec_cli, codex_linux_sandbox_exe).await?;
            }
            ChutesSubcommand::Warmup(args) => {
                let secs = args.secs.unwrap_or(8);
                let model_id = if let Some(m) = args.model.clone() {
                    m
                } else if let Ok(m) = std::env::var("CODEX_MODEL") {
                    m
                } else {
                    // Discover a reasonable default coding model (10B–80B, price cap 3.0, text, coding,code)
                    let (m, _item) = select_best(&RecommendArgs {
                        min_params: 10_000_000_000,
                        require_modalities: Some("text".to_string()),
                        require_capabilities: Some("coding,code".to_string()),
                        max_params: Some(80_000_000_000),
                        max_output_ppm: Some(3.0),
                        json: false,
                        show_base: false,
                    })
                    .await?;
                    m
                };
                let base = std::env::var("CHUTES_API_BASE")
                    .ok()
                    .unwrap_or_else(|| "https://llm.chutes.ai/v1".to_string());
                let t0 = std::time::Instant::now();
                let dry = args.dry_run || std::env::var("CHUTES_WARMUP_DRYRUN").map(|v| v=="1").unwrap_or(false);
                if dry {
                    let ms = t0.elapsed().as_millis();
                    println!("warmup: ok (dry-run) model={model_id} base={base} latency_ms={ms}");
                    return Ok(());
                }
                match warmup_chat_completion(&base, &model_id, secs).await {
                    Ok(()) => {
                        let ms = t0.elapsed().as_millis();
                        println!("warmup: ok model={model_id} base={base} latency_ms={ms}");
                    }
                    Err(e) => {
                        println!("warmup: error model={model_id} base={base} err={e}");
                        std::process::exit(1);
                    }
                }
            }
        }

        Ok(())
    }
}

async fn warmup_chat_completion(
    base_url: &str,
    model_id: &str,
    budget_secs: u64,
) -> anyhow::Result<()> {
    use std::time::Duration;
    use std::time::Instant;
    let key = std::env::var("CHUTES_API_KEY")
        .map_err(|_| anyhow::anyhow!("CHUTES_API_KEY required for warm-up"))?;
    // Normalize base_url (no trailing slash)
    let base = base_url.trim_end_matches('/');
    let url = format!("{base}/chat/completions");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let body = serde_json::json!({
        "model": model_id,
        "max_tokens": 1,
        "temperature": 0.0,
        "messages": [{"role":"user","content":"ping"}],
    });
    let deadline = Instant::now() + Duration::from_secs(budget_secs);
    let mut backoff = 1u64;
    while Instant::now() < deadline {
        let resp = client.post(&url).bearer_auth(&key).json(&body).send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                eprintln!("[chutes] warm-up complete for {model_id}");
                return Ok(());
            }
            Ok(r) if r.status().as_u16() == 429 || r.status().is_server_error() => {
                // Backoff and retry
            }
            Ok(_) | Err(_) => {
                // Non-retryable; fall through to backoff
            }
        }
        tokio::time::sleep(Duration::from_secs(backoff)).await;
        backoff = std::cmp::min(backoff * 2, 5);
    }
    eprintln!("[chutes] warm-up timed out for {model_id} ({budget_secs}s)");
    Ok(())
}

fn auth_header() -> Result<String> {
    let key = env::var("CHUTES_API_KEY")
        .map_err(|_| anyhow!("CHUTES_API_KEY is required for Chutes auto-discovery"))?;
    Ok(format!("Bearer {key}"))
}

fn catalog_url() -> Result<Url> {
    let base = env::var("CHUTES_CATALOG_BASE")
        .unwrap_or_else(|_| "https://api.chutes.ai/chutes/".to_string());
    let url = Url::parse(&base).context("invalid CHUTES_CATALOG_BASE")?;
    Ok(url)
}

async fn fetch_catalog() -> Result<Value> {
    if let Ok(fixture) = env::var("CHUTES_CATALOG_FIXTURE")
        && !fixture.trim().is_empty()
    {
        let data = fs::read_to_string(&fixture)
            .with_context(|| format!("reading CHUTES_CATALOG_FIXTURE {fixture}"))?;
        let json: Value =
            serde_json::from_str(&data).with_context(|| "parsing CHUTES_CATALOG_FIXTURE JSON")?;
        return Ok(json);
    }
    let url = catalog_url()?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .context("building reqwest client")?;
    let resp = client
        .get(url)
        .query(&[
            ("include_public", "true"),
            ("include_schemas", "false"),
            ("limit", "10000"),
        ])
        .header("Authorization", auth_header()?)
        .header("Accept", "application/json")
        .send()
        .await
        .context("request to Chutes catalog failed")?;
    let status = resp.status();
    if !status.is_success() {
        if status.as_u16() == 429 {
            bail!("Chutes catalog error: rate limited (429) – retry later");
        } else if status.is_server_error() {
            bail!("Chutes catalog error: upstream server error {status}");
        } else {
            bail!("Chutes catalog error: {status}");
        }
    }
    let json = resp.json::<Value>().await.context("parsing Chutes JSON")?;
    Ok(json)
}

#[derive(Clone, Debug)]
struct ParamsBlock {
    effective: Option<i64>,
}

fn parse_params_from_text(text: &str) -> ParamsBlock {
    let Ok(activated_re) = Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*([MBT])\s*activated") else {
        return ParamsBlock { effective: None };
    };
    let Ok(total_re) =
        Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*([MBT])\s*(?:param|params|parameter|parameters)\b")
    else {
        return ParamsBlock { effective: None };
    };

    fn unit(n: &str, u: &str) -> Option<i64> {
        let n: f64 = n.parse().ok()?;
        let scale = match u.to_ascii_uppercase().as_str() {
            "M" => 1_000_000.0,
            "B" => 1_000_000_000.0,
            "T" => 1_000_000_000_000.0,
            _ => return None,
        };
        Some((n * scale) as i64)
    }

    let mut effective = None;
    if let Some(c) = activated_re.captures(text)
        && let (Some(n), Some(u)) = (c.get(1), c.get(2))
    {
        effective = unit(n.as_str(), u.as_str());
    }
    if effective.is_none()
        && let Some(c) = total_re.captures(text)
        && let (Some(n), Some(u)) = (c.get(1), c.get(2))
    {
        effective = unit(n.as_str(), u.as_str());
    }
    ParamsBlock { effective }
}

fn effective_params(item: &Value) -> i64 {
    let tagline = item.get("tagline").and_then(Value::as_str).unwrap_or("");
    let readme = item.get("readme").and_then(Value::as_str).unwrap_or("");
    let p = parse_params_from_text(tagline);
    if let Some(v) = p.effective {
        return v;
    }
    let p2 = parse_params_from_text(readme);
    p2.effective.unwrap_or(0)
}

fn output_ppm(item: &Value) -> f64 {
    item.get("current_estimated_price")
        .and_then(|v| v.get("per_million_tokens"))
        .and_then(|v| v.get("output"))
        .and_then(|v| v.get("usd"))
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(f64::NAN)
}

fn input_ppm(item: &Value) -> f64 {
    item.get("current_estimated_price")
        .and_then(|v| v.get("per_million_tokens"))
        .and_then(|v| v.get("input"))
        .and_then(|v| v.get("usd"))
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(f64::INFINITY)
}

fn context_len(item: &Value) -> i64 {
    item.get("max_input_tokens")
        .or_else(|| item.get("context_length"))
        .and_then(Value::as_i64)
        .unwrap_or_else(|| {
            item.get("limits")
                .and_then(|v| v.get("max_input_tokens"))
                .and_then(Value::as_i64)
                .unwrap_or(0)
        })
}

fn is_multimodal(item: &Value, require: Option<&str>) -> bool {
    let mods = item.get("modalities").and_then(Value::as_array);
    let Some(mods) = mods else { return false };
    let set: Vec<String> = mods
        .iter()
        .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
        .collect();
    if !set.iter().any(|m| m == "text") {
        return false;
    }
    if let Some(req) = require {
        for needed in req.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            if !set.iter().any(|m| m == needed) {
                return false;
            }
        }
    } else if set.len() < 2 {
        return false;
    }
    true
}

fn has_required_capabilities(item: &Value, require: Option<&str>) -> bool {
    let Some(req) = require else { return true };
    let caps = item.get("capabilities").and_then(Value::as_object);
    let Some(caps) = caps else { return false };
    let keys_lower: std::collections::HashSet<String> =
        caps.keys().map(|k| k.to_lowercase()).collect();
    for needed in req.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if !keys_lower.contains(&needed.to_lowercase()) {
            return false;
        }
    }
    true
}

pub async fn select_best(args: &RecommendArgs) -> Result<(String, Value)> {
    let catalog = fetch_catalog().await?;
    let items = catalog
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("invalid catalog payload: missing items"))?;

    let debug = std::env::var("CHUTES_DISCOVERY_DEBUG")
        .map(|v| v == "1")
        .unwrap_or(false);
    // (out_ppm, in_ppm, eff, ctx, model_id, item)
    let mut candidates: Vec<(f64, f64, i64, i64, String, Value)> = Vec::new();
    let mut price_nan_exclusions: usize = 0;
    let mut any_seen = false;

    for item in items {
        any_seen = true;
        if !is_multimodal(item, args.require_modalities.as_deref()) {
            if debug {
                eprintln!("[chutes] skip: not multimodal");
            }
            continue;
        }
        if !has_required_capabilities(item, args.require_capabilities.as_deref()) {
            if debug {
                eprintln!("[chutes] skip: missing required capabilities");
            }
            continue;
        }
        let eff = effective_params(item);
        if eff < args.min_params {
            if debug {
                eprintln!("[chutes] skip: eff_params {eff} < min {}", args.min_params);
            }
            continue;
        }
        if let Some(maxp) = args.max_params
            && eff > maxp
        {
            if debug {
                eprintln!("[chutes] skip: eff_params {eff} > max {maxp}");
            }
            continue;
        }
        let out_ppm = output_ppm(item);
        if let Some(cap) = args.max_output_ppm {
            if !out_ppm.is_finite() {
                price_nan_exclusions += 1;
                if debug {
                    eprintln!("[chutes] skip: price NaN under cap");
                }
                continue;
            }
            if out_ppm > cap {
                if debug {
                    eprintln!("[chutes] skip: price {out_ppm} > cap {cap}");
                }
                continue;
            }
        }
        let in_ppm = input_ppm(item);
        let ctx = context_len(item);
        let name = item
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("missing name field"))?;
        let model_id = format!("openai/{name}");
        candidates.push((out_ppm, in_ppm, eff, ctx, model_id, item.clone()));
    }

    // If everything was filtered due to NaN price while a cap was set, retry selection inline without the cap.
    if candidates.is_empty()
        && args.max_output_ppm.is_some()
        && price_nan_exclusions > 0
        && any_seen
    {
        if debug {
            eprintln!("[chutes-relax] relaxing price cap (all candidates had NaN price)");
        }
        let mut relaxed_candidates: Vec<(f64, f64, i64, i64, String, Value)> = Vec::new();
        for item in items {
            if !is_multimodal(item, args.require_modalities.as_deref()) {
                continue;
            }
            if !has_required_capabilities(item, args.require_capabilities.as_deref()) {
                continue;
            }
            let eff = effective_params(item);
            if eff < args.min_params {
                continue;
            }
            if let Some(maxp) = args.max_params
                && eff > maxp
            {
                continue;
            }
            let out_ppm = output_ppm(item);
            // When relaxed, accept NaN/out-of-spec price.
            let in_ppm = input_ppm(item);
            let ctx = context_len(item);
            let name = match item.get("name").and_then(Value::as_str) {
                Some(n) => n,
                None => continue,
            };
            let model_id = format!("openai/{name}");
            relaxed_candidates.push((out_ppm, in_ppm, eff, ctx, model_id, item.clone()));
        }
        if let Some(best) = relaxed_candidates.into_iter().min_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.2.cmp(&a.2))
                .then(b.3.cmp(&a.3))
                .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        }) {
            return Ok((best.4, best.5));
        }
    }

    if candidates.is_empty() {
        let mut hints = Vec::new();
        if let Some(cap) = args.max_output_ppm {
            hints.push(format!("max_output_ppm={cap}"));
        }
        if let Some(req) = &args.require_capabilities {
            hints.push(format!("capabilities={req}"));
        }
        if let Some(mods) = &args.require_modalities {
            hints.push(format!("modalities={mods}"));
        }
        if let Some(mx) = args.max_params {
            hints.push(format!("max_params={mx}"));
        }
        let hint = if hints.is_empty() {
            String::new()
        } else {
            format!(" (filters: {})", hints.join(", "))
        };
        bail!(
            "No suitable Chutes model found (>= {} params, multi-modal){}",
            args.min_params,
            hint
        );
    }

    // O(n) selection: out_ppm asc, eff desc, ctx desc, in_ppm asc
    let best = candidates
        .into_iter()
        .min_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.2.cmp(&a.2))
                .then(b.3.cmp(&a.3))
                .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        })
        .expect("non-empty");
    Ok((best.4, best.5))
}

pub fn derive_base_url(item: &Value) -> Option<String> {
    if let Some(dom) = item.get("domain").and_then(Value::as_str)
        && !dom.is_empty()
    {
        let dom = dom
            .trim()
            .trim_end_matches('/')
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        let sanitized: String = dom
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.')
            .collect();
        if sanitized.is_empty() || sanitized != dom {
            return None;
        }
        return Some(format!("https://{sanitized}/v1"));
    }
    let owner = item
        .get("owner")
        .or_else(|| item.get("username"))
        .or_else(|| item.get("user"))
        .and_then(Value::as_str);
    let slug = item
        .get("slug")
        .and_then(Value::as_str)
        .map(std::string::ToString::to_string)
        .or_else(|| {
            item.get("name")
                .and_then(Value::as_str)
                .map(|s| s.split('/').next_back().unwrap_or(s).to_string())
        });
    match (owner, slug) {
        (Some(o), Some(s)) if !o.is_empty() && !s.is_empty() => {
            let sanitize = |x: &str| {
                x.chars()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
                    .collect::<String>()
            };
            let so = sanitize(o);
            let ss = sanitize(&s);
            if so.is_empty() || ss.is_empty() {
                return None;
            }
            Some(format!("https://{so}-{ss}.chutes.ai/v1"))
        }
        _ => None,
    }
}

/// Convenience wrapper used by other CLI paths to auto‑discover a model and
/// return `(model_id, derived_base_url)` where `model_id` is
/// `openai/<catalog_id>` and `derived_base_url` is a best‑effort per‑chute
/// OpenAI‑compatible base URL (None if not derivable).
pub async fn discover_model_and_base(
    min_params: i64,
    require_modalities: Option<String>,
) -> Result<(String, Option<String>)> {
    let (model_id, item) = select_best(&RecommendArgs {
        min_params,
        require_modalities,
        require_capabilities: None,
        max_params: None,
        max_output_ppm: None,
        json: false,
        show_base: false,
    })
    .await?;
    let base = derive_base_url(&item);
    Ok((model_id, base))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_params_activated() {
        let p = parse_params_from_text("Uses 72B activated params, next-gen");
        assert!(p.effective.unwrap() >= 70_000_000_000);
    }

    #[test]
    fn parse_params_total() {
        let p = parse_params_from_text("Model with 12.5B parameters, optimized");
        assert_eq!(p.effective.unwrap(), 12_500_000_000);
    }

    #[test]
    fn derive_sanitized_base() {
        let item = serde_json::json!({"owner":"alpha-team","slug":"ultra-model"});
        assert_eq!(
            derive_base_url(&item),
            Some("https://alpha-team-ultra-model.chutes.ai/v1".to_string())
        );
    }
}
