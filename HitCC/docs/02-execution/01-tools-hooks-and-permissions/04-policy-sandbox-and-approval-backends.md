# Managed Policy、Sandbox 与审批 Backend

## 本页用途

- 单独整理 managed policy 如何改写 permission 规则来源，以及 sandbox 如何与 permission rules 合流。
- 单独整理 ask 之后的统一审批协议，包括 TUI、remote/direct/ssh、headless/SDK/bridge 与 MCP permission prompt tool。

## 相关文件

- [../01-tools-hooks-and-permissions.md](../01-tools-hooks-and-permissions.md)
- [../04-non-main-thread-prompt-paths.md](../04-non-main-thread-prompt-paths.md)
- [../06-context-runtime-and-tool-use-context.md](../06-context-runtime-and-tool-use-context.md)
- [../../03-ecosystem/01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md](../../03-ecosystem/01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md)
- [../../03-ecosystem/02-remote-persistence-and-bridge.md](../../03-ecosystem/02-remote-persistence-and-bridge.md)
- [../../03-ecosystem/03-plan-system.md](../../03-ecosystem/03-plan-system.md)

## Permission Policy、Sandbox 与审批合流

### managed policy 对普通 permission rules 的覆盖不是“merge 后优先级更高”，而是会改写来源集

`allowManagedPermissionRulesOnly` 对应的是 `gQ6()`。

在 `qP4(...)` 中，如果这个开关开启，会先把这些 destination 的 allow/deny/ask 全部清空：

- `userSettings`
- `projectSettings`
- `localSettings`
- `cliArg`
- `session`

然后再重建规则集。

这意味着对普通 permission rules，更准确的还原应写成：

- managed policy 不是单纯“排前面”
- 而是会让 user/project/local/CLI/session 这些来源整体失效

这里还有一个值得单独记录的实现边界，也是之前最容易误判成“可能漏清”的点：

- 当前本地清空列表里**没有** `command`

现在可以把这一点再收紧。

#### `command` 更像运行时 overlay，不是普通 settings source

`toolPermissionContext` 的 destination 总表里，当前至少包括：

- `userSettings`
- `projectSettings`
- `localSettings`
- `policySettings`
- `flagSettings`
- `cliArg`
- `command`
- `session`

但 `command` 这一层和前面几类 source 的性质并不一样。

当前本地 bundle 可直接看到两条活写入路径：

1. slash-command / prompt command 编译结果里的 `allowedTools`
   - `AU8(...)` 会产出 `allowedTools`
   - REPL 主线程与 SDK/headless 路径都会把它写到：
     - `toolPermissionContext.alwaysAllowRules.command`
2. SkillTool / fork skill 的运行态授权
   - skill 的 `allowedTools` 会通过包装后的 `getAppState()` 或 query 前置更新
   - 合并进同一个 `alwaysAllowRules.command`

这说明 `command` 不是“设置文件里另一份静态 permission rules”，而是：

- 当前命令/skill/slash-command 编译阶段注入的运行时 allow overlay
- 生命周期更接近“本次 query / 当前 prompt 派生授权”
- 不是 user/project/local/policy 这些持久化来源的同类项

#### `command` 当前还带两个“只读/非持久化”信号

这层现在还能再写硬一点：

- `jQ(...)` 的持久化分支只会落 user/project/local settings，不会把 `command` 写回 settings 文件
- 删除规则时，`WE4(...)` 直接把 `command` 和 `policySettings / flagSettings` 一起视为 read-only source

因此本地实现更像是在表达：

- `command` 可以参与 permission merge
- 但它不是用户可持久编辑的配置来源
- 更像“由运行时编译链产生、在当前上下文生效的临时授权层”

因此更稳妥的理解应改成：

- managed-only 主要封住的是 settings / CLI / session 这些**配置来源**
- `command` 被刻意保留，更像为了不打断 slash-command / skill / local command 已经显式声明的运行时能力边界
- 所以它不像遗漏，更像产品上有意保留的一层“command-scope runtime authorization”

### sandbox 与 permission rules 的合流发生在 `RG8(...)`

`RG8(settings)` 会把普通 permission rules 与 sandbox settings 合成为一份真正下发给 sandbox runtime 的配置。

#### 网络侧合流

allow side 当前至少来自两类来源：

1. `sandbox.network.allowedDomains`
2. `WebFetch(domain:...)` 形式的 allow permission rules

deny side 则来自：

- `WebFetch(domain:...)` 形式的 deny permission rules

#### `allowManagedDomainsOnly`

若 `policySettings.sandbox.network.allowManagedDomainsOnly === true`，allow side 会收缩成：

- `policySettings.sandbox.network.allowedDomains`
- `policySettings.permissions.allow` 中的 `WebFetch(domain:...)`

但 deny side 仍继续读取所有来源。  
因此它不是“完全只看 managed”，而是：

- allow 只看 managed
- deny 仍是全来源收口

#### 文件系统侧合流

当前可直接确认：

- `Edit(...)` allow rule -> 并入 sandbox `allowWrite`
- `Edit(...)` deny rule -> 并入 sandbox `denyWrite`
- `Read(...)` deny rule -> 并入 sandbox `denyRead`
- `sandbox.filesystem.allowWrite/denyWrite/denyRead/allowRead`
  - 再继续叠加到对应字段

也就是说 sandbox 不是另一套独立的文件权限表，而是直接消费 permission rule 的路径级结果。

#### `allowManagedReadPathsOnly`

若 `policySettings.sandbox.filesystem.allowManagedReadPathsOnly === true`，`allowRead` 只保留：

- `policySettings.sandbox.filesystem.allowRead`

其他来源的 `allowRead` 会被忽略。  
但 `denyRead` 并没有同步被缩成 managed-only。

#### `autoAllowBashIfSandboxed` 的真实含义

它不是“开了 sandbox 就免审批跑 Bash”。

当前本地逻辑更准确是：

1. 先按 bash 子命令做 rule 检查
2. 若命中 deny，仍然 deny
3. 若命中 ask，仍然 ask
4. 只有在所有子命令都能被 sandbox 包住、且没有 ask/deny 冲突时
   - 才返回 `allow`
   - `decisionReason` 记为 `Auto-allowed with sandbox`

因此它是“sandbox 成功覆盖后的一条 allow shortcut”，不是对规则系统的总绕过。

#### `allowUnsandboxedCommands`

这个开关控制的是：

- `dangerouslyDisableSandbox` 这类 unsandboxed fallback 是否真的生效

当它为 `false` 时，即使命令自己请求 unsandboxed fallback，也会被 sandbox 侧硬关掉。

### permission prompt backend 还有一个 MCP tool 分支

当前普通本地 prompt backend 之外，还存在 `--permission-prompt-tool`：

- 只在 `--print` 路径生效
- 要求目标必须是 MCP tool
- `Aa4(...)` 会把它包装成 `canUseTool(...)`
- MCP tool 的返回文本再被解析成 allow / deny / ask 结果

因此 permission prompt 不一定总是本地 TUI/SDK 对话框，也可以外接一个 MCP 审批器。

### `YP(...) -> ask` 之后的外层审批 backend 现在可以收成一条统一协议

之前比较模糊的点，是 ask 之后到底是“三套系统”，还是“一套协议 + 多个 transport/UI”。

当前更稳的答案是后者。

#### 交互式 TUI 主入口：`Uhz(...)`

交互式主线程当前真正消费 `YP(...)` 结果的是 `Uhz(...)`。

它的顺序已经可以直接写成：

```text
YP(...)
  -> allow
     -> 直接放行
  -> deny
     -> 直接拒绝
  -> ask
     -> 若 awaitAutomatedChecksBeforeDialog:
          yU4()   // automated PermissionRequest hook / 自动静默检查
          -> hU4() // teammate/worker 审批上卷
     -> SU4()
          -> push toolUseConfirmQueue
          -> 绑定 bridge / channel / UI callbacks
          -> 必要时异步继续跑 hooks
```

这说明真正的“审批核心对象”不是某个弹窗组件，而是 `toolUseConfirmQueue item`。

#### `awaitAutomatedChecksBeforeDialog` 会改变自动检查的时机

这是这次补出来的关键细节。

- `awaitAutomatedChecksBeforeDialog === true`
  - 先跑 `yU4(...)`
  - 再跑 `hU4(...)`
  - 只有都没直接 resolve，才进入 `SU4(...)` 入队
- `awaitAutomatedChecksBeforeDialog === false`
  - 先进入 `SU4(...)` 入队
  - 然后 `SU4(...)` 内部再异步跑 `runHooks(...)`

因此“先静默检查还是先展示审批项”不是固定行为，而受 context 中这一位控制。

#### 这个位在 async / non-main-thread 场景下会被显式打开

`BN(...)` 当前可直接确认：

- 若 `isAsync === true`
- 且 permission prompts 没被整体避免

则会把：

- `toolPermissionContext.awaitAutomatedChecksBeforeDialog = true`

写进包装后的上下文。

因此 subagent / async worker 路径不是简单复用主线程 prompt 体验，而是会优先尝试“自动检查先行”。

#### `yU4(...)` 与 `hU4(...)` 的职责边界

- `yU4(...)`
  - 当前活逻辑主要是跑 `PermissionRequest` hooks
  - 若 hook 直接给出 allow/deny，会在入队前短路
- `hU4(...)`
  - 只在 `b7() && Ro6()` 条件下进入
  - 会构造 worker/teammate 权限请求
  - 当前更像 team/swarm 场景的审批上卷桥

这说明 automated checks 不只是“本地 heuristic”，还包括：

- hook 自动裁决
- teammate/leader 审批上卷

#### `SU4(...)` 才是真正的审批汇合点

`SU4(...)` 当前至少同时接三类下游：

1. 本地 TUI / 用户交互
2. `bridgeCallbacks`
3. `channelCallbacks`

其动作顺序可以收敛成：

- 先 `pushToQueue(...)`
- 生成统一的 queue item：
  - `assistantMessage`
  - `tool`
  - `description`
  - `input`
  - `toolUseID`
  - `permissionResult`
  - `onAbort/onAllow/onReject/recheckPermission`
- 若有 bridge callbacks
  - 立刻发 `sendRequest(...)`
  - 等待 `onResponse(...)`
- 若有 channel callbacks，且工具不需要直接用户交互
  - 给 channel-capable MCP server 发 `permission_request` notification
- 若当前不是“先自动检查再展示”
  - 再异步补跑 `runHooks(...)`

因此当前更准确的理解不是：

```text
TUI backend
SDK backend
bridge backend
三套互相独立的权限系统
```

而是：

```text
统一 queue item / callback 协议
  -> 本地 UI
  -> bridge transport
  -> channel notification
```

#### `recheckPermission()` 说明队列项不是静态快照

queue item 上当前都带：

- `recheckPermission()`

它会重新调用 `YP(...)`。  
若新的规则/模式已允许该工具，则会：

- 移除 queue item
- 直接 buildAllow

这也是为什么权限变更、mode 切换、rule 更新后，现有待审批项可以被自动重算。

### remote / direct-connect / SSH 侧不是另一套审批模型，而是把远端请求投影成同一种 queue item

`useRemoteSession`、`useDirectConnect`、`useSSHSession` 当前都能直接看到同一个模式：

1. 收到远端 `control_request(subtype="can_use_tool")`
2. 构造 synthetic `assistant` tool_use message
3. 生成本地 `permissionResult = { behavior:"ask", ... }`
4. 压入同一类 `toolUseConfirmQueue`
5. `onAllow/onReject/onAbort` 再回写 `respondToPermissionRequest(...)`

并且这三条路径生成的 queue item 字段结构几乎同构：

- `assistantMessage`
- `tool`
- `description`
- `input`
- `toolUseID`
- `permissionPromptStartTimeMs`
- `onAbort`
- `onAllow`
- `onReject`
- `recheckPermission`

因此远端审批当前更准确的还原应写成：

- 远端只负责把 `can_use_tool` request 发过来
- 本地 UI 仍使用和主线程一致的审批数据结构
- 差别只在最终响应通过 remote/direct/ssh manager 回送

### Headless / SDK / bridge 的 ask backend 也已能拆清

`runHeadless()` 里最终传给主循环的是：

- `Aa4("stdio", ...)`
- 或 `Aa4(permissionPromptToolName, ...)`
- 或 `YP`

三种之一。

#### `stdio`：结构化 control_request/control_response 协议

当走 `sdk-url` / stream-json / bridge 时，`Aa4("stdio", ...)` 会落到：

- `StructuredIO.createCanUseTool(...)`

它的实际流程是：

```text
YP(...)
  -> ask
  -> 并行等待:
       LSz(...) PermissionRequest hooks
       sendRequest({ subtype:"can_use_tool", ... })
  -> 谁先 resolve 用谁
  -> 另一边取消
```

因此 SDK/bridge 不是在 `YP` 外面再包一层独立审批，而是：

- 仍先过 `YP`
- ask 后通过 `control_request can_use_tool` 往外发
- 再把回包解析回同一套 allow/deny 结果

#### `permission-prompt-tool`

当指定 `--permission-prompt-tool` 时：

- ask 不走 `control_request`
- 而是调用 MCP tool
- tool 输出再经 `AR6(...)` 解析成 allow/deny

#### 纯 `YP`

若既不是 `stdio`，也没指定 `permission-prompt-tool`，headless 当前就直接使用 `YP`。  
因此 plain print/headless 不会天然拥有第二层交互 backend。

### 本地 `PermissionRequest` hooks 与 SDK `can_use_tool` request 的先后关系也已可收紧

在 `StructuredIO.createCanUseTool(...)` 里，当前是：

```text
hook promise = LSz(...)
sdk promise  = sendRequest(can_use_tool)
Promise.race([hook, sdk])
```

也就是说：

- hook 不再是永远先跑完再决定要不要发 SDK request
- 而是与 SDK permission request 并行竞争
- 谁先给出可用结论，谁先落地
- 另一侧随后取消

这比之前“hook -> SDK prompt”那种线性理解更准确。

### orphaned permission 不是日志概念，而是正式还原分支

在 stream-json / bridge 路径里，prompt queue 当前至少支持：

- `prompt`
- `orphaned-permission`
- `task-notification`

其中 `orphaned-permission` 的还原链已经比较明确：

1. 收到一个带 `toolUseID` 的 `control_response`
2. 发现对应 tool_use 在 transcript 中仍 unresolved
3. 把它重新包装成 `mode:"orphaned-permission"` 的 prompt item
4. 再交回同一条 prompt 处理主循环

这说明权限系统在 SDK / bridge 场景下并不要求“审批响应必须同步到场”，而是具备迟到响应的补偿还原能力。

### auto classifier 当前本地只有一个调用点，但有两套活实现

这次再往下追后，可以把“是不是还有其他本地 classifier 入口”收得更紧。

#### 唯一主调用点

当前本地 bundle 可见范围内，真正发起 auto classifier 的主调用点仍只看到一个：

- `YP(...)` 的 auto-mode ask 分支
  - `j = await vV8(...)`

没有看到第二个本地 runtime 会独立调用 classifier。

#### `vV8(...)` 不是单实现，而是 config-selectable dispatcher

`vV8(...)` 当前会按 `vK_()` 分成两条活路径：

1. `vK_() === true`
   - 走 `WK_(...)`
   - 即 XML classifier
   - 细分：
     - `fast`
     - `thinking`
     - `both/xml_2stage`
2. `vK_() === false`
   - 仍走旧的 `classify_result` tool-choice 路径

因此旧 `classify_result` 不是单纯遗留文本，而是当前仍可达的 fallback/默认实现分支。

#### 切换位

当前切换主要受：

- `tengu_auto_mode_config.twoStageClassifier`
- `tengu_auto_mode_config.jsonlTranscript`
- `tengu_auto_mode_config.model`

影响。

更具体地说：

- `twoStageClassifier`
  - 决定是否切到 XML classifier 族
- `jsonlTranscript`
  - 只改变 classifier transcript 的序列化格式
  - 不是另一套决策器
- `model`
  - 决定 classifier 自身用哪个模型

#### 仍能确认的边界

当前本地 bundle 内：

- 没看到第三套 classifier runtime
- 没看到另一个独立调用点
- 没看到“server-side classifier 本地桩”之类额外入口

因此这里更稳的结论应改成：

- 本地目前是“一处调用点，两套活实现”
- 而不是“很多可能的本地实验分支没找到”

### 当前仍未完全钉死

- channel permission request 的服务端/插件端消费者当前只看到 notification 下发与本地回调接线，远端 UI/插件侧具体呈现不可见
- auto mode classifier 当前已能确认本地只有一个调用点、两套活实现；仍不可见的是 bundle 外/服务端是否还有额外实验分支
- `allowManagedPermissionRulesOnly` 当前不会清掉 `command` slice；这更像刻意保留的运行时授权层，但 bundle 内未见更明确注释说明其产品语义

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
