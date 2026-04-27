# Context 运行态与 ToolUseContext

## 本页用途

- 用来把目前散落在 prompt、tool runner、subagent、session state 多页里的 “Context” 概念收拢到同一页。
- 用来明确 `AppState / userContext / systemContext / ToolUseContext` 这四层上下文的职责边界。
- 用来减少后续继续把 `contextModifier`、sidechain 继承规则、fork/subagent 的上下文裁剪混写到其他页面。

## 相关文件

- [../01-runtime/02-session-and-persistence.md](../01-runtime/02-session-and-persistence.md)
- [../01-runtime/04-agent-loop-and-compaction.md](../01-runtime/04-agent-loop-and-compaction.md)
- [01-tools-hooks-and-permissions.md](./01-tools-hooks-and-permissions.md)
- [02-instruction-discovery-and-rules.md](./02-instruction-discovery-and-rules.md)
- [03-prompt-assembly-and-context-layering.md](./03-prompt-assembly-and-context-layering.md)
- [04-non-main-thread-prompt-paths.md](./04-non-main-thread-prompt-paths.md)
- [05-attachments-and-context-modifiers.md](./05-attachments-and-context-modifiers.md)
- [../05-appendix/01-glossary.md](../05-appendix/01-glossary.md)
- [../05-appendix/02-evidence-map.md](../05-appendix/02-evidence-map.md)

## Context 不是一个东西，而是四层

当前 bundle 里的 “Context” 至少应该拆成四层理解：

### 1. 全局 `AppState` / session store

- 这是最长生命周期的一层。
- 存放的是当前 cwd、sessionId、`invokedSkills`、`additionalDirectoriesForClaudeMd`、`systemPromptSectionCache`、权限上下文、TUI/UI 状态、MCP/plugin 状态等全局或会话级状态。
- `toolUseContext.getAppState()` 实际上就是访问这层。

### 2. request-level `userContext`

已直接确认由 `_$()` 生成，当前本地 bundle 的默认字段只有：

```ts
{
  ClaudeMd?: string
  currentDate: `Today's date is ${eU6()}.`
}
```

它不是最终 request 的 `system` 字段，而是会被 `Lx8(...)` 包成前置 `<system-reminder>` user meta message。

### 3. request-level `systemContext`

已直接确认由 `vO()` 生成，当前本地 bundle 的默认字段只有：

```ts
{
  gitStatus?: string
}
```

它会被 `dj4(...)` 通过 `Object.entries(systemContext)` 序列化后，追加到 system prompt sections 末尾。

### 4. per-turn / per-run `ToolUseContext`

- 这是工具执行、hook、subagent、compact 辅助路径真正传递的运行态上下文。
- 它不只是 prompt 片段，也不只是权限状态。
- 它同时承载：
  - 工具执行需要的即时状态
  - 当前 messages / options / abort controller
  - 访问 `AppState` 的桥
  - 若干只在本轮或本 sidechain 内有效的触发器与缓存

## `ToolUseContext` 的职责边界

从 `ts6(...)` 的 clone 逻辑可以直接反推，`ToolUseContext` 至少包含以下几类字段：

### 文件与内容状态

- `readFileState`
- `contentReplacementState`
- `fileReadingLimits`
- `updateFileHistoryState`
- `userModified`

### 技能 / memory / discovery 触发器

- `nestedMemoryAttachmentTriggers`
- `dynamicSkillDirTriggers`
- `discoveredSkillNames`

### 执行控制

- `abortController`
- `setInProgressToolUseIDs`
- `setResponseLength`
- `setStreamMode`
- `setSDKStatus`
- `openMessageSelector`
- `requireCanUseTool`

### app-state bridge

- `getAppState`
- `setAppState`
- `setAppStateForTasks`
- `localDenialTracking`
- `updateAttributionState`
- `addNotification`
- `setToolJSX`
- `pushApiMetricsEntry`

### request / agent 元数据

- `options`
- `messages`
- `agentId`
- `agentType`
- `queryTracking`
- `criticalSystemReminder_EXPERIMENTAL`

因此它更接近：

```ts
interface ToolUseContext {
  readFileState: MapLike
  nestedMemoryAttachmentTriggers: Set<string>
  dynamicSkillDirTriggers: Set<string>
  discoveredSkillNames: Set<string>
  toolDecisions?: unknown
  contentReplacementState?: {
    seenIds: Set<string>
    replacements: Map<string, unknown>
  }
  abortController: AbortController
  getAppState(): AppState
  setAppState(...): void
  setAppStateForTasks?: (...)
  localDenialTracking: unknown
  setInProgressToolUseIDs(...): void
  setResponseLength(...): void
  pushApiMetricsEntry?: (...)
  updateFileHistoryState(...): void
  updateAttributionState(...): void
  addNotification?: (...)
  setToolJSX?: (...)
  setStreamMode?: (...)
  setSDKStatus?: (...)
  openMessageSelector?: (...)
  options: RuntimeOptions
  messages: TranscriptMessage[]
  agentId: string
  agentType?: string
  queryTracking?: { chainId: string; depth: number }
  fileReadingLimits?: unknown
  userModified?: boolean
  criticalSystemReminder_EXPERIMENTAL?: unknown
  requireCanUseTool?: boolean
}
```

这里仍是文档化接口，不是源码中出现的正式类型名。

## `ts6(...)`：fork-family 的 `ToolUseContext` 克隆器

`ts6(parentCtx, overrides)` 现在可以明确看成 sidechain/fork-family 的上下文克隆器。

### 已直接确认的克隆规则

- `readFileState`
  - 总是通过 `bx(...)` 克隆
- `contentReplacementState`
  - 若 override 显式提供则直接用 override
  - 否则若 parent 有值，则通过 `J2q(...)` 深克隆
- `abortController`
  - 可复用父 controller，也可新建
- `options`
  - 默认继承 parent，可被 override 覆盖
- `messages`
  - 默认继承 parent，可被 override 覆盖
- `agentId`
  - 默认新生成
- `agentType`
  - 可由 override 指定
- `queryTracking`
  - 一定会生成新的 `chainId`
  - `depth = (parent.depth ?? -1) + 1`

### 已直接确认的重置规则

以下字段在 `ts6(...)` 中不是共享父状态，而是重建：

- `nestedMemoryAttachmentTriggers = new Set()`
- `dynamicSkillDirTriggers = new Set()`
- `discoveredSkillNames = new Set()`
- `toolDecisions = undefined`

这说明 fork-family 默认不会沿用父链已经触发过的动态发现状态。

### 已直接确认的“降权 / 静默化”规则

如果没有显式共享 `getAppState` / `setAppState` / `setResponseLength` 等能力，`ts6(...)` 会把很多 UI 相关能力替换成 noop 或更保守的包装：

- `setAppState` 可能变成 noop
- `updateFileHistoryState` 在 fork-family clone 中默认是 noop
- `setToolJSX` / `setStreamMode` / `setSDKStatus` / `openMessageSelector` 默认不可用
- `getAppState` 还会包一层，必要时把 `shouldAvoidPermissionPrompts` 置为 `true`

所以 `ToolUseContext` 不是简单复制，而是**按运行场景裁剪能力的 capability object**。

## `BN(...)`：subagent/sidechain 的 request context 继承规则

`BN(...)` 这条链同时处理 request-level context 和 tool-level context。

### `readFileState` 的继承

当前已直接看到：

```ts
let p = forkContextMessages !== undefined
  ? bx(K.readFileState)
  : Cx($F)
```

这意味着：

- 有 fork context 时，subagent 会带着父链 `readFileState` 的克隆进入
- 没有 fork context 时，不是共享父链，而是起一份新的 baseline

### `Cx($F)` / `Ly6(...)`：baseline 现在可以写得更实

把 `Cx / Zjq / Ly6 / av6` 这一组一起看完后，`readFileState` 不该再只写成抽象的 “MapLike”。

#### `Cx($F)` 不是普通 `Map`

当前可直接还原为：

```ts
const DEFAULT_READ_FILE_CACHE_MAX = 100
const DEFAULT_READ_FILE_CACHE_MAX_SIZE = 25 * 1024 * 1024

function Cx(max = DEFAULT_READ_FILE_CACHE_MAX) {
  return new Zjq(max, DEFAULT_READ_FILE_CACHE_MAX_SIZE)
}
```

而 `Zjq` 本身还做了两件关键事：

- key 会先做路径归一化
- 底层是带 `max / maxSize` 的有界缓存
- size 计算不是按对象大小，而是按 `Buffer.byteLength(entry.content)`

所以 `Cx($F)` 的真实含义更接近：

- 一份默认最多 100 条
- 总内容体积默认最多约 25MB
- 以归一化路径为 key
- 以 `content` 文本字节数计容量

#### `Ly6(...)` 会从 transcript 重建一份“full-file baseline”

headless / print 路径在启动时不是只拿一个空的 `Cx($F)`，而是同时构造：

- `N = Ly6(messages, cwd, $F)`：从 transcript 回放出的历史 full-file baseline
- `E = Cx($F)`：本轮新增的增量缓存

随后通过 `av6(N, E)` 按时间戳合并，较新的条目覆盖较旧条目。

`Ly6(...)` 当前已直接看到只回放两类 full-file 事件：

1. `FileReadTool`
   - 只接受 `offset === undefined && limit === undefined`
   - 只在成功 `tool_result` 上还原
2. `FileWrite`
   - 从 tool input 里的 `file_path + content` 还原

两条路径最终都写成同一种 entry 核心形状：

```ts
{
  content: string
  timestamp: number
  offset: undefined
  limit: undefined
}
```

这说明当前本地 bundle 里，“无 fork context 时的新 baseline”并不是某个神秘初始对象，而是：

- 基础容器：`Cx($F)` 生成的空有界缓存
- 在 resume / print 等入口上，再叠加 `Ly6(...)` 从 transcript 回放出的 full-file 已读状态

#### 当前本地 bundle 可见的 `readFileState` 写入族已经能枚举

若只看当前本地 bundle，可见的 producer 已能整理成下面几类：

1. 容器构造 / 复制
   - `Cx(...)`：生成空缓存
   - `bx(...)`：复制现有缓存
2. transcript 回放
   - `Ly6(...)`：从历史 `FileReadTool` full-file 成功结果、`FileWrite` 输入回放
3. 真实工具执行
   - `FileReadTool`
     - 文本 / notebook 路径写 `{ content, timestamp, offset, limit }`
   - `FileEditTool`
     - 写回编辑后的全文 `{ content, timestamp, offset: undefined, limit: undefined }`
   - `FileWriteTool`
     - 写回落盘后的全文 `{ content, timestamp, offset: undefined, limit: undefined }`
4. prompt / memory 侧运行态注入
   - CLAUDE.md / rules 装载
   - nested memory 注入
5. remote bridge 控制面
   - `seed_read_state`

因此如果讨论范围限定在**当前本地 bundle 可见代码**，`readFileState` 的来源已经基本闭环，不再只是“可能还有别的本地写入点待追”。

#### `isPartialView` 不是 baseline 核心字段，但确实是重要边缘位

现在能确认 `readFileState` 的核心字段仍然是：

- `content`
- `timestamp`
- `offset`
- `limit`

但某些非基线写入路径会额外带上：

- `isPartialView`

当前已直接看到：

- CLAUDE.md / rules 装载写入缓存时，若内容与磁盘不同，会写 `isPartialView: true`
- nested memory 注入缓存时也会沿用同样语义
- `Edit` / `Write` 的输入校验会把 `!entry || entry.isPartialView` 统一视为“尚未可靠读取全文”

因此更稳妥的接口近似应改成：

```ts
interface ReadFileStateEntry {
  content: string
  timestamp: number
  offset?: number
  limit?: number
  isPartialView?: boolean
}
```

### `userContext/systemContext` 的默认来源

```ts
userContext = override.userContext ?? _$()
systemContext = override.systemContext ?? vO()
```

因此 subagent/fork-family 默认不是“完全复用父线程已渲染后的 request context”，而是重新生成，再按分支裁剪。

### 已直接确认的裁剪

- `omitClaudeMd`
  - 当 agentDefinition 设了 `omitClaudeMd: true`
  - 且没有显式 override `userContext`
  - 会把 `userContext.ClaudeMd` 去掉
- `Explore` / `Plan`
  - 会把 `systemContext.gitStatus` 去掉
  - 不是整层 `systemContext` 失效，只是裁掉该字段

### permission / effort overlay 不是直接写死进 `ToolUseContext`

`BN(...)` 里对下列状态的变更，是通过包装后的 `getAppState()` 暴露给后续执行链，而不是直接改全局 store：

- `toolPermissionContext.mode`
- `toolPermissionContext.shouldAvoidPermissionPrompts`
- `toolPermissionContext.awaitAutomatedChecksBeforeDialog`
- `toolPermissionContext.alwaysAllowRules.session`
- `effortValue`

这点和 SkillTool 的 `contextModifier` 很像：都是**返回一个带包装 `getAppState()` 的新 context**，而不是立即硬改外层 app state。

## `contextModifier` 与 `ToolUseContext` 的边界

`contextModifier` 的接口、consumer、并发提交规则，以及当前本地 bundle 内已确认的 concrete producer，已经收敛到 [05-attachments-and-context-modifiers/03-context-modifier-and-executor-consumers.md](./05-attachments-and-context-modifiers/03-context-modifier-and-executor-consumers.md)。

本页只保留和 context runtime 直接相关的结论：

- `contextModifier` 不是 UI 附件，而是 `ToolUseContext -> ToolUseContext` 的运行态改写协议
- 执行器会把它应用到共享上下文，因此下一工具调用、下一 side-path、以及部分 fork/subagent helper 看到的是改写后的 `ToolUseContext`
- 当前本地 bundle 中，SkillTool 是唯一已正证的 tool-returned concrete producer；remote / bridge 的状态写入则是旁路缓存更新，不是同一协议

## `discoveredSkillNames`：当前更像未消费字段

这一项可以进一步收紧，不必继续只写“待补点”。

当前已直接看到：

- 主线程 `ToolUseContext` 会把外层 `nQ.current` 作为 `discoveredSkillNames` 传入
- SDK runner 也维护自己的 `this.discoveredSkillNames`
- `clearConversation(...)` 会显式对它执行 `clear()`
- `ts6(...)` 在 fork/subagent clone 时会把它重建成 `new Set()`

但对整份本地 bundle 做字符串级追踪后，仍然没有看到：

- `discoveredSkillNames.add(...)`
- `discoveredSkillNames.has(...)`
- `discoveredSkillNames.size`

与之相对，其他同层 trigger 都已经能看到完整消费链：

- `nestedMemoryAttachmentTriggers.add(...) -> p6z(...) -> nested_memory`
- `dynamicSkillDirTriggers.add(...) -> d6z(...) -> dynamic_skill`
- `skill_listing` 的增量发送则由另一组全局状态 `Qy6 / Vu8` 控制

因此当前更稳妥的判断是：

- `discoveredSkillNames` 在本地 bundle 中大概率是预留字段、旧字段残留，或为未启用路径保留的运行态槽位
- 至少在当前可见主线程 / SDK / fork-family 路径里，它**不是 skill 增量发送的主控制器**

## 当前更稳的总图

```text
AppState / session store
  -> getAppState()
  -> feeds permission / invokedSkills / prompt caches / additionalDirectories / UI state

request-level context
  userContext = _$() = { ClaudeMd?, currentDate }
  systemContext = vO() = { gitStatus? }
  -> consumed by Lx8 / dj4

tool-level context
  ToolUseContext
    = files + contentReplacement + triggers + app-state bridge + options/messages + abort/query metadata
  -> consumed by tools / hooks / subagents / compact helpers

runtime mutation
  tool.call(...)
    -> may return contextModifier
    -> executor applies modifyContext(ctx)
    -> next tool / next side-path sees updated ToolUseContext
```

如果把范围明确限制在**当前本地 bundle 可见代码**，那么下面两点已经可以视为闭环结论：

- `readFileState` 的构造、回放、工具写回、prompt/memory 注入、bridge 控制面写入来源都已能枚举
- `systemContext` 的本地来源只看到：
  - `vO()` 生成的 `{ gitStatus? }`
  - 显式 override 透传
  - 个别专用路径传入的空对象 `{}`

### `vO()` 预留第二槽位：在 Context 页里的更稳解释

就 Context 这一页的职责边界，可以把 `vO()` 里的第二槽位收紧成下面三件事：

1. `systemContext` 的序列化器 `dj4(...)` 是泛型 `Object.entries(...)`
2. `vO()` 的完成日志仍保留 `has_injection`
3. 但当前本地 `vO()` 实现里，对应变量固定为 `null`，返回对象也没有第二个有效字段

因此：

- 这个槽位说明 `systemContext` 设计上允许后续再扩字段
- 但当前本地 bundle 可执行路径里，**没有看到第二个默认字段进入 request-level `systemContext`**
- 在 Context 视角下，更合理的写法是“预留能力仍在，活数据流暂时不存在”，而不是“本地还有一个没找全的当前字段”
- 还能再补一条很关键的负面证据：
  - `BN(...)` 对 `Explore / Plan` 的通用裁剪只显式移除 `gitStatus`
  - 写法是 `{ gitStatus, ...rest } = systemContext`，随后仅在这两类 agent 上把 `rest` 继续传下去
  - 这意味着如果第二槽位当时是活字段，它**不会**被这条裁剪逻辑顺手去掉
  - 因此它看起来也不像“另一个与 gitStatus 同类、专门给 Explore/Plan 一起屏蔽的字段”

### 远端 transport 与 request-level context 的边界

这一页还需要把一个容易混淆的边界写清。

当前本地 bundle 可见代码里：

- `sdk-url / remote-control` 会切 transport / ingress
- 但 request-level context 仍由本地 `_$() / vO() / bC(...) / _I4(...)` 这一套生成
- 没有看到 bridge/session manager 在子进程外再改：
  - `userContext`
  - `systemContext`
  - `messages`
  - `system`

因此这里剩下的未知点是：

- 服务端收到本地 payload 后，是否再叠加一层黑箱 `systemContext`

而不是：

- 本地 bundle 里是否还有另一套隐藏的 context builder

## 当前仍未完全钉死

- `vO()` 里预留第二注入槽位的原始用途
- 远端/服务端路径是否还会额外叠加一层 server-side `systemContext`
- 远端/服务端侧是否存在本地 bundle 未暴露的第二个 `contextModifier` producer
- `discoveredSkillNames` 是否只在远端/灰度/已裁剪路径中被真正消费

## 结论

目前“Context 系统”的理解不应再停留在 prompt layering。

更稳妥的工程化结论是：

1. request-level context 只有 `userContext/systemContext` 两条 prompt 注入链
2. 真正驱动工具执行与 sidechain 行为的，是能力更强的 `ToolUseContext`
3. `contextModifier` 的核心价值，不是展示，而是把工具结果转成下一步运行态
4. fork/subagent 不是简单继承父上下文，而是“克隆部分状态 + 重置部分触发器 + 包装部分 capability”

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
