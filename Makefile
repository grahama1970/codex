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
	@env RUSTUP_TOOLCHAIN="$(RUSTUP_TOOLCHAIN)" cargo build --manifest-path $(RUST_MANIFEST) -p codex-cli --release
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

# -- Rapid deploy & versioning -------------------------------------------------

# Create a versioned binary under dist/releases and point dist/bin/codex at it.
# Version stamp: YYYYMMDDHHMM-<branch>-<shortsha>

BRANCH := $(shell git rev-parse --abbrev-ref HEAD | tr '/' '-' )
SHA := $(shell git rev-parse --short HEAD)
STAMP ?= $(shell date -u +%Y%m%d%H%M)-$(BRANCH)-$(SHA)
RELEASE_DIR := dist/releases

.PHONY: release deploy list-releases switch rollback

release: package
	@mkdir -p $(RELEASE_DIR)
	@cp -f $(BIN_DIR)/$(BIN_NAME) $(RELEASE_DIR)/$(BIN_NAME)-$(STAMP)
	@chmod +x $(RELEASE_DIR)/$(BIN_NAME)-$(STAMP)
	@ln -sfn ../releases/$(BIN_NAME)-$(STAMP) $(BIN_DIR)/$(BIN_NAME)
	# Provide a stable alias 'cxplus' that points at the canonical 'codex' symlink
	@ln -sfn $(BIN_NAME) $(BIN_DIR)/cxplus
	@echo "==> Deployed $(RELEASE_DIR)/$(BIN_NAME)-$(STAMP) and updated $(BIN_DIR)/$(BIN_NAME) (+ cxplus alias)"
	# Write manifest with the current release stamp
	@printf '{"stamp":"%s","binary":"%s"}\n' "$(STAMP)" "$(RELEASE_DIR)/$(BIN_NAME)-$(STAMP)" > $(DIST_DIR)/release.json
	# Create Windows-friendly wrappers
	@printf "@echo off\r\nsetlocal\r\nset DIR=%%~dp0\r\nset EXE=%%DIR%%codex.exe\r\nif exist \"%%EXE%%\" (\r\n  \"%%EXE%%\" %%*\r\n) else (\r\n  \"%%DIR%%codex\" %%*\r\n)\r\n" > $(BIN_DIR)/cxplus.cmd
	@{ printf '%s\n' \
	'$ErrorActionPreference = ''Stop''' \
	'$dir = Split-Path -Parent $MyInvocation.MyCommand.Path' \
	'$exe = Join-Path $dir ''codex.exe''' \
	"if (Test-Path $exe) { & $exe @args } else { & (Join-Path $dir 'codex') @args }" \
	; } > $(BIN_DIR)/cxplus.ps1

# Deploy with an explicit STAMP=... if desired
deploy: release

list-releases:
	@ls -lt $(RELEASE_DIR) 2>/dev/null || echo "(none)"

# Switch symlink to an existing version: make switch VERSION=<stamp>
switch:
	@if [ -z "$(VERSION)" ]; then echo "Usage: make switch VERSION=YYYYMMDDHHMM-branch-sha"; exit 1; fi
	@test -x $(RELEASE_DIR)/$(BIN_NAME)-$(VERSION) || (echo "Missing $(RELEASE_DIR)/$(BIN_NAME)-$(VERSION)" && exit 1)
	@ln -sfn ../releases/$(BIN_NAME)-$(VERSION) $(BIN_DIR)/$(BIN_NAME)
	@echo "==> Switched $(BIN_DIR)/$(BIN_NAME) -> $(BIN_NAME)-$(VERSION)"

# Roll back to the previous version by timestamp sort
rollback:
	@last=$$(ls -1t $(RELEASE_DIR)/$(BIN_NAME)-* 2>/dev/null | sed -n '2p'); \
	 if [ -z "$$last" ]; then echo "No previous release to roll back to"; exit 1; fi; \
	 ln -sfn ../releases/$$(basename $$last) $(BIN_DIR)/$(BIN_NAME); \
	 ln -sfn $(BIN_NAME) $(BIN_DIR)/cxplus; \
	 echo "==> Rolled back to $$(basename $$last) (+ refreshed cxplus alias)"

# Print current active binary and target release
.PHONY: current doctor
current:
	@echo "codex -> $$(readlink -f $(BIN_DIR)/$(BIN_NAME) 2>/dev/null || echo '(missing)')"
	@echo "cxplus -> $$(readlink -f $(BIN_DIR)/cxplus 2>/dev/null || echo '(missing)')"

# Basic PATH check for cxplus usability
doctor:
	@which cxplus >/dev/null 2>&1 || { echo "cxplus not on PATH; run 'make install-local' or add dist/bin to PATH"; exit 1; }
	@echo "cxplus on PATH: $$(command -v cxplus)"
	@echo "OK"

# Windows packaging (zip) – assumes dist/bin contains codex(.exe) and cxplus wrappers
.PHONY: package-windows
package-windows: release
	@cd $(BIN_DIR) && zip -9r ../cxplus-windows.zip cxplus.cmd cxplus.ps1 codex codex.exe 2>/dev/null || true
	@echo "==> Wrote $(DIST_DIR)/cxplus-windows.zip (includes cxplus wrappers and codex/codex.exe if present)"

# Install or uninstall a user-level symlink named 'cxplus' without touching any global 'codex' binary.
.PHONY: install-local uninstall-local
LOCAL_BIN ?= $(HOME)/.local/bin
install-local: package
	@mkdir -p $(LOCAL_BIN)
	@ln -sfn $(abspath $(BIN_DIR))/cxplus $(LOCAL_BIN)/cxplus
	@echo "==> Linked $(LOCAL_BIN)/cxplus -> $(abspath $(BIN_DIR))/cxplus"

uninstall-local:
	@rm -f $(LOCAL_BIN)/cxplus
	@echo "==> Removed $(LOCAL_BIN)/cxplus"

verify: test
	@if [ "$(RUN_LIVE)" = "1" ]; then \
	  $(MAKE) scenarios; \
	else \
	  echo "Skipping live scenarios (set RUN_LIVE=1 to enable)"; \
	fi

clean:
	@rm -rf $(DIST_DIR)
	@echo "Cleaned $(DIST_DIR)"
# -------- Documentation (auto-reference) --------------------------------------
.PHONY: docs-gen docs-drift

docs-gen: ensure-bin
	@echo "==> Generating CLI / config / events reference"
	@cargo run --manifest-path $(RUST_MANIFEST) -p codex-docs -- generate-cli docs/generated/cli
	@cargo run --manifest-path $(RUST_MANIFEST) -p codex-docs -- generate-config docs/generated/config
	@cargo run --manifest-path $(RUST_MANIFEST) -p codex-docs -- generate-events docs/generated/events
	@cargo run --manifest-path $(RUST_MANIFEST) -p codex-docs -- generate-slash docs/generated/slash
	@cargo run --manifest-path $(RUST_MANIFEST) -p codex-docs -- generate-index docs/generated

# Fail CI if generated references are stale
docs-drift: docs-gen
	@git diff --quiet -- docs/generated || (echo "Docs drift detected: run 'make docs-gen' and commit changes." && exit 1)

.PHONY: ensure-bin
ensure-bin:
	@if [ ! -x $(BIN_DIR)/$(BIN_NAME) ]; then \
	  echo "==> Compiled codex not found; building with stable toolchain"; \
	  RUSTUP_TOOLCHAIN=1.90.0 $(MAKE) package; \
	else \
	  echo "==> Using existing $(BIN_DIR)/$(BIN_NAME)"; \
	fi

.PHONY: docs-fix
docs-fix:
	@$(MAKE) docs-gen
	@git add docs/generated || true
	@echo "Docs regenerated and staged (if this is a git repo)."

# -------- mdBook site (Phase C) ---------------------------------------------
.PHONY: docs-book-clean docs-book-gen docs-book-build

docs-book-clean:
	@rm -rf docs/book/src/generated

docs-book-gen: docs-gen docs-book-clean
	@mkdir -p docs/book/src/generated
	@cp -R docs/generated/* docs/book/src/generated/
	@echo "==> Copied generated docs into mdBook src/"

docs-book-build: docs-book-gen
	@command -v mdbook >/dev/null 2>&1 || cargo install mdbook
	@mdbook build docs/book
