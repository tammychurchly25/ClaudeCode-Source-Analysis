# WebFetchTool 深挖

## 本页用途

- 单独拆出当前 CLI 里的 `WebFetch` 工具实现、权限门、URL 获取路径、内容二次摘要与协议边界。
- 明确区分“本地 `WebFetch` 工具”和“bundle 中同时出现的 server-side `web_fetch` capability”。

## 相关文件

- [05-model-adapter-provider-and-auth.md](./05-model-adapter-provider-and-auth.md)
- [06-stream-processing-and-remote-transport.md](./06-stream-processing-and-remote-transport.md)
- [07-web-search-tool.md](./07-web-search-tool.md)
- [../05-appendix/02-evidence-map.md](../05-appendix/02-evidence-map.md)

## 一句话结论

当前 bundle 里真正落地实现的是一个名为 `WebFetch`、用户可见名为 `Fetch` 的本地工具：它先在本地做 URL 校验、domain preflight、HTTP 获取、HTML 转 markdown，然后再把页面内容和用户 prompt 交给一个小模型做二次提炼。

因此它和 [07-web-search-tool.md](./07-web-search-tool.md) 里的 `WebSearchTool` 不是同一类实现：

- `WebSearchTool` 已明确是把 `web_search` schema 注入 beta messages API 的 server-side wrapper
- 当前 `WebFetch` 则是“本地抓取 + 本地再摘要”

同时还存在另一层单独的事实：

- streaming/UI/usage 统计和 bundle 内嵌 SDK 文档，确实都认识 `web_fetch_tool_result` / `web_fetch_20260209`
- 但当前运行时代码里，还没找到一个与 `WebSearchTool.call(...)` 同等级、会主动构造 `web_fetch_202xxxx` schema 并发起 server-side `web_fetch` 调用的本地包装器

## 证据落点

- `cli.js`
  - `PO = "WebFetch"` 与工具 prompt 文本 `Ed7`
  - `Qo1 / CY4 / bY4 / IY4 / do1 / co1 / lo1`
  - `WebFetch` 的 schema、权限、`call(...)`、`mapToolResultToToolResultBlockParam(...)`
  - `web_fetch_tool_result` 被流式 UI 归为 `tool-input`
  - `server_tool_use` / result block 的通用 streaming 累积逻辑
  - `UZ(...)` 小模型调用辅助函数
  - bundle 内嵌 SDK/示例文档里的 `WebFetchTool20260209` / `web_fetch_20260209`
- `claude/sdk-tools.d.ts`
  - `usage.server_tool_use.web_search_requests / web_fetch_requests`
  - `WebFetchOutput`
  - `WebFetchInput`

## 本地工具契约

当前本地输入 schema 已直接可还原：

```text
{
  url: string,
  prompt: string
}
```

本地输出 schema 已直接可还原：

```text
{
  bytes: number,
  code: number,
  codeText: string,
  result: string,
  durationMs: number,
  url: string
}
```

除此之外，还能直接确认：

- 工具名是 `WebFetch`
- `userFacingName() -> "Fetch"`
- `searchHint = "fetch and extract content from a URL"`
- `maxResultSizeChars = 100000`
- `shouldDefer = true`
- `isReadOnly() -> true`
- `isConcurrencySafe() -> true`

内嵌 prompt 文字也说明了它的设计定位：

- 先取网页内容
- HTML 会转成 markdown
- 再用一个“小而快”的模型按 prompt 提炼
- 如果有 MCP 提供的 web fetch 工具，优先用 MCP 版本
- 对 GitHub URL，优先建议用 `gh` 而不是这个工具

## 权限模型不是简单的“允许联网”

`WebFetch` 的权限决策比 `WebSearchTool` 更像本地网络工具：

1. 先检查 `mb8` 里的预批准 host/path 列表
2. 再把 URL 归一化成权限规则键：

```text
domain:<hostname>
```

3. 再按 `toolPermissionContext` 中的 `deny / ask / allow` 规则决策

所以它的权限结果可能是：

- 直接 `allow`
- 命中规则后 `deny`
- 未授权时 `ask`

这和 `WebSearchTool` 的 `passthrough` server-side permission gate 明显不同。

## URL 获取主链

`WebFetch.call(...)` 当前可还原成：

```text
WebFetch.call({ url, prompt }, context)
  -> co1(url, abortController)
    -> CY4(url) 校验
    -> 15 分钟缓存命中则直接返回
    -> 自动把 http 升到 https
    -> bY4(hostname) 做 domain preflight
    -> do1(url) 发 GET
    -> HTML 转 markdown
    -> 必要时落地保存二进制附件
  -> 如果需要，再 lo1(prompt, markdown, signal, isNonInteractiveSession, isPreapprovedUrl)
    -> yd7(...) 组装提示词
    -> UZ(...)
  -> 返回 WebFetchOutput
```

这条链里最重要的点是：

- 页面抓取在本地完成，不是把 URL 包成 `web_fetch` schema 交给服务端
- prompt 应用阶段也不是 `web_fetch_tool_result`，而是本地再做一次普通模型调用

## URL 校验、preflight 与重定向规则

当前已能直接确认的本地前置规则包括：

- URL 长度不能超过 `2000`
- 不能带用户名或密码
- hostname 至少要像正常域名
- HTTP 会自动升级到 HTTPS

domain preflight 也不是纯本地表，而是会请求：

```text
https://api.anthropic.com/api/web/domain_info?domain=<hostname>
```

它的返回只被本地解释成三种状态：

- `allowed`
- `blocked`
- `check_failed`

如果 preflight 失败，代码会抛出专门的本地错误，而不是继续盲抓。

domain preflight 还有两个容易漏掉的本地细节：

- 本地允许缓存 `allowed` 结果
  - cache 最大条目数 `128`
  - TTL `5 分钟`
- enterprise/managed settings 可通过：
  - `skipWebFetchPreflight`
  - 直接跳过这一步

真正抓取页面时，`do1(...)` 使用的是本地 HTTP GET：

- `maxRedirects: 0`
- `responseType: "arraybuffer"`
- `maxContentLength: 10485760`
- `timeout: 60000`
- `Accept: "text/markdown, text/html, */*"`

重定向也有限制：

- 仅允许同 protocol
- 同 port
- 目标 URL 不能带用户名密码
- hostname 只允许 `www.` 归一化后仍相同

如果跳到了不同 host，不会自动继续，而是返回一个特殊结果文本，要求模型重新发起一次新的 `WebFetch` 请求。

## 预批准表：不只是 host，还有少量 path 级白名单

`mb8` 不是纯 domain set，而是 **host/path 混合集合**。

当前能直接确认的特征是：

- 大量官方文档 host 被整站预批准
  - `platform.anthropic.com`
  - `code.claude.com`
  - `modelcontextprotocol.io`
  - `docs.python.org`
  - `developer.mozilla.org`
  - `docs.aws.amazon.com`
  - `kubernetes.io`
  - `git-scm.com`
  - 等
- 同时存在少量 path-scoped 项
  - `github.com/anthropics`
  - `vercel.com/docs`
  - `docs.netlify.com`
  - `devcenter.heroku.com/`

这意味着：

- `github.com` 整站并不在预批准列表里
- 当前可正证的 GitHub 预批准只覆盖 `github.com/anthropics...`
- 权限与“是否允许直接返回原始 markdown”都受这个预批准表影响

## 内容转换、本地缓存与二次摘要

`co1(...)` 收到响应后会：

- 把 HTML 交给 turndown 转成 markdown
- 非 HTML 则直接按 UTF-8 文本读取
- 某些二进制内容会额外落地保存，并在结果尾部追加保存路径说明
- 抓取结果会写入一个：
  - TTL 为 `15 分钟`
  - 总大小上限为 `50 MiB`
  的本地缓存

“某些二进制内容”现在也可以写得更精确：

- `SLq(contentType)` 为真时会尝试落地保存
- 它的规则不是白名单，而是“排除文本型”：
  - `text/*` -> 不落地
  - `application/json` / `*+json` -> 不落地
  - `application/xml` / `*+xml` -> 不落地
  - `application/javascript` -> 不落地
  - `application/x-www-form-urlencoded` -> 不落地
  - 其余 content-type -> 视为二进制，允许落地

因此当前更稳的理解是：

- `WebFetch` 并不是只对“图片/PDF/音频”落地
- 它对“非文本内容类型”采用统一的二进制兜底策略

随后 `call(...)` 会决定是否真的再调用模型：

```text
if (isPreapprovedUrl && contentType.includes("text/markdown") && content.length < 100000)
  result = raw markdown
else
  result = lo1(prompt, markdown, ...)
```

也就是说，预批准 URL 且内容已经是较短 markdown 时，工具会直接把原始 markdown 返回，而不是总是再做摘要。

这里还有两个实现级细节：

- 直接返回原文的条件是：
  - `Qo1(url) === true`
  - `contentType.includes("text/markdown")`
  - `content.length < 100000`
- cache key 当前仍是原始输入 URL
  - 即便抓取前会把 `http:` 升成 `https:`
  - 返回缓存时仍按最初传入的 URL 命中

## `lo1(...)` 不是 server-side `web_fetch`

`lo1(...)` 的真实行为是：

1. 把网页内容截断到 `100000` 字符
2. 用 `yd7(...)` 组装一个普通 user prompt：

```text
Web page content:
---
<markdown>
---

<user prompt>
```

3. 再调用：

```text
UZ({
  systemPrompt: [],
  userPrompt,
  options: {
    querySource: "web_fetch_apply",
    tools: [],
    model: mH(),
    thinkingConfig: { type: "disabled" }
  }
})
```

所以当前可直接确认：

- 这里的“提炼网页内容”走的是普通模型调用
- `querySource` 明确是 `"web_fetch_apply"`
- 传给模型的 `tools` 是空数组
- 没有发现 `extraToolSchemas:[{ type: "web_fetch_..." }]`

`yd7(...)` 还能再写实一点：

- prompt 骨架固定是：

```text
Web page content:
---
<content>
---

<user prompt>
```

- 若 URL 是预批准的：
  - 只要求“基于内容给出 concise response”
  - 允许包含相关细节、代码例子、文档摘录
- 若 URL 不是预批准的：
  - 会额外追加本地安全约束
  - 包括：
    - 单次引用上限 `125` 字符
    - 精确引用必须加引号
    - 不要评论自身 legality
    - 不要输出歌曲歌词

这意味着 `WebFetch` 的“二次摘要 prompt”本地侧其实有 **trusted / untrusted 两档模板**。

这就是为什么当前不能把本地 `WebFetch` 直接写成“server-side `web_fetch` 包装器”。

## transcript 里的 `tool_result` 也只是纯文本

`mapToolResultToToolResultBlockParam(...)` 的逻辑非常简单：

```text
tool_result.content = result
```

这意味着：

- `WebFetchOutput` 虽然是结构化对象
- 但回灌给 transcript 的只有 `result` 字符串
- 不像 `WebSearchTool` 那样还会拼装 `Links:` 和 source reminder 模板

## UI 层展示暴露出的事实

当前渲染层可直接确认：

- `renderToolUseMessage`
  - 默认展示 URL
  - verbose 模式下会把 prompt 一起带上
- `renderToolUseProgressMessage`
  - 只有一个静态状态：`Fetching…`
- `renderToolResultMessage`
  - 至少展示 `Received <size> (<code> <codeText>)`
  - verbose 模式下再附上正文结果

这和 `WebSearchTool` 那种从流事件里自己拼“Searching / Found N results”是不同的。

## 当前 bundle 里关于 `web_fetch` 的两层证据

### 第一层：真实运行时本地工具

当前真正有完整实现链的是：

- `WebFetch` / `Fetch`
- 本地校验 URL
- 本地做 domain preflight
- 本地 HTTP GET
- 本地 HTML -> markdown
- 本地再调用小模型提炼

这一层已经足以指导重写一个高相似本地 `Fetch` 工具。

### 第二层：generic streaming + 内嵌 SDK 文档

同时 bundle 里也确实有这些证据：

- `usage.server_tool_use.web_fetch_requests`
- UI 能识别 `web_fetch_tool_result`
- stream 累积逻辑能承接 `web_fetch_tool_result`
- 内嵌 SDK 文档出现 `WebFetchTool20260209`
- 示例文档出现 `{ type: "web_fetch_20260209", name: "web_fetch" }`

但这层证据目前只够说明：

1. 当前代码库知道 server-side `web_fetch` 这个概念
2. 通用流处理层已经准备好展示和统计它

还不够说明：

1. 当前 CLI 已经有一条本地包装器主动去调用 `web_fetch_20260209`
2. 当前用户态 `Fetch` 工具就是 server-side `web_fetch`

## 与 `WebSearchTool` 的区别

和 [07-web-search-tool.md](./07-web-search-tool.md) 对照时，差异可以压缩成三点：

1. `WebSearchTool` 已确认会构造 `web_search_20250305` schema 并通过 `extraToolSchemas` 注入 beta messages API；当前 `WebFetch` 没看到对应的 `web_fetch_202xxxx` 运行时构造器。
2. `WebSearchTool` 的用户可见进度来自 `server_tool_use / input_json_delta / web_search_tool_result`；当前 `WebFetch` 只有本地静态 `Fetching…`。
3. `WebSearchTool` 的核心结果是 server-side search hit 与 source link；当前 `WebFetch` 的核心结果是本地抓取内容经本地模型提炼后的文本。

## 一个重要的“版本漂移”现象

关于 `web_fetch`，当前 bundle 同样存在版本漂移：

### 运行时代码路径

当前可执行本地路径里，看到的是 `WebFetch` 本地工具，而不是显式的：

```text
web_fetch_20260209
```

### bundle 内嵌 SDK/示例文档

内嵌文本里却明确出现了：

```text
WebFetchTool20260209
web_fetch_20260209
```

这至少说明：

1. bundle 内嵌了一份更偏 SDK/平台文档视角的说明文本
2. 平台侧确实存在 server-side `web_fetch` 能力
3. 但当前 CLI 本地 `Fetch` 实现并没有直接等价为那条能力

### 继续追运行时代码后的更硬结论

这块现在还可以再收紧一刀。

当前本地 bundle 里：

- `web_fetch_20260209`
- `WebFetchTool20260209`

的字面命中，仍只落在：

- 内嵌 SDK 文档
- 示例文本
- 能力说明文本

而真正运行时可执行的 `WebFetch.call(...)` 主链里，当前只看到：

- `co1(...)`
- `lo1(...)`
- `UZ(... querySource="web_fetch_apply")`

没看到：

- `extraToolSchemas:[{ type: "web_fetch_20260209", ... }]`
- 类似 `WebSearchTool.call(...)` 的 server-side wrapper
- 由 feature flag 控制切换到 `web_fetch_20260209` 的本地分支

因此截至当前本地证据，更稳的判断应改成：

- **server-side `web_fetch_20260209` 概念确实存在于 bundle 文档层**
- **但当前 CLI 用户态 `Fetch` 工具仍是本地 fetch + 本地 apply**
- **至少在本地可见运行时代码里，还没找到把两者切换起来的 gating 分支**

## 当前已经能稳下来的结论

1. 当前 CLI 中真正可执行的 `WebFetch` 是本地工具，不是已确认的 server-side `web_fetch` wrapper。
2. 它会在本地做 URL 校验、domain preflight、HTTP 获取、HTML 转 markdown 和缓存。
3. 它随后通过 `UZ(... querySource="web_fetch_apply")` 用普通模型对页面内容做二次提炼。
4. `WebFetchOutput` 是本地工具契约；回灌 transcript 时只保留 `result` 字符串。
5. 通用 stream/UI/usage 统计层同时认识 `web_fetch_tool_result` / `web_fetch_requests`，说明 server-side `web_fetch` 概念在代码库中确实存在。
6. 但当前还没有找到一个与 `WebSearchTool` 同级、主动调用 `web_fetch_202xxxx` 的本地包装器。

## 当前还不能钉死的点

- `mb8` 的维护策略与未来是否热更新，当前仍不可见
- `WebFetch` 的本地抓取是否在某些环境下会被远端代理改写
- server-side `web_fetch_20260209` 是否已经在别的客户端版本中被真实启用
- 当前 CLI 是否存在尚未命中的 feature flag / 死代码分支，会把本地 `WebFetch` 切到 server-side `web_fetch`

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
