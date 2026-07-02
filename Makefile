.DEFAULT_GOAL := help

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-22s\033[0m %s\n", $$1, $$2}'

build: ## Compile the project
	@cargo build

check: ## Fast compile check (no codegen)
	@cargo check --workspace --all-targets --all-features

run: ## Build and run the CLI with --help
	@cargo run -- --help

test: ## Run tests with nextest
	@cargo nextest run --all-features

fmt: ## Check code formatting with nightly rustfmt
	@cargo +nightly fmt -- --check

clippy: ## Lint with pedantic clippy rules
	@cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic

lint: fmt clippy ## Run fmt and clippy

audit: ## Run security audit (deps + supply chain)
	@cargo deny check
	@cargo audit
	@cargo vet check 2>/dev/null || echo "cargo-vet not configured; run 'cargo vet init' to set up"

install-tools: ## Install development toolchain
	@pip install pre-commit 2>/dev/null || echo "Install pre-commit manually"
	@cargo install cargo-deny --locked 2>/dev/null || true
	@cargo install cargo-audit --locked 2>/dev/null || true
	@cargo install cargo-nextest --locked 2>/dev/null || true
	@cargo install cargo-vet --locked 2>/dev/null || true
	@cargo install typos-cli 2>/dev/null || true
	@cargo install cargo-release --locked 2>/dev/null || true
	@which gitleaks >/dev/null 2>&1 || echo "Install gitleaks: https://github.com/gitleaks/gitleaks#installing"
	@pre-commit install
	@echo "Run 'pre-commit run --all-files' to verify."

install-skills: ## Install skills to ~/.claude/skills/
	@echo "Installing skills to ~/.claude/skills/..."
	@mkdir -p ~/.claude/skills/
	@cp -r skills/* ~/.claude/skills/
	@echo "Skills installed."

install-hooks: ## Register hook config in .claude/settings.json
	@bash scripts/install.sh --no-build --no-skills

install: completions-install ## Full install: build binary + skills + hooks + completions (delegates to install.sh)
	@bash scripts/install.sh --no-build

list-skills: ## List installed gitflow skills
	@ls ~/.claude/skills/ 2>/dev/null | grep gitflow || echo "No gitflow skills installed."

uninstall-skills: ## Remove gitflow skills from ~/.claude/skills/
	@echo "Removing gitflow skills from ~/.claude/skills/..."
	@rm -rf ~/.claude/skills/gitflow-*
	@echo "Gitflow skills removed."

completions: build ## Generate shell completions (bash, zsh, fish) into ./completions/
	@bash scripts/generate-completions.sh

completions-install: build ## Install completion script for current shell (auto-detected from $SHELL)
	@cargo run -- completions --install

completions-uninstall: ## Uninstall completion script for current shell
	@cargo run -- completions --uninstall

watch: ## Watch for changes and check (requires cargo-watch)
	@cargo watch -x check

test-watch: ## Watch for changes and run tests (TDD mode)
	@cargo watch -x "nextest run --all-features"

bench: ## Run benchmarks
	@cargo bench --workspace

bench-cli: build ## Benchmark CLI binary with hyperfine
	@which hyperfine >/dev/null 2>&1 || { echo "Install hyperfine first"; exit 1; }
	@hyperfine --warmup 3 'cargo run -- --help'

coverage: ## Generate test coverage report
	@cargo llvm-cov --html --open

docs: ## Generate and open API documentation
	@cargo doc --no-deps --open

changelog: ## Generate CHANGELOG.md from conventional commits (requires git-cliff)
	@echo "Generating CHANGELOG.md..."
	@git cliff -o CHANGELOG.md
	@echo "CHANGELOG.md updated."

release-dry-run: ## Preview release without executing
	@cargo release --dry-run

update-submodule: ## Update git submodules recursively
	@git submodule update --init --recursive --remote

check-agent-sync: ## Verify CLAUDE.md exists
	@test -f CLAUDE.md || { \
		echo "CLAUDE.md is required for project-level agent instructions."; \
		exit 1; \
	}

smoke-test: ## Run multi-platform smoke test (auto-detect platform)
	@bash scripts/smoke-test.sh --read-only

smoke-test-github: ## Run smoke test for GitHub platform
	@bash scripts/smoke-test.sh --platform github --read-only

smoke-test-gitlab: ## Run smoke test for GitLab platform
	@bash scripts/smoke-test.sh --platform gitlab --read-only

smoke-test-gitcode: ## Run smoke test for GitCode platform
	@bash scripts/smoke-test.sh --platform gitcode --read-only

smoke-test-write: ## Run smoke test with write commands (help only)
	@bash scripts/smoke-test.sh --write

release: ## Tag and publish a release
	@cargo release tag --execute
	@git cliff -o CHANGELOG.md
	@git commit -a -n -m "chore: update CHANGELOG.md" || true
	@git push origin master
	@cargo release push --execute

.PHONY: help build check run test test-watch fmt clippy lint audit install-tools install-skills install-hooks install \
        list-skills uninstall-skills completions completions-install completions-uninstall \
        watch bench bench-cli coverage docs release-dry-run \
        update-submodule check-agent-sync release \
        smoke-test smoke-test-github smoke-test-gitlab smoke-test-gitcode smoke-test-write completions-install completions-uninstall changelog
