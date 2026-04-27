# Fork-Family、cacheSafeParams 与 Request Snapshot 复用

## 本页用途

- 用来把 `lZ(...)` 家族从“也会走 `CC(...)`”进一步拆到 `cacheSafeParams` 生命周期层。
- 用来固定 request snapshot 的 producer、全局单槽缓存、以及 reuse 与 fresh-build 的精确边界。

## 相关文件

- [../04-non-main-thread-prompt-paths.md](../04-non-main-thread-prompt-paths.md)
- [../03-prompt-assembly-and-context-layering.md](../03-prompt-assembly-and-context-layering.md)
- [../06-context-runtime-and-tool-use-context.md](../06-context-runtime-and-tool-use-context.md)
- [../../01-runtime/04-agent-loop-and-compaction.md](../../01-runtime/04-agent-loop-and-compaction.md)

## 一句话结论

- `lZ(...)` 自己不重新 discovery；它只是消费已有的 `cacheSafeParams` 再调用 `CC(...)`。
- 真正决定是否会再次触发 `sj() / InstructionsLoaded` 的，是 `cacheSafeParams` 来自 `ML(...) / xe6()` 复用，还是来自 `I1z(...) / fD4(...)` 这类 fresh build。

## `lZ(...)` 家族还要再拆一层：关键不在 merge，而在 `cacheSafeParams`

这一点现在也能写得更硬。

`lZ(...)` 自己不会重新 discovery。它只是：

```text
let {
  systemPrompt,
  userContext,
  systemContext,
  toolUseContext,
  forkContextMessages
} = cacheSafeParams

let forkCtx = ts6(toolUseContext, overrides)

CC({
  messages: [...forkContextMessages, ...promptMessages],
  systemPrompt,
  userContext,
  systemContext,
  toolUseContext: forkCtx,
  ...
})
```

因此对于 `lZ` fork-family，真正决定会不会再次触发 `sj() / InstructionsLoaded` 的，不是 `lZ(...)` 本体，而是 **`cacheSafeParams` 从哪来**。

### request snapshot 生命周期：`ML -> Cj4 -> xe6 -> I1z`

这一条可以单独写成一条显式生命周期，因为它描述的是 **request-level prompt snapshot**，不是服务端 prompt cache 本身。

```text
主线程 / SDK request 已完成装配
  -> ML(ctx)
       打包:
         systemPrompt
         userContext
         systemContext
         toolUseContext
         forkContextMessages = messages

主线程 / SDK 可复用点
  -> Cj4(snapshot)
       覆盖写全局单槽

旁路 side query 需要旧前缀
  -> xe6()
       读取最近一次 snapshot

/btw 没命中 snapshot
  -> I1z(ctx)
       fresh build:
         $X(...)
         _$()
         vO()
```

按函数职责拆开后，更适合记成下面这张表：

| 函数 | 角色 | 输入 | 输出 / 副作用 | 当前本地边界 |
| --- | --- | --- | --- | --- |
| `ML(A)` | snapshot packer | 当前 request context | 产出 `cacheSafeParams` 形状对象 | 只打包当前已装配好的 request，不做新 discovery |
| `Cj4(snapshot)` | latest-slot writer | 一个 snapshot | 覆盖写全局槽位 `aj4` | 不是 map，不保留历史版本 |
| `xe6()` | latest-slot reader | 无 | 返回 `aj4` | 命中时复用旧 snapshot；未命中时返回空 |
| `I1z(ctx)` | `/btw` side-question adapter | 当前 `ToolUseContext` | 命中 `xe6()` 则复用旧 prompt snapshot；否则 fresh build 一份新的 `cacheSafeParams` | `/btw` fallback 会重新跑 `$X / _$ / vO()`，SDK `side_question` 当前看不到同等 fallback |

因此更稳的生命周期结论是：

- `ML / Cj4 / xe6` 这条链描述的是**最近一次可复用 request snapshot 的产生、落槽与读回**
- `I1z` 不是 snapshot producer 本体，而是 `/btw` 的**命中优先、未命中 fresh-build** 适配层
- 命中 `xe6()` 时，side query 复用的是旧的 request-level prompt 产物，不会因此再次触发 `sj()`
- 只有 `I1z(...)` fallback 这类 fresh-build 分支，才会重新具备 `_$() / vO()` 以及 `InstructionsLoaded` 的本地条件

### 第一类：直接复用当前 request snapshot

这一类通常来自 `ML(ctx)`，本质是把当前已构好的：

- `systemPrompt`
- `userContext`
- `systemContext`
- `messages`

原样打包给 `lZ(...)`。  
当前本地直接看到的路径包括：

- `prompt_suggestion`
- `speculation`
- `extract_memories`
- `auto_dream`
- `session_memory`
- `agent_summary`
  - 这里的参数来自 `BN(...)` 的 `onCacheSafeParams`
- 若已在主链上生成过 cache-safe params 的 compact/fallback helper

这些路径的共同点是：

- **复用旧的 `userContext/systemContext`**
- **`lZ(...)` 本次不会 fresh-run `_$() / vO()`**
- 因而**不会因为这一步再次触发 `InstructionsLoaded`**

如果目标是 rewrite 时逐个复刻，仅写“这些都来自 `ML(...)`”还不够。  
当前已经可以把 producer 形态展开成一张完整表。

| 调用方 | 本地可见入口 | `cacheSafeParams` producer | `forkContextMessages` 来源 | 是否 fresh build | 备注 |
| --- | --- | --- | --- | --- | --- |
| `prompt_suggestion` | `bj4(...) -> Rs1(...) -> lZ(...)` | `bj4(...)` 先做 `K = ML(A)`，再传给 `Rs1(...)` | 当前 `A.messages` | 否 | `skipTranscript: true`、`skipCacheWrite: true` |
| `speculation` | `bs1(...) -> lZ(...)` | `z ?? ML(q)` | 当前 `q.messages` | 否 | 既支持调用方显式传 snapshot，也支持自己就地 `ML(q)` |
| `extract_memories` | `Eo_().Y(...) -> lZ(...)` | `f = ML(w)` | 当前 `W.messages` | 否 | `skipTranscript: true` |
| `auto_dream` | auto-dream worker -> `lZ(...)` | `ML(K)` | 当前 worker `K.messages` | 否 | `canUseTool` 还会额外包一层只读 Bash 限制 |
| `session_memory` | `Tuz(...) -> lZ(...)` | `ML(A)` | 当前 `A.messages` | 否 | 只在 `querySource === "repl_main_thread"` 时进入 |
| `agent_summary` | `nn6(...).H() -> lZ(...)` | 先拿上游 `K`，再改成 `D = {...K, forkContextMessages: X}` | `X = nu1(P.messages)`，即 agent transcript 归一化结果 | 否 | 这里复用的是旧 prompt snapshot，但消息链被替换成 agent 自己的 transcript |
| `compact` shared-prefix | `jk6(...) / ZVq(...) -> lZ(...)` | 调用方传入 `cacheSafeParams` | 调用方决定 | 取决于调用方 | `ZVq(...)` 自己不 fresh build，只消费外部给它的 snapshot |

这里还能再补一个容易漏掉的点：

- `prompt_suggestion` 的 producer 不是 `Rs1(...)` 自己造的
- 而是上游 `bj4(...)` 先对当前主线程 request 做了一次 `ML(A)`

`speculation` 则更微妙一些：

1. prompt suggestion 触发 speculation 时
   - 当前直接可见调用是 `bs1(_.suggestion, A, setAppState, false, K)`
   - 这里 `K` 就是前面 `bj4(...)` 造好的 `ML(A)`
2. pipelined suggestion promote 时
   - 当前可见调用是 `bs1(W, v, K, true)`
   - 没显式传第 5 个参数
   - 因而 `bs1(...)` 会走自己的 `ML(q)`

所以对 `speculation`，更完整的说法应是：

- **它不会走 `xe6()`**
- **也不会 fresh-run `_$() / vO()`**
- **它只是在“调用方显式给 snapshot”与“自己对当前上下文做 `ML(...)`”之间二选一**

### 第二类：复用最近一次主线程 / SDK 已缓存的 snapshot

这里的关键是 `xe6()`。

`Rj4(...)` 在 `querySource === "repl_main_thread" || "sdk"` 时，会把当前 turn 的：

- `systemPrompt`
- `userContext`
- `systemContext`
- `toolUseContext`
- `messages`

通过 `Cj4(ML(j))` 缓存在全局槽位里。  
后续若旁路请求直接取 `xe6()`，拿到的是**上一条主线程 / SDK turn 已经构好的上下文快照**，而不是重新 discovery。

这里还能继续收紧两个关键点。

### `xe6()` 不是多版本缓存，而是单槽覆盖

当前本地定义非常直接：

```ts
function Cj4(snapshot) {
  aj4 = snapshot
}

function xe6() {
  return aj4
}
```

因此这里不是：

- LRU
- per-session map
- per-query-source 多槽缓存

而就是：

- **一个全局单槽 latest snapshot**
- 每次 `Cj4(...)` 都是覆盖写

所以更稳的说法应是：

- `xe6()` 代表“最近一次主线程 / SDK 可复用 request 快照”
- 不是任意历史版本回溯器

当前本地直接看到：

- `/btw` side question：优先走 `I1z(...) -> xe6()`
- SDK `side_question`：直接取 `xe6()`

这又能继续分成两支：

- `xe6()` 命中
  - 直接复用旧 snapshot
  - 不会因为这一步再次触发 `sj()`
- `xe6()` 未命中
  - `/btw` 会退回 `I1z(...)` 的 fresh build：`$X(...) + _$() + vO()`
  - SDK `side_question` 则直接返回 `null`，当前看不到 fallback fresh-run

所以 side question 也不能笼统写成“会”或“不会”：

- **命中 `xe6()` 时不会**
- **仅 `/btw` 的 fallback fresh build 才会**
- **SDK side question 在没有 snapshot 时当前不会补跑 discovery**

### `/btw` 与 SDK side question 复用 `xe6()` 的粒度并不一样

这点以前也容易被写粗。

当前本地直接看到：

1. `/btw` 的 `I1z(...)`
   - 命中 `xe6()` 后返回：
     - `systemPrompt = xe6().systemPrompt`
     - `userContext = xe6().userContext`
     - `systemContext = xe6().systemContext`
     - `toolUseContext = 当前传入 A`
     - `forkContextMessages = b1z(A.messages)` 当前消息链
2. SDK `side_question`
   - 直接把 `xe6()` 整体 spread 成 `cacheSafeParams`
   - 只改：
     - `toolUseContext.abortController`

因此两者都叫“命中 `xe6()`”，但复用深度不同：

- `/btw` 更像**复用旧的 request-level prompt snapshot，但仍绑定当前本地消息链与当前 tool context**
- SDK side question 更像**近乎原样复用上一条缓存快照**

所以若要写严谨，应拆开说：

- `/btw` 命中 `xe6()` 不等于“连 messages 也完全复用旧 snapshot”
- 真正更接近“整包复用”的，是 SDK `side_question`

### 第三类：显式 fresh-build cache-safe params

这类 helper 虽然最后也把结果交给 `lZ(...)` 或 compact helper，但它们在交付前会自己重新构造：

- `systemPrompt`
- `userContext = _$()`
- `systemContext = vO()`

当前本地已直接看到的代表有：

- `I1z(...)` 的 fallback 分支
- `fD4(...)`

因此这类路径虽然调用的是 `lZ(...)` 或 compact helper，但**会再次具备 `sj() / InstructionsLoaded` 的本地条件**。

同样也可以把 producer 拆成具体表：

| fresh-build helper | 构造方式 | 下游消费者 | 是否经过 `lZ(...)` | 备注 |
| --- | --- | --- | --- | --- |
| `I1z(...)` miss `xe6()` | `$X(...) + _$() + vO()` | `/btw` side question | 是 | `forkContextMessages` 用的是 `b1z(A.messages)`，不是旧 snapshot 的 messages |
| `fD4(...)` | `$X(...) + bC(...) + _$() + vO()` | `jk6(...)` / reactive compact helper | 间接 | 这里连 `systemPrompt` 也 fresh-build，不只是 `user/systemContext` |

因此 `compact` 不能再只写成“有时复用，有时 fresh-build”，而应明确分开：

- `ZVq(...)` shared-prefix 尝试本身：**只消费外部给的 `cacheSafeParams`**
- `fD4(...)`：**显式新造一份 cache-safe params**
- 当前是否会重新具备 `InstructionsLoaded` 条件，取决于你走的是哪一支

## 因而 fork-family 现在应写成两层判断

单纯写“fork-family 复用主骨架”还不够，还需要再补一层：

- **`BN(...)` 家族**
  - 默认会重新走 `_$() / vO()`
  - 只有显式 override `userContext/systemContext` 时才旁路
- **`lZ(...)` 家族**
  - 自己不重新 discovery
  - 要看 `cacheSafeParams` 是来自 `ML(...)` / `xe6()` 复用，还是来自 `I1z(...) / fD4(...)` 这类 fresh build

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
