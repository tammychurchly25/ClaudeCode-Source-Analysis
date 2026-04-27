# 控制面 API 与外围服务：GitHub 接入、Telemetry 与 MCP Proxy

## GitHub 接入控制面

### 仓库访问能力检查

#### `/api/oauth/organizations/{orgUUID}/code/repos/{owner}/{repo}`

- 用途：检查 GitHub App 是否已装到仓库
- 调用入口：`Xt6(owner, repo)`
- 认证：Bearer + `x-organization-uuid`
- 返回重点：`status.app_installed`

#### `/api/oauth/organizations/{orgUUID}/sync/github/auth`

- 用途：检查 web-setup 的 GitHub token 是否已同步
- 调用入口：`bu_()`
- 认证：Bearer + `x-organization-uuid`
- 返回重点：`is_authenticated`

这两条链分别代表：

- GitHub App 安装态
- OAuth/token-sync 安装态

CLI 会把它们并列看成 remote repo access 的两种来源。

### `/v1/code/github/import-token`

- 用途：把本地 `gh auth token` 上传到 Claude web side
- 调用入口：`OC4(secretToken)`
- 认证：Bearer + `x-organization-uuid` + 专用 beta header
- 典型调用场景：`/web-setup`

因此它不是 repo-level check，而是 **把本地 GitHub credential 导入远端控制面** 的接口。

## telemetry event logging

### `/api/event_logging/batch`

- 用途：first-party event logging 批量上报
- 调用入口：
  - `Qg7()` 初始化 1P event logging
  - `ZZ1.sendBatchWithRetry(...)` 真正发送
- 默认 path：`/api/event_logging/batch`
- 认证：
  - 优先走 `BH()` 拿到的 bearer / api key
  - 若 trust 未建立或 OAuth 已过期，会降级为无 auth 发送
  - 若带 auth 命中 `401`，会再试一次无 auth
- 固定 headers：
  - `Content-Type: application/json`
  - `User-Agent`
  - `x-service-name: claude-code`

它和 `/v1/messages` 的关系不是“顺手打点”，而是单独的上传通道，失败时还会把批次落盘到：

- `telemetry/1p_failed_events.<session>.<uuid>.json`

### 事件类型

当前至少能直接看到两族：

- `ClaudeCodeInternalEvent`
- `GrowthbookExperimentEvent`

因此 telemetry event logging 本身就是一个控制面子系统，而不是附着在某个 provider 请求上的小功能。

### 发送策略与配置来源

`/api/event_logging/batch` 这一层不是完全写死常量。  
当前已能确认它会吃 `tengu_event_sampling_config`、`tengu_1p_event_batch_config`，并在部分字段缺省时回退到本地 env（例如 `OTEL_LOGS_EXPORT_INTERVAL`）。

因此更稳的理解是：

- 1P event logging 的采样、flush cadence、endpoint、认证策略与重试上限都不是固定编译时常量
- 这一页只保留 endpoint 视角的结论
- 配置来源、初始化时机与 gating 矩阵，主落点见：
  - [../09-api-lifecycle-and-telemetry.md](../09-api-lifecycle-and-telemetry.md)

### event payload 形状

`ZZ1.transformLogsToEvents(...)` 会把 OTEL log record 序列化成两类 payload，见 `cli.js`。

#### `GrowthbookExperimentEvent`

这一类会走 `WZ1.toJSON(...)`，当前最关键字段包括：

- `event_id`
- `timestamp`
- `experiment_id`
- `variation_id`
- `environment`
- `device_id`
- `session_id`
- `auth`

此外本地还能直接看到它也会携带：

- `user_attributes`
- `experiment_metadata`

其中 `auth` 只有在 `account_uuid` 或 `organization_uuid` 至少存在一个时才会附带。

#### `ClaudeCodeInternalEvent`

这一类会走 `lP8.toJSON(...)`，而且不是把原始 attributes 平铺直塞，而是先做一次结构化拆层：

- `core`
  - 由 `core_metadata` 提供
  - 当前可直接看到的核心字段包括 `session_id`、`model`、`user_type`、`is_interactive`、`client_type`
  - 某些场景还会补 `betas`、`entrypoint`、`agent_sdk_version`、`agent_id`、`parent_session_id`、`agent_type`、`team_name`
- `env`
  - 由 `core_metadata.envContext` 提供
  - 当前至少包括 `platform`、`arch`、`node_version`、`terminal`、`package_managers`、`runtimes`、`deployment_environment`
  - 命中时还会补 `remote_environment_type`、`claude_code_container_id`、`claude_code_remote_session_id`、GitHub Actions / WSL / Linux distro 等上下文
- `process`
  - 由 `core_metadata.processMetrics` 转成 base64 JSON
- `auth`
  - 由 `user_metadata.accountUuid / organizationUuid` 汇总
- `additional_metadata`
  - 来自 `yg7(...).additional`
  - 会先去掉 `_PROTO_*` 保留字段，再转成 base64 JSON

这里还有两个不能漏掉的行为：

- `_PROTO_skill_name`、`_PROTO_plugin_name`、`_PROTO_marketplace_name` 不进入 `additional_metadata`，而是提升成顶层 `skill_name`、`plugin_name`、`marketplace_name`
- `device_id` 与 `email` 也会被单独提升到顶层

#### 缺 `core_metadata` 时的兜底

如果 log record 缺 `core_metadata`，不会直接丢弃，而是退化成最小 `ClaudeCodeInternalEvent`：

- 仍保留 `event_id`、`event_name`、`client_timestamp`、`session_id`
- `additional_metadata` 会变成一个 base64 JSON
- 该 JSON 里固定写入 `transform_error: "core_metadata attribute is missing"`

因此更稳的理解是：

- `ClaudeCodeInternalEvent` 的正常形态是 `core/env/process/auth/additional_metadata` 五层结构
- `additional_metadata` 不是任意字符串，而是编码后的 JSON 余量区
- transform 失败也不是静默吞掉，而是显式留下 `transform_error` 兜底痕迹

## MCP proxy

### `https://mcp-proxy.anthropic.com/v1/mcp/{server_id}`

- 用途：以 `claudeai-proxy` transport 代理远端 MCP server
- 调用入口：MCP client transport 创建分支
- 认证：
  - `ow_(fetch)` 注入 Bearer access token
  - `401` 后根据 token 是否变化决定是否重试
- 会额外带：
  - `User-Agent`
  - `X-Mcp-Client-Session-Id: <sessionId>`

它和普通 MCP `http/sse/ws` transport 的根本区别是：

- 不是用户直连服务器
- 而是先过 Anthropic 的 OAuth 代理层

所以 `claudeai-proxy` 更像 **MCP 控制面入口**，不是一般的 transport 变体。

## 当前仍未完全钉死

- bridge/worker 侧 `environment_secret` 与 `session_ingress_token` 的服务端签发语义还没有单独拆页
- `/v1/code/github/import-token` 的 beta 头常量已看到调用位，但当前没有继续追到完整命名来源
- remote environment 的**完整**返回 schema 仍未单独列表化；但本地主流程依赖的消费字段已基本收敛到 `environment_id / name / kind`

不过对“控制面接口图”这个层级来说，上述缺口已经不再妨碍分类与重写边界判断。

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
