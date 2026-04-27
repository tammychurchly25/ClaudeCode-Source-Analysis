# Compat、Agent Definitions 与指令进入非主线程的载体

## 本页用途

- 用来收口 compat、`CLAUDE.md`、agent definitions 与目录扫描规则，说明这些输入如何进入非主线程 prompt。
- 用来把 verification 残留资产与真正活着的 agent definitions/source precedence 链拆开。

## 相关文件

- [../04-non-main-thread-prompt-paths.md](../04-non-main-thread-prompt-paths.md)
- [../02-instruction-discovery-and-rules.md](../02-instruction-discovery-and-rules.md)
- [../../03-ecosystem/06-plugin-system.md](../../03-ecosystem/06-plugin-system.md)
- [../../03-ecosystem/01-resume-fork-sidechain-and-subagents.md](../../03-ecosystem/01-resume-fork-sidechain-and-subagents.md)

## 一句话结论

- compat 没有独立的非主线程本地注入链；它只能先变成主线程或父链已经装好的 prompt 产物，再被非主线程复用或继承。
- agent definitions 的本地来源、winner 规则与可见集裁剪链已经能闭环；当前没有看到 runtime 再偷偷 materialize 一个 verification agent 的路径。

## compat 在非主线程里的真实进入面：现在可以收成一条闭环

这部分以前容易和 verification 一起挂成“可能还有第二套 hidden prompt path”，但当前本地 bundle 已经足够把它们拆开。

对于 non-main-thread prompt，compat 当前更稳的本地闭环是：

```text
compat-family files
  -> /init builtin prompt 读取
  -> 重要内容写入 CLAUDE.md / claude.local.md
  -> 后续 sj() 扫描
  -> userContext.ClaudeMd
  -> 非主线程通过已有三种 carrier 进入：
       1. BN(...) fresh build
       2. lZ(...) snapshot reuse
       3. 显式 override / 父 prompt 复用
```

也就是说，compat 进入非主线程的方式当前只应写成三类：

1. **`BN(...)` fresh build**
   - 调 `_$()`
   - 若 `/init` 已把 compat 内容物化进 `CLAUDE.md` 体系
   - 那它会随 `userContext.ClaudeMd` 进入 `Lx8(...)`
2. **`lZ(...)` snapshot reuse**
   - 不重新扫描 compat
   - 只是复用旧的 `systemPrompt / userContext / systemContext`
   - 因而 compat 若存在，也是**随旧 snapshot 继承**
3. **显式 override / 父 prompt 复用**
   - 如 `magic_docs`
   - 或 fork-family 对父 `systemPrompt` 的复用
   - compat 若存在，也是**通过父链已装配产物传入**

反过来说，当前本地没有看到 compat 在下面这些路径里拥有独立注入面：

- `hook_prompt`
- `hook_agent`
- `compact` dedicated summarize fallback
- `lZ(...)` 本体

因此更稳的本地表述不是“compat 在非主线程里也许另有隐藏 prompt”，而是：

- **compat 没有独立的非主线程本地注入链**
- **它只能先变成主线程/父链已经装好的 prompt 产物，然后被非主线程复用或继承**

## 本地仍保留的最后边界

就“非主线程是否完全沿用同一套 merge”这个问题，当前本地 bundle 可见范围内已经基本闭环：

- `verification_agent` 这个字符串当前只直接出现在 `_I4(...)` 的 agentic-query 识别位
- 没有发现任何本地 `querySource: "verification_agent"` 发起点
- 本地可见 subagent 正常发起时，`querySource` 走的是：
  - built-in agent -> `agent:builtin:<agentType>`
  - custom agent -> `agent:custom`
  - 而不是 `verification_agent`
- 当前 built-in agent 注册表里，直接可见的内建项是：
  - `general-purpose`
  - `statusline-setup`
  - `Explore`
  - `Plan`
  - `claude-code-guide`
- 当前没有直接看到 `agentType: "verification"` 被注册进去
- 还可以再把“没有完整本地 wiring”写成一组**反证闭环**：
  - `verification_agent`
    - 在整份本地 `cli.js` 静态扫描里只命中 1 次
    - 且就是 `_I4(...)` 中的 `Y.querySource === "verification_agent"` 识别位
    - 没有任何 `querySource: "verification_agent"` 本地发起点
  - `dtw`
    - `You are a verification specialist.` / `VERDICT: PASS` 这组 verifier 文本各只命中 1 次
    - 且都落在 `Cvq()` 里对 `dtw` 的文本赋值
    - 没有看到 `getSystemPrompt: () => dtw` 或别的本地 consumer
  - `subagent_type="verification"`
    - 不是 launch path
    - 只在 `TodoWrite` / `Task` 的 tool-result 文案模板里出现
    - 它提示的是“去 spawn verification agent”，不是 runtime 已经存在该 agent
  - querySource 生成器也不支持它
    - `vC8(agentType, isBuiltIn)` 只会产出 `agent:builtin:<agentType>`、`agent:default` 或 `agent:custom`
    - 当前没有从 agent launch 正常落成 `verification_agent` 的本地分支
- 这意味着当前本地其实不是“少找到了一根线”，而是：
  - **verification 的三个残留碎片彼此没有接上**
  - **识别位、prompt 资产、tool-result nudge 互相独立，无法在本地自动闭成可执行 agent**
- 因而 verification 家族现在应拆成：
  - **活路径**：`hook_prompt`、`hook_agent`
  - **残留资产**：`dtw` verifier prompt、`verification_agent` 识别位、`TodoWrite/Task` tool-result 里的 `subagent_type="verification"` 提示文案
- 当前本地确实存在动态 agent definition 注入，但路径都已能点名：
  - `gC(i1())` 读取 `.claude/agents`
  - plugin manifest 的 `agents / agentsPaths`
  - `flagSettings.agents` 经 `z68(..., "flagSettings")` 注入
- 这些入口最后都会显式落进 `agentDefinitions.allAgents / activeAgents`
- 当前没看到“bundle 不显式注册、但运行时再偷偷 materialize 一个 verification agent”的本地路径
- 对还原 non-main-thread prompt 主逻辑而言，当前不应再把 compat 与 verification 一起视作同等级未决项
- 更窄、也更合理的剩余边界是：
  - **本地发行版之外是否还有别的 build 把 `dtw` 这类 verifier prompt 真正接成了 agent**
  - **远端服务端收到本地 payload 后，是否还会再额外拼装 compat / verification 指令**
- `currentDate / gitStatus / ClaudeMd` 的本地后续命中也已基本穷尽，没有再看到 `BN(...)` 之外的专门裁剪点

### 最后封口：本地可能 materialize verification agent 的入口已被逐项排除

把当前 bundle 里所有理论上可能“补完 verification wiring”的入口并排后，可以直接收成下面这条反证链：

```text
1. agent registry source
   -> gC(...)
   -> built-in Rk8() + plugin K68() + disk Ao("agents", root) + flagSettings.agents
   -> MV(...)
   -> activeAgents
   -> 未见 verification

2. agent launch querySource
   -> vC8(agentType, isBuiltIn)
   -> agent:builtin:<agentType> | agent:default | agent:custom
   -> 未产出 verification_agent

3. verifier prompt asset
   -> dtw in Cvq()
   -> 只有文本赋值
   -> 未接 getSystemPrompt / 注册表 / launch site

4. verification hint text
   -> TodoWrite / Task tool_result
   -> 只是 nudge
   -> 不 materialize agent
```

因此当前更硬的本地结论可以直接写成：

- **本地 bundle 内不存在一条“把 `verification_agent` 识别位、`dtw` prompt、`subagent_type="verification"` 文案自动接成完整 agent wiring”的路径**
- **如果未来还要继续追 verification，只该追 bundle 外 build 差异或服务端黑箱，不应再把它列为本地主逻辑未闭环**

## Agent definitions：来源、winner 与裁剪链现在也能写闭环

这一页不该只记“有哪些注入路径”，还应补上**谁覆盖谁、谁会被裁掉**。

### 1. 基础来源：`gC(...)` 先汇总 built-in / plugin / `.claude/agents`

`gC(projectRoot)` 当前的主装配顺序可直接写成：

```text
Rk8()                 -> built-in agents
K68()                 -> plugin agents
Ao("agents", root)    -> markdown agents from disk
=> allAgents
=> MV(allAgents)
=> activeAgents
```

其中：

- `Rk8()`
  - 内建 agent 定义表
- `K68()`
  - 从当前启用 plugins 收集 plugin agents
- `Ao("agents", root)`
  - 从磁盘目录收集 markdown agents

### 2. `.claude/agents` 的磁盘来源：managed 永远在，user/project 受 gate 影响

`Ao("agents", root)` 对 agent 目录的 source 当前可收成三类：

- `policySettings`
  - `pP()/.claude/agents`
- `userSettings`
  - `~/.claude/agents`
- `projectSettings`
  - 项目根及相关 roots 下的 `.claude/agents`

但它不是无条件全开：

- 若 `strictPluginOnlyCustomization("agents")` 命中
  - `Ao("agents", ...)` 会直接跳过 `userSettings / projectSettings`
  - 只保留 managed 目录
- plugin agents 不走这条目录 loader
  - 不受这个跳过分支影响

因此这条 gate 的真实语义不是“plugin 优先级更高”，而是：

- **在 source load 阶段就切掉 user/project `.claude/agents`**

### 3. 同一文件去重与同名 agent 覆盖是两层事

`Ao("agents", ...)` 里先做的是**文件级 dedupe**：

- 先对收集到的文件跑 `realpath`
- 若多个 source 命中同一个物理文件
  - 后命中的文件会被直接跳过

这一步解决的是：

- 同一物理 agent 文件被多个路径别名重复扫到

它还不是最终的“同名 agent 谁生效”。  
真正决定同名 `agentType` winner 的，是后面的 `MV(...)`。

### 4. `flagSettings.agents` 不在 `gC(...)` 内，而是在外层追加后再 `MV(...)`

`flagSettings.agents` 当前不是 `gC(...)` 自己读进来的。

本地可见链路是：

- CLI/control 输入里的 `agents` JSON
  - 走 `z68(..., "flagSettings")`
- 外层把结果 append 到 `J_.allAgents`
- 再对 `[...J_.allAgents, ...Bz]` 重新跑 `MV(...)`

因此 `flagSettings.agents` 的真实位置是：

- **后置注入**
- **但仍受 `MV(...)` 的 source precedence 约束**

### 5. `MV(...)` 的 source precedence 现在可以直接写死

`MV(allAgents)` 不是按数组顺序盲取最后一个，而是先按 source 分桶，再固定按这个顺序写入 `Map(agentType -> agent)`：

1. `built-in`
2. `plugin`
3. `userSettings`
4. `projectSettings`
5. `flagSettings`
6. `policySettings`

因为 `Map.set(...)` 后写覆盖前写，所以当前同名 `agentType` 的 winner 规则可以直接写成：

```text
policySettings
  > flagSettings
  > projectSettings
  > userSettings
  > plugin
  > built-in
```

这条规则正是“某个 agent 为什么生效/失效”的主闭环。

也就是说：

- plugin 可以覆盖 built-in
- user/project 可以覆盖 plugin
- flag 可以覆盖 built-in / plugin / user / project
- policy 永远是最终 winner

### 6. prompt 可见集还会继续经过三层裁剪

即使某个 agent 已经是 `MV(...)` 的 winner，它也不一定会出现在最终可见集里。  
当前至少还有三层运行时裁剪：

1. `requiredMcpServers`
2. subagent-launch deny rules
3. `allowedAgentTypes`

#### 6.1 `requiredMcpServers`

`kt6(...) / cC8(...)` 会按当前可见 MCP server name 过滤 agent：

- prompt / listing 阶段会先过滤掉 requirement 不满足的 agent
- 真正 launch 时还会再检查一次
- 若相关 MCP server 仍在 pending
  - launch 路径会先短暂等待
  - 最后仍不满足就硬失败

因此这条裁剪不只是“列表不好看”，而是：

- **launch 时也会阻止 agent 真正执行**

#### 6.2 deny rules：可以把某个 `agentType` 从可见集里剔掉

`Tt6(...)` 会读取针对 subagent-launch tool 的 deny rules，并按 `ruleContent = agentType` 过滤 agent 列表。

这条裁剪当前至少影响两处：

- agent listing / delta 附件
- 真正的 agent launch 校验

因此某个 agent 明明是 `MV(...)` winner，但用户仍看不到 / 用不了，当前最直接的本地原因之一就是：

- **它被 agent-launch tool 的 deny rule 按 `agentType` 裁掉了**

#### 6.3 `allowedAgentTypes`：来自当前 agent 的 tool allowlist，不是另一套注册表

`allowedAgentTypes` 当前不是 agent registry 自己生成的 second source。  
它来自 `xn(...)` 对当前 agent `tools` frontmatter 的解析：

- 若当前 agent 的 `tools` 里包含 subagent-launch tool
- 且该 rule 带了 `ruleContent`
  - 就把 `ruleContent` 按逗号拆成 `allowedAgentTypes`

随后这份名单会被带进：

- `options.agentDefinitions.allowedAgentTypes`
- 模型调用时的 `callModel(..., allowedAgentTypes)`
- agent listing delta 生成
- subagent launch 时的最终过滤

因此它的真实语义是：

- **当前 agent 对“可继续派生哪些 agentType”的白名单裁剪**
- **不是新的 agent source，也不是新的 prompt producer**

### 7. 最终可见集：现在可以写成一条完整链

把上面几层串起来，当前更稳的 agent visibility 链应写成：

```text
built-in
  + plugin agents
  + .claude/agents (policy / user / project; user/project may be gated off)
  + flagSettings.agents
  -> MV(...) choose winner per agentType
  -> requiredMcpServers filter
  -> deny rules on subagent-launch tool
  -> allowedAgentTypes whitelist
  -> final visible / launchable agent set
```

这也意味着当前对“某个 agent 为什么没生效”的排查顺序应改成：

1. 是否进入了 source load
2. 是否在 `MV(...)` 里输给更高优先级 source
3. 是否被 `requiredMcpServers` 裁掉
4. 是否被 deny rule 裁掉
5. 是否不在 `allowedAgentTypes` 白名单里

## 祖先目录遍历规则

`sj()` 会把当前工作目录一直向上推到文件系统根，再按 `root -> cwd` 顺序依次加载。  
这意味着更靠近当前项目的 `CLAUDE.md` 会排在后面，天然具备“更近路径覆盖更远路径”的追加效果。  
但这个“覆盖”当前更应理解为：

- 在 `userContext.ClaudeMd` 文本串里的后追加
- 随后通过 `Lx8(...)` 前插到消息链最前部

## worktree 相关结论

扫描逻辑里显式比较了当前目录、worktree 根和 git 根。  
高可信推断是：**当 CLI 运行在 git worktree 中时，会跳过位于 worktree 根之外、但仍在仓库主根之内的那一段 project instructions**，以避免把主仓库根的 project prompt 无条件灌进当前 worktree 会话。

## external include 相关结论

- `Managed / Project / Local / additionalDirectories` 的 external include 受审批位控制
- `User` 级 `CLAUDE.md` 与 user `.claude/rules/` 走的是 `includeExternal: true`
- `xy()` 有递归深度上限 `PQ9 = 5`
- `HQ9()` 会剥离 HTML comments，并从 markdown/token 流里提取 `@include`
- 非文本文件扩展名会被跳过

## 当前未完全钉死

- 远端服务端路径里，`remote-control / sdk-url` 对接的 server-side transport 是否会在服务端额外拼装 compat / verification 指令
- `dtw` 这类 verifier prompt 资产在更完整 build 中是否曾被接到真实 agent 注册或 launch path 上

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
