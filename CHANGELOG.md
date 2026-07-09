# Changelog

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

---
## [unreleased]

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
