# Skills/Tooling Batch6 Decisions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 落地三个已由用户拍板的决策（#369 #339 #352），均为纯配置/脚本改动，无 Rust 编译代码。

**Architecture:** 3 个任务分别落在 3 组不相关文件（`.pre-commit-config.yaml`、`skills/_common.sh` + `scripts/_common.sh` + `scripts/install.sh`、`Makefile`），互不重叠，可合并成一个 PR 交付。

**Tech Stack:** Bash / GNU Make / YAML

**Spec:** `docs/superpowers/specs/2026-09-20-skills-tooling-batch6-design.md`

## Global Constraints

- `.pre-commit-config.yaml` 是 CLAUDE.md 保护文件，本次改动已获用户明确许可（拍板选"自动写回"）
- `_common.sh` 删除范围只限代码文件，`CHANGELOG.md`/历史报告文档里的提及不动
- `docs/` 同步目标是安装根的**上一级**（`$(dirname "$$D")`），不是 `$$D` 内部

---

### Task 1: cargo-fmt hook 去掉 `--check`（#369）

**Files:**
- Modify: `.pre-commit-config.yaml:19-26`（`cargo-fmt` hook）

**Interfaces:**
- 无跨任务接口

- [ ] **Step 1: 改 entry**

把：

```yaml
      - id: cargo-fmt
        # NOTE: requires `rustup toolchain install nightly`
        name: cargo fmt
        description: Format files with rustfmt.
        entry: bash -c 'cargo +nightly fmt -- --check'
        language: system
        files: \.rs$
        pass_filenames: false
```

替换为：

```yaml
      - id: cargo-fmt
        # NOTE: requires `rustup toolchain install nightly`
        name: cargo fmt
        description: Format files with rustfmt (writes back; pre-commit will
          report "files were modified by this hook" and block the commit if
          it did — re-`git add` and commit again, per Issue #369).
        entry: bash -c 'cargo +nightly fmt'
        language: system
        files: \.rs$
        pass_filenames: false
```

- [ ] **Step 2: 语法检查**

Run: `python3 -c "import yaml; yaml.safe_load(open('.pre-commit-config.yaml'))" && echo "YAML syntax OK"`
Expected: `YAML syntax OK`

- [ ] **Step 3: 真实场景验证——制造一处格式问题，走一次完整提交流程**

Run:
```bash
# 故意破坏格式（在已有 Rust 文件里插入一处多余空格，不改变逻辑）
sed -i '' 's/pub fn new()/pub  fn new()/' crates/core/src/lib.rs
git add crates/core/src/lib.rs
git commit -m "test: batch6 task1 probe (will be reset)" 2>&1 | tail -15
```
Expected: 提交被拦下，输出包含 `cargo fmt` 的 `Failed`/`files were modified by this hook`（或直接显示已被写回的 diff），且 `crates/core/src/lib.rs` 此时已被自动改回正确格式（`git diff` 应为空，因为 `cargo fmt` 已把多余空格删掉）

Run:
```bash
git diff crates/core/src/lib.rs
git reset crates/core/src/lib.rs
git checkout -- crates/core/src/lib.rs 2>/dev/null || true
git status --porcelain crates/core/src/lib.rs
```
Expected: 恢复探针改动前的干净状态，不留下测试痕迹

- [ ] **Step 4: 提交**

```bash
git add .pre-commit-config.yaml
git commit -m "fix(tooling): cargo-fmt pre-commit hook now writes back instead of only checking (#369)"
```

---

### Task 2: 删除 `_common.sh`（#339）

**Files:**
- Delete: `skills/_common.sh`
- Delete: `scripts/_common.sh`
- Modify: `scripts/install.sh:362-382`

**Interfaces:**
- 无跨任务接口

- [ ] **Step 1: 删除两个文件**

```bash
git rm skills/_common.sh scripts/_common.sh
```

- [ ] **Step 2: 清理 `install.sh` 的分发逻辑与旧注释**

把：

```bash
        # 跳过非 gf 前缀的目录（如 _common.sh 所在的父目录不会被遍历）
        local target_path="${SKILLS_TARGET_DIR}/${skill_name}"

        if [[ -d "$target_path" ]]; then
            warn "冲突: Skill '${skill_name}' 已存在于 ${target_path}，跳过"
            (( skipped++ )) || true
            continue
        fi

        # 复制整个 skill 目录
        cp -r "$skill_dir" "$target_path"
        (( installed++ )) || true
        info "安装 Skill: ${skill_name}"
    done

    # 复制 _common.sh 共享库
    if [[ -f "${skills_source}/_common.sh" ]]; then
        cp "${skills_source}/_common.sh" "${SKILLS_TARGET_DIR}/_common.sh"
        info "安装共享库: _common.sh"
    fi

    echo ""
```

替换为：

```bash
        local target_path="${SKILLS_TARGET_DIR}/${skill_name}"

        if [[ -d "$target_path" ]]; then
            warn "冲突: Skill '${skill_name}' 已存在于 ${target_path}，跳过"
            (( skipped++ )) || true
            continue
        fi

        # 复制整个 skill 目录
        cp -r "$skill_dir" "$target_path"
        (( installed++ )) || true
        info "安装 Skill: ${skill_name}"
    done

    echo ""
```

- [ ] **Step 3: 验证零残留引用**

Run: `grep -rn "_common.sh" --include="*.sh" --include="Makefile" .`
Expected: 无命中（历史 `docs/*.md` 文档不在这个 grep 范围内，不必零命中）

Run: `bash -n scripts/install.sh && echo "syntax OK"`
Expected: `syntax OK`

- [ ] **Step 4: 提交**

```bash
git add -A skills/_common.sh scripts/_common.sh scripts/install.sh
git commit -m "chore(skills): remove _common.sh — zero real callers, unwired shared library (#339)"
```

---

### Task 3: `install-skills` 同步 `docs/` 目录（#352）

**Files:**
- Modify: `Makefile:85-101`（`install-skills` target）

**Interfaces:**
- 无跨任务接口

- [ ] **Step 1: 改 target**

把：

```make
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
```

替换为：

```make
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
	PARENT=`dirname "$$D"`; \
	cp -r docs "$$PARENT"/; \
	echo "已同步 docs/ 到 $$PARENT/docs（供 ../../docs/... 相对链接在安装后解析，见 #352）"; \
	echo "Skills installed."
```

- [ ] **Step 2: 验证**

Run:
```bash
TMPDIR_PROBE=$(mktemp -d)
make install-skills SKILLS_DIR="$TMPDIR_PROBE/skills" 2>&1 | tail -5
test -d "$TMPDIR_PROBE/docs" && echo "docs 同步 OK"
test -f "$TMPDIR_PROBE/docs/references/pr-review-checklist.md" && echo "关键文件存在 OK"
rm -rf "$TMPDIR_PROBE"
```
Expected: 两行 `OK` 都打印

- [ ] **Step 3: 验证相对链接真的能从安装后的位置解析**

Run:
```bash
TMPDIR_PROBE=$(mktemp -d)
make install-skills SKILLS_DIR="$TMPDIR_PROBE/skills" >/dev/null 2>&1
python3 -c "
import os
skill_md = os.path.join('$TMPDIR_PROBE', 'skills', 'gf-pr-review', 'SKILL.md')
skill_dir = os.path.dirname(skill_md)
resolved = os.path.normpath(os.path.join(skill_dir, '../../docs/references/pr-review-checklist.md'))
print('resolved to:', resolved)
print('exists:', os.path.isfile(resolved))
"
rm -rf "$TMPDIR_PROBE"
```
Expected: `exists: True`

- [ ] **Step 4: 提交**

```bash
git add Makefile
git commit -m "fix(makefile): install-skills now syncs docs/ so ../../docs/... skill references resolve after install (#352)"
```

---

## Final Verification

- [ ] **Step 1: pre-commit 全量**

Run: `pre-commit run --all-files`
Expected: 全部 `Passed`（本次未改 Rust 代码，`cargo check` 应为 `Skipped`；`cargo-fmt` 这次本身也应该 `Passed`，因为它只格式化改过的 `.rs` 文件且当前工作区应无待格式化内容）

- [ ] **Step 2: 完整安装流程走一遍（非探针，确认不报错）**

Run: `bash -n scripts/install.sh && make -n install-skills`
Expected: 两条命令都不报错（`-n` 分别是 bash 语法检查和 make 的 dry-run）
