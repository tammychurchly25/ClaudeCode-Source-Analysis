# 模型适配层、Provider 选择与鉴权

## 本页用途

- 用来说明 `Jk6 / hC1 / VN8` 这一层如何向主循环暴露统一的模型调用门面。
- 用来集中梳理 provider 工厂、first-party / 3P 分支以及 auth token / apiKey 的来源优先级。

## 相关文件

- [04-agent-loop-and-compaction.md](./04-agent-loop-and-compaction.md)
- [07-web-search-tool.md](./07-web-search-tool.md)
- [06-stream-processing-and-remote-transport.md](./06-stream-processing-and-remote-transport.md)
- [10-control-plane-api-and-auxiliary-services.md](./10-control-plane-api-and-auxiliary-services.md)
- [../04-rewrite/02-open-questions-and-judgment.md](../04-rewrite/02-open-questions-and-judgment.md)
- [../05-appendix/01-glossary.md](../05-appendix/01-glossary.md)

## 模型调用适配层：Jk6 / H.callModel / VN8

### 依赖注入点

`mj4()` 提供：

- `callModel: Jk6`
- `microcompact`
- `autocompact`
- `uuid`

说明 `CC` 本身不关心底层 API，只关心 `callModel` 这个高层流接口。

### `Jk6` 的职责

高可信职责：

- 将 `messages + systemPrompt + tools + options` 组装成一次高层模型请求
- 统一暴露 `stream_event / assistant / system(api_retry) / attachment` 事件流
- 负责把底层 SDK streaming 归一成 CC 所需要的事件模型

但从可读 bundle 看，`Jk6` 本身其实很薄：

- `Jk6(...)` 只是 `yield *hC1(..., () => _I4(...))`
- `Xo(...)` 则是“跑完整条流，只取最后一个 assistant”的便捷包装

因此真正的 provider 调用、streaming 状态累积、fallback 与错误收敛，核心都在 `_I4(...)`，不是 `Jk6` 这个入口名本身。

### `hC1(...)`：测试夹具/VCR 包装层，不是运行时 replay

`hC1(A, q)` 的行为现在也已可见：

- 若 `LC1()` 未开启：直接 `yield *q()`
- 若开启：先尝试 `KT8(A, ...)` 取 replay
- 有 replay 则直接 `yield *replay`
- 否则收集真实结果数组，结束后再原样 `yield *K`

继续往下追后，这条链已经可以从“像 replay”改写成更精确的结论：

- `LC1()` 在发行版里是硬编码 `return false`
- `KT8(...)` 不是 session/transcript replay，而是测试期的 API fixture 读写器
- fixture 根目录为 `process.env.CLAUDE_CODE_TEST_FIXTURES_ROOT ?? cwd`
- 文件名来自“过滤掉 meta user message 后的输入消息内容”哈希：
  - 先做 message content 归一化
  - 再对每段内容取 sha1 前 6 位
  - 最终落成 `fixtures/<hash>-<hash>-....json`
- 若 fixture 存在：读取 `output`，做占位符反向展开后直接返回
- 若 fixture 不存在：
  - CI 且未开启 `VCR_RECORD` -> 直接报错，要求录制 fixture
  - 否则执行真实请求，并把 `input/output` 写回 fixture

因此 `hC1 / KT8 / LC1` 并不是生产运行时的 replay/还原机制，而是测试夹具层；对真实 provider/gateway/transport 主路径可以视为硬旁路。

### `VN8`：重试/降级包装层

已确认处理：

- 401/token 失效 -> 刷新/重建 client
- 429/overloaded -> retry-after / 退避 / 关闭 fastMode 再试
- 连续 529 + fallbackModel -> 抛 `Lw6(original, fallback)`
- context limit overflow -> 下调 `maxTokensOverride` 后重试

现在还能更精确地描述：

- `VN8(A, q, K)` 是 async generator
- `A` 通常就是 lazy client factory：`() => _y(...)`
- `q` 是“拿到 client 后真正发请求”的回调
- `K` 是 retry context，至少含 `model / thinkingConfig / signal / fallbackModel`

其内部逻辑已可归纳为：

- 仅在以下情况重建 client：
  - 首次请求
  - 401
  - OAuth token revoked
  - Bedrock auth 错误
  - Vertex credential refresh 错误
- fastMode 下遇到 429/529，不会立刻失败，而是优先关闭 fastMode 再试
- 命中 `input length and max_tokens exceed context limit` 时，会解析报错字符串并写回 `z.maxTokensOverride`
- 普通 retry 前会 `yield rx1(...)`

还能再收紧成一张更接近实现的矩阵：

- 最大重试次数来源：
  - `K.maxRetries`
  - 否则 `CLAUDE_CODE_MAX_RETRIES`
  - 再否则默认 `10`
- fast mode 特殊分支：
  - 只有 `u4()` 路径才会真的读写 `z.fastMode`
  - 若 `429/529` 且带 `anthropic-ratelimit-unified-overage-disabled-reason`
    - 直接记录原因
    - 关闭 `fastMode`
    - 继续下一次 attempt
  - 若 `retry-after` 很短
    - 先按 header 等待
    - 不一定立刻关 `fastMode`
  - 若等待窗口较长
    - 记录 unified reset / overage telemetry
    - 关闭 `fastMode`
    - 再继续尝试
  - 若命中 `"Fast mode is not enabled"`
    - 也会直接关 `fastMode`
- `529 / overloaded` 不是无限重试：
  - 会累计连续 `529`
  - `initialConsecutive529Errors` 还能从 streaming fallback 继承
  - 达到阈值 `3` 后：
    - 有 `fallbackModel` -> 抛 `Lw6(original, fallback)`
    - 无 fallback -> 进入统一 overloaded 失败路径
- retryable 判定当前至少包括：
  - 网络层 `m0`
  - `408`
  - `409`
  - `429`
  - `401`
  - `5xx`
  - `x-should-retry: true`
  - remote 模式下的 `401/403`
  - overloaded / context overflow 解析成功
- context overflow 处理不是简单减一点：
  - 从错误字符串解析 `inputTokens / maxTokens / contextLimit`
  - 目标值近似 `contextLimit - inputTokens - 1000`
  - 再与 `thinking budget + 1`、`FLOOR_OUTPUT_TOKENS` 取上界
  - 回写为 `z.maxTokensOverride`
- 持续性重试与普通重试不是同一条时钟：
  - 普通重试按 attempt 编号指数退避
  - `NN8() && MNq(error)` 命中的持久性重试单独累计次数
  - 等待期间会反复 `yield rx1(...)`

`rx1(...)` 现在也已经能确认形状：

```ts
{
  type: "system",
  subtype: "api_error",
  error,
  retryInMs,
  retryAttempt,
  maxRetries,
}
```

因此 `VN8` 不只是“内部重试器”，也承担了向上游报告 API retry 的职责。

### 底层来源

这层现在已经可以从“Anthropic 风格封装”收敛到更具体的判断：

- 发行版内嵌的是改名后的 `Anthropic` SDK，而不是直接依赖外部 Anthropic SDK。
- first-party 默认 `baseURL` 是 `https://api.anthropic.com`。
- 仍沿用 Anthropic 风格环境变量：
  - `ANTHROPIC_BASE_URL`
  - `ANTHROPIC_API_KEY`
  - `ANTHROPIC_AUTH_TOKEN`
- SDK 认证头支持两路：
  - `X-Api-Key`
  - `Authorization: Bearer ...`
- 通用版本头为 `anthropic-version: 2023-06-01`。
- beta 特性通过 `anthropic-beta` 叠加，而不是 Anthropic 原始的 beta 头名字。

### provider 工厂：`_y(...)`

bundle 中已经能确认存在统一的 client factory：

```ts
async function _y({
  apiKey,
  maxRetries,
  model,
  fetchOverride,
  source,
})
```

其职责是：

- 组装 first-party / 3P provider 通用 `defaultHeaders`
- 注入 request wrapper、timeout、retry、fetchOptions
- 按环境变量选择最终 provider client

通用 headers 至少包括：

- `x-app: cli`
- `User-Agent`
- `x-claude-remote-container-id`
- `x-claude-remote-session-id`
- `x-client-app`
- `x-anthropic-additional-protection`（条件开启）

### provider 选择分支

已确认分为 4 条明确分支：

- `firstParty` -> `new Eb(...)`
- `Bedrock` -> `new AnthropicBedrock(...)`
- `Foundry` -> `new AnthropicFoundry(...)`
- `Vertex` -> `new AnthropicVertex(...)`

切换位不是运行时自动探测，而是环境变量控制：

- `CLAUDE_CODE_USE_BEDROCK`
- `CLAUDE_CODE_USE_VERTEX`
- `CLAUDE_CODE_USE_FOUNDRY`

因此重写时应把“provider 选择”做成显式策略层，而不是把 first-party 与 3P provider 混在一个 client 内部用 if/else 打补丁。

### first-party / subscription / token 语义

从 auth 判定逻辑可确认：

- first-party 才会真正涉及 Claude account subscription / OAuth token 语义。
- 如果启用了 Bedrock / Vertex / Foundry，则 subscription token 逻辑基本被短路。
- first-party 模式下同时存在多种 auth 来源：
  - `CLAUDE_CODE_OAUTH_TOKEN`
  - `ANTHROPIC_AUTH_TOKEN`
  - `ANTHROPIC_API_KEY`
- `apiKeyHelper`
- bundle 中还专门有 auth conflict 警告，说明原版明确区分“Claude 账号订阅 token”与“外部 API key / auth token”。

现在还能进一步把判定链写清：

- `QH()` 近似等于“是否启用 Anthropic account auth 语义”
- 只要满足以下任一条件，`QH()` 就会返回 `false`：
  - 启用了 Bedrock / Vertex / Foundry
  - 存在 `ANTHROPIC_AUTH_TOKEN`
  - 配置了 `apiKeyHelper`
  - 存在 `CLAUDE_CODE_API_KEY_FILE_DESCRIPTOR`
  - 能解析到显式 API key 来源（`ANTHROPIC_API_KEY / apiKeyHelper / login managed key`）
  - 处于 `--bare / CLAUDE_CODE_SIMPLE`

也就是说，subscription / OAuth 模式并不是“只要有 OAuth token 就算”，而是一个更严格的“没有落到外部 key/token 模式、也没有切到 3P provider”才成立的分支。

### `dA()` / `QH()` / `n_6()`：三种 first-party 身份态

这三个判断现在可以分开理解：

- `QH()`：是否启用 Anthropic account auth 语义
- `dA()`：是否是 claude.ai subscriber 路径
  - 本质上是 `QH() && oA()?.scopes` 包含 `user:inference`
- `n_6()`：是否是 first-party API customer
  - 本质上是“不是 3P provider，且不是 `dA()` subscriber”

因此 first-party 至少有两种非 3P 身份：

- subscriber：走 OAuth account / subscription 语义
- API customer：走 first-party API key / auth token 语义

### `_y(...)` 最终如何选 `apiKey` 与 `authToken`

在 first-party 分支里，`_y(...)` 最终传给 `new Eb(...)` 的关键字段已经能写死：

```ts
{
  apiKey: dA() ? null : (passedApiKey || Hv()),
  authToken: dA() ? oA()?.accessToken : undefined,
}
```

这说明：

- subscriber 模式不会再给 SDK 传 `apiKey`
- subscriber 模式直接走 `authToken = OAuth access token`
- API customer 模式则走 `apiKey`

另外，`_y(...)` 在构造 client 前还有一条额外路径：

```ts
if (!dA()) await k19(defaultHeaders, lA())
```

这里的 `k19(...)` 会把：

- `ANTHROPIC_AUTH_TOKEN`
- 或 `apiKeyHelper` 返回值

写入 `Authorization: Bearer ...`。

也就是说，API customer 模式下可能同时存在：

- SDK 级 `apiKey`
- header 级 `Authorization`

这也是为什么原版会专门区分“subscription token”和“外部 auth token / API key”。

### `QR()`：auth token 来源优先级

从 `QR()` 可以还原出一条比较明确的优先级链：

1. `--bare / CLAUDE_CODE_SIMPLE` 下，只承认 `apiKeyHelper`
2. `ANTHROPIC_AUTH_TOKEN`（仅非 remote/desktop）
3. `CLAUDE_CODE_OAUTH_TOKEN`
4. `CLAUDE_CODE_OAUTH_TOKEN_FILE_DESCRIPTOR` / CCR OAuth token file
5. `apiKeyHelper`（仅非 remote/desktop）
6. 本地缓存的 claude.ai OAuth token，且 scope 含 `user:inference`
7. 否则 `none`

### `C$()` / `Hv()`：API key 来源优先级

API key 侧也已经能还原出主优先级：

1. `ANTHROPIC_API_KEY`
2. API key file descriptor
3. `apiKeyHelper`
4. `/login managed key`
  - macOS keychain 或本地 config 中保存的 primaryApiKey
5. 否则 `none`

其中：

- `Hv()` 只是 `C$().key`
- `aG1()` 是“是否存在任意 Anthropic API key auth 来源”的快速判定

### 一个容易误读的点：`lA()` 不是 token getter

`_y()` 里有一段：

```ts
await k19(defaultHeaders, lA())
```

这里的 `lA()` 不是在“取 token”，而只是：

```ts
function lA() {
  return !V8.isInteractive
}
```

也就是“当前是否非交互”。它只是被传给 `apiKeyHelper` 路径，用来影响 trust / 执行策略，不能把它理解成 auth source。

### `tp / UP8 / HZ1 / MZ1 / jD`：account/profile 元数据链已经闭环

这一组函数现在也不再需要保留为未知点，但它们属于 first-party auth / control-plane 元数据链，不属于模型 inference data-plane 本身：

- `tp(accessToken)`
  - 原始接口：`GET ${BASE_API_URL}/api/oauth/profile`
  - 认证：`Authorization: Bearer <oauth access token>`
  - 返回 account / organization 原始 profile
- `UP8(accessToken)`
  - 是 `tp(...)` 的语义包装
  - 把 `organization_type` 映射成 `subscriptionType`
    - `claude_max -> max`
    - `claude_pro -> pro`
    - `claude_enterprise -> enterprise`
    - `claude_team -> team`
  - 额外抽取：
    - `rateLimitTier`
    - `hasExtraUsageEnabled`
    - `billingType`
    - `displayName`
    - `accountCreatedAt`
    - `subscriptionCreatedAt`
  - 并保留 `rawProfile`
- `HZ1(accessToken)`
  - 调 `GET ${ROLES_URL}`
  - 把结果写入本地 `oauthAccount`
    - `organizationRole`
    - `workspaceRole`
    - `organizationName`
- `MZ1()`
  - 启动期补资料入口
  - 先读取环境变量 `CLAUDE_CODE_ACCOUNT_UUID / CLAUDE_CODE_USER_EMAIL / CLAUDE_CODE_ORGANIZATION_UUID` 作为最小 account seed
  - 然后 `await rz()` 刷新 token（若需要）
  - 仅当满足以下条件才继续拉 profile：
    - `dA()` 为真，即 subscriber 路径
    - token 具有 `user:profile` scope
    - 本地 `oauthAccount` 尚未具备 `billingType / accountCreatedAt / subscriptionCreatedAt`
  - 满足时才调用 `tp(accessToken)` 并覆盖/补齐本地 `oauthAccount`
- `jD()`
  - 统一“拿 organization UUID”的 helper
  - 先读 `oauthAccount.organizationUuid`
  - 若没有且 token 具备 `user:profile`，则退回 `tp(accessToken)` 取 `organization.uuid`

### 精确时机：启动期 / 登录期 / 刷新期

这条元数据链的触发时机也已经明确：

- 启动期
  - 初始化主流程里直接调用 `MZ1()`
  - 调用点位于全局 init 早期，紧跟 1P event logging 初始化之后
  - 这里是“异步触发但不阻塞后续 init”的 best-effort profile backfill
- 登录期
  - `fO6(...)` 是安装 OAuth token 的主入口
  - 它会先写 `oauthAccount` 的基础资料，然后 `y06(...)` 持久化 token
  - 随后总是尝试 `HZ1(accessToken)` 拉取 role/name
  - 若是 subscriber（`xR(scopes)`），再调用 `n14()` 拉 `first_token_date`
  - 若不是 subscriber，则改走 `JZ1(accessToken)` 为 first-party API customer 创建 API key
- 刷新期
  - `WU6(refreshToken)` 在 refresh 成功后，只在“本地还缺 profile 衍生字段”时才补调用 `UP8(newAccessToken)`
  - 它不会重新调用 `HZ1(...)`
  - 因此 role/name 主要在登录安装 token 时更新，而不是每次 refresh 时刷新

### 对 gateway / transport 的实际意义

这条链对主模块的意义现在也已明确：

- inference data-plane
  - `_y(...)` / `VN8(...)` / `_I4(...)` 主要只依赖 `oA()`、`dA()`、`Hv()` 这类“已选定凭据”
- control-plane / web sessions / remote
  - `UM()` 要求必须有 claude.ai OAuth access token，而不是 API key
  - `UM()` 还会调用 `jD()` 取 organization UUID
  - `fetchSession / list sessions / environment providers / remote create` 都走这条 `accessToken + orgUUID` 路径

所以 `tp / UP8 / HZ1 / MZ1 / jD` 的职责不是“决定如何发模型请求”，而是“补齐 first-party account metadata，并支撑需要 org UUID 的 control-plane API”。

### first-party API 面

当前可确认的 first-party 数据面接口至少有：

- `/v1/messages?beta=true`
- `/v1/messages/count_tokens?beta=true`
- `/v1/models?beta=true`
- `/v1/files`

同时还存在控制面/外围接口：

- OAuth token / roles / create_api_key
- `remote-control` environment 注册
- telemetry event logging
- MCP proxy

因此“gateway”不应理解成只有一个推理 endpoint，而是：

- inference/data plane：`api.anthropic.com` 上的 `/v1/...`
- control plane：OAuth / remote / file / telemetry / MCP proxy 等外围服务

控制面接口的完整归类、host 分层与远程环境/session 关系，已经单独拆到 [10-control-plane-api-and-auxiliary-services.md](./10-control-plane-api-and-auxiliary-services.md)，本页不再继续堆叠 endpoint 总表。

### beta/header 族

目前已补出的 beta/header 证据包括：

- `structured-outputs-2025-12-15`
- `token-counting-2024-11-01`
- `message-batches-2024-09-24`
- `files-api-2025-04-14`
- `skills-2025-10-02`

这说明：

- `Jk6` 不只是“发消息”
- 还会驱动 structured output、token counting、files、skills 等能力面
- `sdkBetas` 是 session/runtime 级别状态，而不是单次请求局部常量

### 3P provider 细节

已补出的 3P provider 行为：

- Bedrock
  - 使用 `AnthropicBedrock`
  - 读取 AWS region / AWS creds
  - 支持 `AWS_BEARER_TOKEN_BEDROCK`
  - 可通过 `CLAUDE_CODE_SKIP_BEDROCK_AUTH` 跳过鉴权
- Foundry
  - 使用 `AnthropicFoundry`
  - 读取 `ANTHROPIC_FOUNDRY_BASE_URL` 或 `ANTHROPIC_FOUNDRY_RESOURCE`
  - `resource` 会展开成 `https://<resource>.services.ai.azure.com/anthropic/`
  - 认证是 API key 与 Azure AD token provider 二选一
- Vertex
  - 使用 `AnthropicVertex`
  - 借助 `GoogleAuth`
  - `/v1/messages` 会被改写到 `projects/.../publishers/anthropic/models/...:rawPredict`

### 模型名映射层

provider 之间并不共享完全相同的模型 ID。

bundle 中存在 first-party / bedrock / vertex / foundry 四路模型名映射表，例如：

- `claude-sonnet-4-6`
- `us.anthropic.claude-sonnet-4-6`
- `claude-sonnet-4-6`
- `claude-sonnet-4-6`

这会直接影响：

- `--model`
- `--fallback-model`
- `small-fast-model`
- capability 检查

因此重写时需要单独保留 model-name normalization / provider remap 层。

底层流事件类型：

- `message_start`
- `message_delta`
- `message_stop`
- `content_block_start`
- `content_block_delta`
- `content_block_stop`

并在本地重建：

- `text_delta`
- `input_json_delta`
- `thinking_delta`
- `signature_delta`
- `citations_delta`
- `compaction_delta`

### 一个关键发现：server tool use

`WebSearchTool.call(...)` 不是自己发 HTTP 搜索，而是再次调用 `Jk6` 去驱动 `server_tool_use`。

这说明 `Jk6` 并不只服务于普通聊天或本地 tool use，也会被拿来驱动 server-side tool。

`web_search` 这一条链现在已经单独拆到 [07-web-search-tool.md](./07-web-search-tool.md)：

- 本页只保留一个结论：`Jk6` 不只是普通聊天请求门面，也能驱动 server-side tool
- `web_search` 的 schema、版本、流事件回收、`WebSearchOutput` 与 transcript 模板统一在专题页维护
- 这样可以避免 provider 总览页与专题页双份漂移

### 结论

`Jk6` 不是普通聊天请求器，而是：

- 普通模型对话请求器
- structured output 生成器
- server-side tool use 驱动器

三合一门面。

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
