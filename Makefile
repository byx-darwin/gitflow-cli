.DEFAULT_GOAL := help

# 已安装 skill 的目标目录。可覆盖以便测试：make check-skills-drift SKILLS_DIR=/tmp/x
SKILLS_DIR ?= $(HOME)/.claude/skills

# 注入到需要判定 skill 归属的 recipe 中的 shell 辅助函数。
#
# is_project_skill <name>：当 skills/<name> 现存、或曾存在于 git 历史中时为真。
# 归属以 git 历史为准而非名称前缀 —— ~/.claude/skills 是混装目录，其中既有
# 以 gf- 开头却不属于本仓库的第三方 skill，也可能有不以 gf- 开头的本项目 skill。
# 浅克隆下历史可能不完整，此时判定为「非本项目」而跳过，失败方向是漏删而非误删。
SKILL_FNS = is_project_skill() { [ -d "skills/$$1" ] || [ -n "$$(git log --all --oneline -- "skills/$$1/*" 2>/dev/null | head -1)" ]; }; \
            is_retired_skill() { [ ! -d "skills/$$1" ] && is_project_skill "$$1"; };

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-22s\033[0m %s\n", $$1, $$2}'

build: ## Compile the project
	@cargo build
	@echo "✓ Debug build complete: target/debug/gf"

build-release: ## Compile the project (release mode, optimized)
	@cargo build --release
	@echo "✓ Release build complete: target/release/gf"

local-install: ## Install gf to ~/.cargo/bin (release build)
	@echo "Installing gf to ~/.cargo/bin..."
	@cargo install --path apps/cli --force --locked
	@echo "✓ Installed successfully"
	@gf --version

stage-skills-for-publish: ## Copy workspace skills/ into apps/cli/skills/ for crates.io packaging
	@echo "Staging skills for crates.io publish..."
	@mkdir -p apps/cli/skills
	@rsync -a --exclude='.git' --exclude='target' skills/ apps/cli/skills/
	@echo "✓ Staged $$(find apps/cli/skills -type f | wc -l | tr -d ' ') skill files into apps/cli/skills/"

clean-staged-skills: ## Remove staged skills from apps/cli/skills/
	@rm -rf apps/cli/skills
	@echo "✓ Cleaned staged skills"

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
	@cargo vet check

sbom: ## Generate CycloneDX SBOM for the gf binary (target/sbom/gf.cdx.json)
	@mkdir -p target/sbom
	@cargo cyclonedx --format json --describe binaries --spec-version 1.5
	@mv apps/cli/gf_bin.cdx.json target/sbom/gf.cdx.json
	@rm -f crates/release-signer/release-signer_bin.cdx.json
	@echo "SBOM written to target/sbom/gf.cdx.json"

install-tools: ## Install development toolchain
	@pip install pre-commit 2>/dev/null || echo "Install pre-commit manually"
	@cargo install cargo-deny --locked 2>/dev/null || true
	@cargo install cargo-audit --locked 2>/dev/null || true
	@cargo install cargo-nextest --locked 2>/dev/null || true
	@cargo install cargo-vet --locked 2>/dev/null || true
	@cargo install cargo-cyclonedx --locked 2>/dev/null || true
	@cargo install typos-cli 2>/dev/null || true
	@cargo install cargo-release --locked 2>/dev/null || true
	@which gitleaks >/dev/null 2>&1 || echo "Install gitleaks: https://github.com/gitleaks/gitleaks#installing"
	@pre-commit install
	@echo "Run 'pre-commit run --all-files' to verify."

install-skills: ## Install skills to ~/.claude/skills, pruning ones this project retired (override SKILLS_DIR)
	@$(SKILL_FNS) \
	D="$(SKILLS_DIR)"; \
	echo "Installing skills to $$D..."; \
	mkdir -p "$$D"; \
	for p in "$$D"/*; do \
		[ -e "$$p" ] || continue; \
		[ -L "$$p" ] && continue; \
		[ -d "$$p" ] || continue; \
		n=`basename "$$p"`; \
		if is_retired_skill "$$n"; then \
			echo "  - 移除已下线的 skill: $$n"; \
			rm -rf "$$p"; \
		fi; \
	done; \
	cp -r skills/* "$$D"/; \
	echo "Skills installed."

install-hooks: ## Register hook config in .claude/settings.json
	@bash scripts/install.sh --no-build --no-skills

install: completions-install ## Full install: build binary + skills + hooks + completions (delegates to install.sh)
	@bash scripts/install.sh --no-build

list-skills: ## List installed skills belonging to this project
	@$(SKILL_FNS) \
	D="$(SKILLS_DIR)"; FOUND=0; \
	for p in "$$D"/*; do \
		[ -e "$$p" ] || continue; \
		[ -L "$$p" ] && continue; \
		[ -d "$$p" ] || continue; \
		n=`basename "$$p"`; \
		if [ -d "skills/$$n" ]; then echo "  $$n"; FOUND=1; \
		elif is_project_skill "$$n"; then echo "  $$n  (已下线，make install-skills 会移除)"; FOUND=1; fi; \
	done; \
	[ $$FOUND -eq 1 ] || echo "No skills from this project installed."

uninstall-skills: ## Remove this project's skills from ~/.claude/skills (override SKILLS_DIR)
	@$(SKILL_FNS) \
	D="$(SKILLS_DIR)"; \
	echo "Removing this project's skills from $$D..."; \
	for p in "$$D"/*; do \
		[ -e "$$p" ] || continue; \
		[ -L "$$p" ] && continue; \
		[ -d "$$p" ] || continue; \
		n=`basename "$$p"`; \
		if is_project_skill "$$n"; then echo "  - $$n"; rm -rf "$$p"; fi; \
	done; \
	echo "Done."

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

check-agent-sync: ## Verify agent instructions exist and skill docs stay consistent
	@test -f CLAUDE.md || { \
		echo "✗ CLAUDE.md is required for project-level agent instructions."; \
		exit 1; \
	}
	@echo "✓ CLAUDE.md 存在"
	@bash scripts/verify-skills-when-not-to-use.sh
	@bash scripts/validate-skill-commands.sh
	@bash scripts/validate-skill-links.sh

check-skills-drift: ## Report drift between skills/ and ~/.claude/skills, read-only (override SKILLS_DIR)
	@$(SKILL_FNS) \
	D="$(SKILLS_DIR)"; DRIFT=0; \
	if [ ! -d "$$D" ]; then echo "✗ 安装目录不存在: $$D"; exit 1; fi; \
	for p in "$$D"/*; do \
		[ -e "$$p" ] || continue; \
		[ -L "$$p" ] && continue; \
		[ -d "$$p" ] || continue; \
		n=`basename "$$p"`; \
		if is_retired_skill "$$n"; then \
			echo "✗ 残留  $$n —— 本项目已下线，仍安装在 $$D"; DRIFT=1; \
		fi; \
	done; \
	for p in skills/*/; do \
		n=`basename "$$p"`; \
		if [ ! -d "$$D/$$n" ]; then echo "✗ 缺失  $$n —— 仓库中存在，尚未安装"; DRIFT=1; fi; \
	done; \
	if [ $$DRIFT -eq 0 ]; then echo "✓ 无漂移：$$D 与 skills/ 一致"; \
	else echo "运行 'make install-skills' 同步"; fi; \
	exit $$DRIFT

check-smell-skill: ## Verify gf-smell skill meets Issue #327 acceptance criteria
	@S=skills/gf-smell/SKILL.md; R=skills/gf-smell/references; FAIL=0; \
	if [ ! -f "$$S" ]; then echo "✗ missing $$S"; exit 1; fi; \
	if grep -nEi 'clippy|cargo|gocyclo|staticcheck|eslint|ruff|radon|pmd|checkstyle|Cargo\.toml|go\.mod|package\.json|pyproject\.toml|pom\.xml' "$$S"; then \
		echo "✗ AC#1 SKILL.md 正文含单一语言的工具名或文件名"; FAIL=1; \
	else echo "✓ AC#1 正文无语言专属标识"; fi; \
	MISS=0; \
	for L in rust go node python java; do \
		if [ ! -f "$$R/$$L.md" ]; then echo "✗ AC#2 缺少 $$R/$$L.md"; MISS=1; FAIL=1; fi; \
	done; \
	[ $$MISS -eq 0 ] && echo "✓ AC#2 references 五份齐备"; \
	for L in rust go node python java; do \
		[ -f "$$R/$$L.md" ] || continue; \
		for H in '## 检测命令' '## 阈值' '## 类目映射' '## 工具缺失降级'; do \
			grep -qF "$$H" "$$R/$$L.md" || { echo "✗ 契约 $$L.md 缺少 $$H"; FAIL=1; }; \
		done; \
	done; \
	grep -qF 'gf-quality/references/detector.md' "$$S" \
		&& echo "✓ AC#3 复用既有语言探测" \
		|| { echo "✗ AC#3 未引用 detector.md"; FAIL=1; }; \
	EVID=0; \
	for K in Measured Observed Inferred; do \
		grep -qF "$$K" "$$S" || { echo "✗ AC#5 缺少证据强度档位 $$K"; EVID=1; FAIL=1; }; \
	done; \
	grep -qF '证据强度' "$$S" || { echo "✗ AC#5"; EVID=1; FAIL=1; }; \
	[ $$EVID -eq 0 ] && echo "✓ AC#5 证据强度三档齐备"; \
	grep -qF 'Candidate requiring measurement' "$$S" \
		&& grep -qF '不计入严重度统计' "$$S" \
		&& echo "✓ AC#6 待测量候选独立成档" \
		|| { echo "✗ AC#6 待测量候选未独立或未声明不计入统计"; FAIL=1; }; \
	NOTFLAG=0; \
	grep -qF 'What NOT to Flag' "$$S" || { echo "✗ AC#7 缺少 What NOT to Flag 章节"; NOTFLAG=1; FAIL=1; }; \
	for E in 冷路径 有意权衡 已优化代码; do \
		grep -qF "$$E" "$$S" || { echo "✗ AC#7 排除项缺少 $$E"; NOTFLAG=1; FAIL=1; }; \
	done; \
	[ $$NOTFLAG -eq 0 ] && echo "✓ AC#7 排除章节齐备"; \
	grep -qF '根因合并' "$$S" && grep -qF '模式合并' "$$S" \
		&& echo "✓ AC#8 去重两条规则齐备" \
		|| { echo "✗ AC#8 缺少根因合并/模式合并规则"; FAIL=1; }; \
	A=`grep -m1 '^allowed-tools:' "$$S"`; \
	if [ -z "$$A" ]; then echo "✗ AC#9 缺少 allowed-tools"; FAIL=1; \
	elif echo "$$A" | grep -qE '(Write|Edit)'; then echo "✗ AC#9 allowed-tools 含 Write/Edit"; FAIL=1; \
	else echo "✓ AC#9 工具集不含 Write/Edit"; fi; \
	RPT=0; \
	REP=`ls docs/smell-report-*.md 2>/dev/null | head -1`; \
	if [ -z "$$REP" ]; then RPT=1; fi; \
	if [ -n "$$REP" ] && [ ! -s "$$REP" ]; then RPT=1; fi; \
	if [ $$RPT -eq 0 ]; then echo "✓ AC#4 smell 报告已落盘且非空: $${REP}"; \
	else echo "✗ AC#4 未找到非空的 smell-report-*.md"; FAIL=1; fi; \
	LOC=0; \
	STRAY=`find . -name 'smell-report-*.md' -not -path './docs/*' \
		-not -path './target/*' -not -path './.worktree/*' \
		-not -path './.claude/worktrees/*' 2>/dev/null | head -5`; \
	if [ -z "$$REP" ]; then LOC=1; fi; \
	case "$$REP" in docs/*) ;; *) LOC=1;; esac; \
	if [ -n "$$STRAY" ]; then echo "  散落在 docs/ 之外: $${STRAY}"; LOC=1; fi; \
	if [ $$LOC -eq 0 ]; then echo "✓ AC#10 报告落盘到 docs/"; \
	else echo "✗ AC#10 报告未落盘到 docs/"; FAIL=1; fi; \
	if [ -f "$$R/rust.md" ]; then \
		grep -qF -- '--force-warn' "$$R/rust.md" \
			&& echo "✓ Rust 层使用 --force-warn 穿透 allow" \
			|| { echo "✗ Rust 层未用 --force-warn"; FAIL=1; }; \
		grep -qF '单次运行' "$$R/rust.md" \
			&& echo "✓ Rust 层声明单次运行约束" \
			|| { echo "✗ Rust 层缺少单次运行约束"; FAIL=1; }; \
		grep -qF '只能标 Observed' "$$R/rust.md" \
			&& echo "✓ Rust 层声明结构扫描证据强度上限" \
			|| { echo "✗ Rust 层缺少结构扫描证据强度约束"; FAIL=1; }; \
	fi; \
	SUPP=0; \
	grep -qF 'Three outcomes:' "$$S" || SUPP=1; \
	grep -qF 'but the code contradicts it' "$$S" || SUPP=1; \
	grep -qF 'a finding in its own right' "$$S" || SUPP=1; \
	if grep -qF 'Bare suppression, no reason' "$$S"; then SUPP=1; fi; \
	if [ $$SUPP -eq 0 ]; then echo "✓ EXTRA#1 抑制判定为三分支（含理由被代码证伪）"; \
	else echo "✗ EXTRA#1 抑制判定未采用三分支形式"; FAIL=1; fi; \
	if [ -f "$$R/rust.md" ]; then \
		DEAD=0; \
		grep -qF 'OUT_DEAD' "$$R/rust.md" || DEAD=1; \
		grep -qF '不能用 `--all-targets`' "$$R/rust.md" || DEAD=1; \
		[ `grep -c '^cargo clippy ' "$$R/rust.md"` -eq 2 ] || DEAD=1; \
		[ `grep -c '^  --force-warn dead_code' "$$R/rust.md"` -eq 1 ] || DEAD=1; \
		if grep '^cargo clippy ' "$$R/rust.md" | grep -qF 'dead_code'; then DEAD=1; fi; \
		if [ $$DEAD -eq 0 ]; then echo "✓ EXTRA#2 rust.md 记录了 dead_code 与结构类 lint 的分离捕获流程（仅核查文档措辞，不保证实际未被合并）"; \
		else echo "✗ EXTRA#2 rust.md 未记录 dead_code 与结构类 lint 分离捕获流程的预期措辞"; FAIL=1; fi; \
	fi; \
	if [ $$FAIL -ne 0 ]; then echo "FAILED"; exit 1; fi; \
	echo "ALL CHECKS PASSED"

check-walkthrough-skill: ## Verify gf-walkthrough skill meets Issue #329 acceptance criteria
	@S=skills/gf-walkthrough/SKILL.md; \
	T=docs/superpowers/templates/walkthrough-report-template.md; FAIL=0; \
	if [ ! -f "$$S" ]; then echo "✗ missing $$S"; exit 1; fi; \
	if grep -nEi 'cargo|go\.mod|package\.json|pyproject\.toml|pom\.xml|pytest|eslint|mvn|clippy' "$$S"; then \
		echo "✗ #1 SKILL.md 正文含单一语言的工具名或文件名"; FAIL=1; \
	else echo "✓ #1 正文无语言专属标识"; fi; \
	TIER=0; \
	for K in Measured Inferred Unverified; do \
		grep -qF "$$K" "$$S" || { echo "✗ #2 缺少证据档位 $$K"; TIER=1; FAIL=1; }; \
	done; \
	[ $$TIER -eq 0 ] && echo "✓ #2 三档标记齐备"; \
	HEADER=0; \
	for H in '失败用例' '最后修改 commit' '是否 base 祖先'; do \
		grep -qF "$$H" "$$S" || { echo "✗ #4 失败测试表缺少列「$$H」"; HEADER=1; FAIL=1; }; \
	done; \
	[ $$HEADER -eq 0 ] && echo "✓ #4 失败测试表三列齐备"; \
	grep -qF 'unrelated' "$$S" \
		&& echo "✓ #5 禁用词规则已声明" \
		|| { echo "✗ #5 未声明禁止写 unrelated"; FAIL=1; }; \
	A=`grep -m1 '^allowed-tools:' "$$S"`; \
	if [ -z "$$A" ]; then echo "✗ #9 缺少 allowed-tools"; FAIL=1; \
	elif echo "$$A" | grep -qE 'Edit'; then echo "✗ #9 allowed-tools 含 Edit"; FAIL=1; \
	elif ! echo "$$A" | grep -qE 'Write'; then echo "✗ #9 allowed-tools 缺 Write（落盘所需）"; FAIL=1; \
	else echo "✓ #9 工具集含 Write 不含 Edit"; fi; \
	grep -qF 'gf-quality/references/detector.md' "$$S" \
		&& echo "✓ #10 复用既有语言探测" \
		|| { echo "✗ #10 未引用 detector.md"; FAIL=1; }; \
	W=`perl -0 -ne 's/^---\n.*?^---\n//ms; s/\x60\x60\x60.*?\x60\x60\x60//gs; s/\x60[^\x60]+\x60//g; print scalar(()=/\p{L}+/g)' "$$S"`; \
	if [ "$$W" -le 500 ]; then echo "✓ #11 词数 $$W ≤ 500"; \
	else echo "✗ #11 词数 $$W 超出 500 硬限"; FAIL=1; fi; \
	if [ -f "$$T" ]; then echo "✓ #12 报告模板存在"; \
	else echo "✗ #12 缺少 $$T"; FAIL=1; fi; \
	REP=`ls docs/walkthrough-*.md 2>/dev/null | head -1`; \
	if [ -n "$$REP" ] && [ -s "$$REP" ]; then echo "✓ #8 走查包已落盘: $$REP"; \
	else echo "✗ #8 未找到非空的 docs/walkthrough-*.md"; FAIL=1; REP=""; fi; \
	STRAY=`find . -name 'walkthrough-*.md' -not -path './docs/*' -not -path './target/*' \
		-not -path './.worktree/*' -not -path './.git/*' 2>/dev/null | head -5`; \
	if [ -n "$$STRAY" ]; then echo "✗ #8 报告散落在 docs/ 之外: $$STRAY"; FAIL=1; fi; \
	if [ -z "$$REP" ]; then \
		echo "— #3 跳过（无报告）"; echo "— #6 跳过（无报告）"; echo "— #7 跳过（无报告）"; \
	else \
		MCNT=`grep -c '^- \[Measured\]' "$$REP"`; \
		FCNT=`awk '/^- \[Measured\]/{f=0; for(i=1;i<=3;i++){if((getline line)>0){if(line ~ /^ *\x60\x60\x60/) f=1}}; if(!f) c++} END{print c+0}' "$$REP"`; \
		if [ "$$MCNT" -eq 0 ]; then echo "✗ #3 报告中无 [Measured] 条目"; FAIL=1; \
		elif [ "$$FCNT" -eq 0 ]; then echo "✓ #3 全部 $$MCNT 条 [Measured] 均附输出块"; \
		else echo "✗ #3 有 $$FCNT 条 [Measured] 未附输出块"; FAIL=1; fi; \
		OPEN=`awk '/^## /{if(seen){exit}; seen=1; next} seen && NF {print; exit}' "$$REP"`; \
		case "$$OPEN" in \
			'`'*|/*|\#*) echo "✗ #6 开场首句以标识符或路径开头: $$OPEN"; FAIL=1;; \
			*) echo "✓ #6 开场首句未以标识符或路径开头";; \
		esac; \
		grep -qiF 'blast radius' "$$REP" \
			&& echo "✓ #7 含 blast radius 说明" \
			|| { echo "✗ #7 报告缺少 blast radius 章节"; FAIL=1; }; \
	fi; \
	[ $$FAIL -eq 0 ] && echo "全部硬约束通过" || echo "存在未通过项"; \
	exit $$FAIL

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

release: ## Interactive release with safety checks and version preview
	@bash scripts/release.sh

release-quick: ## Quick release without interactive previews (for automation)
	@bash scripts/release.sh --quick

release-rehearse: ## Full dry-run release drill (mandatory checklist, no changes)
	@bash scripts/release.sh --rehearse

release-from-dev: ## Release from dev branch (auto-switches to main, merges, releases)
	@bash scripts/release-from-dev.sh

release-push: ## Legacy: Step 1: bump version, commit, generate changelog, tag, push
ifndef VERSION
	$(error Usage: make release-push VERSION=patch|minor|major)
endif
	@cargo release version $(VERSION) --execute --workspace --no-confirm
	@cargo release commit --execute --no-confirm
	@git cliff -o CHANGELOG.md
	@git commit -a -n -m "chore: update CHANGELOG.md" || true
	@cargo release tag --execute --workspace --no-confirm
	@git push origin main --tags
	@echo ""
	@echo "==> Step 1 complete: tag pushed, waiting for CI..."
	@echo "==> gh run list --limit 1"
	@echo "==> Then: make release-publish"

release-publish: ## Placeholder: add crates.io when ready
	@echo "Not published to crates.io yet."
	@echo "GitHub Release with multi-platform artifacts created automatically on tag push."
	@echo "Release URL: https://github.com/byx-darwin/gitflow-cli/releases"

package: ## Build and package current platform binary into dist/
	@mkdir -p dist
	@cargo build --release
	@TARGET=$$(rustc -vV | sed -n 's|host: ||p'); \
	BIN=target/release/gf; \
	if [ "$$(uname)" = "Darwin" ] || [ "$$(uname)" = "Linux" ]; then \
		tar -czvf dist/gf-$${TARGET}.tar.gz -C target/release gf; \
	else \
		echo "Windows: use 7z to package target\\release\\gf.exe"; \
	fi
	@echo "Packaged to dist/"

.PHONY: help build build-release local-install check run test test-watch fmt clippy lint audit sbom install-tools install-skills install-hooks install \
        list-skills uninstall-skills completions completions-install completions-uninstall \
        watch bench bench-cli coverage docs release-dry-run \
        update-submodule check-agent-sync check-smell-skill check-walkthrough-skill check-skills-drift release release-quick release-rehearse \
        smoke-test smoke-test-github smoke-test-gitlab smoke-test-gitcode smoke-test-write completions-install completions-uninstall changelog release-push release-publish package

.PHONY: compatibility-matrix
compatibility-matrix: ## 从 JSON 生成兼容性矩阵 Markdown
	cargo run -p gitflow-core --example gen_compat_matrix
