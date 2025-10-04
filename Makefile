.PHONY: build config package test scenarios verify clean rust-prepare

CODEX_RS_DIR := codex-rs
BIN_NAME := codex
DIST_DIR := dist
BIN_DIR := $(DIST_DIR)/bin
CONFIG_DIR := $(DIST_DIR)/config
CODEX_BIN := $(BIN_DIR)/$(BIN_NAME)

RUST_MANIFEST := $(CODEX_RS_DIR)/Cargo.toml

build:
	@echo "==> Building $(BIN_NAME) (release)"
	cargo build --manifest-path $(RUST_MANIFEST) -p codex-cli --release
	@mkdir -p $(BIN_DIR)
	@cp -f $(CODEX_RS_DIR)/target/release/$(BIN_NAME) $(CODEX_BIN)
	@echo "Built: $(CODEX_BIN)"

config: $(CONFIG_DIR)/config.toml

$(CONFIG_DIR)/config.toml:
	@echo "==> Creating minimal config at $(CONFIG_DIR)/config.toml"
	@set -a; [ -f .env ] && . ./.env; set +a; \
	mkdir -p $(CONFIG_DIR); \
	printf "model = \"o3\"\n" > $(CONFIG_DIR)/config.toml; \
	printf "approval_policy = \"never\"\n" >> $(CONFIG_DIR)/config.toml; \
	printf "sandbox_mode = \"read-only\"\n" >> $(CONFIG_DIR)/config.toml; \
	printf "\n[model_providers.chutes]\n" >> $(CONFIG_DIR)/config.toml; \
	printf "name = \"Chutes (OpenAI-compatible)\"\n" >> $(CONFIG_DIR)/config.toml; \
	printf "base_url = \"%s\"\n" "$${CHUTES_API_BASE:-https://llm.chutes.ai/v1}" >> $(CONFIG_DIR)/config.toml; \
	printf "env_key = \"CHUTES_API_KEY\"\n" >> $(CONFIG_DIR)/config.toml; \
	printf "wire_api = \"chat\"\n" >> $(CONFIG_DIR)/config.toml; \
	echo "Wrote: $(CONFIG_DIR)/config.toml"

package: build config
	@echo "==> Package ready under $(DIST_DIR)"

# Discover and write Chutes profiles (coding + multimodal) into config.toml.
# Requires CHUTES_API_KEY in .env; uses compiled binary for discovery.
.PHONY: chutes-profiles
chutes-profiles: build config
	@set -euo pipefail; set -a; [ -f .env ] && . ./.env; set +a; \
	if [ -z "$$CHUTES_API_KEY" ]; then echo "CHUTES_API_KEY missing in .env"; exit 1; fi; \
	CODING_MODEL="$$( $(CODEX_BIN) chutes recommend --min-params 10000000000 --max-params 80000000000 --max-output-ppm 3.0 --require-modalities text --require-capabilities coding,code 2>/dev/null || echo '' )"; \
	if [ -z "$$CODING_MODEL" ]; then \
	  CODING_MODEL="$$( $(CODEX_BIN) chutes recommend --min-params 10000000000 --max-params 120000000000 --max-output-ppm 3.5 --require-modalities text --require-capabilities coding,code 2>/dev/null || echo '' )"; \
	fi; \
	MM_MODEL="$$( $(CODEX_BIN) chutes recommend --min-params 10000000000 --max-params 120000000000 --max-output-ppm 4.0 --require-modalities text,image --require-capabilities coding,code 2>/dev/null || echo '' )"; \
	if [ -z "$$CODING_MODEL" ] || [ -z "$$MM_MODEL" ]; then \
	  echo "No suitable models found for one or both profiles"; exit 1; \
	fi; \
	echo "==> Writing Chutes profiles to $(CONFIG_DIR)/config.toml"; \
	{
	  echo ""; \
	  echo "model_provider = \"chutes\""; \
	  echo ""; \
	  echo "[profiles.coding]"; \
	  echo "model_provider = \"chutes\""; \
	  echo "model = \"$${CODING_MODEL}\""; \
	  echo ""; \
	  echo "[profiles.multimodal]"; \
	  echo "model_provider = \"chutes\""; \
	  echo "model = \"$${MM_MODEL}\""; \
	} >> $(CONFIG_DIR)/config.toml; \
	echo "Profiles written: coding=$${CODING_MODEL} multimodal=$${MM_MODEL}"

# Install common Rust workspace helpers if missing
rust-prepare:
	@command -v just >/dev/null 2>&1 || cargo install just
	@command -v rg >/dev/null 2>&1 || echo "ripgrep missing: install it via your package manager"
	@command -v cargo-insta >/dev/null 2>&1 || cargo install cargo-insta

test: package
	@echo "==> Running deterministic tests"
	@set -a; [ -f .env ] && . ./.env; set +a; CODEX_BIN=$(CODEX_BIN) CODEX_HOME=$(CONFIG_DIR) pytest -q tests

scenarios: package
	@echo "==> Running live scenarios (may require network/creds)"
	@set -a; [ -f .env ] && . ./.env; set +a; CODEX_BIN=$(CODEX_BIN) pytest -q -m live scenarios

verify: test
	@if [ "$(RUN_LIVE)" = "1" ]; then \
	  $(MAKE) scenarios; \
	else \
	  echo "Skipping live scenarios (set RUN_LIVE=1 to enable)"; \
	fi

clean:
	@rm -rf $(DIST_DIR)
	@echo "Cleaned $(DIST_DIR)"
