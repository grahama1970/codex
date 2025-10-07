<!-- generated: commit ab66b462 -->
# Exec CLI

_Auto-generated; do not edit manually._

```
Run Codex non-interactively

Usage: codex exec [OPTIONS] [PROMPT] [COMMAND]

Commands:
  resume  Resume a previous session by id or pick the most recent with --last
  help    Print this message or the help of the given subcommand(s)

Arguments:
  [PROMPT]
          Initial instructions for the agent. If not provided as an argument (or if `-` is used),
          instructions are read from stdin

Options:
  -c, --config <key=value>
          Override a configuration value that would otherwise be loaded from `~/.codex/config.toml`.
          Use a dotted path (`foo.bar.baz`) to override nested values. The `value` portion is parsed
          as JSON. If it fails to parse as JSON, the raw string is used as a literal.
          
          Examples: - `-c model="o3"` - `-c 'sandbox_permissions=["disk-REDACTED-full-read-access"]'` - `-c
          shell_environment_policy.inherit=all`

  -i, --image <FILE>...
          Optional image(s) to attach to the initial prompt

  -m, --model <MODEL>
          Model the agent should use

      --oss
          

  -s, --sandbox <SANDBOX_MODE>
          Select the sandbox policy to use when executing model-generated shell commands
          
          [possible values: read-only, workspace-write, danger-full-access]

  -p, --profile <CONFIG_PROFILE>
          Configuration profile from config.toml to specify default options

      --full-auto
          Convenience alias for low-friction sandboxed automatic execution (-a on-failure, --sandbox
          workspace-write)

      --dangerously-bypass-approvals-and-sandbox
          Skip all confirmation prompts and execute commands without sandboxing. EXTREMELY
          DANGEROUS. Intended solely for running in environments that are externally sandboxed

  -C, --cd <DIR>
          Tell the agent to use the specified directory as its working root

      --skip-git-repo-check
          Allow running Codex outside a Git repository

      --output-schema <FILE>
          Path to a JSON Schema file describing the model's final response shape

      --color <COLOR>
          Specifies color settings for use in the output
          
          [default: auto]
          [possible values: always, never, auto]

      --json
          Print events to stdout as JSONL

      --include-plan-tool
          Whether to include the plan tool in the conversation

  -o, --output-last-message <FILE>
          Specifies file where the last message from the agent should be written

      --run-timeout-secs <RUN_TIMEOUT_SECS>
          Optional global wall-clock timeout for the run in seconds. When exceeded, Codex will send
          an interrupt and shut down gracefully

      --summary-dir <DIR>
          Directory where a machine-readable run summary will be written. Defaults to `.codex/runs`
          under the current working directory

      --shutdown-grace-ms <SHUTDOWN_GRACE_MS>
          Grace period (ms) to wait after sending Interrupt before forcing Shutdown when a
          run-timeout occurs. Default: 800ms

      --prehook-enabled
          Enable the pre-execution hook (default: disabled)

      --prehook-backend <PREHOOK_BACKEND>
          Prehook backend to use: mcp | script | chained (default: mcp)
          
          [default: mcp]

      --prehook-on-error <PREHOOK_ON_ERROR>
          On error policy: fail | warn | skip (default: fail)
          
          [default: fail]

      --prehook-mcp-server <PREHOOK_MCP_SERVER>
          MCP server endpoint (e.g., stdio:/path/to/server [args...])

      --prehook-mcp-tool <PREHOOK_MCP_TOOL>
          MCP tool name to call (default: codex.prehook.review)

      --prehook-script <PREHOOK_SCRIPT_CMD>
          Script command to execute for script backend

      --prehook-timeout-ms <PREHOOK_TIMEOUT_MS>
          Prehook timeout (ms; default 5000ms)
          
          [default: 5000]

      --prehook-mcp-connect-timeout-ms <PREHOOK_MCP_CONNECT_TIMEOUT_MS>
          MCP connect timeout (ms). Overrides the default when set

      --prehook-mcp-call-timeout-ms <PREHOOK_MCP_CALL_TIMEOUT_MS>
          MCP call timeout (ms). Overrides the default when set

      --force-cli-source
          Treat this headless run as if it were an interactive (CLI) session upstream. Sets
          SessionSource::Cli instead of SessionSource::Exec (metrics / attribution parity)

      --keep-approval-policy
          Keep the configured approval policy instead of forcing AskForApproval::Never. Use when you
          want identical approval gating behavior to interactive sessions

      --seed <U64>
          Optional deterministic seed (foundation for reproducible sampling). Currently persisted to
          summary only; later will drive model sampling & internal RNG

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
