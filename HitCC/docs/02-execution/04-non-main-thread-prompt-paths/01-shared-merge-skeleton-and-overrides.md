# 共享 Merge 骨架与前置裁剪

## 本页用途

- 用来固定非主线程路径里“仍沿用主线程 request-level merge 骨架”的那一组分支。
- 用来单独记录 `BN(...)` 在进入 `CC(...)` 前做的裁剪、override 与 `SubagentStart` hook 注入顺序。

## 相关文件

- [../04-non-main-thread-prompt-paths.md](../04-non-main-thread-prompt-paths.md)
- [../03-prompt-assembly-and-context-layering.md](../03-prompt-assembly-and-context-layering.md)
- [../02-instruction-discovery-and-rules.md](../02-instruction-discovery-and-rules.md)
- [../../03-ecosystem/01-resume-fork-sidechain-and-subagents.md](../../03-ecosystem/01-resume-fork-sidechain-and-subagents.md)

## 一句话结论

- 主线程普通 query、`BN(...)` 驱动的普通 subagent/sidechain、以及 `lZ(...)` 驱动的 fork-family，请求级别最终仍复用同一套 `CC -> dj4/Lx8 -> callModel` 骨架。
- 真正的差异主要出现在进入 `CC(...)` 前的输入裁剪与 override，而不是另起一套 merge 协议。

## 非主线程路径：前两类

### 非主线程路径：现在可以拆成三类

把 `BN(...)`、`lZ(...)`、`CC(...)`、`hook_agent`、`compact` 几条线对完后，当前更稳的结论不应再写成“非主线程大体复用主线程 merge”。

更准确的写法是：

1. **共享同一套 merge 骨架**
2. **共享 merge 骨架，但会先裁剪/override 输入**
3. **完全旁路主线程 merge，改走专用 summarize / verification prompt**

### 第一类：共享同一套 merge 骨架的路径

当前已可直接确认，下面这些路径最终都会落到同一个请求装配骨架：

```text
CC(...)
  -> a = WK(dj4(systemPromptSections, systemContext))
  -> messages = Lx8(normalizedMessages, userContext)
  -> callModel({ messages, systemPrompt: a, ... })
```

这意味着它们**最终 request 级别仍是同一套 `system/messages` 分层**，不是另一套协议：

- 主线程普通 query
- `BN(...)` 驱动的普通 subagent / sidechain
- `lZ(...)` 驱动的 fork-family 请求

其中 `lZ(...)` 已可视为一个“fork runner”：

- 输入直接吃 `cacheSafeParams`
  - `systemPrompt`
  - `userContext`
  - `systemContext`
  - `forkContextMessages`
- 然后直接调用 `CC(...)`

因此下面这些路径现在也应归入 **共享主骨架的 fork-family**：

- `session_memory`
- `prompt_suggestion`
- `extract_memories`
- `agent_summary`
- `side_question`
- `speculation`
- 以及其他通过 `lZ(...)` 发起的 side query

### 第二类：共享 merge 骨架，但会裁剪或 override 输入

这一类**不是换了 merge 算法**，而是在进入 `CC(...)` 前先改 `systemPrompt / userContext / systemContext / messages`。

#### `BN(...)` 的统一前置规则

`BN(...)` 的关键形状现在可以写成：

```text
base:
  userContext = override.userContext ?? _$()
  systemContext = override.systemContext ?? vO()
  systemPrompt = override.systemPrompt ?? WK(await ex_(...))

then trim:
  if agentDefinition.omitClaudeMd and no override.userContext:
    remove userContext.ClaudeMd

  if agentType === "Explore" || agentType === "Plan":
    remove systemContext.gitStatus

then:
  CC({ messages, systemPrompt, userContext, systemContext, ... })
```

所以现在已经能钉死：

- `omitClaudeMd` 不是“换成另一套 merge”
- 它只是**在 `userContext` 层删掉 `ClaudeMd` 后，仍走同一个 `Lx8(...)`**
- `Explore / Plan` 不是“没有 systemContext”
- 而是**裁掉 `gitStatus` 后，仍走同一个 `dj4(...)`**

#### 目前已直接看到的裁剪/override 分支

- `Explore` built-in agent：`omitClaudeMd: true`
- Claude guide agent：`omitClaudeMd: true`
- `fork`
  - 优先复用父线程已渲染好的 `systemPrompt`
  - 但 `userContext/systemContext` 仍回到 `BN(...)` 这套装配/裁剪逻辑
- `magic_docs`
  - 直接把父 query 的 `systemPrompt / userContext / systemContext` 整包 override 给 `BN(...)`
  - 因此不是重新 discovery，而是**复用父上下文再走同骨架**

#### `SubagentStart` hooks 的位置

这里还有一个容易漏掉的点：

- `BN(...)` 先构造：
  - `I = [...(forkContextMessages ? nu1(forkContextMessages) : []), ...promptMessages]`
- 然后才执行 `Ri1(agentId, agentType, signal)` 这条 `SubagentStart` hook runner
- `Ri1(...)` 返回的 `additionalContexts` 会先累积进 `z6[]`
- `z6[]` 非空时，`BN(...)` 才会额外构造一个：
  - `attachment.type = "hook_additional_context"`
  - `hookName = "SubagentStart"`
- 这个 attachment 不是插到前面，而是 `I.push(...)` 追加到 sidechain `messages` 末尾
- 随后 `CC(...)` 接到的就是：
  - `messages: I`
  - `systemPrompt: $6`
  - `userContext: o`
  - `systemContext: K6`

这意味着 `SubagentStart` 的本地线性顺序现在可以写得更硬：

```text
forkContextMessages
  -> nu1(...) 过滤未闭合 tool_use
  -> promptMessages
  -> hook_additional_context(SubagentStart)
  -> CC(...)
  -> po_(...)
  -> Lx8(F, userContext) 前插到最前
  -> H.callModel({ messages: Lx8(F, userContext), systemPrompt: WK(dj4(...)) })
```

因此最终送给模型的对象级顺序更接近：

```text
[
  Lx8(userContext) 生成的前置 user meta,
  ...forkContextMessages,
  ...promptMessages,
  hook_additional_context(SubagentStart)
]
```

但还要再补一层实现细节：

- `hook_additional_context` 作为 attachment 进入 `_X(...)` 后，会被 `dt1(...)` 物化成 meta user message
- 若它左侧已经是 user message，`_X(...) / Mg8(...)` 可能把它继续并进同一个 user message
- 所以“位于末尾”说的是 **进入 `CC(...)` 前的 transcript entry 顺序**
- 最终 API payload 里，它未必保留成独立 `message object`

多个 `SubagentStart` hook 的顺序也不能简单写成“配置顺序”：

- `et1(...)` 先按 source/type 组装匹配结果，顺序本身是稳定的
- 但 `FC(...)` 会把这些 hook 映射成并发 generator，再经 `MC8(...)` 用 `Promise.race(...)` 汇合
- `BN(...)` 对 `Ri1(...)` 的消费又只是在 `for await` 里按到达顺序 `push(...additionalContexts)`
- 因而 **同一事件下多个 hook 的 `additionalContext` 最终顺序是完成顺序，不是 matcher/配置顺序**

还有一个容易误判的边界：

- `Ri1(...)` 底层仍能产出 `blockingError`、`preventContinuation`、`message` 等通用 hook 结果
- 但 `BN(...)` 这个调用点当前只消费 `additionalContexts`
- 所以对 `SubagentStart` 而言，本地活语义实际上只有 **附加上下文注入**
- 没看到 stop/block/systemMessage 在这一 callsite 有活控制效果

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
