# 控制面 API 与外围服务：Host、认证与数据面

## 一句话结论

Claude Code CLI 的 first-party 网络面至少要分成四层看：

```text
inference/data plane
  -> /v1/messages?beta=true
  -> /v1/messages/count_tokens?beta=true
  -> /v1/messages/batches?beta=true
  -> /v1/models?beta=true
  -> /v1/files

account / auth control plane
  -> /v1/oauth/token
  -> /api/oauth/profile
  -> /api/oauth/claude_cli/roles
  -> /api/oauth/claude_cli/create_api_key

remote / environment / session control plane
  -> /v1/environment_providers
  -> /v1/environment_providers/cloud/create
  -> /v1/sessions
  -> /v1/sessions/{id}
  -> /v1/sessions/{id}/events
  -> /v1/sessions/{id}/archive
  -> /v1/sessions/ws/{id}/subscribe
  -> /v1/environments/{id}/bridge/reconnect
  -> /v1/environments/{id}/work/{workId}/heartbeat
  -> /v1/environments/{id}/work/{workId}/stop
  -> /v1/environments/bridge/{id}

peripheral services
  -> /api/oauth/organizations/{org}/code/repos/{owner}/{repo}
  -> /api/oauth/organizations/{org}/sync/github/auth
  -> /v1/code/github/import-token
  -> /api/claude_code/settings
  -> /api/event_logging/batch
  -> https://mcp-proxy.anthropic.com/v1/mcp/{server_id}
```

关键点不是“endpoint 很多”，而是：

- `/v1/messages` 一族只是数据面，不等于全部 first-party API
- `OAuth access token + organization UUID` 是大量控制面接口的共同门禁
- `remote-control / bridge / MCP proxy / telemetry` 都有各自独立 transport，不应继续塞进 provider 适配层叙述

## Host 与职责分层

### `https://api.anthropic.com`

主要承载：

- `/v1/messages*`
- `/v1/models*`
- `/v1/files*`
- `/api/oauth/profile`
- `/api/oauth/claude_cli/roles`
- `/api/oauth/claude_cli/create_api_key`
- `/v1/environment_providers*`
- `/v1/sessions*`
- `/api/event_logging/batch`

因此 `BASE_API_URL` 不是纯推理网关，而是 **数据面 + 大部分控制面** 的统一宿主。

### `https://platform.anthropic.com`

主要承载 OAuth 浏览器登录与 token 交换：

- `/oauth/authorize`
- `/v1/oauth/token`
- `/oauth/code/success`
- `/oauth/code/callback`

也就是说，登录授权页与 token 交换并不走 `api.anthropic.com`。

### `https://claude.ai` / `https://claude.com`

这组 host 当前不应被当成“纯产品文案域名”忽略掉。  
本地 bundle 里还能直接看到：

- `claude.ai`
  - web/app origin
  - remote-control、marketplace、settings/connectors、usage、privacy 等产品入口文案
- `claude.com`
  - `CLUADE_AI_AUTHORIZE_URL` 相关 OAuth authorize host

因此更稳的 host 结论是：

- `platform.anthropic.com`
  - console-side OAuth / token / callback host
- `claude.ai` / `claude.com`
  - web/app origin 与另一组可见 OAuth / 产品入口 host

不能把 first-party web/auth host 简化成只剩 `platform.anthropic.com`。

### `https://mcp-proxy.anthropic.com`

这是 MCP 代理专用宿主：

- `/v1/mcp/{server_id}`

它不是普通 provider endpoint，也不是 `BASE_API_URL` 的子路径。

### local / custom override

bundle 里同时保留了：

- localhost 开发版 URL 集合
- `CLAUDE_CODE_CUSTOM_OAUTH_URL`

因此这些 host 不是常量写死，而是有环境切换层。

## 认证形态总表

### 数据面

`BH()` 暴露的是统一 first-party auth header 选择器：

- subscriber 路径：`Authorization: Bearer <oauth access token>`，并附 `anthropic-beta: oauth-2025-04-20`
- API key 路径：`x-api-key: <key>`

它服务的是：

- `/v1/messages*`
- `/v1/models*`
- `/v1/files*`
- `/api/event_logging/batch` 的有信任路径

另外还要补一个边界：

- 同一套 first-party bearer / api key 选择逻辑
  - 也会被 `/api/claude_code/settings` 复用
  - 但这不意味着该接口应被归入数据面

### OAuth / account 控制面

这组接口只认：

- `Authorization: Bearer <oauth access token>`

其中：

- `/api/oauth/profile`
- `/api/oauth/claude_cli/roles`
- `/api/oauth/claude_cli/create_api_key`

都不走 API key。

### first-party 远端配置通道

`/api/claude_code/settings` 这一条还要单独留出来。  
它不属于 `/v1/messages` 数据面，也不等价于 org-scoped remote/session control plane。

当前能确定的是：

- endpoint：
  - `GET ${BASE_API_URL}/api/claude_code/settings`
- 认证：
  - `x-api-key`
  - 或 `Authorization: Bearer <oauth access token>`，并附 `anthropic-beta`

因此它更准确属于：

- first-party remote managed settings control path

而不是：

- 纯数据面 supporting endpoint
- 或必须带 `x-organization-uuid` 的 org-scoped remote control plane

### org-scoped 远程控制面

`UM()` 是远程会话/环境接口的共同门禁，要求：

- 必须有 claude.ai OAuth access token
- 必须能拿到 `organizationUuid`

共同 header 形态：

- `Authorization: Bearer <oauth access token>`
- `Content-Type: application/json`
- `anthropic-version: 2023-06-01`
- `x-organization-uuid: <orgUUID>`
- 大多数路径还会加 `anthropic-beta: ccr-byoc-2025-07-29`

这说明 remote / environment / code session 这组接口是 **org-scoped control plane**，不是普通用户级 API。

### MCP proxy

`claudeai-proxy` transport 不直接复用 `BH()`，而是：

- 用 `ow_(fetch)` 在请求前注入 `Authorization: Bearer <oauth access token>`
- 首次 `401` 后会检测 token 是否变化并尝试重放
- 固定补 `X-Mcp-Client-Session-Id: <sessionId>`

因此它的身份绑定点除了 OAuth，还显式绑到了当前 CLI session。

## 数据面与 supporting endpoints

### `/v1/messages?beta=true`

- 用途：主模型请求、server-side tool、structured outputs
- 调用入口：`client.beta.messages.create(...)`，上层来自 `_I4(...) / Jk6(...)`
- 认证：`BH()` 选出的 bearer 或 api key
- 与 session/org 的关系：
  - 本地 session 会进 request metadata / telemetry
  - 但从本地代码看，它不像 remote API 那样强制 `x-organization-uuid`

### `/v1/messages/count_tokens?beta=true`

- 用途：token counting
- 调用入口：`client.beta.messages.countTokens(...)`
- 认证：同数据面
- 特征：附加 `token-counting-2024-11-01`

### `/v1/messages/batches?beta=true`

当前 SDK wrapper 已能确认这些子路径：

- `POST /v1/messages/batches?beta=true`
- `GET /v1/messages/batches/{id}?beta=true`
- `GET /v1/messages/batches?beta=true`
- `DELETE /v1/messages/batches/{id}?beta=true`
- `POST /v1/messages/batches/{id}/cancel?beta=true`

用途是 Messages API 的异步批处理 supporting endpoints，不应再遗漏。

### `/v1/models?beta=true`

已确认：

- `GET /v1/models?beta=true`
- `GET /v1/models/{id}?beta=true`

用途：

- 模型能力发现
- context window / capability 辅助判断
- 支撑 CLI 的 model selection 与 fast-path capability 判断

### `/v1/files`

已确认：

- `GET /v1/files`
- `POST /v1/files`
- `GET /v1/files/{id}`
- `GET /v1/files/{id}/content`
- `DELETE /v1/files/{id}`

虽然它不是 `/v1/messages` 本体，但仍属于 first-party 数据面 supporting API，而不是远程控制面。

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
