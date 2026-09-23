## PR Review — #212 (wf-2026-08-18-006)

> ⚠️ 自审限制：PR 作者与评审账号均为 `byx-darwin`，GitHub 禁止批准自己的 PR，故以 comment 记录，不构成正式 approve。

**变更面：** Rust（T4 错误分类）+ hook（T3）+ 依赖（T1 h2）+ 配置（T2 label）+ 文档，共 9 文件 +765 增删。

6 维度评估：

| 维度 | 判定 | 说明 |
|---|---|---|
| Correctness | ✅ | T4 过滤逻辑正确：`UserInputError`（miette code `gf::user_input`）→ main.rs `e.code()` 匹配 → `USER_INPUT_ERROR` → reporter 跳过。过滤位置在 `is_co_contribution_enabled` 之后、构造 report 之前，顺序合理 |
| Security | ✅ | 用户输入错误不再落盘 → 减少误报暴露；`0o600`/脱敏逻辑未受影响；h2 升级修复 DoS advisory |
| Reliability | ✅ | h2 0.4.16 消除 RUSTSEC-2026-0258，CI 依赖审查 job 从红转绿（因果闭环验证）；hook banner 引用正确技能名 |
| Maintainability | ✅ | `errors.rs` 单一职责；`is_user_input_error` 提取为可测谓词；测试 4 种 case 覆盖边界 |
| Test coverage | ✅ | 新增单元测试（分类谓词）+ bats（banner 技能名 + MUST 指令）；`make test` 1314 全过 |
| Documentation | ✅ | PR body 含 `Closes #211`；integration-guide matcher 同步 |

**非阻塞观察：**
1. `errors.rs` 的 `UserInputError` 目前仅覆盖 issue/pr 的 state 校验；其他参数校验错误（若存在）可后续同类化——本 PR 已解决评估报告 E37 指出的核心误报实例。
2. hook bats 测试依赖 `bats` 二进制（本地未安装），已在 CI 运行；手动模拟验证 GREEN。
3. dev 分支成功率 7 天 80.6%，h2 修复后应回升，建议合并后观察。

**Verification：** `make test` 1314 通过 · clippy 干净 · `cargo deny` 无 h2 · CI 4/4 success · 端到端确认用户输入错误不写 pending.json

Closes #211
