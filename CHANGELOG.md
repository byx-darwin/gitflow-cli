# Changelog

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

---
## [2.0.0](https://github.com/byx-darwin/gitflow-cli/compare/v1.9.0..v2.0.0) - 2026-09-23

### Bug Fixes

- **(architecture-diagram)** final review fixes for Issue #331 (I3-I6, M1) - ([455d369](https://github.com/byx-darwin/gitflow-cli/commit/455d3695fd21224145eadc5e092db3a12b4e5e1b)) - baoyuexing
- **(cleanup)** honour --yes and confirm before bulk branch deletion - ([124c93f](https://github.com/byx-darwin/gitflow-cli/commit/124c93fce08bf07808bac9db0c8fa2b29000a4bf)) - baoyuexing
- **(docs)** correct dogfooding checklist command params to match actual CLI (#361) - ([74f8944](https://github.com/byx-darwin/gitflow-cli/commit/74f894448f7fee69a4d7fb840c6340f09a42b94b)) - baoyuexing
- **(docs)** resolve skill-conventions word-count contradiction, raise threshold to 650 - ([2ae1e26](https://github.com/byx-darwin/gitflow-cli/commit/2ae1e26430a0c8ca4338657c93e04b3c2d1f7a0b)) - baoyuexing
- **(e2e)** resolve gf from the workspace build instead of PATH - ([78a3ae4](https://github.com/byx-darwin/gitflow-cli/commit/78a3ae4507aeba3ca467dba7f883d3e94848b5f3)) - baoyuexing
- **(e2e)** noauth tests skip gracefully when gh/glab CLI not installed instead of failing misleadingly (#356) - ([041977b](https://github.com/byx-darwin/gitflow-cli/commit/041977b22f9488afe9625f4e8f404a0cebc64dee)) - baoyuexing
- **(e2e-core)** clear inherited GIT_DIR/GIT_WORK_TREE in scratch_repo_dir - ([9cd8f87](https://github.com/byx-darwin/gitflow-cli/commit/9cd8f87598cf476cba1263884cc2fa056199cb83)) - baoyuexing
- **(e2e-core,ci)** treat empty-string secrets as absent; scope ci.yml Test job - ([0e98357](https://github.com/byx-darwin/gitflow-cli/commit/0e983579b0708ed7cbca09fcb3bf3587173709c1)) - baoyuexing
- **(gf-quality)** Gate 3 judges total line coverage, N/A on no-op - ([47d69bf](https://github.com/byx-darwin/gitflow-cli/commit/47d69bf9e754f363c8bda94b5decb18da49ecbbb)) - baoyuexing
- **(gf-quality)** unify Rust coverage tool to cargo-llvm-cov - ([2cab9ca](https://github.com/byx-darwin/gitflow-cli/commit/2cab9ca663f39d634303430a741e6d86402ebe6f)) - baoyuexing
- **(gf-quality)** align 4 language layers to total line coverage - ([1c391dd](https://github.com/byx-darwin/gitflow-cli/commit/1c391dd784d76f201a8e51b25cf3baf3c9e20f42)) - baoyuexing
- **(gf-quality)** correct Bun coverage threshold in node.md - ([a1b2595](https://github.com/byx-darwin/gitflow-cli/commit/a1b2595303668633044fd48fdc63e24b534e9e17)) - baoyuexing
- **(gf-quality)** add missing references/ruby.md - ([ce243e8](https://github.com/byx-darwin/gitflow-cli/commit/ce243e895970ac961b7517add75a34e59fec2e0a)) - baoyuexing
- **(gf-quality)** align ruby.md structure with sibling references - ([a9b0497](https://github.com/byx-darwin/gitflow-cli/commit/a9b049771fa895e36943ce5eb16f44001a6f127e)) - baoyuexing
- **(gf-quality)** unnest bash fence inside markdown report template - ([7becc8f](https://github.com/byx-darwin/gitflow-cli/commit/7becc8f91921ed1443bac220b8ba9ddd3be6e979)) - baoyuexing
- **(gf-quality)** move ancestry-check instructions outside the literal report-template fence - ([0df9843](https://github.com/byx-darwin/gitflow-cli/commit/0df9843e9ddbd888d36041f4353da86e7d0f3dc8)) - baoyuexing
- **(gf-walkthrough)** remove dead code from check #3 awk script - ([53e6a52](https://github.com/byx-darwin/gitflow-cli/commit/53e6a524b5cf25cf4795a9a446be6849863e77d2)) - baoyuexing
- **(gf-walkthrough)** restore scope statement and ancestry contrast reasoning - ([450fce3](https://github.com/byx-darwin/gitflow-cli/commit/450fce3789fd59450d3d229239f17e4977b88ea3)) - baoyuexing
- **(gf-walkthrough)** final review fixes for diffstat, missing gate, self-narration - ([aa70fc7](https://github.com/byx-darwin/gitflow-cli/commit/aa70fc768d850d722fc1f02c98f5e06dba96e21f)) - baoyuexing
- **(gf-workflow)** compute worktree symlink depth dynamically instead of hardcoding ../../ - ([919d9cd](https://github.com/byx-darwin/gitflow-cli/commit/919d9cd1e14add45b0645b13aa43fa49e5747815)) - baoyuexing
- **(gf-workflow)** split worktree symlink depth formula for .cache/workflows vs .claude (#353) - ([663f921](https://github.com/byx-darwin/gitflow-cli/commit/663f9219cf234b7eb42be8b15ba3b607e24e669e)) - baoyuexing
- **(gf-workflow-batch)** address final review findings on #337 dependency resolution - ([434e53b](https://github.com/byx-darwin/gitflow-cli/commit/434e53b912bc408a203b7677a1ec9457718fcc12)) - baoyuexing
- **(gf-workflow-batch)** don't report --limit cutoff as blocked-on-dependencies (#374 review finding) - ([79be765](https://github.com/byx-darwin/gitflow-cli/commit/79be7655ff1e4ecc824dfa7dee6334766e75fb88)) - baoyuexing
- **(gitcode)** stop an unrelated `gc` binary from shadowing the GitCode CLI - ([8f74699](https://github.com/byx-darwin/gitflow-cli/commit/8f74699413353cc5530d0ae29412a8e8b0341f2e)) - baoyuexing
- **(gitcode)** inject env source into auth provider - ([b434642](https://github.com/byx-darwin/gitflow-cli/commit/b434642eb87da82896e83fa3df231d52fd9856e4)) - baoyuexing
- **(gitcode)** page through issue list instead of single-shot --limit - ([d0ae638](https://github.com/byx-darwin/gitflow-cli/commit/d0ae638a39d699efc1ed813c9ba8a25ba0977677)) - baoyuexing
- **(gitcode)** page through pr list instead of single-shot --limit - ([59bc215](https://github.com/byx-darwin/gitflow-cli/commit/59bc21526eec906138608a5f623fa6ce70821bfd)) - baoyuexing
- **(gitcode)** enable pagination for label and milestone list - ([ee2d6da](https://github.com/byx-darwin/gitflow-cli/commit/ee2d6da3352c6037a06d0aa1394b54194a4e29b7)) - baoyuexing
- **(gitcode)** fetch releases via api with pagination; drop GITCODE_DEFAULT_LIST_LIMIT - ([67045b4](https://github.com/byx-darwin/gitflow-cli/commit/67045b47d8a7e2109e7ad9c3f1cc03650b90dde9)) - baoyuexing
- **(gitcode)** map release api response via intermediate type; tighten query-string assertions - ([7cb55ea](https://github.com/byx-darwin/gitflow-cli/commit/7cb55eaa7a7a8dc237f43ed5dd05fe2e5141e7ce)) - baoyuexing
- **(gitcode)** correct pagination doc claim and cover short-page non-truncation - ([4cdcb01](https://github.com/byx-darwin/gitflow-cli/commit/4cdcb01786a524705dc44a1016054d80c30e9a09)) - baoyuexing
- **(gitcode)** release id/null tolerance + correct false degrade claim (F1) - ([6f4bc05](https://github.com/byx-darwin/gitflow-cli/commit/6f4bc054f73100b0c049ab28394bbe9aa2172b72)) - baoyuexing
- **(gitcode)** tolerate null release id / author login, fix doc over/under-claims (F1 round 2) - ([fa18fd5](https://github.com/byx-darwin/gitflow-cli/commit/fa18fd5df8dfd6f6aea461de44dc8c95e237586f)) - baoyuexing
- **(gitcode)** MilestoneApiResponse match upstream snake_case, stop silently zeroing due_on/issue counts (audit finding alongside #364) - ([3cbd2c2](https://github.com/byx-darwin/gitflow-cli/commit/3cbd2c23bc6c049cd994f2428f8d4e6b9cf39381)) - baoyuexing
- **(gitcode)** parse pure-date due_on format for milestones - ([da6c696](https://github.com/byx-darwin/gitflow-cli/commit/da6c6967b079961a8d06cc4ff8efb7c9cd257760)) - baoyuexing
- **(gitcode)** resolve pr create milestone against the overridden repo, not the provider default - ([7de3ca3](https://github.com/byx-darwin/gitflow-cli/commit/7de3ca3d2b312423648dac61409c813052ad27b3)) - baoyuexing
- **(gitcode)** pr close/reopen deserialize into full PrData via view() - ([6580e28](https://github.com/byx-darwin/gitflow-cli/commit/6580e282e808032ea85bf8c0065efaa42c73d703)) - baoyuexing
- **(gitcode)** issue close/reopen deserialize into full IssueData via view() - ([b91d653](https://github.com/byx-darwin/gitflow-cli/commit/b91d65385d1e975d00171e6625e795c9f2224d26)) - baoyuexing
- **(github)** gate pipeline report terminal state on run/job status, not conclusion presence - ([5051966](https://github.com/byx-darwin/gitflow-cli/commit/50519664c3c0fdf723d224c5b42d6b81ea3bb778)) - baoyuexing
- **(github)** inject env source into auth provider - ([68a7d66](https://github.com/byx-darwin/gitflow-cli/commit/68a7d66f57bd1c29394e90f57cd19cdcb2d26802)) - baoyuexing
- **(github)** MilestoneApiResponse match upstream snake_case, stop silently zeroing due_on/issue counts (#364) - ([727a0f0](https://github.com/byx-darwin/gitflow-cli/commit/727a0f0cc2442163f60dc4885220b55492ab4096)) - baoyuexing
- **(github)** wire milestone field into issue close/reopen response - ([d6ec877](https://github.com/byx-darwin/gitflow-cli/commit/d6ec87745c7979d07aa1bcd8c6381ecab92d1ea0)) - baoyuexing
- **(gitlab)** inject env source into auth provider - ([d84ee15](https://github.com/byx-darwin/gitflow-cli/commit/d84ee155d6f3ca25510d24cb36c8d3d2a2458a36)) - baoyuexing
- **(gitlab)** normalize label color to # prefix, fix help text (#372) - ([ae252b8](https://github.com/byx-darwin/gitflow-cli/commit/ae252b85bc8ca7038af1e49b3e95fbf466e4d288)) - baoyuexing
- **(gitlab)** parse per-host auth status, stop collapsing mixed hosts to unauthenticated (#362) - ([1777899](https://github.com/byx-darwin/gitflow-cli/commit/17778991baa1bb3f3f28e368eef2f6519e04410a)) - baoyuexing
- **(gitlab)** always pass --description to glab issue create, fixing no-body 100% failure (#375) - ([ea9d9c2](https://github.com/byx-darwin/gitflow-cli/commit/ea9d9c2d6904bc8b16e1e71b17146302aa0be375)) - baoyuexing
- **(gitlab)** address final review findings — edit() color normalization, host-parse false positive, trim whitespace (#372 #362) - ([087fe81](https://github.com/byx-darwin/gitflow-cli/commit/087fe816de425f1b5cd20a2897105f0216f440d8)) - baoyuexing
- **(gitlab)** resolve milestone --project against bare repo, not repo_target (which is only correct for --repo-flavored commands) - ([a181308](https://github.com/byx-darwin/gitflow-cli/commit/a181308963256faf6c753bf7e0a1b9ab90d081e8)) - baoyuexing
- **(gitlab)** milestone commands always use bare repo for --project - ([fdbd84a](https://github.com/byx-darwin/gitflow-cli/commit/fdbd84afc3cec97586453b0dd3ab8480f3f7c45d)) - baoyuexing
- **(issue)** paginate issue list to a cap and forward label filter - ([95fc7ef](https://github.com/byx-darwin/gitflow-cli/commit/95fc7ef724c91b192ad8307701d519678ac84196)) - baoyuexing
- **(issue)** paginate issue comments instead of returning only page one - ([01b1ae3](https://github.com/byx-darwin/gitflow-cli/commit/01b1ae39603996560435c50e38235ac0796b9488)) - baoyuexing
- **(issue)** reject mixed-case URLs in precheck input - ([650737b](https://github.com/byx-darwin/gitflow-cli/commit/650737b3e95a5c84ec0b76261c8f1fc7798cf71f)) - baoyuexing
- **(jev)** read macOS Keychain when env key is absent - ([d634c0a](https://github.com/byx-darwin/gitflow-cli/commit/d634c0a6ac730bc684c60a7b0e476279a3982e10)) - baoyuexing
- **(label)** paginate label and milestone lists, route them through runner - ([703ac37](https://github.com/byx-darwin/gitflow-cli/commit/703ac376421d1d3d9169d70a6ee2c02a75ed22ef)) - baoyuexing
- **(label)** guard truncated reads at CLI/gitlab layers, close test gaps - ([1caab51](https://github.com/byx-darwin/gitflow-cli/commit/1caab515a06f52b8a481c2ac2ba67b734dc643cd)) - baoyuexing
- **(list-pagination)** drop unneeded Clone bound, cap gitlab per_page, bound --limit - ([ec6796d](https://github.com/byx-darwin/gitflow-cli/commit/ec6796d800f59b07eb61282774368290548fbb86)) - baoyuexing
- **(makefile)** check-skills-drift now diffs content, not just directory presence (#363) - ([4428f37](https://github.com/byx-darwin/gitflow-cli/commit/4428f37b01df4227829896a5e5f6d463763faf10)) - baoyuexing
- **(makefile)** check-walkthrough-skill no longer drops adjacent [Measured] entries via getline (#355) - ([ef3954b](https://github.com/byx-darwin/gitflow-cli/commit/ef3954bd7c440cdf70e3d20f7e04285f9c30bf0a)) - baoyuexing
- **(makefile)** install-skills now syncs docs/ so ../../docs/... skill references resolve after install (#352) - ([80a65a4](https://github.com/byx-darwin/gitflow-cli/commit/80a65a4b0bb9de9dcbc06d67fdf2ec9e23ddef51)) - baoyuexing
- **(pagination)** close FINAL-review gaps in list-pagination branch - ([364b3a3](https://github.com/byx-darwin/gitflow-cli/commit/364b3a30fe86436dab06d1238acefb925cdcf1a4)) - baoyuexing
- **(pipeline)** exclude in-progress runs from GitHub report total_runs - ([f8e302e](https://github.com/byx-darwin/gitflow-cli/commit/f8e302ea1bb5d143c9fbc383ffee66e49cdfd432)) - baoyuexing
- **(pipeline)** exclude non-terminal pipelines from GitLab report total_runs - ([4930f42](https://github.com/byx-darwin/gitflow-cli/commit/4930f4238074134187a6fcbfd3598a2aeabfb4e3)) - baoyuexing
- **(pipeline)** redact alternate CI credential forms - ([8aa2af6](https://github.com/byx-darwin/gitflow-cli/commit/8aa2af69d50deba3686b0c55393f69b130029ee4)) - baoyuexing
- **(platforms)** use epoch for malformed GitHub timestamps - ([81d8873](https://github.com/byx-darwin/gitflow-cli/commit/81d8873cb0f53726980302127ba0ca1551d0abda)) - baoyuexing
- **(pr)** detect repo default branch instead of hardcoding "main" - ([7bdfedf](https://github.com/byx-darwin/gitflow-cli/commit/7bdfedf9e3ce9e100da5c7c5e830a915c10f1520)) - baoyuexing
- **(pr)** paginate pr list to a cap instead of inheriting CLI default - ([41ecd8d](https://github.com/byx-darwin/gitflow-cli/commit/41ecd8d7eeb72b6389dd30857841963a09c2db67)) - baoyuexing
- **(pr)** harden cleanup confirmation per review round 1 - ([a912e90](https://github.com/byx-darwin/gitflow-cli/commit/a912e907d744cbf708cf55c17286668f41752bc0)) - baoyuexing
- **(quality)** sync remaining normative docs to llvm-cov, wire link check - ([9250b58](https://github.com/byx-darwin/gitflow-cli/commit/9250b588927660ad9668212d4b07442e4039de52)) - baoyuexing
- **(release)** honour --limit in release list instead of discarding it - ([ac9fe12](https://github.com/byx-darwin/gitflow-cli/commit/ac9fe124d90628220fd9bee76f26e0eaf1b1b2a9)) - baoyuexing
- **(review)** preserve PR precheck evidence identity - ([f143498](https://github.com/byx-darwin/gitflow-cli/commit/f143498aa2fb9d4ddb59099cacdaca0dfd8d9ddb)) - baoyuexing
- **(review)** address website and smoke findings - ([e202268](https://github.com/byx-darwin/gitflow-cli/commit/e202268b0ad9048f86ddf85e8b22686b5013603b)) - baoyuexing
- **(scripts)** isolate per-file rendering faults in dashboard renderer - ([1fc5f75](https://github.com/byx-darwin/gitflow-cli/commit/1fc5f75609edeb043b347495b4bb5ae73ebd17f5)) - baoyuexing
- **(scripts)** keep parser strict on unrecognized hunk lines - ([b9cad2f](https://github.com/byx-darwin/gitflow-cli/commit/b9cad2f09af388285c275b6feab22c35c6d1fa5a)) - baoyuexing
- **(scripts)** diff review annotation card noise + hunk boundary loss - ([3bd7cf3](https://github.com/byx-darwin/gitflow-cli/commit/3bd7cf34bc5460784a8d9eaa2fceb6353a2222f8)) - baoyuexing
- **(scripts)** restore meaningful click behavior on file-level annotation cards - ([f7e526c](https://github.com/byx-darwin/gitflow-cli/commit/f7e526c6d5b1f31c4ebece7f8ee898d1567cd49d)) - baoyuexing
- **(skills)** correct ../references paths to ../../docs/references - ([8d5c42a](https://github.com/byx-darwin/gitflow-cli/commit/8d5c42a3dc59e5ab55df85036bcff1760c3db952)) - baoyuexing
- **(skills)** address code-review findings on gf-pr-apply-feedback loop (#334) - ([afaa6ae](https://github.com/byx-darwin/gitflow-cli/commit/afaa6aeea52a885c6cfb4696e6129cbd909e410d)) - baoyuexing
- **(skills)** address review findings on #335 slice/falsifiable wording - ([829f9be](https://github.com/byx-darwin/gitflow-cli/commit/829f9be3ca7c329db7cfeecee56abda8ee206340)) - baoyuexing
- **(skills)** move gf-pr's illegal frontmatter key into See Also (#338) - ([7055e20](https://github.com/byx-darwin/gitflow-cli/commit/7055e202b9b343325037c3d4c91e41865f59622b)) - baoyuexing
- **(skills)** compare independent architecture extractions - ([600239b](https://github.com/byx-darwin/gitflow-cli/commit/600239b0feed89b5d691dc2539274a81fe3c4de7)) - baoyuexing
- **(skills)** constrain read-only skill tool grants (#343) - ([a0d41c2](https://github.com/byx-darwin/gitflow-cli/commit/a0d41c2b2aff5b7fb0aba399228310671c725220)) - baoyuexing
- **(smoke)** run API checks in an explicit repository - ([051aa03](https://github.com/byx-darwin/gitflow-cli/commit/051aa03965b7bcab245489273d4993a133cf16a2)) - baoyuexing
- **(tooling)** cargo-fmt pre-commit hook now writes back instead of only checking (#369) - ([2bb7fb5](https://github.com/byx-darwin/gitflow-cli/commit/2bb7fb51f5d7f41d23f397b7fb37b9f70fb42347)) - baoyuexing
- **(tooling)** quote cargo-fmt hook description so #369 isn't parsed as a YAML comment (review finding) - ([506c251](https://github.com/byx-darwin/gitflow-cli/commit/506c25197197180c401a110fef2863e7bf22da50)) - baoyuexing
- **(workflow)** address review findings on #336 error classification table - ([02e5d50](https://github.com/byx-darwin/gitflow-cli/commit/02e5d50578f7d0b42bef5bf3b96cd18b9a341cb2)) - baoyuexing

### Documentation

- **(agent)** streamline merge authorization and cleanup - ([be54d99](https://github.com/byx-darwin/gitflow-cli/commit/be54d994a1b56bbd58e56d54aab20c2076eef1b6)) - baoyuexing
- **(architecture)** restore semantic internals diagram - ([9980c9a](https://github.com/byx-darwin/gitflow-cli/commit/9980c9aee29093c672c0613557f72672b24bae63)) - baoyuexing
- **(architecture-diagram)** add implementation plan for gf-architecture-diagram - ([3bfe261](https://github.com/byx-darwin/gitflow-cli/commit/3bfe261d386678bfed4252c01bdba5bf78c38ad7)) - baoyuexing
- **(compat)** verify glab 1.115.0 on GitLab - ([2258ff1](https://github.com/byx-darwin/gitflow-cli/commit/2258ff1dd1c016a91c24a4cf022c7f885e69be47)) - baoyuexing
- **(core)** clarify platform.rs is URL detection, not a unified trait - ([6119a2a](https://github.com/byx-darwin/gitflow-cli/commit/6119a2a232838be83e1ca2f69bea94e6158a11cb)) - baoyuexing
- **(design)** record the residual gitcode --limit 101 risk and fix an API drift - ([ec847a7](https://github.com/byx-darwin/gitflow-cli/commit/ec847a71cf46e989b354e3728e1070e3001ba11a)) - baoyuexing
- **(dogfooding)** Phase 4 dogfooding summary for Issue #333 (GitHub/GitLab/GitCode, 0 bugs) - ([bfd01e8](https://github.com/byx-darwin/gitflow-cli/commit/bfd01e8cc8c17efb7af804b1301fec14f35c20fb)) - baoyuexing
- **(dogfooding)** Phase 4 dogfooding summary for Issue #334 (1 bug found) - ([b852754](https://github.com/byx-darwin/gitflow-cli/commit/b85275403861f7b584fc657c4c4275d4105f0ff7)) - baoyuexing
- **(evidence-grading)** design for gf-quality/gf-pr-review three-tier evidence tagging - ([28b9588](https://github.com/byx-darwin/gitflow-cli/commit/28b9588a5c88697902f2ebd727053ca99f316dfc)) - baoyuexing
- **(evidence-grading)** implementation plan for gf-quality/gf-pr-review evidence tagging - ([7bd3126](https://github.com/byx-darwin/gitflow-cli/commit/7bd3126b42f2f932f5e5084c72501efc0c80d536)) - baoyuexing
- **(gf-issue-decompose)** update stale note now that #337 ships a consumer - ([2823e6c](https://github.com/byx-darwin/gitflow-cli/commit/2823e6c3a5222e4504d9e7c553f39b52a59f82b1)) - baoyuexing
- **(gf-pipeline-analyzer)** add consecutive-watch-tier escalation rule - ([f53ba50](https://github.com/byx-darwin/gitflow-cli/commit/f53ba50f21b2c9674f78d6942d848a812929f0c0)) - baoyuexing
- **(gf-pipeline-analyzer)** note gf --version freshness check before snapshotting - ([a92249f](https://github.com/byx-darwin/gitflow-cli/commit/a92249fbf57bbf4296411cd7c595953879207c4e)) - baoyuexing
- **(gf-pipeline-analyzer)** translate version freshness note to English - ([0595932](https://github.com/byx-darwin/gitflow-cli/commit/0595932839af840aec047fff29e8187c7fa108b5)) - baoyuexing
- **(gf-smell)** route fixing smells back to /gf-refactor - ([d77f5c5](https://github.com/byx-darwin/gitflow-cli/commit/d77f5c54b7482cab31c2fb2d37b28d6167853983)) - baoyuexing
- **(gf-workflow)** worktree symlinks write to shared info/exclude (#318) - ([e32c090](https://github.com/byx-darwin/gitflow-cli/commit/e32c0908dfec135970d3baf2d6e4658e1ebf8abc)) - baoyuexing
- **(gf-workflow)** pre-delivery symlink-commit guard in Phase 3 Step 3 (#318) - ([94363e1](https://github.com/byx-darwin/gitflow-cli/commit/94363e14bf57b87c9171e881024ecc3147f526e8)) - baoyuexing
- **(gf-workflow)** worktree example writes symlinks to shared info/exclude (#318) - ([f494e17](https://github.com/byx-darwin/gitflow-cli/commit/f494e17fd8beeb4a63e695263b25958482cb6c00)) - baoyuexing
- **(gf-workflow)** explain why worktree symlinks must never reach main branch (#318) - ([c0a6f4f](https://github.com/byx-darwin/gitflow-cli/commit/c0a6f4fffe2268fb905d6646889dcda5729b1950)) - baoyuexing
- **(gf-workflow)** dedupe redundant references.md pointer in Phase 3 Step 1 (#318) - ([e032282](https://github.com/byx-darwin/gitflow-cli/commit/e0322827c440aebfc61870755ecf2d60658b64c7)) - baoyuexing
- **(gf-workflow)** fix variable casing in symlink guard command (#318) - ([656970b](https://github.com/byx-darwin/gitflow-cli/commit/656970bad8286fbd3b1334e0ce24b8daef4c70e3)) - baoyuexing
- **(gf-workflow)** remove Mode ① background agent from Execution Modes - ([99ac4a5](https://github.com/byx-darwin/gitflow-cli/commit/99ac4a55f8b6b554b93d8312908a031e2ac7484e)) - baoyuexing
- **(gf-workflow)** trim GO gate to two execution modes - ([55d0794](https://github.com/byx-darwin/gitflow-cli/commit/55d07944f307e9c64d1b0267f1f157332501ea19)) - baoyuexing
- **(gf-workflow)** sync GO gate description in gates.md to two modes - ([17ea322](https://github.com/byx-darwin/gitflow-cli/commit/17ea3228250cc3d8b0daedf43030a5629d3ee4b6)) - baoyuexing
- **(gf-workflow)** make pipeline analysis optional, default delivery to local merge - ([04dabde](https://github.com/byx-darwin/gitflow-cli/commit/04dabded38b3e907280079eff963b35a98eab7ba)) - baoyuexing
- **(gf-workflow)** split skill-source sentinels by invocation policy - ([fc4eeb3](https://github.com/byx-darwin/gitflow-cli/commit/fc4eeb3eea0a27a4edc3eb6f3d6a90fe03908319)) - baoyuexing
- **(gf-workflow-batch)** describe dependency resolution + add 6 test scenarios (#337) - ([87ec92a](https://github.com/byx-darwin/gitflow-cli/commit/87ec92af6d380efd3d0ed66d9858c45f1b946d8e)) - baoyuexing
- **(pagination)** wf-2026-09-17-003 Phase 1-2 artifacts for #360 - ([dde1aa0](https://github.com/byx-darwin/gitflow-cli/commit/dde1aa0f1946d3e862e64f737e8b57fcebb1f84c)) - baoyuexing
- **(phase4)** archive report threshold overflow + add Issue #333 post-delivery reports - ([b76ce52](https://github.com/byx-darwin/gitflow-cli/commit/b76ce527d18dec9b1fe5b15576564549e8665d64)) - baoyuexing
- **(phase4)** Issue #334 post-delivery reports + specs index fix - ([9cffaca](https://github.com/byx-darwin/gitflow-cli/commit/9cffacad0305e012a8a4adbbd627be914e533fe7)) - baoyuexing
- **(phase4)** Issue #335 pipeline report — 3rd escalation on Windows/macOS test flakiness (#373) - ([b6b1657](https://github.com/byx-darwin/gitflow-cli/commit/b6b1657223d7c05df4ff60d6837aa6a7f6305088)) - baoyuexing
- **(phase4)** Issue #336 pipeline report — clean, prior escalation tracked by #373 - ([383e0d2](https://github.com/byx-darwin/gitflow-cli/commit/383e0d2428e27e1848ddfb4c6585aea5aabb7110)) - baoyuexing
- **(phase4)** Issue #337 post-delivery reports — triage 2 new, review 6 minor (followup #374) - ([73dba50](https://github.com/byx-darwin/gitflow-cli/commit/73dba5071b5cf80cd01e71cb08516126c35b3889)) - baoyuexing
- **(phase4)** Issue #337 dogfooding report — GitLab issue-create bug found (#375) - ([9f90b1a](https://github.com/byx-darwin/gitflow-cli/commit/9f90b1ae30cc6bfff5f190b60dd09c459a4e5915)) - baoyuexing
- **(plans)** fix paging tests that could never observe page increment - ([bccde9c](https://github.com/byx-darwin/gitflow-cli/commit/bccde9c6c4da46a0f5c82da5eaee7bb185957c96)) - baoyuexing
- **(plans)** implementation plan for gf-pr-apply-feedback review loop (#334) - ([574c63c](https://github.com/byx-darwin/gitflow-cli/commit/574c63c75f6746ef28de1ec19497dabf1af0c26a)) - baoyuexing
- **(pr)** fix stale --base help text after default-branch detection (#305) - ([ffbdd65](https://github.com/byx-darwin/gitflow-cli/commit/ffbdd655e575a10a481b90f6281c3cd61d598ce6)) - baoyuexing
- **(refactor)** fix RF-001 commit SHA in dogfooding report - ([431e4df](https://github.com/byx-darwin/gitflow-cli/commit/431e4df2693645a78c27d40e0b42eb763e5295f8)) - baoyuexing
- **(roadmap)** add v2 multi-role product evaluation for v1.9.0 - ([647bc5b](https://github.com/byx-darwin/gitflow-cli/commit/647bc5b5db28277d7d2685eb9755b5dfe969b8d3)) - baoyuexing
- **(skills)** drop the nonexistent --page fallback, add truncation checks - ([9600a59](https://github.com/byx-darwin/gitflow-cli/commit/9600a59af91de2e86dc1894c416fbc3fbc619b99)) - baoyuexing
- **(skills)** fix review round 1 — English error-handling row, drop nonexistent --since - ([6f74b6f](https://github.com/byx-darwin/gitflow-cli/commit/6f74b6f93daad42a3479c68b64c575328bb7e7cd)) - baoyuexing
- **(specs)** add gitcode pagination fix design and plan for #365 - ([bfb43c5](https://github.com/byx-darwin/gitflow-cli/commit/bfb43c54bdf11a62feff63f88ad4aff49ce5e3a5)) - baoyuexing
- **(specs)** replace gitcode assumptions with measured findings from #365 - ([d6a8b9e](https://github.com/byx-darwin/gitflow-cli/commit/d6a8b9ebee5510a28ff5028462d9241066e7d69f)) - baoyuexing
- **(specs)** index the quality-review evidence grading plan - ([2738406](https://github.com/byx-darwin/gitflow-cli/commit/2738406e4dc69db3e32b5f7166de8c82b202359c)) - baoyuexing
- **(specs)** design doc for gf-pr-apply-feedback review-loop contract (#334) - ([51b2496](https://github.com/byx-darwin/gitflow-cli/commit/51b249611bbd14ebd1e03da48c8792810c74b63f)) - baoyuexing
- **(specs)** design doc + implementation plan for gf-issue vertical slice (#335) - ([f72480b](https://github.com/byx-darwin/gitflow-cli/commit/f72480b20d0441248f2c350a1dc23ab61733d446)) - baoyuexing
- **(specs)** design doc + implementation plan for gf-workflow error classification (#336) - ([8d38c8e](https://github.com/byx-darwin/gitflow-cli/commit/8d38c8ecd49dcab5d914520058d41d830df7d75e)) - baoyuexing
- **(website)** align legacy content with current CLI - ([1f8566b](https://github.com/byx-darwin/gitflow-cli/commit/1f8566b2c2326812f4004fa2f986d05a853d7832)) - baoyuexing
- **(workflow)** wf-2026-09-04-001 Phase 1-2 artifacts (#318) - ([e07c4d2](https://github.com/byx-darwin/gitflow-cli/commit/e07c4d28469183b8f71a185279f8e23170d49201)) - baoyuexing
- **(workflow)** wf-2026-09-04-005 Phase 1-2 artifacts - ([d8aff13](https://github.com/byx-darwin/gitflow-cli/commit/d8aff1361b0e241d6c0b3136c5a13d99223f6a59)) - baoyuexing
- **(workflow)** wf-2026-09-17-001 Phase 1-2 artifacts - ([39e93e5](https://github.com/byx-darwin/gitflow-cli/commit/39e93e5f0efc6be3267c7d546d4404dd28d88d3f)) - baoyuexing
- **(workflow)** wf-2026-09-17-002 Phase 1-2 artifacts - ([14c2e2f](https://github.com/byx-darwin/gitflow-cli/commit/14c2e2fe03c8773481c7735cc01db17a2460216e)) - baoyuexing
- **(workflow)** design gf-workflow-batch dependency-edge topo ordering (#337) - ([eaf09f5](https://github.com/byx-darwin/gitflow-cli/commit/eaf09f5648232b691bae8b44a6204d63ff83872a)) - baoyuexing
- **(workflow)** add implementation plan for gf-workflow-batch topo ordering (#337) - ([73a3bf1](https://github.com/byx-darwin/gitflow-cli/commit/73a3bf1545a1909d473b5804cf35666cd9744a48)) - baoyuexing
- **(workflow)** add batch1 design + plan (#372 #362 #375) - ([1771cb4](https://github.com/byx-darwin/gitflow-cli/commit/1771cb4360d2c335f68be840052b74c425fa8f20)) - baoyuexing
- **(workflow)** add #364 design + plan - ([8e1b6d1](https://github.com/byx-darwin/gitflow-cli/commit/8e1b6d117849bb5ae4942fffde413869b7fc0317)) - baoyuexing
- **(workflow)** add #373 design + plan - ([91a6297](https://github.com/byx-darwin/gitflow-cli/commit/91a6297b9ad09f8ff50a123212fbc81fa064e40d)) - baoyuexing
- **(workflow)** add #374 design + plan - ([887d393](https://github.com/byx-darwin/gitflow-cli/commit/887d393420b142438b20b12d88622b0f83e04b32)) - baoyuexing
- **(workflow)** add batch5 design + plan (#353 #363 #355 #338) - ([bb686d3](https://github.com/byx-darwin/gitflow-cli/commit/bb686d39f792eda7f3381cd5eee9d81933cea689)) - baoyuexing
- **(workflow)** add batch6 design + plan (#369 #339 #352) - ([82008b6](https://github.com/byx-darwin/gitflow-cli/commit/82008b613da041cca840f82dbef3c01d7bbe296d)) - baoyuexing
- **(workflow)** add batch7 design + plan (#361 #356) - ([bf730d1](https://github.com/byx-darwin/gitflow-cli/commit/bf730d154c261fccbb23acd53b9718f6b481fab3)) - baoyuexing
- **(workflow)** add plan for gf --version git sha traceability (#378) - ([aa23e09](https://github.com/byx-darwin/gitflow-cli/commit/aa23e0954c7de1a2a7f9a6a24b4b9b23dd489611)) - baoyuexing
- **(workflow)** add design + plan for ReleaseData.created_at optional (#366) - ([b739cb5](https://github.com/byx-darwin/gitflow-cli/commit/b739cb5fb9a466f72bc3e524b21434b4f237ac6b)) - baoyuexing
- **(workflow)** add design + plan for milestone issue/PR attachment (#357) - ([a3ec432](https://github.com/byx-darwin/gitflow-cli/commit/a3ec4327dd75c249946d5c0e2ba36973787c183a)) - baoyuexing
- **(workflow)** add design for issue/pr/review/pipeline created_at optional (#380) - ([9bfe3a1](https://github.com/byx-darwin/gitflow-cli/commit/9bfe3a1c4bdd2b90b53b40e1b3f47d99a7131a9d)) - baoyuexing
- **(workflow)** add implementation plan for #380 (issue/pr/review/pipeline created_at optional) - ([d85f22a](https://github.com/byx-darwin/gitflow-cli/commit/d85f22a63843e725a0c0abdf38d3a586ad1099a9)) - baoyuexing
- **(workflow)** reuse existing open issues in phase one - ([c044f3a](https://github.com/byx-darwin/gitflow-cli/commit/c044f3aefedd66b88b03d87691704dde92a51ac9)) - baoyuexing
- add Phase 4 reports for PR #297 (pipeline analysis, review) - ([bf0d5d7](https://github.com/byx-darwin/gitflow-cli/commit/bf0d5d73d4a0f91688563291d785dc214e1b7b84)) - baoyuexing
- add design and plan for pipeline report in-progress fix - ([e87891e](https://github.com/byx-darwin/gitflow-cli/commit/e87891e68682a63766b604b85bbddceb7949e5bf)) - baoyuexing
- add Phase 4 pipeline report for PR #298 - ([fe8ac10](https://github.com/byx-darwin/gitflow-cli/commit/fe8ac100cc4fd9f0007aa040cb8b0969a4eeb5ef)) - baoyuexing
- regenerate pipeline-analysis-report for issue #289 - ([d6574a6](https://github.com/byx-darwin/gitflow-cli/commit/d6574a64924b820f3a6728ce0ebbf631525e29b8)) - baoyuexing
- link pipeline-analysis-report-2026-09-02-issue289 to follow-up Issue #301 - ([b04fb3d](https://github.com/byx-darwin/gitflow-cli/commit/b04fb3dbad6dfc1cdc1605ce615de371ac081912)) - baoyuexing
- add PR #302 pipeline analysis report - ([c73da43](https://github.com/byx-darwin/gitflow-cli/commit/c73da436c91eda4079bb9150033b69759aa0c321)) - baoyuexing
- add design spec and implementation plan for e2e-gitlab/e2e-gitcode - ([fc392eb](https://github.com/byx-darwin/gitflow-cli/commit/fc392ebe67bc1ff690db550ca463ed3ed75cb065)) - baoyuexing
- add Phase 4 reports for issue #291 (pipeline analysis, triage, code review) - ([c714ed0](https://github.com/byx-darwin/gitflow-cli/commit/c714ed001c76078372160d98dda53efe870bc083)) - baoyuexing
- add Phase 4 reports for PR #309 (issue #292 pipeline analysis, code review) - ([c2c2aaa](https://github.com/byx-darwin/gitflow-cli/commit/c2c2aaac1b1f3f31b5f33cce36a2b76a056164d8)) - baoyuexing
- add pipeline analysis report for PR #311 (issue #293) - ([1312e1e](https://github.com/byx-darwin/gitflow-cli/commit/1312e1e165e91bc307ddd4bc34880b826e3d3aaf)) - baoyuexing
- add pipeline analysis report for PR #312 - ([0c24309](https://github.com/byx-darwin/gitflow-cli/commit/0c24309cece3a27bfbb80fc12ec9265cb3713944)) - baoyuexing
- add design + plan for #305 pr create default branch detection - ([f4308a8](https://github.com/byx-darwin/gitflow-cli/commit/f4308a83fa4922bc07b7786abc41daee40542653)) - baoyuexing
- add pipeline analysis and code review reports for PR #313 - ([4a758f6](https://github.com/byx-darwin/gitflow-cli/commit/4a758f60f7c9fbfd2b271b2ad00a7ce5edebdc21)) - baoyuexing
- add pipeline analysis report for PR #315 (issue #295) - ([56c9b14](https://github.com/byx-darwin/gitflow-cli/commit/56c9b142bc4beb99362117ecaf8007e45f3616c1)) - baoyuexing
- add pipeline analysis report for PR #316 (issue #296) - ([89c5b49](https://github.com/byx-darwin/gitflow-cli/commit/89c5b492abcf7522208b8ffa57ca4aac6d42e44a)) - baoyuexing
- add design + plan for temp-file test isolation (issue #301) - ([3fec3a9](https://github.com/byx-darwin/gitflow-cli/commit/3fec3a92a38d1e334f4eb50c3861b54e6329cfaf)) - baoyuexing
- add pipeline analysis + code review reports for PR #317 (issue #301) - ([176c2ad](https://github.com/byx-darwin/gitflow-cli/commit/176c2ad7fd3dc0fbc8c1377c44ade246490fce28)) - baoyuexing
- add design + plan for e2e-regression dedup verification (issue #310) - ([ddc7ca7](https://github.com/byx-darwin/gitflow-cli/commit/ddc7ca7ee4f3bbca1b8a6c14bc9af0bbfa6ca539)) - baoyuexing
- record e2e-regression dedup verification outcome (issue #310) - ([7f14a1e](https://github.com/byx-darwin/gitflow-cli/commit/7f14a1e2eb00c8a94b730f09df2ff19cba96080f)) - baoyuexing
- fix nested code fence in dedup verification plan (code review finding) - ([e3055c1](https://github.com/byx-darwin/gitflow-cli/commit/e3055c1c2cf2a92f01cd2ae47c370d37ae30716a)) - baoyuexing
- add pipeline analysis + code review reports for PR #320 (issue #310) - ([4322027](https://github.com/byx-darwin/gitflow-cli/commit/43220271a79dc1a65d2ef18f5dc135338f2a8c61)) - baoyuexing
- index worktree-symlink-exclude-guard design doc - ([e0b482c](https://github.com/byx-darwin/gitflow-cli/commit/e0b482c2c04cdb914e99c36d30a888322cd15b5c)) - baoyuexing
- add pipeline analysis + post-merge review reports for PR #321 (issue #318) - ([87ad8d8](https://github.com/byx-darwin/gitflow-cli/commit/87ad8d801d4599a78e99735fe00c2efdcd6ea5b5)) - baoyuexing
- Phase 4 reports for PR #323 + archive reports beyond retention cap - ([f35bd03](https://github.com/byx-darwin/gitflow-cli/commit/f35bd035442f09b319ef63e40f795650564cba5f)) - baoyuexing
- Phase 4 reports for Issue #324 + archive reports beyond retention cap - ([2f6d32a](https://github.com/byx-darwin/gitflow-cli/commit/2f6d32aa362427a0c8032916362e06bcf751f758)) - baoyuexing
- add design and plan for removing gf-workflow Mode ① background agent - ([78d031c](https://github.com/byx-darwin/gitflow-cli/commit/78d031c49a0783eccdeb3765fccce831e59b8ffb)) - baoyuexing
- Phase 4 reports for PR #326 + archive reports beyond retention cap - ([99a9e28](https://github.com/byx-darwin/gitflow-cli/commit/99a9e2889e30f6babf50f329553aeef7cc7288d4)) - baoyuexing
- Phase 4 code review report for Issue #378 + archive reports beyond retention cap - ([38e6fab](https://github.com/byx-darwin/gitflow-cli/commit/38e6fabcdc81e0c1c10ea0a33b557a1723763329)) - baoyuexing
- Phase 4 code review report for Issue #377 + archive report beyond retention cap - ([0b7bcbd](https://github.com/byx-darwin/gitflow-cli/commit/0b7bcbd77eaf48c89737dcee59670ad1c271c772)) - baoyuexing
- Phase 4 code review report for Issue #368 + archive report beyond retention cap - ([b93b4bd](https://github.com/byx-darwin/gitflow-cli/commit/b93b4bdba5fb08d572f8a693e0bd45867a75e1e5)) - baoyuexing
- Phase 4 review + dogfooding reports for Issue #366 + archive report beyond retention cap - ([358a249](https://github.com/byx-darwin/gitflow-cli/commit/358a2496c4a7f20653788d52f2a3249c89cfbbed)) - baoyuexing
- Phase 4 review report for Issue #357 + archive report beyond retention cap - ([00ab619](https://github.com/byx-darwin/gitflow-cli/commit/00ab61918e2709e4c2f7103c1938867224fa15c2)) - baoyuexing
- design for gf-security-check/gf-regression gate in gf-workflow - ([2113d85](https://github.com/byx-darwin/gitflow-cli/commit/2113d8503bb3dee454ce3beef5cb5166d202ee3b)) - baoyuexing
- add implementation plan for Issue #344 - ([78ed16f](https://github.com/byx-darwin/gitflow-cli/commit/78ed16f754753af3ee80a85068102e7b7560d910)) - baoyuexing
- Phase 4 triage + review reports for Issue #344 - ([c5ec8ac](https://github.com/byx-darwin/gitflow-cli/commit/c5ec8aca91e5a48fa344ebf330b7960adcc24bfe)) - baoyuexing
- design for workflow contract dashboard renderer - ([7f0db7f](https://github.com/byx-darwin/gitflow-cli/commit/7f0db7f51008e406cdfa0522dad3ad9c85a75068)) - baoyuexing
- add implementation plan for Issue #345 - ([70b2025](https://github.com/byx-darwin/gitflow-cli/commit/70b2025db27bc4cc163957e1ee7430e65faabd26)) - baoyuexing
- Phase 4 triage + review reports for Issue #345 - ([a24d7f7](https://github.com/byx-darwin/gitflow-cli/commit/a24d7f79c82b0fd194b0d7a22ce197bab2ee9e36)) - baoyuexing
- design for diff interactive review page renderer - ([9d4d8a6](https://github.com/byx-darwin/gitflow-cli/commit/9d4d8a62038830fe6c0f09513ad98aff53bf9934)) - baoyuexing
- add implementation plan for Issue #346 - ([bdfcff9](https://github.com/byx-darwin/gitflow-cli/commit/bdfcff990fc5f422cb6bc7b2c1776b19c90194e7)) - baoyuexing
- Phase 4 triage + review reports for Issue #346 - ([3762630](https://github.com/byx-darwin/gitflow-cli/commit/3762630383a072bdbf2ee05370c2cca869d5ec8f)) - baoyuexing
- Phase 4 review report for Issue #394 - ([c853b08](https://github.com/byx-darwin/gitflow-cli/commit/c853b08a298f26b98f446dcc5f840824258f2d0d)) - baoyuexing
- Phase 4 review report for Issue #396 - ([0aa0281](https://github.com/byx-darwin/gitflow-cli/commit/0aa02811a2080ba37e7811532e134e7f42a2fff9)) - baoyuexing
- Phase 4 review report for Issue #395 - ([2da839c](https://github.com/byx-darwin/gitflow-cli/commit/2da839c41531311c535fc02d0c9cc3d5dbe083b1)) - baoyuexing
- Phase 4 review report for Issue #399 - ([7189484](https://github.com/byx-darwin/gitflow-cli/commit/718948471d8a6a55b8a4cbfe8f9870ab10c2fdbf)) - baoyuexing
- correct #380 design — GitHub needs Some() wraps at 4 Category C sites for compile compatibility - ([e15ad7c](https://github.com/byx-darwin/gitflow-cli/commit/e15ad7c7a6e6dee02ac6ff2c1e953758b2364b1a)) - baoyuexing
- Phase 4 review report for Issue #380 - ([d298b1e](https://github.com/byx-darwin/gitflow-cli/commit/d298b1e3440af85eafd82b66fbfa2ac618696c7e)) - baoyuexing
- update compatibility matrix for v2.0.0 - ([1782712](https://github.com/byx-darwin/gitflow-cli/commit/17827124aefde25ff27935770b4323ba452f3ab4)) - baoyuexing

### Features

- **(adapter-utils)** add injectable EnvSource abstraction - ([2e6e4a8](https://github.com/byx-darwin/gitflow-cli/commit/2e6e4a8479d7593ae0a305b51c97f2fcbb0abc01)) - baoyuexing
- **(cli)** embed git short SHA in gf --version for build traceability - ([119656d](https://github.com/byx-darwin/gitflow-cli/commit/119656db5af9167d83d723801fdc0e0cd6547beb)) - baoyuexing
- **(cli)** add --milestone/--remove-milestone flags to issue create/edit/list and pr create - ([789e235](https://github.com/byx-darwin/gitflow-cli/commit/789e23547194cb8124617d138f9b1740b3bd1448)) - baoyuexing
- **(context)** add selective workflow evidence manifests - ([9000ff6](https://github.com/byx-darwin/gitflow-cli/commit/9000ff6ff950637a696c18b8ea8f4a8a3150dc44)) - baoyuexing
- **(core)** add Paged<T> and fetch_capped pagination primitive - ([bc2a830](https://github.com/byx-darwin/gitflow-cli/commit/bc2a830639948ca9f15e7d22630caadbe69bd9b1)) - baoyuexing
- **(core)** add optional per-host breakdown to AuthStatus (#362) - ([66a059f](https://github.com/byx-darwin/gitflow-cli/commit/66a059f16cbd39aa10e0b403d11a037a4c815d6f)) - baoyuexing
- **(core)** add MilestoneRef and resolve_milestone_identifier for issue/PR milestone attachment - ([8bf91a0](https://github.com/byx-darwin/gitflow-cli/commit/8bf91a090d82eed9dc5651739aefbc7384ba61f7)) - baoyuexing
- **(core,cli)** add PaginationMeta envelope field and truncation warning - ([6f4e2ca](https://github.com/byx-darwin/gitflow-cli/commit/6f4e2ca6a903c1dc30feb43f18aa095f13873a6b)) - baoyuexing
- **(decision)** add optional Jev adapter and typed decide CLI - ([7e75f35](https://github.com/byx-darwin/gitflow-cli/commit/7e75f358c2d0f394e16c789d59d12381d1d6fb87)) - baoyuexing
- **(decision)** add offline evaluation and threshold calibration - ([0e554b0](https://github.com/byx-darwin/gitflow-cli/commit/0e554b063b967e166870f0f7d5489db862884d58)) - baoyuexing
- **(diff-review)** add opt-in semantic enrichment - ([ecb9c7b](https://github.com/byx-darwin/gitflow-cli/commit/ecb9c7bc52a04f468cc7f7cdac9b9d991bf7ac4e)) - baoyuexing
- **(e2e-core)** add gitlab/gitcode fields and accessors to TestConfig - ([42a1327](https://github.com/byx-darwin/gitflow-cli/commit/42a1327974a60ac65fbcf3f77a9dd84334866642)) - baoyuexing
- **(e2e-core)** add TtyRunner::dir() working-directory override - ([d9b7e92](https://github.com/byx-darwin/gitflow-cli/commit/d9b7e924e865ec5c14492af719e5918b9083bb61)) - baoyuexing
- **(e2e-core)** add scratch_repo_dir() for remote-pointed test checkouts - ([da92448](https://github.com/byx-darwin/gitflow-cli/commit/da92448b0547e87d1c3601c0dc186687244b20d7)) - baoyuexing
- **(e2e-gitcode)** add crate scaffold with auth/noauth E2E tests - ([fa6cdb3](https://github.com/byx-darwin/gitflow-cli/commit/fa6cdb3e45b15cb8f2a296e108445ba158a0860b)) - baoyuexing
- **(e2e-gitcode)** add issue/pr E2E tests via scratch_repo_dir - ([20ad92d](https://github.com/byx-darwin/gitflow-cli/commit/20ad92dfe047aa231a8b078fb716dbfebad3d7a8)) - baoyuexing
- **(e2e-gitlab)** add crate scaffold with auth/noauth E2E tests - ([bde019c](https://github.com/byx-darwin/gitflow-cli/commit/bde019c04a8b0dcac9f6661be0c0db01f002d827)) - baoyuexing
- **(e2e-gitlab)** add issue/pr E2E tests via scratch_repo_dir - ([f1a995f](https://github.com/byx-darwin/gitflow-cli/commit/f1a995f85578829457583a9b8c8c60996e4fcb64)) - baoyuexing
- **(gf-pr-review)** tag per-dimension verdicts with evidence grading - ([8855ff0](https://github.com/byx-darwin/gitflow-cli/commit/8855ff02cef7fb35ae53319a8b5fb084018208fd)) - baoyuexing
- **(gf-quality)** add three-tier evidence grading and failing-test ancestry table - ([737a23a](https://github.com/byx-darwin/gitflow-cli/commit/737a23aae6f059ecb43e541567a374de2c881db2)) - baoyuexing
- **(gf-walkthrough)** add SKILL.md body (TDD GREEN, Task 2) - ([95189b0](https://github.com/byx-darwin/gitflow-cli/commit/95189b07bec914ffd0f2310df00b1ef673d36f94)) - baoyuexing
- **(gf-workflow)** insert Phase 3 change-surface gate for security/regression checks - ([780510e](https://github.com/byx-darwin/gitflow-cli/commit/780510e391ac93fcf9593bbd6d0ca0ba7bd3f2b9)) - baoyuexing
- **(gf-workflow-batch)** add dependency-edge resolution to pending derivation (#337) - ([1fb7492](https://github.com/byx-darwin/gitflow-cli/commit/1fb7492b57f2359625c8fb7c907134de64a70d23)) - baoyuexing
- **(gitcode)** support --milestone on issue create/edit/list, and pr create via create+edit two-step - ([ea24e96](https://github.com/byx-darwin/gitflow-cli/commit/ea24e96b52c18e2c99848b204f07269406d80329)) - baoyuexing
- **(github)** support --milestone on issue create/edit/list and pr create - ([0f3abba](https://github.com/byx-darwin/gitflow-cli/commit/0f3abba1c04d9d02fc087a2142169ec8f47eebc8)) - baoyuexing
- **(gitlab)** support --milestone on issue create/edit/list and mr create - ([a329059](https://github.com/byx-darwin/gitflow-cli/commit/a3290593fb9a4fcf1e7b1ab2b2974c5b7de06b89)) - baoyuexing
- **(issue)** add advisory requirement-quality precheck - ([3557093](https://github.com/byx-darwin/gitflow-cli/commit/35570936cc21bf0dcb20d211bc80c8f41af2a4f0)) - baoyuexing
- **(makefile)** add render-workflow-dashboard target - ([e1e160c](https://github.com/byx-darwin/gitflow-cli/commit/e1e160cc748fad8bd1941346e97bcdba75395f24)) - baoyuexing
- **(makefile)** add render-diff-review target - ([02a6e96](https://github.com/byx-darwin/gitflow-cli/commit/02a6e96f17914724c1f093a5c3e11322df086948)) - baoyuexing
- **(pagination)** [**breaking**] eliminate silent truncation across gf list commands - ([4020506](https://github.com/byx-darwin/gitflow-cli/commit/4020506e74ed70ac46ca3405f664516fe9959d91)) - baoyuexing
- **(pipeline)** attribute topFailures to job names via job-level fetch - ([7ef86b2](https://github.com/byx-darwin/gitflow-cli/commit/7ef86b25ba8a74ee531e4048da4ea03550614376)) - baoyuexing
- **(pipeline)** add advisory CI failure classification - ([4ad1397](https://github.com/byx-darwin/gitflow-cli/commit/4ad139790915b98f4beb8524e57634355df5deb2)) - baoyuexing
- **(pr)** add default_branch() to PrProvider and implement per platform - ([f5baabf](https://github.com/byx-darwin/gitflow-cli/commit/f5baabfba86f3c691030e1ec4e43373ad15b700a)) - baoyuexing
- **(query)** add typed Issue and PR search - ([1b8d016](https://github.com/byx-darwin/gitflow-cli/commit/1b8d016a6b5153d75760a72520584b46f789cf6a)) - baoyuexing
- **(review)** add advisory PR semantic precheck - ([3524965](https://github.com/byx-darwin/gitflow-cli/commit/3524965f2c95da4dfcf7988fc930a98a2acb6783)) - baoyuexing
- **(scripts)** add skill doc link validator with fixture tests - ([ad8adde](https://github.com/byx-darwin/gitflow-cli/commit/ad8addec5509f9fbcd63e8e506dd0292ad2dac2e)) - baoyuexing
- **(scripts)** add workflow contract dashboard renderer - ([11ab061](https://github.com/byx-darwin/gitflow-cli/commit/11ab06122d02fdde9689774d3df28fd8e802d1c6)) - baoyuexing
- **(scripts)** add unified diff parser core for diff review renderer - ([c96365f](https://github.com/byx-darwin/gitflow-cli/commit/c96365fe74f520fa4a537cdcb5ed0341167fe106)) - baoyuexing
- **(scripts)** detect renamed files in diff parser - ([327916b](https://github.com/byx-darwin/gitflow-cli/commit/327916b15f3ec79de01351a2e3a0867b8198ef98)) - baoyuexing
- **(scripts)** detect binary files and scan untracked files - ([f73aa9f](https://github.com/byx-darwin/gitflow-cli/commit/f73aa9f7a54c82b620e9341f66292bf003d8b8f1)) - baoyuexing
- **(scripts)** assemble annotations.json and wire up scan CLI - ([831d8ff](https://github.com/byx-darwin/gitflow-cli/commit/831d8ff6ccf0259a761bd2179054dd8a9aad447e)) - baoyuexing
- **(scripts)** render three-pane HTML structure from annotations - ([49ddbbe](https://github.com/byx-darwin/gitflow-cli/commit/49ddbbe41ea2994d2f5083fb1f7f7ba2d1577b28)) - baoyuexing
- **(scripts)** add lightweight regex-based syntax highlighter - ([b2869b4](https://github.com/byx-darwin/gitflow-cli/commit/b2869b49eaa51adc9f1d92bb2705a4c9fb4d4b4a)) - baoyuexing
- **(scripts)** add JS interactivity (anchor scroll, hover link, drag-resize) - ([a2f9782](https://github.com/byx-darwin/gitflow-cli/commit/a2f9782f2b7fd8fddc74f7a44fa6ff828a1e89b0)) - baoyuexing
- **(skills)** add gf-issue-decompose for vertical-slice batch decomposition - ([b113bb2](https://github.com/byx-darwin/gitflow-cli/commit/b113bb22acdde4080a9bcf45b3c3c05d071b38e2)) - baoyuexing
- **(skills)** close review loop in gf-pr-apply-feedback with 3-round cap (#334) - ([f404031](https://github.com/byx-darwin/gitflow-cli/commit/f4040318dfd4dd0370148566f941a0251e37997b)) - baoyuexing
- **(skills)** gf-issue-create adds vertical-slice + falsifiable-acceptance constraints (#335) - ([6626308](https://github.com/byx-darwin/gitflow-cli/commit/6626308b39c7cad1f8722cdf5a187e66e734100d)) - baoyuexing
- **(skills)** gf-issue-review adds slice-direction dimension + base-commit-red check (#335) - ([03e4108](https://github.com/byx-darwin/gitflow-cli/commit/03e41082a5df7e3ef6107ab2c1d1f1e8a8b3a257)) - baoyuexing
- **(skills)** add Claude Code plugin distribution - ([e39db93](https://github.com/byx-darwin/gitflow-cli/commit/e39db93f6504b57cb943478742499d57c661a40e)) - baoyuexing
- **(skills)** add read-only Jev skill suggestions (#384) - ([ea0c285](https://github.com/byx-darwin/gitflow-cli/commit/ea0c285b057bab0183d3caa6317a8c21002e703b)) - baoyuexing
- **(website)** feature Jev decision layer - ([1a7a994](https://github.com/byx-darwin/gitflow-cli/commit/1a7a9944518ff7f2b7bd9d1a8fca0b3578f362c4)) - baoyuexing
- **(website)** strengthen Jev GEO signals - ([3734607](https://github.com/byx-darwin/gitflow-cli/commit/3734607a36e96505a9d141c9970521225d2bbc1a)) - baoyuexing
- **(website)** publish Jev engineering article - ([47f5616](https://github.com/byx-darwin/gitflow-cli/commit/47f561645152483907d6512b2b8842fdbdb38290)) - baoyuexing
- **(workflow)** add Phase 3 execution error classification table (#336) - ([1e08881](https://github.com/byx-darwin/gitflow-cli/commit/1e08881497576d3b648edbb8f6eb764c4911f373)) - baoyuexing
- **(workflow)** Phase 3 execution engine references error classification table (#336) - ([45cfe8d](https://github.com/byx-darwin/gitflow-cli/commit/45cfe8d39dad68eb45633e21444407fe4dc6a937)) - baoyuexing
- **(workflow)** generate conditional diff review before delivery - ([3bbe7e1](https://github.com/byx-darwin/gitflow-cli/commit/3bbe7e166e2593c21a6f8eb48caebde9f35b211d)) - baoyuexing
- **(workflow)** add advisory Jev mode recommendation - ([5b5f59a](https://github.com/byx-darwin/gitflow-cli/commit/5b5f59a0ae8781e474ec369eb8075e88bcfbeca1)) - baoyuexing
- **(workflow)** add advisory semantic rule checks - ([e07d6ce](https://github.com/byx-darwin/gitflow-cli/commit/e07d6ce04a5407d4b8bbcae66f4c25dac08aca10)) - baoyuexing
- **(workflow)** assess trace progress with replayable advisory reports - ([01e1a94](https://github.com/byx-darwin/gitflow-cli/commit/01e1a94b0be3baf8f7b901d05673933aae79195c)) - baoyuexing

### Miscellaneous Chores

- **(cli)** [**breaking**] remove deprecated run subcommand - ([0d1e5b7](https://github.com/byx-darwin/gitflow-cli/commit/0d1e5b7e0fe0b76047c98d1b781cc79831b5759b)) - baoyuexing
- **(community)** mark good-first-issue candidates and add contributor entry link - ([6f85f3b](https://github.com/byx-darwin/gitflow-cli/commit/6f85f3bb73a5befe3a1c3b49f737c9abff07a1c8)) - baoyuexing
- **(compat)** verify glab 1.118.0 and update matrix - ([10a1fc1](https://github.com/byx-darwin/gitflow-cli/commit/10a1fc1f65c7bc8bf15e47666f0231f5e499aca3)) - baoyuexing
- **(compat)** verify gh 2.98.0 and update matrix - ([5a9d6a7](https://github.com/byx-darwin/gitflow-cli/commit/5a9d6a730e1cc19698ae23fb647a854420e3a24d)) - baoyuexing
- **(compat)** verify GitCode CLI 0.11.1 and update matrix - ([2552626](https://github.com/byx-darwin/gitflow-cli/commit/2552626e4eb29ac0aafa3518d0738b30abeeacaa)) - baoyuexing
- **(compat)** verify current upstream CLI versions - ([99f940e](https://github.com/byx-darwin/gitflow-cli/commit/99f940e22e7ddf1db2a9117d9eb455fbf51d4d82)) - baoyuexing
- **(deps)** drop temp-env after EnvSource injection - ([339d9e0](https://github.com/byx-darwin/gitflow-cli/commit/339d9e05e32c2728a888383eab1ba61f8cd4f0d4)) - baoyuexing
- **(docs)** archive overflow report files and document archiving policy - ([a6b3f37](https://github.com/byx-darwin/gitflow-cli/commit/a6b3f3776dee1717ef641ceeae4d86e2d2e77fea)) - baoyuexing
- **(gitcode)** satisfy clippy::pedantic unreadable_literal in new test - ([8b67419](https://github.com/byx-darwin/gitflow-cli/commit/8b6741924a59ba7a3584ed64b02f61f683eacaf1)) - baoyuexing
- **(gitlab,github,gitcode)** post-review polish for EnvSource injection - ([e4e319c](https://github.com/byx-darwin/gitflow-cli/commit/e4e319c7237bb0b81653e31bcf0a23c6734b9d71)) - baoyuexing
- **(merge)** align compatibility checks and smoke test - ([7972c06](https://github.com/byx-darwin/gitflow-cli/commit/7972c061e47d9b316af957a4dc635da4b8a79e1a)) - baoyuexing
- **(scripts)** remove unused import left over from Task 4 main() rewrite - ([e4267c0](https://github.com/byx-darwin/gitflow-cli/commit/e4267c0b17f1a3dcfd17db1ea5702f9d2580aebc)) - baoyuexing
- **(skills)** remove _common.sh — zero real callers, unwired shared library (#339) - ([5c36d1b](https://github.com/byx-darwin/gitflow-cli/commit/5c36d1b95d7c3c34618cb19bfaf352437d3826c2)) - baoyuexing
- **(skills)** include install.sh cleanup missed by prior commit's failed pathspec (#339) - ([9dbcd53](https://github.com/byx-darwin/gitflow-cli/commit/9dbcd53ca34eae73890ca18a263fa60db3efa7cd)) - baoyuexing
- **(supply-chain)** enforce cargo-vet gate and generate release SBOM - ([ae1038b](https://github.com/byx-darwin/gitflow-cli/commit/ae1038b4c38f6c22a301a09176026687187e02a3)) - baoyuexing
- release v2.0.0 - ([e9c1580](https://github.com/byx-darwin/gitflow-cli/commit/e9c1580981252f806ffcd3a9315546ae76ff72a4)) - baoyuexing

### Other

- **(e2e)** alert on scheduled e2e regression failure - ([3d98b8a](https://github.com/byx-darwin/gitflow-cli/commit/3d98b8a288ea4160efd69e56d6c28ac85e4f6929)) - baoyuexing
- **(gf-workflow-batch)** address 6 minor dependency resolution findings (#374) - ([1f7190f](https://github.com/byx-darwin/gitflow-cli/commit/1f7190fc450eee50e03d833266f54857f8eab8ce)) - baoyuexing
- Merge pull request #297 from byx-darwin/feat/285-pipeline-report-in-progress-run

fix(pipeline): exclude non-terminal runs from report total_runs - ([92c37ef](https://github.com/byx-darwin/gitflow-cli/commit/92c37ef4b23d75b2027fec91aa72b6e5cab66e60)) - mc-ai
- Merge branch 'dev' of github.com:byx-darwin/gitflow-cli into dev - ([ab276f1](https://github.com/byx-darwin/gitflow-cli/commit/ab276f139289be67519797af78c49d4510e3834e)) - baoyuexing
- add e2e-gitlab and e2e-gitcode jobs to e2e-tests.yml - ([a045947](https://github.com/byx-darwin/gitflow-cli/commit/a045947169d36cd83f8314fc9034daf379a1d78b)) - baoyuexing
- Merge pull request #304 from byx-darwin/feat/291-e2e-gitlab-gitcode-coverage

test(e2e): add e2e-gitlab and e2e-gitcode coverage - ([53597c0](https://github.com/byx-darwin/gitflow-cli/commit/53597c05c81c4c58a461c06ce5824997f639bd31)) - mc-ai
- Merge pull request #306 from byx-darwin/fix/291-e2e-gitlab-gitcode-ci-followup

fix(e2e-core,ci): treat empty-string secrets as absent; scope ci.yml Test job - ([e670f6b](https://github.com/byx-darwin/gitflow-cli/commit/e670f6b9ffcc82453cbc585e0d8d69f4874c9677)) - mc-ai
- Merge pull request #309 from byx-darwin/feat/292-e2e-failure-alert

ci(e2e): alert on scheduled e2e regression failure - ([8207753](https://github.com/byx-darwin/gitflow-cli/commit/8207753e6ab6a45fc0c24e2f6f40fca47ca1c711)) - mc-ai
- Merge pull request #312 from byx-darwin/feat/294-remove-run-command

chore(cli): remove deprecated run subcommand - ([da74b29](https://github.com/byx-darwin/gitflow-cli/commit/da74b29014dc1047eb3f41d64b3b693c79a7ce20)) - mc-ai
- Merge pull request #313 from byx-darwin/feat/305-pr-create-default-branch

fix(pr): detect repo default branch instead of hardcoding "main" - ([dbf68f1](https://github.com/byx-darwin/gitflow-cli/commit/dbf68f10fbe2706dfe29bb8bde3821750ad984e4)) - mc-ai
- Merge pull request #314 from byx-darwin/docs/305-fix-base-help-text

docs(pr): fix stale --base help text after default-branch detection - ([bd50e8c](https://github.com/byx-darwin/gitflow-cli/commit/bd50e8c119203546901b032cb5351c2b7c8caa44)) - mc-ai
- Merge pull request #315 from byx-darwin/feat/295-clarify-platform-naming

docs(core): clarify platform.rs is URL detection, not a unified trait - ([bba947f](https://github.com/byx-darwin/gitflow-cli/commit/bba947f446584d05d153460befae8081859e95a7)) - mc-ai
- Merge pull request #316 from byx-darwin/feat/296-cargo-vet-sbom

chore(supply-chain): enforce cargo-vet gate and generate release SBOM - ([a93f73c](https://github.com/byx-darwin/gitflow-cli/commit/a93f73c36fc1c5e76ca04a87c238ea60f8dd484d)) - mc-ai
- Merge pull request #317 from byx-darwin/feat/301-temp-file-test-isolation

test: harden temp-file-path tests against shared fixed filenames - ([cf6ab18](https://github.com/byx-darwin/gitflow-cli/commit/cf6ab18d57a7af669ebebd07b22dfe24890fba45)) - mc-ai
- Merge pull request #320 from byx-darwin/feat/310-e2e-regression-dedup-verification

ci(e2e): verify e2e-regression alert CJK title dedup reliability - ([76b17b5](https://github.com/byx-darwin/gitflow-cli/commit/76b17b5e31859382c7c631bfc087b3b91847d298)) - mc-ai
- Merge pull request #321 from byx-darwin/feat/318-worktree-symlink-exclude-guard

fix(gf-workflow): guard worktree shared symlinks against accidental commit - ([d9483cd](https://github.com/byx-darwin/gitflow-cli/commit/d9483cd09291827bc70a6d08371d73289107712e)) - mc-ai
- Merge pull request #323 from byx-darwin/feat/322-worktree-symlink-depth-fix

fix(gf-workflow): compute worktree symlink depth dynamically instead of hardcoding ../../ - ([bf91af5](https://github.com/byx-darwin/gitflow-cli/commit/bf91af524310680d74dc0073778fc2ada03fc616)) - mc-ai
- Merge pull request #326 from byx-darwin/feat/325-remove-mode1-background-agent

docs(gf-workflow): remove Mode ① background agent execution mode - ([1ea04a2](https://github.com/byx-darwin/gitflow-cli/commit/1ea04a28600a9ae307c18d6e669dda04bc974073)) - mc-ai
- Merge branch 'fix/359-envsource-injection' into dev

EnvSource injection eliminating process-global env mutation in the three
platform auth providers' tests. Closes #359. - ([388733d](https://github.com/byx-darwin/gitflow-cli/commit/388733d56efb095b016e931873c28b95011fd225)) - baoyuexing
- run doctests, which nextest never executes - ([9c1a2c9](https://github.com/byx-darwin/gitflow-cli/commit/9c1a2c92bc9b05b9f30da545b168cd2f2567a138)) - baoyuexing
- Merge branch 'feat/365-gitcode-list-pagination' into dev

fix(gitcode): make all five list commands paginate honestly (#365)

#360 claimed to eliminate silent truncation in gf's list commands but did
not fix the gitcode platform: it merely moved the threshold from 30 to 100
while still reporting truncated: false. This was found only after gitcode
CLI first became installable and was measured against a real server.

Measured with gitcode-cli 0.12.0 against openharmony/docs:
- per_page is silently capped at 100 (101 and 1001 both return 100, no error)
- --page genuinely paginates; --per-page takes precedence over --limit
- label list and milestone list DO support --page/--per-page, contrary to
  #360's assumption that made them pass no paging flags at all
- release list supports NO paging flags whatsoever, contrary to this issue's
  own AC#1, so it moves to the gitcode api endpoint instead

Changes:
- issue/pr/label/milestone list -> FetchStrategy::Paged with explicit
  --per-page/--page; --limit no longer passed
- release list -> gitcode api /repos/{repo}/releases?per_page&page, mapped
  through a ReleaseApiResponse intermediate type (ReleaseData carries
  serde(rename_all = camelCase), so reusing it directly would have failed
  with 'missing field tagName' on any repo that has releases)
- remove GITCODE_DEFAULT_LIST_LIMIT; all five fall back to DEFAULT_LIST_LIMIT
- argv-level regression tests asserting --per-page/--page and page increment,
  plus an honest-non-truncation test ending on a short page
- new read-only e2e against public openharmony/docs, demonstrated to fail
  against the pre-fix code
- e2e noauth tests made hermetic against a config-file gitcode login
- retire falsified claims from the #360 design doc and from this branch's own

Closes #365 - ([e77407d](https://github.com/byx-darwin/gitflow-cli/commit/e77407da6e0ee84e63e8c3cf28ab3b1c37e69959)) - baoyuexing
- Merge branch 'fix/gitcode-release-api-tolerance' into dev

fix(gitcode): make release api deserialization tolerate real gitcode shapes (F1)

Found by the post-delivery code review of wf-2026-09-18-001.

Both the ReleaseApiResponse doc comment and design §7 claimed that
#[serde(default)] on every field made shape mismatches degrade per-field
rather than failing wholesale. That was false: serde(default) only covers
ABSENT fields. A present-but-wrong-typed or null field failed the whole
array, and with it the entire 'gf release list' command.

The risk was concrete, not theoretical: this crate already knows gitcode
encodes numeric ids as JSON strings (IssueApiResponse.number is a String,
and ReleaseUserApi.id was given a string-or-number deserializer for exactly
that reason) — yet the release's own id was left a bare u64. /releases is
also the one path with no real-server verification.

Changes:
- id tolerates both number and string encodings, and null
- tag_name / draft / prerelease / author.login tolerate null
- both false 'degrades predictably' claims corrected to state the real
  tolerance boundary, in each direction: every Option field tolerates null,
  while a present-but-wrong-typed value, a non-RFC3339 timestamp, and a null
  ReleaseUserApi.id (core deserializer, out of scope) still fail
- 5 tests pinning that boundary so docs and code cannot drift again

This was the same class of error that caused #365 itself: an unverified
inference written down as a settled fact — this time inside the section
meant to record honest gaps.

Refs #365 - ([95fa1dc](https://github.com/byx-darwin/gitflow-cli/commit/95fa1dc50560f41751528ca038e4ad11ec5e0b32)) - baoyuexing
- Merge branch 'feat/330-issue-decompose' into dev - ([0897585](https://github.com/byx-darwin/gitflow-cli/commit/0897585a35e5c562c64013b0e3006b9754fa423e)) - baoyuexing
- Merge branch 'feat/331-gf-architecture-diagram' into dev

Adds skills/gf-architecture-diagram/: dependency-derived, geometrically
verified, deterministic architecture-diagram generation, replacing the
hand-maintained docs/architecture-diagram.dot. Regenerates this repo's
own diagram from real cargo metadata output as a dogfooding pass.

Closes #331 - ([cc5ec3e](https://github.com/byx-darwin/gitflow-cli/commit/cc5ec3e58b3d471b9858f838b2bc5a1c246e35fa)) - baoyuexing
- Merge branch 'feat/333-evidence-grading' into dev

Backfills the Measured/Inferred/Unverified evidence-grading vocabulary
and failing-test commit-ancestry table (established by #329) into
gf-quality's gate report and gf-pr-review's dimension assessment.

Closes #333 - ([ecd533a](https://github.com/byx-darwin/gitflow-cli/commit/ecd533a521e401e154032aaf83d14a75338edb47)) - baoyuexing
- Merge branch 'feat/333-evidence-grading' into dev

Follow-up fix for Issue #333: post-delivery code review (Finding 1)
caught the ancestry-check instructions still leaking inside the
literal report-template fence after the first fence fix. Moved them
fully outside the template, mirroring gf-walkthrough's own pattern. - ([290f1cf](https://github.com/byx-darwin/gitflow-cli/commit/290f1cfe14b7d7d0bf29cad7c0808168ab1f4320)) - baoyuexing
- Merge branch 'feat/334-review-loop-contract' into dev

Closes #334 - ([e49106a](https://github.com/byx-darwin/gitflow-cli/commit/e49106a6d0fb4dd19fefc4945ccf9a1080a82aa5)) - baoyuexing
- Merge branch 'feat/335-vertical-slice-falsifiable-acceptance' into dev - ([2350511](https://github.com/byx-darwin/gitflow-cli/commit/23505114b1431bae0618a4666f034e623dfc927e)) - baoyuexing
- Merge branch 'feat/336-error-classification-retry-cap' into dev - ([6e95d39](https://github.com/byx-darwin/gitflow-cli/commit/6e95d390d38c6c69b2be581b016ca12045873032)) - baoyuexing
- Merge branch 'feat/337-gf-workflow-batch-topo-order' into dev

gf-workflow-batch now parses `Blocked by: #N` edges across all open Issues,
resolves blocker completion via issue state, detects cycles via three-color
DFS (abort before dispatching anything on any error), and derives a `ready`
set for the dispatch loop instead of dispatching pending[0] blind to
dependency order.

Closes #337 - ([3013139](https://github.com/byx-darwin/gitflow-cli/commit/30131395dd3a312f63fe21b436721d80a14bb187)) - baoyuexing
- Merge branch 'feat/372-gitlab-adapter-batch1' into dev

Batch1: three independent GitLab adapter fixes bundled into one PR.

- fix(gitlab): normalize label color to # prefix in create() and edit(),
  fix help text (#372)
- feat(core): add optional per-host breakdown to AuthStatus (#362)
- fix(gitlab): parse per-host auth status, stop collapsing mixed hosts to
  unauthenticated (#362)
- fix(gitlab): always pass --description to glab issue create, fixing
  no-body 100% failure (#375)

Closes #372
Closes #362
Closes #375 - ([4b0aba3](https://github.com/byx-darwin/gitflow-cli/commit/4b0aba316a108865c1105d8825941105accc3874)) - baoyuexing
- Merge branch 'feat/364-milestone-api-casing' into dev

Fix MilestoneApiResponse naming-direction bug on GitHub and GitCode:
rename_all="camelCase" was applied to an inbound type that must match
upstream's real snake_case field names, silently zeroing due_on /
closed_issues / open_issues via #[serde(default)].

- fix(github): MilestoneApiResponse match upstream snake_case (#364)
- fix(gitcode): same defect found via audit, confirmed with real API data
- 17-type audit across github/gitlab/gitcode *ApiResponse recorded in
  docs/superpowers/specs/2026-09-19-milestone-api-response-casing-design.md

Closes #364 - ([6717cc4](https://github.com/byx-darwin/gitflow-cli/commit/6717cc49c3ed96e34f48a885f634eb933e6e8e98)) - baoyuexing
- add timeout and failure diagnostics to Test job matrix (#373) - ([7cb9caf](https://github.com/byx-darwin/gitflow-cli/commit/7cb9cafde7986ee5a8796d2548708fb0840051a6)) - baoyuexing
- Merge branch 'feat/373-ci-test-job-hardening' into dev

Investigation found no systemic Windows/macOS CI infra problem: most of
the historical "instability" signal across 5 pipeline reports was an
artifact of a separate, already-known gf pipeline report bug that miscounts
in-progress jobs as failures (split out to Issue #378). Only one genuine,
non-reproducible Windows test failure occurred in the last ~3 weeks.

Adds job-level timeout and failure diagnostics upload to the Test job
matrix as defensive hardening, per the investigation's conclusion.

Closes #373 - ([dba7353](https://github.com/byx-darwin/gitflow-cli/commit/dba735331d44e1896b7b224228f036428aee6605)) - baoyuexing
- Merge branch 'feat/374-gf-workflow-batch-dependency-hardening' into dev

Address 6 minor findings from #337 review on gf-workflow-batch's
Dependency Resolution pseudocode (skills/gf-workflow-batch/references.md):

- reuse `body` already returned by list instead of a redundant `view` call
- document why the `Blocked by:` regex intentionally stays strict (no fix)
- run dependency resolution only when `pending` is non-empty, so unrelated
  cycles/bad references can't block Discussion Mode
- rewrite cycle-detection DFS as explicit-stack iteration (no RecursionError)
- distinguish "genuinely blocked" from "--limit cutoff" in the post-loop
  summary (fixed during final review: the first version misreported --limit
  exits as blocked)

Closes #374 - ([a9a550f](https://github.com/byx-darwin/gitflow-cli/commit/a9a550f68e49edb483085d87c7c7fe81dd164e57)) - baoyuexing
- Merge branch 'feat/353-skills-tooling-batch5' into dev

Four independent mechanical fixes to skill/tooling infrastructure:

- fix(gf-workflow): split worktree symlink depth formula for
  .cache/workflows vs .claude — the old shared formula left .claude
  dangling in every worktree this session created (#353)
- fix(makefile): check-skills-drift now diffs content, not just
  directory presence (#363)
- fix(makefile): check-walkthrough-skill no longer drops adjacent
  [Measured] entries via getline (#355)
- fix(skills): move gf-pr's illegal frontmatter key into See Also (#338)

Issue #350 was originally in scope but pulled out after discovering it
was closed prematurely without checking two of its three sub-problems
(word-count/inline-code contradiction + 26/30 skills over the 500-word
limit) — reopened, left for a dedicated follow-up since it needs a
policy decision, not a mechanical fix.

Closes #353
Closes #363
Closes #355
Closes #338 - ([b38bb50](https://github.com/byx-darwin/gitflow-cli/commit/b38bb508fcb67a2cf71ad1ddf40ffc841bd1b34d)) - baoyuexing
- Merge branch 'feat/369-skills-tooling-batch6' into dev

Three user-decided infrastructure changes:

- fix(tooling): cargo-fmt pre-commit hook now writes back instead of
  only checking (#369) — user chose auto-write-back over the two other
  presented options; verified with a real probe commit showing the
  expected "files were modified by this hook" / re-add flow
- chore(skills): remove _common.sh — zero real callers, unwired shared
  library (#339)
- fix(makefile): install-skills now syncs docs/ so ../../docs/...
  skill references resolve after install (#352)

Closes #369
Closes #339
Closes #352 - ([5ffc96d](https://github.com/byx-darwin/gitflow-cli/commit/5ffc96db626a814ffd0b04b9d00271e12e5c452b)) - baoyuexing
- Merge branch 'feat/361-dogfooding-noauth-tests' into dev

- fix(docs): correct dogfooding checklist command params to match
  actual CLI — --tag-name/--body instead of positional tag/--notes,
  --label instead of --labels (#361)
- fix(e2e): noauth tests for GitHub/GitLab skip gracefully when
  gh/glab CLI isn't installed instead of failing with a misleading
  "expected login guidance" message (#356)

Closes #361
Closes #356 - ([5a2f955](https://github.com/byx-darwin/gitflow-cli/commit/5a2f95514676bb6dd412c58cd6fa071a1a33cfac)) - baoyuexing
- Merge branch 'feat/378-pipeline-report-version-traceability' into dev

Closes #378 - ([a9da23f](https://github.com/byx-darwin/gitflow-cli/commit/a9da23f70ac134e6a1d74544835bd95f41775ce4)) - baoyuexing
- Merge branch 'fix/377-gitcode-milestone-due-on-pure-date' into dev

Closes #377 - ([1bf8c67](https://github.com/byx-darwin/gitflow-cli/commit/1bf8c6704c30dac955498ed11513a83c5a4ea9b0)) - baoyuexing
- Merge branch 'test/368-gitcode-release-real-verification' into dev

Closes #368 - ([4bba6b5](https://github.com/byx-darwin/gitflow-cli/commit/4bba6b50c9c8fac7addb043ad6ca267656d29e1a)) - baoyuexing
- Merge branch 'refactor/366-release-created-at-optional' into dev

Closes #366 - ([fa31d27](https://github.com/byx-darwin/gitflow-cli/commit/fa31d279e32153a5dcdafced921d50fbb50d05f6)) - baoyuexing
- Merge branch 'feat/357-milestone-issue-pr-attachment' into dev

Closes #357 - ([48d1acb](https://github.com/byx-darwin/gitflow-cli/commit/48d1acb7f289bd6de47b425e2ac0a7b73811d9d9)) - baoyuexing
- Merge branch 'feat/350-skill-word-limit' into dev

Closes #350 - ([74e1449](https://github.com/byx-darwin/gitflow-cli/commit/74e14491c179f7948fd2d798c3fc53cd97e180bd)) - baoyuexing
- Merge branch 'feat/344-security-regression-gate' into dev

Closes #344 - ([bb8318b](https://github.com/byx-darwin/gitflow-cli/commit/bb8318bcab8cda7dbfbd4250c6b2a88bee4d3da9)) - baoyuexing
- Merge branch 'feat/345-workflow-dashboard' into dev

Closes #345 - ([d352cc3](https://github.com/byx-darwin/gitflow-cli/commit/d352cc38c6577f454b5596427d0f3a495c02ee5b)) - baoyuexing
- Merge branch 'fix/345-card-html-fault-isolation' into dev

Phase 4 review fix-round for Issue #345. - ([cdf7946](https://github.com/byx-darwin/gitflow-cli/commit/cdf7946a7112172815ee3051d4b803469888bb18)) - baoyuexing
- Merge branch 'feat/346-diff-review-renderer' into dev

Closes #346 - ([1b6bf81](https://github.com/byx-darwin/gitflow-cli/commit/1b6bf81eb264c6b80de292283981f56abd4806a4)) - baoyuexing
- Merge branch 'fix/394-pr-close-reopen-deserialize' into dev

Closes #394 - ([879cc43](https://github.com/byx-darwin/gitflow-cli/commit/879cc4328bd34ddcfba18928506255c1fb6af9f8)) - baoyuexing
- Merge branch 'fix/396-milestone-project-flag' into dev

Closes #396 - ([fb2e290](https://github.com/byx-darwin/gitflow-cli/commit/fb2e2902fc18a2d1ca8048aa6b78d02db66e88a6)) - baoyuexing
- Merge branch 'fix/395-milestone-close-reopen' into dev

Closes #395 - ([ba7f114](https://github.com/byx-darwin/gitflow-cli/commit/ba7f114cc24243b5a16265de629285828f19341b)) - baoyuexing
- Merge branch 'fix/399-diff-review-annotations' into dev - ([b645c90](https://github.com/byx-darwin/gitflow-cli/commit/b645c90ecd9dac74e6400a6d5a97687a829aa631)) - baoyuexing
- Merge branch 'refactor/380-created-at-optional' into dev - ([2c2a427](https://github.com/byx-darwin/gitflow-cli/commit/2c2a4276c657a9400a54fe8688ad7d457f65f3b2)) - baoyuexing
- Merge codex/fix-401 into dev - ([f8f2c81](https://github.com/byx-darwin/gitflow-cli/commit/f8f2c811c1ee742b4caaecf0b0f183010e60370e)) - baoyuexing
- Merge codex/fix-370 into dev - ([12a99af](https://github.com/byx-darwin/gitflow-cli/commit/12a99af187041a69b12f2f70a9758bdb7ddcccd1)) - baoyuexing
- Merge codex/feat-397 into dev - ([c1e776d](https://github.com/byx-darwin/gitflow-cli/commit/c1e776dc1947ff990b922d61dabc7979025f3f77)) - baoyuexing
- Merge codex/check-240 into dev - ([2a16526](https://github.com/byx-darwin/gitflow-cli/commit/2a16526197040fee6cf8e450e634900d1a6bcbb6)) - baoyuexing
- Merge codex/docs-371 into dev - ([803ea45](https://github.com/byx-darwin/gitflow-cli/commit/803ea455975bad62e6166403cd3af2115248b986)) - baoyuexing
- Merge codex/feat-347 into dev - ([7eaf2f4](https://github.com/byx-darwin/gitflow-cli/commit/7eaf2f4e46b0a693c55d8d0682f84f506cd2b05f)) - baoyuexing
- Merge codex/feat-398 into dev - ([62d58a7](https://github.com/byx-darwin/gitflow-cli/commit/62d58a713e762aff757c83ff75b4ed2230a39b25)) - baoyuexing
- Merge codex/refactor-349 into dev - ([c78a016](https://github.com/byx-darwin/gitflow-cli/commit/c78a01654abd034c03ac36c8b4eac2984b5222da)) - baoyuexing
- Merge codex/issue-387-quality-precheck into dev - ([e31ab9c](https://github.com/byx-darwin/gitflow-cli/commit/e31ab9c257a8b0382092aa63b9e6776df797657e)) - baoyuexing
- Merge codex/issue-386-pr-precheck into dev - ([4ef7f25](https://github.com/byx-darwin/gitflow-cli/commit/4ef7f259d187c265b8c3fd10f625d1d6dafc3a51)) - baoyuexing
- Merge codex/issue-385-pipeline-failure-analysis into dev - ([3a34d4d](https://github.com/byx-darwin/gitflow-cli/commit/3a34d4d8f67423652a7647d9a09a332b8cd5d3fe)) - baoyuexing
- Merge codex/issue-388-workflow-semantic-rules into dev - ([7261562](https://github.com/byx-darwin/gitflow-cli/commit/7261562030ebabce38be3261049c024a5e860b49)) - baoyuexing
- Merge codex/issue-389-semantic-regression into dev - ([0944f69](https://github.com/byx-darwin/gitflow-cli/commit/0944f69cfc2edee375441b08caf7d6bd55359254)) - baoyuexing
- Merge codex/issue-390-typed-query into dev - ([c41ced1](https://github.com/byx-darwin/gitflow-cli/commit/c41ced1cae4bf8846d593679bda61713d2bdfe2e)) - baoyuexing
- Merge codex/issue-391-context-compaction into dev - ([2c52bd2](https://github.com/byx-darwin/gitflow-cli/commit/2c52bd226f97964b67e6b8da0b5d4110ad926f8b)) - baoyuexing
- issue #392 workflow progress assessment - ([4e27fe7](https://github.com/byx-darwin/gitflow-cli/commit/4e27fe769b4d624ad82fac039b941c5df0b5f9e3)) - baoyuexing
- issue #402 glab 1.118.0 compatibility - ([548a0e3](https://github.com/byx-darwin/gitflow-cli/commit/548a0e3e6b67d4f631cb30224e02eb10b8be5766)) - baoyuexing
- issue #227 gh 2.98.0 compatibility - ([29fc2bf](https://github.com/byx-darwin/gitflow-cli/commit/29fc2bf775e95e697c76e00d6a2415bbcc8af1fa)) - baoyuexing
- issue #188 GitCode CLI 0.11.1 compatibility - ([605906d](https://github.com/byx-darwin/gitflow-cli/commit/605906da4a644ebbf8b0e3b272fc784942c93509)) - baoyuexing

### Refactoring

- **(core,gitlab,gitcode)** [**breaking**] make ReleaseData.created_at optional - ([88798a7](https://github.com/byx-darwin/gitflow-cli/commit/88798a707bfcee68e74b75ed8d322726b0228aac)) - baoyuexing
- **(core,gitlab,gitcode,github)** [**breaking**] make issue/pr/review/comment/pipeline timestamps optional - ([2e119c2](https://github.com/byx-darwin/gitflow-cli/commit/2e119c241c5c8d70b59b817a7a8a776aefc2fea5)) - baoyuexing
- **(gf-walkthrough)** externalize report template, compress to 499 words - ([c25a837](https://github.com/byx-darwin/gitflow-cli/commit/c25a837e664d419535ce7e0986f2e5c5fc1fdfa1)) - baoyuexing
- **(skills)** centralize language profiles - ([370daea](https://github.com/byx-darwin/gitflow-cli/commit/370daea561ada2d28407f847db8518838a40a525)) - baoyuexing

### Style

- **(skills)** match gf-pr See Also bullet style (final review finding) - ([58f1eba](https://github.com/byx-darwin/gitflow-cli/commit/58f1eba387e3b627e7167d7d4658648e2f527c1b)) - baoyuexing

### Tests

- **(commit)** use NamedTempFile instead of fixed shared temp filename - ([4d95286](https://github.com/byx-darwin/gitflow-cli/commit/4d952863b9afd758350b3faf59ab9116d4841e40)) - baoyuexing
- **(core)** cover N+1 probe when cap is a multiple of per_page - ([7336be7](https://github.com/byx-darwin/gitflow-cli/commit/7336be7d317b2285a20d0a58dba9e9c3db9fdcb2)) - baoyuexing
- **(decision)** report Jev triage pilot and harden input - ([3ab490d](https://github.com/byx-darwin/gitflow-cli/commit/3ab490d8b2a9b3e85fc4a64b4c62840111a6e2d9)) - baoyuexing
- **(e2e-gitcode)** make noauth tests hermetic against config-file login - ([363fa14](https://github.com/byx-darwin/gitflow-cli/commit/363fa14fd067a0ae9ca192aee47726e9497eeb57)) - baoyuexing
- **(e2e-gitcode)** assert issue list pages past the 100-item API cap - ([aac2d3c](https://github.com/byx-darwin/gitflow-cli/commit/aac2d3c1a7d8774761b3326428f0ee2b76374c26)) - baoyuexing
- **(gf-walkthrough)** add check-walkthrough-skill validator (RED) - ([8c889bb](https://github.com/byx-darwin/gitflow-cli/commit/8c889bb1829eb3dba1264901edbdb44e3704fc82)) - baoyuexing
- **(gitcode)** record argv in SequencedMockCommandRunner - ([784b31d](https://github.com/byx-darwin/gitflow-cli/commit/784b31d6ee375a35e3240ee127bf7a76fc300de2)) - baoyuexing
- **(gitcode)** pin real release API response shape (Issue #368) - ([91f4c7a](https://github.com/byx-darwin/gitflow-cli/commit/91f4c7a9b9adcbd7b1620f66553afa70637d7d0d)) - baoyuexing
- **(github)** pin that missing createdAt degrades to None, not an error - ([9c03c57](https://github.com/byx-darwin/gitflow-cli/commit/9c03c57af03317cbfb0347bb145107e9de0ad6bc)) - baoyuexing
- **(issue)** use NamedTempFile instead of fixed shared temp filename - ([ff22799](https://github.com/byx-darwin/gitflow-cli/commit/ff22799e45b538c8519f6a71c74b6c10844d5d73)) - baoyuexing
- **(pr)** use NamedTempFile instead of fixed shared temp filename - ([da8c33c](https://github.com/byx-darwin/gitflow-cli/commit/da8c33c61d983ce635efc5028c99fb178557b8b2)) - baoyuexing
- **(quality-review)** add check-quality-review-evidence-skill assertion (RED) - ([e646b84](https://github.com/byx-darwin/gitflow-cli/commit/e646b84ca54a5dcc2097f2340c5525be9bd678bc)) - baoyuexing
- **(regression)** add advisory semantic oracle - ([251e5f5](https://github.com/byx-darwin/gitflow-cli/commit/251e5f568bb6c88869141db6237a30c3ffdf65b1)) - baoyuexing
- **(release)** use NamedTempFile instead of fixed shared temp filename - ([b36d6d5](https://github.com/byx-darwin/gitflow-cli/commit/b36d6d5d72080715f4253b182ffc08df0746bba6)) - baoyuexing
- **(scripts)** cover HTML-injection escaping for rename old_path and error message - ([5c7a04b](https://github.com/byx-darwin/gitflow-cli/commit/5c7a04bb7fff36657a1b16d13ab2f4f9324b19da)) - baoyuexing

---
## [1.9.0](https://github.com/byx-darwin/gitflow-cli/compare/v1.8.0..v1.9.0) - 2026-09-02

### Bug Fixes

- **(gf-workflow-batch)** trim SKILL.md to meet 500-word budget - ([40918bf](https://github.com/byx-darwin/gitflow-cli/commit/40918bf2a54d456e3cea732771ec344cdfdca76d)) - baoyuexing
- **(gf-workflow-batch)** correct stale Rationalization Excuses citation in test scenario 5 - ([ab7ac66](https://github.com/byx-darwin/gitflow-cli/commit/ab7ac66350986a72abf71e9a2e18d57b934d1232)) - baoyuexing
- **(gf-workflow-batch)** correct mislabeled Rationalization citation in test scenario 2 - ([d6eca90](https://github.com/byx-darwin/gitflow-cli/commit/d6eca90e13f035fb2f209c6e3c813f9f95cc908d)) - baoyuexing
- **(gf-workflow-batch)** address skill validation findings - ([500fc49](https://github.com/byx-darwin/gitflow-cli/commit/500fc4981c43085e0f364d68c1a108828dbf9309)) - baoyuexing
- **(gf-workflow-batch)** address final review findings (--limit, failure memory, stale citations, index format) - ([a47094c](https://github.com/byx-darwin/gitflow-cli/commit/a47094cef6c967c12548162756501e5637b070f9)) - baoyuexing
- remove remaining dead bug-report producer and stale design-spec references - ([8a14404](https://github.com/byx-darwin/gitflow-cli/commit/8a14404ec64a586db300d1bf1c5f3939fc36bdbb)) - baoyuexing

### Documentation

- **(gf-workflow-batch)** add pending-derivation and discussion-mode reference - ([2b16a30](https://github.com/byx-darwin/gitflow-cli/commit/2b16a30a6888dfd22a8986d4295d0ca2fc29cfe4)) - baoyuexing
- **(workflow)** add design doc and implementation plan for autoreport-bug removal (#278) - ([94068cd](https://github.com/byx-darwin/gitflow-cli/commit/94068cd0f19eccc2f9faec2f539a2daa99146f3f)) - baoyuexing
- remove stale auto-report-bug and co-contribution references - ([d3dfcf3](https://github.com/byx-darwin/gitflow-cli/commit/d3dfcf369370bfb862f72d1bb3384a205a57f7a1)) - baoyuexing
- fix residual auto-filing implications in gf-regression skill - ([48dd784](https://github.com/byx-darwin/gitflow-cli/commit/48dd784d63471cf1601f7d236c48370176df90b8)) - baoyuexing
- add Phase 4 pipeline analysis and code review reports for PR #279 - ([9a785a3](https://github.com/byx-darwin/gitflow-cli/commit/9a785a3c5a96ede855ebc02a44c8326a8e7b2bda)) - baoyuexing
- add gf-workflow-batch design spec (#280) - ([052b69e](https://github.com/byx-darwin/gitflow-cli/commit/052b69ed6266a23482b534cf07723079a07a745f)) - baoyuexing
- add gf-workflow-batch implementation plan (#280) - ([20b87da](https://github.com/byx-darwin/gitflow-cli/commit/20b87da606ed2de9c81e4f6b181d39aecc5a88f2)) - baoyuexing
- index gf-workflow-batch skill - ([600fb88](https://github.com/byx-darwin/gitflow-cli/commit/600fb885229787fe121e6f4ad2bef85ce764c437)) - baoyuexing
- add Phase 4 reports for PR #281 (pipeline, triage, review) - ([108e56d](https://github.com/byx-darwin/gitflow-cli/commit/108e56dc5d44fb32b146dab3baba18860c3ab73c)) - baoyuexing

### Features

- **(gf-workflow-batch)** add skill scaffold with core pattern and gates - ([4ca3577](https://github.com/byx-darwin/gitflow-cli/commit/4ca35775f09a5082d1b45c3e050c059516050b01)) - baoyuexing

### Miscellaneous Chores

- **(install)** remove dead Stop Hook registration for deleted auto-report-bug script - ([d71c4a2](https://github.com/byx-darwin/gitflow-cli/commit/d71c4a2ab528d26cdad2f3dafd8d006ef830497c)) - baoyuexing
- sync main back to dev after release v1.8.0 - ([d7c05cc](https://github.com/byx-darwin/gitflow-cli/commit/d7c05ccf32e41ca1e97297357eb693a2fe558946)) - baoyuexing
- remove auto-report-bug hook script and gf-autoreport-bug skill - ([c587ec9](https://github.com/byx-darwin/gitflow-cli/commit/c587ec972ab4a346e8b924b377b5513d8e65aecf)) - baoyuexing
- remove stale pending.json comment in json_output_test - ([26dbec4](https://github.com/byx-darwin/gitflow-cli/commit/26dbec469c7cb28f97d167cb57bc888f88e22370)) - baoyuexing
- release v1.9.0 (#283) - ([61035dd](https://github.com/byx-darwin/gitflow-cli/commit/61035dddffbd2046c8ca485c04a181003bdd1e68)) - mc-ai

### Other

- Merge pull request #281 from byx-darwin/feat/280-gf-workflow-batch

feat(gf-workflow-batch): add serial batch driver for multiple open Issues - ([c07550b](https://github.com/byx-darwin/gitflow-cli/commit/c07550bf12832053efc6fafea525574f37fc816a)) - mc-ai
- Merge pull request #282 from byx-darwin/dev

chore: sync dev to main for release - ([01a8cfc](https://github.com/byx-darwin/gitflow-cli/commit/01a8cfc214408708f01dae70c7663050388b8440)) - mc-ai

### Refactoring

- **(cli)** remove error_reporter, co-contribution and autoreport-bug hook install logic - ([a18ce11](https://github.com/byx-darwin/gitflow-cli/commit/a18ce117c799a05519b257027e98618946014179)) - baoyuexing

### Tests

- **(gf-workflow-batch)** add stress test scenarios - ([89e2d84](https://github.com/byx-darwin/gitflow-cli/commit/89e2d8479e69a5233fd168070a9c2c59b1615303)) - baoyuexing

---
## [1.8.0](https://github.com/byx-darwin/gitflow-cli/compare/v1.7.0..v1.8.0) - 2026-09-01

### Bug Fixes

- **(autoreport)** prune archived pending.json reports beyond retention cap - ([d8f6a49](https://github.com/byx-darwin/gitflow-cli/commit/d8f6a4963cd63bb54fdbbb654120297e771a3964)) - baoyuexing
- **(autoreport)** skip error reporting when running in CI - ([ed4132d](https://github.com/byx-darwin/gitflow-cli/commit/ed4132da18106142225743e3c104e8be46a2b980)) - baoyuexing
- **(autoreport)** fail loud when the auto-report label is missing - ([e948140](https://github.com/byx-darwin/gitflow-cli/commit/e948140a7bec3eec6e555e3aca58fafa79c7aa1f)) - baoyuexing
- **(autoreport)** make label-missing bats assertions actually gate pass/fail - ([e4c36a3](https://github.com/byx-darwin/gitflow-cli/commit/e4c36a35e0daad38834cf2173eb00b252b23fde0)) - baoyuexing
- **(autoreport)** parametrize target repo from Cargo.toml instead of hardcoding it - ([9ab9a49](https://github.com/byx-darwin/gitflow-cli/commit/9ab9a49d50b1fc0ed1b93ac1a10b83f2f38172ad)) - baoyuexing
- **(autoreport)** validate each repo-slug segment is non-empty, not just the whole remainder - ([ae3177b](https://github.com/byx-darwin/gitflow-cli/commit/ae3177b7a414549467fe9e7694e50c1a2c032c2e)) - baoyuexing
- **(autoreport)** redact credential-named env var values and sk-/glpat- tokens - ([3c17293](https://github.com/byx-darwin/gitflow-cli/commit/3c172935838cb0e67d8ed4bd0ca361b1cdee59d9)) - baoyuexing
- **(autoreport)** require project-level confirmation for a global-only opt-in - ([7129e6b](https://github.com/byx-darwin/gitflow-cli/commit/7129e6b46bfca2337ea2752698095c270b80bb44)) - baoyuexing
- **(autoreport-bug)** address final-review findings across sanitizer, doctor, and slug validation - ([949ee3e](https://github.com/byx-darwin/gitflow-cli/commit/949ee3ee4b46e83ad17c3ac2bfb3a0591f356ca8)) - baoyuexing
- **(cli)** validate --body-file with SafePath in resolve_body - ([36fdd84](https://github.com/byx-darwin/gitflow-cli/commit/36fdd84d437e663736908062b15c0ef9e5a90501)) - baoyuexing
- **(cli)** use git remote URL as GitLab --repo target for issue commands - ([a82837a](https://github.com/byx-darwin/gitflow-cli/commit/a82837ad36412309f1607f0b20589bcde914aa04)) - baoyuexing
- **(cli)** use git remote URL as GitLab --repo/--project target for pr/release/pipeline/label/milestone commands - ([c37f831](https://github.com/byx-darwin/gitflow-cli/commit/c37f831f8ad357c17086a03119cf2d55c54618bb)) - baoyuexing
- **(cli)** allow too_many_lines on handle_label after remote_url branching - ([dbe47ad](https://github.com/byx-darwin/gitflow-cli/commit/dbe47ad2f1ab990811afbd06bb9c43c762c4c5fd)) - baoyuexing
- **(core)** allow Windows drive-letter colon in SafePath::new_allow_absolute - ([d24a502](https://github.com/byx-darwin/gitflow-cli/commit/d24a5020f4dd9087438839d5bddd332eb65b0e0b)) - baoyuexing
- **(gf-autoreport-bug)** resolve 5 cross-cutting findings from final review - ([6abd60b](https://github.com/byx-darwin/gitflow-cli/commit/6abd60bf4741bc4c5f894898a8e06c3429123727)) - baoyuexing
- **(gf-workflow)** restore Phase 4 output phrasing dropped by parallel-dispatch rewrite - ([3023c13](https://github.com/byx-darwin/gitflow-cli/commit/3023c132683679fb7fc51ec3369b4300b87ab94f)) - baoyuexing
- **(gitlab)** use glab issue update --label/--unlabel instead of nonexistent issue edit - ([72a1662](https://github.com/byx-darwin/gitflow-cli/commit/72a1662a71fe8ff2052bc460f58092615a4da80e)) - baoyuexing
- **(gitlab)** route add_labels/remove_label --repo through repo_target - ([c99a753](https://github.com/byx-darwin/gitflow-cli/commit/c99a7532b54c925f0320c4067e2bd0c9b7fc38ea)) - baoyuexing
- **(gitlab)** route remaining issue verbs' --repo through repo_target - ([d8d16ba](https://github.com/byx-darwin/gitflow-cli/commit/d8d16ba352354b40862403e2af21080935f864f3)) - baoyuexing
- **(gitlab)** log raw glab stderr on CLI failure for diagnosability - ([d9a2169](https://github.com/byx-darwin/gitflow-cli/commit/d9a216900db09b99e181785703698df0bfc6cdc8)) - baoyuexing
- **(gitlab)** route GitLabMrProvider --repo through repo_target - ([4ca5fbb](https://github.com/byx-darwin/gitflow-cli/commit/4ca5fbb9d40b9d6c3bf54eeac0763a8df694ccc2)) - baoyuexing
- **(gitlab)** route GitLabReleaseProvider --repo through repo_target - ([744faa4](https://github.com/byx-darwin/gitflow-cli/commit/744faa45991c360164e6b0de863387138f85441f)) - baoyuexing
- **(gitlab)** route GitLabPipelineProvider --repo through repo_target - ([6b49182](https://github.com/byx-darwin/gitflow-cli/commit/6b4918254a52b5dbf6375f684fbb80bb917094c6)) - baoyuexing
- **(gitlab)** route GitLabLabelProvider --repo through repo_target - ([ada8e8d](https://github.com/byx-darwin/gitflow-cli/commit/ada8e8dde57c2572624dd0b34b6c1b0cb30545b4)) - baoyuexing
- **(gitlab)** route GitLabMilestoneProvider --project through project_target - ([cd35797](https://github.com/byx-darwin/gitflow-cli/commit/cd35797162769f2e4fc00ecb9a876ec585f22ed1)) - baoyuexing
- **(hooks)** resolve HOOK_SCRIPT via BATS_TEST_DIRNAME, not BASH_SOURCE - ([0d5e32b](https://github.com/byx-darwin/gitflow-cli/commit/0d5e32bfd14431e13482b564e49e346a76942e63)) - baoyuexing
- **(pr)** name the scheduled merge instead of returning an empty message - ([49b21ec](https://github.com/byx-darwin/gitflow-cli/commit/49b21eca8e2a74284d0d58ca4d3f58085de395d2)) - baoyuexing
- **(security)** validate --path and --body-file with SafePath - ([197861f](https://github.com/byx-darwin/gitflow-cli/commit/197861f9748d48e8d3659df228bf11d962557701)) - baoyuexing
- **(security)** validate release asset paths with SafePath - ([ec7cf2e](https://github.com/byx-darwin/gitflow-cli/commit/ec7cf2e247c350ff1f7ea4d40f154db686a283ef)) - baoyuexing
- **(workflow)** add worktree preflight before git worktree add - ([4f93be7](https://github.com/byx-darwin/gitflow-cli/commit/4f93be7401b439a9186b97b802000b930d9700f3)) - baoyuexing

### Documentation

- **(autoreport)** add hardening design spec and implementation plan - ([1a2480e](https://github.com/byx-darwin/gitflow-cli/commit/1a2480e68d2301adfd543ae7c6ce4272974b1aaa)) - baoyuexing
- **(autoreport)** default unattended preview to skip, document CI/label gates - ([5fe1d0e](https://github.com/byx-darwin/gitflow-cli/commit/5fe1d0eb7d7b482ccc07f5c9cd51525c9cc0d896)) - baoyuexing
- **(autoreport)** add global-install hardening design spec and plan - ([f2092db](https://github.com/byx-darwin/gitflow-cli/commit/f2092db3c19ab9f244aa0c293708bb349931ee9b)) - baoyuexing
- **(gf-workflow)** tune subagent thresholds and Phase 4 report verbosity for token cost - ([e9f7bdb](https://github.com/byx-darwin/gitflow-cli/commit/e9f7bdbe3b277438d09939a77f1cdd114900f220)) - baoyuexing
- **(index)** index missing docs/ entries by category - ([66ae420](https://github.com/byx-darwin/gitflow-cli/commit/66ae420db76b8bbec0f710a6ee23a2960275e547)) - baoyuexing
- **(specs)** index specs/README.md - ([8e4575f](https://github.com/byx-darwin/gitflow-cli/commit/8e4575f42e2b66decd15952c2787febe4f243641)) - baoyuexing
- add design doc for gf-workflow local-merge delivery path (#265) - ([6f1e3f9](https://github.com/byx-darwin/gitflow-cli/commit/6f1e3f92811ee000ebfb10913efd27c5911e2cf2)) - baoyuexing
- add implementation plan for gf-workflow local-merge delivery path (#265) - ([30d2af1](https://github.com/byx-darwin/gitflow-cli/commit/30d2af1e67a2cc15201ee6eff11083a7f4eb9396)) - baoyuexing
- add Phase 4 pipeline analysis and code review reports for PR #268 - ([20185e0](https://github.com/byx-darwin/gitflow-cli/commit/20185e054596f70df2fcbce8b6b367030cac4e9d)) - baoyuexing
- add design doc for gf issue edit subcommand (#266) - ([f5205da](https://github.com/byx-darwin/gitflow-cli/commit/f5205da062db1b17746bf7595358db0f37c41320)) - baoyuexing
- add implementation plan for gf issue edit subcommand (#266) - ([6956e25](https://github.com/byx-darwin/gitflow-cli/commit/6956e2511397b0161038bc3a43570492640d038c)) - baoyuexing
- add Phase 4 analysis reports for PR #269 (#266) - ([2701b3d](https://github.com/byx-darwin/gitflow-cli/commit/2701b3dd6ef2d6ce6fc6b29e50cd4b2ac5004f48)) - baoyuexing
- add design doc for GitLab label mutation fix (#270) - ([ba5d589](https://github.com/byx-darwin/gitflow-cli/commit/ba5d589df030d2e18424948f120ecfa4903de7af)) - baoyuexing
- add Phase 4 analysis reports for PR #272 (#270) - ([d95356a](https://github.com/byx-darwin/gitflow-cli/commit/d95356ada6e5afabd963f9b468715244bc759e45)) - baoyuexing
- correct PR #272 pipeline report to final all-green CI status - ([7c21f9d](https://github.com/byx-darwin/gitflow-cli/commit/7c21f9d4bc77cb042b11bd36d6c9c99e9cff272a)) - baoyuexing
- add design for Issue #271 resolve_body SafePath fix - ([eb50562](https://github.com/byx-darwin/gitflow-cli/commit/eb50562284b6c60ed71346760eda6a903a634bae)) - baoyuexing
- add implementation plan for Issue #271 resolve_body SafePath fix - ([0070964](https://github.com/byx-darwin/gitflow-cli/commit/00709646939c09795147f229f1e5165b1fe4d1ac)) - baoyuexing
- add Phase 4 analysis reports for PR #273 (#271) - ([645ac1e](https://github.com/byx-darwin/gitflow-cli/commit/645ac1e49a5ee6a5990f88d0d942bb55bdbccbed)) - baoyuexing
- add design and implementation plan for Issue #267 - ([da90a48](https://github.com/byx-darwin/gitflow-cli/commit/da90a48dea85b7a812061f83c19000c9c266e95c)) - baoyuexing
- add Phase 4 analysis reports for PR #274 (#267) - ([5dd6e9b](https://github.com/byx-darwin/gitflow-cli/commit/5dd6e9b40dab4d08c5022a60adda3796e6e0db6d)) - baoyuexing
- add design and implementation plan for Issue #275 - ([8f7607a](https://github.com/byx-darwin/gitflow-cli/commit/8f7607a56c5c55bd66608903e1c7947f7e121150)) - baoyuexing
- add Phase 4 analysis reports for PR #276 (#275) - ([b031d27](https://github.com/byx-darwin/gitflow-cli/commit/b031d27e2c0fdbaa802f8fa0d339b19ef8e57513)) - baoyuexing

### Features

- **(cli)** add gf issue edit subcommand - ([c292446](https://github.com/byx-darwin/gitflow-cli/commit/c292446bfd7972864057736b723ccc500277d0ee)) - baoyuexing
- **(gf-workflow)** add delivery_mode/merge_commit to Phase 3 evidence schema - ([0b7fc7d](https://github.com/byx-darwin/gitflow-cli/commit/0b7fc7d787c7f223c701875768bdb0b770be6a6f)) - baoyuexing
- **(gf-workflow)** Gate 3→4 accepts local_merge delivery evidence - ([b1d394d](https://github.com/byx-darwin/gitflow-cli/commit/b1d394dfb6c95927cc9df27a4ae0c61e5b40a448)) - baoyuexing
- **(gf-workflow)** Phase 3 delivery choice + Phase 4 Branch Finish local_merge path - ([0d2d504](https://github.com/byx-darwin/gitflow-cli/commit/0d2d504167654dd651a08e8943251cbb9ffee66c)) - baoyuexing
- **(gitlab)** add repo_target field and test-only constructor to GitLabIssueProvider - ([ae906a5](https://github.com/byx-darwin/gitflow-cli/commit/ae906a52422a230a0aa35cee04ebb0491691d31d)) - baoyuexing
- **(issue)** add IssueProvider::edit and implement for GitHub/GitLab/GitCode - ([3468569](https://github.com/byx-darwin/gitflow-cli/commit/3468569a10124d1afde35c2080faf7768b49300b)) - baoyuexing
- **(pr)** add queued merge via pr merge --auto - ([b0fb4b9](https://github.com/byx-darwin/gitflow-cli/commit/b0fb4b93f083d7314cb6d6026b0936980b59e9be)) - baoyuexing
- **(pr)** expose merged_at and wire queued merge through gf-workflow - ([e2f8a1e](https://github.com/byx-darwin/gitflow-cli/commit/e2f8a1e2e9a2d6fdfdfe5f40f1c3fbcd81ef7790)) - baoyuexing

### Miscellaneous Chores

- **(deps)** drop rand for rand_core, fix e2e-* edition/version drift - ([209978f](https://github.com/byx-darwin/gitflow-cli/commit/209978f7939f229f6c3e666686f43600bf23ce11)) - baoyuexing

### Other

- Merge pull request #256 from byx-darwin/dev

fix(ci): authenticate homebrew tap push so formula updates actually land - ([6643b55](https://github.com/byx-darwin/gitflow-cli/commit/6643b5505548df6051fc42cf0a3cc0da0a04a94d)) - mc-ai
- Merge remote-tracking branch 'origin/main' into dev - ([7c1948a](https://github.com/byx-darwin/gitflow-cli/commit/7c1948ac30f51cfdce8e3f31d468ed55eef93930)) - baoyuexing
- Merge pull request #257 from byx-darwin/dev

refactor(release): make CD the single owner of the Homebrew formula - ([712ba72](https://github.com/byx-darwin/gitflow-cli/commit/712ba72bcfc6ee34e01fa22728bb6596b3d980e2)) - mc-ai
- Merge pull request #258 from byx-darwin/dev

fix(workflow): add worktree preflight before git worktree add - ([5dd4ae4](https://github.com/byx-darwin/gitflow-cli/commit/5dd4ae408a75e0a06f83858d42d43de9270874a0)) - mc-ai
- Merge pull request #259 from byx-darwin/dev

feat(pr): queue merges with pr merge --auto so CI no longer blocks the human - ([ab0ea93](https://github.com/byx-darwin/gitflow-cli/commit/ab0ea93e51fa667df11a3e8a69fcea46c8418cf3)) - mc-ai
- Merge remote-tracking branch 'origin/main' into dev - ([bdbff96](https://github.com/byx-darwin/gitflow-cli/commit/bdbff9613e339390c7c35a148d8781d96e428c5b)) - baoyuexing
- stop re-running the full matrix on every push to dev - ([da1f2b9](https://github.com/byx-darwin/gitflow-cli/commit/da1f2b94819fe37a782d1aed8682d0c12c8ca0e8)) - baoyuexing
- Merge pull request #260 from byx-darwin/dev

fix(pr): name the scheduled merge instead of returning an empty message - ([8e8f615](https://github.com/byx-darwin/gitflow-cli/commit/8e8f615b06676cbd8fd491bb63794ec97b2fc8da)) - mc-ai
- Merge remote-tracking branch 'origin/main' into dev - ([cd9ccaa](https://github.com/byx-darwin/gitflow-cli/commit/cd9ccaaa077bb8dd4827c155dd02fe5bbf0b0e04)) - baoyuexing
- Merge pull request #261 from byx-darwin/dev

ci: stop re-running the full matrix on every push to dev - ([72bcac6](https://github.com/byx-darwin/gitflow-cli/commit/72bcac6c7affd5c299692f3ba1ca9351f2963442)) - mc-ai
- Merge remote-tracking branch 'origin/main' into dev - ([ded80c5](https://github.com/byx-darwin/gitflow-cli/commit/ded80c57e939fad19728c8bcbe27d1c4fde9f4a7)) - baoyuexing
- merge duplicate build.yml checks into ci.yml - ([2baf05c](https://github.com/byx-darwin/gitflow-cli/commit/2baf05c6aa58d3eff816ceb028d7bda4f43bde38)) - baoyuexing
- Merge pull request #268 from byx-darwin/feat/265-gf-workflow-local-merge-delivery

feat(gf-workflow): support local-merge as an alternative to PR delivery - ([c66fae0](https://github.com/byx-darwin/gitflow-cli/commit/c66fae061f8528ffc20bac3b0e6614a69490c390)) - mc-ai
- Merge pull request #269 from byx-darwin/feat/266-gf-issue-edit

feat(issue): add gf issue edit subcommand - ([209c15e](https://github.com/byx-darwin/gitflow-cli/commit/209c15e0937c725f522c0892be9674597cdd8d44)) - mc-ai
- Merge pull request #262 from byx-darwin/dev

feat(pr): expose merged_at so merged and closed-unmerged stop colliding - ([975c735](https://github.com/byx-darwin/gitflow-cli/commit/975c73538ae894af0bb8b3dfa3f80240d22c28f9)) - mc-ai
- Merge remote-tracking branch 'origin/dev' into feat/270-gitlab-label-fix

# Conflicts:
#	crates/gitlab/src/issue.rs - ([aabb851](https://github.com/byx-darwin/gitflow-cli/commit/aabb851b0ef2a149d7f271929eef7b97bc6e044b)) - baoyuexing
- Merge pull request #272 from byx-darwin/feat/270-gitlab-label-fix

fix(gitlab): use glab issue update --label/--unlabel instead of nonexistent issue edit - ([c7b7129](https://github.com/byx-darwin/gitflow-cli/commit/c7b71292bfd9ad8b38b7c6bf501ddf87829bb8c2)) - mc-ai
- Merge branch 'dev' of github.com:byx-darwin/gitflow-cli into dev - ([117ff8b](https://github.com/byx-darwin/gitflow-cli/commit/117ff8b72425a6d0bda67a37f16ae46c29a6e630)) - baoyuexing
- Merge pull request #273 from byx-darwin/feat/271-resolve-body-safepath

fix(cli): validate --body-file with SafePath in resolve_body - ([af78445](https://github.com/byx-darwin/gitflow-cli/commit/af7844520d073d9941af159f6bb8547a626a6b3f)) - mc-ai
- Merge pull request #274 from byx-darwin/feat/267-gitlab-issue-repo-target

fix(gitlab): use git remote URL as --repo target for issue commands - ([541cd25](https://github.com/byx-darwin/gitflow-cli/commit/541cd25999e938f66199063620086e6fb43102c4)) - mc-ai
- Merge pull request #276 from byx-darwin/feat/275-gitlab-non-issue-repo-target

fix(gitlab): route mr/release/pipeline/label/milestone --repo through repo_target - ([d4d69eb](https://github.com/byx-darwin/gitflow-cli/commit/d4d69eb3d5768fa6a151c8ae0a6b3beb5de64523)) - mc-ai

### Performance

- **(gf-workflow)** parallelize Phase 4 pipeline/triage/review dispatch - ([0978cc4](https://github.com/byx-darwin/gitflow-cli/commit/0978cc495bda2f783d3ed82059d6594bbd7f29e1)) - baoyuexing

### Tests

- **(autoreport)** record first end-to-end verification run - ([a1cb535](https://github.com/byx-darwin/gitflow-cli/commit/a1cb535e03f959c543311b70349761b5e5f7b8ca)) - baoyuexing

---
## [1.7.0](https://github.com/byx-darwin/gitflow-cli/compare/v1.6.0..v1.7.0) - 2026-08-30

### Bug Fixes

- **(ci)** grant attestations:write ceiling in CD caller workflow - ([1b14579](https://github.com/byx-darwin/gitflow-cli/commit/1b145796c2d0b91ee77a424bf4922cc6e700d8f5)) - baoyuexing
- **(ci)** authenticate homebrew tap push with host-scoped basic auth - ([401c280](https://github.com/byx-darwin/gitflow-cli/commit/401c280f6051f9a4a802bf9741ae362ed603a4b5)) - baoyuexing
- **(release)** verify CD outcome and release assets before declaring success - ([ea60501](https://github.com/byx-darwin/gitflow-cli/commit/ea60501ee0e952b2c9ad227e3e7346a3e3b76115)) - baoyuexing

### Documentation

- **(website)** trigger rebuild for v1.6.0 changelog - ([9e8066d](https://github.com/byx-darwin/gitflow-cli/commit/9e8066d616ae15a6cf6f6ad557453fdfbda82369)) - baoyuexing

### Miscellaneous Chores

- trigger E2E workflow - ([a8b5608](https://github.com/byx-darwin/gitflow-cli/commit/a8b560877a277acba0bf8d09ae406dc2f05f7470)) - baoyuexing

### Other

- Merge pull request #234 from byx-darwin/dev

docs(changelog): rewrite v1.6.0 entries in Chinese - ([9c2fada](https://github.com/byx-darwin/gitflow-cli/commit/9c2fadaf87f1c506fe4713a164c32f3b9b294092)) - mc-ai
- Merge pull request #235 from byx-darwin/chore/website-trigger-rebuild

docs(website): trigger rebuild for v1.6.0 changelog - ([b791cf2](https://github.com/byx-darwin/gitflow-cli/commit/b791cf25ad1c44c69dbead62c097f291d5e7f69d)) - mc-ai
- Merge branch 'main' into release/v1.7.0 - ([8b7c721](https://github.com/byx-darwin/gitflow-cli/commit/8b7c721918263a2b0111cb5658494d607147ae40)) - baoyuexing
- Merge pull request #254 from byx-darwin/release/v1.7.0

chore: release v1.7.0 - ([c039dc6](https://github.com/byx-darwin/gitflow-cli/commit/c039dc61b897e03c4bbc89fe8010ab2d4328720d)) - mc-ai
- Merge pull request #255 from byx-darwin/dev

fix(ci): restore CD by matching permission ceiling; gate release on assets - ([b1655d6](https://github.com/byx-darwin/gitflow-cli/commit/b1655d660a9f9a5bba4cf9992478ee3ea201212a)) - mc-ai

### Refactoring

- **(release)** make CD the single owner of the Homebrew formula - ([41a6fb1](https://github.com/byx-darwin/gitflow-cli/commit/41a6fb1e7940fac8946b9288fd231f60b4721f7e)) - baoyuexing

---
## [1.6.0](https://github.com/byx-darwin/gitflow-cli/compare/v1.5.0..v1.6.0) - 2026-08-26

### Bug Fixes

- **(cli)** document SIGPIPE investigation results — no safe reset needed - ([d23b015](https://github.com/byx-darwin/gitflow-cli/commit/d23b01557f9a0aadf668b8c37d9cfe9ecccb4d65)) - baoyuexing
- **(release)** update compatibility matrix after version bump - ([2b93ebf](https://github.com/byx-darwin/gitflow-cli/commit/2b93ebf5804840a2e02b1a9e154408c3d7a5349e)) - baoyuexing
- **(workflow)** add design_doc_path validation to Phase 2 gate (#238) - ([6777785](https://github.com/byx-darwin/gitflow-cli/commit/6777785ca33460b665bf6af928a334d33fc8eb64)) - mc-ai

### Documentation

- **(changelog)** rewrite v1.6.0 entries in Chinese - ([fc9953b](https://github.com/byx-darwin/gitflow-cli/commit/fc9953b53dde92d63d8bb6f447da4b58a6937c7a)) - baoyuexing
- **(core)** document sync git call rationale in resolve_platform - ([b23e2f8](https://github.com/byx-darwin/gitflow-cli/commit/b23e2f87caac50171453fadae5a3f7f74611743f)) - baoyuexing
- **(ops)** add CLI version compatibility matrix - ([fe8abbe](https://github.com/byx-darwin/gitflow-cli/commit/fe8abbe5362d7425e5aa1a233e4bb11bddca8409)) - baoyuexing
- add design spec for Pi Code Agent and OpenCode path fix - ([f6fa91e](https://github.com/byx-darwin/gitflow-cli/commit/f6fa91ee28e4e2ed8ba868f1b530becb69f5186f)) - baoyuexing
- add implementation plan for Pi Code Agent and OpenCode path fix - ([e17a275](https://github.com/byx-darwin/gitflow-cli/commit/e17a275237a169423d58c2149563d570b3533564)) - baoyuexing
- remove stale superpowers design docs - ([5885b74](https://github.com/byx-darwin/gitflow-cli/commit/5885b74aaba665d05b908fc8c580ad60c4aca716)) - baoyuexing
- update compatibility matrix - ([332e422](https://github.com/byx-darwin/gitflow-cli/commit/332e4225343c6e02699b81b189ff54029b62b869)) - baoyuexing

### Features

- **(cli)** add Standard variant to WorkflowMode - ([92cc0ce](https://github.com/byx-darwin/gitflow-cli/commit/92cc0ce11d0f9bb1db09e7b9bea94af1f24d3385)) - baoyuexing
- **(cli)** add --closes/--fixes arguments to pr create - ([9514e6c](https://github.com/byx-darwin/gitflow-cli/commit/9514e6c3e97b90c0cf984bd73903259d304bf700)) - baoyuexing
- **(core)** add format_closing_body helper for PR issue linking - ([178e2d2](https://github.com/byx-darwin/gitflow-cli/commit/178e2d258c39d56f1f8cf588c9f94782b071cf10)) - baoyuexing
- **(core)** add closes_issues field to CreatePrArgs - ([40e56d4](https://github.com/byx-darwin/gitflow-cli/commit/40e56d48729b8c74c462f13dff1e5fe9177f6a3c)) - baoyuexing
- **(core)** add shared adapter Session context for workflow chains - ([5219402](https://github.com/byx-darwin/gitflow-cli/commit/521940280f5af14231f7127bd8d953b27eb439ef)) - baoyuexing
- **(gitcode)** append closing keywords in PR create - ([aa66b5d](https://github.com/byx-darwin/gitflow-cli/commit/aa66b5d0401fd6d2709532283bb3131d3d247ab9)) - baoyuexing
- **(github)** append closing keywords in PR create - ([fbf2f70](https://github.com/byx-darwin/gitflow-cli/commit/fbf2f703ca1cba4ebcbf5858fed3554f25a12a68)) - baoyuexing
- **(gitlab)** append closing keywords in MR create - ([f0b7955](https://github.com/byx-darwin/gitflow-cli/commit/f0b7955d57c15947a76e22f65d969621ecc80619)) - baoyuexing
- **(skills)** add Qoder variant to AgentPlatform enum - ([332fbd2](https://github.com/byx-darwin/gitflow-cli/commit/332fbd2217fdb4dea6b47eb594c950be3d0f5356)) - baoyuexing
- **(skills)** add Pi Code Agent platform & fix OpenCode global path - ([9d883c7](https://github.com/byx-darwin/gitflow-cli/commit/9d883c7fed4818985761a705a05e8479190f3dd9)) - baoyuexing
- **(skills)** add --source parameter for external skill set installation - ([0f70954](https://github.com/byx-darwin/gitflow-cli/commit/0f709543b3bb1f802dc1a97d9494c42fdc55b8b1)) - baoyuexing

### Miscellaneous Chores

- sync main back to dev after release v1.5.0 - ([8513a85](https://github.com/byx-darwin/gitflow-cli/commit/8513a852f5c39dddc3a7f9185d4414144b05a61e)) - baoyuexing
- sync main back to dev after PR #225 - ([d6ae1b1](https://github.com/byx-darwin/gitflow-cli/commit/d6ae1b1c5340017fe46d50a6ba0b9f753e1297f6)) - baoyuexing
- sync main back to dev after PR #226 - ([a1a770f](https://github.com/byx-darwin/gitflow-cli/commit/a1a770fde258a556ebcbfcac18dcfc734073f0e6)) - baoyuexing
- update CHANGELOG.md for v1.6.0 release - ([2198b76](https://github.com/byx-darwin/gitflow-cli/commit/2198b7662b9d8a121fc0382c84fdd15ef79c44ed)) - baoyuexing
- release v1.6.0 - ([0f7cb08](https://github.com/byx-darwin/gitflow-cli/commit/0f7cb0868927241936253752f7a18dbc99260cb8)) - baoyuexing
- release v1.7.0 - ([a5ec22b](https://github.com/byx-darwin/gitflow-cli/commit/a5ec22b264cfd26bbb2b2f3aa0b24be331fb3e66)) - baoyuexing
- update CHANGELOG.md for v1.7.0 - ([a5f5e2b](https://github.com/byx-darwin/gitflow-cli/commit/a5f5e2b00fb2fb17e8ca74a1f389ba9a64c831c3)) - baoyuexing

### Other

- Merge branch 'main' into feat/229-workflow-standard-mode - ([ea5a8cc](https://github.com/byx-darwin/gitflow-cli/commit/ea5a8cc5b93d2b32a2c1caaa91a25dc5086bbb34)) - mc-ai
- Merge pull request #232 from byx-darwin/feat/229-workflow-standard-mode

feat(cli): add Standard variant to WorkflowMode - ([8eacc9a](https://github.com/byx-darwin/gitflow-cli/commit/8eacc9a95bcdd7bce604b58cb4611eca6e0ae600)) - mc-ai
- apply rustfmt formatting fixes - ([3ec5923](https://github.com/byx-darwin/gitflow-cli/commit/3ec59239705eb7139d4bc4fba6e5cc4b0612d6ef)) - baoyuexing
- Merge pull request #246 from byx-darwin/feat/245-pi-agent-opencode-fix

feat(skills): add Pi Code Agent platform & fix OpenCode global path - ([b69a598](https://github.com/byx-darwin/gitflow-cli/commit/b69a598bdf89688a757636b3af0dced3b8ecba77)) - mc-ai
- Merge pull request #233 from byx-darwin/dev

chore: sync dev to main for v1.6.0 release - ([79f14dc](https://github.com/byx-darwin/gitflow-cli/commit/79f14dc40da94d7d21afdbee08132873619ae787)) - mc-ai

### Refactoring

- **(arch)** P1-P3 architecture improvements — docs, platform detection, adapter dedup - ([1fa355c](https://github.com/byx-darwin/gitflow-cli/commit/1fa355c2da3fca10b4f1c8282492bbb2fef2edcc)) - baoyuexing

### Style

- **(skills)** fix nightly rustfmt formatting in tests - ([ba2bbcd](https://github.com/byx-darwin/gitflow-cli/commit/ba2bbcdd7ff6deb6871b78f0aa81ace1010e233a)) - baoyuexing

### Tests

- **(skills)** add tests for Qoder agent platform - ([2e0e752](https://github.com/byx-darwin/gitflow-cli/commit/2e0e7525367f03a7ed46a9fd4cf47f9fb0525f53)) - baoyuexing

---
## [1.5.0](https://github.com/byx-darwin/gitflow-cli/compare/v1.4.0..v1.5.0) - 2026-08-19

### Bug Fixes

- **(release)** tolerate gh pr checks pending exit in CI wait loop - ([2247155](https://github.com/byx-darwin/gitflow-cli/commit/2247155db2ca377e7f6f8889290ce25b018a0277)) - baoyx
- **(release-signer)** open temp output read+write for zip signing - ([554daed](https://github.com/byx-darwin/gitflow-cli/commit/554daed55b38c1da9f7049bb9f9a781d0b8ce3a9)) - baoyx

### Documentation

- **(workflow)** plan + design for autoreport-bug P0 fix (wf-2026-08-18-006) - ([0862eda](https://github.com/byx-darwin/gitflow-cli/commit/0862edaecc9d9bd0af96f800e4a9d7359e04c513)) - baoyx
- archive #198 glab 1.113 matrix workflow artifacts - ([5b120b2](https://github.com/byx-darwin/gitflow-cli/commit/5b120b271157dea9e306c64a9ba878185b125890)) - baoyx

### Miscellaneous Chores

- **(deps)** add glab 1.113.0 to compatibility matrix (#206) - ([d68c42b](https://github.com/byx-darwin/gitflow-cli/commit/d68c42b7abd04e4db47f9d6e7e0f06663af13fb3)) - mc-ai
- sync main back to dev after release v1.4.0 - ([900d76e](https://github.com/byx-darwin/gitflow-cli/commit/900d76ee6e3f00c6787e18403ee12ee07f0d13be)) - baoyx

### Refactoring

- **(core)** derive matrix gf version from CARGO_PKG_VERSION (#208) - ([d2ec7f9](https://github.com/byx-darwin/gitflow-cli/commit/d2ec7f9bfb6b48c26fad3d88cf3e9a818bab42b3)) - mc-ai

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
