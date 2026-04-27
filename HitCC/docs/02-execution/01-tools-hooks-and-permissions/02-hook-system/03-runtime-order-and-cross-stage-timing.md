# Hook 时序与跨阶段顺序图

## 本页用途

- 单独承接 hook 与 instruction 装载、permission、compact、subagent 之间的相邻时序。
- 把单次 turn 和跨阶段总顺序图固定下来，不再和非主线程覆盖面、registry/dispatcher 混写。

## 相关文件

- [../02-hook-system.md](../02-hook-system.md)
- [04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md](./04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md)
- [../../03-prompt-assembly-and-context-layering.md](../../03-prompt-assembly-and-context-layering.md)
- [../03-permission-mode-and-classifier.md](../03-permission-mode-and-classifier.md)
- [../04-policy-sandbox-and-approval-backends.md](../04-policy-sandbox-and-approval-backends.md)

### Hook 与权限相邻的时序现在可以再收紧

之前“`InstructionsLoaded` hooks 与其他 hook 类型在同一 turn 内的先后关系不明”这个说法太宽了。  
就当前本地 bundle，可先把它拆成四个明确阶段：

#### 1. 指令装载期：`InstructionsLoaded`

- `sj()` 主扫描结束后
- 若 `WQ9()` 返回了当前 `load_reason`
- 且存在 `Xi6() === true`
- 就会对本轮装载到的 `User / Project / Local / Managed` memory 文件逐个调用 `Di6(...)`

如果把 compact 放进同一条时间线上，当前本地更准确的顺序应是：

```text
compact success
  -> Cn(querySource)
    -> Mi6("compact")
    -> clear sj/user-context related caches

next main request
  -> _$()
    -> sj()
      -> collect memory files
      -> WQ9() consume current reason
      -> Di6(...) per file
```

也就是说：

- `compact` 改的是“下一次扫描的默认 reason”
- 不是“立即执行一次 `InstructionsLoaded` hook”

这一步发生在 instruction / memory 装载链里，属于：

- `CLAUDE.md / .claude/rules / include / nested traversal` 进入 `userContext.ClaudeMd` 的同一阶段

因此它不是 tool loop 内的 hook，也不是 ask 之后的审批 hook。

#### 2. 输入提交期：`UserPromptSubmit`

- `AU8(...)`
  - 先调用 `ihz(...)` 做附件、slash command、多模态、命令预处理
  - 然后才执行 `qe1(...)`

因此 `UserPromptSubmit` 的位置已可写成：

```text
ihz(...)
  -> 生成普通当前轮输入 / 本地命令结果
  -> qe1(...) UserPromptSubmit
  -> 追加 hook attachments / additional context
  -> 进入 query
```

#### 3. 工具执行前：`PreToolUse`

- tool 真正进入执行器后
- `qs1(...)` 先跑 `PreToolUse`
- 其返回的 `permissionBehavior / updatedInput`
  会先被消费
- 然后才进入普通 permission merge / `YP(...)`

因此 `PreToolUse` 不是 ask backend 的一部分，而是：

- 先于本地 permission core
- 可以改输入
- 也可以直接产出 allow / ask / deny

#### 4. ask 之后：`PermissionRequest`

`PermissionRequest` 现在也可以拆成两类本地活路径：

- 交互式主线程
  - `YP(...) -> ask`
  - 若 `awaitAutomatedChecksBeforeDialog`
    - 先 `yU4(...)`
    - 其中活逻辑就是 `PermissionRequest` hooks
  - 然后才决定是否入 `toolUseConfirmQueue`
- SDK/headless/bridge
  - `StructuredIO.createCanUseTool(...)`
  - `LSz(...)` 与 `sendRequest(can_use_tool)` 并行 race

因此 `PermissionRequest` 不是“tool 前 hook”，而是：

- 已经过了本地 permission core
- 只在 ask 分支触发
- 在不同 transport 下，时机是“ask 后、人工审批前或与外部审批并行”

### 当前更稳的本地顺序图

```text
sj()
  -> Di6(...) InstructionsLoaded

AU8(...)
  -> ihz(...)
  -> qe1(...) UserPromptSubmit
  -> query / main loop

tool_use
  -> qs1(...) PreToolUse
  -> D0z / YP permission core
  -> if ask:
       yU4(...) / LSz(...) PermissionRequest
       -> queue / sdk can_use_tool / bridge / teammate approval
```

这意味着当前真正还不清楚的，已经不是“这些 hook 是否混在一起乱序执行”，而是更窄的边角：

- `InstructionsLoaded` 在不同非主线程分支里是否都一定会出现
- 多个 `InstructionsLoaded` 文件之间的绝对顺序是否还有额外特殊分支
- bundle 外是否还有额外 hook producer

### 跨阶段时序：现在可以写成单次会话 / 单次 turn 两张图

如果把 `Setup / SessionStart / InstructionsLoaded / UserPromptSubmit / PreToolUse / PostToolUse / Stop / PreCompact / PostCompact` 放到同一张 runtime 图里，当前本地实现已经可以拆成两层：

1. 会话启动期
2. 单次 turn 执行期

#### 1. 会话启动期：`Setup` 先于 `SessionStart`

当前本地启动链里，`Setup` 与 `SessionStart` 不是同一个阶段。

- `setup()` 结束后，会先看 `setupTrigger`
- 若存在 `setupTrigger`
  - 先执行 `dN8(setupTrigger)`，也就是 `Setup`
- 然后才进入 `ouz(...)`
  - fresh start 时由 `ouz(...)` 触发 `BD("startup")`
  - `init-only` 路径甚至是显式顺序：
    - `await dN8("init")`
    - `await BD("startup")`

因此启动期更稳的本地顺序应写成：

```text
setup()
  -> Setup
  -> load / resume initial messages
  -> SessionStart(source = "startup" | other bootstrap source)
```

这也意味着：

- `Setup` 是 session bootstrap hook
- `SessionStart` 是“生成开场 hook messages / additional context / initial user message”的后续阶段
- 当前本地证据里，`Setup` 先于 `SessionStart`

#### 2. fresh turn：`InstructionsLoaded` 先于 `UserPromptSubmit`

对于一次普通主线程 turn，当前本地顺序现在可以进一步钉死为：

```text
_$()
  -> sj()
    -> InstructionsLoaded

AU8(...)
  -> ihz(...)
  -> UserPromptSubmit
  -> query / CC(...)
```

这里要特别分清：

- `InstructionsLoaded`
  - 属于 `userContext.ClaudeMd` 的装载期
  - 发生在真正送模型前的上下文准备阶段
- `UserPromptSubmit`
  - 属于“本轮用户输入已经成形之后”的提交期
  - `ihz(...)` 先完成附件 / slash command / pasted content / multimodal 预处理
  - 然后才交给 `qe1(...)`

所以当前本地不应再把两者写成并列未知，而应写成：

- **`InstructionsLoaded` 在 fresh-run `sj()` 路径上更早**
- **`UserPromptSubmit` 在当前轮输入已经 materialize 之后才发生**

#### 3. subagent turn：`SubagentStart` 是进入 `CC(...)` 前的前置阶段

这一点现在也能放回总图里：

```text
BN(...)
  -> forkContextMessages / promptMessages
  -> SubagentStart
  -> CC(...)
  -> subagent loop
```

也就是说：

- `SubagentStart` 不在 subagent loop 结束时
- 它是 sidechain request 真正进入 `CC(...)` 前的前置注入阶段
- 当前 callsite 只消费 `additionalContext`

#### 4. tool round：`PreToolUse -> PermissionRequest? -> tool.call -> PostToolUse / PostToolUseFailure`

工具轮现在可以明确写成：

```text
tool_use
  -> PreToolUse
  -> D0z / YP permission core
  -> if ask:
       PermissionRequest
  -> tool.call(...)
  -> PostToolUse
     or PostToolUseFailure
```

更具体地说：

- `PreToolUse`
  - 先于本地 permission core
  - 可以改输入、给 permission 行为、直接 stop
- `PermissionRequest`
  - 只在 ask 分支出现
  - 不是所有 tool_use 都会跑
- `PostToolUse`
  - 在成功 `tool.call(...)` 之后
  - 在最终 `tool_result / attachment / contextModifier` 收尾前插入
- `PostToolUseFailure`
  - 只在失败分支出现
  - 与成功分支互斥

因此当前一轮工具执行里，不应再把 `PermissionRequest` 当成“所有 tool 前 hook”，更准确是：

- **先 `PreToolUse`**
- **ask 才有 `PermissionRequest`**
- **`tool.call(...)` 之后才进入 `PostToolUse` 或 `PostToolUseFailure`**

#### 5. no-tool completion：`Stop / SubagentStop` 晚于模型后处理

`Stop` 系列也不该只记成“最后执行一下”。  
从 `Rj4(...) -> Zs1(...)` 这条线看，当前本地顺序更接近：

```text
CC(...) no-tool completion
  -> prompt_suggestion / extract_memories / post-turn cleanup
  -> Stop or SubagentStop
  -> completed / stop_hook_blocking / stop_hook_prevented
```

也就是说：

- `Stop / SubagentStop` 不是 assistant 最后一段文本一出来就立刻触发
- 它发生在该轮模型输出结束后的收尾阶段
- 其结果才决定：
  - 直接完成
  - 追加 blocking feedback 再补一轮
  - 直接 `preventContinuation`

#### 6. compact：`PreCompact -> summarize -> SessionStart("compact") -> PostCompact`

compact 这一段之前最容易误写。  
现在本地代码已经能把它拆成两层顺序：

```text
compact request
  -> PreCompact
  -> summarize core
  -> restore attachments / plan / task status / invoked skills
  -> SessionStart(source = "compact")
  -> PostCompact
  -> caller Cn(querySource)
  -> next main request's InstructionsLoaded(load_reason = "compact")
```

这里有两个必须分开的点：

1. compact 过程中**确实会跑一次 `SessionStart("compact")`**
   - 调用点是 `jk6(...) / fVq(...)` 里的 `BD("compact")`
   - 它属于 compact result 自身的一部分
2. `InstructionsLoaded(load_reason="compact")` **不是同一时刻触发**
   - 它要等 `Cn(querySource) -> Mi6("compact")`
   - 再等下一次 fresh-run `sj()`
   - 才会在下一次主请求的装载期出现

因此 compact 相关事件的本地时序不应再写成“PreCompact / PostCompact 之间也许顺手跑了 InstructionsLoaded”，而应写成：

- **compact 内部：`PreCompact -> SessionStart("compact") -> PostCompact`**
- **下一次主请求装载期：`InstructionsLoaded(load_reason="compact")`**

### 当前可直接复用的总顺序图

把会话启动、普通 turn、subagent、tool round、compact 都放到一起，当前更稳的本地 runtime 图可以压成：

```text
session bootstrap
  -> Setup
  -> SessionStart(startup / clear / resume-miss / compact ...)

fresh main turn
  -> InstructionsLoaded
  -> UserPromptSubmit
  -> CC(...)
     -> if tool_use:
          PreToolUse
          -> permission core
          -> PermissionRequest (ask only)
          -> tool.call
          -> PostToolUse | PostToolUseFailure
          -> next turn
     -> else:
          Stop

subagent turn
  -> SubagentStart
  -> CC(...)
  -> SubagentStop

compact
  -> PreCompact
  -> summarize
  -> SessionStart("compact")
  -> PostCompact
  -> next fresh request
  -> InstructionsLoaded(load_reason="compact")
```

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
