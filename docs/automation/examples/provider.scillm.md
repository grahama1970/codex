# scillm as primary provider (LiteLLM gateway)

Use scillm’s OpenAI‑compatible gateway as Codex’s model endpoint. This keeps Codex unchanged and lets scillm handle routing/caching/costs.

1) Run your scillm/LiteLLM gateway (see litellm/QUICKSTART.md).
2) Export the API key:

```bash
export SCILLM_API_KEY="sk-..."
```

3) Point Codex (cx‑plus) at scillm via a provider override in `~/.cx-plus/config.toml`:

```toml
[model_provider]
base_url = "http://127.0.0.1:4000/v1"   # scillm (LiteLLM) endpoint
api_key_env = "SCILLM_API_KEY"           # env var holding your key
# Optional default routed model name
# default_model = "gpt-4o-mini"
```

Notes
- No code changes needed; Codex will use the configured base_url and key.
- Keep Memory Agent as the prehook brain; scillm handles model inference.
- If scillm is down, change base_url back to your direct provider.
