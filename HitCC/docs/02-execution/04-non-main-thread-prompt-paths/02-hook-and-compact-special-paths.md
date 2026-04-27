# Hook 与 Compact 的专用 Prompt 路径

## 本页用途

- 用来收拢非主线程三分类里的第三类，也就是明确旁路主线程 merge 的路径。
- 用来把 `hook_prompt`、`hook_agent`、verification 残留资产，以及 compact summarize 的 shared-prefix/fallback 分支拆开。

## 相关文件

- [../04-non-main-thread-prompt-paths.md](../04-non-main-thread-prompt-paths.md)
- [../01-tools-hooks-and-permissions/02-hook-system.md](../01-tools-hooks-and-permissions/02-hook-system.md)
- [../01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md](../01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md)
- [../03-prompt-assembly-and-context-layering.md](../03-prompt-assembly-and-context-layering.md)
- [../../01-runtime/04-agent-loop-and-compaction.md](../../01-runtime/04-agent-loop-and-compaction.md)

## 一句话结论

- `hook_prompt` 与 compact summarize fallback 明确旁路主线程 `system/messages` merge。
- `hook_agent` 只是复用 `CC(...)` loop，不复用主线程 context layering；compact shared-prefix 则是借 `lZ(...)` 读旧 snapshot，而不是再造一套 summarize merge。

## 第三类：旁路主线程 merge 的路径

这类路径不能再说“沿用同一套 merge”，最多只能说“底层仍会调用模型或复用 `CC`”。

### `hook_prompt`

`hook_prompt` 是直接模型调用：

- 不走 `BN(...)`
- 不走 `CC(...)` 主循环骨架
- 也没有 `Lx8(userContext)` / `dj4(systemContext)`

因此它本质上是 **独立的 prompt-eval 路径**。

### `hook_agent`

`hook_agent` 虽然会直接调用 `CC(...)`，但传入的是：

- `userContext: {}`
- `systemContext: {}`
- `systemPrompt: verifier-only prompt`

因此它对 `Lx8(...)` / `dj4(...)` 来说只是**空输入 no-op**。  
它更适合归类成：

- 复用 `CC` loop
- 但不复用主线程 context layering

### `hook_agent` verifier runner 还要再拆三层

如果只写“注入 `VM4()` + `dontAsk` + `Read(transcript)`”仍然偏粗。  
当前本地 bundle 已经能把它拆成三层独立边界。

#### 1. 最终可用工具集不是“父 tools 原样继承”

`RM4(...)` 里真正交给 verifier 的 `tools` 是：

```text
E = [
  ...Y.options.tools
    .filter(tool => !d3(tool, zX))
    .filter(tool => !XT6.has(tool.name)),
  VM4()
]
```

因此最终 tool set 不是简单“沿用父上下文全部工具”，而是：

- 先从父 `Y.options.tools` 起步
- 去掉已经等价于 verifier 返回通道的 `zX` tool，避免重复 completion channel
- 再去掉共享黑名单 `XT6` 里的工具
- 最后**强制追加**专用 structured-output tool `VM4()`

所以更稳的表述应是：

- `hook_agent` 有自己的一层**静态工具裁剪**
- 它不是把父 agent 的 tools 原样塞进 `CC(...)`

#### 2. `VM4()` 不是“建议调用”，而是完成握手

`VM4()` 本身的 prompt 已经写死：

- 必须在结束时调用
- 返回结构只接受 `ok` / `reason`

但真正的强制性来自三处一起配合：

1. `VM4().prompt()`
   - 明确要求 *exactly once at the end*
2. `yu8(...)`
   - 给当前 verifier agent 挂一个 reminder
   - 提示 `You MUST call the ${zX} tool ... Call this tool now.`
3. `RM4(...)` 主循环
   - 只把 `attachment.type === "structured_output"` 视为成功完成
   - 并且还要通过 `j68()` schema parse
   - 若 50 个 assistant turn 内始终没拿到该 structured output，就直接 abort

因此 `hook_agent` 的“完成”不是：

- assistant 回了自然语言

而是：

- **必须产出可解析的 `structured_output` attachment**
- **否则整个 verifier run 记为 cancelled / incomplete**

#### 3. transcript path、`Read(...)` allow 和 `dontAsk` 都是临时包装，不是全局改写

这里也能再收紧。

`RM4(...)` 里 transcript path 的来源是：

- 若当前本来就在 subagent 中：`H = $0(Y.agentId)`
- 否则：`H = Cz()`

也就是说 verifier 看的不是抽象的“当前会话”，而是：

- 主线程 stop hook -> 主 transcript
- subagent stop hook -> 对应 agent transcript

同时它只附加一条精确的 session allow rule：

- `Read(/${H})`

不是：

- 整个 transcript 目录白名单
- `Read(**/*.jsonl)` 之类宽泛规则

并且这条 allow 规则是通过 verifier 自己的 `p.getAppState()` 包装动态返回的。  
当前没有看到它把该规则持久写回父 app state。

`dontAsk` 也一样要写清边界：

- verifier wrapper 把 `toolPermissionContext.mode` 固定改成 `dontAsk`
- 但 `dontAsk` 不是 auto-allow
- `YP(...)` 看到 ask 时会直接降成 deny

所以三者合起来，`hook_agent` 的真实权限边界应写成：

- **只临时放行 `Read(exact transcript path)`**
- **其他工具仍要过 `YP(...)`**
- **凡是需要 interactive approval 的 ask，在 verifier 中都会被 `dontAsk` 压成 deny**

因此“最终可用工具集”更准确地说是：

- 静态过滤后的 `tools`
- 再交给 `YP(...)` 跑一次运行期权限判定
- 其中唯一明确额外放开的，是那条精确的 transcript `Read(...)`

### verification prompt 家族：现在应拆成“活路径”和“残留资产”

这一点现在可以比“verification 还有很多未知”写得更硬。

当前本地真正活着的 verification-like prompt，只直接看到两条：

1. **`hook_prompt`**
   - 走 `Xo(...)`
   - 固定 system prompt：`You are evaluating a hook in Claude Code.`
   - 显式带：
     - `querySource: "hook_prompt"`
     - `agents: []`
     - `hasAppendSystemPrompt: false`
     - `outputFormat: json_schema`
   - 因而它是**完全独立的 prompt-eval request**
2. **`hook_agent`**
   - 走 `CC(...)`
   - 固定 system prompt：`You are verifying a stop condition in Claude Code...`
   - 显式带：
     - `querySource: "hook_agent"`
     - `userContext: {}`
     - `systemContext: {}`
   - 另外还会：
     - 注入专用 structured-output tool `VM4()`
     - 把 `getAppState().toolPermissionContext.mode` 压成 `dontAsk`
     - 自动追加 `Read(transcript)` always-allow 规则
   - 因而它虽然复用 `CC`，但本质上是**stop-hook 专用 verifier runner**

这两条都不是普通 `subagent_type` 路径，也不是 `BN(...)` / `lZ(...)` 上的另一套 prompt merge。

与之相对，bundle 里还残留着一整段更强硬的 verifier 文本：

- `dtw = "You are a verification specialist..."`
- 文本末尾要求输出 `VERDICT: PASS / FAIL / PARTIAL`

但当前本地可见范围里，它只看到：

- `Cvq()` 中的声明与赋值

没有看到：

- `getSystemPrompt: () => dtw`
- 任意 built-in / custom agent 注册把它挂进去
- 任意本地 launch site 直接消费它

因此当前更稳的本地结论应改写为：

- **活着的 verification prompt 路径是 `hook_prompt` 与 `hook_agent`**
- **`dtw` 更像 verifier prompt 资产残留，不是当前发行版里可直接跑通的本地 agent wiring**

还可以再把 `subagent_type="verification"` 的来源收紧一层。

当前本地 bundle 里，这个字符串并不是来自某个 prompt producer 把 verifier agent 真正注册进 runtime，而是来自：

- `TodoWrite` 的 `mapToolResultToToolResultBlockParam(...)`
- `Task` 更新完成分支的 `tool_result` 文案生成

两处都只是把常量 `T28 = "verification"` 拼进：

```text
NOTE: ... spawn the verification agent (subagent_type="verification")
```

因此这里更准确的定位是：

- 它是 **tool-result nudge 文案**
- 不是 built-in agent 注册
- 不是 `getSystemPrompt: () => dtw`
- 也不是某个会自动 materialize agent definition 的 prompt producer

### `compact` summarize 核心路径

`compact` 现在必须拆成两层看：

1. **外围 orchestration**
   - 有些入口会先用现成的 `systemPrompt/userContext/systemContext`
   - 例如 `fD4(...)` 会生成一份 cache-safe params
2. **真正的 summarize 调用 `ZVq(...)`**
   - 直接走 `Jk6(...)`
   - `systemPrompt` 是固定的 summarize prompt
   - `messages` 是 `_X(...)` 后的专用 summarize 输入
   - 不走 `Lx8(...)`
   - 也不走 `dj4(...)`

因此更稳的表述是：

- `compact` 家族**不是一条单一 merge 路径**
- 其中 summarize 核心调用明确**旁路主线程 `system/messages` 分层**
- 但某些 compact helper / cache-sharing path 又会复用主线程前缀参数

### `compact` 家族内部还要再拆成“旧前缀复用尝试”和“专用 summarize fallback”

把 `jk6(...) / ZVq(...) / lZ(...) / Jk6(...)` 串起来后，当前本地已经能把 compact 的 prompt-cache sharing 写成更硬的两段式：

```text
full / partial compact
  -> ZVq({ summaryRequest, cacheSafeParams, ... })
     -> if tengu_compact_cache_prefix:
          try lZ({
            promptMessages: [summaryRequest],
            cacheSafeParams,
            querySource: "compact",
            maxTurns: 1,
            skipCacheWrite: true
          })
          -> assistant text exists ? success : fallback
        else:
          fallback

fallback:
  Jk6({
    systemPrompt: "You are a helpful AI assistant tasked with summarizing conversations.",
    messages: normalized summarize-only input,
    querySource: "compact",
    thinking: disabled
  })
```

这意味着当前本地应明确拆成两种 compact summarize 分支：

- **shared-prefix 尝试**
  - 只在 `tengu_compact_cache_prefix` 开启时发生
  - 走的是 `lZ(...)` fork-family
  - 依赖调用方提前提供的 `cacheSafeParams`
  - 明确带 `skipCacheWrite: true`
- **dedicated summarize fallback**
  - 走 `Jk6(...)` 专用 summarize request
  - 使用固定 summarize system prompt
  - 输入是 `_X($4_(O4_([...LN(messages), summaryRequest])), tools)` 这类 summarize-only 归一化消息
  - 不复用主线程 `Lx8(...) / dj4(...)`

这里还能继续压实几个高价值边界。

### 哪些 compact 分支有资格共享旧前缀

当前本地直接可见：

- **full compact `jk6(...)`**
  - 会把上游给它的 `cacheSafeParams` 继续传给 `ZVq(...)`
- **partial compact helper**
  - 同样调用 `ZVq(...)`
  - 也具备 shared-prefix 尝试入口
- **session-memory compact `_V8(...)`**
  - 不走 `ZVq(...)`
  - 因而当前看不到 prompt-cache prefix sharing

因此更稳的说法应是：

- **只有走 `ZVq(...)` 的 full / partial compact 分支，当前本地才具备“复用旧 prompt cache 前缀”的实现入口**
- `session_memory` 那条 compact 线当前应视为**没有这条共享旧前缀路径**

### shared-prefix 成功条件与回退条件

`ZVq(...)` 对共享前缀路径的成功条件并不宽松。  
当前本地直接看到：

- `lZ(...)` 返回后，会取 `SW(messages)` 的最后 assistant
- 只有在该 assistant **存在且能提取出合法文本 summary** 时，才算 `tengu_compact_cache_sharing_success`

下面这些情况都会立即退回 dedicated summarize fallback：

- `lZ(...)` 抛错
- 返回里没有可用 assistant text
- 文本提取后为空或不合法

因此当前更准确的表述不是“开了 flag 就一定共享旧前缀”，而是：

- **先尝试共享旧前缀**
- **共享失败就立刻 fresh-build 专用 summarize request**

### `skipCacheWrite` 在 compact 里的真实位置

这一项以前也容易写粗。

当前本地 bundle 可直接确认：

- `side_question`
- `prompt_suggestion`
- `compact` 的 shared-prefix 尝试

都会显式传 `skipCacheWrite: true`。

其中 compact 这一支的意义最明确：

- 它不是“禁止读 cache”
- 而是**在复用旧 snapshot 做 summarize side query时，不再给这次 side query 额外写新的 message cache breakpoint**

因此 compact shared-prefix 更接近：

- **读旧前缀**
- **拿 summary**
- **不把这次 summarize side query 再当成新的 cache 写入机会**

### streaming retry 只属于 fallback path

`tengu_compact_streaming_retry` 只包在 `Jk6(...)` 那条 dedicated summarize fallback 外层。  
shared-prefix 的 `lZ(...)` 尝试本身当前没有第二层 retry loop。

因此失败语义应拆开写：

- **shared-prefix 失败**
  - 直接转 fallback
- **fallback streaming 失败**
  - 才会受 `tengu_compact_streaming_retry` 控制重试次数

这又能推出一个更稳的结论：

- compact 家族当前并不是“先共享前缀失败后还继续共享前缀重试”
- 而是“共享前缀只试一次，之后改走 dedicated summarize request”

### `bC()` 之后是否还有 system 重排：本地答案基本已出

这一项现在也可以从“完全未知”收紧到更具体的本地结论。

`bC(...)` 之后，本地还会经过 `WK(...)` 包装；但这层的职责更接近：

- 抽出 `x-anthropic-billing-header`
- 抽出特定 cache key / org-scope header
- 在存在 boundary marker 时，把 boundary 前后的普通 system blocks 分成 `global` 与 `null` cache scope 两段
- 不改变普通 section 在各自分段内部的相对顺序

因此更稳的本地表述应是：

- **有后处理**
- 但它主要是 **cache-scope packaging / special header hoisting**
- 不是再发明一套新的 system section 语义顺序
- 对普通 system sections 而言，目前没看到在 `bC()` 之后再发生任意重排

### 当前更稳的分类结论

如果把“是否沿用同一套 merge”说得更严格一些，那么现在应改写为：

- **主线程 / subagent / fork-family(`lZ`)**：沿用同一套 request-level merge 骨架
- **Explore / Plan / omitClaudeMd / magic_docs / fork override**：沿用同骨架，但输入被裁剪或 override
- **hook_agent**：复用 `CC` loop，但把 `userContext/systemContext` 清空，不能算完整沿用主线程 layering
- **hook_prompt / compact summarize core**：明确旁路主线程 merge

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
