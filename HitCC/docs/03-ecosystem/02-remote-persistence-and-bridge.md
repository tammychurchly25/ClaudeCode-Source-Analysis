# 远程控制、Bridge 与远程持久化

## 本页用途

- 用来单独整理 `remote-control / sdk-url / bridge` 这组外围接入形态。
- 用来固定远程 transcript 持久化与 `409` 冲突还原的闭环行为。

## 相关文件

- [01-resume-fork-sidechain-and-subagents.md](./01-resume-fork-sidechain-and-subagents.md)
- [../01-runtime/06-stream-processing-and-remote-transport.md](../01-runtime/06-stream-processing-and-remote-transport.md)
- [07-tui-system.md](./07-tui-system.md)
- [../05-appendix/02-evidence-map.md](../05-appendix/02-evidence-map.md)

## 远程控制、远程持久化与冲突还原

### Remote control / bridge

已确认有：

- `remote-control`
- `sdk-url`
- `stream-json` 输入输出
- Chrome/native-host/computer-use 旁路模式

### 本地产品入口与工作流矩阵

这一块现在已经不该再只写成“有 remote-control / sdk-url”。

当前本地可见入口至少分成 4 类：

| 入口 | 本地动作 | 产物/后果 | 更稳的定位 |
| --- | --- | --- | --- |
| `claude --remote-control [name]` / `claude rc [name]` / `/remote-control` | 在当前本地 REPL 上开启 bridge | 当前 session 获得 `session_url / connect_url / environment_id`，并允许 web/app 入站接管 | **本地会话开放远程接入** |
| `claude --remote [description]` | 先创建远端 code session；若 `tengu_remote_backend` 打开，再用 `RemoteSessionManager` 连进 TUI | CLI 变成远端 session 的前端；否则只打印 `View` 与 `--teleport` 提示 | **创建并附着远端 session** |
| `--sdk-url + --print + stream-json` | headless 子进程把 structured I/O 切到远端 transport | 仍跑本地主循环，只是 transport 改成 ingress/bridge | **agent backend / SDK transport 模式** |
| `useDirectConnect(wsUrl)` | 直接对某个 WebSocket endpoint 收发 user/control JSON | 本地只做消息适配与审批 UI | **更轻量的直连调试/嵌入模式** |

这 4 条路径虽然都带 “remote” 味道，但本地语义并不一样：

- `--remote-control`
  - 是把**当前本地会话**暴露给远端
- `--remote`
  - 是先拿到**远端 session**，再让 CLI 充当前端
- `--sdk-url`
  - 是把 CLI 当成**可嵌入 agent backend**
- `useDirectConnect`
  - 更像内部/调试侧的**裸 transport adapter**

因此后续文档不应再把它们混写成一个泛化的“远程模式”。

### `remoteControlAtStartup` 不是临时态，而是正式全局偏好

这一点现在也已经能写实：

- `remoteControlAtStartup`
  - 是正式 global config key
  - 默认读取逻辑是：
    - 若用户显式设置则取该值
    - 否则默认 `false`
- TUI 初始 app state 会把：
  - `replBridgeEnabled = cliFlagRemoteControl || remoteControlAtStartup || remoteSessionFrontend`
- 旧状态里的 `replBridgeEnabled`
  - 还会被迁移成新的 `remoteControlAtStartup`

因此 “启动时自动开 Remote Control” 不是散落在某个临时 dialog 里的行为，而是稳定进入全局设置与初始状态装配。

### `/remote-control` 的本地产品工作流已经闭环

这一块现在已经能按用户可见路径写出来：

1. 先跑 preflight
   - policy `allow_remote_control`
   - 账号/订阅/组织资格
   - bridge 版本门槛
2. 若当前已连接或已启用
   - 不再重复 connect
   - 而是进入 disconnect/status dialog
3. 若第一次命中且满足 callout 条件
   - 不会立刻 connect
   - 而是先弹 `remote-callout`
4. 用户在 callout 中选 `enable`
   - 写 `showRemoteCallout=false`
   - 写 `replBridgeEnabled=true`
   - 写 `replBridgeExplicit=true`
5. bridge hook 初始化成功后
   - app state 会拿到：
     - `replBridgeConnectUrl`
     - `replBridgeSessionUrl`
     - `replBridgeEnvironmentId`
     - `replBridgeSessionId`
   - 同时消息流会追加 `bridge_status`
6. 若之后再次执行 `/remote-control`
   - 进入 disconnect/status dialog
   - 可断开当前 session，也可显示 QR / connect URL

因此本地产品面已经不是“也许有个开关”，而是一条完整的：

```text
preflight
  -> first-time callout (optional)
  -> enable bridge
  -> state synced into app state
  -> transcript/footer/status dialog expose session URL
  -> user can later disconnect from same command
```

### 远端 UI / web / app 配合现在已有本地硬证据

前面更偏 transport，这里补产品侧结论：

- `remote-callout` 文案直接写明：
  - 当前 CLI session 可从 web (`claude.ai/code`) 或 Claude app 访问
- `bridge_status`
  - 文案是 `/remote-control is active. Code in CLI or at <url>`
  - 说明本地 transcript 会显式给出远端打开入口
- footer 区若存在 `remoteSessionUrl`
  - 会显示一个 `remote` 链接项
- disconnect/status dialog
  - 会展示当前 Remote Control URL
  - 还能切换 QR code 显示

所以“是否真有 web/app 配合”现在不再是推断，而是已被本地 UI 文案与状态字段坐实。

### IDE / desktop 配合也不再只是旁证

这部分虽然不属于 bridge transport 核心，但已经足以写成正式结论：

- CLI 命令面存在：
  - `mcp add-from-claude-desktop`
  - 只支持 macOS 与 WSL
  - 会读取 Claude Desktop 本地配置文件中的 `mcpServers`
  - 配置文件路径当前可见是：
    - macOS: `~/Library/Application Support/claude/claude_desktop_config.json`
    - WSL: `/mnt/c/Users/.../AppData/Roaming/claude/claude_desktop_config.json`
- 启动期存在 IDE auto-connect 注入：
  - 当 `autoConnectIde` / `autoConnectIdeFlag` / `CLAUDE_CODE_AUTO_CONNECT_IDE` / `CLAUDE_CODE_SSE_PORT` 命中时
  - 动态 MCP config 会被写入：
    - `ide.type = "sse-ide" | "ws-ide"`
    - `url`
    - `ideName`
    - `authToken`
    - `ideRunningInWindows`
    - `scope = "dynamic"`
- 这条 auto-connect 链不是“盲注入”
  - `PLq()` 会最多轮询约 `30s`
  - 只接受当前 cwd/workspace 命中的 IDE
  - 若显式给了 `CLAUDE_CODE_SSE_PORT`
    - 还会优先锁定对应 port
  - 某些本地 IDE 场景下
    - 还会额外校验当前进程与 IDE pid 关系
- auto-connect 之后还有产品动作：
  - 若未跳过 auto-install
    - 会检查 IDE 扩展 `anthropic.claude-code`
    - 缺失或版本落后时会尝试自动安装/升级
  - 若扩展刚装好且此前未展示过
    - 会触发 IDE onboarding dialog
- IDE MCP 真正连上后
  - 本地会额外发送一次 `ide_connected`
  - 负载当前只带 `pid: process.pid`
- `sse-ide / ws-ide` 也不是完全同构：
  - `ws-ide`
    - 会额外发 `X-Claude-Code-Ide-Authorization`
  - `sse-ide`
    - 直接走本地 SSE transport，不挂 OAuth provider

因此更稳的说法应是：

- 远程控制本身不是“IDE 专属功能”
- 但 Claude desktop / IDE 配套面在本地 bundle 中确实存在，而且已经进入正式启动链、Desktop config 导入、扩展安装/onboarding 与 MCP 动态装配

### 推断

CLI 可被上层桌面端/IDE/桥接端作为 agent backend 调用。

### bridge 与 session ingress 的真实边界

这一层现在可以进一步写实：

- bridge/session manager 启动的是：
  - `--print`
  - `--sdk-url`
  - `--input-format=stream-json`
  - `--output-format=stream-json`
- 子进程里仍跑本地 headless 主循环；远端 transport 只负责把事件接到 session ingress
- 可见 transport 族的上行对象分别是：
  - WebSocket 单条 `JSON line`
  - hybrid `POST { events: [...] }`
  - CCR v2 `/worker/events`、`/worker/internal-events`、`/worker/events/delivery`、`/worker`
- 下行可见对象也只是：
  - `client_event.payload`
  - control/permission 相关事件
  - keep_alive / worker 状态相关事件

因此当前更稳的判断是：

- bridge 是 **会话事件转运层**
- 不是本地外再加一层 prompt assembler 的代理层
- 如果远端还存在额外 `verification / context / systemContext` 拼装，更可能发生在服务端黑箱，而不是 bridge transport 本身

### bridge API client 的本地可见请求面

这一层现在也能写到接口级：

- `POST /v1/environments/bridge`
  - body:
    - `machine_name`
    - `directory`
    - `branch`
    - `git_repo_url`
    - `max_sessions`
    - `metadata.worker_type`
    - 可选 `environment_id`（reuse 时）
  - 响应当前至少消费：
    - `environment_id`
    - `environment_secret`
- `GET /v1/environments/{environmentId}/work/poll`
  - 可选 query:
    - `reclaim_older_than_ms`
- `POST /v1/environments/{environmentId}/work/{workId}/ack`
- `POST /v1/environments/{environmentId}/work/{workId}/stop`
  - body:
    - `force`
- `POST /v1/environments/{environmentId}/bridge/reconnect`
  - body:
    - `session_id`
- `DELETE /v1/environments/bridge/{environmentId}`

也就是说 bridge transport 前面还有一层明确的 **environment/work lease control plane**，不是 transport 自己裸连。

### 凭证不是一把钥匙，而是分层的

继续追 bridge 主链后，本地已能明确看到三层不同 credential：

- `environment_secret`
  - 来自 `POST /v1/environments/bridge`
  - 用于 environment/work lease control plane
- `session_ingress_token`
  - 来自 `work.secret` 解包
  - 用于 session ingress
- `worker_jwt`
  - 来自 `POST /v1/code/sessions/{id}/bridge`
  - 用于 env-less CCR v2 worker transport

它们不是同一 token 的别名。

同时还能再收紧一个容易误读的点：

- bridge API client 自己并没有“固定只吃某一种 token”
- 同一个 Authorization bearer 槽位，在不同接口上实际装的是不同 credential：
  - OAuth access token
    - `POST /v1/environments/bridge`
    - `POST /v1/environments/{id}/bridge/reconnect`
    - `DELETE /v1/environments/bridge/{id}`
    - `POST /v1/sessions/{id}/archive`
    - `POST /v1/environments/{id}/work/{workId}/stop`
  - `environment_secret`
    - `GET /v1/environments/{id}/work/poll`
  - `session_ingress_token`
    - `POST /v1/environments/{id}/work/{workId}/ack`
    - `POST /v1/environments/{id}/work/{workId}/heartbeat`
    - `POST /v1/sessions/{id}/events`

因此从本地代码看，bridge 不是“拿到 environment_secret 后所有接口都靠它”，而是一个分层 credential handoff 链。

### standalone/repl bridge：`environment_secret -> work secret -> session_ingress_token`

传统 remote-control / bridge 模式当前可收束成：

```text
OAuth access token
  -> registerBridgeEnvironment()
  -> { environment_id, environment_secret }
  -> control-plane register/reconnect/deregister/archive/stop
environment_secret
  -> pollForWork(environment_secret)
  -> work.secret
  -> decode base64url JSON (version=1)
  -> { session_ingress_token, api_base_url, ... }
  -> ack / heartbeat / transport
```

其中：

- `environment_secret`
  - 只在 work lease control plane 上行出现
- `session_ingress_token`
  - 才是 session 级 ingress credential
  - 本地常见形态还会被当成 JWT-like token：
    - 可选前缀 `sk-ant-si-`
    - 中段 base64url JSON payload
    - 里面至少会尝试读取 `exp`
- 子 session 若已存在
  - 本地会把新的 `session_ingress_token` 通过 stdin 注入子进程
  - 注入变量名是：
    - `CLAUDE_CODE_SESSION_ACCESS_TOKEN`

因此 bridge 模式的 token refresh 不是“重新注册环境”，而更像：

- environment 层继续用 `environment_secret` 拉 work
- session 层按新 work 下发新的 ingress token

同时还能再收紧一层：

- 目前没看到独立的 `environment_secret` refresh
- 本地可见更新方式主要只有：
  - 初次 register
  - environment 丢失后的 re-register
- perpetual/repl 还原时会：
  - 重新 `POST /v1/environments/bridge`
  - 尝试复用旧 `environment_id`
  - 随后再试 `bridge/reconnect`

因此 `environment_secret` 更像 environment lease 的注册结果，而不是单独可续租的 session token。

但还能再收紧一个很重要的边界：

- repl bridge 的 poll loop 遇到 `404`
  - 会把它当成 environment lost / deleted
  - 进入最多 `3` 次的 re-register 流程
- `410`
  - 默认映射到 `environment_expired`
  - 本地文案按“session expired，需要重新启动 remote-control”处理
- `403`
  - 若 message / error type 带 `expired / lifetime`
    - 也按 expired 处理
  - 若 message 带 `external_poll_sessions` 或 `environments:manage`
    - 本地会 suppress 成权限/策略拒绝，而不是 environment lost

因此就当前 bundle 而言，`environment_secret` 相关 fatal 至少被客户端分成三类：

- environment deleted / not found
- session or lifetime expired
- permission or policy denied

但这仍不足以直接推出服务端的真实失效条件，例如固定 TTL、idle timeout、被新环境挤掉，还是后台策略变更。

这里还要补一个模式边界：

- repl/perpetual bridge
  - `404` 会进 `onEnvironmentLost()`，尝试 re-register + reconnect / fresh session
- standalone bridge/session manager
  - `BridgeFatalError` 会直接记 fatal telemetry 并跳出 poll loop
  - 当前没看到同等级的 in-loop re-register 分支
  - 后续更多是 shutdown / archive / deregister 收尾

因此“environment lost 后自动重建”目前不能泛化成所有 bridge 模式的统一行为；它更像 repl/perpetual core 的专门还原链。

### CCR v2 bridge：`session_ingress_token -> registerWorker -> worker_epoch`

standalone/repl bridge 命中 CCR v2 时，还会在 session ingress 之上再叠一层 worker 面：

- `session_ingress_token`
  - 先去调 `POST <sessionUrl>/worker/register`
- 响应要求返回：
  - `worker_epoch`
- 若 `worker_epoch` 不是安全整数
  - 本地直接视为失败

所以 CCR v2 多出来的关键状态不是新 token，而是：

- `worker_epoch`
  - 用于约束 `/worker/events*` 与 `PUT /worker`
  - 还会进入 `/worker/heartbeat`
  - 本地把它当成 worker 版本/epoch guard

它的失败语义也已经比较实：

- `409`
  - 直接按 epoch mismatch 处理
- `401/403`
  - 若 token 已过期但没有收到新 token
    - 也会触发 worker 失效路径
  - 若 token 看起来还有效，但连续 auth failure 达阈值
    - 同样触发 worker 失效路径
    - 当前本地阈值是连续 `10` 次
- repl bridge 下的 CCR v2 transport
  - 会主动关闭 uploader/SSE
  - 把还原动作交回 poll-loop / reconnect 逻辑

同时 worker 初始化还会立刻做一次：

- `PUT /worker`
  - `worker_status: "idle"`
  - `external_metadata.pending_action: null`

这进一步说明 `worker_epoch` 保护的不是单一事件流，而是整个 worker state / event plane。

还能再收紧一层 `pending_action` 的真实形状：

- 当本地进入 `requires_action`
  - 不是只写一个布尔状态
  - 还会同步写：
    - `external_metadata.pending_action.tool_name`
    - `external_metadata.pending_action.action_description`
    - `external_metadata.pending_action.tool_use_id`
- 这份结构来自 `can_use_tool` 控制请求前的本地摘要
  - `tool_name`
  - 面向用户的动作描述 `action_description`
  - `tool_use_id`
- 离开 `requires_action` 后
  - 会把 `external_metadata.pending_action` 清回 `null`

因此从本地 bundle 看，CCR v2 worker state 不只是上报 `idle/running/requires_action` 枚举，还会把“当前卡在哪个待审批动作上”同步到 worker metadata。  
但当前本地没看到它在还原期被 CLI 自己重新消费；更像是提供给远端 worker/session 面板或服务端协调层观察。

还能再把 `external_metadata` 的还原边界收紧一层：

- 它并不是纯观察字段
- 当前本地至少还会把这些运行态写进去：
  - `model`
  - `permission_mode`
  - `is_ultraplan_mode`
- `--resume` 命中 CCR v2 时
  - 会先等 `restoredWorkerState = GET /worker -> worker.external_metadata`
  - 然后只还原：
    - `model`
    - `permission_mode`
    - `is_ultraplan_mode`

这里还能再把读取时序收紧一层：

- `ccrClient.initialize()` 不是先 `PUT /worker` 再读旧状态
  - 而是先发起 `getWorkerState()`
  - 读取对象当前只取 `worker.external_metadata`
  - 随后才立刻 `PUT /worker`
    - `worker_status: "idle"`
    - `external_metadata.pending_action: null`
- 这意味着当前 worker 若远端还残留旧 `pending_action`
  - 本地确实会在初始化早期把它读出来
  - 但随后又会把远端 worker state 的 `pending_action` 主动清空
  - 而本地 resume 仍只消费：
    - `model`
    - `permission_mode`
    - `is_ultraplan_mode`

但这里正好也给出一个关键负证：

- 当前本地 resume 路径没有消费 `pending_action`
- 本地权限 UI 仍由实时 `control_request { subtype: "can_use_tool" }` 驱动
- 取消/完成同样靠实时 `control_cancel_request` / `control_response`

因此截至当前静态证据，`external_metadata` 在本地至少分成两类角色：

- 可还原的 session setting snapshot
  - `model / permission_mode / is_ultraplan_mode`
- 主要提供远端观察的 worker action snapshot
  - `pending_action`

这里还能把“本地到底消不消费 `pending_action`”再收紧一层：

- SDK/bridge 主循环真正接权限请求时
  - 走的是实时 `control_request { subtype: "can_use_tool" }`
  - `RemoteSessionManager` / `useRemoteSession` / `useDirectConnect` 都会把它塞进本地 `pendingPermissionRequests` 队列
  - 再据此构造 permission prompt UI
- `WA8`/structured I/O 里
  - 也单独维护 `pendingRequests` Map
  - `initialize` 重入时报错时，返回的还是 `pending_permission_requests`
  - 来源同样是当前未决 `control_request`，不是 `restoredWorkerState`
- `session_state_changed`
  - 当前只看到 schema、发射点和 stream-json 输出过滤
  - 没看到本地 UI/还原逻辑把它当作待审批来源重新消费

因此当前更硬的本地结论应是：

- `pending_action` 是远端可观察的 worker snapshot
- 本地待审批交互面则是另一套实时 control-request queue
- 二者会被同步维护，但不是同一个还原入口

因此更稳的说法是：

- `worker_epoch` 不只是“请求版本号”
- 它更像当前 worker ownership / lease 的 fencing 条件
- 但“fencing token”仍是根据本地写入面和失败语义作出的推断，不是服务端文档明示

### env-less code session：`OAuth -> /bridge -> worker_jwt`

env-less remote bridge 则是另一条独立链：

```text
OAuth access token
  -> POST /v1/code/sessions
  -> POST /v1/code/sessions/{id}/bridge
  -> { worker_jwt, expires_in, api_base_url, worker_epoch }
  -> create v2 transport
```

这条链里本地看不到 `environment_secret`，而是直接：

- 用 OAuth 去换短期 `worker_jwt`
- 用 `expires_in` 驱动提前刷新
- 用 `worker_epoch` 保护 worker 面写入

这里还能补一条关键负证：

- 当前本地没有看到 `worker_jwt` 走 `wGz(...) / eqA(...)` 的 JWT payload decode
- 它的刷新完全由 `/bridge` 返回的 `expires_in` 调度

因此本地能确认的是：

- `worker_jwt` 是短期 worker credential
- 但不能仅凭本地 bundle 把它写成“常见形态也是 JWT-like”

### 三类 bridge 还原状态机对照

当前客户端侧已经能把三条还原链拆开写，不应再混成一个“remote bridge recovery”：

- repl/perpetual bridge
  - `environment_secret -> pollForWork -> work.secret -> session_ingress_token`
  - `404` poll fatal 会进 `onEnvironmentLost()`
  - 最多 `3` 次 re-register environment
  - transport close 会先尝试 `reconnectEnvironmentWithSession()`
  - heartbeat fatal 会清 `workId/sessionToken`、关闭当前 transport、保留 sequence、快速回到 poll
  - CCR v2 `worker_epoch` superseded 会主动关闭 uploader/SSE，把还原交回 poll-loop
- standalone bridge/session manager
  - 同样有 environment/work/session 三层 credential handoff
  - 但 poll loop 命中 `BridgeFatalError` 后，当前主路径是记 telemetry、跳出循环
  - 随后进入 shutdown / archive / deregister / cleanup
  - 没看到与 repl 同等级的 in-loop re-register environment
- env-less code session
  - `POST /v1/code/sessions -> POST /bridge -> { worker_jwt, expires_in, api_base_url, worker_epoch }`
  - 没有 `environment_secret`
  - 401 close / proactive timer 都会触发 JWT refresh
  - 用新 `worker_jwt / worker_epoch / api_base_url` 重建 transport，并带上 last sequence
  - teardown 主要是 archive session，不是 deregister environment

因此更稳的写法不是“bridge 有一套还原状态机”，而是：

- repl/perpetual：environment/work lease 还原链
- standalone：fatal 后收尾退出链
- env-less code session：JWT refresh + transport rebuild 链

### `kg8(...)` 的实际职责

`kg8(...)` 现在可以更准确地称为：

- token refresh scheduler

它有两种调度入口：

- `schedule(sessionId, jwt)`
  - 从 JWT `exp` claim 反推到期时间
- `scheduleFromExpiresIn(sessionId, expires_in)`
  - 直接按 server 返回的秒数调度

其共同行为：

- 默认提前 `300000ms` 刷新
- 若剩余时间已经小于 buffer
  - 立即刷新
- 若完全拿不到新的 OAuth token
  - 最多容忍 3 次失败
  - 每次间隔约 60 秒

这说明 bridge token refresh 不是 transport 内部的附带小逻辑，而是独立的 credential lifecycle 组件。

### 隐藏的 bridge 故障注入入口

发行版里还保留了一个关闭状态的本地 debug tool：

- 名称：`bridge-kick`
- 默认：
  - `isEnabled() === false`

但它暴露出的子命令很有信息量：

- `close <code>`
  - 人工触发 transport close
- `poll <status> [type]`
  - 让下一次 `pollForWork` 抛 `BridgeFatalError`
- `poll transient`
  - 让下一次 poll 抛 axios-style transient error
- `register fail [N]`
  - 让后续 `registerBridgeEnvironment` 暂时失败
- `register fatal`
  - 让 register 直接 `403`
- `reconnect-session fail`
  - 让 `/bridge/reconnect` 返回 `404`
- `heartbeat <status>`
  - 让 heartbeat 抛 fatal
- `reconnect`
  - 直接触发 `reconnectEnvironmentWithSession()`
- `status`
  - 打印当前 bridge debug state

这至少说明两件事：

- 原始产品本地确实考虑过对 bridge recovery 做手工故障注入验证
- 后续如果要做“无真实服务端”的运行时验证，不一定只能从零搭 mock server；也可以优先参考这组 bundle 内建的 fault model

但这里还要保留一个边界：

- 当前已经确定 `bridge-kick` 命令面与预期 handle 形状
  - `fireClose / injectFault / wakePollLoop / forceReconnect / describe`
- 当前发行版文本里，`eJz` 的命中只剩：
  - `var eJz = null`
  - `iL4() { return eJz }`
  - 没看到任何赋值点或 setter
- 因此目前能稳写的是：
  - bundle 确实保留了 bridge fault model
  - 但当前发行版中的 debug handle 实际上处于“命令面保留、注册句柄失联”的状态
  - upstream 源码里是否曾有赋值点，当前只能保留为 build-strip / dead debug residue 级别的猜测

### 不依赖真实服务端的验证矩阵

结合 `bridge-kick` 与当前静态还原链，至少已经能整理出一组本地可验证目标：

- `poll 404`
  - 验 repl bridge 是否进入 environment lost / re-register
  - 同时对照 standalone bridge 是否直接 fatal 退出
- `reconnect-session fail`
  - 验 in-place reconnect 失败后是否按预期退化到 fresh session
- `heartbeat 401/404`
  - 验 `onHeartbeatFatal` 是否清掉当前 `workId/sessionToken`
  - 是否进入 fast-poll 取新 work
- `register fail [N] / register fatal`
  - 验 register retry / terminal fail 的本地预算与收尾路径
- `close <code>`
  - 验 transport close 后的 reconnect / teardown 分支

但仍有几类问题无法靠这套本地故障注入单独坐实：

- `environment_secret` 的真实 TTL / idle timeout / 挤占规则
- `worker_epoch` 的服务端正式命名语义
- 服务端完整 `error.type` 宇宙及其稳定性

### Remote transcript persistence

远程持久化不是无脑 append，而是链式 lastUuid 控制。

### 冲突还原逻辑

当远程写入遇到 `409`：

1. 看 `x-last-uuid`
2. 若与当前 entry UUID 相同 -> 视为幂等成功
3. 若服务端给了新 `lastUuid` -> adopt 后重试
4. 若没给 -> 拉取远端 loglines，取最后 UUID 再重试
5. 最多约 10 次，指数退避

### 设计意义

保证本地与远程 transcript 在并发/重试下尽可能保持链式一致。

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
