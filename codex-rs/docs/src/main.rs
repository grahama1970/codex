use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about = "Codex reference doc generators")]
enum DocCmd {
    /// Generate CLI markdown (root, exec, chutes) into a directory
    GenerateCli {
        #[arg(value_name = "OUT_DIR")]
        out_dir: String,
    },
    /// Generate config & environment variable reference
    GenerateConfig {
        #[arg(value_name = "OUT_DIR")]
        out_dir: String,
    },
    /// Generate events (JSON schemas / samples)
    GenerateEvents {
        #[arg(value_name = "OUT_DIR")]
        out_dir: String,
    },
    /// Generate a minimal slash commands index (from docs/slash-commands.md)
    GenerateSlash {
        #[arg(value_name = "OUT_DIR")]
        out_dir: String,
    },
    /// Generate a top-level index page under docs/generated
    GenerateIndex {
        #[arg(value_name = "OUT_DIR")]
        out_dir: String,
    },
}

fn main() -> Result<()> {
    let cmd = DocCmd::parse();
    match cmd {
        DocCmd::GenerateCli { out_dir } => gen_cli(&out_dir)?,
        DocCmd::GenerateConfig { out_dir } => gen_config(&out_dir)?,
        DocCmd::GenerateEvents { out_dir } => gen_events(&out_dir)?,
        DocCmd::GenerateSlash { out_dir } => gen_slash(&out_dir)?,
        DocCmd::GenerateIndex { out_dir } => gen_index(&out_dir)?,
    }
    Ok(())
}

fn ensure_dir<P: AsRef<Path>>(p: P) -> Result<()> {
    fs::create_dir_all(&p).with_context(|| format!("creating directory {}", p.as_ref().display()))
}

fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::File::create(path)?;
    f.write_all(content.as_bytes())?;
    Ok(())
}

fn read_cmd_stdout(bin: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new(bin).args(args).output()?;
    if !out.status.success() {
        return Err(anyhow!("command failed: {:?}", out.status));
    }
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&out.stdout));
    Ok(s)
}

fn codex_bin() -> PathBuf {
    env::var_os("CODEX_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist/bin/codex"))
}

// ---------------------------------------------------------------------------
// CLI GENERATION
// ---------------------------------------------------------------------------
fn gen_cli(out_dir: &str) -> Result<()> {
    ensure_dir(out_dir)?;

    let bin = codex_bin();

    // Root help
    let root = read_cmd_stdout(&bin, &["--help"])?;
    write_file(
        &Path::new(out_dir).join("root.md"),
        &wrap_md("Root CLI", &root),
    )?;

    // Exec help
    let exec = read_cmd_stdout(&bin, &["exec", "--help"])?;
    write_file(
        &Path::new(out_dir).join("exec.md"),
        &wrap_md("Exec CLI", &exec),
    )?;

    // Chutes help (if available)
    if let Ok(chutes) = read_cmd_stdout(&bin, &["chutes", "--help"]) {
        write_file(
            &Path::new(out_dir).join("chutes.md"),
            &wrap_md("Chutes Subcommand", &chutes),
        )?;
    }

    Ok(())
}

fn sanitize(input: &str) -> String {
    let mut s = String::from(input);
    s = s.replace("sk-", "sk-REDACTED-");
    s = s
        .lines()
        .filter(|l| !l.contains("experimental_"))
        .collect::<Vec<_>>()
        .join("\n");
    s
}

fn wrap_md(title: &str, body: &str) -> String {
    let body = sanitize(body);
    let sha = commit_sha();
    format!(
        "<!-- generated: commit {sha} -->\n# {title}\n\n_Auto-generated; do not edit manually._\n\n```\n{body}\n```\n"
    )
}

fn commit_sha() -> String {
    if let Ok(s) = std::env::var("DOCS_BUILD_SHA") {
        return s;
    }
    if let Ok(out) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    "unknown".to_string()
}

// ---------------------------------------------------------------------------
// CONFIG + ENV VARS
// ---------------------------------------------------------------------------
fn gen_config(out_dir: &str) -> Result<()> {
    ensure_dir(out_dir)?;
    let mut lines = Vec::<String>::new();
    lines.push("# Configuration & Environment Reference\n".into());
    lines.push("_Auto-generated; keys subject to change pre-beta._\n".into());

    lines.push("## Environment Variables\n".into());
    lines.push("| Name | Default | Description |\n|------|---------|-------------|".into());

    let env_entries = vec![
        ("CODEX_API_KEY", "(unset)", "API key override for exec mode"),
        ("CHUTES_API_KEY", "(unset)", "Chutes OpenAI-compatible key"),
        (
            "CHUTES_API_BASE",
            "https://llm.chutes.ai/v1",
            "Override base URL for Chutes inference",
        ),
        (
            "CHUTES_CATALOG_BASE",
            "https://api.chutes.ai/chutes/",
            "Catalog endpoint",
        ),
        ("CHUTES_WARMUP", "0", "Enable warm-up call before exec"),
        ("CHUTES_WARMUP_SECS", "8", "Warm-up budget seconds"),
        (
            "CHUTES_CATALOG_FIXTURE",
            "(unset)",
            "Path to a static catalog JSON (testing)",
        ),
        (
            "CONTEXT_FORCE_MINIMAL",
            "0",
            "Force minimal context provider",
        ),
        (
            "CONTEXT_DEBUG",
            "0",
            "Emit verbose context summary debugging",
        ),
    ];
    for (k, d, desc) in env_entries {
        lines.push(format!("| `{k}` | `{d}` | {desc} |"));
    }

    lines.push("\n## `[context]` Table Keys\n".into());
    lines.push(
        "| Key | Type | Default | Description |\n|-----|------|---------|-------------|".into(),
    );
    let context_entries = vec![
        (
            "provider",
            "string",
            "minimal",
            "`minimal` or `arango` (experimental)",
        ),
        (
            "max_context_tokens",
            "integer",
            "8192",
            "Global token budget for Knowledge-First bundle",
        ),
        (
            "[context.budget].recent_pct",
            "integer%",
            "15",
            "Recent turns token share",
        ),
        (
            "[context.budget].plan_pct",
            "integer%",
            "10",
            "Plan section token share",
        ),
        (
            "[context.budget].evidence_pct",
            "integer%",
            "60",
            "Evidence token share",
        ),
        (
            "[context.budget].tools_pct",
            "integer%",
            "15",
            "Tool deltas token share",
        ),
        (
            "[context.arango].endpoint",
            "string",
            "http://localhost:8529",
            "Arango endpoint (Phase-1)",
        ),
        (
            "[context.arango].database",
            "string",
            "codex",
            "Arango DB name",
        ),
        (
            "[context.arango].mcp_tool",
            "string",
            "memory-agent",
            "MCP tool id",
        ),
    ];
    for (k, ty, def, desc) in context_entries {
        lines.push(format!("| `{k}` | {ty} | `{def}` | {desc} |"));
    }

    write_file(
        &Path::new(out_dir).join("config_keys.md"),
        &lines.join("\n"),
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// EVENTS (JSON schemas / samples)
// ---------------------------------------------------------------------------
fn gen_events(out_dir: &str) -> Result<()> {
    ensure_dir(out_dir)?;
    let obj = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "context.summary.v1",
        "type": "object",
        "required": ["kind","version","provider","max_context_tokens","budget"],
        "properties": {
            "kind": { "const": "context.summary" },
            "version": { "type": "integer", "const": 1 },
            "provider": { "type": "string" },
            "max_context_tokens": { "type": "integer" },
            "budget": {
                "type": "object",
                "required": ["recent_pct","plan_pct","evidence_pct","tools_pct"],
                "properties": {
                    "recent_pct": { "type": "integer" },
                    "plan_pct": { "type": "integer" },
                    "evidence_pct": { "type": "integer" },
                    "tools_pct": { "type": "integer" }
                }
            }
        },
        "additionalProperties": true
    });

    let path = Path::new(out_dir).join("context-summary-v1.json");
    write_file(&path, &serde_json::to_string_pretty(&obj)?)?;
    // Minimal index page summarizing available schemas
    let idx = "<!-- generated -->\n# Events Index\n\n- `context.summary.v1` — summary of context budgeting and provider info.\n\nSee JSON schema files in this directory for details.";
    write_file(&Path::new(out_dir).join("events_index.md"), idx)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// SLASH COMMANDS INDEX (from existing docs)
// ---------------------------------------------------------------------------
fn gen_slash(out_dir: &str) -> Result<()> {
    ensure_dir(out_dir)?;
    let source = Path::new("docs/slash-commands.md");
    let raw =
        fs::read_to_string(source).with_context(|| format!("reading {}", source.display()))?;
    let mut lines = Vec::<String>::new();
    lines.push("<!-- generated -->".into());
    lines.push("# Slash Commands (Index)".into());
    lines.push("".into());
    for l in raw.lines() {
        let t = l.trim_start();
        if t.starts_with("- `/") {
            lines.push(t.to_string());
        }
    }
    if lines.len() <= 3 {
        lines.push("_No commands discovered in source._".into());
    }
    write_file(
        &Path::new(out_dir).join("slash_commands.md"),
        &lines.join("\n"),
    )
}

// ---------------------------------------------------------------------------
// GENERATED INDEX (links to generated sections)
// ---------------------------------------------------------------------------
fn gen_index(out_dir: &str) -> Result<()> {
    ensure_dir(out_dir)?;
    let sha = commit_sha();
    let content = format!(
        "<!-- generated: commit {sha} -->\n# Auto-Generated Reference\n\nThis directory contains generated reference docs. Do not edit manually.\n\n- [CLI — Root](./cli/root.md)\n- [CLI — Exec](./cli/exec.md)\n- [CLI — Chutes](./cli/chutes.md)\n- [Config & Env](./config/config_keys.md)\n- [Events Index](./events/events_index.md)\n- [Slash Commands](./slash/slash_commands.md)\n"
    );
    write_file(&Path::new(out_dir).join("README.md"), &content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::SystemTime;

    #[test]
    fn generates_config_and_events() {
        let tmp = std::env::temp_dir().join(format!("codex-docs-test-{:?}", SystemTime::now()));
        let cfg_dir = tmp.join("cfg");
        let evt_dir = tmp.join("evt");
        gen_config(cfg_dir.to_str().unwrap()).expect("gen_config");
        gen_events(evt_dir.to_str().unwrap()).expect("gen_events");
        let cfg = cfg_dir.join("config_keys.md");
        let evt = evt_dir.join("context-summary-v1.json");
        assert!(cfg.exists(), "missing config_keys.md");
        assert!(evt.exists(), "missing context-summary-v1.json");
        let cfg_s = fs::read_to_string(cfg).unwrap();
        let evt_s = fs::read_to_string(evt).unwrap();
        assert!(cfg_s.contains("Environment Variables"));
        assert!(evt_s.contains("context.summary"));
    }
}
