# Hook Schema、运行时与事件输入

## 本页用途

- 这页不再承载全部细节，而改成 `01-tools-hooks-and-permissions` 下 hook 主题的总览与导航。
- 原先混在一页里的内容，已经拆成 schema/`InstructionsLoaded`/事件输入、特殊输出与消费语义、跨阶段时序图、非主线程覆盖面与 dispatcher 边界四个专题页。

## 相关文件

- [01-schema-instructionsloaded-and-event-inputs.md](./02-hook-system/01-schema-instructionsloaded-and-event-inputs.md)
- [02-special-output-and-event-consumer-semantics.md](./02-hook-system/02-special-output-and-event-consumer-semantics.md)
- [03-runtime-order-and-cross-stage-timing.md](./02-hook-system/03-runtime-order-and-cross-stage-timing.md)
- [04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md](./02-hook-system/04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md)
- [../01-tools-hooks-and-permissions.md](../01-tools-hooks-and-permissions.md)
- [../02-instruction-discovery-and-rules.md](../02-instruction-discovery-and-rules.md)
- [../04-non-main-thread-prompt-paths.md](../04-non-main-thread-prompt-paths.md)
- [../06-context-runtime-and-tool-use-context.md](../06-context-runtime-and-tool-use-context.md)

## 拆分后的主题边界

### Hook schema / `InstructionsLoaded` / HookEvent 输入

见：

- [01-schema-instructionsloaded-and-event-inputs.md](./02-hook-system/01-schema-instructionsloaded-and-event-inputs.md)

这一页集中放：

- hook event 与 hook def schema
- `PreToolUse` 返回结构
- hook 结果进入 transcript/UI 的 attachment 形态
- `InstructionsLoaded` 的 payload、入口、触发面
- 其他 HookEvent 输入字段

### `hookSpecificOutput` / `SubagentStart` / 特殊事件消费语义

见：

- [02-special-output-and-event-consumer-semantics.md](./02-hook-system/02-special-output-and-event-consumer-semantics.md)

这一页集中放：

- `hookSpecificOutput` 已确认分支
- `SubagentStart` 的输入、并发顺序与本地活消费面
- `Mi6("compact")` 的本地活语义
- `Stop / SessionEnd / SubagentStop / TaskCreated / TaskCompleted / Worktree*` 的 JSON output 消费边界

### Hook 时序 / 跨阶段顺序图

见：

- [03-runtime-order-and-cross-stage-timing.md](./02-hook-system/03-runtime-order-and-cross-stage-timing.md)

这一页集中放：

- `InstructionsLoaded -> UserPromptSubmit -> PreToolUse -> PermissionRequest` 的相邻时序
- `Setup / SessionStart / Stop / Compact` 的跨阶段顺序图
- 会话启动、fresh turn、tool round、subagent、compact 的总顺序图

### `InstructionsLoaded` 在非主线程的覆盖面 / producer 与 dispatcher 边界

见：

- [04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md](./02-hook-system/04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md)

这一页集中放：

- `InstructionsLoaded` 在 `BN(...)`、`lZ(...)`、`hook_agent`、compact summarize 中的覆盖面
- hook registry/source、dispatcher 与当前未决边界

## 建议阅读顺序

1. 先看 [01-schema-instructionsloaded-and-event-inputs.md](./02-hook-system/01-schema-instructionsloaded-and-event-inputs.md)，建立 hook 配置、事件和输入字段。
2. 再看 [02-special-output-and-event-consumer-semantics.md](./02-hook-system/02-special-output-and-event-consumer-semantics.md)，补齐 event-specific output 与调用方真正消费的语义。
3. 然后看 [03-runtime-order-and-cross-stage-timing.md](./02-hook-system/03-runtime-order-and-cross-stage-timing.md)，建立跨阶段时序图。
4. 最后看 [04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md](./02-hook-system/04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md)，把非主线程覆盖面与 dispatcher 边界闭环起来。

## 与其它专题的边界

- 工具执行主链、并发执行器与 `tool_result` 回写，见 [01-tool-execution-core.md](./01-tool-execution-core.md)。
- permission mode、classifier 与 ask 前后的审批主状态机，见 [03-permission-mode-and-classifier.md](./03-permission-mode-and-classifier.md)。
- managed policy、sandbox 与 ask backend transport，见 [04-policy-sandbox-and-approval-backends.md](./04-policy-sandbox-and-approval-backends.md)。
- 非主线程 prompt 路径里 `hook_prompt / hook_agent / compact summarize` 的旁路 prompt 结构，见 [../04-non-main-thread-prompt-paths.md](../04-non-main-thread-prompt-paths.md)。

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
