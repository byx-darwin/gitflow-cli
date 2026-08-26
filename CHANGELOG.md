# Changelog

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

---
## [1.6.0](https://github.com/byx-darwin/gitflow-cli/compare/v1.5.0..v1.6.0) - 2026-08-26

### ✨ 新特性

* **cli** — 新增 `WorkflowMode::Standard` 变体，`gf workflow create --mode standard` 现已可用 - ([92cc0ce](https://github.com/byx-darwin/gitflow-cli/commit/92cc0ce))
* **release** — 支持 crates.io 与 Homebrew 自动发布 - ([7ddd135](https://github.com/byx-darwin/gitflow-cli/commit/7ddd135))
* **release** — 发布脚本适配 PR 工作流 - ([3b5da59](https://github.com/byx-darwin/gitflow-cli/commit/3b5da59))
* 新增 release-from-dev 自动化脚本 - ([22c2f59](https://github.com/byx-darwin/gitflow-cli/commit/22c2f59))

### 🐛 Bug 修复

* **gitlab** — 修复嵌套 group 项目路径 `%2F` 编码不全导致 issue 评论 404 (#221) - ([44e1cdf](https://github.com/byx-darwin/gitflow-cli/commit/44e1cdf))
* **gitlab** — review approve 去掉 `--repo` 参数，修复嵌套 group 项目失败 (#231) - ([dff3c33](https://github.com/byx-darwin/gitflow-cli/commit/dff3c33))
* **release** — 修复 gf update 签名校验失败（消除重复发布、统一签名资产）(#226) - ([d3d0a87](https://github.com/byx-darwin/gitflow-cli/commit/d3d0a87))
* **github** — 修复 gf label 编辑假失败（gh 2.97 移除 label view 子命令）(#203) - ([60c332a](https://github.com/byx-darwin/gitflow-cli/commit/60c332a))
* **bug-report** — 修复主动上报 bug 功能 P0/P1/P2 遗留问题（认证统一、触发确定性、错误分类等）(#212, #214, #218)
* **bug-report** — 上报路径改回 gh（边界修正：项目托管在 GitHub）(#216) - ([4a60cda](https://github.com/byx-darwin/gitflow-cli/commit/4a60cda))

### 📝 文档

* 补齐安装前置条件，skills install 阻断提示内联 Node 版本说明 (#220)
* 多角色评估报告 — 主动上报 bug 功能是否 OK (#210)

### 🔧 维护

* **ci** — 恢复 update-homebrew job + e2e paths 治本 (#225)
* **chore** — 升级 glab tested_version 至 1.114.0 (#230)

### ♻️ 重构

* **core** — 兼容性矩阵 gf 版本改为从 `CARGO_PKG_VERSION` 派生 (#208)

---
## [unreleased]

---
## [1.4.0](https://github.com/byx-darwin/gitflow-cli/compare/v1.3.0..v1.4.0) - 2026-08-18

### Bug Fixes

- **(gitlab)** compat with glab 1.113 for GitLab write operations (#201) - ([980d5c5](https://github.com/byx-darwin/gitflow-cli/commit/980d5c5180cd96a2587910b82ee9496729359cf5)) - mc-ai

### Documentation

- archive #199 glab 1.113 compat workflow artifacts (#202) - ([94a5d5a](https://github.com/byx-darwin/gitflow-cli/commit/94a5d5aec3979e55f8f6869f7a917c898f03912a)) - mc-ai
- archive #200 gh 2.97 label edit workflow artifacts - ([38f9d95](https://github.com/byx-darwin/gitflow-cli/commit/38f9d95acfeb0ba2fd682931e3801b0a2b5b229f)) - baoyx

### Features

- **(release)** update release script for PR-based workflow - ([3b5da59](https://github.com/byx-darwin/gitflow-cli/commit/3b5da59b16d5f532da32aeb645635be676179571)) - baoyx
- **(release)** add crates.io and Homebrew publishing - ([7ddd135](https://github.com/byx-darwin/gitflow-cli/commit/7ddd135152d84d0ddc158f3fc40aa4f13592ecae)) - baoyx
- **(release)** add crates.io and Homebrew publishing (#195) - ([324c7f8](https://github.com/byx-darwin/gitflow-cli/commit/324c7f81ab7a586013e08ecd15a5fe375eb77c65)) - mc-ai

### Miscellaneous Chores

- remove HomebrewFormula directory (#197) - ([858ba10](https://github.com/byx-darwin/gitflow-cli/commit/858ba101f1d214c7e72fa2c3ac2883258b10ec51)) - mc-ai

### Other

- Merge branch 'main' into dev - ([f0b8b12](https://github.com/byx-darwin/gitflow-cli/commit/f0b8b122b2c9f3dfb588a8ac17a7ee3bc1e13913)) - baoyx
- Merge remote-tracking branch 'origin/main' into dev - ([7917b45](https://github.com/byx-darwin/gitflow-cli/commit/7917b45aeaec162424f2149aeae91d87bfbd3178)) - baoyx
- Merge remote-tracking branch 'origin/main' into dev - ([fc1b9e3](https://github.com/byx-darwin/gitflow-cli/commit/fc1b9e39e058a5b6b9fccdebbe2f6d8248f35135)) - baoyx
- Merge remote-tracking branch 'origin/main' into dev - ([de642d4](https://github.com/byx-darwin/gitflow-cli/commit/de642d48c6301bed9eb47a51204ef0f54e3a72f4)) - baoyx

---
## [1.3.0](https://github.com/byx-darwin/gitflow-cli/compare/v1.2.0..v1.3.0) - 2026-08-11

### Bug Fixes

- **(release)** use --registry crates-io for publishing - ([a8c170a](https://github.com/byx-darwin/gitflow-cli/commit/a8c170a69d5d5a368c42774aef8aa12e52549236)) - baoyx
- **(release)** use cargo release publish for workspace dependency order - ([3225bd1](https://github.com/byx-darwin/gitflow-cli/commit/3225bd14e50605c93defc2b9f08aac2eead0ffa1)) - baoyx
- **(release)** add --allow-dirty for crates.io publish - ([53bbfa9](https://github.com/byx-darwin/gitflow-cli/commit/53bbfa9bb60cc5d7dc92718331bf4f0c03bb937e)) - baoyx
- **(update)** use zipsign format for release signatures - ([1b6e7c0](https://github.com/byx-darwin/gitflow-cli/commit/1b6e7c00a9ddbf15a3fa9f6c44efc2661687d568)) - baoyx
- **(workflow)** symlink .claude directory for worktree compatibility - ([221cf73](https://github.com/byx-darwin/gitflow-cli/commit/221cf73dbe010121a6a230be606630c1210e444b)) - baoyx
- **(workflow)** remove .claude symlink and use relative paths - ([e1b88aa](https://github.com/byx-darwin/gitflow-cli/commit/e1b88aa9b5d98b740221086583e257308e46e5bc)) - baoyx

### Features

- **(doctor)** add gf doctor command + error audit + issue ops (#186) - ([d1410ac](https://github.com/byx-darwin/gitflow-cli/commit/d1410ac41a1311a804cd2120c66d5083d96c91ce)) - mc-ai
- **(pr)** add gf pr diff and patch commands (#191) - ([0591faa](https://github.com/byx-darwin/gitflow-cli/commit/0591faafbe6842f583d98a38a00d1b29ebcc4ff7)) - mc-ai
- **(website)** SEO optimization and content matrix guides (#189) - ([24767ba](https://github.com/byx-darwin/gitflow-cli/commit/24767babf7ba2f7c24ad171dbc41e8ffff827a54)) - mc-ai
- add release-from-dev automation script - ([22c2f59](https://github.com/byx-darwin/gitflow-cli/commit/22c2f597aee1dcf753d3cc56354e14e582e0ec7e)) - baoyx

### Miscellaneous Chores

- **(release)** add format check to preflight - ([c61b877](https://github.com/byx-darwin/gitflow-cli/commit/c61b8776e8d412009a6ee4ca77bace99cbf5e523)) - baoyx
- update Homebrew formula to v1.2.0 - ([304d5b0](https://github.com/byx-darwin/gitflow-cli/commit/304d5b040a25b29f575a918efb3144feeb628550)) - github-actions[bot]
- release v1.3.0 - ([a21a401](https://github.com/byx-darwin/gitflow-cli/commit/a21a4014b8af995e65cac2f459801cf942301213)) - baoyx
- fix release-from-dev script (#193) - ([04002e6](https://github.com/byx-darwin/gitflow-cli/commit/04002e6b6dbcc9624d9b5ab9826e2ce9a6bc7055)) - mc-ai
- release v1.3.0 (#194) - ([41e8dfd](https://github.com/byx-darwin/gitflow-cli/commit/41e8dfd36cb6638c92553b591b4c9aea153724dd)) - mc-ai

### Other

- Merge pull request #184 from byx-darwin/fix/update-signature-verification

fix(update): use zipsign format for release signatures - ([9973db9](https://github.com/byx-darwin/gitflow-cli/commit/9973db98fa9a8f0eb59ee576367067cb167b2bb7)) - mc-ai
- Merge pull request #185 from byx-darwin/fix/worktree-symlink-compatibility

fix(workflow): symlink .claude directory for worktree compatibility - ([a42213f](https://github.com/byx-darwin/gitflow-cli/commit/a42213f44d8594f0414b5b64f8e35ab9cdaa5c69)) - mc-ai
- trigger E2E tests - ([1acd611](https://github.com/byx-darwin/gitflow-cli/commit/1acd611980f22b0cc27206f8a53fe215f3880e39)) - baoyx
- Merge pull request #187 from byx-darwin/fix/worktree-symlink-broken

fix(workflow): remove .claude symlink and use relative paths - ([66ea209](https://github.com/byx-darwin/gitflow-cli/commit/66ea209572845f5b98f58a09f0a7c3d70ba4a057)) - mc-ai
- update workflows to support dev branch - ([ea4be64](https://github.com/byx-darwin/gitflow-cli/commit/ea4be6497e3119b59e4fa86af9ca83b57be602ce)) - baoyx
- Merge branch 'main' into dev - ([a4f3e58](https://github.com/byx-darwin/gitflow-cli/commit/a4f3e58cf7a9de33c8160e7a755af9686e565e78)) - baoyx

### Style

- **(auth)** fix nightly rustfmt violation and docs typos - ([6ea3960](https://github.com/byx-darwin/gitflow-cli/commit/6ea3960fb59af6a012e08734380c1fb0c3e0dcfc)) - baoyx

---
## [1.2.0](https://github.com/byx-darwin/gitflow-cli/compare/v1.1.0..v1.2.0) - 2026-08-10

### Bug Fixes

- **(auth)** parse token scopes from gh auth status output - ([8c3db1a](https://github.com/byx-darwin/gitflow-cli/commit/8c3db1a95dbf5f75afbe09f114afe134a4008331)) - baoyx
- **(examples)** remove main() from Go example to meet coverage threshold - ([715f54a](https://github.com/byx-darwin/gitflow-cli/commit/715f54abc1e871fa6d0dc547597710562333ab38)) - baoyx
- **(gitcode)** use 'gc' instead of 'gitcode' in error hints - ([6382696](https://github.com/byx-darwin/gitflow-cli/commit/6382696cea477a332096bae0c9863754a575c28f)) - baoyx
- **(website)** update version display to 1.1.0 on homepage - ([f434383](https://github.com/byx-darwin/gitflow-cli/commit/f434383ce10e5f022d42098633538059d69e699d)) - baoyx
- include skills in crates.io package for embedded binary (#165) - ([6539d70](https://github.com/byx-darwin/gitflow-cli/commit/6539d705af2bdd89d04ccfa5306e62433e20a63f)) - mc-ai

### Documentation

- **(gf-quality)** add Configuration and Troubleshooting to Rust reference - ([93db94d](https://github.com/byx-darwin/gitflow-cli/commit/93db94d76a551968e2a9674e099f702f7b1d61c8)) - baoyx
- **(gf-quality)** add Configuration and Troubleshooting to Go reference - ([75be309](https://github.com/byx-darwin/gitflow-cli/commit/75be309f2c829386b77ba77870d27cedcf1a8dfd)) - baoyx
- **(gf-quality)** add Configuration and Troubleshooting to Node.js reference - ([6a90a6b](https://github.com/byx-darwin/gitflow-cli/commit/6a90a6b94014c819a90fd7f07072e56beb8e351e)) - baoyx
- **(gf-quality)** add Configuration and Troubleshooting to Python reference - ([9ee8df6](https://github.com/byx-darwin/gitflow-cli/commit/9ee8df609e7747077ffa3c59bb1ce9d1d09c1d12)) - baoyx
- **(gf-quality)** add Configuration and Troubleshooting to Java reference - ([79036c2](https://github.com/byx-darwin/gitflow-cli/commit/79036c230904b425ffe861ac9155ef30b733fedc)) - baoyx
- **(gf-quality)** enhance detector with workspace detection and project tree - ([0a83578](https://github.com/byx-darwin/gitflow-cli/commit/0a8357855cfe11832787c3aba8abd0b2d4f8a404)) - baoyx
- **(gf-quality)** enhance SKILL.md with workspace-aware execution - ([a76e7a2](https://github.com/byx-darwin/gitflow-cli/commit/a76e7a290ae6825cf6c568135716a9e5fe3c5dbc)) - baoyx
- **(gf-quality)** add dogfooding validation reports - ([0a3047e](https://github.com/byx-darwin/gitflow-cli/commit/0a3047eb391e811e02a13d9ef16fd6bfe5d961f9)) - baoyx
- **(gf-quality)** sync documentation with enhanced skill - ([8fef988](https://github.com/byx-darwin/gitflow-cli/commit/8fef9889b939c353f3a2c70571830b04fd11d008)) - baoyx
- **(plans)** add gf-quality multi-language enhancement implementation plan - ([0a838fa](https://github.com/byx-darwin/gitflow-cli/commit/0a838fa0f624908fde553a2651d41c3168974144)) - baoyx
- **(skill)** rewrite When to Use and add When NOT to Use - ([50a66ea](https://github.com/byx-darwin/gitflow-cli/commit/50a66ea1c4db1095bb63bceebb6f481a267ce20e)) - baoyx
- **(skill)** replace trigger keywords with three-tier system - ([204b575](https://github.com/byx-darwin/gitflow-cli/commit/204b575e9977b4817bdc3765e3a2c1af78bb6ed9)) - baoyx
- **(skill)** add usage examples and quick start guide - ([d4bda41](https://github.com/byx-darwin/gitflow-cli/commit/d4bda41d5d22a84e5474b9597928b33d2aedc375)) - baoyx
- **(skills)** add When NOT to Use sections to all 25 skills - ([7d53954](https://github.com/byx-darwin/gitflow-cli/commit/7d5395450aa242b073c0f9d4c3d0e0febfbecf2a)) - baoyx
- **(skills)** add CLI requirement section to enforce gf usage - ([fcfca80](https://github.com/byx-darwin/gitflow-cli/commit/fcfca80d63f4f8924ba08fb40a03ee6cca9354d0)) - baoyx
- **(skills)** fix CLI command references to match actual implementation - ([7b14456](https://github.com/byx-darwin/gitflow-cli/commit/7b1445639fa642f09ff9a27433a4f712abef894a)) - baoyx
- **(specs)** add gf-quality multi-language enhancement design - ([55cb548](https://github.com/byx-darwin/gitflow-cli/commit/55cb548b574b222dbb1754b5850e5265e2125ff6)) - baoyx
- add skills multi-role review and roadmap design - ([4fc379d](https://github.com/byx-darwin/gitflow-cli/commit/4fc379d318d78eb75b7d5d7cc153eeb7a5112f5a)) - baoyx
- add comprehensive skills multi-role review report - ([0d4c081](https://github.com/byx-darwin/gitflow-cli/commit/0d4c0818f2ae5807dce1416870ec3af7b9b17d11)) - baoyx
- add gf-regression skill redesign spec - ([d8b9d66](https://github.com/byx-darwin/gitflow-cli/commit/d8b9d6644256635eed0c2410e407b089a260d80a)) - baoyx
- add gf-regression skill redesign implementation plan - ([fa6dd1d](https://github.com/byx-darwin/gitflow-cli/commit/fa6dd1d951c271da0a9d3c0e2e58d456d14d888b)) - baoyx

### Features

- **(pr)** add cleanup command for post-merge branch and worktree management (#178) - ([4595f75](https://github.com/byx-darwin/gitflow-cli/commit/4595f759b4e65bef77cb4abb8f73b03e66408005)) - mc-ai
- **(website)** add navigation links to new GEO pages - ([6121946](https://github.com/byx-darwin/gitflow-cli/commit/612194640dce3e0991dc7e639071a2c9d8bbb57c)) - baoyx
- **(website)** add static pages for architecture and release-workflow - ([5a41bc9](https://github.com/byx-darwin/gitflow-cli/commit/5a41bc97e9d80c5a260e402f50977dd58b5d1edb)) - baoyx

### Miscellaneous Chores

- update Homebrew formula to v1.1.0 - ([82809d8](https://github.com/byx-darwin/gitflow-cli/commit/82809d8f41f9be35a295a2b53e1917a9376eb532)) - github-actions[bot]
- remove gf-weekly-report skill - ([b0a6b4a](https://github.com/byx-darwin/gitflow-cli/commit/b0a6b4adcc3ba455db268da29643594dfb2b3108)) - baoyx
- release v1.2.0 - ([89e1d6d](https://github.com/byx-darwin/gitflow-cli/commit/89e1d6df441cb56e3c17e281f987d802f951d4d8)) - baoyx
- update CHANGELOG.md for v1.2.0 - ([0b80ba3](https://github.com/byx-darwin/gitflow-cli/commit/0b80ba3de4ecf4f79a0ff1fdef396aab0fd0cd1c)) - baoyx

### Other

- **(gf-quality)** add minimal Go project for validation - ([55bc6ea](https://github.com/byx-darwin/gitflow-cli/commit/55bc6eae1456e3378983d607ec0e3a8436a7a807)) - baoyx
- **(gf-quality)** add minimal Node.js project for validation - ([d1c289f](https://github.com/byx-darwin/gitflow-cli/commit/d1c289fd27dd91552530730a7c36149cf616d88d)) - baoyx
- **(gf-quality)** add minimal Python project for validation - ([e1ad91f](https://github.com/byx-darwin/gitflow-cli/commit/e1ad91f53c55cf2734086a7dded06c75a6b702a7)) - baoyx
- Merge pull request #175 from byx-darwin/feat/170-skills-when-not-to-use

docs(skills): add When NOT to Use sections to all 25 skills - ([4d1f341](https://github.com/byx-darwin/gitflow-cli/commit/4d1f3416076f7aa2a28714074dc5f6f721e4df73)) - mc-ai
- Merge pull request #177 from byx-darwin/feat/173-docs-cli-requirement

docs(skills): add CLI requirement to enforce gf usage - ([8426ea2](https://github.com/byx-darwin/gitflow-cli/commit/8426ea245169ca25961ea3771b922617292a77e6)) - mc-ai
- Merge pull request #180 from byx-darwin/feat/179-skill-command-reference-fix

docs(skills): fix CLI command references to match actual implementation - ([82a663c](https://github.com/byx-darwin/gitflow-cli/commit/82a663cbcd5f8d78e1381da7c42ef48d5f70249f)) - mc-ai
- Merge pull request #182 from byx-darwin/fix/181-auth-status-scopes

fix(auth): parse token scopes from gh auth status output - ([f3237d5](https://github.com/byx-darwin/gitflow-cli/commit/f3237d519fa90b864d3b223702e74b16f82dfb1d)) - mc-ai
- Merge pull request #183 from byx-darwin/fix/gitcode-error-hint-command-name

fix(gitcode): use 'gc' instead of 'gitcode' in error hints - ([3e0231e](https://github.com/byx-darwin/gitflow-cli/commit/3e0231e3bb3aad94c8fe0e0842b53e7942a2ca1f)) - mc-ai

### Refactoring

- **(skills)** change worktree path from .claude/worktree to .worktree - ([8cbc3bf](https://github.com/byx-darwin/gitflow-cli/commit/8cbc3bf05911797a85d3e72f6824c08fd6829007)) - baoyx

---
## [1.1.0](https://github.com/byx-darwin/gitflow-cli/compare/v1.0.0..v1.1.0) - 2026-08-09

### Bug Fixes

- **(ci)** point upstream-patrol at relocated compatibility matrix - ([792c270](https://github.com/byx-darwin/gitflow-cli/commit/792c2704c8d8b918f734a1077f223de04b948afc)) - baoyx
- **(ci)** disable fail-fast on build-rust matrix - ([a6963d9](https://github.com/byx-darwin/gitflow-cli/commit/a6963d9f8f9263840beb5d57f0dffa6ea6626a57)) - baoyx
- **(ci)** use matrix.archive for artifact upload path - ([03da6b4](https://github.com/byx-darwin/gitflow-cli/commit/03da6b4ea841ff066642834afe6b4d5d0db1144e)) - baoyx
- **(cli)** make zsh completions install dir deterministic under home override - ([eef4074](https://github.com/byx-darwin/gitflow-cli/commit/eef407487a02dfe3c01f04bc2e7212c5bc891944)) - baoyx
- **(cli)** scope metadata binding to unix cfg in permissions test - ([2aa9b0d](https://github.com/byx-darwin/gitflow-cli/commit/2aa9b0d714ae3442b515daab944cb88334af0e0c)) - baoyx
- **(hook)** correct skill path hardcoding - ([7a92f7e](https://github.com/byx-darwin/gitflow-cli/commit/7a92f7e72082c8b6e3c021a2c568ab3833b9979a)) - baoyx
- **(hooks)** register auto-report-bug hook globally pointing at tracked script - ([16f6310](https://github.com/byx-darwin/gitflow-cli/commit/16f6310db967da869c15cf2d1c80cfc5cd554390)) - baoyx
- **(release)** update Homebrew formula SHA256 to v1.0.0 real values + fix auto-update (#113) - ([865375e](https://github.com/byx-darwin/gitflow-cli/commit/865375e30b7910608ac6320072fd4f4e46b1292d)) - baoyx
- **(release)** use {{version}} template syntax for cargo-release 1.1.3 (#132) - ([93c7cc8](https://github.com/byx-darwin/gitflow-cli/commit/93c7cc88e43517c8bf364568b33da909afc16f19)) - baoyx
- **(release)** expand template residue detection to catch single-brace syntax (#132) - ([e0de3b5](https://github.com/byx-darwin/gitflow-cli/commit/e0de3b59554267fe6e783388b19f438bfa704158)) - baoyx
- **(release)** use shared-version = "workspace" to fix {{version}} rendering (#132) - ([7bb75a7](https://github.com/byx-darwin/gitflow-cli/commit/7bb75a726be7857c2ead10a76e7b9efebbf4ac14)) - baoyx
- **(release)** only check headings for template residue in changelog - ([3d8a902](https://github.com/byx-darwin/gitflow-cli/commit/3d8a90262a68c96ae9bb1cbed7b7d841c512d57a)) - baoyx
- **(security)** set pending.json file permissions to 0o600 - ([d9aad15](https://github.com/byx-darwin/gitflow-cli/commit/d9aad151aec99b9dd0dc07b35b66a36984346fee)) - baoyx
- **(skills)** copy_skills_dir collects failures instead of aborting (#154) - ([31921c0](https://github.com/byx-darwin/gitflow-cli/commit/31921c0ac064dda04b89ee3b17deaafaeab39f9b)) - mc-ai
- **(website)** correct stale compatibility matrix path in llms-full.txt - ([9c1f4e6](https://github.com/byx-darwin/gitflow-cli/commit/9c1f4e625c4caad169d89da3e8643b91a7a4529b)) - baoyx
- add flate to typos ignore list (crate name false positive) - ([2fb6894](https://github.com/byx-darwin/gitflow-cli/commit/2fb689482675c0b1f30cc5a2e2fab58469923c17)) - baoyx
- resolve CI failures (clippy unnecessary_wraps + BSD-3-Clause license) - ([f2840de](https://github.com/byx-darwin/gitflow-cli/commit/f2840dece2051805db168d027976975c63399cfa)) - baoyx

### Documentation

- **(workflow)** gates.md GO-gate sub-step and source-agnostic evidence note (#141) - ([a75da44](https://github.com/byx-darwin/gitflow-cli/commit/a75da44567afacf2553346691b75fb53a4964146)) - baoyx
- dual skill source adaptation sections (#141) - ([529d1fb](https://github.com/byx-darwin/gitflow-cli/commit/529d1fb4550c20a8b29e9ba084bd4e01fe05c2a9)) - baoyx
- add design spec for gf update signature verification (#152) - ([f67c742](https://github.com/byx-darwin/gitflow-cli/commit/f67c74208413fd42ba025ee4927180b43de813d7)) - baoyx
- add implementation plan for gf update signature verification (#152) - ([02edf78](https://github.com/byx-darwin/gitflow-cli/commit/02edf784678d1c3464493860ea10fa5c27ade675)) - baoyx
- add design spec for release.toml placeholder fix (#132) - ([5d59283](https://github.com/byx-darwin/gitflow-cli/commit/5d59283084920d0eb69c444f396deff5d4879c91)) - baoyx
- add code coverage improvement design spec - ([a74c6ae](https://github.com/byx-darwin/gitflow-cli/commit/a74c6ae549f4d8a1b8595cb47ca8585b377ce79e)) - baoyx
- add code coverage improvement implementation plan - ([afb1deb](https://github.com/byx-darwin/gitflow-cli/commit/afb1debe3256c6bf684886e91a91ea37997072fc)) - baoyx
- add code review reports and release design specs - ([dcaa11a](https://github.com/byx-darwin/gitflow-cli/commit/dcaa11a0050fd6d36550bba7c1525ae8d5d7a142)) - baoyx

### Features

- **(issue)** add list_comments API and gf issue comments command - ([7c324f0](https://github.com/byx-darwin/gitflow-cli/commit/7c324f07b6e24eb2d8d31678eda32b51bfee9a74)) - baoyx
- **(security)** add sensitive data filtering for error messages - ([4cf4237](https://github.com/byx-darwin/gitflow-cli/commit/4cf42377f5ede8cdd255a3b46773c61ad720d528)) - baoyx
- **(skill)** add success notification after Issue creation - ([3fcd7c2](https://github.com/byx-darwin/gitflow-cli/commit/3fcd7c273f6d43f6f82ec1f934f333492e2b4857)) - baoyx
- **(skills)** install-time skill source detection and hard-block (#141) - ([342afef](https://github.com/byx-darwin/gitflow-cli/commit/342afef81547e4e031a84bba6d4d327915d01a77)) - baoyx
- **(update)** add gf update command and skills version management (#150) - ([dd2318e](https://github.com/byx-darwin/gitflow-cli/commit/dd2318ebf543a458481f57265eb308799af6c1bc)) - mc-ai
- **(update)** add ed25519 signature verification for release binaries - ([c29cbb8](https://github.com/byx-darwin/gitflow-cli/commit/c29cbb8c879d38c59a5584eca8d585c26c89e8c5)) - baoyx
- **(workflow)** standardize worktree path to .claude/worktree/ - ([bc68c18](https://github.com/byx-darwin/gitflow-cli/commit/bc68c18b98236443c16691f19bd137507c1207b8)) - baoyx
- **(workflow)** contract schema adds skill_source and ticket_refs (#141) - ([a90af1d](https://github.com/byx-darwin/gitflow-cli/commit/a90af1d3239b329cb9251c77241158bcb30b3578)) - baoyx
- **(workflow)** contract struct aligns with skill_source/ticket_refs schema (#141) - ([25ee030](https://github.com/byx-darwin/gitflow-cli/commit/25ee03091b2dc8a8ed524bdd0eb2414c1d98db94)) - baoyx
- **(workflow)** skill source resolution, role aliases, GO gate in SKILL.md (#141) - ([796b4df](https://github.com/byx-darwin/gitflow-cli/commit/796b4dfbd449a66a67480040500be4df3b65ea1d)) - baoyx
- **(workflow)** dual-source mapping table and branch semantics in references.md (#141) - ([8ad4fc3](https://github.com/byx-darwin/gitflow-cli/commit/8ad4fc39b8d1260becc71dcb0f6ecbd2aafbfa2e)) - baoyx
- **(workflow)** fix worktree path to .claude/worktree/ (#146) - ([ed6ad52](https://github.com/byx-darwin/gitflow-cli/commit/ed6ad5226efa71cd9955f00cf620d234bbefd42e)) - baoyx

### Miscellaneous Chores

- **(build)** refresh stale package.metadata.binstall metadata - ([55318bb](https://github.com/byx-darwin/gitflow-cli/commit/55318bb19e103dec7e87496434b6a02fbf74c0d4)) - baoyx
- **(deps)** add gitcode 0.10.3 to compatibility matrix - ([0d35cdc](https://github.com/byx-darwin/gitflow-cli/commit/0d35cdc181986c21bc18305b4866032c8901e7a6)) - baoyx
- **(deps)** add glab 1.112.0 to compatibility matrix - ([9f8a471](https://github.com/byx-darwin/gitflow-cli/commit/9f8a4711bc623d85c077465b1b56ffc9104781a1)) - baoyx
- **(gitcode)** rename CLI binary from gitcode to gc (#134) - ([fa36151](https://github.com/byx-darwin/gitflow-cli/commit/fa3615183e8eec3345ef7ae551091ebc32936369)) - mc-ai
- fmt fix for skills.rs, preserve create-a-full-plan text in SKILL.md (#141) - ([513539e](https://github.com/byx-darwin/gitflow-cli/commit/513539e5dbf8335804bc2f95c15be75b75ab08d0)) - baoyx
- embed real ed25519 public key for release verification - ([af84b2f](https://github.com/byx-darwin/gitflow-cli/commit/af84b2fd2441bf4f63f5835b429306b970e939b2)) - baoyx
- remove docs/assets (moved to resume repo) - ([3b622f1](https://github.com/byx-darwin/gitflow-cli/commit/3b622f10675a026c32e11d39778edb9d735b21db)) - baoyx
- restore docs/assets/demo.svg - ([5c35ebd](https://github.com/byx-darwin/gitflow-cli/commit/5c35ebd68026c8b30e26cc72577559981324189e)) - baoyx
- release v1.1.0 - ([b4c6f7b](https://github.com/byx-darwin/gitflow-cli/commit/b4c6f7bfd6474c2f370e6864711090e263e64bdb)) - baoyx
- update CHANGELOG.md for v1.1.0 - ([3da604d](https://github.com/byx-darwin/gitflow-cli/commit/3da604d7ede473d43976fac029b5a62af3ac8edc)) - baoyx
- update Homebrew formula to v1.1.0 - ([8a76d6d](https://github.com/byx-darwin/gitflow-cli/commit/8a76d6dcc227e34550154d02cc0b2aad07fd0ad2)) - github-actions[bot]

### Other

- Merge pull request #136 from byx-darwin/feat/135-autoreport-bug-improvements

fix(auto-report): P0/P1 improvements for auto-report-bug feature - ([51cb2d4](https://github.com/byx-darwin/gitflow-cli/commit/51cb2d47aa0825a86c6aff640e732e77db098818)) - mc-ai
- Merge pull request #137 from byx-darwin/fix/hook-worktree-registration

fix(hooks): make auto-report-bug hook work in worktrees (global + tracked script) - ([6c8e91f](https://github.com/byx-darwin/gitflow-cli/commit/6c8e91fbaf4c3340d60d7d34f545e703c13166af)) - mc-ai
- Merge pull request #140 from byx-darwin/fix/website-llms-matrix-path

fix(website): correct stale compatibility matrix path in llms-full.txt - ([e66712e](https://github.com/byx-darwin/gitflow-cli/commit/e66712e833cdb9bbb10fcec629bc518b472f2f0d)) - mc-ai
- Merge pull request #147 from byx-darwin/feat/141-dual-skill-sources

feat(workflow): dual skill source compatibility (superpowers + mattpocock/skills) - ([da71e12](https://github.com/byx-darwin/gitflow-cli/commit/da71e1289262bc808a5045b215f26065601ef388)) - mc-ai
- Merge remote-tracking branch 'origin/main' into feat-146-worktree-path

# Conflicts:
#	skills/gf-workflow/SKILL.md - ([f3e823e](https://github.com/byx-darwin/gitflow-cli/commit/f3e823e850ee7fa04319177148fee71e16a12428)) - baoyx
- Merge pull request #148 from byx-darwin/feat-146-worktree-path

feat(workflow): standardize worktree path to .claude/worktree/ - ([fb6ac4d](https://github.com/byx-darwin/gitflow-cli/commit/fb6ac4df00d518c67024ffc5e0f696c363bf6a16)) - mc-ai
- Merge pull request #155 from byx-darwin/feat/152-ed25519-signature-verify

feat(update): add ed25519 signature verification for release binaries - ([2f0d4b8](https://github.com/byx-darwin/gitflow-cli/commit/2f0d4b8e1666122a4c514f37493e43a97892b2dd)) - mc-ai
- Merge pull request #156 from byx-darwin/feat/108-gitcode-0.10.3-compat

chore(deps): add gitcode 0.10.3 to compatibility matrix - ([c3221d6](https://github.com/byx-darwin/gitflow-cli/commit/c3221d6fea25e6d65e98fce68e6b4c0fa9ce87fa)) - mc-ai
- Merge pull request #157 from byx-darwin/feat/144-glab-1.112.0-compat

chore(deps): add glab 1.112.0 to compatibility matrix - ([b682112](https://github.com/byx-darwin/gitflow-cli/commit/b682112711f7ab7b0e88499c68ad9f54372d516d)) - mc-ai
- Merge pull request #159: fix(release) use {{version}} template syntax for cargo-release 1.1.3

fix(release): use {{version}} template syntax for cargo-release 1.1.3 (#132) - ([e554f4d](https://github.com/byx-darwin/gitflow-cli/commit/e554f4d7022bf9cf06c084dd2aff1798a403cadf)) - mc-ai
- Merge pull request #160 from byx-darwin/fix/132-shared-version-workspace

fix(release): use shared-version = "workspace" to fix {{version}} rendering (#132) - ([178895e](https://github.com/byx-darwin/gitflow-cli/commit/178895e42700b40318e27d3d38c1ebc36ee20619)) - mc-ai
- Merge remote-tracking branch 'origin/main' into feat/153-binstall-metadata-fix

# Conflicts:
#	crates/gitcode/src/release.rs
#	crates/github/src/release.rs
#	crates/gitlab/src/release.rs
#	crates/release-signer/src/main.rs - ([77c6e3d](https://github.com/byx-darwin/gitflow-cli/commit/77c6e3d762f262b6cd25e417c17b327d204f9237)) - baoyx
- Merge pull request #162 from byx-darwin/feat/153-binstall-metadata-fix

chore(build): refresh stale package.metadata.binstall metadata - ([1bfdd56](https://github.com/byx-darwin/gitflow-cli/commit/1bfdd563b610a211dbde30be1b08dfb82397846c)) - mc-ai

### Style

- apply nightly rustfmt to gen_compat_matrix example (#113) - ([b50cfa5](https://github.com/byx-darwin/gitflow-cli/commit/b50cfa59dd8fb23ae0c2eaf12dffbdbc43022a27)) - baoyx
- fix formatting in error_reporter.rs - ([ee167c3](https://github.com/byx-darwin/gitflow-cli/commit/ee167c356238cc02d010586d096456ac07878ab4)) - baoyx
- apply cargo fmt to merged files - ([e58b325](https://github.com/byx-darwin/gitflow-cli/commit/e58b325a972b3022f8a89e39e3da770e7e53a2d3)) - baoyx
- fix trailing whitespace in docs - ([af87714](https://github.com/byx-darwin/gitflow-cli/commit/af87714420909c8f611bcc603a998ea85238ffbd)) - baoyx

### Tests

- **(gitcode)** add deserialization, constructor, and success-path tests for release module - ([eb19c47](https://github.com/byx-darwin/gitflow-cli/commit/eb19c479e76b27db8def75412b82487437b76622)) - baoyx
- **(github)** add success-path and is_release_not_found unit tests for release module - ([311b1d9](https://github.com/byx-darwin/gitflow-cli/commit/311b1d9473aad9cd2bfede9cd12791f737ff3083)) - baoyx
- **(gitlab)** add conversion, constructor, and success-path tests for release module - ([8bbece8](https://github.com/byx-darwin/gitflow-cli/commit/8bbece805dda9486f8e94797b4aa42dc64022d7d)) - baoyx
- **(hook)** add Bats test suite for auto-report-bug.sh - ([f5ac7d4](https://github.com/byx-darwin/gitflow-cli/commit/f5ac7d41fc0e489711112c464e739bf204248139)) - baoyx
- **(release-signer)** add edge-case and error-path tests for signing - ([fd76547](https://github.com/byx-darwin/gitflow-cli/commit/fd76547ee7961b024f6fb2dae60586153c1ffe36)) - baoyx
- improve code coverage with 114 new tests (#161) - ([1b8f106](https://github.com/byx-darwin/gitflow-cli/commit/1b8f10667a0b97f8de0820aa6a32280a053b96ba)) - mc-ai

---
## [1.0.0](https://github.com/byx-darwin/gitflow-cli/compare/v0.9.0..v1.0.0) - 2026-08-05

### Bug Fixes

- **(core)** update residual gitflow command references to gf (#126) - ([29f4053](https://github.com/byx-darwin/gitflow-cli/commit/29f4053174bb8efe0154cf4a395304fc2c87c95e)) - baoyx
- **(core)** move compatibility matrix JSON into crate for crates.io packaging (#113) - ([ffec02b](https://github.com/byx-darwin/gitflow-cli/commit/ffec02ba9e057a0a7184ee0524a647fd410376c6)) - baoyx
- **(github)** remove unsupported --json flag from review commands (#120) - ([11ab135](https://github.com/byx-darwin/gitflow-cli/commit/11ab1350151ccf6fcec33e235d050d0b77b0eac6)) - mc-ai
- **(github)** use gh api POST for issue/pr comment (#111) (#123) - ([9d4d7db](https://github.com/byx-darwin/gitflow-cli/commit/9d4d7dbcd3681cd3d25a53d00a779eb5bf4c889c)) - mc-ai
- **(github,gitlab,gitcode)** auto-create missing labels on issue create/add-label - ([a2444e5](https://github.com/byx-darwin/gitflow-cli/commit/a2444e5bbbd351f7ddf788445eeaa91621c6afa1)) - baoyx
- **(pipeline)** filter report runs by --days date window - ([9c1c79b](https://github.com/byx-darwin/gitflow-cli/commit/9c1c79be1760ddac986ec6716e89f1540708c0d3)) - baoyx
- **(prerequisites)** replace [[PLATFORM]] placeholder with actual platform name (#128) - ([2025d19](https://github.com/byx-darwin/gitflow-cli/commit/2025d197388f92218b7bef0b7362ff57ecf644e2)) - mc-ai
- **(scripts)** use --in flag for svg-term compatibility - ([db3e081](https://github.com/byx-darwin/gitflow-cli/commit/db3e081c0742565282b60fffe28fa71ce34deacd)) - baoyx

### Documentation

- **(reports)** add Phase 4 delivery reports for PR #127 (skill rename) (#126) - ([425fd3e](https://github.com/byx-darwin/gitflow-cli/commit/425fd3ef4ca96d92d10c50bdc3086914ff346ce0)) - baoyx
- **(skills)** add plan and design doc for gitflow-* to gf-* rename (#126) - ([3a160a7](https://github.com/byx-darwin/gitflow-cli/commit/3a160a7097e87969ad4a54b7ab41ac1c724ead76)) - baoyx
- **(website)** add support policy page and navigation link - ([c0b6259](https://github.com/byx-darwin/gitflow-cli/commit/c0b62592a2ddc1166c5dbf2194cb543dcc88b2b7)) - baoyx
- add SUPPORT.md with support policy and EOL matrix - ([3cf688f](https://github.com/byx-darwin/gitflow-cli/commit/3cf688ffe577eb5f29918e2a53fd1b090e910705)) - baoyx
- update compatibility matrix to v1.0.0 - ([1a86509](https://github.com/byx-darwin/gitflow-cli/commit/1a8650946d0ce065de6b8b6fee8baa0f3f0bf384)) - baoyx
- update demo.svg version to 1.0.0 - ([172f867](https://github.com/byx-darwin/gitflow-cli/commit/172f867142c76952f2b53e427da4cf80d930246a)) - baoyx
- add v1.0.0 release design spec and implementation plan (#113) - ([e167707](https://github.com/byx-darwin/gitflow-cli/commit/e167707591d481f229bb64cb4ba22e0ecc95f8c6)) - baoyx
- add v1.0.0 dogfooding report (PASS after #130 fix) - ([b9a4817](https://github.com/byx-darwin/gitflow-cli/commit/b9a4817fc651d64416699b4937f7d2500a7123ce)) - baoyx

### Features

- **(workflow)** add Branch Finish step to Phase 4 (#92) - ([1593bf2](https://github.com/byx-darwin/gitflow-cli/commit/1593bf2ec2196a52d89525079532bc39c6be1de2)) - mc-ai

### Miscellaneous Chores

- **(deps)** update compatibility matrix for gh 2.97.0 and glab 1.111.0 (#116) - ([2f5a750](https://github.com/byx-darwin/gitflow-cli/commit/2f5a750833149e1d2815c191b561093350d9956c)) - mc-ai
- update Homebrew formula to v0.9.0 - ([95e8fd5](https://github.com/byx-darwin/gitflow-cli/commit/95e8fd59d0a872c63e88236d8777491684a8683c)) - github-actions[bot]
- update Homebrew formula to v0.3.0 - ([a7fb36a](https://github.com/byx-darwin/gitflow-cli/commit/a7fb36a061bdc8a4b0014409751e770d2cd93028)) - github-actions[bot]
- release v1.0.0 - ([a1ec903](https://github.com/byx-darwin/gitflow-cli/commit/a1ec903021ebff8833c2f619d10448415a37c01b)) - baoyx
- update CHANGELOG.md for v1.0.0 - ([2b8ca96](https://github.com/byx-darwin/gitflow-cli/commit/2b8ca96f7d7a471cb5fadeae7cdded152dde2392)) - baoyx
- update Homebrew formula to v1.0.0 - ([3023310](https://github.com/byx-darwin/gitflow-cli/commit/3023310bf7c8362e630cc690cdc34542f0d86749)) - github-actions[bot]

### Other

- Merge pull request #129 from byx-darwin/feat/113-v1.0.0-release

docs: v1.0.0 release documentation preparation - ([4378c25](https://github.com/byx-darwin/gitflow-cli/commit/4378c2516707e4d1d45f4e372b1f14dc5303098a)) - mc-ai
- Merge pull request #131 from byx-darwin/fix/130-release-delete-cleanup-tag

fix(github): release delete adds --cleanup-tag + idempotent not-found (#130) - ([d508ebb](https://github.com/byx-darwin/gitflow-cli/commit/d508ebbe6951fee64204e7e0712c96b096dbcda2)) - mc-ai

### Refactoring

- **(crates)** rename crates to gitflow-* prefix for crates.io publishing (#113) - ([ca84b53](https://github.com/byx-darwin/gitflow-cli/commit/ca84b53665d1f409eef8444756972db9e20ab741)) - baoyx
- **(skills)** update skill prefix filter from gitflow- to gf- (#126) - ([d897a74](https://github.com/byx-darwin/gitflow-cli/commit/d897a745734c4c46f4ce8f547bbc6008128924b5)) - baoyx
- **(skills)** update test assertions to gf-* skill names (#126) - ([ae5b822](https://github.com/byx-darwin/gitflow-cli/commit/ae5b822a12a2b4194d8dd0a878851c4d0360f54d)) - baoyx
- **(skills)** update Makefile and install.sh skill filters to gf-* (#126) - ([b51a989](https://github.com/byx-darwin/gitflow-cli/commit/b51a989cba536d15233dc3a9414b2f9ac8f75ea5)) - baoyx
- **(skills)** update auto-report-bug hook to gf-autoreport-bug (#126) - ([0ba4344](https://github.com/byx-darwin/gitflow-cli/commit/0ba43444ac3069489233370969983a01ccbe0076)) - baoyx
- **(skills)** rename skill directories from gitflow-* to gf-* (#126) - ([7bc66ed](https://github.com/byx-darwin/gitflow-cli/commit/7bc66ed8f5acd50a80d808edb910ed31918259a6)) - baoyx
- **(skills)** rename skill reference docs to gf-* (#126) - ([7f1567a](https://github.com/byx-darwin/gitflow-cli/commit/7f1567a03e4b25f90bfd60f454cab90059f6a234)) - baoyx
- **(skills)** replace gitflow-* skill references with gf-* across repo (#126) - ([8b85005](https://github.com/byx-darwin/gitflow-cli/commit/8b8500596092acc23e0fdaa7fad3375c19022a69)) - baoyx
- **(skills)** regression fixes for residual gitflow- references (#126) - ([e401595](https://github.com/byx-darwin/gitflow-cli/commit/e401595ec34328b36d9e266dbb8ac4383b58a9d5)) - baoyx
- **(skills)** fix residual gitflow-* glob strings in website and assets (#126) - ([4794db5](https://github.com/byx-darwin/gitflow-cli/commit/4794db54e0f8f5612901971988f5f19ecf1bc1be)) - baoyx

### Style

- **(github)** add backticks to doc comment (clippy::doc_markdown) - ([ac3fac6](https://github.com/byx-darwin/gitflow-cli/commit/ac3fac622e5ed10a944229be793efdf5b2baf60f)) - baoyx
- **(github)** fix comment typo Unparseable -> Unparsable (typos hook) - ([8c7074c](https://github.com/byx-darwin/gitflow-cli/commit/8c7074c7876b2adeecc9ca9d5c829b7df7ce56af)) - baoyx
- apply nightly rustfmt to baseline (format-only, no behavior change) - ([2ed5af4](https://github.com/byx-darwin/gitflow-cli/commit/2ed5af41b9715a1425a00a8a95c929f30f2f06f7)) - baoyx

---
## [0.9.0](https://github.com/byx-darwin/gitflow-cli/compare/v0.8.0..v0.9.0) - 2026-07-14

### Bug Fixes

- **(core)** add platform serde aliases for State and ReviewState - ([984ef21](https://github.com/byx-darwin/gitflow-cli/commit/984ef2196f3e549fbbe0f60f7aa8ef9552f92a7b)) - baoyx
- **(core)** add backticks to GitCode in doc comment - ([fbba901](https://github.com/byx-darwin/gitflow-cli/commit/fbba90126cdbb5853cd3afcf8b91b8fe33941b26)) - baoyx
- **(skills)** harden gitflow-workflow orchestrator and fix SDO violations - ([8a2464d](https://github.com/byx-darwin/gitflow-cli/commit/8a2464d0ae8608c27ffecaaf34683d9bd06152b6)) - baoyx
- **(skills)** add cross-session recovery to Contract First rule - ([fbb4e33](https://github.com/byx-darwin/gitflow-cli/commit/fbb4e334c11fb80305c11aeb6de8e78d43b4dfc2)) - baoyx
- **(skills)** add Fast Mode — Required Skills Checklist section - ([c3b07e2](https://github.com/byx-darwin/gitflow-cli/commit/c3b07e2000b6c108104d7094738897255480f295)) - baoyx
- **(skills)** add Phase 2 quality check details to SKILL.md - ([d2a16a2](https://github.com/byx-darwin/gitflow-cli/commit/d2a16a2805ebc4d37ff50bdfa85f32fdd2f85778)) - baoyx
- **(skills)** add 'create a full plan' phrase to Phase 2 - ([4c6d366](https://github.com/byx-darwin/gitflow-cli/commit/4c6d36657f2630b86ba056e671f9860cd3e3ba83)) - baoyx
- **(skills)** add Phase 4 output descriptions to SKILL.md - ([38d229f](https://github.com/byx-darwin/gitflow-cli/commit/38d229feff4baa1c95ab63f393ac8944fa262f3a)) - baoyx
- include hooks directory in crates.io package - ([b508cf6](https://github.com/byx-darwin/gitflow-cli/commit/b508cf668cb66b373a84727fbbfa820527f4eb4c)) - baoyx
- correct include path for hooks directory - ([7c7fc52](https://github.com/byx-darwin/gitflow-cli/commit/7c7fc52cfb8cacbd2b8f77bf4882d2aa826d61ae)) - baoyx
- copy hooks directory to apps/cli for crates.io packaging - ([a7238ad](https://github.com/byx-darwin/gitflow-cli/commit/a7238ad83d12d677ec93d99f5a0148b22199b826)) - baoyx
- correct include_bytes path for hooks in crates.io packaging - ([dc894db](https://github.com/byx-darwin/gitflow-cli/commit/dc894db74b324573fa945d6897ba21d7437a7476)) - baoyx
- remove duplicate tag-message key in release.toml - ([6e92434](https://github.com/byx-darwin/gitflow-cli/commit/6e92434de44f99202a3163242fe64ccd36abc214)) - baoyx

### Documentation

- add release workflow section to CLAUDE.md - ([6aff293](https://github.com/byx-darwin/gitflow-cli/commit/6aff29301bac55f85200cbcc7259080d6bbd6e12)) - baoyx
- add individual README for each crate - ([33793fc](https://github.com/byx-darwin/gitflow-cli/commit/33793fcc09c346140d2dc90ea842d7c484ab0480)) - baoyx

### Features

- **(skills)** language-agnostic quality gate with dynamic detection - ([83272b8](https://github.com/byx-darwin/gitflow-cli/commit/83272b8e2f946839e322ed5bb78f57ffd460d95a)) - baoyx
- **(skills)** add multi-language project support to quality gate - ([f5070cc](https://github.com/byx-darwin/gitflow-cli/commit/f5070cc769999c8a3a8446bcf850212591ed2e39)) - baoyx
- add crates.io publishing support - ([6e146fe](https://github.com/byx-darwin/gitflow-cli/commit/6e146fe57cd12e239e35f232c8297b3556ce1bf0)) - baoyx
- add CI check before crates.io publish - ([e66faa7](https://github.com/byx-darwin/gitflow-cli/commit/e66faa701b934a73fcd969ed493eb692f746861b)) - baoyx

### Miscellaneous Chores

- update Homebrew formula to v0.8.0 - ([c9158ee](https://github.com/byx-darwin/gitflow-cli/commit/c9158ee8194ce3883f2dee27560c8078a91a7ffb)) - github-actions[bot]
- add version requirements to workspace crates for crates.io publishing - ([73b8bc2](https://github.com/byx-darwin/gitflow-cli/commit/73b8bc24b78bcaf48c663eacf0c07f1f77a2a01f)) - baoyx
- release v{{version}} - ([9331bfa](https://github.com/byx-darwin/gitflow-cli/commit/9331bfa31d49894eebf7e3247bc2347377d5789c)) - baoyx
- update CHANGELOG.md for v0.9.0 - ([7b274c7](https://github.com/byx-darwin/gitflow-cli/commit/7b274c7fcbf7c02edae39aa3b07dfb861985fb09)) - baoyx

### Other

- build gitflow-cli binary before running e2e tests - ([deeac04](https://github.com/byx-darwin/gitflow-cli/commit/deeac04f2149ca1842011d77b4d214d1afe007fa)) - baoyx
- build gitflow-cli binary before running tests in build workflow - ([bf6a3b7](https://github.com/byx-darwin/gitflow-cli/commit/bf6a3b7ca9bce14ef50852e7a617fcf841f06b82)) - baoyx

### Refactoring

- **(skills)** compress gitflow-workflow SKILL.md by 49% - ([b86c463](https://github.com/byx-darwin/gitflow-cli/commit/b86c46324d6edd7f5afc61b1ec9c6c323e246dd1)) - baoyx

---
## [0.8.0](https://github.com/byx-darwin/gitflow-cli/compare/v0.7.0..v0.8.0) - 2026-07-10

### Bug Fixes

- **(skills)** make --report-bug flag negatable with --report-bug=false - ([73a4cb5](https://github.com/byx-darwin/gitflow-cli/commit/73a4cb55236c7174a02a8026a67861a9ac053d85)) - baoyx
- **(skills)** fix bundled-counter overwritten dead code in install_single_skill_bundled - ([e76ace7](https://github.com/byx-darwin/gitflow-cli/commit/e76ace7ce79cf83d89cbc684441e2ce5160a3dca)) - baoyx
- **(skills)** remove outdated comment referencing Task 2 - ([c167583](https://github.com/byx-darwin/gitflow-cli/commit/c167583278b1f21a2d0e4eb0bcd15b1f9f5c808f)) - baoyx

### Documentation

- **(autoreport-bug)** add unauthenticated fallback branch to skill - ([c7cf8cc](https://github.com/byx-darwin/gitflow-cli/commit/c7cf8cce2a8f75d4f988a8209ef7e68c63dac704)) - baoyx
- **(autoreport-bug)** reconcile SKILL.md sections with new auth flow - ([12468b1](https://github.com/byx-darwin/gitflow-cli/commit/12468b1d0c3c8b56aaf4233e7c5023621df278ab)) - baoyx
- **(skills)** clarify co_contribution is user-level setting - ([312a2e1](https://github.com/byx-darwin/gitflow-cli/commit/312a2e1034a8c062048a9d082a8f8371aff2db5d)) - baoyx
- **(workflow)** add dogfooding checklist to Phase 4 (#73) - ([622c90a](https://github.com/byx-darwin/gitflow-cli/commit/622c90ae24735298dec4828a6f54459d5843c38a)) - baoyx
- update README badges - ([8250826](https://github.com/byx-darwin/gitflow-cli/commit/8250826c34ab74d463a32a85330ba6010198b165)) - baoyx
- add code of conduct and update contributing guidelines - ([dc85ba8](https://github.com/byx-darwin/gitflow-cli/commit/dc85ba8c2a2c871b168e9d8f15d1758e603d956f)) - baoyx
- add co-contribution plan design spec - ([5688fa2](https://github.com/byx-darwin/gitflow-cli/commit/5688fa24268e394d8cdd1fc9c92036adc3bb59c1)) - baoyx
- add co-contribution plan implementation plan - ([0df399c](https://github.com/byx-darwin/gitflow-cli/commit/0df399cfc306def126f4f4979bba5af45a16c432)) - baoyx
- add design spec for co-contribution global-only marker (#82) - ([e14b819](https://github.com/byx-darwin/gitflow-cli/commit/e14b819a716f08fadb900101b12f9c4f051e17bc)) - baoyx
- add implementation plan for co-contribution global-only marker (#82) - ([1a294a5](https://github.com/byx-darwin/gitflow-cli/commit/1a294a5fe3ac677114ce315b2d148b14f4cbea03)) - baoyx
- add Phase 4 dogfooding checklist (#73) - ([851bb9e](https://github.com/byx-darwin/gitflow-cli/commit/851bb9e08605eb18d9c51b5b85197956688316c9)) - baoyx
- fix dogfooding checklist table count and references (#73) - ([6cef5bc](https://github.com/byx-darwin/gitflow-cli/commit/6cef5bc728139d3a3a69d285b7060bf4ab2dc057)) - baoyx
- add dogfooding checklist to docs index (#73) - ([bd384f4](https://github.com/byx-darwin/gitflow-cli/commit/bd384f4b51688e729c9e3a0579c1781853416b41)) - baoyx
- fix dogfooding checklist reference link depth (#73) - ([2e687ac](https://github.com/byx-darwin/gitflow-cli/commit/2e687ac71db4bacd3e16ec43e40a64292b2395d2)) - baoyx
- add design spec for Phase 4 dogfooding checklist (#73) - ([42c9292](https://github.com/byx-darwin/gitflow-cli/commit/42c9292d4c58c7c1d4914a43ef45f30bc64160f5)) - baoyx
- fix self-review issues in dogfooding spec (#73) - ([e77248f](https://github.com/byx-darwin/gitflow-cli/commit/e77248f3f059daf687278e15acc46f7d2ba77f5c)) - baoyx
- add pipeline analysis report for PR #86 - ([3e227e9](https://github.com/byx-darwin/gitflow-cli/commit/3e227e962e5b584bab11f094471c7908f8388bb7)) - baoyx
- add implementation plan for Phase 4 dogfooding checklist (#73) - ([7ea690a](https://github.com/byx-darwin/gitflow-cli/commit/7ea690a676b7cada1c04bf61cb0e9f6d75d3c66d)) - baoyx
- add issue triage report for Phase 4 - ([aeaf415](https://github.com/byx-darwin/gitflow-cli/commit/aeaf415d1c2f7a76a947c6d9f6981b80166958e6)) - baoyx
- add code review report for PR #86 (self-review) - ([0499262](https://github.com/byx-darwin/gitflow-cli/commit/0499262b96b66cd5cc4523b2b8f44730e48e58bd)) - baoyx
- add release workflow demo script - ([1155376](https://github.com/byx-darwin/gitflow-cli/commit/115537699aef9f42d38a8efdea341c1fd90b085d)) - baoyx

### Features

- **(error-reporter)** gate bug reporting on co-contribution opt-in marker - ([e0f5b9f](https://github.com/byx-darwin/gitflow-cli/commit/e0f5b9f569a69771723b273cf2ca1775a70e40d4)) - baoyx
- **(hook)** add auth failure fallback with login guide and Issue template - ([707a40d](https://github.com/byx-darwin/gitflow-cli/commit/707a40d2fdff1448e2762dd083ec55fe12f637de)) - baoyx
- **(skills)** add confirm() helper for interactive Y/n prompts - ([4f29840](https://github.com/byx-darwin/gitflow-cli/commit/4f2984009f9dda6cc358f5a2d3de984202a24d96)) - baoyx
- **(skills)** add merge_co_contribution() for settings.json marker - ([f18ea3f](https://github.com/byx-darwin/gitflow-cli/commit/f18ea3fed3d7975db249416e2ccb416a332c50d9)) - baoyx
- **(skills)** add co-contribution plan flow to install_skills() - ([cc731bd](https://github.com/byx-darwin/gitflow-cli/commit/cc731bd24f2a2032b95284c06c77498d9376e26e)) - baoyx
- **(workflow)** add auto-trigger orchestration for gitflow-workflow (#83) - ([b4f60c1](https://github.com/byx-darwin/gitflow-cli/commit/b4f60c1718443872df5aeeae083f7f88387b9122)) - mc-ai
- improve release workflow with safety checks and interactive preview - ([c126220](https://github.com/byx-darwin/gitflow-cli/commit/c126220b84a4c11ab0c44cf1557a608e957b71f2)) - baoyx

### Miscellaneous Chores

- update Homebrew formula to v0.7.0 - ([f4fe430](https://github.com/byx-darwin/gitflow-cli/commit/f4fe430c7f5f5eed627892cf0c49dc4c92a3a32c)) - github-actions[bot]
- unify license to MIT - ([eb79794](https://github.com/byx-darwin/gitflow-cli/commit/eb797940705e3522ac7f04e95aca9c0759b3931e)) - baoyx
- remove redundant LICENSE.md - ([e87d1a4](https://github.com/byx-darwin/gitflow-cli/commit/e87d1a49f70de026f911f1f392a60d558c08b60d)) - baoyx
- remove obsolete plan-issue-59.md - ([623546f](https://github.com/byx-darwin/gitflow-cli/commit/623546f52426655e3adfc92c2030240850598476)) - baoyx
- add .codegraph to gitignore and remove from tracking - ([2b224be](https://github.com/byx-darwin/gitflow-cli/commit/2b224be01f41f72233731d2919f819e5a4416236)) - baoyx
- remove unused .tokeignore - ([0f1662a](https://github.com/byx-darwin/gitflow-cli/commit/0f1662a7946a5982de5c3c8240ae2603ad1d480e)) - baoyx
- release v{{version}} - ([0b0e9d7](https://github.com/byx-darwin/gitflow-cli/commit/0b0e9d75274570db05b251c5d57bad7d366f7624)) - baoyx
- update CHANGELOG.md for v0.8.0 - ([3630a9c](https://github.com/byx-darwin/gitflow-cli/commit/3630a9c0f72e018e50519edd96b0dd53095e82f2)) - baoyx

### Other

- Merge pull request #81 from byx-darwin/feat/co-contribution-plan

feat(skills): add co-contribution plan with GitHub auth verification - ([f1d4eef](https://github.com/byx-darwin/gitflow-cli/commit/f1d4eef9f47d49f45e7bde8ac96c53e78ad5906a)) - mc-ai
- Merge pull request #84 from byx-darwin/feat/82-co-contribution-global-only

feat(config): force co-contribution marker to global settings (#82) - ([5b73482](https://github.com/byx-darwin/gitflow-cli/commit/5b73482b59e297551e25b580bb670b69eb741063)) - mc-ai
- Merge pull request #87 from byx-darwin/feat/73-dogfooding-checklist

docs: add Phase 4 dogfooding checklist (#73) - ([8466bbe](https://github.com/byx-darwin/gitflow-cli/commit/8466bbec827ee3d783381802f2991c096affb3e9)) - mc-ai

### Refactoring

- **(skills)** simplify AgentPlatform::detect() to always return Claude - ([feaf3db](https://github.com/byx-darwin/gitflow-cli/commit/feaf3db1cb985e48b52e123ecaab5f3bf9bfcd58)) - baoyx
- **(skills)** extract co-contribution flow into try_enable_co_contribution() helper - ([ade3aad](https://github.com/byx-darwin/gitflow-cli/commit/ade3aad15ceb60c2899ce10577df38a70c34294a)) - baoyx
- **(skills)** force global write for co_contribution marker - ([7347178](https://github.com/byx-darwin/gitflow-cli/commit/734717863b9ed3de9445792368dee6e4860e337a)) - baoyx

### Tests

- **(skills)** add failing test for global-only co_contribution write - ([7480016](https://github.com/byx-darwin/gitflow-cli/commit/748001699649a469085e5e7c57ea86aed1ae6f99)) - baoyx
- add E2E non-interactive test framework (#71) (#86) - ([1413133](https://github.com/byx-darwin/gitflow-cli/commit/1413133fce05be1740afabe3765269ede7e1ebc7)) - mc-ai

---
## [0.7.0](https://github.com/byx-darwin/gitflow-cli/compare/v0.6.0..v0.7.0) - 2026-07-09

### Bug Fixes

- **(workflow)** remind Closes #N keyword for auto-close - ([937b04a](https://github.com/byx-darwin/gitflow-cli/commit/937b04a5d53b2359c58f2621208b2854b37279d4)) - baoyx

### Documentation

- add skill language convention to CLAUDE.md - ([9a85d91](https://github.com/byx-darwin/gitflow-cli/commit/9a85d91896a3baa840d0ce6e8002f04badd52ef4)) - baoyx
- add workflow orchestrator design docs and implementation plan - ([0576852](https://github.com/byx-darwin/gitflow-cli/commit/05768520e33e99e9a52c3dcf0e05246afcda8404)) - baoyx

### Features

- add TOON output format for LLM token optimization - ([a75b9f7](https://github.com/byx-darwin/gitflow-cli/commit/a75b9f730e59ed951857260e1551ab9c76de8b4e)) - baoyx

### Miscellaneous Chores

- **(skills)** translate all 26 skill bodies to English (#77) - ([874d20f](https://github.com/byx-darwin/gitflow-cli/commit/874d20f240d958368fe46efdb669e72a123428ad)) - mc-ai
- release v0.7.0 - ([a63937c](https://github.com/byx-darwin/gitflow-cli/commit/a63937cac632ab4aa958ff3866d1169172747754)) - baoyx
- update CHANGELOG.md - ([3387e71](https://github.com/byx-darwin/gitflow-cli/commit/3387e71d3517b215818405d74d55796241565400)) - baoyx

---
## [0.6.0](https://github.com/byx-darwin/gitflow-cli/compare/v0.5.0..v0.6.0) - 2026-07-08

### Bug Fixes

- **(gitcode)** resolve authentication check bugs and refactor architecture (#54) - ([7bde64d](https://github.com/byx-darwin/gitflow-cli/commit/7bde64d842f01cc44d4982b77287719dd5b66ea2)) - mc-ai
- **(gitcode)** remove unsupported strategy flags from pr merge (#59) (#61) - ([802ae83](https://github.com/byx-darwin/gitflow-cli/commit/802ae83e6c90e2bdcbab41987ea2315ac5fe6c65)) - mc-ai
- **(github)** remove unsupported --json flag from issue/pr comment and pr create (#60) - ([b2b2c5c](https://github.com/byx-darwin/gitflow-cli/commit/b2b2c5cf0f5fd8acc59d0931deea4ec9acdbca6f)) - mc-ai

### Miscellaneous Chores

- update Homebrew formula to v0.5.0 - ([ef6ba47](https://github.com/byx-darwin/gitflow-cli/commit/ef6ba4738d36abbec52ecc4065bddd8266962959)) - github-actions[bot]
- remove .superpowers/sdd/ temporary files (should be gitignored) - ([86e6335](https://github.com/byx-darwin/gitflow-cli/commit/86e6335104d8ac1cdf59e24f2fe6adaed7d2332d)) - baoyx
- update Homebrew formula to v0.6.0 - ([149ed8e](https://github.com/byx-darwin/gitflow-cli/commit/149ed8ee06dec10c2a981a3867057ceee3c10da2)) - github-actions[bot]
- update CHANGELOG.md for v0.6.0 - ([5e1d902](https://github.com/byx-darwin/gitflow-cli/commit/5e1d90248f59503be177e51f843dffe7d5f796eb)) - baoyx

---
## [0.5.0](https://github.com/byx-darwin/gitflow-cli/compare/v0.4.0..v0.5.0) - 2026-07-07

### Bug Fixes

- **(auth)** parse both old and new gh CLI status formats - ([4ef75be](https://github.com/byx-darwin/gitflow-cli/commit/4ef75be6382cf731e49214c40b8cbc415bf905b2)) - baoyx
- **(gitcode)** fix issue close and comment serialization for GitCode API (#11, #12) - ([046338c](https://github.com/byx-darwin/gitflow-cli/commit/046338c488df26b10b0e9d44e6bd64e1d4f7c6ab)) - baoyx
- **(gitcode)** fix issue close and comment serialization for GitCode API (#11, #12) - ([cbaa58e](https://github.com/byx-darwin/gitflow-cli/commit/cbaa58e747c921a3f87d35f0fdb4bc4db49ace39)) - baoyx
- **(gitcode)** fix issue close and comment serialization for GitCode API (#11, #12) - ([f567606](https://github.com/byx-darwin/gitflow-cli/commit/f5676069747a720da28a14c9a20f09d2730909e9)) - baoyx
- **(gitcode)** fix issue close and comment serialization for GitCode API (#11, #12) - ([32deb01](https://github.com/byx-darwin/gitflow-cli/commit/32deb0197fee73decbf8ae291fdb36a861af2424)) - baoyx
- **(gitcode)** fix issue close and comment serialization for GitCode API (#11, #12) - ([b45d967](https://github.com/byx-darwin/gitflow-cli/commit/b45d967077333dc8aa8cf17f12e3fff4978da327)) - baoyx
- **(gitcode)** fix issue close and comment serialization for GitCode API (#11, #12) - ([f4c59a8](https://github.com/byx-darwin/gitflow-cli/commit/f4c59a8ad698ed6fd20127066438bb507defc5b2)) - baoyx
- **(makefile)** use correct path for workspace cargo install - ([141f073](https://github.com/byx-darwin/gitflow-cli/commit/141f07379891777bec633bdeb47b15090ef842f9)) - baoyx
- **(skill)** add explicit ## Overview section to gitflow-release-helper - ([f63a80d](https://github.com/byx-darwin/gitflow-cli/commit/f63a80dec69b9fe97aa3520bc99821fa149f106c)) - baoyx
- **(skill)** clarify parallel execution in worktree - ([ecffc51](https://github.com/byx-darwin/gitflow-cli/commit/ecffc51dd265360a09a4695896263c7858ed3eb1)) - baoyx
- **(skills)** install project hook to hooks/ to match settings.json command path - ([d43123a](https://github.com/byx-darwin/gitflow-cli/commit/d43123a45ed40d962479e718a58fca5a30fb09b7)) - baoyx
- update label-stats SKILL.md to refactored version with trigger format - ([5a0d103](https://github.com/byx-darwin/gitflow-cli/commit/5a0d103f092b7d85845904b2d8d5ba869e493dc2)) - baoyx
- add Common Mistakes to 5 skills + overflow wordcount trim - ([114183c](https://github.com/byx-darwin/gitflow-cli/commit/114183c328348fd4ac3c4ddf0aa0b7a7bc30c155)) - baoyx
- resolve hook path mismatch and auth status parsing bugs (#46) - ([166a7ca](https://github.com/byx-darwin/gitflow-cli/commit/166a7ca773de59f718212f99d29906ba493300b6)) - mc-ai

### Documentation

- update version badge to v0.4.0 - ([5ce33bc](https://github.com/byx-darwin/gitflow-cli/commit/5ce33bcc0c368dabea66aad7962d3f48e0bcd61e)) - baoyx
- analyze gitflow-auth skill (#15) - ([1362f6f](https://github.com/byx-darwin/gitflow-cli/commit/1362f6f4dab1f7e398ce7fb4c8894ea3fb80ab71)) - baoyx
- analyze gitflow-commit skill (#16) - ([0ba11cb](https://github.com/byx-darwin/gitflow-cli/commit/0ba11cb51afe72df055daf5142db957775f5c4fa)) - baoyx
- analyze gitflow-label-milestone skill (#17) - ([9024318](https://github.com/byx-darwin/gitflow-cli/commit/902431857e4e9cf600a86fdf2d1f981d59074fc7)) - baoyx
- analyze gitflow-release skill (#18) - ([dcf645d](https://github.com/byx-darwin/gitflow-cli/commit/dcf645d7e6abf3c0d6836bd0a84a2586ffcf08b6)) - baoyx
- analyze gitflow-repo skill (#19) - ([d0b0d26](https://github.com/byx-darwin/gitflow-cli/commit/d0b0d26fb5ec21deb643ac7a406cc778a7c2fdb4)) - baoyx
- analyze gitflow-repo-onboarding skill (#20) - ([0a15b70](https://github.com/byx-darwin/gitflow-cli/commit/0a15b7091b4a7408d2cc3db639327f4a7c8ece45)) - baoyx
- analyze gitflow-security-check skill (#22) - ([fd352c0](https://github.com/byx-darwin/gitflow-cli/commit/fd352c0ec8ab5eb4050a7144c8fdba9356328b35)) - baoyx
- analyze gitflow-weekly-report skill (#23) - ([2b3ad66](https://github.com/byx-darwin/gitflow-cli/commit/2b3ad66f2006061beecc57920522f4ee1efdcb4c)) - baoyx
- analyze gitflow-precommit skill (#24) - ([b1be00b](https://github.com/byx-darwin/gitflow-cli/commit/b1be00b83a69c180e16f4c54097179dc166c80c1)) - baoyx
- analyze gitflow-regression skill (#25) - ([491810d](https://github.com/byx-darwin/gitflow-cli/commit/491810d3cee10c1561164b3b5e3cbd0537389e1d)) - baoyx
- analyze gitflow-issue-create skill (#26) - ([7b3a9a6](https://github.com/byx-darwin/gitflow-cli/commit/7b3a9a6ae1467cf2ecae7358d6ab22f7581780d2)) - baoyx
- analyze gitflow-pr-create skill (#27) - ([336048f](https://github.com/byx-darwin/gitflow-cli/commit/336048f02bd0414159deb808c951b09542eb58c9)) - baoyx
- analyze gitflow-pipeline-analyzer skill (#28) - ([78df1b9](https://github.com/byx-darwin/gitflow-cli/commit/78df1b9315e924c8f42da9d18a3e26aa52c17d28)) - baoyx
- analyze gitflow-issue-triage skill (#29) - ([de40bca](https://github.com/byx-darwin/gitflow-cli/commit/de40bca18b6b99d943178aab3c6a39e5ec595c4f)) - baoyx
- analyze gitflow-label-stats skill (#30) - ([0640f93](https://github.com/byx-darwin/gitflow-cli/commit/0640f9395b47d3fe0329139f2034e23530ba9b41)) - baoyx
- complete Phase 2-4 skill analysis (12 skills, parallel execution) - ([1c7f6de](https://github.com/byx-darwin/gitflow-cli/commit/1c7f6deee876270c4ffab0c8c3e71fcf25326c74)) - baoyx
- create comprehensive skills refactor analysis and implementation plan - ([2e9f780](https://github.com/byx-darwin/gitflow-cli/commit/2e9f7808e2243f3864cab438a2232a355ba6a032)) - baoyx
- quality gate passed for skills refactor analysis - ([127b7c5](https://github.com/byx-darwin/gitflow-cli/commit/127b7c53629473a71ff04af986ac4d5f9e93d4e2)) - baoyx
- add unified skill template and conventions for 26-skill refactor - ([94b248c](https://github.com/byx-darwin/gitflow-cli/commit/94b248c72d98592a2184086f80fdba4d1fc64a43)) - baoyx
- analyze gitflow-auth skill (#15) - ([64dd080](https://github.com/byx-darwin/gitflow-cli/commit/64dd080c6c6ce46e09b7c115d982d1b7974658b1)) - baoyx
- analyze gitflow-commit skill (#16) - ([3b7aeee](https://github.com/byx-darwin/gitflow-cli/commit/3b7aeee44aec3354c8b9898b32603a82e748bcf1)) - baoyx
- analyze gitflow-label-milestone skill (#17) - ([0ab0064](https://github.com/byx-darwin/gitflow-cli/commit/0ab0064d70d961dcbd5ce4d831d30af09ad1858c)) - baoyx
- analyze gitflow-release skill (#18) - ([be6b1db](https://github.com/byx-darwin/gitflow-cli/commit/be6b1db25bbcc442753ec9ec401647738e583f7e)) - baoyx
- analyze gitflow-repo skill (#19) - ([3728a85](https://github.com/byx-darwin/gitflow-cli/commit/3728a85004fff0d86ce29a02bc3f5f8485421fd5)) - baoyx
- analyze gitflow-repo-onboarding skill (#20) - ([c9b629d](https://github.com/byx-darwin/gitflow-cli/commit/c9b629d2709b0fe147c0f9d23460ce2e91772d15)) - baoyx
- analyze gitflow-security-check skill (#22) - ([31e3290](https://github.com/byx-darwin/gitflow-cli/commit/31e32903dd2d83e16275a99342dceaa81a428dd7)) - baoyx
- analyze gitflow-weekly-report skill (#23) - ([08a6455](https://github.com/byx-darwin/gitflow-cli/commit/08a6455b3257062d1af74db330d96a85bb744de8)) - baoyx
- analyze gitflow-precommit skill (#24) - ([7f46a3f](https://github.com/byx-darwin/gitflow-cli/commit/7f46a3fad0e3a922787aec2225fd671b19893a36)) - baoyx
- analyze gitflow-regression skill (#25) - ([a38ec36](https://github.com/byx-darwin/gitflow-cli/commit/a38ec36d7daf06eec08a66c6e28a508311549b8b)) - baoyx
- analyze gitflow-issue-create skill (#26) - ([0265a55](https://github.com/byx-darwin/gitflow-cli/commit/0265a5535aaf4e7b40c8c30b2fd6df7228ba1c4b)) - baoyx
- analyze gitflow-pr-create skill (#27) - ([08992ab](https://github.com/byx-darwin/gitflow-cli/commit/08992ab390cd0e7e92e26b93edbbefb1603ac5e1)) - baoyx
- analyze gitflow-pipeline-analyzer skill (#28) - ([4779466](https://github.com/byx-darwin/gitflow-cli/commit/4779466211ed85d275d793cf53b04fb386c116d7)) - baoyx
- analyze gitflow-issue-triage skill (#29) - ([5bdff31](https://github.com/byx-darwin/gitflow-cli/commit/5bdff31ed51ea2b28f6ffd184ffa030ef8c1363c)) - baoyx
- analyze gitflow-label-stats skill (#30) - ([d750d36](https://github.com/byx-darwin/gitflow-cli/commit/d750d36f51259854b149fd6f5218f6e8a15b7a5d)) - baoyx

### Features

- **(makefile)** re-add local-install target (lost during rebase) - ([2cdcf51](https://github.com/byx-darwin/gitflow-cli/commit/2cdcf512fae9c5ed8c2ec8b015cfc997254e88ee)) - baoyx
- add --repo parameter to issue create command (#51) - ([86978ba](https://github.com/byx-darwin/gitflow-cli/commit/86978ba213bd075e5a985b14b1960e6cc3608615)) - mc-ai

### Miscellaneous Chores

- **(skill)** apply lint-driven sync to gitflow-pr SKILL.md - ([92ace82](https://github.com/byx-darwin/gitflow-cli/commit/92ace821e4bba68185b9251a616233f50b486490)) - baoyx
- update Homebrew formula to v0.4.0 - ([a3b08b3](https://github.com/byx-darwin/gitflow-cli/commit/a3b08b3cf9952dfc70bd8a4bc7aab358b933597d)) - github-actions[bot]
- release v0.5.0 - ([93043b1](https://github.com/byx-darwin/gitflow-cli/commit/93043b19b9fba16e4ba85019cff849039b21281f)) - baoyx
- update CHANGELOG.md - ([984acef](https://github.com/byx-darwin/gitflow-cli/commit/984acefb22aa47f62770f1b6dd56bf726c935eb8)) - baoyx

### Other

- resolve conflicts with main — keep Superpowers format - ([2854830](https://github.com/byx-darwin/gitflow-cli/commit/28548309e1f236744e086976e2b6a3683f580d9f)) - baoyx

### Refactoring

- **(skill)** rewrite gitflow-pr-inline-review to Superpowers template - ([cdf07ec](https://github.com/byx-darwin/gitflow-cli/commit/cdf07ec5cc5baecd6f643879de2aa395f6110ee5)) - baoyx
- **(skill)** gitflow-security-check — conform to Superpowers template - ([ccda568](https://github.com/byx-darwin/gitflow-cli/commit/ccda568f20008302f662fc361d506c1443ad0890)) - baoyx
- **(skill)** rewrite gitflow-review to Superpowers template (#39) - ([f9c2dc7](https://github.com/byx-darwin/gitflow-cli/commit/f9c2dc714b330064ea8f15e16b90c6ea5776d395)) - baoyx
- **(skill)** gitflow-quality — conform to Superpowers template (#35) - ([00e41fc](https://github.com/byx-darwin/gitflow-cli/commit/00e41fcbc926a09733ce8b89174b9ba8bcac776d)) - baoyx
- **(skill)** gitflow-pr-apply-feedback — conform to Superpowers template (#33) - ([908f4fe](https://github.com/byx-darwin/gitflow-cli/commit/908f4feeebdbfb4a89844e4910fab3385fa273a1)) - baoyx
- **(skill)** gitflow-precommit — conform to Superpowers template (#24) - ([32eed77](https://github.com/byx-darwin/gitflow-cli/commit/32eed778a7a04b527abd0c6d643ac519a98a2985)) - baoyx
- **(skill)** rewrite gitflow-release-helper per template - ([ba1e699](https://github.com/byx-darwin/gitflow-cli/commit/ba1e6999cb4debea897e3fcc5e65a479a79334dc)) - baoyx
- **(skill)** gitflow-release — conform to Superpowers template (#18) - ([bc39f98](https://github.com/byx-darwin/gitflow-cli/commit/bc39f98f503588ed87b3a46578d83e10d9517eb1)) - baoyx
- **(skill)** gitflow-regression — conform to Superpowers template (#25) - ([9fb05fb](https://github.com/byx-darwin/gitflow-cli/commit/9fb05fba654a06d9a0bf89e407d85de9b0727797)) - baoyx
- **(skill)** compress gitflow-workflow from 1725 to 498 words per template - ([fa1bbd2](https://github.com/byx-darwin/gitflow-cli/commit/fa1bbd22c0d87e010801de3fe560793cae5a0318)) - baoyx
- **(skill)** compress gitflow-release-helper to 480 words - ([c6b3dc2](https://github.com/byx-darwin/gitflow-cli/commit/c6b3dc22fd8ca2282ade7d638a9a86808eedad05)) - baoyx
- **(skill)** gitflow-label-milestone — conform to Superpowers template (#17) - ([ebd84e7](https://github.com/byx-darwin/gitflow-cli/commit/ebd84e73ad222ecb2d5f62471b7f645a1ccc333d)) - baoyx
- **(skill)** gitflow-autoreport-bug — conform to Superpowers template - ([d6a567c](https://github.com/byx-darwin/gitflow-cli/commit/d6a567c38e5ef8b5df0d6cff06e7ba317ccc4a2a)) - baoyx
- **(skill)** gitflow-issue-triage — conform to Superpowers template - ([86f36f9](https://github.com/byx-darwin/gitflow-cli/commit/86f36f95f367f911d23a0437cec2c8a0d43f119b)) - baoyx
- **(skill)** gitflow-repo — conform to Superpowers template - ([253dd09](https://github.com/byx-darwin/gitflow-cli/commit/253dd09071625a4f1d4a17e81e5e92c74dc36466)) - baoyx
- **(skill)** gitflow-pipeline-analyzer — conform to Superpowers template - ([88c1a96](https://github.com/byx-darwin/gitflow-cli/commit/88c1a96c90021aec47103ff8ce504041b23d4ff8)) - baoyx
- **(skill)** gitflow-pr, gitflow-pr-create — conform to Superpowers template (#27) - ([07ad0e3](https://github.com/byx-darwin/gitflow-cli/commit/07ad0e3331e3e7e315a8f0cf477f4b46a9fa25e3)) - baoyx
- **(skill)** gitflow-pr-review, gitflow-issue-review, gitflow-pr-create sync — conform to Superpowers template (#27) - ([352a561](https://github.com/byx-darwin/gitflow-cli/commit/352a561c7bf05824e2cfa9f3dfbf374e7a459d12)) - baoyx
- **(skill)** gitflow-issue-review — conform to Superpowers template (#33) - ([b82cc58](https://github.com/byx-darwin/gitflow-cli/commit/b82cc58fd10cce21c53d909ba04526d93f8cddd4)) - baoyx
- **(skill)** gitflow-pr-review — conform to Superpowers template (#34) - ([f7428fa](https://github.com/byx-darwin/gitflow-cli/commit/f7428fab66bf77c0fd690b478167374044056a47)) - baoyx
- **(skill)** rewrite gitflow-issue per template with boundaries, flowchart, tests - ([e465bc9](https://github.com/byx-darwin/gitflow-cli/commit/e465bc95259c668f8c8852bdce325bff7fc25eb3)) - baoyx
- **(skill)** gitflow-pr child skills — add delegation model + word-count compliance - ([19a552a](https://github.com/byx-darwin/gitflow-cli/commit/19a552aa344212face9551cf20af080c32d8a7a0)) - baoyx
- **(skill)** gitflow-issue-create — conform to Superpowers template (#26) - ([53f1b8d](https://github.com/byx-darwin/gitflow-cli/commit/53f1b8d2fc630d49424ca45e9a260bf6471d530f)) - baoyx
- **(skill)** gitflow-label-stats — conform to Superpowers template - ([bc97bea](https://github.com/byx-darwin/gitflow-cli/commit/bc97bea547d07ce3ecf7f02b5b13d5e8c37f7707)) - baoyx
- **(skill)** compress gitflow-repo-onboarding 968→497 words - ([c79d544](https://github.com/byx-darwin/gitflow-cli/commit/c79d5446320d1667d3dc0107ff9b9cc39b45c0fb)) - baoyx
- **(skill)** gitflow-weekly-report — conform to Superpowers template with no-fabrication and no-performance-evaluation boundaries (#23) - ([2008b53](https://github.com/byx-darwin/gitflow-cli/commit/2008b5377d775851fc9bc4ca8d2052e934633d86)) - baoyx
- **(skill)** gitflow-auth — conform to Superpowers template with token safety boundaries (#15) - ([606d356](https://github.com/byx-darwin/gitflow-cli/commit/606d356d858e8b41b07ab3614c7210b194c0119b)) - baoyx
- **(skill)** gitflow-commit — conform to Superpowers template (#16) - ([8b6a43a](https://github.com/byx-darwin/gitflow-cli/commit/8b6a43a25f38fc8da0420fe3433350e8af71b5cb)) - baoyx
- **(skill)** gitflow-label-stats — compress to 444 words with Mermaid flowchart - ([c3bcdc5](https://github.com/byx-darwin/gitflow-cli/commit/c3bcdc526d53698ce8bef3fbfc95d9688fa30f6f)) - baoyx
- Phase 2 P1 completion — compress all 26 skills to ≤500 words with full section coverage - ([855ce58](https://github.com/byx-darwin/gitflow-cli/commit/855ce58ca2735fd9d69b9a1c7ee22694eb8057fb)) - baoyx
- Phase 3 P2 stress tests + 4 Mermaid flowcharts - ([402ffa6](https://github.com/byx-darwin/gitflow-cli/commit/402ffa68695c5608f4a55ba6feb3f4ae080ba8ab)) - baoyx

### Tests

- add weekly-report test scenarios and externalize template - ([cb94ba4](https://github.com/byx-darwin/gitflow-cli/commit/cb94ba432ec6720e05eadf96c35e90f5cb5f6de2)) - baoyx

---
## [0.4.0](https://github.com/byx-darwin/gitflow-cli/compare/v0.2.0..v0.4.0) - 2026-07-06

### Bug Fixes

- skills source dir now resolves relative to binary, not cwd - ([61108d5](https://github.com/byx-darwin/gitflow-cli/commit/61108d56ab518a70c25a706d624b6f2327434951)) - baoyx
- collapse nested if-let in skills_source_dir (clippy) - ([e0d6f06](https://github.com/byx-darwin/gitflow-cli/commit/e0d6f06a388080b0af7a1a8dbcf62d336abd4e6b)) - baoyx
- skills install now works outside git repo and project root - ([f943942](https://github.com/byx-darwin/gitflow-cli/commit/f943942debada5675588ee578cce239480652116)) - baoyx
- extract shared test helper and suppress clippy warnings in test-only code - ([8fdbacb](https://github.com/byx-darwin/gitflow-cli/commit/8fdbacb84fbf0c0130d65fbfdf3a1b5fdff5f2aa)) - baoyx
- subagent-dev must be required in fast mode - ([ccd7f18](https://github.com/byx-darwin/gitflow-cli/commit/ccd7f1884f176253f127e7dfab053ec9d494088b)) - baoyx

### Miscellaneous Chores

- **(deps)** upgrade crossbeam-epoch to 0.9.20 (RUSTSEC-2026-0204) - ([01d07d3](https://github.com/byx-darwin/gitflow-cli/commit/01d07d3ca49826a608d51650bfa2fc19aa31a024)) - baoyx
- **(release)** remove per-crate README replacement (uses root README) - ([13c5e35](https://github.com/byx-darwin/gitflow-cli/commit/13c5e35858a9e100a3b29c73437a668199953384)) - baoyx
- update Homebrew formula to v0.2.0 - ([c93fd73](https://github.com/byx-darwin/gitflow-cli/commit/c93fd73cbe49b860aff9041b15a095771a630a08)) - github-actions[bot]
- backup SKILL.md before refactoring - ([3e58f86](https://github.com/byx-darwin/gitflow-cli/commit/3e58f86c4d2610fb696a673b7cf23426c2215ac7)) - baoyx
- remove backup file before release - ([0e68d95](https://github.com/byx-darwin/gitflow-cli/commit/0e68d95fc453771cb2d47948ebe4ccf745258ef3)) - baoyx
- untrack .claude/settings.json, add version badge, restore release config - ([2eb31cf](https://github.com/byx-darwin/gitflow-cli/commit/2eb31cf044c0b21090fc35b1b4697870aee99707)) - baoyx
- release v0.4.0 - ([c060beb](https://github.com/byx-darwin/gitflow-cli/commit/c060beb6568cf5430ce1e76d423c881bf0391329)) - baoyx

### Other

- consolidate std imports in build.rs - ([62a4ef9](https://github.com/byx-darwin/gitflow-cli/commit/62a4ef9bc12a59b9f96b058cc2e3f4a206a52fa9)) - baoyx
- fix line formatting in skills.rs - ([eb58afd](https://github.com/byx-darwin/gitflow-cli/commit/eb58afd022534bb1e825f69507e41f2bcf75003d)) - baoyx

### Tests

- add SKILL.md structure verification tests for all 4 workflow phases - ([0b6b5b8](https://github.com/byx-darwin/gitflow-cli/commit/0b6b5b87067beb77c10906fc39a5479e04340192)) - baoyx
- add workflow modes structure verification tests - ([a3744f7](https://github.com/byx-darwin/gitflow-cli/commit/a3744f7f55e3b5f0ed8e1d7efd7ec9a4e66de18f)) - baoyx

---
## [0.2.0] - 2026-07-06

### Bug Fixes

- **(ci)** harden smoke test skip patterns for gitlab/gitcode/pipeline - ([5abecb0](https://github.com/byx-darwin/gitflow-cli/commit/5abecb0745a7336c8dc5afe0ca6868a7aba867f5)) - baoyx
- **(cli)** wrap command output in CliOutput envelope and remove dead libc dep (#1) - ([82f7efc](https://github.com/byx-darwin/gitflow-cli/commit/82f7efcf5060c6161056451cef31970ba2e5e3e5)) - baoyx
- **(cli)** use standard zsh site-functions dir for completions install - ([63e5167](https://github.com/byx-darwin/gitflow-cli/commit/63e51672551d9bb22546307a6ff3ac337dfe813d)) - baoyx
- **(cli)** use dirs::home_dir() for cross-platform completions install - ([22fd215](https://github.com/byx-darwin/gitflow-cli/commit/22fd215ea45682ac9fdc7418ede1096ebc9b7d1c)) - baoyx
- **(cli)** improve prerequisite error messages with install hints - ([ea2fa18](https://github.com/byx-darwin/gitflow-cli/commit/ea2fa18f851e5ccc334b8a15f6ae2d4f992ec910)) - baoyx
- **(cli)** resolve clippy pedantic warnings in CI - ([c8a59da](https://github.com/byx-darwin/gitflow-cli/commit/c8a59daf3dc09d1b95716c68f8254104a038224f)) - baoyx
- **(core)** fix Label serde policy and revert out-of-scope changes (#1) - ([836c974](https://github.com/byx-darwin/gitflow-cli/commit/836c974a3c3ec23b1ba1b5fb541202642205853e)) - baoyx
- **(core)** change UserSummary.id from u64 to String for GitHub compat - ([4ee5184](https://github.com/byx-darwin/gitflow-cli/commit/4ee5184df9a69b68596853e8cc8afe0a501c8ad1)) - baoyx
- **(gitcode)** use platform-conditional binary name gc/gitcode per official docs - ([7c3516e](https://github.com/byx-darwin/gitflow-cli/commit/7c3516e35f8364165e641b1ee95453973a8a2075)) - baoyx
- **(gitcode)** support gitcode CLI with version subcommand and pip install paths - ([d6d3f9e](https://github.com/byx-darwin/gitflow-cli/commit/d6d3f9e2bc658dd8b58f9477b76ea24349b00550)) - baoyx
- **(gitcode)** use gitcode CLI natively with correct flags and JSON mapping - ([4041085](https://github.com/byx-darwin/gitflow-cli/commit/40410855c9fd38f50210a40196d7ede71f4952c5)) - baoyx
- **(github)** add missing args in pr close and debug log in issue reopen - ([495226b](https://github.com/byx-darwin/gitflow-cli/commit/495226b5df47bc284246cf042c7c8aa3fd6b8825)) - baoyx
- **(github)** add missing chrono dependency - ([22cadab](https://github.com/byx-darwin/gitflow-cli/commit/22cadab0c2823d68146e6308c243f43702a011ca)) - baoyx
- **(github)** count all failure types in pipeline report (#4) - ([c90c816](https://github.com/byx-darwin/gitflow-cli/commit/c90c8161e7d50b21d712d0b03fd1c4a81e971bd5)) - baoyx
- **(gitlab)** use "closed" state filter for MR list to include all closed MRs (#4) - ([4738dd6](https://github.com/byx-darwin/gitflow-cli/commit/4738dd644aad0b6cfacf230589cd84f448b3b68e)) - baoyx
- **(scripts)** prevent settings.json overwrite and fix dead code in install.sh (#5) - ([b9057ac](https://github.com/byx-darwin/gitflow-cli/commit/b9057ac2d946f5bb4015902d39a894511ca2dd2a)) - baoyx
- **(skills)** address review findings for _common.sh (#5) - ([78d1e12](https://github.com/byx-darwin/gitflow-cli/commit/78d1e12c6048f0f91d9316008ceeb4d0fc661dc5)) - baoyx
- **(skills)** align workflow quality checks with design spec (#5) - ([2997ac7](https://github.com/byx-darwin/gitflow-cli/commit/2997ac7a4fff5b07af8e92adc395c8fcce0bb275)) - baoyx
- **(skills)** improve quality gate coverage commands and env var support (#5) - ([972fea9](https://github.com/byx-darwin/gitflow-cli/commit/972fea9d08cf5cefdb1ca76c10bf7330c6ae0b42)) - baoyx
- **(skills)** add failed.log write path and gitflow CLI guard to autoreport (#5) - ([d7f2da4](https://github.com/byx-darwin/gitflow-cli/commit/d7f2da465bf0d9855c06d9412c1be807d27c36cf)) - baoyx
- remove duplicate bans.deny table in deny.toml - ([8d8e520](https://github.com/byx-darwin/gitflow-cli/commit/8d8e5200674e00c07383a5a8bfe7c586bb08097b)) - baoyx
- use GITCODE_TOKEN as the only gitcode auth env var - ([2f9ec58](https://github.com/byx-darwin/gitflow-cli/commit/2f9ec58d51d4b1af593e43b5dbbee31d4c70ddd1)) - baoyx
- use absolute path for auto-report-bug hook - ([df9f5f2](https://github.com/byx-darwin/gitflow-cli/commit/df9f5f266161a34b12b3aee87704e5640af012b5)) - baoyx
- use nested hooks format for Stop Hook config - ([e6cc606](https://github.com/byx-darwin/gitflow-cli/commit/e6cc6061ad9d785cfd777ec051c51b8dd79fce47)) - baoyx
- correct pending.json field names to match Rust ErrorReport struct - ([6709c84](https://github.com/byx-darwin/gitflow-cli/commit/6709c84ae8662d0a0d312ffd2ef34e9f8777e639)) - baoyx
- address final review findings (spec staleness, dead code, matcher rationale) - ([b5f8a80](https://github.com/byx-darwin/gitflow-cli/commit/b5f8a80937567c8c36c15f190cff60d127f720f6)) - baoyx
- remove unused licenses from deny.toml + add pre-push hooks - ([bf62e25](https://github.com/byx-darwin/gitflow-cli/commit/bf62e2538a55de8f095369a1f59dab796a59860d)) - baoyx
- revert wildcards=allow + add CLAUDE.md rule to protect config files - ([8949277](https://github.com/byx-darwin/gitflow-cli/commit/89492772dc82a69cd60327f3fa0df66c75ec9578)) - baoyx
- use wildcards=warn for workspace dependency compatibility - ([ae351f6](https://github.com/byx-darwin/gitflow-cli/commit/ae351f60eb02a167570b1e11c2d640bf82ede8c6)) - baoyx
- resolve pre-existing clippy warnings to pass CI - ([3a59d8c](https://github.com/byx-darwin/gitflow-cli/commit/3a59d8c140ea3ed8bff31dc7a671f4151dc6504d)) - baoyx
- mark shell completion tests as Unix-only - ([5463fe6](https://github.com/byx-darwin/gitflow-cli/commit/5463fe64e3e46288b3c267177093c70ab9acf8a8)) - baoyx
- mark hook uninstall tests as Unix-only - ([9d21e91](https://github.com/byx-darwin/gitflow-cli/commit/9d21e9197d577f3bbc086c7e9b1c2aa2ca4a97cf)) - baoyx
- use platform-aware absolute path in SafePath test - ([8579ca1](https://github.com/byx-darwin/gitflow-cli/commit/8579ca164329ebb1b2293fd7005fed24e4619559)) - baoyx
- remove crates.io publish (not yet registered on crates.io) - ([a869c64](https://github.com/byx-darwin/gitflow-cli/commit/a869c6446c06cc9973cd1e6252bb39d1a37db094)) - baoyx

### Documentation

- add Superpowers integration guide (#5) - ([12cb622](https://github.com/byx-darwin/gitflow-cli/commit/12cb62257de430c4335ea42e53e3557b6e1a297b)) - baoyx
- update index with integration guide (#5) - ([fbc1e8e](https://github.com/byx-darwin/gitflow-cli/commit/fbc1e8e06b53f4bcc9edf637ac62e58953151852)) - baoyx
- add Phase 2/4/5 implementation plans - ([8e1d8e9](https://github.com/byx-darwin/gitflow-cli/commit/8e1d8e95b7626bb4020d1c201040042afef096f7)) - baoyx
- rewrite README with workflow-oriented structure - ([7df6f5f](https://github.com/byx-darwin/gitflow-cli/commit/7df6f5fdacc77ddb931521868754357e63c44861)) - baoyx
- remove stale 'planned' markers from gitflow-workflow - ([e3fe89b](https://github.com/byx-darwin/gitflow-cli/commit/e3fe89b31af3da8afde2f04051b5deca69ef4ffb)) - baoyx
- add platform support section to README - ([26d0eec](https://github.com/byx-darwin/gitflow-cli/commit/26d0eecac2b1bfb9be364b0b382bce4fd87854d6)) - baoyx
- note GitHub Enterprise and GitLab self-hosted support - ([95f15e9](https://github.com/byx-darwin/gitflow-cli/commit/95f15e98043e7b45b9ca393f6000ac8416e0fb64)) - baoyx
- replace all gitflow command references with gitflow-cli in README - ([4a677c5](https://github.com/byx-darwin/gitflow-cli/commit/4a677c5f3a8d969d273a826a961fd27cfe5b4b59)) - baoyx
- replace gitflow with gitflow-cli in all skill files - ([9981bd4](https://github.com/byx-darwin/gitflow-cli/commit/9981bd4ecf133e2eda81bf022d546ed747c39257)) - baoyx
- fix gitflow-review and gitflow-workflow descriptions to use gitflow-cli - ([1311c4d](https://github.com/byx-darwin/gitflow-cli/commit/1311c4d2cd50f454cba4459f8f7d2602260d91bf)) - baoyx
- add hook config format fix and report-bug toggle design - ([fa01ffc](https://github.com/byx-darwin/gitflow-cli/commit/fa01ffc035e7a710a2411fc8995de448116d5344)) - baoyx
- add implementation plan for hook config format fix - ([e6fb17e](https://github.com/byx-darwin/gitflow-cli/commit/e6fb17ecda5efe19e4973ac87acbc1981e3be961)) - baoyx

### Features

- **(cli)** add native CLI prerequisite checker (#1) - ([70439b2](https://github.com/byx-darwin/gitflow-cli/commit/70439b28eeecf1b40486c57dde704131898ee885)) - baoyx
- **(cli)** extend CLI structure with platform detection (#1) - ([92ecaa1](https://github.com/byx-darwin/gitflow-cli/commit/92ecaa12fdc3133ef4ab54e9eade9f490b56ddeb)) - baoyx
- **(cli)** implement gitflow issue create/list/view commands (#1) - ([da7fc82](https://github.com/byx-darwin/gitflow-cli/commit/da7fc8240d07cc11da72c212c617fb4ed72fd262)) - baoyx
- **(cli)** implement gitflow pr create/list/view commands (#1) - ([c0bbc08](https://github.com/byx-darwin/gitflow-cli/commit/c0bbc08425666f527189c2ed4f6bb7510e97e7ac)) - baoyx
- **(cli)** add error auto-report module and Stop Hook (#1) - ([3c9f8b4](https://github.com/byx-darwin/gitflow-cli/commit/3c9f8b4887576652a0f057bda91beaa62354064f)) - baoyx
- **(cli)** extend issue and pr commands with full operation set (#3) - ([34b1a6f](https://github.com/byx-darwin/gitflow-cli/commit/34b1a6f6ef977891395e443d17fb03f2b710d074)) - baoyx
- **(cli)** add release, review, and auth commands (#3) - ([15aa11b](https://github.com/byx-darwin/gitflow-cli/commit/15aa11bf043c3ae131615c8fac879a91c25ecce3)) - baoyx
- **(cli)** add label, milestone, and commit commands (#3) - ([1d495fb](https://github.com/byx-darwin/gitflow-cli/commit/1d495fb5e57760faab5e4225f57dd76f0b17bbbc)) - baoyx
- **(cli)** add multi-platform dispatch and pipeline commands (#4) - ([6f4f7d9](https://github.com/byx-darwin/gitflow-cli/commit/6f4f7d9b1f069241e23c52279fac0851e9964321)) - baoyx
- **(cli)** enhance shell completions with --install/--uninstall flags (#6) - ([140b1fa](https://github.com/byx-darwin/gitflow-cli/commit/140b1fa9e7c8245acc21d5c4ffd187d01d967d05)) - baoyx
- **(cli)** add --output text human-friendly formatting (#6) - ([2c8a11f](https://github.com/byx-darwin/gitflow-cli/commit/2c8a11f64509a2d4350047bd0c5044800af9b5de)) - baoyx
- **(cli)** implement skills install/list/uninstall command and complete community docs (#6) - ([0c8ef6a](https://github.com/byx-darwin/gitflow-cli/commit/0c8ef6a2c49e0a3a647a95e85240965ac18a2699)) - baoyx
- **(cli)** add multi-agent and multi-target support to skills install - ([398ec33](https://github.com/byx-darwin/gitflow-cli/commit/398ec330022ffb174383c93a7ff55340c726efcf)) - baoyx
- **(cli)** add Gemini and Copilot back to agent platform support - ([cbf39b0](https://github.com/byx-darwin/gitflow-cli/commit/cbf39b0cf1d3c66609060259c3b5727155ec5fe8)) - baoyx
- **(cli)** agent-parseable prerequisite errors with auto-install and login hints - ([621d16f](https://github.com/byx-darwin/gitflow-cli/commit/621d16f410433a5dd6a922283cdbca6dbd8eedb2)) - baoyx
- **(core)** add JSON output types and public re-exports (#1) - ([f31eafc](https://github.com/byx-darwin/gitflow-cli/commit/f31eafc8b1dc1e1ffffdee09114abb42198ce20f)) - baoyx
- **(core)** add domain types and Platform error variant (#1) - ([b90e0f4](https://github.com/byx-darwin/gitflow-cli/commit/b90e0f446e6a0d6b8933c3f03622922c26a23be3)) - baoyx
- **(core)** add Platform enum with remote URL detection (#1) - ([f7f7d5c](https://github.com/byx-darwin/gitflow-cli/commit/f7f7d5c069af09bdcf7e27a12b1e3f24bc96533e)) - baoyx
- **(core)** define IssueProvider and PrProvider traits (#1) - ([e69402c](https://github.com/byx-darwin/gitflow-cli/commit/e69402c5bcb950ae182debfc8a910ee351326b32)) - baoyx
- **(core)** add JSON output types and public re-exports (#1) - ([9ff806d](https://github.com/byx-darwin/gitflow-cli/commit/9ff806d5b95155b2fcebb11acfda9f55271344c8)) - baoyx
- **(core)** extend IssueProvider and PrProvider with full operation set (#3) - ([7588df1](https://github.com/byx-darwin/gitflow-cli/commit/7588df10f73e0dd7e60696fad410cadca7761a6e)) - baoyx
- **(core)** add ReleaseProvider, ReviewProvider, and AuthProvider traits (#3) - ([1225a3b](https://github.com/byx-darwin/gitflow-cli/commit/1225a3bd62a7da3a338286c7197f1a6e345acbd1)) - baoyx
- **(core)** add LabelProvider, MilestoneProvider, and CommitProvider traits (#3) - ([dffcaac](https://github.com/byx-darwin/gitflow-cli/commit/dffcaac9d6e96ff98ce8d8c5f15e45144f7a04c2)) - baoyx
- **(core)** add PipelineProvider trait for CI/CD pipeline analysis (#4) - ([e467853](https://github.com/byx-darwin/gitflow-cli/commit/e467853fef888b1e6f79d8cd57f3c093b46f722c)) - baoyx
- **(gitcode)** add full GitCode platform support crate (#4) - ([973d4c3](https://github.com/byx-darwin/gitflow-cli/commit/973d4c3a707647658b5dd1a35ceafb1bcc3a75d6)) - baoyx
- **(github)** add GitHubIssueProvider and GitHubPrProvider (#1) - ([550b1cd](https://github.com/byx-darwin/gitflow-cli/commit/550b1cdca64adeb95d9bcb1f452ac885fcbc82b3)) - baoyx
- **(github)** extend GitHubIssueProvider and GitHubPrProvider with full operations (#3) - ([6b783b0](https://github.com/byx-darwin/gitflow-cli/commit/6b783b050974c181c7e4d9a9805fb849cddd9e64)) - baoyx
- **(github)** add GitHubReleaseProvider and GitHubReviewProvider (#3) - ([8747a2c](https://github.com/byx-darwin/gitflow-cli/commit/8747a2c5b1abffacfe9d206a3324a43e8fe7b454)) - baoyx
- **(github)** add Auth, Label, Milestone, and Commit providers (#3) - ([e7e44ee](https://github.com/byx-darwin/gitflow-cli/commit/e7e44ee0a544c6be7d974f0eda8313fbc8e927fc)) - baoyx
- **(github)** add GitHubPipelineProvider (#4) - ([879d04f](https://github.com/byx-darwin/gitflow-cli/commit/879d04f1a943c655fa28ef437b2d06b22ca14c6c)) - baoyx
- **(gitlab)** add full GitLab platform support crate (#4) - ([11379a0](https://github.com/byx-darwin/gitflow-cli/commit/11379a0895fa1201201f14d9aaa4ef91aa8f2504)) - baoyx
- **(hooks)** enhance auto-report-bug hook with interactive detection (#5) - ([3d3426a](https://github.com/byx-darwin/gitflow-cli/commit/3d3426ae54c438c4e0a8e7dff0cce1b4224d40c8)) - baoyx
- **(skills)** add core command layer skills for all resource types (#3) - ([c2d09d0](https://github.com/byx-darwin/gitflow-cli/commit/c2d09d026eda69a20f8880a7be690ee2a54c4939)) - baoyx
- **(skills)** add workflow layer skills (#3) - ([79cf890](https://github.com/byx-darwin/gitflow-cli/commit/79cf89004496a92910475ed066d5bb5f2795219a)) - baoyx
- **(skills)** add shared shell function library with error reporting (#5) - ([fecbffa](https://github.com/byx-darwin/gitflow-cli/commit/fecbffa41fa42186204e9f7375cfd88644fc757a)) - baoyx
- **(skills)** add gitflow-workflow orchestration skill (#5) - ([ee33027](https://github.com/byx-darwin/gitflow-cli/commit/ee33027ffcf0927e84fbf7d4d4516a056c230f74)) - baoyx
- **(skills)** add gitflow-quality gate skill (#5) - ([d123b0d](https://github.com/byx-darwin/gitflow-cli/commit/d123b0d48727cf750a5546d9e920d9455b726bc3)) - baoyx
- **(skills)** add gitflow-autoreport-bug complete skill with deduplication (#5) - ([6982fd5](https://github.com/byx-darwin/gitflow-cli/commit/6982fd5078419c91f9dd26bb3c0bc455918732dc)) - baoyx
- **(skills)** add issue review, triage, inline review, feedback, and release helper skills (#6) - ([5b7a8ff](https://github.com/byx-darwin/gitflow-cli/commit/5b7a8ffa23fb6b791d36b8f9bddfa57e8c4df7a6)) - baoyx
- **(skills)** add pipeline analyzer, repo, precommit, regression, and label stats skills (#6) - ([d24f1f5](https://github.com/byx-darwin/gitflow-cli/commit/d24f1f5dac6046fa60b702493628e06dff50894c)) - baoyx
- add one-click install script (#5) - ([99c7ed9](https://github.com/byx-darwin/gitflow-cli/commit/99c7ed9ab780f8579da834ae208be24ba0a766b7)) - baoyx
- add Homebrew formula and GitHub Release workflow (#6) - ([5998b1c](https://github.com/byx-darwin/gitflow-cli/commit/5998b1c9653b224b9bb89f4ad5099a128aab3b04)) - baoyx
- integrate auto-report-bug hook into skills install - ([9ce8cf6](https://github.com/byx-darwin/gitflow-cli/commit/9ce8cf6a596037391a973cf57508ecce40d49a95)) - baoyx
- add --report-bug flag to toggle Stop Hook installation - ([108a85e](https://github.com/byx-darwin/gitflow-cli/commit/108a85ec8346048904bad5468570552ce89df088)) - baoyx
- add gitflow-weekly-report skill (ported from ncgo-code-skills) - ([d9cae88](https://github.com/byx-darwin/gitflow-cli/commit/d9cae883e6f7c692e033e6910fb58846d382f7d6)) - baoyx
- merge auto-report-bug with auth cache, JSON validation, failed.log retry - ([4a99a5d](https://github.com/byx-darwin/gitflow-cli/commit/4a99a5d1a225f409e01029851f51c244335ef42d)) - baoyx
- add sync-readme-check hook + register in settings.json - ([32a6bb7](https://github.com/byx-darwin/gitflow-cli/commit/32a6bb764fababc066500c083fb6aee85595c60f)) - baoyx
- Homebrew formula uses pre-built release binaries (architecture-aware) - ([fe448dd](https://github.com/byx-darwin/gitflow-cli/commit/fe448dd2600fa25e2dba5b64a01aa97fd810292c)) - baoyx

### Miscellaneous Chores

- **(github)** fix clippy pedantic warnings in pipeline.rs - ([c54b8a3](https://github.com/byx-darwin/gitflow-cli/commit/c54b8a359480f3af1a8b1c5d4988192303475c61)) - baoyx
- add Phase 1 prerequisites — Rust 1.96.0, design spec, plan file - ([15a67fe](https://github.com/byx-darwin/gitflow-cli/commit/15a67fe003bda078d29b622494c70e273f6d1466)) - baoyx
- associate plan with Issue #1 - ([c8e777a](https://github.com/byx-darwin/gitflow-cli/commit/c8e777a6c19b76e2f132e7fc330bd2e4a450ea94)) - baoyx
- final lint and formatting pass for Phase 1 (#1) - ([6e12a23](https://github.com/byx-darwin/gitflow-cli/commit/6e12a23b85abb5fab8c02d8dff7dc7820443c6c7)) - baoyx
- final lint and formatting pass for Phase 3 (#4) - ([363c67d](https://github.com/byx-darwin/gitflow-cli/commit/363c67d9f0e7b964c1e7ebbb922621913eb1cb54)) - baoyx
- update Makefile with install targets and specs index (#5) - ([0629a09](https://github.com/byx-darwin/gitflow-cli/commit/0629a09b33cbbbc4ba92ccede863402e1fc065b5)) - baoyx
- install pre-commit hooks and fix typos allowlist - ([02d4fd0](https://github.com/byx-darwin/gitflow-cli/commit/02d4fd0ba550c5316ea2eda21ef45b5077fd6fa0)) - baoyx
- remove .superpowers/ and add to .gitignore - ([39e257b](https://github.com/byx-darwin/gitflow-cli/commit/39e257bbd8316352b1b53eb29f44d7fa031eb79b)) - baoyx
- release v0.2.0 - ([ff93b98](https://github.com/byx-darwin/gitflow-cli/commit/ff93b98709afa4513915eb771775c1873b34d065)) - baoyx
- update CHANGELOG.md - ([05b6d1b](https://github.com/byx-darwin/gitflow-cli/commit/05b6d1b87af4f9d54f7d9aa46aa4d2b17d70af95)) - baoyx
- update Homebrew formula to v0.2.0 - ([ee13482](https://github.com/byx-darwin/gitflow-cli/commit/ee134824e56c72db5411073e4d7d0a92bcfe8d23)) - github-actions[bot]

### Other

- Initial commit - ([cf104d6](https://github.com/byx-darwin/gitflow-cli/commit/cf104d6d611aad24c33bb0cb3e09bf51d24cb645)) - mc-ai
- Merge branch 'worktree-agent-a64693338b83d2f05' - ([ffc518c](https://github.com/byx-darwin/gitflow-cli/commit/ffc518c469ac2fe1a9e38b61dec62566e87dcb23)) - baoyx
- Merge branch 'worktree-agent-a9b6077fd4a0f551d' - ([2b7d068](https://github.com/byx-darwin/gitflow-cli/commit/2b7d068258912860003071cf38cce0694a6b2583)) - baoyx
- enhance build pipeline and release config to match agent-proxy-rust - ([fb8c89a](https://github.com/byx-darwin/gitflow-cli/commit/fb8c89a84363cf6c0b2e1aaf822c54d3bcadd396)) - baoyx
- install nightly rustfmt component for cargo +nightly fmt check - ([816f619](https://github.com/byx-darwin/gitflow-cli/commit/816f61914fd3ce19fee933919294dea4e2095c73)) - baoyx
- add design spec and implementation plan - ([327a255](https://github.com/byx-darwin/gitflow-cli/commit/327a255f51b773e993b7390b43fbded89458fca3)) - baoyx
- add pre-commit as 6th quality gate step - ([8b5b390](https://github.com/byx-darwin/gitflow-cli/commit/8b5b39053467c66d512b1171a90dfaa709069025)) - baoyx
- add compliance checklists, --body-file rule, enforcement header - ([2c29cfb](https://github.com/byx-darwin/gitflow-cli/commit/2c29cfb1146ce5b074b6f71db45947cd5ef6ad64)) - baoyx
- add GitHub Actions workflows + update Makefile release pipeline - ([ad8f773](https://github.com/byx-darwin/gitflow-cli/commit/ad8f773a65adcee830faf9d8408316fc1b86e159)) - baoyx
- auto-update Homebrew formula on release - ([0c730f4](https://github.com/byx-darwin/gitflow-cli/commit/0c730f444e2b2af4e2b6069e4397bd29ba4c1933)) - baoyx
- install nightly rustfmt component in Lint job - ([65cdce5](https://github.com/byx-darwin/gitflow-cli/commit/65cdce5b76957132b74cc535d45fe31ea755d729)) - baoyx
- also trigger CI on tag push for CD gate - ([804d292](https://github.com/byx-darwin/gitflow-cli/commit/804d292b5aafd3661734edd447f569ec93b7a113)) - baoyx
- remove ci-gate job from CD workflow - ([347203d](https://github.com/byx-darwin/gitflow-cli/commit/347203d0aee226656613bea650db71d22e458428)) - baoyx
- fix Homebrew formula push auth in release workflow - ([c7738fa](https://github.com/byx-darwin/gitflow-cli/commit/c7738fa51fce1bd8097ece63023f86f3f2746c23)) - baoyx
- use Python to update Homebrew formula (sed was missing arm64 entries) - ([880ecf5](https://github.com/byx-darwin/gitflow-cli/commit/880ecf5462e61fd0f3c0139be6de99c89cfdaf32)) - baoyx

### Refactoring

- **(cli)** trim agent platforms to Claude/Codex/OpenCode - ([139a9a2](https://github.com/byx-darwin/gitflow-cli/commit/139a9a291e668a82f775d73884d6e7ce7000f41c)) - baoyx
- **(cli)** default to project-level skills install, -g for global - ([bfecadf](https://github.com/byx-darwin/gitflow-cli/commit/bfecadfd5da630537749049b4b53ca00e439730c)) - baoyx
- extract HOOK_CONFIG variable and use nested format - ([ff7d780](https://github.com/byx-darwin/gitflow-cli/commit/ff7d7803e24893cf719fbf68d827dbd8a9f7e0af)) - baoyx
- rename gitflow-cli-cli to gitflow-cli - ([31b531e](https://github.com/byx-darwin/gitflow-cli/commit/31b531eec10676a25a196e17d217a816bcdbe21b)) - baoyx
- rename gc to gitcode across entire codebase - ([fa5ca01](https://github.com/byx-darwin/gitflow-cli/commit/fa5ca01c0af9547031010ea2638e755db96137d8)) - baoyx

### Tests

- add Phase 1 smoke test script and integration tests (#1) - ([b48dfba](https://github.com/byx-darwin/gitflow-cli/commit/b48dfba0e1dc39ae3da292e05444db5e92e941c8)) - baoyx
- extend smoke test for Phase 2 commands (#3) - ([1bac8be](https://github.com/byx-darwin/gitflow-cli/commit/1bac8be3115289f2e2cdb6a0183da13c98001429)) - baoyx
- add multi-platform smoke test and CI matrix (#4) - ([1e6a152](https://github.com/byx-darwin/gitflow-cli/commit/1e6a1528dd7c58f259f83e9085412817dea29476)) - baoyx
- add failing tests for nested hook format - ([0b33fec](https://github.com/byx-darwin/gitflow-cli/commit/0b33fec1b9da9f70caf8cbfd150f3cd956f10479)) - baoyx
- verify uninstall_hook works with nested hook format - ([b25f4cd](https://github.com/byx-darwin/gitflow-cli/commit/b25f4cda39d53a775e7620e85f8827157b96e8e9)) - baoyx

<!-- generated by git-cliff -->
