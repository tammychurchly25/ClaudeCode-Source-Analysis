# 无工具分支：recovery、stop hook 与 reactive compact

## 本页用途

- 单独承接 `z6 === false` 时的 turn 尾部分支，不再和工具轮、压缩链正文混写。
- 固定 reactive compact、`max_output_tokens` 还原和 stop hook 子状态机的真实边界。

## 相关文件

- [../04-agent-loop-and-compaction.md](../04-agent-loop-and-compaction.md)
- [02-compaction-pipeline-and-auto-compact-tracking.md](./02-compaction-pipeline-and-auto-compact-tracking.md)
- [../05-model-adapter-provider-and-auth.md](../05-model-adapter-provider-and-auth.md)
- [../../02-execution/01-tools-hooks-and-permissions/02-hook-system.md](../../02-execution/01-tools-hooks-and-permissions/02-hook-system.md)
- [../../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md](../../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md)

## 无工具分支：stop / recovery / compact

### 分支入口条件

只要 `z6 === false`，就进入“本轮没有 tool_use”分支。

### 1. reactive compact 分支

触发条件：

- 最后一条 assistant 是 prompt-too-long 类 API error
- 或 withheld media size error

执行顺序：

1. 调 `tryReactiveCompact(...)`。
2. 成功则：
   - 扣 `taskBudget`
   - `yield` compact 产物
   - 重写 `J.transition = { reason: "reactive_compact_retry" }`
   - `continue`
3. 失败则：
   - `yield` 原错误 assistant
   - 跑 `stop failure hook`
   - 返回：
     - `prompt_too_long`
     - 或 `image_error`

### reactive compact 当前能确认到的内部合同

这一块现在可以进一步从“内部 producer 未透视”收紧到“当前 bundle 内看起来根本没有接线上”。

已确认的事实：

- `po_` 在 reactive compact 成功后，直接执行 `let t6 = hn(o6)`。
- 手动 compact UI 的 reactive-only 路径 `V7z(...)` 也把 `$.result` 直接包装成普通 `compactionResult` 返回。
- 说明 reactive compact 的成功产物，至少在公共接口上必须满足：
  - **`hn(...)`-compatible compactionResult**

但继续向内追时，出现了一个更强的静态信号。

#### 主循环里的 `J26` 当前是未接线槽位

对 `cli.js` 做精确搜索后，`J26` 只出现 8 次：

- 1 次声明：`var J26 = null`
- 7 次调用/判定：
  - `isReactiveCompactEnabled`
  - `isWithheldPromptTooLong`
  - `isWithheldMediaSizeError`
  - `tryReactiveCompact`

没有看到任何赋值位点：

- 没有 `J26 = ...`
- 没有 destructuring 赋值
- 没有对象属性回填到这个局部变量

在这份 bundle 的静态语义下，这意味着：

- `X6 = J26?.isReactiveCompactEnabled() ?? false` 恒为 `false`
- `J26?.isWithheldPromptTooLong(...)` / `isWithheldMediaSizeError(...)` 都不会命中
- `if ((S6 || b6) && J26)` 也不会成立

所以主循环中的 reactive compact 分支当前更像：

- **预留但未接线的 dead slot**

#### 手动 `/compact` 的 reactive-only 后端 `DD4` 也是未接线槽位

手动 compact 侧同样存在一条 reactive-only 分支：

- `if (DD4?.isReactiveOnlyMode()) return await V7z(...)`

但 `DD4` 在当前 bundle 内也只看到：

- 1 次声明：`var DD4 = null`
- 1 次可选调用：`DD4?.isReactiveOnlyMode()`
- 1 次作为 `V7z(...)` 参数传入

同样没有任何赋值位点。

因此手动 `/compact` 当前真实活路径仍然是：

- 先试 `_V8(...)`
- 否则走 `jk6(...)`

而不是 reactive-only compact backend。

#### 当前更稳的最终结论

综合这些证据，本地 bundle 下对 reactive compact 最稳的结论应改写成：

- 从接口设计看，reactive compact 若被接上，成功产物必须满足 `hn(...)` 公共合同
- 但在**当前发行版 bundle** 里：
  - 主循环的 `J26` reactive compact backend 未见赋值
  - 手动 `/compact` 的 `DD4` reactive-only backend 也未见赋值
- 因此：
  - **reactive compact 更像保留接口/死槽位，而不是当前活着的 compact producer**
  - **不能再把它当作需要和 full / partial compact 做结构同构比较的活路径**

### 2. `max_output_tokens` 还原分支

触发条件：

- `gj4(J6)`，即最后 assistant 带 `apiError === "max_output_tokens"`。

行为：

- 若 `E < 3`
  - 补一条 meta user continuation：
    - “Resume directly”
    - “no apology, no recap”
    - “Pick up mid-thought”
  - 重写 `J.transition = { reason: "max_output_tokens_recovery", attempt: E + 1 }`
  - `continue`
- 若已经到上限
  - 直接 `yield` 该 assistant error
  - 不再续写

### 3. stop hook 分支

`Rj4(F, M6, ...)` 不是简单 bool 检查，而是一条会自己 `yield` 消息的子状态机。

它至少会：

- 运行 `Stop` hook
- 若当前是 teammate，再跑：
  - `TaskCompleted` hook
  - `TeammateIdle` hook
- `yield` hook progress / hook attachment / hook summary
- 在需要阻止继续时 `yield hook_stopped_continuation`

返回值只有两类核心位：

```ts
{
  blockingErrors: TranscriptLikeMessage[]
  preventContinuation: boolean
}
```

分支含义：

- `preventContinuation = true`
  - `po_` 直接返回 `{ reason: "stop_hook_prevented" }`
- `blockingErrors.length > 0`
  - 把这些 blockingErrors 追加进消息链
  - `stopHookActive = true`
  - `transition.reason = "stop_hook_blocking"`
  - 再补一轮
- 否则
  - 返回 `{ reason: "completed" }`

### `stopHookActive` 的真实作用

这一位不是装饰字段。

当前 bundle 里能确认的活消费面只有一处：

- 被带进 `executeStopHooks / Zs1(...)` 的 hook input

- main-thread：`hook_event_name = "Stop"`
- subagent：`hook_event_name = "SubagentStop"`
- 并且显式附带 `stop_hook_active: _`

因此它的语义更接近：

- “这不是第一次进入 stop hook 判定”
- “上一轮已经因为 stop-hook blocking message 被强制补过一次”

对 `cli.js` 做精确搜索后，没有看到除此之外的其他读点。  
所以当前可以把结论再收紧一层：

- **`stopHookActive` 是 stop hook input 的重入标记**
- **当前 bundle 内没有别的活消费方**

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
