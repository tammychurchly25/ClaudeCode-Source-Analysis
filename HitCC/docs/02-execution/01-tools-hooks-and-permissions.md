# 工具执行、Hook 与权限系统

## 本页用途

- 这页不再承载全部细节，而改成 `02-execution` 下这一组主题的总览与导航。
- 原先混写在一页里的内容，已经拆成工具执行内核、Hook 系统、permission 状态机/auto classifier、managed policy/sandbox/审批 backend 四个专题页。

## 相关文件

- [01-tools-hooks-and-permissions/01-tool-execution-core.md](./01-tools-hooks-and-permissions/01-tool-execution-core.md)
- [01-tools-hooks-and-permissions/02-hook-system.md](./01-tools-hooks-and-permissions/02-hook-system.md)
- [01-tools-hooks-and-permissions/02-hook-system/01-schema-instructionsloaded-and-event-inputs.md](./01-tools-hooks-and-permissions/02-hook-system/01-schema-instructionsloaded-and-event-inputs.md)
- [01-tools-hooks-and-permissions/02-hook-system/02-special-output-and-event-consumer-semantics.md](./01-tools-hooks-and-permissions/02-hook-system/02-special-output-and-event-consumer-semantics.md)
- [01-tools-hooks-and-permissions/02-hook-system/03-runtime-order-and-cross-stage-timing.md](./01-tools-hooks-and-permissions/02-hook-system/03-runtime-order-and-cross-stage-timing.md)
- [01-tools-hooks-and-permissions/02-hook-system/04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md](./01-tools-hooks-and-permissions/02-hook-system/04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md)
- [01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md](./01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md)
- [01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md](./01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md)
- [03-prompt-assembly-and-context-layering.md](./03-prompt-assembly-and-context-layering.md)
- [05-attachments-and-context-modifiers.md](./05-attachments-and-context-modifiers.md)
- [06-context-runtime-and-tool-use-context.md](./06-context-runtime-and-tool-use-context.md)
- [../03-ecosystem/03-plan-system.md](../03-ecosystem/03-plan-system.md)
- [../05-appendix/02-evidence-map.md](../05-appendix/02-evidence-map.md)

## 拆分后的主题边界

### 工具执行内核 / `tool_result` / 并发执行器

见：

- [01-tools-hooks-and-permissions/01-tool-execution-core.md](./01-tools-hooks-and-permissions/01-tool-execution-core.md)

这一页集中放：

- `he6 / Mo_ / Re6 / Zx8` 主执行链
- deferred tools / `ToolSearch`
- `tool_result` 两层形态、配对修复与落盘
- `AskUserQuestion` 的特例

### Hook Schema / Runtime / Event Payload

见：

- [01-tools-hooks-and-permissions/02-hook-system.md](./01-tools-hooks-and-permissions/02-hook-system.md)

这一组子页集中放：

- hook 配置层 schema
- `InstructionsLoaded` 触发链与 payload
- `hookSpecificOutput` 分支
- `Stop / SessionEnd / Worktree*` 等特殊事件的消费语义

### Permission Mode / 状态机 / Auto Classifier

见：

- [01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md](./01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md)

这一页集中放：

- `D0z / YP` 两层 permission core
- `za / Hy6 / oy6 / LqA / Qs6 / SV / IqA`
- dangerous allow rules 剥离与还原
- auto classifier 的输入、默认规则、fast-path 与失败语义

### Managed Policy / Sandbox / Ask Backend

见：

- [01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md](./01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md)

这一页集中放：

- `allowManagedPermissionRulesOnly`
- `RG8(...)` 的 sandbox 合流
- `--permission-prompt-tool`
- `toolUseConfirmQueue`、remote/direct/ssh、headless/SDK/bridge 的 ask backend
- `orphaned-permission` 与当前未决边界

## 建议阅读顺序

1. 先看 [01-tool-execution-core.md](./01-tools-hooks-and-permissions/01-tool-execution-core.md)，建立工具执行主链。
2. 再看 [02-hook-system.md](./01-tools-hooks-and-permissions/02-hook-system.md) 及其子页，补齐 Hook event、payload 和运行时时序。
3. 然后看 [03-permission-mode-and-classifier.md](./01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md)，理解 permission 状态机与 auto classifier。
4. 最后看 [04-policy-sandbox-and-approval-backends.md](./01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md)，把组织策略、sandbox、审批 transport 合回同一条执行链。

Creative Commons Attribution 4.0 International

Copyright (c) 2026 Hitmux contributors

This work is licensed under the Creative Commons Attribution 4.0 International License.

You are free to:
- Share — copy and redistribute the material in any medium or format
- Adapt — remix, transform, and build upon the material for any purpose, even commercially

Under the following terms:
- Attribution — You must give appropriate credit, provide a link to the license, and indicate if changes were made.

License details:
- Human-readable summary: https://creativecommons.org/licenses/by/4.0/
- Full legal code: https://creativecommons.org/licenses/by/4.0/legalcode
