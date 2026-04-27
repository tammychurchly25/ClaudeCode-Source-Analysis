# 控制面 API 与外围服务：Bridge 凭据与 Worker 生命周期

## environment / bridge 辅助接口

在 bridge API client 中还能直接看到：

- `POST /v1/environments/bridge`
- `POST /v1/environments/{environmentId}/bridge/reconnect`
- `GET /v1/environments/{environmentId}/work/poll`
- `POST /v1/environments/{environmentId}/work/{workId}/ack`
- `POST /v1/environments/{environmentId}/work/{workId}/heartbeat`
- `POST /v1/environments/{environmentId}/work/{workId}/stop`
- `DELETE /v1/environments/bridge/{environmentId}`

其中 `POST /v1/environments/bridge` 的本地 request body 当前已能直接写成：

- `machine_name`
- `directory`
- `branch`
- `git_repo_url`
- `max_sessions`
- `metadata.worker_type`
- 可选：
  - `environment_id`
    - 仅在 reuse/re-register 时带上

它的本地响应消费当前至少包括：

- `environment_id`
- `environment_secret`

这里还能继续把 bridge API client 的 bearer 分工写实：

其中一组明显仍在使用 OAuth access token 的调用是。  
它们都走 `getAccessToken()` / `onAuth401()` 包装，本地看到的是 OAuth access token：

- `registerBridgeEnvironment(...)`
- `reconnectSession(...)`
- `deregisterEnvironment(...)`
- `archiveSession(...)`

而下面这些调用则直接拿 `environment_secret` 或 `session_ingress_token` 占用同一个 bearer 槽位：

- `pollForWork(environmentId, environment_secret, ...)`
  - bearer 才是 `environment_secret`
- `acknowledgeWork(...)`
- `heartbeatWork(...)`
- `sendPermissionResponseEvent(...)`
  - bearer 来自 `session_ingress_token`
- `stopWork(environmentId, workId, force)`
  - 本地调用仍走 OAuth bearer，而不是 `environment_secret`

也就是说，本地看到的不是“一把 remote-control token 管全部接口”，而是同一个 control-plane header 槽位被不同 credential 占用：

- OAuth access token
  - environment register / reconnect / archive / deregister / stop
- `environment_secret`
  - work poll
- `session_ingress_token`
  - work ack / heartbeat / session ingress / permission-response events

而 `/bridge/reconnect` 的 request body 只需要：

- `session_id`

这些接口说明 remote-control 不只是“远程 session 列表 + 创建”，还存在：

- environment registration / deregistration
- work lease / heartbeat
- bridge reconnect

也就是说，`remote-control` 背后实际是一整套 environment worker control plane。

### `environment_secret -> work secret -> session_ingress_token` 的本地链

work poll 命中 session work 后，本地会把 `work.secret` 当成 base64url JSON 解开。  
当前可直接确认：

- secret payload 要求 `version === 1`
- 至少必须包含：
  - `session_ingress_token`
  - `api_base_url`
- 否则会被当成 invalid work secret，并主动 stop work

解出后的本地消费链可收束为：

```text
OAuth access token
  -> POST /v1/environments/bridge
  -> { environment_id, environment_secret }
environment_secret
  -> /work/poll
  -> work.secret
  -> decode
  -> session_ingress_token
  -> ack / heartbeat / transport / child session refresh
```

更细一点：

- `acknowledgeWork(...)`
  - 认证头直接用 `session_ingress_token`
- 已存在 session 收到新 work 时
  - 会把新的 `session_ingress_token` 注入现有子进程
  - 注入方式是 stdin 下发：
    - `type: "update_environment_variables"`
    - `CLAUDE_CODE_SESSION_ACCESS_TOKEN=<new token>`
- 非 CCR v2 transport
  - 会把 `session_ingress_token` 交给 `kg8(...)` 做本地 JWT 到期前刷新调度
- CCR v2 transport
  - 先用 `session_ingress_token` 调 `POST <sessionUrl>/worker/register`
  - 响应里再拿 `worker_epoch`

`session_ingress_token` 的本地形状现在也能再收紧一点：

- `kg8.schedule(sessionId, jwt)` 不只是把它当 opaque string
- 本地会先尝试：
  - 去掉可选前缀 `sk-ant-si-`
  - 再按 `header.payload.signature` 形态切开
  - 对中段做 base64url JSON decode
  - 读取 `exp`
- 解不出 `exp` 时
  - 只会保留现有 timer
  - 不会因此判定 token 非法

因此更稳的本地结论是：

- `session_ingress_token` 至少**常见形态**是带 `exp` claim 的 JWT-like token
- 但当前不能只靠这份 bundle 断言它在服务端总是标准 JWT

因此更稳的结论不是“bridge 只有一个 secret 到底”，而是：

- OAuth access token 负责 environment register/reconnect 级控制面
- `environment_secret` 管 environment lease
- `session_ingress_token` 管具体 session ingress
- CCR v2 还会在 session ingress 之上再叠 `worker_epoch`

### `code session -> /bridge` credentials 响应字段

env-less bridge / CCR v2 的 `/v1/code/sessions/{id}/bridge` 响应字段现在也已收紧：

- `worker_jwt`
- `expires_in`
- `api_base_url`
- `worker_epoch`

本地还会继续验证：

- `worker_epoch`
  - 允许原始值是 string 或 number
  - 但最终必须能转成安全整数

因此这一层已经不再只是“拿一个 token”，而是完整的：

- ingress auth
- API base URL
- worker epoch/version
- 到期时间

四元组。

### `worker_jwt` 的刷新与重建链

env-less code session 路径的本地 token 生命周期，现在也已经能写到调度级：

- 初次流程：
  - `POST /v1/code/sessions`
  - `POST /v1/code/sessions/{id}/bridge`
  - 用返回的 `worker_jwt / api_base_url / worker_epoch`
    - 建 `S18(...)` v2 transport
- 刷新调度器：
  - 复用同一个 `kg8(...)`
  - 这里走的是 `scheduleFromExpiresIn(sessionId, expires_in)`
  - 默认刷新 buffer 来自 remote bridge config：
    - `token_refresh_buffer_ms = 300000`
  - 计算方式是：
    - `max(expires_in * 1000 - buffer, 30000)`
- proactive refresh：
  - 定时器触发后，先尝试拿最新 OAuth token
  - 再重新请求 `/v1/code/sessions/{id}/bridge`
  - 若成功，会保留 `lastSequenceNum`
  - 用新 `worker_jwt / worker_epoch / api_base_url` 重建 transport
- recovery refresh：
  - 若 v2 transport close code 是 `401`
  - 会立刻走同一条 `/bridge` 刷新链
  - 成功后同样重建 transport

因此 `worker_jwt` 的本地语义不是“一次性桥接票据”，而是：

- 有明确到期时间
- 有提前刷新窗口
- 刷新失败会使 transport 进入 failed
- 刷新成功后会带着 sequence 续接

同时这里还能继续收紧一个容易混淆的边界：

- 当前本地没有看到 `worker_jwt` 走 `wGz(...) / eqA(...)` 这条 JWT payload decode 路
- env-less 路径的刷新完全由 `/bridge` 返回的 `expires_in` 驱动

因此截至当前本地证据：

- `worker_jwt` 明确是短期 worker credential
- 但还不能像 `session_ingress_token` 那样，把它进一步写成“常见形态是 JWT-like token”

### `worker_epoch` 的本地失败语义

`worker_epoch` 当前已经不只是“请求里带一个版本号”，而是会触发明确的 worker supersession 处理：

- 同一个 `worker_epoch` 会被附到：
  - `PUT /worker`
  - `POST /worker/events`
  - `POST /worker/internal-events`
  - `POST /worker/events/delivery`
  - `POST /worker/heartbeat`
- 初始化 `PUT /worker` 还会显式上报：
  - `worker_status: "idle"`
  - `external_metadata.pending_action: null`
  - 这说明 `worker_epoch` 不只保护 event append，也保护 worker state 面
- 初始化时还有一个关键时序：
  - 会先 `GET /worker`
  - 但当前只读取 `worker.external_metadata`
  - 随后才做初始化 `PUT /worker`
- 当状态转成 `requires_action`
  - 本地还会把 `external_metadata.pending_action` 写成结构化对象：
    - `tool_name`
    - `action_description`
    - `tool_use_id`
  - 离开 `requires_action` 后再清回 `null`
- 同时还会单独上报：
  - `requires_action_details`
    - `tool_name`
    - `action_description`
  - 但不带 `tool_use_id`

- `409`
  - 直接视为 epoch mismatch
  - 触发 `handleEpochMismatch()`
- `401/403`
  - 若当前 token 自己看起来已经过期
    - 记 `cli_worker_token_expired_no_refresh`
    - 同样走 `onEpochMismatch()`
  - 若 token 看起来仍有效，但连续 `401/403` 达到阈值
    - 记 `cli_worker_auth_failures_exhausted`
    - 也走 `onEpochMismatch()`
    - 当前本地阈值常量是 `10`
- 在 repl CCR v2 transport 里
  - `onEpochMismatch()` 会主动关闭 worker uploader 与 SSE
  - 把控制权交还给 poll-loop recovery / reconnect

因此更稳的理解是：

- `worker_epoch` 不是纯装饰字段
- 它更像当前 worker lease / ownership 的 fencing token
- 这代 worker 一旦被服务端 supersede，本地会按“必须重建 transport / 甚至重建 session”处理

这里最后一句仍属于从本地调用方式推出的推断。  
更保守地说，也至少可以确认：

- `worker_epoch` 是 worker 面所有写操作共用的服务端承认条件
- 它的语义已经明显强于普通请求版本号
- worker state 面并不只上报粗粒度枚举
  - 还会把待审批动作摘要一起塞进 `external_metadata.pending_action`
- worker state 与 metadata 也不是完全同一个槽位
  - `requires_action_details` 更像当前状态摘要
  - `external_metadata.pending_action` 则保留了额外的 `tool_use_id`
  - 当前 resume 只读取 `worker.external_metadata`
  - 没看到本地还原链读取 `requires_action_details`
  - 即便初始化早期读到了旧 `pending_action`
    - 后续本地还原也只把它当作 `external_metadata` 的一部分拿到
    - 没看到它被重新构造成待审批 UI

### `environment_secret` 的本地轮换边界

关于 `environment_secret`，本地现在至少能更硬地排除一件事：

- 没看到独立的 `refresh environment secret` endpoint
- 没看到仅替换 `environment_secret` 而不重建 environment 的本地分支

当前本地可见的更新方式只有：

- 初次 `POST /v1/environments/bridge`
- repl/perpetual 还原时再次 `POST /v1/environments/bridge`
  - 可带原 `environment_id` 作为 `reuseEnvironmentId`
  - 然后尝试 `POST /v1/environments/{id}/bridge/reconnect`
  - 失败再新建 session

因此截至当前本地证据，更稳的表述应是：

- `environment_secret` 没看到独立 refresh
- 它的“轮换”更像 environment 丢失后的 re-register 结果
- standalone bridge 普通 fatal 退出路径里，也没看到后台自动替换旧 `environment_secret`

还能把 poll fatal 的本地分类再收紧一层：

- `404`
  - 在 repl bridge 的 poll loop 里，会被直接当成 environment lost / deleted
  - 命中 `onEnvironmentLost()`，最多尝试 `3` 次 re-register
- `410`
  - 默认 error type 会落成 `environment_expired`
  - 本地文案按“remote-control session expired，需要重新启动”处理
- `403`
  - 若 `error.type` 或 message 带 `expired / lifetime`
    - 也按 session expired 处理
  - 若 message 带 `external_poll_sessions` 或 `environments:manage`
    - 本地会把它当成权限/策略拒绝并 suppress，而不是 environment lost

因此截至当前静态证据，至少可以排除一种过度泛化：

- 不是所有 `environment_secret` 失效相关 fatal 都会触发同一种还原路径
- 当前本地明确区分了：
  - environment not found / deleted
  - session expired / lifetime expired
  - permission or policy denied
- 但这些分类仍然只是客户端对服务端响应的消费语义，不等于服务端正式文档定义

还需要补一个模式边界：

- repl/perpetual bridge core
  - `404` 会走 `onEnvironmentLost()`，尝试 re-register / reconnect / fresh session
- standalone bridge/session manager
  - 同类 `BridgeFatalError` 当前只会记 fatal telemetry 并退出 poll loop
  - 没看到同等级的 in-loop environment re-register

因此“environment lost 后自动重建”当前只能写在 repl/perpetual 路径上，不能直接泛化到全部 bridge 模式。

### 当前能确认、但还不能过度推断的边界

现在已经可以明确区分三种本地可见 credential：

- OAuth access token
  - bridge/code-session 控制面的上游凭据
- `environment_secret`
  - bridge environment control plane bearer
- `session_ingress_token`
  - work secret 内携带的 session ingress bearer
- `worker_jwt`
  - env-less code session `/bridge` 返回的短期 worker token

但当前仍不能只靠本地 bundle 正证：

- 这三者在服务端是否同源签发
- `environment_secret` 的真实 TTL 与服务端失效条件
- `session_ingress_token` 是否总是短于 `environment_secret`
- `worker_jwt` 与 `session_ingress_token` 是否共享统一 claims 结构

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
