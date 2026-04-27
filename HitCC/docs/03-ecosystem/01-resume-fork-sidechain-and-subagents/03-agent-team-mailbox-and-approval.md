# Agent Team 的 Mailbox、审批与生命周期协议

## 本页用途

- 单独整理 Agent Team 的 mailbox protocol，而不是继续和 team roster、backend、task model 混写。
- 把 `SendMessage` 的外显 API、磁盘 inbox envelope、runtime 内部 control schema 明确拆开。
- 补齐 message schema 表和外显 API -> runtime schema 的映射关系。

## 相关文件

- [../01-resume-fork-sidechain-and-subagents.md](../01-resume-fork-sidechain-and-subagents.md)
- [02-agent-team-and-task-model.md](./02-agent-team-and-task-model.md)
- [04-teammate-runtime-and-backends.md](./04-teammate-runtime-and-backends.md)
- [../../02-execution/01-tools-hooks-and-permissions.md](../../02-execution/01-tools-hooks-and-permissions.md)

## 先拆开 3 层：不要再把它们混成一个 schema

当前本地实现里，至少并存 3 层不同结构：

1. `SendMessage` 工具对模型暴露的输入 schema
2. 磁盘 mailbox 文件里的 envelope schema
3. `text` 字段里承载的 runtime control message schema

这三层高度相关，但绝不是同一个对象。

## 第 1 层：`SendMessage` 对外暴露的输入 schema

`SendMessage` 工具输入当前可直接还原为：

```ts
type SendMessageInput = {
  to: string
  summary?: string
  message: string | LegacyStructuredProtocolMessage
}

type LegacyStructuredProtocolMessage =
  | { type: "shutdown_request"; reason?: string }
  | { type: "shutdown_response"; request_id: string; approve: boolean; reason?: string }
  | { type: "plan_approval_response"; request_id: string; approve: boolean; feedback?: string }
```

### 对外输入约束

- `message` 是字符串时：
  - `summary` 必填
  - `to` 可以是具体 teammate 名，也可以是 `"*"`
- `message` 是 structured object 时：
  - 不允许 `to: "*"`
  - `shutdown_response` 只能发给 `team-lead`
  - 拒绝 `shutdown_response` 时必须带 `reason`

### `backfillObservableInput(...)` 暴露了工具层的“可观察形态”

`SendMessage` 不只校验输入，还会把外显 API 归一化成一套更适合 telemetry / observable input 的形状：

| 外显输入 | 归一化后字段 |
| --- | --- |
| `to="*"` + plain text | `type="broadcast"` + `content=message` |
| `to="<name>"` + plain text | `type="message"` + `recipient=to` + `content=message` |
| structured message | `type=message.type` + `recipient=to` + 可选 `request_id / approve / content` |

这里的 `content` 规则是：

- plain text -> 原始文本
- structured message -> `reason ?? feedback`

因此外层 classifier / transcript 可见的“SendMessage 输入”已经不是原始 schema，而是一个观测用投影。

## 第 2 层：磁盘 mailbox 的 envelope schema

mailbox 不是抽象队列，而是：

```text
~/.claude/teams/<team>/inboxes/<agent>.json
```

文件内容是 JSON array，不是 JSONL。

### 单条 inbox envelope

写盘时真实追加的是：

```ts
interface MailboxEnvelope {
  from: string
  text: string
  timestamp: string
  color?: string
  summary?: string
  read: boolean
}
```

其中：

- `read` 写入时固定补成 `false`
- structured protocol message 不会单独占字段，仍然塞进 `text`
- `text` 的内容要么是 plain text，要么是 `JSON.stringify(protocolMessage)`

### 读写语义

- `readMailbox`：直接读整个数组
- `readUnreadMessages`：按 `!message.read` 过滤
- `markMessageAsReadByIndex`：按 index 改单条 `read: true`
- `markMessagesAsRead`：把整份 inbox 全部改成 `read: true`
- `writeToMailbox` / `markMessageAsReadByIndex` / `markMessagesAsRead` / `markMessagesAsReadByPredicate` 都带 lockfile
- `readMailbox` / `readUnreadMessages` 只是普通读文件，当前**不带 lock**

因此 mailbox runtime 的第一个硬边界是：

- **磁盘上只有 envelope**
- **真正的 protocol schema 是 envelope.text 里的 JSON**

### 已确认的标读竞态

当前这套 mailbox 实现里，已经能直接确认一条“快照读取”和“全量标读”之间的竞态窗口：

1. `G$6(...)` 先调用 `Or(...)` 无锁读取 inbox，并按 `!read` 产出 unread 快照。
2. runtime 在内存里对这份快照做分类、消费、回灌或 cleanup。
3. 最后 `No6(...)` 再次读取**当前磁盘文件**，并把此刻所有 unread 一次性改成 `read: true`。

这意味着只要有新消息在“`G$6(...)` 返回之后、`No6(...)` 真正拿锁并重读之前”写入 inbox，它就会：

- 不在本轮快照里
- 不会被本轮分类 / 消费 / 投递
- 却仍可能被 `No6(...)` 当作“当前 unread”直接标成已读

这已经不是文档缺口，而是当前 bundle 下静态可达的实现风险。

### 两条受影响路径

#### 1. 主 inbox poller：`WQ4(...)`

这条路径的真实时序是：

1. `G$6(P, teamName)` 拿 unread 快照
2. 在内存里完成分类、审批副作用、transcript materialize
3. 通过局部函数 `D()` 调 `No6(P, teamName)`

这里还有两个会放大窗口的实现细节：

- `D()` 内部只是调用 `No6(...)`，**没有 `await`**
- `WQ4(...)` 本身通过 `GD(() => void j(), 1000)` 以 fire-and-forget 方式周期轮询

因此主 poller 不是“处理完当前快照再原子标读”，而是：

- 先无锁取快照
- 处理中允许其他 writer 继续 append
- 末尾异步发起一次“把当前所有 unread 全量标读”

如果新消息恰好落在这个窗口里，它可以被直接吞掉。

#### 2. leader `print.ts` 旁路

leader 侧旁路也有同样的基础模式：

1. `G$6("team-lead", teamName)` 先拿 unread 快照 `J6`
2. 若 `J6.length > 0`，立即 `await No6("team-lead", teamName)`
3. 然后只遍历旧快照 `J6` 做 `shutdown_approved` cleanup 与 prompt 回灌

这条路径虽然 `await` 了 `No6(...)`，但问题仍然存在，因为 `No6(...)` 标读的是**当前文件里的所有 unread**，而后续处理的却只是更早拿到的旧快照 `J6`。  
所以新消息若落在 `G$6(...)` 与 `No6(...)` 之间，依然可能“被标读，但不在 `J6` 里，因此本轮完全没处理”。

### 不受这条竞态影响的相邻路径

in-process teammate 的等待循环 `$j_()` 不是这套“快照后全量标读”模型：

- 它先 `Or(...)` 读整份 inbox
- 选中一个具体 index
- 再用 `ko6(...)` 只把该 index 标成 read

因为 `writeToMailbox(...)` 只会 append，新消息只会加到数组尾部，所以 `$j_()` 的主要风险不是“新消息被 `mark all` 吞掉”，而是普通轮询延迟。

## 第 3 层：runtime 内部 control message schema

`KL8(...)` 明确识别的 control/protocol family 当前有：

- `permission_request`
- `permission_response`
- `sandbox_permission_request`
- `sandbox_permission_response`
- `shutdown_request`
- `shutdown_approved`
- `team_permission_update`
- `mode_set_request`
- `plan_approval_request`
- `plan_approval_response`

注意：

- `shutdown_rejected` 明明存在 parse schema，但 **不在** `KL8(...)` 的 control family 判定里。
- `idle_notification` 会被单独 parse，但也 **不在** `KL8(...)` 里。
- `task_assignment` 通过 mailbox 发送，但明确不属于 control family。

## message schema 总表

下面这张表只写当前本地 bundle 下能直接确认的字段。

| 类型 | schema | 创建侧 | 消费侧 | 备注 |
| --- | --- | --- | --- | --- |
| `idle_notification` | `{ type, from, timestamp, idleReason?, summary?, completedTaskId?, completedStatus?, failureReason? }` | `QIq(...)`；另有 teammate init 的 `Stop` hook 也会发 | `Eo6(...)` / `FA4(...)` / transcript renderer filter | protocol-adjacent；会进 transcript/attachment，但不在 `KL8` 里 |
| `permission_request` | `{ type, request_id, agent_id, tool_name, tool_use_id, description, input, permission_suggestions[] }` | worker -> leader | `yo6(...)` / leader inbox poller | snake_case |
| `permission_response` | success: `{ type, request_id, subtype:"success", response:{ updated_input?, permission_updates? } }`; error: `{ type, request_id, subtype:"error", error }` | leader -> worker | `v$6(...)` / `RN6(...)` | snake_case |
| `sandbox_permission_request` | `{ type, requestId, workerId, workerName, workerColor?, hostPattern:{ host }, createdAt }` | worker -> leader | `oy8(...)` / leader inbox poller | camelCase |
| `sandbox_permission_response` | `{ type, requestId, host, allow, timestamp }` | leader -> worker | `Lo6(...)` / `mIq(...)` | camelCase |
| `shutdown_request` | `{ type, requestId, from, reason?, timestamp }` | leader or teammate | `mA6(...)` / inbox poller / model-facing handoff | 真正 runtime request |
| `shutdown_approved` | `{ type, requestId, from, timestamp, paneId?, backendType? }` | teammate approval path | `zT(...)` / leader inbox poller | 不是 `shutdown_response` |
| `shutdown_rejected` | `{ type, requestId, from, reason, timestamp }` | `SendMessage(shutdown_response approve=false)` -> `Pg1(...)` | `sy8(...)` / `mA4(...)` / `BA4(...)` / `FA4(...)` | 当前只看到 render/summary 侧消费，未看到 leader runtime cleanup consumer |
| `plan_approval_request` | `{ type, from, timestamp, planFilePath, planContent, requestId }` | teammate `ExitPlanMode` | `T$6(...)` / leader inbox poller | camelCase `requestId` |
| `plan_approval_response` | `{ type, requestId, approved, feedback?, timestamp, permissionMode? }` | leader -> teammate | `EN6(...)` / teammate inbox poller | runtime 不是 `approve`，而是 `approved` |
| `mode_set_request` | `{ type, mode, from }` | leader -> teammate | `qL8(...)` / inbox poller | 向下同步 permission mode |
| `team_permission_update` | 仅有 loose parse：至少要求 `{ type, toolName, directoryPath, permissionUpdate:{ rules, behavior } }` | 当前 bundle **未找到** object creator / serializer / zod schema | `ey8(...)` / inbox poller `AY(addRules)` | 更像遗留/未启用 consumer；当前 team 级权限同步主路径反而是 `teamAllowedPaths` + teammate init 重放 |
| `task_assignment` | `{ type, taskId, subject, description, assignedBy, timestamp }` | `TaskUpdate(owner=...)` | `ty8(...)` / UI | coordination message，不是 control family |

## `SendMessage` 外显 API 与 runtime schema 的映射

这是当前最容易写错的地方。

### 1. plain text message

外显输入：

```json
{"to":"researcher","summary":"assign task 1","message":"start on task #1"}
```

落盘 envelope：

```ts
{
  from: senderName,
  text: "start on task #1",
  summary: "assign task 1",
  timestamp,
  color,
  read: false
}
```

没有额外 runtime control schema。

### 2. `shutdown_request`

外显输入：

```json
{"to":"worker-a","message":{"type":"shutdown_request","reason":"task complete"}}
```

runtime 落盘 payload：

```ts
{
  type: "shutdown_request",
  requestId,
  from,
  reason,
  timestamp
}
```

这是近似直通，但 runtime 会补：

- `requestId`
- `from`
- `timestamp`

### 3. `shutdown_response` 不是 runtime 真正落盘类型

外显输入：

```json
{"to":"team-lead","message":{"type":"shutdown_response","request_id":"...","approve":true}}
```

runtime 实际分两路：

- `approve=true`
  - 转成 `shutdown_approved`
- `approve=false`
  - 转成 `shutdown_rejected`

也就是：

| 对外字段 | runtime 字段 |
| --- | --- |
| `request_id` | `requestId` |
| `approve=true` | `type="shutdown_approved"` |
| `approve=false` | `type="shutdown_rejected"` |
| `reason` | reject 时的 `reason` |

批准路径还会额外补：

- `paneId?`
- `backendType?`

因为批准 shutdown 不只是逻辑 ACK，还要给 leader 侧提供真正终止 pane / in-process teammate 所需的信息。

### 4. `plan_approval_response`

外显输入：

```json
{"to":"researcher","message":{"type":"plan_approval_response","request_id":"...","approve":false,"feedback":"add error handling"}}
```

runtime 落盘 payload：

```ts
{
  type: "plan_approval_response",
  requestId,
  approved,
  feedback?,
  timestamp,
  permissionMode?
}
```

字段映射：

| 对外字段 | runtime 字段 |
| --- | --- |
| `request_id` | `requestId` |
| `approve` | `approved` |
| `feedback` | `feedback` |
| 无 | `timestamp` |
| 无 | `permissionMode?` |

其中 `permissionMode` 只有 leader 批准 plan 时才会补上。

## inbox poller 的真实消费链

inbox poller 不是“收到消息就直接塞 transcript”，而是先做 protocol 分类。

### 分类桶

当前主路径把 unread mailbox message 分成：

- `permission_request`
- `permission_response`
- `sandbox_permission_request`
- `sandbox_permission_response`
- `shutdown_request`
- `shutdown_approved`
- `team_permission_update`
- `mode_set_request`
- `plan_approval_request`
- 其他消息

### 处理优先级

当前主路径更准确地说是：

1. teammate 路径先单独扫 `plan_approval_response`
   - 若来自 `team-lead`，会先本地改 mode / 清 `awaitingPlanApproval`
2. 再把 unread mailbox 分类成各个桶
3. 一部分 control message 先触发本地副作用
   - 例如更新 permission queue、切 mode、更新 teammate state、auto-approve plan
4. 最后只有 `C` 桶里的消息会被 materialize 成 transcript / inbox item
5. 无论是否进 `C`，本轮 unread 最后都会被 `No6(...)` 统一标记为 read

因此 mailbox 不是简单“所有消息都喂给模型”，但也不是“被 runtime 处理过的 control message 就绝不再进 transcript”。

更稳的结论应写成：

- **runtime 会先尝试消费控制协议并更新本地状态**
- **只有部分消息会继续 materialize 给 session**
- **还有一部分消息会被纯本地吞掉，然后直接标记已读**

### 哪些类型一定本地吞掉，哪些还会继续投递

| 类型 | 本地副作用 | 是否进 `C` / transcript | 备注 |
| --- | --- | --- | --- |
| `permission_request` | leader UI queue / callback 注册 | 否 | 纯审批输入 |
| `permission_response` | worker callback `RN6(...)` | 否 | 纯审批回包 |
| `sandbox_permission_request` | leader network permission queue | 否 | 纯审批输入 |
| `sandbox_permission_response` | worker callback `mIq(...)` | 否 | 纯审批回包 |
| `team_permission_update` | `AY(addRules)` 改 session permission context | 否 | 当前只见 consumer |
| `mode_set_request` | `AY(setMode)` + `uN6(...)` 回写 roster mode | 否 | 纯状态同步 |
| `plan_approval_response` | teammate 先本地退出/维持 plan mode | 是 | **先消费再继续投递**，因此两边都走 |
| `plan_approval_request` | leader 路径会 auto-approve 并写回 response | 仅 leader 路径会进 `C` | 当前实现有不对称；非 leader 命中此桶时不会落入 `C` |
| `shutdown_request` | inbox poller 本身无本地副作用 | 是 | 交给后续 session / model 感知 |
| `shutdown_approved` | leader cleanup / pane kill / roster 删除 | 是 | **先 cleanup 再继续投递** |
| `shutdown_rejected` | 无专门 runtime 副作用 | 是 | 靠 catch-all 进入 transcript，再由 renderer 解释 |
| `idle_notification` | 无专门 runtime 副作用 | 是 | 靠 catch-all 进入 transcript / attachment |
| `task_assignment` | 无专门 runtime 副作用 | 是 | coordination message |
| plain text DM | 无 | 是 | 普通 mailbox 文本 |

### 还有一条 leader 侧旁路：`print.ts`

除了 `WQ4(...)` 这条 inbox poller，leader 在主打印循环里还有一条专门的 team-lead inbox 轮询：

- 它会反复扫 `team-lead` inbox
- 命中 unread 后先 `No6("team-lead", ...)` 一次性全部标读
- 其中只对 `shutdown_approved` 做额外 cleanup
- 其他 unread 统一包装成 teammate message prompt 回灌当前会话

这意味着：

- `shutdown_approved` 不只在 `WQ4(...)` 里有 consumer
- `shutdown_rejected`、`idle_notification` 在这条旁路里**没有**专门 runtime consumer，只会作为普通结构化消息继续回灌

## 权限上卷链：worker -> leader -> worker

### tool permission

链路当前已能写成：

```text
worker tool wants permission
  -> _L8(...) build pending request object
  -> zL8(...) serialize to permission_request
  -> writeToMailbox(leader)
  -> leader inbox poller parses yo6(...)
  -> leader UI queue / callback
  -> YL8(...) serialize to permission_response
  -> writeToMailbox(worker)
  -> worker inbox poller parses v$6(...)
  -> RN6(...) resolve registered callback
```

### sandbox/network permission

链路等价，只是消息类型换成：

- `sandbox_permission_request`
- `sandbox_permission_response`

worker 侧 callback registry 也分成另一套：

- `xIq(...)`
- `mIq(...)`

因此可以明确下结论：

- teammate 没有最终 permission authority
- authority 默认上卷到 leader
- mailbox 不是通知层，而是审批闭环本体

## `team_permission_update` 当前更像遗留 consumer，而不是活协议

这部分现在可以比“还没找到创建侧”更收紧一点。

- `ey8(...)` 只是 `JSON.parse` 后检查 `type==="team_permission_update"`，没有对应 zod schema
- inbox poller 进一步只硬检查：
  - `permissionUpdate.rules`
  - `permissionUpdate.behavior`
- 当前 bundle 内没有找到任何 `type:"team_permission_update"` 的对象构造、serializer 或发送点
- 当前真正可见的 team 级权限同步路径反而是：
  - team file 里可选的 `teamAllowedPaths[]`
  - teammate 启动时 `b5A(...)` 读取它
  - 再把每条 `{ toolName, path }` 转成 session `addRules`

因此更稳的判断是：

- `team_permission_update` 在当前发行版里更像**遗留 / 预留 consumer**
- 真正活跃的 team 级权限分发，更像**持久化到 team file，再在 teammate init 时重放**
- `teamAllowedPaths` 自身当前也只看到读取侧，没看到明确写入侧；所以这块仍要保守写成“现存读路径”，不要过度假设完整闭环

## plan approval 链：teammate -> leader -> teammate

### request 生成点

`ExitPlanMode` 在 teammate 场景下不会直接退出 plan，而是：

1. 读当前 plan file
2. 构造：

```ts
{
  type: "plan_approval_request",
  from,
  timestamp,
  planFilePath,
  planContent,
  requestId
}
```

3. 写到 `team-lead` inbox
4. 当前 tool result 返回：
   - `awaitingLeaderApproval: true`
   - `requestId`

### leader 当前主路径是 auto-approve

leader inbox poller 当前看见：

- `Found X plan approval request(s), auto-approving`

并且直接回：

```ts
{
  type: "plan_approval_response",
  requestId,
  approved: true,
  timestamp,
  permissionMode
}
```

其中：

- `permissionMode = leader 当前 mode`
- 若 leader 当前 mode 是 `plan`，会降成 `default`

当前本地 bundle 里，没有看到额外的人审或条件 gate。

## idle 与 shutdown：都不是普通文本消息

### `idle_notification`

当前行为：

- teammate 每个 turn 结束会进入 idle
- idle 时仍能收消息
- 会自动向 leader 发 `idle_notification`
- `summary` 默认来自最近一次 `SendMessage` peer DM 摘要提取

这说明 idle notification 的语义不是“日志”，而是：

- teammate turn boundary signal
- collaboration summary 回传

但它在 runtime 里的落点比“控制协议”更偏旁路：

- 有专门 parse：`Eo6(...)`
- 有专门 summary string：`cI_(...)`
- `FA4(...)` 会把它格式化成 `"Agent idle"` 风格摘要
- transcript 行 renderer 会主动隐藏它的正文
- teammate mailbox attachment renderer 也会把它过滤掉
- 它不在 `KL8(...)`，也没有专门 state-mutation consumer

因此更准确的归类应是：

- **structured mailbox message**
- **protocol-adjacent**
- **不是 control family 成员**

### shutdown

shutdown 不是 kill instruction，而是 mailbox 协议：

```text
shutdown_request
  -> shutdown_approved | shutdown_rejected
  -> leader side cleanup / pane kill / task state update
```

批准路径里还有一层运行位置信息补充：

- `paneId?`
- `backendType?`

说明 leader 在收到 approval 后会继续做 backend-specific termination。

而 reject 路径当前明显不对称：

- `shutdown_rejected` 有独立 schema 与发送函数
- 但不在 `KL8(...)`
- inbox poller / `print.ts` 都没有专门 cleanup consumer
- 当前只看到 render/summary consumer

所以更稳的说法不是“只是还没追到另一入口”，而是：

- **当前 bundle 下，reject 更像 transcript-visible status message**
- **approve 才会触发真正的 runtime cleanup**

## `task_assignment` 与 control family 的边界

`TaskUpdate(owner=...)` 会额外发：

```ts
{
  type: "task_assignment",
  taskId,
  subject,
  description,
  assignedBy,
  timestamp
}
```

但它只是 coordination message，不属于 control family。

因此 mailbox 至少要分成两层理解：

1. control/protocol
   - permission / sandbox / shutdown / mode / plan approval
2. business/coordinator
   - plain text DM
   - `task_assignment`
   - 某些 idle 通知

## `SendMessage` 与“本地还原已停止 agent”之间的边界

`SendMessage` 并不总是写 mailbox。

### 先看本地优先分流

当满足以下条件时：

- `message` 是 plain text
- `to !== "*"`
- `to` 能在 `agentNameRegistry` 里解析到本地 `local_agent`

则 `SendMessage` **优先不走 mailbox**：

- 若该 `local_agent` 仍在 running
  - 直接 `mO4(...)` 追加到 `pendingMessages`
  - 等它下一次 tool round 自己消费
- 若该 `local_agent` 已 stopped / completed / failed
  - 直接 `ka1(...)` 从 transcript 还原为后台 agent
- 若 registry 还在，但 task 已不在前台 state
  - 也尝试 `ka1(...)` 从 transcript 还原

这里走的是：

- 本地 task state
- 本地 transcript
- background local agent runtime

而不是 team mailbox。

### 哪些情况才稳定走 mailbox

下面这些仍然会写 inbox：

- 普通 teammate plain text DM
- team broadcast
- `shutdown_request`
- `shutdown_response` 映射后的 `shutdown_approved` / `shutdown_rejected`
- `plan_approval_response`

也就是说：

- `agentNameRegistry` 命中的**本地 background/local agent**，优先走“还原本地 agent”
- team teammate 协作消息，优先走 mailbox

这也是为什么 in-process teammate 自己的等待循环 `$j_()` 仍然是 mailbox 驱动：

- 它消费的是 `pendingUserMessages` + mailbox unread + task auto-claim
- 但 `SendMessage` 本身并不会把普通 teammate DM 直接塞进 `pendingUserMessages`
- 普通 teammate DM 仍是先写 inbox，再由 `$j_()` 轮询取出

## 当前最稳的还原接口

如果按“可以复刻”的目标写，当前 mailbox/runtime 最稳的三层接口应是：

### 1. 工具输入

```ts
interface SendMessageToolInput {
  to: string
  summary?: string
  message: string | LegacyStructuredProtocolMessage
}
```

### 2. 磁盘 inbox

```ts
interface MailboxEnvelope {
  from: string
  text: string
  timestamp: string
  color?: string
  summary?: string
  read: boolean
}
```

### 3. runtime control payload

```ts
type RuntimeMailboxPayload =
  | IdleNotification
  | PermissionRequest
  | PermissionResponse
  | SandboxPermissionRequest
  | SandboxPermissionResponse
  | ShutdownRequest
  | ShutdownApproved
  | ShutdownRejected
  | PlanApprovalRequest
  | PlanApprovalResponse
  | ModeSetRequest
  | TeamPermissionUpdate
  | TaskAssignment
```

## 当前仍需保守表述的点

- `team_permission_update`
  - 当前已经可以基本判定：本地 bundle 里没有独立 producer / serializer / zod schema。
  - 但还不能证明它一定“永远不会被发送”；更保守的写法仍应是**当前发行版未见活跃发送路径**。
- `teamAllowedPaths`
  - 只看到 teammate init 读取并转成 session allow rules。
  - 没看到明确写入侧，因此不要把它写成完整、已证实的全链路协议。
- `plan_approval_request`
  - 当前在 leader 路径会 auto-approve 并继续投递 transcript。
  - 若它意外到达非 leader 会话，现实现看起来会被分类后直接标读，不会进入 `C`；这是一个真实的不对称边界。

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
