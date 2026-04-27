# WebSearchTool 深挖

## 本页用途

- 单独拆出 `web_search_tool` 的本地实现、远端调用链、流事件回收、结果整形与版本边界。
- 避免把这部分细节继续堆在 provider 总览页里。

## 相关文件

- [05-model-adapter-provider-and-auth.md](./05-model-adapter-provider-and-auth.md)
- [06-stream-processing-and-remote-transport.md](./06-stream-processing-and-remote-transport.md)
- [08-web-fetch-tool.md](./08-web-fetch-tool.md)
- [../05-appendix/02-evidence-map.md](../05-appendix/02-evidence-map.md)

## 一句话结论

`WebSearchTool` 不是本地搜索引擎客户端，而是一个把 `query / domain filter` 包装成 `web_search` server-side tool schema、再通过 `client.beta.messages.create()` 发给 Anthropic message API 的本地包装器。

因此当前能直接还原的是：

- 本地 schema 与输入校验
- 请求装配方式
- 流式事件回收逻辑
- `WebSearchOutput` 与 transcript `tool_result` 的整理逻辑

当前还不能直接还原的是：

- Anthropic 服务端内部到底用了哪家网页搜索供应商
- 服务端是否按地区、provider、模型切换不同搜索后端
- 服务端是否还隐式注入了 ranking/filter/user-location 等参数

## 证据落点

- `cli.js`
  - `Ow4 / jw4 / Hw4 / Ka1 / _l_`
  - `WebSearchTool` schema、`call(...)`、`mapToolResultToToolResultBlockParam(...)`
  - `Jk6(...)`
  - `client.beta.messages.create(...) -> POST /v1/messages?beta=true`
  - `web_search_tool_result` 与 `web_fetch_tool_result` 都被归为 `tool-input`
  - bundle 内嵌 SDK 文档里的 `WebSearchTool20260209`
- `claude/sdk-tools.d.ts`
  - `usage.server_tool_use.web_search_requests / web_fetch_requests`
  - `WebSearchOutput`
  - `WebSearchInput`

## 本地工具契约

当前本地输入 schema 已直接可还原：

```text
{
  query: string,
  allowed_domains?: string[],
  blocked_domains?: string[]
}
```

本地输出 schema 已直接可还原：

```text
{
  query: string,
  results: ({ tool_use_id, content: [{ title, url }] } | string)[],
  durationSeconds: number
}
```

除此之外，还能直接确认：

- `isReadOnly() -> true`
- `isConcurrencySafe() -> true`
- `maxResultSizeChars = 100000`
- `shouldDefer = true`
- `searchHint = "search the web for current information"`

输入校验规则也很明确：

- `query` 不能为空
- `allowed_domains` 与 `blocked_domains` 不能同时出现

权限模型也不是“本地发 HTTP 请求”的那一类，而是单独走一个工具权限 gate：

```text
{
  behavior: "passthrough",
  message: "WebSearchTool requires permission."
}
```

这说明它在权限层面被当作“需要用户同意的 server-side capability”，而不是普通本地网络工具。

## server-side tool schema

`Kl_(A)` 当前实际构造的 schema 形状是：

```text
{
  type: "web_search_20250305",
  name: "web_search",
  allowed_domains,
  blocked_domains,
  max_uses: 8
}
```

这里有三个非常关键的结论：

1. 当前 bundle 运行时路径里，`web_search` 的 server tool version 仍然是 `20250305`。
2. 本地传上去的可见字段只有 `allowed_domains / blocked_domains / max_uses`，没有看到 `user_location`。
3. CLI 侧没有任何 Google/Bing/SerpAPI 一类 provider adapter；它只是把 schema 塞进 message API。

## `call(...)` 的真实装配路径

`WebSearchTool.call(...)` 当前可以还原成下面这条链：

```text
WebSearchTool.call(input, context)
  -> user message:
       "Perform a web search for the query: <query>"
  -> O = Kl_(input)
  -> Jk6({
       messages: [user message],
       systemPrompt: "You are an assistant for performing a web search tool use",
       tools: [],
       options: {
         extraToolSchemas: [O],
         querySource: "web_search_tool",
         toolChoice?: { type: "tool", name: "web_search" },
         model,
         effortValue,
         ...
       }
     })
```

这条链里最重要的不是“调用了 `Jk6`”，而是它怎么调用：

- 传给 `Jk6` 的 `tools` 是空数组
- `web_search` 不是普通本地 tool，而是通过 `extraToolSchemas` 注入
- `systemPrompt` 是专用的
- `querySource` 被明确标成 `"web_search_tool"`

因此它更像是“借主模型通道驱动一次受限 server tool use”，而不是把搜索能力注册成普通本地工具再让主循环执行。

## `toolChoice` 不是恒定强制

`toolChoice` 只在 `tengu_plum_vx3` flag 打开时才显式变成：

```text
{ type: "tool", name: "web_search" }
```

同一个 flag 还会联动：

- 把 `thinkingConfig` 改成 `{ type: "disabled" }`
- 把 `model` 改成 `mH()`

否则会走：

- `thinkingConfig = q.options.thinkingConfig`
- `model = q.options.mainLoopModel`
- `toolChoice = undefined`

所以更严格的说法不是“`WebSearchTool` 总是强制调用 `web_search`”，而是：

- `extraToolSchemas:[web_search]` 是恒定存在的
- `toolChoice:web_search` 只在特定 feature flag 分支下被显式强制

## 从本地包装器到远端 API 的链

这一段已经没有实质歧义：

```text
WebSearchTool.call(...)
  -> Jk6(...)
    -> hC1(...)
      -> _I4(...)
        -> VN8(() => _y(...), requestFn, retryContext)
          -> client.beta.messages.create(...)
            -> POST /v1/messages?beta=true
```

所以网页搜索能力对客户端来说，最终并不是打某个公开搜索 endpoint，而是打：

- `https://api.anthropic.com/v1/messages?beta=true`

这意味着：

- `web_search` 是 Anthropic message API 暴露出来的 server-side tool capability
- CLI 只负责把工具 schema 和 prompt 打包进一次 beta messages 请求

## 流式事件如何被本地二次解释

`WebSearchTool.call(...)` 不会直接等一个“完整搜索结果对象”，而是边收流边维护本地进度状态。

它至少监听三类关键事件：

### 1. `content_block_start` + `server_tool_use`

作用：

- 记录 `tool_use_id`
- 重置当前累积的 partial query

### 2. `content_block_delta` + `input_json_delta`

作用：

- 追加 partial JSON
- 用正则从 partial JSON 里抽出 `"query": "..."` 的当前值
- 当 query 变化时，发出本地进度事件：

```text
{
  type: "query_update",
  query
}
```

这一点很重要，因为 CLI 上看到的“Searching: <query>”不是服务端直接给 UI 的状态，而是客户端自己从 `input_json_delta` 二次提炼出来的。

### 3. `content_block_start` + `web_search_tool_result`

作用：

- 读取 `tool_use_id`
- 读取 `content`
- 发出本地进度事件：

```text
{
  type: "search_results_received",
  resultCount: Array.isArray(content) ? content.length : 0,
  query
}
```

也就是说，“Found N results for <query>”同样是客户端根据流事件自己算出来的。

## UI 层展示并不复杂，但很能说明真实协议

`renderToolUseMessage`、`renderToolUseProgressMessage`、`renderToolResultMessage` 这几层可以反推 UI 侧到底认哪些状态。

### `Ow4(...)`

工具使用摘要会展示：

- 查询词本身
- verbose 模式下的 `allowed_domains`
- verbose 模式下的 `blocked_domains`

### `jw4(...)`

只识别两种进度事件：

- `query_update` -> `Searching: <query>`
- `search_results_received` -> `Found <count> results for "<query>"`

### `Hw4(...)`

工具结果摘要只展示：

- 一共做了几次 search
- 总耗时

这里用 `tc_(results)` 统计 `results` 数组里“非字符串项”的数量，把它当作 search 次数。

这进一步说明 `WebSearchOutput.results` 是一个混合数组：

- 对象项表示某次 `web_search_tool_result`
- 字符串项表示模型文本说明

## `_l_(...)`：最终结构化结果的整理器

收完流之后，代码会把所有 `assistant` 消息块拍平，再交给 `_l_(...)`。

`_l_(...)` 的实际行为可以概括成：

1. 遇到 `server_tool_use`：
   - 结束前一段文本聚合
   - 自己不产出结果项
2. 遇到 `web_search_tool_result`：
   - 如果 `content` 不是数组，转成 `Web search error: <error_code>`
   - 如果 `content` 是数组，只保留每项的：
     - `title`
     - `url`
3. 遇到 `text`：
   - 聚合为普通字符串说明

最终输出：

```text
{
  query,
  results: [
    { tool_use_id, content: [{ title, url }, ...] },
    "model commentary",
    ...
  ],
  durationSeconds
}
```

这里还可以额外下一个判断：

- 客户端当前没有保留 search hit 的摘要、snippet、rank、domain、published_at 等更丰富字段
- 即使服务端内部真实结果更丰富，当前 CLI 路径最终对外暴露的稳定字段只有 `title` 和 `url`

## transcript 里的最终 `tool_result` 不是 JSON

`mapToolResultToToolResultBlockParam(...)` 会把结构化结果再转成一段给模型继续消费的文本，而不是把 `WebSearchOutput` 原样塞进 transcript。

文本模板大致是：

```text
Web search results for query: "<query>"

<string commentary>

Links: <格式化后的 title/url 列表>

REMINDER: You MUST include the sources above in your response to the user using markdown hyperlinks.
```

这意味着两件事：

1. CLI 自己的工具返回是结构化的，但回灌给模型继续推理时会被降成文本。
2. “必须引用上面的 sources 并用 markdown hyperlink” 这一要求，不是系统层自动注入的引用协议，而是 `web_search_tool` 的 transcript 文本模板里显式追加的提醒。

## 与 usage 统计的关系

`sdk-tools.d.ts` 里的 `AgentOutput.usage.server_tool_use` 已直接暴露：

```text
{
  web_search_requests: number,
  web_fetch_requests: number
}
```

这说明对 CLI 的上层统计来说，`web_search` 与 `web_fetch` 这两类 **server-side tool result / usage 类型** 都已经进入统一计数面。

但这里要保留一个重要边界：

- `web_search`
  - 已确认有本地 wrapper 主动驱动 server-side tool use
- `web_fetch`
  - 当前只确认代码库认识其 server-side result/usage 类型
  - 不能据此反推当前用户态 `WebFetch` 已经等价于 server-side tool

这也和 streaming 层里把：

- `server_tool_use`
- `web_search_tool_result`
- `web_fetch_tool_result`

都归到 `tool-input` UI 状态是一致的。

## 一个重要的“版本漂移”现象

当前 bundle 里同时存在两套关于 `web_search` 的证据：

### 运行时代码路径

真实运行时构造的是：

```text
web_search_20250305
```

### bundle 内嵌的 SDK/示例文档

内嵌文本里却出现了：

```text
WebSearchTool20260209
web_search_20260209
AllowedDomains
BlockedDomains
MaxUses
UserLocation
```

这说明至少存在下面两种可能：

1. bundle 内嵌了一份比当前运行时代码更“新”的上游 SDK 文档。
2. Anthropic server tool schema 在其他分支/版本里已经升级过，但当前 CLI 实际走的本地包装器还没切过去。

当前不能直接下结论说：

- 运行时实际上已经在用 `web_search_20260209`
- 服务端一定收到了 `UserLocation`

因为本地可执行路径里，当前能直接看到的还是 `web_search_20250305`，也没有本地 `user_location` 传参。

## 与 `web_fetch` 的关系

当前只保留与本页直接相关的对照结论：

- 两者都属于 `server_tool_use` 相关类型族
- 两者都在 usage 统计中单独计数
- 两者的 result block 都会被流式 UI 归到 `tool-input`
- `web_fetch` 的本地实现与 server-side 边界已拆到 [08-web-fetch-tool.md](./08-web-fetch-tool.md)，本页不再展开

## 当前已经能稳下来的结论

1. `WebSearchTool` 是本地包装器，不是本地搜索 provider adapter。
2. 它通过 `Jk6 -> _I4 -> client.beta.messages.create()` 走统一 beta messages API。
3. `web_search` 是通过 `extraToolSchemas` 注入的 server-side tool，不是普通本地 `tools` 数组成员。
4. CLI 侧自己从 `server_tool_use / input_json_delta / web_search_tool_result` 解析出用户可见进度。
5. 对外暴露的稳定结构化结果是 `WebSearchOutput`，但回灌给模型的是文本 `tool_result`。
6. 当前本地运行时代码使用的是 `web_search_20250305`，与 bundle 内嵌文档里的 `web_search_20260209` 存在版本漂移。

## 当前还不能钉死的点

- Anthropic 服务端内部的真实网页搜索供应商
- 服务端返回的原始结果字段是否比 `{ title, url }` 更丰富
- `web_search_20260209` 与 `UserLocation` 是否已经在别的客户端版本启用
- `tengu_plum_vx3` 在真实线上环境中的默认开启条件
- 服务端是否对 query 做了额外重写、地域扩展、safe-search 或 ranking/filtering

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
