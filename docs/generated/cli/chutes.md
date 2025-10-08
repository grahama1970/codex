<!-- generated: commit ab66b462 -->
# Chutes Subcommand

_Auto-generated; do not edit manually._

```
Use Chutes: auto‑discover a model and/or run exec with it

Usage: codex chutes [OPTIONS] <COMMAND>

Commands:
  recommend  Print the best multi‑modal (>=2 modalities incl. text) Chutes model >= min‑params,
             cheapest output USD/1M
  exec       Run `exec` using the recommended Chutes model
  warmup     Perform a lightweight warm-up request against a Chutes model (max_tokens=1) and print a
             result line
  plan       Capacity planning helper: estimate throughput, instances, walltime, and cost for a
             Chutes deployment
  help       Print this message or the help of the given subcommand(s)

Options:
  -c, --config <key=value>
          Override a configuration value that would otherwise be loaded from `~/.codex/config.toml`.
          Use a dotted path (`foo.bar.baz`) to override nested values. The `value` portion is parsed
          as JSON. If it fails to parse as JSON, the raw string is used as a literal.
          
          Examples: - `-c model="o3"` - `-c 'sandbox_permissions=["disk-REDACTED-full-read-access"]'` - `-c
          shell_environment_policy.inherit=all`

  -h, --help
          Print help (see a summary with '-h')
```
