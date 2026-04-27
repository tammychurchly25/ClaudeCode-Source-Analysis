# 流事件处理与 Remote Transport

## 本页用途

- 用来梳理 `_I4(...)` 如何把底层 SDK streaming 事件映射成 CLI 内部事件。
- 用来单独记录 non-streaming fallback 与 `sdk-url / bridge / ingress` 这组 transport 路径。

## 相关文件

- [04-agent-loop-and-compaction.md](./04-agent-loop-and-compaction.md)
- [05-model-adapter-provider-and-auth.md](./05-model-adapter-provider-and-auth.md)
- [09-api-lifecycle-and-telemetry.md](./09-api-lifecycle-and-telemetry.md)
- [07-web-search-tool.md](./07-web-search-tool.md)
- [08-web-fetch-tool.md](./08-web-fetch-tool.md)
- [../03-ecosystem/02-remote-persistence-and-bridge.md](../03-ecosystem/02-remote-persistence-and-bridge.md)
- [../05-appendix/02-evidence-map.md](../05-appendix/02-evidence-map.md)

## 流事件处理与 Remote Transport

### `_I4(...)`：当前已知的真实主实现

当前已能把主链写成：

```text
Jk6
  -> hC1(...)
    -> _I4(...)
      -> VN8(() => _y(...), requestFn, retryContext)
        -> client.beta.messages.create({ stream: true }).withResponse()
```

其中 `_I4(...)` 已确认负责：

- 组装最终 request body
  - normalized messages
  - system prompt
  - tools / extraToolSchemas
  - toolChoice
  - betas
  - metadata
  - max_tokens
  - thinking
  - output_config / context_management / speed
- 管理 streaming 本地状态
  - `t`: 当前 partial message
  - `_6`: 当前 content block 数组
  - `l`: 已产出的 assistant 片段
  - `Z6`: usage 聚合
  - `P6`: stop_reason
- 在每个 SDK event 到来后：
  - 先更新本地状态
  - 再 `yield { type: "stream_event", event: J8, ... }`
- 在 `content_block_stop` 时：
  - 把单个 block 包成一条标准 `assistant` 消息
  - 立刻向上游产出
- 在 `message_delta` 时：
  - 更新最近一条 assistant 的 `usage / stop_reason`
  - 必要时补 refusal / max_output_tokens 相关系统错误

这说明上层看到的 `assistant` 不是只在 `message_stop` 后一次性出现，而是“按 block 收口的 assistant 增量片段”和原始 `stream_event` 并行存在。

### stream event 到内部事件的映射

这一层现在已经不再是纯推断：

- SDK 先把 SSE `message_start / message_delta / message_stop / content_block_start / content_block_delta / content_block_stop` 解析成事件对象
- SDK 内部再把 block 增量累积进当前 message：
  - `text_delta` -> 追加到 `text`
  - `citations_delta` -> 追加到 `citations`
  - `input_json_delta` -> 追加 partial JSON，并尝试填充 `tool_use / server_tool_use`
  - `thinking_delta` -> 追加到 `thinking`
  - `signature_delta` -> 写入 thinking signature
  - `compaction_delta` -> 追加 compaction 内容
- SDK 还会额外触发语义化事件：
  - `text`
  - `citation`
  - `inputJson`
  - `thinking`
  - `signature`
  - `compaction`

CLI 这一层 `_I4(...)` 自己也做了一次 UI/上层语义映射：

- `message_start` -> 记录 `ttftMs`
- `content_block_start(text)` -> UI 进入 `responding`
- `content_block_start(thinking)` -> UI 进入 `thinking`
- `content_block_start(tool_use / server_tool_use / compaction / mcp_tool_use / result 类 block)` -> UI 进入 `tool-input`
- `content_block_delta(text_delta)` -> 推进响应文本
- `content_block_delta(input_json_delta)` -> 推进 tool/server_tool_use 输入 JSON
- `message_delta(stop_reason)` -> 更新 usage 与 stop reason

因此“stream event -> internal event”的主映射链已经基本还原完成。

这里的“result 类 block”现在也能举到更具体：

- `web_search_tool_result`
- `web_fetch_tool_result`
- `tool_search_tool_result`
- `bash_code_execution_tool_result`
- `text_editor_code_execution_tool_result`
- `mcp_tool_result`

这说明 UI/stream adapter 对 tool 协议的识别不是“只懂本地 `tool_use`”，而是统一覆盖：

- 本地 tool
- server-side tool
- MCP tool
- 内建 schema discovery / tool search

### non-streaming fallback：触发条件与返回形态

fallback 现在已经可以写到相当细：

- stream 正常结束，但：
  - 没收到 `message_start`
  - 或收到了 `message_start`，但没有任何完成的 content block
- stream watchdog 超时
- stream 循环抛错，且未禁用 fallback
- streaming endpoint 创建阶段直接 404

进入 fallback 后走的是：

```text
AI4(...)
  -> VN8(() => _y(...), nonStreamingRequestFn, retryContext)
  -> client.beta.messages.create({ stream: false })
```

`AI4(...)` 的行为也已明确：

- 使用单独 timeout
- 仍复用 `VN8`，所以 fallback 期间仍可能产出 `system/api_error`
- 自己不会产出 `stream_event`
- 只在完成后返回最终 assistant message payload
- `_I4(...)` 再把这个 payload 包成标准 `assistant` 事件向外 `yield`

另外还有两个很具体的 fallback 分支：

- 若主 streaming 路径失败，fallback 会继承 `initialConsecutive529Errors: qk6(error) ? 1 : 0`
- 若 streaming 创建阶段直接 404，也会进入同一条 non-streaming fallback，而不是立刻 fatal

### 请求生命周期与 telemetry 三件套：`lj4 / Es1 / ij4`

这一组现在已经可以从“知道名字”推进到“闭环结构基本钉死”。

更稳的主链应写成：

```text
po_(...)
  -> 生成 queryTracking { chainId, depth }
  -> _I4(...)
    -> previousRequestId = NZz(messages)
    -> llmSpan = GYq(model, requestContext, normalizedMessages, fastMode)
    -> lj4(...)                         // request start
    -> VN8(() => _y(...), requestFn, retryContext)
         -> attemptStartTimes.push(Date.now())
         -> beta.messages.create(...)
         -> requestId = response.request_id
    -> stream success / non-streaming fallback / final error
    -> Es1(...)                         // request error
    -> ij4(...)                         // request success
```

这里最关键的结构点有三个：

- `lj4(...)` 只负责“请求发出前”的 telemetry，不带 `requestId`
- `Es1(...)` 只在最终失败路径调用，不会在每次可还原重试时都打一次
- `ij4(...)` 只在最终成功后调用一次，并把 usage/cost/duration 闭合掉

#### `lj4(...)`：真正的 request start telemetry

`lj4(...)` 在 `_I4(...)` 里、真正进入 `VN8(...)/beta.messages.create(...)` 之前触发。  
它发的是 `tengu_api_query`，当前已能确认的字段包括：

- `model`
- `messagesLength`
- `temperature`
- `betas`
- `permissionMode`
- `querySource`
- `queryTracking`
- `thinkingType`
- `effortValue`
- `fastMode`
- `previousRequestId`
- `provider`
- `buildAgeMins`
- `ANTHROPIC_BASE_URL / ANTHROPIC_MODEL / ANTHROPIC_SMALL_FAST_MODEL` 这类环境侧上下文

这说明：

- start telemetry 记录的是“这次准备怎么发”，不是“服务端实际返回了什么”
- 因为此时还没有 response，所以 `requestId` 不可能出现在 `lj4(...)`
- `permissionMode` 来自 `getToolPermissionContext()` 的解析结果，不是单纯抄 CLI 原始参数

#### `VN8(...)`：重试容器，也是 duration 语义的关键前提

`VN8(...)` 不是 telemetry helper，但它决定了后面 `Es1 / ij4` 的时间语义。

当前更稳的理解是：

- `o = Date.now()`：整次 request 生命周期起点，包含重试与 fallback
- `w6 = Date.now()`：每次 attempt 真正发请求前都会重置一次
- `K6.push(w6)`：记录每次 attempt 的开始时间，供 tracing / perfetto 使用
- `A6`：当前 attempt 编号，由 `VN8(...)` 回调写回
- `B6`：当前 attempt 的 `fastMode` 实际状态，也由 `VN8(...)` 回写

因此：

- `durationMs` 指向“最后一次 attempt 自己花了多久”
- `durationMsIncludingRetries` 指向“从第一次进入 request 到最终结束总共多久”
- `fastMode` 在 start 与 end/error 两侧不一定完全相同
  - `lj4(...)` 记录的是初始请求意图
  - `Es1(...)` / `ij4(...)` 记录的是 `VN8(...)` 调整后的实际状态

这点很重要，因为 `VN8(...)` 会在 fast mode 不可用、429/529、统一 overage 等条件下把 `fastMode` 关掉再继续尝试。

#### `Es1(...)`：最终失败路径的 telemetry

`Es1(...)` 的调用点已经能收束成两类：

- streaming 路径失败，且 fallback 也失败
- 非 fallback 的最终请求失败

也就是说：

- 普通可还原重试不会直接走 `Es1(...)`
- 只有“整次 request 最终失败”才会落这里

它至少会做四件事：

1. 归一化错误信息  
   - `Fo_(error)`：抽错误字符串
   - `fTq(error)`：归类 error type
   - `QT6(error)`：补连接类错误细节

2. 补 request 关联字段  
   - `requestId`
   - `clientRequestId`
   - `previousRequestId`
   - `queryTracking`
   - `querySource`

3. 识别 gateway  
   - `Ns1(headers)` 会根据 header 前缀识别 `litellm / helicone / portkey / cloudflare-ai-gateway`

4. 同时写三套 telemetry/tracing  
   - `d("tengu_api_error", ...)`
   - `GO("api_error", ...)`
   - `JS1(llmSpan, { success: false, ... })`

这里几个字段现在可以写得更精确：

- `durationMs`
  - 来自 `Date.now() - w6`
  - 表示最后一次 attempt 的耗时
- `durationMsIncludingRetries`
  - 来自 `Date.now() - o`
  - 表示整次 request 从第一次发起到最终失败的总耗时
- `requestId`
  - 优先取当前已保存的 `r`
  - 否则从异常对象的 `requestID / error.request_id` 回补
- `clientRequestId`
  - 来自 `x-client-request-id`
  - 主要用于把客户端日志和服务端日志对齐
- `didFallBackToNonStreaming`
  - 只表示这次 request 是否中途切到 non-streaming，不等于 fallback 一定成功

另外还能确认一个行为细节：

- 即使是用户 abort，最终也仍会先走 `Es1(...)`，然后再结束 request

#### `ij4(...)`：最终成功路径的 telemetry

`ij4(...)` 在 `_I4(...)` 的成功尾部触发。  
这里已经能确认它不是单纯“打个 success 日志”，而是 request completion 的总收口点。

它会做的事包括：

1. 计算成功态 duration  
   - `S = Date.now() - start`
   - `g = Date.now() - startIncludingRetries`

2. 回写全局 API 计时  
   - `Gd8(g, S)`
   - 第一个参数进入 `totalAPIDuration`
   - 第二个参数进入 `totalAPIDurationWithoutRetries`

3. 发 success telemetry  
   - `Qo_(...)` -> `tengu_api_success`
   - `GO("api_request", ...)`
   - `JS1(llmSpan, { success: true, ... })`

4. 汇总输出侧结构信息  
   - `textContentLength`
   - `thinkingContentLength`
   - `toolUseContentLengths`

这意味着 `ij4(...)` 实际上同时闭合了：

- request 成功日志
- usage/cost 统计
- perfetto / llm span 完成事件
- 全局 session 级 API duration 聚合

#### 关键字段语义：当前可直接写死的版本

- `requestId`
  - 成功 streaming 路径取 `withResponse().request_id`
  - 错误路径可从异常对象回补
  - 若 streaming 建连阶段先 404，再走 non-streaming fallback 成功，最终 success telemetry 里的 `requestId` 可能仍为 `null`

- `clientRequestId`
  - 只在 first-party 路径显式生成
  - 通过 `x-client-request-id` 发出
  - 当前主要在错误路径上报和日志提示里使用

- `durationMs`
  - 只看最终 attempt
  - 不包含之前失败重试的等待与重建成本

- `durationMsIncludingRetries`
  - 覆盖整次 request 生命周期
  - 包含重试等待、client 重建、stream -> non-streaming fallback 等前置损耗

- `ttftMs`
  - streaming 路径在第一次 `message_start` 时写入
  - non-streaming fallback 成功时通常没有这个值

- `costUSD`
  - 由 `Le(model, usage)` 定价，再经 `rv6(...)` 计入全局 usage
  - 会递归纳入 advisor 子 usage

- `queryTracking`
  - `po_(...)` 为每次 query 生成 `{ chainId, depth }`
  - 同一条链上的后续 query 复用 `chainId`
  - 深度随嵌套/继续调用递增

- `previousRequestId`
  - 由 `NZz(messages)` 从当前 transcript 里向后找最近一条带 `requestId` 的 assistant message
  - 用来把相邻 request 串成链，而不是只靠 query depth

- `permissionMode`
  - 来自当次 `getToolPermissionContext()` 的结果
  - 因此它记录的是 request 发出时的实际权限模式

- `fastMode`
  - start telemetry 记录初始意图
  - success/error telemetry 记录重试器修正后的实际状态

- `requestSetupMs`
  - 来自 `w6 - o`
  - 更准确地说是“从整次 request 进入 `_I4(...)` 到最终成功 attempt 发出前”的累计 setup 时间
  - 不是纯粹的序列化耗时

- `attemptStartTimes`
  - 每次 attempt 发出前 push 一次时间戳
  - 当前主要用于 `JS1(...)` 对 perfetto/tracing 的 retry 片段还原

这里还要补一个分类边界：

- `model`、`effortLevel`、`fastMode`、provider 切换、permission/sandbox 相关模式，都会直接改写这些 telemetry 字段的值
- 但它们不应被误写成“telemetry 专用设置”
- 更准确地说，它们属于业务运行态设置，而 telemetry 只是把这些运行态选择记录下来

#### 当前还保留的边界

这一段虽然已经能闭环，但仍有两个边界要保留：

- `promptCategory` 当前已能确认只出现在 `Es1(...)` 形参与 spread 中，本地 bundle 未发现任何生产调用方实际传值，因此更像未接线保留槽位
- `gateway` 识别当前只看到 `Ns1(...) -> Uo_` 这条 header-prefix 分类链；本地 bundle 未发现第二套 gateway classifier，但也不能据此证明服务端侧不存在更多分类

### `stream_request_start` 的真实来源

这个点也已经钉死：

- `stream_request_start` 不是 `Jk6` 发的
- 它来自主循环 `po_(...)`
- 每轮 `callModel` 前，`po_` 都会先 `yield { type: "stream_request_start" }`

所以它属于“主循环 UI 协议事件”，不属于 provider/SDK event。

### 本模块剩余状态

到这一步，`provider / gateway / transport` 主模块已经没有阻碍重写的未知点。

还没逐项归档完的，只剩一些外围细枝末节：

- telemetry 初始化链、startup telemetry 与非必要流量 gating 仍未在本页展开
- 这些内容现已转移到 [09-api-lifecycle-and-telemetry.md](./09-api-lifecycle-and-telemetry.md)
- 少量 cache/header 的边缘行为仍未完全枚举
- 少量显然未启用的 dead-code/feature-flag stub（例如恒为 `false/null` 的 override helper）没有继续浪费时间做无效深挖

这些都不再构成 `provider / gateway / transport` 的结构性未知点。

### 远程 transport：`sdk-url` / bridge / ingress

这一块现在也能与 provider 层明确分离：

- `sdk-url` 只在 `--print + --input-format=stream-json + --output-format=stream-json` 下启用
- structured input source 会从本地实现切到远端 transport
- transport factory `Ro4(url, headers, sessionId, refreshHeaders)` 至少分三路：
  - `CLAUDE_CODE_USE_CCR_V2=1` -> `Nj6`
  - `ws/wss + CLAUDE_CODE_POST_FOR_SESSION_INGRESS_V2=1` -> `L18`
  - `ws/wss` 默认 -> `y18`

其中：

- `y18` 是纯 WebSocket transport
  - 支持 `X-Last-Request-Id`
  - 支持重放 buffered messages
  - 支持 4003 后 refresh token 再重连
  - 自带 ping/pong、keep_alive、sleep 检测
- `L18` 是 hybrid ingress transport
  - 下行读取 `client_event`
  - 用 `Last-Event-ID` / `from_sequence_num` 续传
  - 上行通过独立 POST 写入
- `Nj6` 是 CCR v2 路径
  - 会把 `ws/wss` URL 改写为 `http/https` 的 `/worker/events/stream`

这说明：

- provider/gateway 与 remote ingress transport 是两套正交层
- `sdk-url` 不等于“直接把模型请求发到另一个 provider”
- 它更像“把 CLI 主循环绑定到远端 session ingress”

#### transport factory 的精确分流

`Ro4(...)` 现在其实可以写到 URL 改写级别：

- `CLAUDE_CODE_USE_CCR_V2=1`
  - 不再保留 `ws/wss`
  - 会把：
    - `wss:` -> `https:`
    - `ws:` -> `http:`
  - 再把 pathname 改成：
    - `.../worker/events/stream`
  - 最终产物是 `new Nj6(url, headers, sessionId, refreshHeaders)`
- 否则若协议仍是 `ws/wss`
  - `CLAUDE_CODE_POST_FOR_SESSION_INGRESS_V2=1` -> `new L18(...)`
  - 否则 -> `new y18(...)`

因此 CCR v2 不是“在 WebSocket 上切协议”，而是一开始就切成 **HTTP/SSE + worker subpaths**。

#### transport 协议粒度还能再收紧

这一层现在已经不只是“像 event pipe”，而是可以直接写到收发对象形态：

- `sU8` 会把 transport 包成一个 `PassThrough` 输入流
  - `transport.setOnData(...)` 收到的每段字符串，会直接写回本地 `inputStream`
  - 因此 headless `--print --sdk-url --input-format=stream-json` 看到的仍是同一套 `stream-json` 事件流
- `y18.write(A)`
  - 直接把单个事件对象做 `JSON.stringify(A) + "\\n"` 后发到 WebSocket
  - 若对象自带 `uuid`，只会顺手进入本地 replay buffer
  - 没看到任何把事件再提升成 `messages/system/userContext/systemContext` 级请求对象的包装
- `L18.write(A)`
  - 只会对 `stream_event` 做短暂缓冲
  - 真正上行时发的是 `POST { events: [...] }`
  - 也就是把同一批本地事件对象整体送进 ingress，而不是重新拼 prompt
- `Nj6`
  - 下行 SSE 只接受 `event: client_event`
  - 解析后只取 `payload`
  - 若 `payload.type` 存在，就原样 `JSON.stringify(payload) + "\\n"` 回灌给本地
  - 上行 POST 也只是把当前事件对象原样发给 worker/session 侧 endpoint
- `R18`
  - CCR v2 上行拆成：
    - `/worker/events`
    - `/worker/internal-events`
    - `/worker/events/delivery`
    - `/worker`
  - 承载的是 client event、internal event、delivery ack、worker state/metadata
  - 不是 prompt assembly 请求体

这说明：

- `sdk-url / remote-control / CCR v2` 这组远端接入，当前本地可见职责都是 **session event plane**
- 这里传的是：
  - `stream-json` 事件
  - control request / response
  - keep_alive
  - worker metadata / delivery 状态
- 这里没看到传：
  - 重新组装后的 `messages/system`
  - 新的 `userContext/systemContext`
  - 额外 compat / verification prompt 片段

#### 三个 transport 的本地状态机还能继续写实

##### `y18`：WebSocket close/reconnect 规则

- 默认 autoReconnect 打开
- permanent close code 集合当前已能直接写死：
  - `1002`
  - `4001`
  - `4003`
- 但 `4003` 有一个明确例外：
  - 若提供了 `refreshHeaders()`
  - 且刷新后的 `Authorization` 与旧值不同
  - 就不会当成 permanent close
  - 而是更新 headers 后继续 reconnect
- 建连时若已有 `lastSentId`
  - 会补 `X-Last-Request-Id`
  - open 后按对端确认的 last id 重放本地 buffered messages
- 连接期间还会主动发两类保活：
  - websocket ping/pong
  - 周期性 `keep_alive` data frame

##### `L18`：hybrid ingress 的上行批处理参数

- `stream_event` 不会立刻 POST
  - 先放进 `streamEventBuffer`
  - `100ms` 后批量 flush
- uploader 参数当前可直接写成：
  - `maxBatchSize: 500`
  - `maxQueueSize: 100000`
  - `baseDelayMs: 500`
  - `maxDelayMs: 8000`
  - `jitterMs: 1000`
- 真正 POST body 固定是：

```json
{ "events": [ ... ] }
```

- 单次 POST timeout 是 `15000ms`
- `4xx` 且不是 `429`
  - 视为 permanent client error
  - 丢弃当前 batch
- `429`、`5xx` 与网络错误
  - 视为 retryable
  - 交给 uploader 指数退避
- `close()` 时会：
  - 先尝试 flush
  - 最多等 `3000ms`
  - 再真正关闭 uploader 和基类 transport

##### `L18` 下行续传字段

Hybrid/SSE 下行建连时，当前已能直接看到：

- 若 `lastSequenceNum > 0`
  - query string 补 `from_sequence_num=<n>`
  - header 也补 `Last-Event-ID: <n>`
- 固定补：
  - `Accept: text/event-stream`
  - `anthropic-version: 2023-06-01`
- 若 auth 走 `Cookie`
  - 会显式删除 `Authorization`

这说明它不是只靠本地 buffer，而是显式支持 **服务端 sequence-based replay**。

##### `Nj6` / `R18`：CCR v2 的 worker 面

- `R18` 初始化时会建立四个独立 uploader：
  - `eventUploader` -> `POST /worker/events`
  - `internalEventUploader` -> `POST /worker/internal-events`
  - `deliveryUploader` -> `POST /worker/events/delivery`
  - `workerState` -> `PUT /worker`
- 这些请求都固定带：
  - `worker_epoch`
  - `anthropic-version: 2023-06-01`
- 初始化 `PUT /worker` 的首个状态是：
  - `worker_status: "idle"`
  - `external_metadata.pending_action: null`
- `409` 会被解释成 worker epoch mismatch
- `401/403` 连续达到阈值，会直接走 epoch mismatch / 退出，而不是无限重试

因此若服务端还会额外追加 `verification / context / systemContext`，更像是：

- 模型 message API 收到本地 payload 之后的黑箱后处理
- 或 bundle 外远端 runtime 的附加能力

而不是当前本地 `sdk-url / remote-control` transport 自己又做了一次 prompt 重写。

### 本地 request build 的最后边界

围绕“远端/服务端是否额外注入 context / compat / verification”，当前本地 bundle 还能再收紧一步。

#### 本地可见的 payload producer 只剩 3 处

当前直接能看到真正把 prompt/request body 交给 `beta.messages.create(...)` 的本地入口只有：

- `_I4(...)`
  - streaming 主路径
- `AI4(...)`
  - non-streaming fallback
- `hN(...)`
  - 少量 side query helper

其中主路径 `_I4(...)` 的最后落体已经是：

```text
payload = {
  model,
  messages: LZz(k, ...),
  system: hZz(q, ...),
  tools,
  tool_choice,
  betas,
  metadata,
  max_tokens,
  thinking,
  output_config,
  context_management,
  speed
}
-> client.beta.messages.create(payload, ...)
```

#### builder callback `i(v6)` 的条件字段表

`_I4(...)` 里的真正 payload builder 不是平铺常量，而是一组条件装配：

- `model`
  - 最后会过 `af(...)` 映射
- `messages`
  - `LZz(k, promptCachingEnabled, querySource, cacheEditMode, cacheBoundary, cacheReads, skipCacheWrite)`
- `system`
  - `hZz(q, enablePromptCaching, { skipGlobalCacheForSystemPrompt, querySource })`
- `tools`
  - 本地工具 schema + `extraToolSchemas`
  - `advisor` 命中时还会额外 push `advisor_20260301`
- `betas`
  - 来自模型默认 beta、tool-search beta、prompt cache beta、fast beta、auto-mode beta、cache-edit beta 等组合
- `metadata`
  - 固定来自 `n16()`
- `max_tokens`
  - 优先级：
    - `v6.maxTokensOverride`
    - `Y.maxOutputTokensOverride`
    - `In6(model)` 默认值
- `thinking`
  - 仅当 thinking 未禁用且模型支持时才存在
  - adaptive thinking 与 budget-based thinking是两条分支
- `temperature`
  - 只有 **不启用 thinking** 时才会显式写入
- `context_management`
  - 只有模型/能力组合满足，且 beta 集合包含对应开关时才会写出
- `output_config`
  - 由 effort / taskBudget / outputFormat 等组合生成
- `speed`
  - 只有 fast mode 实际生效时才会写成 `"fast"`

因此当前更稳的说法不是“payload 有这些字段”，而是：

- `_I4(...)` 先统一求出一份 **条件性 builder**
- streaming 与 non-streaming fallback 都复用同一份 builder
- fallback 只是把 `stream: true` 切成 `stream: false`

`AI4(...)` 自己并不重新拼 prompt。  
它接收的是 `_I4(...)` 里传入的 builder callback `i(v6)`，也就是同一套 `messages/system/tools` 构造结果，只是在最后把：

- `stream: true`

切成：

- `stream: false`

因此 non-streaming fallback 不是第二套 prompt assembly。

#### `sdk-url / remote-control` 本地可见代码里只换 transport，不改 prompt

bridge/session manager 侧做的事情当前可直接写成：

- 用 `--print --sdk-url --input-format=stream-json --output-format=stream-json` 启动子进程
- 注入 `CLAUDE_CODE_SESSION_ACCESS_TOKEN`
- 注入 `CLAUDE_CODE_POST_FOR_SESSION_INGRESS_V2=1`
- 视情况再加 `CLAUDE_CODE_USE_CCR_V2=1`

然后只做：

- stream-json I/O
- control request / permission request 转发
- activity / transcript 采集

当前没看到它在子进程外再修改：

- `messages`
- `system`
- `userContext`
- `systemContext`
- compat / verification 指令

#### `verification_agent` 在本地仍只有识别位，没有发起点

当前本地 bundle 中：

- `verification_agent` 只出现在 `_I4(...)` 的 `isAgenticQuery` 判定里
- 没有发现任何本地 `querySource: "verification_agent"` 发起点
- 可见 subagent 的正常 querySource 生成规则是：
  - built-in -> `agent:builtin:<agentType>`
  - custom -> `agent:custom`
  - 因此即便本地真的存在一个 `subagent_type: "verification"`，按当前可见通路它也不该自然落成 `verification_agent`
- 另外还有一个重要矛盾：
  - bundle 文本里会提示“spawn verification agent (subagent_type=\"verification\")”
  - 也残留了一整段 verifier system prompt
  - 但当前可见 built-in agent 注册表没有把 `verification` 注册进去

因此更稳的本地结论是：

- 本地 bundle 没看到 `_I4/AI4` 之后还有第二次 request-level context 注入
- `sdk-url / remote-control` 在本地可见范围内只是 transport/ingress 层
- `verification_agent` 更像 **服务端/动态注入/未完全接线残留**，而不是当前本地 bundle 可直接跑通的一条 querySource
- 若 `context / compat / verification` 还有额外注入，更可能发生在服务端黑箱，不在本地 bundle 内

---

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
