# API payload 顺序、prompt caching 与最终边界

## 本页用途

- 单独承接最终发给 `beta.messages.create(...)` 的 `messages/system` 对象级顺序。
- 把 `_X / LZz / hZz`、prompt caching、system prompt 分块缓存，以及最终 `payload.system / payload.messages` 的边界固定到同一页。

## 相关文件

- [../03-prompt-assembly-and-context-layering.md](../03-prompt-assembly-and-context-layering.md)
- [../05-attachments-and-context-modifiers.md](../05-attachments-and-context-modifiers.md)
- [../06-context-runtime-and-tool-use-context.md](../06-context-runtime-and-tool-use-context.md)
- [../../01-runtime/06-stream-processing-and-remote-transport.md](../../01-runtime/06-stream-processing-and-remote-transport.md)

## API payload 顺序与 prompt caching

### 最终 API request 的 `messages[]` 对象级顺序

这一段现在已经可以从“内容顺序”推进到“最终发给 `beta.messages.create(...)` 的 message object 顺序”。

`_I4(...)` 的最后一跳是：

```text
k = _X(transcriptLikeMessages, availableTools)
x = hZz(systemSections, enablePromptCaching, ...)
payload = {
  messages: LZz(k, ...),
  system: x,
  ...
}
-> client.beta.messages.create(payload)
```

这里有四个已经可以直接写死的结论。

#### 1. `LZz(...)` 不再改 message 相对顺序

`LZz(...)` 只做两类事：

- 把归一化后的 `user/assistant` transcript message 映射成 API `role/content`
- 注入 prompt cache 相关 `cache_control / cache_reference / cache_edits`

它不会再重排 `messages[]` 的相对先后。  
因此真正决定对象顺序的是 `_X(...)`，不是 `LZz(...)`。

这里还能继续收紧到 API payload 级细节。

#### `LZz(...)` 的 cache breakpoint 选择

当前本地直接看到：

```text
lastBreakpointMessageIndex =
  skipCacheWrite ? messages.length - 2 : messages.length - 1
```

再逐条把 transcript message 交给：

- `GZz(...)`：user message 映射
- `vZz(...)`：assistant message 映射

并把“是否是 breakpoint message”作为布尔位传进去。

因此更稳的语义不是“所有消息都可能带 cache_control”，而是：

- **默认只会把最后一条消息当作写 cache 的候选**
- `skipCacheWrite: true` 时，会把 breakpoint 左移一条
- 这正好对应 compact / side-question 这类“想读旧 cache，但不想污染新 cache”的 helper 路径

#### `cache_reference` 不是全局乱打，而是只补在 breakpoint 之前的旧 `tool_result`

`LZz(...)` 在启用 prompt caching 时，会先找到：

- **最后一个包含 `cache_control` block 的 message index**

然后只对它之前的 user messages 做一次补写：

- 若 block 是 `tool_result`
- 则补上：

```text
cache_reference = tool_use_id
```

因此当前本地更准确应写成：

- `cache_reference` 的目标是**让旧的 tool_result 块引用已缓存前缀**
- 它不会给 breakpoint 之后的新块乱补
- 也不是所有 `tool_result` 一律带 `cache_reference`

#### `cache_edits` 的 consumer 语义已可见，但当前 bundle 里没有活的本地 producer

`LZz(...)` 内部确实保留了一套完整的 `cache_edits` 注入骨架：

1. `Y`：`pinnedEdits`
   - 按 `userMessageIndex` 精确插回指定 user message
2. `z`：`AEq()` 取出的单个 edit block
   - 从后往前找最后一个 user message
   - 插进去

两路在落地前都会过同一个去重器：

- 以 `edit.cache_reference` 去重
- 已出现过的 edit 不再重复注入

如果只看 `LZz(...)` 自身，语义会像：

- **把 cache edits 当成引用旧缓存前缀的删除/裁剪说明**
- **并且严格避免同一 `cache_reference` 重复出现**

但这里必须继续收紧到“当前本地 bundle 的活路径”，否则会写过头。

`_I4(...)` 在构造请求时，当前本地直接是：

```text
let X = false
let D = ""

T6 = X ? AEq() : null
z6 = X ? qEq() : []
i6 = X && firstParty && querySource === "repl_main_thread"

messages = LZz(k, t6, querySource, i6, T6, z6, ...)
```

因此当前可直接写死：

- `cache_edits` 相关注入分支被 `X = false` 整体关掉
- cache-editing beta header 也不会发送
- 即使 `LZz(...)` 有注入逻辑，当前运行态也不会真正把 `AEq()/qEq()` 的结果喂进去

进一步对整份 bundle 做字符串级追踪，还能确认：

- `Gu1`
- `fk6`
- `sVq`
- `resetCachedMCState`

只出现在同一段 microcompact helper 代码里，没有第二处写入或赋值。  
也就是说当前本地并不存在“再往上游一层就能看到的 producer 链”。

这会落成四个更硬的结论：

- `AEq()` 的语义虽然是“取 `Gu1` 后清空”，但 `Gu1` 当前只看到 `null` 初始化
- `qEq()` 虽然读取 `fk6.pinnedEdits`，但 `fk6` 当前只看到 `null` 初始化
- `KEq()` 只有在 `fk6` 已存在时才会 push，当前等价于 no-op
- `Sn()` 虽然会尝试 `resetCachedMCState(fk6)`，但 `fk6` 与 `sVq` 当前都没有活赋值

因此这条链在当前 bundle 里更准确的表述应是：

- **`LZz(...)` 保留了 `cache_edits` 的消费协议**
- **但当前本地 bundle 没有看到活的上游 producer，也没有看到开关被打开**
- **实际可执行路径里，`cache_edits` 注入当前应视为未启用**

#### 若未来启用 `cache_edits`，其插入位置也不是任意点

真正插入时走的是 `iqA(...)`：

- 优先插在该 user message 内最后一个 `tool_result` 后面
- 若该位置已经是尾部，还会补一个 `"."` 文本占位
- 若根本没有 `tool_result`，才退化成插到末尾附近

因此这里的本地实现意图更像：

- 尽量让 `cache_edits` 靠近它所修剪的 tool-result 语境
- 而不是把 edit block 当成独立 message 随便挂在尾部

#### `hZz(...)`：system prompt 也不是一整块 cache，而是按 scope 拆块

`hZz(...)` 不是直接把 `WK(systemSections)` 包成一个 text block。  
它会先经 `lqA(...)` 把 system prompt sections 拆成：

- `cacheScope: null`
- `cacheScope: "org"`
- `cacheScope: "global"`

再逐块映射成 API `system` text blocks，并只给 `cacheScope !== null` 的块加：

- `cache_control: UF({ scope, querySource })`

当前本地还直接看到两条关键分支：

1. 找到动态边界 `Jw6`
   - 边界前静态段可打 `global`
   - 边界后动态段不打全局 cache
2. `skipGlobalCacheForSystemPrompt`
   - 直接退回不用 `global` scope 的拆块策略

其中 `skipGlobalCacheForSystemPrompt` 现在还能再写得更硬。

当前本地不是任意调用方手工决定它，而是 `_I4(...)` 在 request build 阶段按条件计算：

- 先要求全局 system prompt cache 模式已开启
- 再要求本轮 `tools` 数组里存在**活的 MCP tool schema**
- 且这些 MCP tools 不是仅 deferred / pending 的占位项

命中后会把 `skipGlobalCacheForSystemPrompt = true` 传给 `hZz(...)`。  
此时 `lqA(...)` 会改走：

- billing header -> `cacheScope: null`
- org header -> `cacheScope: "org"`
- 其余 system text -> `cacheScope: "org"`

而不是：

- 边界前静态段 -> `global`
- 边界后动态段 -> `null`

因此这里更稳的判断应是：

- `skipGlobalCacheForSystemPrompt` 是**request-level 的全局降级开关**
- 它不是 compact 专属逻辑
- 主线程、subagent、compact fallback、其他 non-main-thread query 只要带上活的 MCP tool schema，都可能触发这条降级
- 一旦触发，即使 prompt 内存在 `__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__`，也不会再产出 `global` scope 那段静态前缀

因此 system prompt 的本地缓存语义现在可以更精确地写成：

- **不是整份 system prompt 一个 cache breakpoint**
- **而是 billing / org / global / dynamic 段拆块后分别决定是否可缓存**
- `__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__` 的真实作用，就是把“可长期复用的静态前缀”和“每轮易变后缀”切开

#### 2. 当前轮进入 `_X(...)` 前的尾部形状

普通当前轮先由 `BU4(...)` 生成：

```text
[current-turn user message, ...attachments]
```

如果有 `UserPromptSubmit` hook 产出的 `hook_additional_context / hook_success / hook_non_blocking_error` attachment，它们会在 `AU8(...)` 里继续被追加到这个尾部数组后面。

因此 `_X(...)` 收到的“当前轮尾部”更准确地是：

```text
current-turn user
-> P6z(...) attachments
-> UserPromptSubmit hook attachments
```

#### 3. `u0z(...)` 先把尾随 attachment 左移到上一个边界

`u0z(...)` 会从后往前扫：

- 遇到 `attachment` 先暂存
- 遇到 `assistant`
- 或遇到 `user` 且其首个 block 是 `tool_result`

就把暂存 attachment 整批插到这个边界之前。

因此 attachment 的最终锚点不是“当前轮 user 后面”，而是：

- 优先贴到最近一个 `assistant` 之后
- 若没有，就贴到最近一个 `tool_result` user 之后
- 再没有，才留在最前部剩余位置

#### 4. `_X(...)` 会把相邻 user message 折叠成更少的对象

`_X(...)` 之后的对象级行为现在也已明确：

- `system` transcript entry 会先转成 user meta message
- `attachment` 会经 `dt1(...)` 展开成一个或多个 user meta message
- 若前一个已输出对象是 `user`，无论是普通 user 还是 meta user，都会立刻用 `Mg8(...)` 合并
- 末尾还会再经过一次 `d0z(...)`，把所有剩余相邻 user message 继续折叠
- 合并后的单个 user message 内部，`cb4(...)` 会把 `tool_result` block 放到前面，再接 text/meta block
- 若两端边界都是 text block，`c0z(...)` 会在中间补一个换行

所以对“普通当前轮 + attachment”来说，更稳的最终近似已经不是“若干独立 message”，而是：

```text
messages[]
  = [
      Lx8(...) 生成的前置 user meta message,
      ...历史 assistant/user message,
      previous assistant 或 previous tool_result-user,
      merged user message {
        tool_result blocks first (if any),
        then attachment-derived meta blocks,
        current-turn user text last
      }
    ]
```

也就是说，当前轮 attachment 在最终 API payload 里通常不会保留成独立 `message object`，而会并进它左侧边界之后的那个 user message。

### `system` 与 `messages` 的最终边界

因此主线程最终 request 现在可以更精确地写成：

```text
payload.system
  = hZz(
      WK(
        lX8
        -> cX8
        -> bC(...)
        -> dj4(..., systemContext)
      )
    )

payload.messages
  = LZz(
      [
        Lx8(userContext) 生成的前置 user meta,
        ..._X(...) 归一化后的 transcript user/assistant objects
      ]
    )
```

在当前本地 bundle 可见代码里：

- `systemContext` 默认仍只看到 `{ gitStatus? }`
- `userContext` 默认仍只看到 `{ ClaudeMd?, currentDate }`
- 当前轮 attachment / hook additional context / plan/skill meta 仍全部走 `messages[]`
- 没看到第二套“把这些内容直接改写进 `system`”的本地路径

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
