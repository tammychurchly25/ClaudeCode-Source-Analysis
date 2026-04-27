# Resume、Fork、Sidechain 与 Subagent

## 本页用途

- 单独整理主执行链之外的还原、分叉与侧链执行路径。
- 把这部分从 Agent Team 协作模型里拆开，避免“会话还原”和“多 agent 协作”继续混写。

## 相关文件

- [../01-resume-fork-sidechain-and-subagents.md](../01-resume-fork-sidechain-and-subagents.md)
- [../../01-runtime/02-session-and-persistence.md](../../01-runtime/02-session-and-persistence.md)
- [../../02-execution/04-non-main-thread-prompt-paths.md](../../02-execution/04-non-main-thread-prompt-paths.md)
- [../02-remote-persistence-and-bridge.md](../02-remote-persistence-and-bridge.md)

## Resume

`I76(...)` 的入口比“读 transcript”更宽，当前已能确认支持 4 种输入：

- 不传参数
  - 取最近可还原 session
- 直接传 jsonl 路径
  - 走 `Cu_(path)` 读文件，再反推 `sessionId`
- 直接传 `sessionId`
  - 走 `Ht6(sessionId)`
- 直接传已加载 session/log 对象
  - 必要时再 `Qu(...)` 补成 full log

还原时不只是读 jsonl，还会按固定顺序补运行态：

- 还原 plan 文件
- 复制/还原 file-history backups
- 做 resume consistency / metadata 校验
- 重新装回 invoked skills
- 清理 interrupted turn
- 注入 resume hooks

更准确地说，`I76(...)` 当前骨架接近：

```text
resolve input
  -> 若是当前/指定 session，则还原 full log
  -> mN8(..., FM(sessionId)) 还原 plan
  -> RC8(...) / Fi1(...) 校正 metadata 与一致性
  -> Su_(messages) 还原 invoked skills runtime
  -> hq4(messages) 修补 interrupted turn
  -> BD("resume", { sessionId }) 注入 resume hook 结果
```

因此 resume 的还原对象是：

- transcript 主链
- session 级 metadata
- plan / file-history / invoked skills
- 还原后应追加的 hook 结果

而不是单纯“把历史消息读回来”。

## interrupted turn 处理

`hq4(...)` 不是一个泛化“补一句 continuation”的小工具，而是一条明确的边界修复链：

1. 先对历史消息做归一化
   - 旧 `permissionMode` 会按当前允许集合清洗
2. 再重建消息链并判断最后一个“真正的 turn 边界”
   - 跳过 `system`
   - 跳过 `progress`
   - 跳过 assistant 的 `apiError`
3. 根据 `hu_(...)` 的结果分两类处理
   - `interrupted_turn`
   - `interrupted_prompt`

`hu_(...)` 的判定已能写得更具体：

- 最后有效消息是 `assistant`
  - 不视为中断
- 最后有效消息是 `user`
  - 若 `isMeta` 或 `isCompactSummary`
    - 不视为中断
  - 若命中 `Jt6(user)`
    - 通常视为 `interrupted_turn`
  - 但若 `Ru_(...)` 发现它只是某个 brief 类工具结果的 user 包装
    - 不补 continuation
  - 否则
    - 记为 `interrupted_prompt`
- 最后有效消息是 `attachment`
  - 直接视为 `interrupted_turn`

真正的修补动作也分两段：

- 若判成 `interrupted_turn`
  - 追加一条 meta user continuation
  - 内容固定为 `Continue from where you left off.`
- 再从尾部回扫最后一个非 `system` / 非 `progress` 消息
  - 若它是 `user`
  - 则在其后补一条 system：`No response requested.`

这说明 `hq4(...)` 修的不是“模型回复被截断”这一种情况，而是：

- 把尾部未闭合的 user/prompt 边界补成可继续执行的 turn
- 同时避免还原后把一个裸 `user` 误判成还在等待即时回答

当前更稳的结论是：

- `turnInterruptionState.kind === "interrupted_turn"`
  - 表示还原器主动补了一条 continuation
- `turnInterruptionState.kind === "interrupted_prompt"`
  - 表示最后停在一个 prompt 边界上，但不一定要补 continuation

## Fork / Branch

已确认存在会话分叉逻辑：

- 新 sessionId
- 复制主链消息
- 重写 parentUuid/sessionId
- 保留当前 session 的 content replacement
- 标题可从首条 user 消息提炼

当前 `yHz(...)` 已能把 fork 的复制边界写得更硬：

- 输入源是当前 transcript
- 只复制：
  - `cr(P)` 可落盘消息
  - 且 `!P.isSidechain`
- 单独复制：
  - `type === "content-replacement"`
  - 且 `sessionId === 当前主 session`

复制时会同时做 3 件事：

- 新消息一律改成新 `sessionId`
- `parentUuid` 不再保留原值，而是按 fork 后的新链重新串
  - 只把非 `progress` 消息当成新的 parent 候选
- 每条复制出来的消息都补：
  - `forkedFrom: { sessionId: 原 session, messageUuid: 原 uuid }`

因此 fork 复制的不是“原文件逐行拷贝”，而是：

- 只抽主链
- 重建 parent 结构
- 保留可追溯的 `forkedFrom`

## Fork / Sidechain 的还原边界

这部分目前可以明确区分 3 条还原入口：

1. 主 session resume
   - 走 `I76(...)`
   - 面向 `<sessionId>.jsonl`
2. fork 后切入新 branch
   - plan 文件不走 `mN8(...)` 还原
   - 而是走 `AVq(oldSession, newSession)` 复制旧 slug 对应 plan
3. subagent / sidechain 还原
   - 不走 `/resume` 主链
   - 走 `Tk6(agentId)` + `Na1(agentId)` 这一组 agent 级还原入口

fork 的“不复制”边界也已经比较清楚：

- 不复制 `isSidechain === true` 的消息
- 不复制 subagent 独立 transcript 文件
- 不把 sidechain 重新并回新 branch 主链

所以 fork 还原的是：

- 主会话消息链
- 主 session 级 content replacement
- plan slug 对应文件

而不是：

- 全量 sidechain/subagent 执行现场
- team/pane 运行态
- 已落到 `subagents/agent-<id>.jsonl` 的子链

## Sidechain / Subagent

`CC` 本身可复用于 sidechain/subagent。  
sidechain transcript 与主线程 transcript 分开写；subagent 走 `subagents/agent-<id>.jsonl`。

## 普通 `Agent` 实际上有 3 条分流

当前可以把“非 teammate 的 Agent”进一步拆成 3 类：

1. implicit fork
   - 触发条件：省略 `subagent_type`
   - 使用内建 `fork` agent definition
   - 明文特征：
     - `agentType: "fork"`
     - `tools: ["*"]`
     - `model: "inherit"`
     - `permissionMode: "bubble"`
     - `maxTurns: 200`
   - 语义不是“fresh agent”，而是：
     - 复用父链 prompt cache 与父线程已渲染 `systemPrompt`
     - `forkContextMessages` 直接取父消息链
     - 但 request-level `userContext / systemContext` 不是整包照搬，而是仍按 `BN(...)` 默认装配再做裁剪/override

2. typed subagent
   - 触发条件：显式传 `subagent_type`
   - 从 agent definitions 中解析对应 agent
   - 语义是 fresh start：
     - 不继承父会话对话上下文
     - prompt 只是一条新的 user message
     - system prompt 来自目标 agent definition

3. teammate
   - 触发条件：同时落在 team 场景并传 `name`
   - 不再走普通 subagent 路径
   - 直接分流到 `Agent Team` runtime

因此“普通 subagent”和“teammate”在 `Agent` 工具入口处就已经分家，不是运行到后面才自然分化。

## Fork 不是任意可重入

当前 bundle 里，implicit fork 还有一个硬限制：

- fork worker 内不能再创建 fork
- 命中条件包括：
  - 当前 `querySource` 已经是 fork builtin agent
  - 或当前消息链已被判定为 forked worker

命中后会直接报：

- `Fork is not available inside a forked worker. Complete your task directly using your tools.`

这说明 fork 不是无限递归树，而更像“主线程向外分叉的一层 sidechain worker”。

## 普通 subagent 的 sync / async task 形态

普通 subagent 无论最终是前台还是后台，当前本地 task system 里对应的核心 task type 都是：

- `local_agent`

差别主要在 `isBackgrounded` 和通知路径：

- 同步 subagent
  - 主线程直接等待 `BN(...)` 流结束
  - 完成后立即把结果作为本次 tool result 返回
- 后台 subagent
  - 先注册 `local_agent` task
  - 返回 `status: "async_launched"`
  - 后续通过 `task_notification` 作为系统通知回流

另外同步 subagent 还有一条自动后台化路径：

- `Y44(...)` 会先注册 `local_agent`
- 初始状态固定：
  - `status: "running"`
  - `isBackgrounded: false`
- 若超过 `autoBackgroundMs` 仍未完成
  - 只把同一个 task 改成 `isBackgrounded: true`
  - 同时 resolve `backgroundSignal`

同步执行侧随后不是“继续等同一条前台流”，而是：

- `Promise.race(stream.next(), backgroundSignal)`
- 一旦发现已经被后台化：
  - 停掉当前前台 iterator
  - 用同一个 `agentId`
  - 用同一个 task 里的 `abortController`
  - 以 `isAsync: true` 重新接一条后台 `BN(...)` 流
  - 结果继续写入同一个 transcript / task / output file

因此自动后台化的关键不是“新建一个后台 agent”，而是：

- **同一个 `local_agent` task 的交互模式切换**
- **同一个 agentId 的执行从前台等待切到后台通知**

所以“后台 agent”不是另一种 transcript/runtime 类型，而更像同一个 local subagent task 的交互模式切换。

## Sidechain transcript 的真实落盘规则

当前已经可以把普通 subagent transcript 写得更具体：

写盘入口现在已经能明确到 writer 分支级别：

- `recordSidechainTranscript(...)` -> `QU(...)`
- `QU(...)` 会调 `insertMessageChain(messages, true, agentId, parentUuid)`
- `insertMessageChain(...)` 会把每条消息都补成：
  - `isSidechain: true`
  - `agentId`
  - `sessionId: 当前主 session`
  - `parentUuid: sidechain 内部链`

真正决定文件目标的是 `appendEntry(...)`：

- 普通主链消息
  - 写主 session jsonl
- `content-replacement` 且带 `agentId`
  - 直接写 `$0(agentId)`
- 普通消息但满足：
  - `isSidechain === true`
  - `agentId !== undefined`
  - 也直接写 `$0(agentId)`

目标路径固定为：

- `$0(agentId)` -> `<project>/<current-session-id>/subagents/agent-<agentId>.jsonl`

同理：

- `content-replacement` 若带 `agentId`
  - 也会写到该 subagent transcript

并且 sidechain 分流还有一个重要副作用：

- sidechain 写盘命中 `w = A.isSidechain && A.agentId !== undefined`
- 命中后虽然会落盘
- 但不会走主链那条 `persistToRemote(...)`

所以在**实时写盘阶段**，sidechain transcript 更像：

- 本地 session 内部的专用持久化分支
- 不经主 transcript 的 remote persistence writer

还原时：

- `Tk6(agentId)` 直接读取该 sidechain 文件
- 只挑出：
  - `agentId === 当前 agent`
  - `isSidechain === true`
- 先用 `parentUuid` 集合找叶子
- 再沿 `x76(...)` 回溯出该 agent 的独立消息链
- 最后把返回值里的：
  - `isSidechain`
  - `parentUuid`
  - 从外部视角剥掉

这说明普通 subagent 持久化不是“主 transcript 打标签”，而是：

- 主链保留摘要/通知语义
- 子链把完整执行过程单独落在 subagent transcript

## CCR v2 / remote 下的 subagent hydrate

这里需要把“本地 writer”和“远端还原”严格分开。

### 运行时写入

当前 bundle 下，普通 sidechain / subagent 的**实时持久化**仍然只有本地文件：

- 普通 sidechain 消息
  - 写 `$0(agentId)`
- 带 `agentId` 的 `content-replacement`
  - 也写 `$0(agentId)`
- 这条路径不走 `persistToRemote(...)`

所以从 writer 视角说：

- sidechain transcript 不是主链 remote writer 直接同步出去的对象

### CCR v2 resume hydrate

但 resume 阶段不能再写成“只依赖本地 jsonl”。

`OqA(sessionId)` 在 CCR v2 下会做两层 hydrate：

1. foreground entries
   - 调 `InternalEventReader()`
   - 把主链 payload 重写回 `<sessionId>.jsonl`
2. subagent entries
   - 调 `InternalSubagentEventReader()`
   - 按 `agent_id` 分组
   - 逐个重写回 `subagents/agent-<id>.jsonl`

这说明 CCR v2 下实际存在第二条还原来源：

- 本地已存在的 subagent transcript
- remote internal events 回灌出的 subagent transcript

### 优先级与覆盖边界

当前本地代码能确认的规则是：

- 若 `InternalSubagentEventReader()` 没注册
  - 不做 subagent hydrate
  - `Tk6(agentId)` 只能继续读现有本地文件
- 若 reader 注册了，但返回空或无 payload
  - 不改写现有 `agent-<id>.jsonl`
- 若 reader 返回了 payload
  - `OqA(...)` 会直接 `writeFile(...)`
  - 按 agent 整文件重写对应 `agent-<id>.jsonl`

因此当前最稳的还原优先级应写成：

- **CCR v2 hydrate 成功写回的 subagent jsonl**
  - 优先于磁盘上先前残留的本地 sidechain 文件
- **没有 hydrate 成功写回时**
  - 才退回读取现有本地 sidechain 文件

### 冲突处理边界

当前 bundle 下没有看到“本地旧文件 + remote payload 增量合并”的逻辑。  
`OqA(...)` 对每个 agent 的处理是：

- 先按 `agent_id` 聚合 payload
- 再把该 agent 的 payload 顺序串成 JSONL
- 直接覆盖目标 `agent-<id>.jsonl`

所以更稳的说法不是“会智能 merge”，而是：

- **按 agent 粒度整文件覆盖**
- **payload 内部顺序沿用 reader 返回顺序**
- **当前未见额外冲突裁决器**

## subagent transcript 的还原规则与限制

这部分现在可以写成明确的“能还原什么、不能还原什么”：

- `Tk6(agentId)` 找不到文件
  - 返回 `null`
- 文件里没有该 `agentId` 的 sidechain 消息
  - 返回 `null`
- 找不到有效叶子
  - 返回 `null`
- 只有成功还原出一条 sidechain 链
  - 才返回 `{ messages, contentReplacements }`

更关键的是还原入口边界：

- 主 `/resume`
  - 会在 lite log enrich 时显式过滤：
    - `isSidechain === true`
    - `teamName !== undefined`
  - 所以 sidechain transcript 不会作为“普通会话”出现在 resume 列表里
- subagent resume
  - 走 `ka1(...)`
  - 先读 `Tk6(agentId)` transcript
  - 再读 `Na1(agentId)` metadata
  - 必要时还原 `worktreePath`
  - fork agent 还要额外重建父线程 system prompt

因此当前 bundle 下更稳的说法是：

- sidechain transcript 是 **内部可还原**
- 但不是 **顶层可枚举、可直接 /resume 的 session**

它服务的是：

- `ResumeAgent` / 后台 agent 继续执行
- agent summary / 子链回放

而不是把 subagent 提升成一级会话。

## ResumeAgent 的真实还原契约

`SendMessage` 对本地 agent 的还原分流已经可以写成明确状态表。

| 条件 | 关键判定 | 行为 |
| --- | --- | --- |
| 命中 `agentNameRegistry`，且 task 是运行中的 `local_agent` | `qV(w) && w.status === "running"` | 不走还原；只把消息塞进 `pendingMessages` |
| 命中 `agentNameRegistry`，且 task 是已停止的 `local_agent` | `qV(w)` 且 `status !== "running"` | 直接调 `ka1(...)` 从 transcript 后台拉起 |
| 命中 `agentNameRegistry`，但当前没有活动 task | registry 有地址，但 `tasks[agentId]` 不再是活的 `local_agent` | 仍尝试 `ka1(...)` |
| `Tk6(agentId)` 返回 `null` | transcript 缺失 / 无 sidechain 消息 / 无有效叶子 | `ka1(...)` 抛 `No transcript found`，外层向用户报“no transcript to resume” |

这张表背后可以再拆成 3 层角色：

- `agentNameRegistry`
  - 只负责把 `to: name` 解析到 `agentId`
  - 它不是还原能力本身
- task state
  - 只决定“追加消息”还是“尝试重启”
- transcript
  - 才是真正的还原闸门

也就是说，当前 bundle 下：

- **registered** 不等于 **recoverable**
- 真正决定能否还原的是 `Tk6(agentId)`

### `cleaned up` 的最稳边界

工具层文案里会把某些失败提示成：

- `registered but has no transcript to resume`
- `It may have been cleaned up`

但从本地静态证据看，更稳的写法应是：

- 当前只能确认“registry 仍可命中，但 transcript 已不可还原”
- 这可能是 cleaned up
- 也可能只是 sidechain 文件不存在、为空、或无法还原叶子

因此不宜把 `cleaned up` 写成当前已独立可判定的 runtime 状态。

### metadata 不是还原硬前提

`ka1(...)` 会并行读取：

- `Tk6(agentId)`
- `Na1(agentId)`

但两者地位不对等：

- `Tk6(agentId)` 缺失
  - 直接失败
- `Na1(agentId)` 缺失
  - 仍可继续还原
  - 只是退化成：
    - `agentType` 用默认 `general-purpose`
    - `description` 用 `"(resumed)"`
    - 不还原 `worktreePath`

所以当前更准确的合同是：

- transcript 是必需还原材料
- metadata 是可选增强材料

### fork agent 的额外门槛

若 metadata 表明该 agent 是 fork agent：

- `ka1(...)` 不只读 transcript
- 还必须重建父线程 system prompt

若父 system prompt 无法重建，会直接报：

- `Cannot resume fork agent: unable to reconstruct parent system prompt`

因此 fork resume 的真实门槛高于普通 typed subagent。

## `agent-<id>.meta.json` 的当前闭环边界

这部分当前可以写实，但还不能过度泛化成完整 schema。

### 路径与读接口

- 路径由 `cC4(agentId)` 派生
  - 即 `agent-<id>.jsonl` 同目录下的 `agent-<id>.meta.json`
- 写接口：`es6(agentId, data)`
- 读接口：`Na1(agentId)`
  - 文件不存在时返回 `null`

### 当前已确认的消费字段

`ka1(...)` 当前明确消费：

- `agentType`
- `description`
- `worktreePath`

消费方式分别是：

- `agentType`
  - 决定还原成 fork agent、指定 typed subagent，还是默认 `general-purpose`
- `description`
  - 用作 resumed task 的描述文本
- `worktreePath`
  - 若目录仍存在，则还原该 agent 的 worktree cwd
  - 若目录已不存在，则记录日志后回退到父 cwd

### 当前已确认的 writer 边界

目前本地 bundle 里直接能钉住的 writer 很少。

至少有一条明确写入路径：

- 普通 subagent 使用 `worktree` 隔离
- 结束时若 worktree 无变更、被清理删除
- 会调用：
  - `es6(agentId, { agentType, description })`

这条路径的语义更像：

- **清掉已失效的 worktree 还原信息**
- 但保留最小的 agent 身份信息

所以当前最稳结论是：

- `agentType` 与 `description` 已有明确 writer
- `worktreePath` 的读侧已钉住
- 但当前页证据还不足以把 `worktreePath` 的 writer 闭成统一 schema 生命周期

因此这里不应写成：

- `.meta.json` 必定完整包含 `agentType / description / worktreePath`

而应写成：

- **当前已证实它是一个可选的 agent-level 还原增强文件**
- **字段集合在本地 bundle 中仍未完全闭环**

## `contentReplacementState` 的还原合并规则

这部分现在可以从 `ka1(...) -> W2q(...) -> nv8(...)` 收紧到实现级别。

还原时顺序是：

1. 先用 `Tk6(agentId)` 还原消息链
2. 规范化成 `H = rebuilt messages`
3. 再计算：
   - `J = W2q(currentContentReplacementState, H, transcriptContentReplacements)`

### 合并规则

`W2q(...)` / `nv8(...)` 当前做的不是“简单拼接”，而是：

- 先扫描还原后消息链 `H`
- 只提取**这条还原链里真实出现过的** `toolUseId`
- 把这些 `toolUseId` 全部标进 `seenIds`
- 然后分两层填 replacement：
  - 第一层：先吃 transcript 自带的 `contentReplacements`
  - 第二层：再用当前运行态 `contentReplacementState.replacements` 补缺

所以重叠键的优先级已经能写死：

- **transcript 还原出的 replacement 优先**
- **当前运行态 replacement 只做 backfill，不覆盖 transcript 同键值**

### 生效范围边界

还有两个边界不能漏：

- 若某个 replacement 对应的 `toolUseId` 不在还原链 `H` 里
  - 不会被带进新的 `contentReplacementState`
- `seenIds` 会按还原链中的 `toolUseId` 重新建立
  - 这保证还原后仍能识别哪些 tool result 已经被持久化替换过

所以这里的真实语义更接近：

- **以还原出来的消息链为主集合**
- **用 transcript replacement 复原旧链状态**
- **再让当前运行态 replacement 对同一消息链做缺口补齐**

### 与 worktree 还原失败无关

`worktreePath` 是否还能还原，只影响 cwd 选择，不影响这一步 replacement 合并。  
也就是说即使：

- metadata 里的 `worktreePath` 已失效
- `ka1(...)` 回退到父 cwd

还原链的 `contentReplacementState` 仍会照常重建。

## worktree isolation 只是普通 subagent 能力，不是 teammate 专属

`Agent` 工具的 `isolation: "worktree"` 适用于普通 subagent。

当前实现已确认：

- 创建 worktree 后，会给 agent 额外补一条上下文提醒
- 明确告诉它：
  - 继承的是父工作目录语义
  - 但自己实际运行在独立 worktree
  - 父链中提到的路径需要映射到新 worktree root
  - 编辑前应重新读取文件

因此 worktree 不是简单“切目录”，而是普通 subagent prompt/runtime 合同的一部分。

## 当前判断

这组能力更适合被视为 runtime / persistence 体系的外延，而不是 Agent Team 的一部分。

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
