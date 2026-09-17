# Walkthrough Report Template

Save the filled report at `docs/walkthrough-<slug>.md`.

## ① 开场叙事

3–8 句：谁受影响、发生了什么变化、为什么现在做。
**首句禁止以标识符、函数名、文件路径、命令名开头。** 术语随文解释。

## ② 变更摘要

`git diff --stat` 汇总行 + 按模块分组的「改了什么 · 为什么」。

## ③ 验证证据

- `[Measured]` <结论>
  ```
  $ <命令原文>
  <输出片段>
  ```
- `[Inferred]` <结论> —— 依据 `path:line`
- `[Unverified]` <结论> —— 未验证原因：<原因>

让命令跑起来的 workaround 须一并记录，否则读者无法复现。

存在失败测试时：

| 失败用例 | 最后修改 commit | 是否 base 祖先 |
|---|---|---|

Ancestry check:

```bash
H=$(git log -1 --format=%H -- "<test file>")
git merge-base --is-ancestor "$H" "$BASE" && echo "先于本次交付存在" || echo "本次引入"
```

## ④ 评审门禁

blast radius + 部署顺序 + merge checklist。
不涉及迁移或开关时，须显式写明「本次变更不涉及迁移与开关，blast radius 限于 <路径>」，
**不得省略该节**——无迁移/开关也要写明事实与 blast radius 范围。

## 自检清单（交付前逐项确认）

- [ ] 1. 每条验证结论都带 `Measured` / `Inferred` / `Unverified` 之一
- [ ] 2. 每条 `Measured` 紧随命令原文与输出块
- [ ] 3. 无「只有结论没有输出」却标 `Measured` 的条目
- [ ] 4. 每条 `Inferred` 写明依据的 `path:line`
- [ ] 5. 每条 `Unverified` 写明未验证原因
- [ ] 6. 补跑失败的条目标为 `Unverified`，未被改标为 `Inferred`
- [ ] 7. 失败测试三列齐全，无 `unrelated` 而缺 commit 的写法
- [ ] 8. 开场首句未以标识符、路径或命令名开头
- [ ] 9. ④ 节非空（不涉及迁移时亦显式写明）
- [ ] 10. 报告落盘于 `docs/walkthrough-*.md`
