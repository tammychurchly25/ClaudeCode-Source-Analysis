# Teammate Runtime、Backend 与任务调度

## 本页用途

- 单独整理 teammate 的运行时实体、backend 分流和 task owner 变更路径。
- 重点钉死 pane / in-process 的 tick、唤醒、claim 竞争、回收与 `F$_()` 的真实地位。

## 相关文件

- [../01-resume-fork-sidechain-and-subagents.md](../01-resume-fork-sidechain-and-subagents.md)
- [02-agent-team-and-task-model.md](./02-agent-team-and-task-model.md)
- [03-agent-team-mailbox-and-approval.md](./03-agent-team-mailbox-and-approval.md)
- [../../02-execution/04-non-main-thread-prompt-paths.md](../../02-execution/04-non-main-thread-prompt-paths.md)

## `in_process_teammate` 是运行时实体，但有两种含义

之前最容易误判的一点，是把 `in_process_teammate` task type 直接等同于“工作一定在 leader 当前进程里跑”。  
本地 bundle 现在已经能把它拆成两种形态。

### 1. 真正的 in-process worker task

`jL8()` 注册的 task 字段当前至少包括：

- `type: "in_process_teammate"`
- `status: "running"`
- `identity`
  - `agentId`
  - `agentName`
  - `teamName`
  - `color`
  - `planModeRequired`
  - `parentSessionId`
- `prompt`
- `model`
- `abortController`
- `awaitingPlanApproval`
- `spinnerVerb`
- `pastTenseVerb`
- `permissionMode`
- `isIdle`
- `shutdownRequested`
- `lastReportedToolCount`
- `lastReportedTokenCount`
- `pendingUserMessages`
- `messages`
- `unregisterCleanup`

这才是由 `OL8() / Oj_()` 真正消费的本地 worker state。

### 2. pane teammate 的 leader 侧本地 task 壳

`Hq4()` 也会注册一个 `type: "in_process_teammate"` 的 task，但字段更少：

- 同样有 `identity`
- 同样有 `prompt / abortController / permissionMode / isIdle / shutdownRequested / pendingUserMessages`
- 但没有 `model`
- 没有 `messages`
- 没有 `spinnerVerb / pastTenseVerb`
- 它的 `abortController.abort()` 会转而 `killPane(...)`

因此要明确：

- `in_process_teammate` 是一个 **task/UI/bookkeeping 类型**
- 不是“执行位置一定在本进程”的充分条件

更准确的划分应是：

- `jL8() -> OL8() / Oj_()`：真正本进程 worker
- `Hq4() -> killPane hook`：pane worker 的 leader 侧本地镜像

## backend 选择：`Du()` 决定是否直接走 in-process，`BA6()` 决定 pane backend 类型

### `Du()`：是否启用 in-process

`Du()` 当前逻辑已经能直接写成：

1. 若当前是 non-interactive session：
   - 直接 `true`
2. 否则读取 teammate mode snapshot：
   - `in-process` -> `true`
   - `tmux` -> `false`
   - `auto` -> 继续判断
3. `auto` 下：
   - 若之前已被 `markInProcessFallback()` 标成 fallback
     - `true`
   - 否则：
     - `insideTmux === false`
     - 且 `inITerm2 === false`
     - 才走 `true`

因此 `auto` 的真实语义不是“随机挑一个 backend”，而是：

- 当前环境不像 pane-capable terminal 时直接 in-process
- 只要环境像 tmux / iTerm2，就优先尝试 pane backend
- pane backend 不可用时，才打 fallback 标记并回落 in-process

### `BA6()`：pane backend 检测矩阵

`BA6()` 当前可直接收敛为：

1. 已在 tmux 内
   - 直接选 `tmux`
   - `isNative: true`
   - `needsIt2Setup: false`
2. 在 iTerm2 内，且 `preferTmuxOverIterm2 !== true`
   - 若 `it2` CLI 可用
     - 选 `iterm2`
     - `isNative: true`
     - `needsIt2Setup: false`
3. 在 iTerm2 内，但 `it2` CLI 不可用
   - 若 tmux 可用
     - 退回 `tmux`
     - `isNative: false`
     - `needsIt2Setup: true`
4. 不在 tmux / iTerm2，但 tmux 可用
   - 选 `tmux`
   - `isNative: false`
   - `needsIt2Setup: false`
5. 否则
   - 抛“没有 pane backend 可用”

### snapshot 语义已经能写死

CLI 启动早期会 `captureTeammateModeSnapshot()`。  
pane teammate spawn 构造命令时，又会显式带：

- `--teammate-mode <snapshot>`

这说明：

- spawn 不是每次重新查配置
- 而是 leader 先截一份 mode snapshot，再传给 pane 子 CLI

## spawn 分流：真正的执行差异只在 `wQ4()` 与 `wu_() / $u_()`

顶层分流是：

```text
Ou_()
  -> if Du() === true
       wq4()
     else
       try BA6()
       catch (auto mode only)
         markInProcessFallback()
         wq4()
       else
         use_splitpane !== false ? wu_() : $u_()
```

### `wQ4()`：真正 in-process teammate

`wQ4()` 会：

1. 归一化 `name`
2. 算出 `agentId`
3. 调 `jL8()` 注册真正的 in-process task
4. 立刻 `oL8()` 启动 `oJ_()` worker loop
5. 回写：
   - `AppState.teamContext.teammates[...]`
   - team file `members.push({... backendType: "in-process" ...})`

### `wu_()` / `$u_()`：pane teammate

两条 pane 路径都遵循同一结构：

1. 创建 pane / window
2. 拼完整 CLI 命令
3. `sendCommandToPane(...)`
4. leader 本地调 `Hq4()` 注册 task 壳
5. team file `members.push({... backendType, tmuxPaneId, prompt, model ...})`
6. 再通过 mailbox 给新 teammate 塞初始 prompt

所以 pane teammate 不是“leader 持续在本地替它跑 agent loop”，而是：

- **leader 只负责 pane 创建、bookkeeping、kill 桥接与初始消息投递**
- **真正的 worker loop 在新 pane 里的 CLI 子进程执行**

## pane 子进程不是 dumb shell，而是完整 CLI teammate runtime

pane spawn 发进 pane 的命令当前已能写得很具体：

- 可执行文件：
  - `process.execPath` 或当前 CLI argv 路径
- flags：
  - `--agent-id`
  - `--agent-name`
  - `--team-name`
  - `--agent-color`
  - `--parent-session-id`
  - 条件性：
    - `--plan-mode-required`
    - `--agent-type`
    - `--model`
    - `--teammate-mode <snapshot>`
- env：
  - `CLAUDECODE=1`
  - `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`
  - 以及透传的 provider / remote / config 相关 env

子进程启动后，本地还能确认：

1. CLI 参数会还原 teammate identity
2. 通过 `setDynamicTeamContext(...)` 注入动态 team context
3. startup / reconnect 阶段会再读 team file
4. 用 team file 还原：
   - `leadAgentId`
   - 自己在 `members[]` 中对应的 `agentId`
   - `teamAllowedPaths`
5. 再安装 teammate idle / Stop hook

因此 pane backend 并不是另一套弱化 runtime。  
更稳的说法是：

- **pane teammate 启动的是同一份 CLI**
- **身份通过 flags 注入**
- **运行态通过 team file + startup init 重新接回**

这也意味着：

- pane worker 的主执行逻辑不是另写一套
- 而是和 in-process teammate 共用同一套 teammate runtime 语义

## in-process worker loop：`oJ_()` 是 turn executor，`$j_()` 是 idle wait loop

### `oJ_()` 的主骨架

`oJ_()` 当前可直接概括为：

```text
初始化 system prompt / agentDefinition / tool set
-> 先尝试一次 dIq() 自动认领 task
-> 把初始 prompt 写入 teammate transcript
-> while not aborted:
     设 currentWorkAbortController
     必要时 compact 历史
     BN(...) 跑 agent turn
     把流事件/assistant/tool结果回写到 teammate task state
     结束后设 isIdle = true
     发 idle notification
     调 $j_() 进入等待
     收到下一条 message / shutdown_request / task prompt 后继续
```

它不是单次 `prompt -> response` 函数，而是完整的多轮 teammate event loop。

### `$j_()`：精确等待顺序

`$j_()` 的顺序现在已经可以写死：

1. 先读本地 task state
   - `pendingUserMessages`
   - 若有，直接弹出第一条并返回
2. 若不是首次轮询
   - `await C_(500)`
3. 检查 abort
4. 查 mailbox
   - 先扫 unread structured shutdown message
   - 找到就优先返回 `shutdown_request`
5. 再扫 leader 发来的 unread message
6. 若没有，再扫任意 unread message
7. 最后尝试 `dIq()` 从 shared task list auto-claim
8. 还没有就继续下一次轮询

因此这个 wait loop 的 wakeup source 有且只有三类：

- 本地 `pendingUserMessages`
- mailbox unread 消息
- task list 中新出现的可 claim task

### tick / sleep / backoff

这部分现在已经能写得比“500ms 级轮询”更硬：

- `pollCount === 0`
  - 不 sleep，立即检查一次
- `pollCount > 0`
  - 每轮固定 `await C_(500)`

也就是说：

- **in-process teammate 的 idle tick 是固定 500ms**
- **没有看到指数退避**
- **没有看到按失败原因拉长间隔**
- **claim 失败、mailbox 空、没有 pendingUserMessages，都会统一回到下一次 500ms tick**

换句话说，当前本地实现是：

- `fixed-interval polling`

而不是：

- `exponential backoff`
- `event-driven blocking wait`

## claim 竞争与原子性：当前只有 per-task lock，没有全局调度器

### `Yj_()`：挑“第一个可做的 pending task”

`Yj_()` 的选择条件非常直接：

- `status === "pending"`
- `owner` 为空
- `blockedBy` 里没有未完成前置任务

然后：

- 直接对 `QD(taskList)` 返回数组做 `find(...)`
- 拿到**第一个**符合条件的 task

因此自动调度的真实策略不是复杂打分，而是：

- **first eligible pending task wins**

### `QD()` 的顺序不是“低 ID 优先”的实现真相

这里现在可以再收紧一层。  
`QD(taskList)` 当前底层就是：

```text
readdir(taskDir)
  -> 过滤 *.json
  -> 去掉扩展名
  -> Promise.all(KU(taskId))
```

中间没有显式：

- `sort((a, b) => ...)`
- 按数字 ID 重排
- 按 mtime / ctime 重排

因此 `Yj_()` 的“第一个可做 task”其实不是：

- “数值最小的 id”

而是：

- **`QD(...)` 当前返回数组里的第一个 eligible task**

也正因为这样，下面三层要分开看：

- auto-claim 主干
  - 吃的是 `QD(...)` 原始顺序
- `TaskList` tool
  - 也只是直接映射 `QD(...)` 结果，不额外排序
- TUI tasks 面板
  - `gy8()` 才会按 `Po6(...)` 对 `completed / in_progress / pending` 分桶后重排
  - `pending` 里还会先把未阻塞项排前，再按 `id` 排

所以当前 bundle 里真正存在的是：

- **claim 顺序与 tool 输出顺序，共同依赖 `QD(...)`**
- **TUI 面板顺序，是单独的 display 重排**

至于“为什么文案总强调低 ID 优先”：

- prompt 文案明确这样要求
- task 相关辅助排序里也有按 `id` 数值/字典序比较的 `Po6(...)`

但 auto-claim 主干本身并没有额外重排逻辑；它吃的是 `QD(...)` 当前顺序。  
因此若以后要复刻运行时，`QD(...)` 是否稳定排序是一个必须单独决策的实现点，不能被 prompt 文案掩盖掉。

### `rp1()`：当前主 claim 路径

`rp1(taskList, taskId, agentName)` 的执行顺序是：

1. 先确认 task 还存在
2. 给该 task 的 lock path 上锁
3. 锁内重新读取 task
4. 拒绝：
   - task 不存在
   - 已被其他 owner 占用
   - 已 `completed`
   - `blockedBy` 仍含未完成前置任务
5. 通过后原子写：
   - `owner: agentName`

关键点在于：

- claim 竞争是靠 **per-task lock** 解决的
- 不是 leader 侧 centralized scheduler

### `dIq()`：claim 成功后的第二步

`dIq()` 在 `rp1()` 成功后还会再做一次：

- `ju(taskId, { status: "in_progress" })`

因此 auto-claim 真实是两段式：

```text
选 task
-> rp1() 原子写 owner
-> ju() 再写 status = in_progress
-> 生成 prompt: "Complete all open tasks. Start with task #..."
```

### claim 失败后的竞争表现

若 `rp1()` 失败：

- `dIq()` 只记 log
- 不会本轮重试其他 task
- 不会立刻 fallback 到 `F$_()`
- 也不会进入更慢 backoff

之后的行为就是：

- 回到 `$j_()` 外层
- 下一次 500ms tick 再重新跑一遍

因此当前 bundle 中 claim 竞争的保守结论是：

- 有原子 owner claim
- 但没有更高层的 retry / rebalance / work stealing 逻辑

## `F$_()` 的真实地位：当前更像未接主干的保守变体

之前只能说“像预留分支”；现在可以进一步收紧。

### `F$_()` 做了什么

`F$_(taskList, taskId, agentName)` 比 `rp1()` 多两件事：

1. 它拿的不是 per-task lock，而是 tasklist 高水位 lock
2. 它会额外检查：
   - 当前 agent 是否已经拥有其他未完成任务
   - 若有则返回：
     - `reason: "agent_busy"`
     - `busyWithTasks: [...]`

### 但它怎么被接入

`F$_()` 的唯一入口是：

- `rp1(..., { checkAgentBusy: true })`

而当前本地 bundle 内：

- `checkAgentBusy` 只命中这一处判断
- 没看到任何活调用点传入 `{ checkAgentBusy: true }`
- auto-claim 的 `dIq()` 用的是普通 `rp1()`

因此这里可以比原文档写得更硬：

- `F$_()` 不是死代码
  - 因为 `rp1()` 确实能分流进去
- 但在**当前本地 bundle 可见活路径**里：
  - 没有调用者
  - 没有接入 auto-claim 主干
  - 没有接入 leader 调度主干

所以当前最稳判断应是：

- **`F$_()` 是已实现但未接入当前主调度链的保守 claim 变体**

若以后要证明它不是预留分支，必须找到新的活调用点；目前本地没有。

## owner 写路径与回收路径

真正会改 task `owner` 的入口，目前可收敛为：

1. `TaskCreate`
   - 初始 `owner: undefined`
2. `TaskUpdate(owner=...)`
   - 显式分配
3. `TaskUpdate(status="in_progress")`
   - team 场景下可能隐式补 `owner = 当前 agent name`
4. auto-claim
   - `rp1()` 写入 `owner = agentName`
5. shutdown reclaim
   - `xA6(taskList, agentId, teammateName, reason)`
   - 把尚未完成的任务改回：
     - `owner: undefined`
     - `status: "pending"`
   - 匹配时同时兼容：
     - `owner === agentId`
     - `owner === teammateName`

因此 leader 在 owner 上的主动行为其实很有限：

- 显式 `TaskUpdate`
- shutdown 后回收

当前没看到 leader 常驻调度器去持续 rebalance 任务。

## pane / in-process 的运行差异边界

### 真正的差异

- in-process
  - worker loop 直接在 leader 当前 Node 进程里跑
- pane
  - worker loop 在新 pane 的 CLI 子进程里跑

### 不该误判成差异的东西

- 两者都通过 team file 还原 identity / roster / permissions
- 两者都通过 mailbox 收发 teammate 消息
- 两者都通过 shared task list auto-claim
- 两者都由同一套 CLI teammate runtime 语义驱动

因此更准确的结论是：

- pane 与 in-process 的**协作语义同构**
- 真正的差别在：
  - 执行进程位置
  - 终止桥接方式
  - leader 是否额外持有一个本地 task 壳

## pane 子 CLI 在注入 teammate identity 后，会回到标准 REPL 主循环

之前这部分只能写到“同一份 CLI + startup init 重新接回”。  
现在还能再往前接一跳。

### 1. pane spawn 传入的不是内部 IPC 协议，而是标准 CLI flags

pane backend 构造命令时，实际塞进 pane 的是：

- 同一个 CLI 可执行文件
- 再加：
  - `--agent-id`
  - `--agent-name`
  - `--team-name`
  - `--agent-color`
  - `--parent-session-id`
  - 条件性：
    - `--plan-mode-required`
    - `--agent-type`
    - `--model`
    - `--teammate-mode`

这说明 pane worker 首先不是通过某个专用 worker entrypoint 启动，而是重走主 CLI 入口。

### 2. CLI 启动早期只把这些 flags 落成 dynamic teammate context

主 CLI 解析 argv 后，会先做一层强校验：

- `--agent-id / --agent-name / --team-name` 必须三者同时提供

然后调用：

- `setDynamicTeamContext({ agentId, agentName, teamName, color, planModeRequired, parentSessionId })`

也就是说，pane 子进程刚启动时拿到的不是完整运行时 state，而只是一份：

- dynamic teammate identity snapshot

### 3. 进入 App / REPL 后，再由 startup hook 把这份 snapshot 接回 team runtime

`f3A(...)` 这个主 REPL 组件初始化时会直接调用：

- `GU4(setAppState, initialMessages, { enabled: !remote })`

而 `GU4(...)` 内部会：

1. 从 session resume 数据里尝试拿 `teamName / agentName`
2. 否则退回 `getDynamicTeamContext()`
3. 调 `WU4(...)` 把 `teamContext` 写进 app state
4. 再调 `b5A(...)`
   - 读取 team file
   - 应用 `teamAllowedPaths`
   - 给 teammate 安装 Stop/idle notification hook

因此 pane 子 CLI 在 runtime 上不是“只靠 argv 直接开始跑”，而是：

```text
argv teammate flags
  -> dynamic teammate context
  -> App/f3A mount
  -> GU4()
     -> WU4() 初始化 teamContext
     -> b5A() 安装 teammate startup hooks / team permissions
```

### 4. 真正发起模型请求时，走的仍是标准 main-thread query path

`f3A(...)` 里后续正常输入、prompt queue、queued command 最终都会落到：

- `qU8(...)`
- `bT(...)`
- `CC({ ..., querySource: At6() })`

而 `At6()` 返回的是：

- `repl_main_thread`
  - 或带 output-style 后缀的 `repl_main_thread:...`

这说明 pane 子 CLI 在注入 teammate identity 之后，最后并不是跳进某条单独的 “pane teammate main loop”：

- 它回到的是**标准 REPL main-thread query path**

更准确的说法应是：

- pane teammate 用标准 CLI 入口启动
- 用 dynamic teammate context 把身份注入进去
- 再在 App 初始化阶段把身份接回 team runtime
- 最后仍通过标准 `repl_main_thread` 主查询链跑模型与工具

## shutdown / kill 的运行时边界

### in-process

- `terminate()`
  - 发 mailbox `shutdown_request`
  - 并标记 `shutdownRequested`
- `kill()`
  - 走 `HL8()`
  - `abortController.abort()`
  - `unregisterCleanup()`
  - 从 `teamContext.teammates` 移除
  - 把 task 状态改成 `killed`

### pane

- `terminate()`
  - 也是先发 `shutdown_request`
- 本地 task 壳的 `abortController.abort()`
  - 会转成 `killPane(...)`
- cleanup 时也会根据 `backendType / tmuxPaneId` 去 kill pane

所以 shutdown 协议仍然是 mailbox first，不是直接硬杀；  
但 pane backend 额外有一个 leader 本地 kill bridge。

## pane hide/show 当前更像残留半实现能力

这部分现在可以比“`hiddenPaneIds` 还没完全收口”写得更硬。

### 后端能力仍然存在

tmux backend `Cg1` 已经实现了：

- `hidePane(paneId)`
  - 通过 `break-pane` 把 pane 移到隐藏 session
- `showPane(paneId, target)`
  - 通过 `join-pane` 把 pane 接回目标 window

同时 backend capability 也显式区分：

- tmux:
  - `supportsHideShow = true`
- iTerm2:
  - `supportsHideShow = false`

因此底层不是“完全没有 hide/show 实现”。

### UI 入口与帮助文案也还在

TeamsDialog 仍保留：

- `h`
  - 当前成员 hide/show
- `H`
  - 批量 hide/show

帮助文案里也仍然直接显示：

- `h hide/show`
- `H hide/show all`

入口函数链也还能直接写出：

```text
TeamsDialog keybinding
  -> _Lz(teammate, teamName)
     -> if isHidden
          vg4(...)
        else
          Gg4(...)
```

### 但真正执行 hide/show 的上层函数在当前 bundle 里是空壳

当前 bundle 内：

- `GG4()` 函数体为空
- `VG4()` 函数体为空

同时全 bundle 也没看到任何活调用点去调用：

- backend `hidePane()`
- backend `showPane()`

也没看到当前活路径去调用：

- `Lj_(team, paneId)`
- `hj_(team, paneId)`

### 因此当前最稳判断

`hiddenPaneIds` / pane hide-show 这条链现在应拆成四层看：

1. team file 字段还在
2. tmux backend 能力还在
3. UI 快捷键与帮助文案还在
4. 但上层动作函数没有真正接到 backend / team-file helpers

因此更稳的结论不是“hide/show 功能完整但文档没补齐”，而是：

- **当前发行版里，pane hide/show 很可能是残留半实现功能**
- **展示层与后端层都还活着，但真正执行链大概率已经断线**

## 当前结论

基于当前本地 bundle，已经可以把 teammate runtime 写成下面这个模型：

```text
spawn
  -> 选 in-process / pane backend
  -> 注册本地 teammate task（真实 worker 或 leader shell）
  -> pane 时额外启动一份完整 CLI 子进程

worker turn
  -> BN(...) 执行一轮
  -> turn 结束进入 idle
  -> 发 idle notification
  -> $j_() 以固定 500ms tick 等待：
       pendingUserMessages
       -> mailbox
       -> task auto-claim

task claim
  -> Yj_() 选第一个可做 pending task
  -> rp1() 原子写 owner
  -> ju() 写 status = in_progress
  -> 失败则仅记 log，下一次 500ms tick 再来

shutdown
  -> mailbox 协议优先
  -> pane 场景再通过本地 task 壳桥接 killPane
  -> xA6() 回收未完成任务 owner/status
```

## 仍需保守表述的点

- pane teammate 的 worker loop虽然高度可判定为复用同一套 CLI teammate runtime，但当前直接抽到的轮询细节证据主要来自 `Oj_() / $j_()` 本身，而不是对子进程内部再次单独反编译出一条新分支
- `F$_()` 当前可非常强地判断为“未接当前主干”，但严格说仍属于**已实现、可被 future caller 激活**的分支，而不是可以写成“无意义死代码”

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
