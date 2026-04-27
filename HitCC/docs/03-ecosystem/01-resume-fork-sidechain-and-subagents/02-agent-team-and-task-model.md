# Agent Team、TaskList 与 Roster 模型

## 本页用途

- 单独整理 Agent Team 的协作模型、生命周期与任务分配结构。
- 重点收紧 `Team = TaskList`、roster 全字段、task 状态机、owner 语义边界，以及 `config.json` / task list 的 source-of-truth。

## 相关文件

- [../01-resume-fork-sidechain-and-subagents.md](../01-resume-fork-sidechain-and-subagents.md)
- [01-resume-fork-sidechain-and-subagent-core.md](./01-resume-fork-sidechain-and-subagent-core.md)
- [03-agent-team-mailbox-and-approval.md](./03-agent-team-mailbox-and-approval.md)
- [04-teammate-runtime-and-backends.md](./04-teammate-runtime-and-backends.md)

## Team = TaskList

`TeamCreate` prompt 已直接写明：

- `Teams have a 1:1 correspondence with task lists (Team = TaskList)`

实现侧创建 team 时，会同时建立：

- `~/.claude/teams/{team-name}/config.json`
- `~/.claude/tasks/{team-name}/`

这里有一处 prompt / 实现冲突需要明确分开：

- `TeamCreate` prompt 中残留了 `~/.claude/teams/{team-name}.json`
- 但真实路径函数 `YU(teamName)` 返回的是：
  - `~/.claude/teams/{team-name}/config.json`
- `readTeamFile / writeTeamFile` 也都实际读写 `config.json`

因此当前应把 Team 理解成：

- 一个 team root 目录
- 一个共享 task list 目录
- 一个由 `config.json` 持久化的 roster / coordination 状态

而不是单纯“会话分组”。

## `config.json` 才是 team source-of-truth

需要把三层东西分清：

1. `~/.claude/teams/{team}/config.json`
   - 持久化真相
2. `AppState.teamContext`
   - leader / teammate 当前进程内的运行时镜像
3. `AppState.teamContext.teammates`
   - 更偏 UI / bookkeeping cache，不是持久化真相

实现上，spawn、reconnect、teammate init、cleanup、mode 变更、active 标记都会回读或回写 `config.json`。  
`hiddenPaneIds` 相关 helper 也会写 team file，但当前发行版里 UI hide/show 动作链本身看起来没有真正接到这些 helper。  
因此真正的 team source-of-truth 不是内存态，而是 team file。

### `AppState.teamContext` 是部分镜像，不是 team file 的等价缓存

当前已经可以把 `teamContext` 的还原边界写得更细：

- 初始 app state 里的 `teamContext` 来自 `fU4()`
  - 它只根据 dynamic teammate context + team file 填：
    - `teamName`
    - `teamFilePath`
    - `leadAgentId`
    - `selfAgentId`
    - `selfAgentName`
    - `isLeader`
  - 但会把 `teammates` 直接置成空对象
- REPL mount 后 `GU4()` 再走一次：
  - 若 resume 数据里有 `teamName / agentName`
    - 调 `wU4()` 用 team file 重建 teammate 视角的 `teamContext`
  - 否则退回 dynamic teammate context
  - `wU4()` 同样只重建 team 元信息，`teammates` 仍然先置空
- 真正把 `teamContext.teammates` 填起来的，是后续 live runtime 路径：
  - leader 侧 spawn teammate
  - kill / shutdown / remove 时的内存删项
  - 其他运行时 bookkeeping

因此当前更准确的说法应是：

- `config.json` 才是完整 roster 真相
- `AppState.teamContext` 更像 process-local coordination mirror
- `AppState.teamContext.teammates` 尤其只是运行中 cache，不保证从磁盘完整回放

### Team 根对象字段

当前本地 bundle 已直接看到或可从活路径确认的 team file 顶层字段如下。

| 字段 | 语义 | 证据边界 |
| --- | --- | --- |
| `name` | team 名 | `TeamCreate` 初始写入 |
| `description` | team 描述 | `TeamCreate` 初始写入，可为空 |
| `createdAt` | 创建时间戳 | `TeamCreate` 初始写入 |
| `leadAgentId` | leader 的 agentId | `TeamCreate` 初始写入；teammate init / reconnect 会回读 |
| `leadSessionId` | leader 所在 sessionId | `TeamCreate` 初始写入 |
| `members` | 平铺 roster 数组 | 所有 spawn / remove / mode / active 更新都围绕它 |
| `teamAllowedPaths` | team 级 permission 规则数组 | teammate init 会读取并转换成 session allow rules；当前 bundle 里还没找到明确 writer |
| `hiddenPaneIds` | UI 隐藏 pane 列表 | helper 可读写；当前发行版上层 hide/show 动作链疑似未真正接线 |

`teamAllowedPaths`、`hiddenPaneIds` 不是 `TeamCreate` 初始必写字段，但已经是本地活实现会消费/维护的 team-level 持久化字段，不能再视作文档外扩展。

### `members[]` 字段全集

`members` 是 flat roster，没有嵌套 team-of-team 结构。  
当前活路径里出现过的成员字段可收敛为下面这一组。

| 字段 | 语义 | 出现方式 |
| --- | --- | --- |
| `agentId` | 成员内部唯一标识 | 初始 leader / spawn teammate 都写入 |
| `name` | 对外通信与任务 owner 的名字 | 初始 leader / spawn teammate 都写入 |
| `agentType` | agent 角色/类型 | 初始 leader / spawn teammate 都写入 |
| `model` | teammate 实际 model | leader 初始成员、spawn teammate 都可写 |
| `prompt` | spawn 时的初始 prompt | spawn teammate 写入；leader 初始成员没有 |
| `color` | UI / mailbox 色彩 | spawn teammate 写入；leader 初始成员不一定持久化 |
| `planModeRequired` | 是否要求 plan approval | spawn teammate 写入 |
| `joinedAt` | 加入时间戳 | leader 初始成员 / spawn teammate 都写入 |
| `tmuxPaneId` | pane/in-process 位置标记 | leader 初始成员、spawn teammate 都写入 |
| `cwd` | 成员工作目录 | leader 初始成员 / spawn teammate 都写入 |
| `subscriptions` | 订阅列表 | 初始成员 / spawn teammate 初始为空数组；当前 bundle 里未见后续 consumer |
| `backendType` | `tmux` / `iterm2` / `in-process` | spawn teammate 写入 |
| `mode` | 成员 mode | 运行时通过 `setMemberMode` / batch mode update 回写 |
| `isActive` | 运行时 active/idle 近似位 | turn start / idle-stop 路径会回写；缺省值也会被当作 active |
| `worktreePath` | worktree 隔离目录 | cleanup 和 teammate UI 会读取；当前 teammate spawn 路径里未见 writer |

需要特别强调两条边界：

- **外部寻址字段只有 `name`。**
  - `SendMessage(to=...)` 用它
  - `TaskUpdate(owner=...)` 也应该用它
- `agentId` 是内部兼容标识，不是日常协作地址。

### roster 的活变更路径

本地 bundle 可见的成员变更不仅有 spawn/remove：

- `TeamCreate`
  - 写入 leader 自己这一条初始 member
- pane / in-process teammate spawn
  - `members.push(...)` 写入新 teammate
- remove teammate
  - 可按 `agentId`
  - 可按 `name`
  - 可按 `tmuxPaneId`
- active 标记
  - `Bo6(team, memberName, isActive)` 回写 `isActive`
- mode 标记
  - `uN6 / rg1` 回写 `mode`
- cleanup
  - 会读取 `backendType / tmuxPaneId / worktreePath`

这说明 `members[]` 不是“spawn 清单”而已，而是完整 roster runtime 的持久化承载。

### `isActive` 的语义边界比“可删除”更弱

`isActive` 当前可以写得比“成员是否存活”更精确：

- 查询主链 `bT(...)` 开始一轮 teammate query 时，会调用 `Bo6(team, memberName, true)`
- teammate 的 `Stop` hook 会调用 `Bo6(team, memberName, false)`
- teammates UI `Fg4()` 把：
  - `isActive !== false`
  - 直接显示成 `running`
- `TeamDelete` 也是按：
  - `members.filter(name !== "team-lead").filter(isActive !== false)`
  - 只要命中就拒绝 cleanup

但同时还要注意一个很重要的默认值边界：

- `TeamCreate` 初始 leader member 不写 `isActive`
- pane / in-process teammate spawn 写入 member 时也不写 `isActive`

因此当前实现里：

- `isActive === false`
  - 才是明确 idle / 可忽略
- `isActive` 缺失
  - 会被 UI 和 `TeamDelete` 一律按 active 处理

所以更稳的结论是：

- `isActive` 是**runtime active/idle 的弱语义位**
- 不是“该成员可安全删除”的强一致状态位

### roster 字段的 producer-consumer 闭环边界

到这里可以再把几个最容易误判的字段拆开。

| 字段 | 已确认 producer | 已确认 consumer | 当前最稳判断 |
| --- | --- | --- | --- |
| `teamAllowedPaths` | 当前 bundle 未找到明确写回 `config.json` 的活路径 | teammate startup init `B5A()` 读取并转成 session allow rules | **consumer 已钉住，writer 仍未闭环** |
| `subscriptions` | `TeamCreate`、pane spawn、in-process spawn 都写 `[]` | 当前 bundle 未看到 roster 读侧 consumer | **更像预留/兼容字段，不宜过度解读** |
| `worktreePath` | 当前 teammate spawn 路径未见写入 `members[]` | `GL8()` cleanup 会删 member worktree，teammate UI `Fg4()` / 详情卡会展示 | **consumer 活着，但 roster writer 仍未钉死** |

这里有两个边界要明确：

- `teamAllowedPaths` 和 mailbox 里的 `team_permission_update` 不是同一层。
  - 前者是 team file 持久化字段
  - 后者当前只看到 teammate 进程在 inbox poller 里把规则加进本 session allow rules，还没看到它反写 `config.json`
- `worktreePath` 和 `Agent` 工具的 `isolation: "worktree"` 也不是一回事。
  - `Agent` 工具的 worktree 结果当前明确落在普通 subagent 路径
  - 该结果会进入 subagent 返回值 / transcript / resume 数据
  - 但当前没看到它自动同步进 team roster 的 `members[].worktreePath`

### `hiddenPaneIds` 当前更像可读的残留持久化字段

`hiddenPaneIds` 这一项现在也能写得更硬。

已确认的 helper / consumer 有：

- `Lj_(team, paneId)`
  - 追加 `hiddenPaneIds`
- `hj_(team, paneId)`
  - 移除 `hiddenPaneIds`
- `lg1(team, paneId)`
  - remove member 时会顺手把同 paneId 从 `hiddenPaneIds` 清掉
- `cg1(team, paneId)`
  - 被 teammates UI `Fg4()` 读取，映射成 `isHidden`
- `GL8(team)`
  - `TeamDelete` 最终直接删整个 team directory，因此 `hiddenPaneIds` 也一起消亡

但当前还存在一个比“闭环未补齐”更强的边界：

- TeamsDialog 的 `h / H` 快捷键入口仍然存在
- `_Lz()` 仍会分流到 `GG4()` / `VG4()`
- tmux backend 也已经实现 `hidePane()` / `showPane()`
- 但 `GG4()` / `VG4()` 在当前 bundle 里函数体为空
- 全 bundle 也没看到任何活调用点去调用 backend `hidePane()` / `showPane()`
- 同时也没看到 `lj_()` / `HJ_()` 的活调用者

因此当前最稳判断应是：

- `hiddenPaneIds` 字段本身还活着
- tmux backend 的 hide/show 能力也还活着
- UI 展示和帮助文案也还在
- 但**当前发行版的上层 hide/show 动作链大概率已经断线**

换句话说，当前 `hiddenPaneIds` 更像：

- 一个还能被读取展示、在 remove/delete 时被清理的持久化残留字段
- 而不是已经证实可由现行 UI 正常驱动的完整活功能

### `teamContext` 与 `config.json` 的漂移边界

把 live 路径并排后，可以直接看出两层 state 并不等价。

| 路径 | `teamContext` | `config.json` | 当前边界 |
| --- | --- | --- | --- |
| `fU4()` 初始 app state | 重建 team 元信息，`teammates = {}` | 只读 | **只还原骨架，不还原完整 roster cache** |
| `wU4()` reconnect/init | 重建 team 元信息，`teammates = {}` | 只读 | **teammate 视角的内存镜像依旧是空 roster 起步** |
| pane / in-process spawn | 追加 leader 内存态 teammate cache | `members.push(...)` 持久化 | **两边都写，但字段集合不对称** |
| `HL8()` in-process kill | 先删 `teamContext.teammates[agentId]` | 再 `removeMemberByAgentId()` | **删内存与删文件是分步完成的** |
| TeamsDialog `q5A()` kill | 先 `killPane()`，再删内存 teammate cache | `removeMemberFromTeam()` + reclaim | **pane kill 路径同样分步** |
| inbox poller / print.ts 处理 shutdown | 删内存 teammate cache | 删 team file member + reclaim task | **协议回收也同时改两层，但顺序不一** |
| `TeamDelete` | 最后整体清空 `teamContext` | `GL8()` 删除 team directory | **最终清理是整队删除，不是精细同步** |

这里还有一条很容易误判的边界：

- `teamContext.teammates[...]` 里的字段是：
  - `name / agentType / color / tmuxSessionName / tmuxPaneId / cwd / spawnedAt`
- `config.json.members[]` 里的字段则还包含：
  - `model / prompt / planModeRequired / subscriptions / backendType / mode / isActive / worktreePath`

因此当前内存镜像不仅可能与磁盘态不同步，而且**本来就是一份更窄、更 UI/bookkeeping 导向的投影**。

## `Agent` 工具有 teammate 分支，不等于普通 subagent

从 `sdk-tools.d.ts` 可以确认，`AgentInput` 除了普通 subagent 参数外，还额外带：

- `name`
- `team_name`
- `mode`
- `isolation`

并且注释明确写了：

- spawned agent 可以通过 `SendMessage({to: name})` 寻址

这说明 `name` 在 team 场景不是展示字段，而是 mailbox address。

另外 bundle 里还能确认几个硬约束：

- teammate roster 是 flat 的
- teammate 不能继续 spawn teammate
- 如果要 spawn 普通 subagent，需要省略 `name`
- in-process teammate 不能再 spawn background agent

因此要明确区分两类东西：

1. 普通 subagent
   - 更像 sidechain worker
   - 通过 transcript / background task 还原
2. teammate
   - 加入某个 team
   - 通过 team mailbox + shared task list 协作
   - 生命周期受 team file、teamContext 与 leader 约束

## 共享 TaskList 是第二个 source-of-truth

`TaskCreate / TaskGet / TaskList / TaskUpdate` 在 team 场景下直接作用于当前 team 的 task list。  
因此 team 的持久化真相并不是只有 `config.json`，还包括：

- `~/.claude/tasks/{team-name}/`

这里存的是共享任务对象，`TaskList` 只是其视图，不是另起一套缓存。

### task 对象字段全集

共享 task 的 schema 当前可直接写成：

| 字段 | 语义 |
| --- | --- |
| `id` | task 标识 |
| `subject` | 简短标题 |
| `description` | 详细说明 |
| `activeForm` | `in_progress` 时 spinner 用的进行时文案 |
| `owner` | 当前 owner，可空 |
| `status` | `pending` / `in_progress` / `completed` |
| `blocks` | 本任务完成后可解锁的后继任务 |
| `blockedBy` | 本任务依赖的前置任务 |
| `metadata` | 任意附加数据 |

其中需要特别补两条边界：

- `metadata._internal` 会被 `TaskList` 过滤掉，不进入普通任务列表视图
- `blockedBy` 的“是否真的阻塞”不是单看数组本身，而是要与**当前未完成任务集合**做交集

也就是说：

- 一个 task 可以持久化保留旧 `blockedBy`
- 但只要其中前置任务都已 `completed`
- 这个 task 在运行时就会被视为“可做”

这解释了为什么：

- `TaskList` 会把已完成 blocker 从展示里的 `blockedBy` 过滤掉
- `Yj_()` / `rp1()` 也都会按“未完成前置任务集合”重新计算 open blockers

### `TaskList` tool 不是排序后的 canonical view

这一点之前容易和 prompt 文案混在一起。  
当前实现里，`TaskList` tool 只是：

- `QD(taskList)` 读取当前 task 数组
- 过滤 `metadata._internal`
- 再把已完成 blocker 从输出里的 `blockedBy` 去掉

它**不会**额外按 `id` 排序。  
因此：

- `TaskList` tool 输出顺序
  - 继承 `QD(...)` 的当前返回顺序
- “Prefer tasks in ID order”
  - 目前仍主要是 prompt 约束
  - 不是 `TaskList` tool 自己硬编码出来的展示真相

## task 状态机：是“约定主流程 + 还原回边”，不是严格单向自动机

先给出最稳的主流程：

```text
TaskCreate
  -> pending

worker/leader 开始做
  -> in_progress

工作完成
  -> completed

显式删除
  -> deleted（物理删除 task）
```

但真实运行时比 prompt 文案更细：

### 1. `pending`

- `TaskCreate` 固定以 `pending` 创建
- 初始同时写：
  - `owner: undefined`
  - `blocks: []`
  - `blockedBy: []`

### 2. `in_progress`

进入 `in_progress` 有 3 条活路径：

- `TaskUpdate(status="in_progress")`
- teammate 自动 claim：
  - `Yj_()` 先挑“第一个 `pending`、无 owner、无未完成 blocker 的 task”
  - `rp1()` 原子写入 `owner`
  - 随后 `dIq()` 再写 `status: "in_progress"`
- team 场景下的隐式 owner 填充：
  - 若 `TaskUpdate` 把任务改成 `in_progress`
  - 且未显式传 `owner`
  - 且原任务也没有 owner
  - 则自动把当前 agent `name` 写成 owner

### 3. `completed`

- `TaskUpdate(status="completed")` 是主入口
- 完成前会跑 `TaskCompleted` hooks
- 完成后 `TaskList` / auto-claim 都把它视为已解锁 blocker

### 4. `deleted`

- `deleted` 不是普通持久化状态
- `TaskUpdate(status="deleted")` 会直接物理删除该 task
- 返回值里会伪装成：
  - `updatedFields: ["deleted"]`
  - `statusChange: { from: oldStatus, to: "deleted" }`

### 5. 还原回边：`pending <- shutdown reclaim`

这条边非常关键，也正是之前文档没写细的地方。

当 teammate shutdown / terminated 后，`xA6()` 会把该 teammate名下**尚未完成**的任务统一改成：

- `owner: undefined`
- `status: "pending"`

因此本地真实状态机不是严格的：

- `pending -> in_progress -> completed`

而是：

```text
pending -> in_progress -> completed
          \-> pending   (teammate shutdown/termination recovery)
```

所以更准确的说法应是：

- `pending -> in_progress -> completed` 是**名义主流程**
- `shutdown reclaim -> pending` 是**运行时还原边**
- `deleted` 是**破坏性移除动作**

## owner 语义边界：规范值是 `name`，`agentId` 只在回收兼容中被接受

这是当前 `TaskList` 模型里最容易误写错的一块。

### 活写路径里的 owner 规范值

当前本地 bundle 里，owner 的活写路径有 4 类：

1. `TaskCreate`
   - 初始 `owner: undefined`
2. `TaskUpdate(owner=...)`
   - 显式写入调用方给的值
3. `TaskUpdate(status="in_progress")` 的隐式 owner 补写
   - 若满足条件，写入当前 agent `name`
4. auto-claim
   - `dIq()` 调 `rp1(taskList, taskId, agentName)`
   - 写入的也是 claimer 的 `name`

因此就当前本地活路径看：

- **owner 的规范值是 teammate `name`**
- **不是 `agentId`**

### 为什么 `xA6()` 又兼容 `agentId`

shutdown 回收函数 `xA6(taskList, ownerByAgentId, ownerByName, reason)` 会匹配：

- `task.owner === agentId`
- 或 `task.owner === teammateName`

这不是因为 `agentId` 是规范 owner，而更像是兼容边界：

- 防止旧数据 / 异常写入 / 历史格式导致 shutdown 后任务回收失败

因此当前最稳的结论是：

- 对外分配、自动 claim、正常协作都应把 owner 理解为 **成员 `name`**
- `xA6()` 对 `agentId` 的兼容只是还原/清理防御，不应倒推成 owner 的标准语义

## 依赖边是双向维护的，不是单字段魔法

`TaskUpdate` 的依赖字段需要分开理解：

- `addBlocks`
  - 表示“当前 task 完成后，哪些 task 会被它解锁”
- `addBlockedBy`
  - 表示“当前 task 依赖哪些前置 task”

真正落盘时不是单边写一个字段，而是通过 `ip1(...)` 建立互相对应的：

- `blocks`
- `blockedBy`

关系。

因此 `TaskList` 不是只有 status/owner 两维，而是同时承担依赖图。

## TeamCreate / TeamDelete 的生命周期模型

`TeamCreate` 会：

- 生成唯一 team 名（冲突时改名）
- 写入 team file
- 建 task list 目录
- 把当前 leader 注入 `teamContext`

`TeamDelete` 会：

- 先检查 `members.filter(name !== "team-lead").filter(isActive !== false)`
- 只要还有 active member，就拒绝 cleanup
- 通过后才清理：
  - team directory
  - task directory
  - worktree
  - 当前 session 的 `teamContext`

因此 team 生命周期是显式管理的，不是会话结束自动隐式销毁。

## 仍需保守表述的点

- `teamAllowedPaths`
  - 当前只钉住了 teammate init 的读侧和 session `addRules` 转换。
  - 没找到明确 writer，因此不要把它写成已经证实的完整 team-permission 持久化闭环。
- `worktreePath`
  - 当前只钉住了 cleanup / UI 读侧。
  - 没看到现行 teammate spawn 正常写入 `members[].worktreePath`，因此不要把它写成 pane / in-process teammate 的稳定产物。
- `hiddenPaneIds`
  - helper、team file 字段、UI 展示和 tmux backend 能力都还在。
  - 但当前发行版里上层 hide/show 动作链大概率断线，因此更稳的表述是“残留持久化字段 + 未接通的动作能力”，而不是完整活功能。
- `AppState.teamContext.teammates`
  - 当前已能确认它不是 team file 的完整回放。
  - 但还不能把所有漂移都写成 bug，因为它本来就更像 runtime/UI cache，而不是持久化真相层。

## 当前结论

基于当前本地 bundle，`Agent Team` 最合理的抽象已经不是：

- `SubAgent + TaskList`

而是：

- `TeamConfig(source-of-truth roster)`
- `SharedTaskList(source-of-truth tasks + dependency graph)`
- `TeamContext(process-local runtime mirror)`

其中最关键的收紧点是：

- `config.json` 而不是 `AppState.teamContext` 才是 team 真相
- `AppState.teamContext` 只是一份更窄的 process-local mirror，`teammates` 不是完整 roster replay
- `members[]` 的活字段比先前文档更大，已包含 `backendType / planModeRequired / mode / isActive / worktreePath` 等运行时持久化信息
- `isActive` 是 runtime active/idle 弱语义位，缺省值也会被按 active 处理
- `hiddenPaneIds` 更像残留持久化字段；hide/show 能力当前大概率未真正接线
- task 状态机存在明确的 shutdown recovery 回边
- owner 的规范语义是 `name`，`agentId` 只在回收兼容层被接受

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
