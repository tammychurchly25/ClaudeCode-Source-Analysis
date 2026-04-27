# 工具轮、下一 turn 与终止返回

## 本页用途

- 单独承接有工具分支、下一轮续转和主循环的各种终止 reason，不再和无工具分支混写。
- 固定工具执行产物、延后一轮消费的 summary 和中断错误收尾语义。

## 相关文件

- [../04-agent-loop-and-compaction.md](../04-agent-loop-and-compaction.md)
- [01-main-loop-state-caches-and-yield-surface.md](./01-main-loop-state-caches-and-yield-surface.md)
- [03-no-tool-branch-recovery-stop-and-reactive-compact.md](./03-no-tool-branch-recovery-stop-and-reactive-compact.md)
- [../06-stream-processing-and-remote-transport.md](../06-stream-processing-and-remote-transport.md)
- [../../02-execution/01-tools-hooks-and-permissions/01-tool-execution-core.md](../../02-execution/01-tools-hooks-and-permissions/01-tool-execution-core.md)
- [../../02-execution/05-attachments-and-context-modifiers.md](../../02-execution/05-attachments-and-context-modifiers.md)
- [../../03-ecosystem/01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md](../../03-ecosystem/01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md)

## 有工具分支：tool round -> 下一 turn

### 1. 工具执行

有两条路径：

- `Re6`
  - gate 开启时的 streaming tool execution
- `Zx8`
  - 传统批次工具执行

二者统一产出：

- `message`
- `newContext`

其中：

- `message` 会被 `yield`，并推入 `$6`
- `newContext` 会把 `W6` 改成工具修改后的 `toolUseContext`

### 2. hook 停止工具后续

若工具阶段产出的 attachment 里出现：

- `hook_stopped_continuation`

则 `E6 = true`，本轮直接返回：

- `{ reason: "hook_stopped" }`

这和 stop hook 分支不同：

- stop hook 发生在“无 tool_use 的 turn 尾部”
- 这里发生在“工具执行阶段内部”

### 3. 附件与后处理

工具阶段结束后，`po_` 还会顺次补这些东西进 `$6`：

- `KE6(...)` 产出的 attachment / queued command / context modifier 物化消息
- `D` 对应的 memdir prefetch 结果
- skill discovery prefetch 结果

### 4. tool summary 是延后一轮消费的

若 gate 打开且本轮存在 `tool_use`：

- 当前 turn 尾部只创建 `R6 = Promise<attachment|null>`
- 不立即 `yield`
- 下一 turn 一开始，先取上轮 `pendingToolUseSummary` 并 `yield`

所以 `pendingToolUseSummary` 本质上是：

- 一个跨 turn 的 deferred attachment 槽位
- 不是本轮即时缓存

## 中断与错误返回

### `aborted_streaming`

触发条件：

- model streaming 过程中 `abortController.signal.aborted`

收尾动作：

- 若有 streaming tool runner，尽量取完 `getRemainingResults()`
- 否则把当前 `M6` 中的 `tool_use` 转成 error `tool_result`
- 清理 `computer-use`
- 非 `interrupt` 原因时补 `Ur({ toolUse: false })`

### `aborted_tools`

触发条件：

- tool execution 过程中 `abortController.signal.aborted`

收尾动作：

- 清理 `computer-use`
- 非 `interrupt` 原因时补 `Ur({ toolUse: true })`
- 若下一个 turn 已超 `maxTurns`，先 `yield max_turns_reached`

### `model_error / image_error`

`callModel` 异常时：

- 图片类错误 -> `image_error`
- 其他错误
  - 把 `M6` 中未闭合的 `tool_use` 统一转成 error `tool_result`
  - `yield` system error
  - 返回 `model_error`

### 其他返回 reason

当前 `po_` 主干可直接看到的返回 reason 至少包括：

- `blocking_limit`
- `completed`
- `stop_hook_prevented`
- `hook_stopped`
- `prompt_too_long`
- `image_error`
- `model_error`
- `aborted_streaming`
- `aborted_tools`
- `max_turns`

## 当前仍需保守表述的点

- `team_permission_update` 这类 team 协议不会直接改本页状态机，但会通过 `toolUseContext.getAppState()` 间接影响下一轮 permission mode。

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
