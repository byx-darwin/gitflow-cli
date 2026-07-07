### Task 1: 添加 --repo 参数支持（Issue #47）
- [ ] 修改 `apps/cli/src/commands/issue.rs`
  - 在 `Create` 变体中添加 `repo: Option<String>` 字段
  - 修改 `handle` 函数，如果提供了 `--repo` 参数，使用它而不是从 remote 提取
- [ ] 修改 `apps/cli/src/main.rs`
  - 在 CLI 参数定义中添加 `--repo` 选项
  - 传递 `repo` 参数到 `issue::handle`
- [ ] 添加测试验证 `--repo` 参数功能
- [ ] 运行 `make test` 确认测试通过
- [ ] 运行 `make clippy` 确认无警告
