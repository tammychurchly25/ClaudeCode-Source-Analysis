# 主循环状态、局部缓存与 `yield` 面

## 本页用途

- 单独承接 `CC / po_` 的主状态机骨架，不把压缩链、stop/recovery 分支和工具执行细节继续混在一起。
- 固定 `J`、turn 内局部缓存和 `yield` 面，方便后续从状态与事件两个视角复原主循环。

## 相关文件

- [../04-agent-loop-and-compaction.md](../04-agent-loop-and-compaction.md)
- [../03-input-compilation.md](../03-input-compilation.md)
- [../05-model-adapter-provider-and-auth.md](../05-model-adapter-provider-and-auth.md)
- [../06-stream-processing-and-remote-transport.md](../06-stream-processing-and-remote-transport.md)
- [../../03-ecosystem/07-tui-system/01-repl-root-and-render-pipeline.md](../../03-ecosystem/07-tui-system/01-repl-root-and-render-pipeline.md)
- [../../03-ecosystem/07-tui-system/02-transcript-and-rewind.md](../../03-ecosystem/07-tui-system/02-transcript-and-rewind.md)

## 主循环内核：`CC / po_`

### `CC(A)` 只是壳，不是状态机主体

`CC(A)` 只做三件事：

1. 建 `q = []`，用来暂存 queued command 的 UUID。
2. `yield* po_(A, q)`。
3. 结束后把 `q` 里记录过的 queued command 全部标成 `completed`。

真正的多轮状态机、分支切换、还原逻辑都在 `po_(A, q)`。

### `po_(...)` 的最小伪代码

```text
J = initialTurnState(input)
D = memdirPrefetchHandle(messages, toolUseContext)
P = undefined                     // taskBudget remaining shadow
X = gateSnapshot()

while (true):
  destructure J
  yield stream_request_start
  clone toolUseContext.queryTracking

  F = normalize(messages)
  F = applyContentReplacement(F)
  F = microcompact(F)

  { compactionResult, consecutiveFailures } = autocompact(F, cacheSafeSnapshot, tracking)
  if compacted:
    yield compact boundary / summary / attachments
    F = compacted transcript
    Q = reset autoCompactTracking
  else:
    Q = update consecutiveFailures

  setup streaming tool runner / permission mode / model selection
  for await event/message from callModel(...):
    yield raw stream_event + assistant fragments + partial tool results
    accumulate M6 / $6 / T6 / z6
    if streaming fallback happened:
      yield tombstone for orphaned M6
      reset M6 / $6 / T6 / z6 / tool runner

  if aborted during streaming:
    flush remaining streaming tool results
    return aborted_streaming

  if pendingToolUseSummary from previous turn exists:
    await and yield it

  if no tool_use in this turn:
    handle reactive compact / max_output_tokens recovery / stop hooks / completed
  else:
    execute tools
    emit attachments / queued commands / skill artifacts
    maybe build pendingToolUseSummary promise
    rewrite J for next turn and continue
```

## 长生命周期状态：`J`

`J` 不是“本轮上下文”，而是 `po_` 在 turn 之间搬运的唯一主状态。

```ts
interface TurnState {
  messages: TranscriptLikeMessage[]
  toolUseContext: ToolUseContext
  maxOutputTokensOverride?: number
  autoCompactTracking?: {
    compacted: boolean
    turnId: string
    turnCounter: number
    consecutiveFailures?: number
  }
  stopHookActive?: boolean
  maxOutputTokensRecoveryCount: number
  hasAttemptedReactiveCompact: boolean
  turnCount: number
  pendingToolUseSummary?: Promise<TranscriptLikeMessage | null> | null
  transition?: { reason: string; [k: string]: unknown }
}
```

### 初始化值

`po_` 入口直接把 `J` 初始化成：

- `messages = A.messages`
- `toolUseContext = A.toolUseContext`
- `maxOutputTokensOverride = A.maxOutputTokensOverride`
- `autoCompactTracking = undefined`
- `stopHookActive = undefined`
- `maxOutputTokensRecoveryCount = 0`
- `hasAttemptedReactiveCompact = false`
- `turnCount = 1`
- `pendingToolUseSummary = undefined`
- `transition = undefined`

### `J` 只在 4 类分支里被整包重写

`po_` 并不是对 `J` 做零散 mutation，而是在关键分支里直接重建整个对象。

#### 1. reactive compact 成功后重试

```ts
J = {
  messages: compactedMessages,
  toolUseContext: v,
  autoCompactTracking: undefined,
  maxOutputTokensRecoveryCount: E,
  hasAttemptedReactiveCompact: true,
  maxOutputTokensOverride: undefined,
  pendingToolUseSummary: undefined,
  stopHookActive: undefined,
  turnCount: p,
  transition: { reason: "reactive_compact_retry" }
}
```

#### 2. `max_output_tokens` 直接续写还原

```ts
J = {
  messages: [...F, ...M6, recoveryMetaUserMessage],
  toolUseContext: v,
  autoCompactTracking: Q,
  maxOutputTokensRecoveryCount: E + 1,
  hasAttemptedReactiveCompact: h,
  maxOutputTokensOverride: undefined,
  pendingToolUseSummary: undefined,
  stopHookActive: undefined,
  turnCount: p,
  transition: { reason: "max_output_tokens_recovery", attempt: E + 1 }
}
```

#### 3. stop hook 产生 blocking error，强制补一轮

```ts
J = {
  messages: [...F, ...M6, ...blockingErrors],
  toolUseContext: v,
  autoCompactTracking: Q,
  maxOutputTokensRecoveryCount: 0,
  hasAttemptedReactiveCompact: h,
  maxOutputTokensOverride: undefined,
  pendingToolUseSummary: undefined,
  stopHookActive: true,
  turnCount: p,
  transition: { reason: "stop_hook_blocking" }
}
```

#### 4. 正常工具轮结束，进入下一 turn

```ts
J = {
  messages: [...F, ...M6, ...$6],
  toolUseContext: G6,
  autoCompactTracking: Q,
  turnCount: p + 1,
  maxOutputTokensRecoveryCount: 0,
  hasAttemptedReactiveCompact: false,
  pendingToolUseSummary: R6,
  maxOutputTokensOverride: undefined,
  stopHookActive: I,
  transition: { reason: "next_turn" }
}
```

### `transition` 在当前 bundle 里是 write-only branch marker

这一点现在可以比之前写得更实。

- `TurnState.transition` 在本地 bundle 里只出现在：
  - `po_` 初始化：`transition = undefined`
  - 4 个整包重写分支：
    - `reactive_compact_retry`
    - `max_output_tokens_recovery`
    - `stop_hook_blocking`
    - `next_turn`
- 对 `cli.js` 做精确搜索后，没有看到：
  - `J.transition` / `.transition`
  - `transition.reason`
  - 基于这个字段的后续分支判断
- 当前能直接看到的真正控制流出口，仍然是：
  - `continue`
  - `return { reason: ... }`
  - `yield` 出去的 runtime event / attachment / system message

所以就这份发行版而言，`transition` 不是还原、UI、compact 决策的活输入，更像：

- 本地调试/状态快照用 branch marker
- 给未来还原逻辑或外层观测预留的状态槽位

更保守但更准确的说法应是：

- **当前 bundle 内没有 `transition` 的活消费方**
- **它记录了“本轮为何重建 `J`”，但不驱动后续分支**

## turn 内局部缓存

### 跨 turn 外但在一次 `po_` 调用内长期存在的局部变量

- `f`
  - disposable stack，`finally` 里统一清理。
- `P`
  - `taskBudget.remaining` 的本地 shadow。
  - compact / reactive compact 成功时会扣减。
- `X`
  - gate snapshot。
  - 当前直接影响：
    - `streamingToolExecution`
    - `emitToolUseSummaries`
    - `fastModeEnabled`
- `D`
  - `Fj4(...)` 返回的 memdir prefetch handle。
  - 不是消息本体的一部分，但会在后面 materialize 成 attachment。

### 每 turn 重建的一组缓存

- `F`
  - 当前 turn 真正送去 `callModel` 的 transcript-like messages。
  - 来源顺序是：`LN(k)` -> `f2q(...)` -> `microcompact(...)` -> `autocompact(...)`。
- `Q`
  - 当前轮后的 `autoCompactTracking` 快照。
- `u`
  - skill discovery prefetch handle。
- `i`
  - streaming tool runner `Re6`，只有 gate 开启时存在。

### callModel 期间的四个核心缓存

- `M6`
  - 当前 turn 产生的 assistant 片段列表。
  - 注意不是“最后一条 assistant”，而是整轮里所有 assistant 增量片段。
- `$6`
  - 当前 turn 产生的下游 user-side message。
  - 里面既有 `tool_result`，也会混入 attachment、queued command、skill 等后处理产物。
- `T6`
  - 当前 turn 所有 `tool_use` block。
- `z6`
  - 当前 turn 是否出现过任何 `tool_use`。

### 其他 turn 级临时槽位

- `R6`
  - tool-use summary promise。
  - 本轮只创建，不立即 `yield`；下一轮开头才消费 `pendingToolUseSummary`。
- `W6`
  - 工具执行后可能被 `contextModifier` 改写过的新 `toolUseContext`。

## `yield` 面：`po_` 对外会发什么

`po_` 不是只吐 assistant 文本。它对外暴露的是一条混合事件流。

### 直接 `yield` 的类别

- `stream_request_start`
  - 每个 turn 一开始先发。
- `stream_event`
  - 原始模型流事件，来自 `Jk6/_I4`。
- `assistant`
  - 按 content block 收口后的 assistant 片段。
- `attachment`
  - compact、hook、attachment producer、skill discovery、file restore、task status 等都走这里。
- `progress`
  - hook/tool/MCP 进度。
- `system`
  - warning / notification / API error message 等。
- `tombstone`
  - streaming 失败后 fallback 到 non-streaming 时，用于作废已经发出去的 orphaned assistant 片段。

### 在本页最重要的几个特殊 `yield`

- `tombstone`
  - 仅在 `callModel` 已经流出 assistant 片段、随后触发 non-streaming fallback 时出现。
  - 会对 `M6` 里的每条 assistant 逐条发 tombstone，然后清空 `M6 / $6 / T6 / z6`。
- `hook_stopped_continuation`
  - stop hook / task-completed hook / teammate-idle hook 要求终止继续时发。
- `max_turns_reached`
  - 两个位置会发：
    - tools 中断后，但下一个 turn 编号已经超限
    - 正常 tool round 结束后，准备进下一轮时超限

### `tombstone` 的下游消费面

这一块现在也能从“会发 tombstone”压到“谁真正处理它”。

#### 1. 主 TUI / direct-connect UI：把已显示的 orphaned assistant 删掉

流式事件适配器 `cy6(...)` 对非 stream event 有一个专门分支：

```ts
if (A.type === "tombstone") {
  removeMessage?.(A.message)
  return
}
```

这说明 tombstone 不是普通 transcript message，而是：

- **对上层 UI 发出的删除信号**

当前已能确认至少两条活调用链会真的传入 `removeMessage`：

- 主 REPL 本地流式适配
- `useRemoteSession.onMessage(...)` 的 direct-connect / remote UI 适配

这两条链里，`removeMessage` 的实质都是：

- 从本地 `messages` 数组里把那条先前插入的 assistant 对象删掉
- 同时清掉与该 UUID 相关的局部流式状态

所以 tombstone 对 TUI/remote UI 来说不是“提示文案”，而是：

- **撤销此前已经显示出来、但随后被 non-streaming fallback 判定为 orphaned 的 assistant 片段**

#### 2. SDK / `--print --output-format=stream-json`：显式吞掉

SDK/headless 查询器 `xo4.submitMessage()` 在主 switch 里有明确分支：

```ts
case "tombstone":
  break
```

效果是：

- 不写入 `this.mutableMessages`
- 不经 `xx8(...)` 对外产出
- 不转成任何 `stream-json` 协议消息

所以对 SDK 消费者来说：

- **不会收到 tombstone 事件本身**
- 他们只能看到最终保留下来的 assistant / user / system / result 流

#### 3. 内部 hook-agent 流式适配：存在分支，但当前通常没有 remover

内部 hook-agent 也复用 `cy6(...)`，但该调用点没有传 `removeMessage` 回调。  
这意味着在那条内部 stop-hook agent 路径里：

- tombstone 分支会命中
- 但因为 `removeMessage` 缺席，实际效果接近 no-op

这条链本身不是用户主界面的显示面，所以它更像：

- 复用通用 adapter 时留下的兼容分支
- 不是 tombstone 的主要业务消费者

#### 4. 更稳的定位

综合这些读点，tombstone 最准确的定位应是：

- **live-output consistency signal**
- 不是长期 transcript 语义对象
- 也不是还原状态机的控制字段

它主要解决的是：

- streaming 已经吐出一段 assistant
- 但底层随后回退到 non-streaming
- 上层需要把这段“已经展示过、但不该保留”的 orphaned assistant 撤销掉

## 当前仍需保守表述的点

- `tombstone` 在主 TUI、remote UI、SDK/headless 的处理已经钉住；但更外围的桥接端、Web/App 前端是否还会有协议层再解释，本地 bundle 里不再强推。

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
