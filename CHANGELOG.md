# Changelog

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

---
## [unreleased]

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
