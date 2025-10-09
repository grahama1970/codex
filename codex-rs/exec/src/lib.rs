// - In the default output mode, it is paramount that the only thing written to
//   stdout is the final message (if any).
// - In --json mode, stdout must be valid JSONL, one event per line.
// For both modes, any other output must be written to stderr.
#![deny(clippy::print_stdout)]

mod cli;
mod event_processor;
mod event_processor_with_human_output;
pub mod event_processor_with_jsonl_output;
pub mod exec_events;

pub use cli::Cli;
use codex_common::slash;
use codex_context::ArangoContextProvider;
use codex_context::ContextMetrics;
use codex_context::ContextProvider as KfProvider;
use codex_context::MinimalContextProvider;
use codex_context::SectionQuotas;
use codex_context::TurnInput;
use codex_core::AuthManager;
use codex_core::BUILT_IN_OSS_MODEL_PROVIDER_ID;
use codex_core::ConversationManager;
use codex_core::NewConversation;
use codex_core::config::Config;
use codex_core::config::ConfigOverrides;
use codex_core::git_info::get_git_repo_root;
use codex_core::protocol::AskForApproval;
use codex_core::protocol::Event;
use codex_core::protocol::EventMsg;
use codex_core::protocol::InputItem;
use codex_core::protocol::Op;
use codex_core::protocol::SessionSource;
use codex_core::protocol::TaskCompleteEvent;
use codex_ollama::DEFAULT_OSS_MODEL;
use codex_protocol::config_types::SandboxMode;
use event_processor_with_human_output::EventProcessorWithHumanOutput;
use event_processor_with_jsonl_output::EventProcessorWithJsonOutput;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use serde_json::Value;
use std::io::IsTerminal;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

fn context_summary_emitted() -> &'static std::sync::atomic::AtomicBool {
    static ONCE: OnceLock<std::sync::atomic::AtomicBool> = OnceLock::new();
    ONCE.get_or_init(|| std::sync::atomic::AtomicBool::new(false))
}
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use supports_color::Stream;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

use crate::cli::Command as ExecCommand;
use crate::event_processor::CodexStatus;
use crate::event_processor::EventProcessor;
use codex_core::default_client::set_default_originator;
use codex_core::find_conversation_path_by_id_str;

pub async fn run_main(cli: Cli, codex_linux_sandbox_exe: Option<PathBuf>) -> anyhow::Result<()> {
    if let Err(err) = set_default_originator("codex_exec") {
        tracing::warn!(?err, "Failed to set codex exec originator override {err:?}");
    }

    let Cli {
        command,
        images,
        model: model_cli_arg,
        oss,
        config_profile,
        full_auto,
        dangerously_bypass_approvals_and_sandbox,
        cwd,
        skip_git_repo_check,
        color,
        last_message_file,
        json: json_mode,
        sandbox_mode: sandbox_mode_cli_arg,
        prompt,
        output_schema: output_schema_path,
        include_plan_tool,
        config_overrides,
        run_timeout_secs,
        summary_dir,
        shutdown_grace_ms,
        force_cli_source,
        keep_approval_policy,
        seed,
        ..
    } = cli;

    // Apply non-interactive sane defaults (avoid mutating process env here to honor sandboxing policies).

    // Determine the prompt source (parent or subcommand) and read from stdin if needed.
    let prompt_arg = match &command {
        // Allow prompt before the subcommand by falling back to the parent-level prompt
        // when the Resume subcommand did not provide its own prompt.
        Some(ExecCommand::Resume(args)) => args.prompt.clone().or(prompt),
        None => prompt,
    };

    let prompt = match prompt_arg {
        Some(p) if p != "-" => p,
        // Either `-` was passed or no positional arg.
        maybe_dash => {
            // When no arg (None) **and** stdin is a TTY, bail out early – unless the
            // user explicitly forced reading via `-`.
            let force_stdin = matches!(maybe_dash.as_deref(), Some("-"));

            if std::io::stdin().is_terminal() && !force_stdin {
                eprintln!(
                    "No prompt provided. Either specify one as an argument or pipe the prompt into stdin."
                );
                std::process::exit(1);
            }

            // Ensure the user knows we are waiting on stdin, as they may
            // have gotten into this state by mistake. If so, and they are not
            // writing to stdin, Codex will hang indefinitely, so this should
            // help them debug in that case.
            if !force_stdin {
                eprintln!("Reading prompt from stdin...");
            }
            let mut buffer = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut buffer) {
                eprintln!("Failed to read prompt from stdin: {e}");
                std::process::exit(1);
            } else if buffer.trim().is_empty() {
                eprintln!("No prompt provided via stdin.");
                std::process::exit(1);
            }
            buffer
        }
    };

    // Handle slash commands (one-line commands starting with '/') locally and exit early.
    if let Some(cmd) = slash::parse(&prompt) {
        handle_slash_command(cmd).await?;
        return Ok(());
    }

    let output_schema = load_output_schema(output_schema_path);

    let (stdout_with_ansi, stderr_with_ansi) = match color {
        cli::Color::Always => (true, true),
        cli::Color::Never => (false, false),
        cli::Color::Auto => (
            supports_color::on_cached(Stream::Stdout).is_some(),
            supports_color::on_cached(Stream::Stderr).is_some(),
        ),
    };
    // Ensure consistent no-color for headless/json contexts (do not mutate env; renderer selection handled above).

    // Build fmt layer (existing logging) to compose with OTEL layer.
    let default_level = "error";

    // Build env_filter separately and attach via with_filter.
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(default_level))
        .unwrap_or_else(|_| EnvFilter::new(default_level));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(stderr_with_ansi)
        .with_writer(std::io::stderr)
        .with_filter(env_filter);

    let sandbox_mode = if full_auto {
        Some(SandboxMode::WorkspaceWrite)
    } else if dangerously_bypass_approvals_and_sandbox {
        Some(SandboxMode::DangerFullAccess)
    } else {
        sandbox_mode_cli_arg.map(Into::<SandboxMode>::into)
    };

    // When using `--oss`, let the bootstrapper pick the model (defaulting to
    // gpt-oss:20b) and ensure it is present locally. Also, force the built‑in
    // `oss` model provider.
    let model = if let Some(model) = model_cli_arg {
        Some(model)
    } else if oss {
        Some(DEFAULT_OSS_MODEL.to_owned())
    } else {
        None // No model specified, will use the default.
    };

    let model_provider = if oss {
        Some(BUILT_IN_OSS_MODEL_PROVIDER_ID.to_string())
    } else {
        None // No specific model provider override.
    };

    // Load configuration and determine approval policy
    let approval_policy_override = if keep_approval_policy {
        None
    } else {
        Some(AskForApproval::Never)
    };
    let overrides = ConfigOverrides {
        model,
        review_model: None,
        config_profile,
        // Preserve headless default unless user opts to keep approval policy.
        approval_policy: approval_policy_override,
        sandbox_mode,
        cwd: cwd.map(|p| p.canonicalize().unwrap_or(p)),
        model_provider,
        codex_linux_sandbox_exe,
        base_instructions: None,
        include_plan_tool: Some(include_plan_tool),
        include_apply_patch_tool: Some(true),
        include_view_image_tool: None,
        show_raw_agent_reasoning: oss.then_some(true),
        tools_web_search_request: None,
        deterministic_seed: seed,
    };
    // Parse `-c` overrides.
    let cli_kv_overrides = match config_overrides.parse_overrides() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing -c overrides: {e}");
            std::process::exit(1);
        }
    };

    let config = Config::load_with_cli_overrides(cli_kv_overrides, overrides)?;

    let otel = codex_core::otel_init::build_provider(&config, env!("CARGO_PKG_VERSION"));

    #[allow(clippy::print_stderr)]
    let otel = match otel {
        Ok(otel) => otel,
        Err(e) => {
            eprintln!("Could not create otel exporter: {e}");
            std::process::exit(1);
        }
    };

    if let Some(provider) = otel.as_ref() {
        let otel_layer = OpenTelemetryTracingBridge::new(&provider.logger).with_filter(
            tracing_subscriber::filter::filter_fn(codex_core::otel_init::codex_export_filter),
        );

        let _ = tracing_subscriber::registry()
            .with(fmt_layer)
            .with(otel_layer)
            .try_init();
    } else {
        let _ = tracing_subscriber::registry().with(fmt_layer).try_init();
    }

    let mut event_processor: Box<dyn EventProcessor> = match json_mode {
        true => Box::new(EventProcessorWithJsonOutput::new(last_message_file.clone())),
        _ => Box::new(EventProcessorWithHumanOutput::create_with_ansi(
            stdout_with_ansi,
            &config,
            last_message_file.clone(),
        )),
    };

    if oss {
        codex_ollama::ensure_oss_ready(&config)
            .await
            .map_err(|e| anyhow::anyhow!("OSS setup failed: {e}"))?;
    }

    let default_cwd = config.cwd.to_path_buf();
    let default_approval_policy = config.approval_policy;
    let default_sandbox_policy = config.sandbox_policy.clone();
    let default_model = config.model.clone();
    let default_effort = config.model_reasoning_effort;
    let default_summary = config.model_reasoning_summary;

    if !skip_git_repo_check && get_git_repo_root(&default_cwd).is_none() {
        eprintln!("Not inside a trusted directory and --skip-git-repo-check was not specified.");
        std::process::exit(1);
    }

    let auth_manager = AuthManager::shared(config.codex_home.clone(), true);
    let session_source = if force_cli_source {
        SessionSource::Cli
    } else {
        SessionSource::Exec
    };
    let conversation_manager = ConversationManager::new(auth_manager.clone(), session_source);

    // Handle resume subcommand by resolving a rollout path and using explicit resume API.
    let NewConversation {
        conversation_id: _,
        conversation,
        session_configured,
    } = if let Some(ExecCommand::Resume(args)) = command {
        let resume_path = resolve_resume_path(&config, &args).await?;

        if let Some(path) = resume_path {
            conversation_manager
                .resume_conversation_from_rollout(config.clone(), path, auth_manager.clone())
                .await?
        } else {
            conversation_manager
                .new_conversation(config.clone())
                .await?
        }
    } else {
        conversation_manager
            .new_conversation(config.clone())
            .await?
    };
    // Print the effective configuration and prompt so users can see what Codex
    // is using.
    event_processor.print_config_summary(&config, &prompt, &session_configured);

    info!("Codex initialized with event: {session_configured:?}");

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
    let start_ts = SystemTime::now();
    let monotonic_start = Instant::now();
    // Initialize run directory + event log early, so we persist even if the process aborts.
    let ts_ms_early = start_ts
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let run_id = format!("exec-{ts_ms_early}");
    let run_dir = summary_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(".codex").join("runs"));
    let _ = std::fs::create_dir_all(&run_dir);
    let events_path = run_dir.join(format!("{run_id}-events.ndjson"));
    let mut events_file = std::fs::File::create(&events_path).ok();
    let mut seq: u64 = 0;
    let timed_out = Arc::new(AtomicBool::new(false));
    let timed_out_at: Arc<tokio::sync::Mutex<Option<Instant>>> =
        Arc::new(tokio::sync::Mutex::new(None));
    let wrote_timeout_event = Arc::new(AtomicBool::new(false));
    let mut last_error_message: Option<String> = None;
    {
        let conv_for_timeout = conversation.clone();
        let timed_out_flag = timed_out.clone();
        let timed_out_at_ref = timed_out_at.clone();
        let run_timeout = run_timeout_secs;
        let shutdown_grace_ms = shutdown_grace_ms.unwrap_or(800);
        // Timer task to enforce global run timeout.
        tokio::spawn(async move {
            if let Some(secs) = run_timeout {
                let dur = Duration::from_secs(secs);
                tokio::time::sleep(dur).await;
                timed_out_flag.store(true, Ordering::SeqCst);
                {
                    let mut t = timed_out_at_ref.lock().await;
                    *t = Some(Instant::now());
                }
                tracing::warn!("Run timeout exceeded ({}s): sending interrupt", secs);
                let _ = conv_for_timeout.submit(Op::Interrupt).await;
                tokio::time::sleep(Duration::from_millis(shutdown_grace_ms)).await;
                let _ = conv_for_timeout.submit(Op::Shutdown).await;
            }
        });
        let conversation = conversation.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        tracing::debug!("Keyboard interrupt");
                        // Immediately notify Codex to abort any in‑flight task.
                        conversation.submit(Op::Interrupt).await.ok();

                        // Exit the inner loop and return to the main input prompt. The codex
                        // will emit a `TurnInterrupted` (Error) event which is drained later.
                        break;
                    }
                    res = conversation.next_event() => match res {
                        Ok(event) => {
                            debug!("Received event: {event:?}");

                            let is_shutdown_complete = matches!(event.msg, EventMsg::ShutdownComplete);
                            if let Err(e) = tx.send(event) {
                                error!("Error sending event: {e:?}");
                                break;
                            }
                            if is_shutdown_complete {
                                info!("Received shutdown event, exiting event loop.");
                                break;
                            }
                        },
                        Err(e) => {
                            error!("Error receiving event: {e:?}");
                            break;
                        }
                    }
                }
            }
        });
    }

    // Send images first, if any.
    if !images.is_empty() {
        let items: Vec<InputItem> = images
            .into_iter()
            .map(|path| InputItem::LocalImage { path })
            .collect();
        let initial_images_event_id = conversation.submit(Op::UserInput { items }).await?;
        info!("Sent images with event ID: {initial_images_event_id}");
        while let Ok(event) = conversation.next_event().await {
            if event.id == initial_images_event_id
                && matches!(
                    event.msg,
                    EventMsg::TaskComplete(TaskCompleteEvent {
                        last_agent_message: _,
                    })
                )
            {
                break;
            }
        }
    }

    // Send the prompt.
    let items: Vec<InputItem> = vec![InputItem::Text {
        text: prompt.clone(),
    }];
    // Emit a single completed context.summary line with real metrics (pre-stream).
    if let Some(f) = events_file.as_mut() {
        if context_summary_emitted().swap(true, Ordering::SeqCst) {
            // Already emitted in this process; skip.
        } else {
            use codex_core::config::ContextProviderKind;
            let provider_kind = &config.context_provider;
            // Build provider and compute metrics based on current prompt.
            let quotas = SectionQuotas {
                recent_pct: config.context_budget.0,
                plan_pct: config.context_budget.1,
                evidence_pct: config.context_budget.2,
                tools_pct: config.context_budget.3,
            };
            let input = TurnInput {
                user_text: prompt.clone(),
                recent_turns: vec![],
                plan_text: None,
                tool_deltas: vec![],
                max_context_tokens: config.context_max_tokens as usize,
                quotas,
            };
            let debug_ctx = std::env::var("CONTEXT_DEBUG").ok().as_deref() == Some("1");
            let mut metrics: ContextMetrics = ContextMetrics::default();
            match provider_kind {
                ContextProviderKind::Minimal => {
                    if let Ok((_b, m)) = MinimalContextProvider.build(&input).await {
                        metrics = m;
                    }
                }
                ContextProviderKind::Arango => {
                    let provider = ArangoContextProvider {
                        mcp_tool: config
                            .context_arango_mcp_tool
                            .clone()
                            .unwrap_or_else(|| "memory-agent".into()),
                        endpoint: config
                            .context_arango_endpoint
                            .clone()
                            .unwrap_or_else(|| "http://localhost:8529".into()),
                        database: config
                            .context_arango_database
                            .clone()
                            .unwrap_or_else(|| "codex".into()),
                        search_k: config.context_arango_search_k,
                        neighbors_depth: config.context_arango_neighbors_depth,
                        timeout_ms: config.context_arango_timeout_ms,
                        max_evidence_items: config.context_arango_max_evidence_items,
                        debug: debug_ctx,
                        allow_code: std::env::var("CONTEXT_EVIDENCE_ALLOW_CODE").ok().as_deref()
                            == Some("1"),
                        fixture_path: std::env::var("CONTEXT_MCP_FIXTURE").ok(),
                    };
                    if let Ok((_b, m)) = provider.build(&input).await {
                        metrics = m;
                    }
                }
            }
            let line = serde_json::json!({
                "ts_unix_ms": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),
                "elapsed_ms": monotonic_start.elapsed().as_millis() as u64,
                "kind": "context.summary",
                "version": 2,
                "provider": format!("{:?}", provider_kind),
                "max_context_tokens": config.context_max_tokens,
                "budget": {
                  "recent_pct": config.context_budget.0,
                  "plan_pct": config.context_budget.1,
                  "evidence_pct": config.context_budget.2,
                  "tools_pct": config.context_budget.3
                },
                "retrieval_ms": metrics.retrieval_ms,
                "evidence_items": metrics.evidence_items,
                "cache_hit": metrics.cache_hit,
                "retry_count": metrics.retry_count,
                "fallback_reason": metrics.fallback_reason,
                "search_k": config.context_arango_search_k,
                "neighbors_depth": config.context_arango_neighbors_depth,
                "reflowed_from": { "plan": metrics.reflowed_from_plan, "recent": metrics.reflowed_from_recent, "tools": metrics.reflowed_from_tools },
                "total_tokens": metrics.total_tokens,
                "section_tokens": { "evidence": metrics.evidence_tokens, "plan": metrics.plan_tokens, "recent": metrics.recent_tokens, "tools": metrics.tools_tokens },
                "truncated": { "evidence": metrics.truncated_evidence, "plan": metrics.truncated_plan, "recent": metrics.truncated_recent, "tools": metrics.truncated_tools }
            });
            if let Ok(s) = serde_json::to_string(&line) {
                let _ = std::io::Write::write_all(f, s.as_bytes());
                let _ = std::io::Write::write_all(f, b"\n");
            }
        }
    }
    let initial_prompt_task_id = conversation
        .submit(Op::UserTurn {
            items,
            cwd: default_cwd,
            approval_policy: default_approval_policy,
            sandbox_policy: default_sandbox_policy,
            model: default_model,
            effort: default_effort,
            summary: default_summary,
            final_output_json_schema: output_schema,
        })
        .await?;
    info!("Sent prompt with event ID: {initial_prompt_task_id}");

    // Run the loop until the task is complete or timeout grace expires.
    // Track whether a fatal error was reported by the server so we can
    // exit with a non-zero status for automation-friendly signaling.
    let mut error_seen = false;
    let mut tick = tokio::time::interval(Duration::from_millis(120));
    loop {
        tokio::select! {
            maybe_event = rx.recv() => {
                if let Some(event) = maybe_event {
        // Tee to NDJSON file best-effort.
        if let Some(f) = events_file.as_mut() {
            let line = serde_json::json!({
                "ts_unix_ms": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),
                "elapsed_ms": monotonic_start.elapsed().as_millis() as u64,
                "kind": "codex_event",
                "seq": seq,
                "run_id": run_id,
                "payload": &event,
            });
            if let Ok(s) = serde_json::to_string(&line) {
                let _ = std::io::Write::write_all(f, s.as_bytes());
                let _ = std::io::Write::write_all(f, b"\n");
            }
            seq = seq.saturating_add(1);
        }
        if let EventMsg::Error(err) = &event.msg {
            error_seen = true;
            last_error_message = Some(err.message.clone());
        }
        let shutdown: CodexStatus = event_processor.process_event(event);
        match shutdown {
            CodexStatus::Running => continue,
            CodexStatus::InitiateShutdown => {
                conversation.submit(Op::Shutdown).await?;
            }
            CodexStatus::Shutdown => {
                break;
            }
        }
                } else {
                    // Sender closed; break to finalize.
                    break;
                }
            },
            _ = tick.tick() => {
                if timed_out.load(Ordering::SeqCst) {
                    // Emit a synthetic timeout marker once for visibility in NDJSON.
                    if let Some(f) = events_file.as_mut()
                        && !wrote_timeout_event.load(Ordering::SeqCst) {
                            let line = serde_json::json!({
                                "ts_unix_ms": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),
                                "elapsed_ms": monotonic_start.elapsed().as_millis() as u64,
                                "kind": "run_timeout",
                                "seq": seq,
                                "run_id": run_id,
                            });
                            if let Ok(s) = serde_json::to_string(&line) {
                                let _ = std::io::Write::write_all(f, s.as_bytes());
                                let _ = std::io::Write::write_all(f, b"\n");
                            }
                            wrote_timeout_event.store(true, Ordering::SeqCst);
                            seq = seq.saturating_add(1);
                        }
                    // If shutdown grace elapsed, break even if backend hasn't acked.
                    let grace_ms = shutdown_grace_ms.unwrap_or(800);
                    let elapsed_ok = {
                        let g = timed_out_at.lock().await;
                        g.map(|inst| inst.elapsed() >= Duration::from_millis(grace_ms)).unwrap_or(false)
                    };
                    if elapsed_ok {
                        break;
                    }
                }
            }
        }
    }
    event_processor.print_final_output();

    // Persist a small run summary to disk to make headless use reliable.
    let run_dir = summary_dir.unwrap_or_else(|| PathBuf::from(".codex").join("runs"));
    let _ = std::fs::create_dir_all(&run_dir);
    let ts_ms = start_ts
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let summary_path = run_dir.join(format!("{run_id}-summary.json"));
    let elapsed_ms = monotonic_start.elapsed().as_millis() as u64;
    let status = if timed_out.load(Ordering::SeqCst) {
        "timeout"
    } else if error_seen {
        "error"
    } else {
        "ok"
    };
    let exit_code = if timed_out.load(Ordering::SeqCst) {
        5
    } else if error_seen {
        1
    } else {
        0
    };
    let summary = serde_json::json!({
        "schema_version": 1,
        "run_id": run_id,
        "status": status,
        "exit_code": exit_code,
        "timed_out": timed_out.load(Ordering::SeqCst),
        "duration_ms": elapsed_ms,
        "event_count": seq,
        "model": config.model,
        "model_provider": config.model_provider,
        "cwd": config.cwd,
        "started_unix_ms": ts_ms,
        "events_path": events_path,
        "last_error": last_error_message,
        "session_source": format!("{:?}", session_source),
        "seed": seed,
        "approval_forced_never": !keep_approval_policy,
    });
    if let Err(e) = std::fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).unwrap_or_default(),
    ) {
        eprintln!(
            "Warning: failed to write run summary to {}: {e}",
            summary_path.display()
        );
    }

    if timed_out.load(Ordering::SeqCst) {
        eprintln!(
            "codex-exec timeout: {}s budget exceeded. Summary: {}  Events: {}",
            run_timeout_secs.unwrap_or_default(),
            summary_path.display(),
            events_path.display()
        );
        if let Some(msg) = &last_error_message {
            eprintln!("last error: {msg}");
        }
        std::process::exit(5);
    } else if error_seen {
        if let Some(msg) = &last_error_message {
            let low = msg.to_ascii_lowercase();
            if msg.contains("429") || low.contains("rate") {
                eprintln!(
                    "hint: provider rate-limited the request; try fewer --jobs or increase backoff."
                );
            } else if low.contains("dns") || low.contains("resolve") {
                eprintln!("hint: network/DNS error; verify connectivity and provider base_url.");
            } else if low.contains("timeout") {
                eprintln!("hint: request timed out; adjust --request-timeout-ms and retries.");
            }
            eprintln!("last error: {msg}");
        }
        eprintln!(
            "codex-exec failed. Summary: {}  Events: {}",
            summary_path.display(),
            events_path.display()
        );
        std::process::exit(1);
    }

    Ok(())
}

async fn handle_slash_command(cmd: slash::SlashCommand) -> anyhow::Result<()> {
    match cmd {
        slash::SlashCommand::Help => {
            eprintln!(
                "Slash commands:\n  /help\n  /status\n  /model <id>\n  /provider <id>\n  /profile <name>\n  /discover [--min-params N --max-params N --max-output-ppm X --require-modalities A,B --require-capabilities X,Y]\n  /warmup [secs]\n  /fmt (Makefile/just fmt)\n  /build (Makefile build)\n  /test (Makefile deterministic tests)\n  /mcp-add <name> -- <cmd...>\n  /mcp-list\n"
            );
        }
        slash::SlashCommand::Light => {
            eprintln!("To use the light theme, run: export CXPLUS_THEME=light (then restart)");
        }
        slash::SlashCommand::Dark => {
            eprintln!("To use the dark theme, run: export CXPLUS_THEME=dark (then restart)");
        }
        slash::SlashCommand::Status => {
            // Print a minimal status (provider/model/profile); values may be set via config or flags at invocation time.
            let provider = std::env::var("CODEX_MODEL_PROVIDER").ok();
            let model = std::env::var("CODEX_MODEL").ok();
            let profile = std::env::var("CODEX_PROFILE").ok();
            eprintln!("provider={provider:?} model={model:?} profile={profile:?}");
        }
        slash::SlashCommand::Model { id } => {
            eprintln!("Use: -c model=\"{id}\" to apply for this run, or add to your profile.");
        }
        slash::SlashCommand::Provider { id } => {
            eprintln!(
                "Use: -c model_provider=\"{id}\" to apply for this run, or set in your config profile."
            );
        }
        slash::SlashCommand::Profile { name } => {
            eprintln!("Use: -p {name} to run with this profile.");
        }
        slash::SlashCommand::Discover(args) => {
            // Delegate to compiled binary subcommand: `codex chutes recommend` with flags; print chosen id.
            let exe =
                std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("cxplus"));
            let mut cmd = tokio::process::Command::new(exe);
            cmd.arg("chutes").arg("recommend");
            if let Some(v) = args.min_params {
                cmd.arg("--min-params").arg(v.to_string());
            }
            if let Some(v) = args.max_params {
                cmd.arg("--max-params").arg(v.to_string());
            }
            if let Some(v) = args.max_output_ppm {
                cmd.arg("--max-output-ppm").arg(v.to_string());
            }
            if let Some(v) = args.require_modalities {
                cmd.arg("--require-modalities").arg(v);
            }
            if let Some(v) = args.require_capabilities {
                cmd.arg("--require-capabilities").arg(v);
            }
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::inherit());
            match cmd.output().await {
                Ok(out) if out.status.success() => {
                    let id = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    eprintln!("discovered model: {id}");
                    eprintln!("Apply immediately with: -c model=\"{id}\"  (or add to a profile)");
                }
                Ok(out) => {
                    eprintln!("/discover failed (status={}):", out.status);
                }
                Err(e) => eprintln!("/discover error: {e}"),
            }
        }
        slash::SlashCommand::Unknown { raw } => {
            eprintln!("Unknown slash command: {raw}. Try /help");
        }
        slash::SlashCommand::Grep { pattern, path } => {
            let path = path.unwrap_or_else(|| ".".to_string());
            let rg = which::which("rg").ok();
            let out = if rg.is_some() {
                tokio::process::Command::new("rg")
                    .arg("-n")
                    .arg("--color=never")
                    .arg(&pattern)
                    .arg(&path)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::inherit())
                    .output()
                    .await
            } else {
                tokio::process::Command::new("grep")
                    .arg("-R")
                    .arg("-n")
                    .arg(&pattern)
                    .arg(&path)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::inherit())
                    .output()
                    .await
            };
            match out {
                Ok(out) => {
                    let s = String::from_utf8_lossy(&out.stdout);
                    let total = s.lines().count();
                    let max_lines = std::env::var("GREP_MAX_LINES")
                        .ok()
                        .and_then(|v| v.parse::<usize>().ok())
                        .filter(|n| *n >= 10)
                        .unwrap_or(200);
                    let lines: Vec<&str> = s.lines().take(max_lines).collect();
                    if total > lines.len() {
                        eprintln!(
                            "{}\n... (truncated; {} of {} lines)",
                            lines.join("\n"),
                            lines.len(),
                            total
                        );
                    } else {
                        eprintln!("{}", lines.join("\n"));
                    }
                }
                Err(e) => eprintln!("/grep error: {e}"),
            }
        }
        slash::SlashCommand::Open { path, line } => match std::fs::read(&path) {
            Ok(raw) => {
                let max_bytes = std::env::var("OPEN_MAX_KB")
                    .ok()
                    .and_then(|v| v.parse::<usize>().ok())
                    .filter(|kb| *kb >= 32)
                    .map(|kb| kb * 1024)
                    .unwrap_or(512 * 1024);
                if raw.len() > max_bytes {
                    eprintln!(
                        "/open refusing large file (>={}KB): {path}",
                        max_bytes / 1024
                    );
                    return Ok(());
                }
                let text = String::from_utf8_lossy(&raw);
                let lines: Vec<&str> = text.lines().collect();
                let (start, end) = if let Some(l) = line {
                    let idx = l.saturating_sub(1);
                    (idx.saturating_sub(3), std::cmp::min(idx + 3, lines.len()))
                } else {
                    (0, std::cmp::min(lines.len(), 200))
                };
                for (i, ln) in lines[start..end].iter().enumerate() {
                    eprintln!("{:>6} {}", start + i + 1, ln);
                }
            }
            Err(e) => eprintln!("/open error: {e}"),
        },
        slash::SlashCommand::Warmup { secs } => {
            // Delegate to compiled binary subcommand: `codex chutes warmup --secs N`.
            let exe =
                std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("cxplus"));
            let mut cmd = tokio::process::Command::new(exe);
            cmd.arg("chutes").arg("warmup");
            if let Some(s) = secs {
                cmd.arg("--secs").arg(s.to_string());
            }
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());
            match cmd.output().await {
                Ok(out) if out.status.success() => {}
                Ok(out) => eprintln!("/warmup failed (status={})", out.status),
                Err(e) => eprintln!("/warmup error: {e}"),
            }
        }
        slash::SlashCommand::Fmt => {
            slash_run_or_print("fmt", vec![]).await;
        }
        slash::SlashCommand::Build => {
            slash_run_or_print("build", vec![]).await;
        }
        slash::SlashCommand::Test => {
            slash_run_or_print("test", vec![]).await;
        }
    }
    Ok(())
}

async fn slash_run_or_print(target: &str, args: Vec<&str>) {
    let allow_write = std::env::var("ENABLE_SLASH_WRITE")
        .map(|v| v == "1")
        .unwrap_or(false);
    static ANNOUNCED: std::sync::Once = std::sync::Once::new();
    if allow_write {
        ANNOUNCED.call_once(|| {
            eprintln!("(write-enabled) slash commands may run build/test/fmt targets");
        });
    }
    if !allow_write {
        eprintln!("(dry-run) would run: make {target}");
        return;
    }
    let mut cmd = tokio::process::Command::new("make");
    cmd.arg(target);
    for a in args {
        cmd.arg(a);
    }
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit());
    match cmd.output().await {
        Ok(out) => {
            if !out.status.success() {
                eprintln!("make {} failed: {}", target, out.status);
            }
            let s = String::from_utf8_lossy(&out.stdout);
            eprintln!("{s}");
        }
        Err(e) => eprintln!("make {target} error: {e}"),
    }
}

async fn resolve_resume_path(
    config: &Config,
    args: &crate::cli::ResumeArgs,
) -> anyhow::Result<Option<PathBuf>> {
    if args.last {
        match codex_core::RolloutRecorder::list_conversations(&config.codex_home, 1, None, &[])
            .await
        {
            Ok(page) => Ok(page.items.first().map(|it| it.path.clone())),
            Err(e) => {
                error!("Error listing conversations: {e}");
                Ok(None)
            }
        }
    } else if let Some(id_str) = args.session_id.as_deref() {
        let path = find_conversation_path_by_id_str(&config.codex_home, id_str).await?;
        Ok(path)
    } else {
        Ok(None)
    }
}

fn load_output_schema(path: Option<PathBuf>) -> Option<Value> {
    let path = path?;

    let schema_str = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!(
                "Failed to read output schema file {}: {err}",
                path.display()
            );
            std::process::exit(1);
        }
    };

    match serde_json::from_str::<Value>(&schema_str) {
        Ok(value) => Some(value),
        Err(err) => {
            eprintln!(
                "Output schema file {} is not valid JSON: {err}",
                path.display()
            );
            std::process::exit(1);
        }
    }
}
