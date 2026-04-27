# 高价值 Attachment Payload 与 Materialization

## 本页用途

- 只处理高价值 attachment 的 payload 形状与 `dt1(...)` 的 materialize 规则。
- 不重复 attachment producer 全貌与 `contextModifier` consumer。

## `dt1(...)` 的三类输出

### 1. 转成工具调用/结果对

这一类会被 materialize 成 synthetic tool transcript：

- `directory`
- `file`
  - `text`
  - `image`
  - `notebook`
  - `pdf`

效果是：

- 先合成一次“Called the X tool with the following input: ...”
- 再合成一次“Result of calling the X tool: ...”

因此文件 at-mention 在最终 prompt 里更像一次伪工具调用，而不是普通说明文字。

### 2. 转成 user meta message

当前已直接确认的高价值类型包括：

- `compact_file_reference`
- `pdf_reference`
- `selected_lines_in_ide`
- `opened_file_in_ide`
- `plan_file_reference`
- `invoked_skills`
- `skill_listing`
- `todo_reminder`
- `task_reminder`
- `nested_memory`
- `relevant_memories`
- `queued_command`
- `output_style`
- `diagnostics`
- `plan_mode`
- `plan_mode_reentry`
- `plan_mode_exit`
- `auto_mode`
- `auto_mode_exit`
- `critical_system_reminder`
- `mcp_resource`
- `agent_mention`
- `task_status`
- `async_hook_response`
- `token_usage`
- `budget_usd`
- `output_token_usage`
- `hook_blocking_error`
- `hook_success`
- `hook_additional_context`
- `hook_stopped_continuation`
- `compaction_reminder`
- `date_change`
- `ultrathink_effort`
- `deferred_tools_delta`
- `agent_listing_delta`
- `mcp_instructions_delta`
- `verify_plan_reminder`
- `teammate_mailbox`
- `team_context`

### 3. 直接丢弃 `[]`

当前已直接确认：

- `dynamic_skill`
- `context_efficiency`
- `already_read_file`
- `command_permissions`
- `edited_image_file`
- `hook_cancelled`
- `hook_error_during_execution`
- `hook_non_blocking_error`
- `hook_system_message`
- `structured_output`
- `hook_permission_decision`

因此 attachment 类型不等于 prompt 可见类型。

## `queued_command`

`queued_command` 的消费规则最特殊。

### 语义

它不是普通提示，而是“延迟提交的下一轮 user input”。

### producer 来源

#### 本地排队命令：`D6z(...)`

payload 至少包含：

- `prompt`
- `source_uuid`
- `imagePasteIds?`
- `commandMode`
- `origin?`
- `isMeta?`

#### team/coordinator 待处理消息：`f6z(...)`

payload 至少包含：

- `prompt`
- `origin: { kind: "coordinator" }`
- `isMeta: true`

### `dt1(...)` materialize 规则

- 若 `prompt` 是 block 数组：
  - 先合并所有 text block
  - 图片 block 原样保留
  - 最终生成一条新的 user message
- 若有 `origin` 或 `isMeta`
  - 生成的新 user message 会继续带 `isMeta`
- `origin` / `uuid(source_uuid)` 会继续保留到新 user message

所以对实现者来说，`queued_command` 更像“未来要 materialize 的 user turn”。

## `plan_file_reference`

这不是普通当前轮 producer，而是 compact / 还原链专门保留的 attachment。

```ts
interface PlanFileReferenceAttachment {
  type: 'plan_file_reference'
  planFilePath: string
  planContent: string
}
```

当前本地 producer 直接命中的是：

- `cN8(agentId)`
  - 读 `AP(agentId)` 当前 plan 文件内容
  - 包成 attachment

`dt1(...)` 会把它转成一条 meta user message，明确告诉模型：

- plan file 路径
- plan file 正文
- 若仍相关且未完成，应继续沿着该 plan 工作

因此它的职责不是“提醒 plan mode 开过”，而是提供一个可还原的 plan file 快照。

## `invoked_skills`

`invoked_skills` 不是普通当前轮发现附件，而是 compact / resume 保留附件。

`dt1(...)` materialize 时会把全部 skill 内容串成：

```text
The following skills were invoked in this session. Continue to follow these guidelines:
```

并逐个展开：

- `Skill: <name>`
- `Path: <path>`
- `<content>`

因此从 `dt1(...)` 的角度看，它的关键语义是：把“已生效 skills”重新注入成持续生效的 meta context。  
更细的还原链、全局 `invokedSkills` 状态与清理规则，见 [../../03-ecosystem/05-skill-system.md](../../03-ecosystem/05-skill-system.md)。

## `relevant_memories`

```ts
interface RelevantMemoryItem {
  path: string
  content: string
  mtimeMs: number
  limit?: number
}

interface RelevantMemoriesAttachment {
  type: 'relevant_memories'
  memories: RelevantMemoryItem[]
}
```

关键边界：

- 最多保留 5 个 memory
- 会排除：
  - 已在 `readFileState` 中的路径
  - 先前 `relevant_memories` 已带过的路径
- 单个 memory 被截断时：
  - `content` 末尾追加截断说明
  - `limit` 记录保留行数

`dt1(...)` 会逐条展开正文，不做摘要化。

## `mcp_resource`

```ts
interface McpResourceAttachment {
  type: 'mcp_resource'
  server: string
  uri: string
  name: string
  description?: string
  content: ReadResourceResult
}
```

producer 来自：

- `m6z(...)`
  - `@server:resource` 命中
  - client 已连接
  - `options.mcpResources[server]` 中存在该 `uri`
  - 调 `client.readResource({ uri })`

`dt1(...)` 规则：

- `content.contents` 为空
  - 输出 `(No content)`
- 某项含 `text`
  - 输出正文
  - 追加“不要重复读该资源”的提醒
- 某项含 `blob`
  - 输出二进制占位说明

所以它更接近“已读资源正文快照”。

## `task_status`

`q8z(...)` 当前可确认产出：

```ts
{
  type: 'task_status'
  taskId: string
  taskType: string
  status: string
  description: string
  deltaSummary?: string
  outputFilePath?: string
}
```

producer 至少有两路：

- `q8z(...)`
  - 普通当前轮 unified task delta
- `kVq(...)`
  - compact 时补带未取回的 `local_agent` 结果

`dt1(...)` materialize 规则：

- `status === "killed"`
  - 转成“任务被用户停止”
- `status === "running"`
  - 强调仍在运行
  - 不要重复 spawn
  - 若有 `deltaSummary`，附带 progress
  - 若有 `outputFilePath`，提示直接读该文件
- 其它状态
  - 统一转成 `Task <id> (type/status/description/Delta/Read output...)`

因此 `task_status` 是明确进入 prompt 的后台任务态提醒。

## `async_hook_response`

`K8z(...)` 会把异步 hook registry 中已完成响应转成：

```ts
{
  type: 'async_hook_response'
  processId: string
  hookName: string
  hookEvent: string
  toolName?: string
  response: HookResponse
  stdout?: string
  stderr?: string
  exitCode?: number
}
```

但 `dt1(...)` 对它的消费非常克制，只吃：

- `response.systemMessage`
- `response.hookSpecificOutput.additionalContext`

因此下面这些字段当前不会直接展开进主 prompt：

- `stdout / stderr / exitCode`
- `processId / hookEvent / toolName`

它们更接近 transcript / 调试侧带。

## `hook_additional_context`

这是 hook 系统共用的追加上下文桥接协议。

### producer

- `UserPromptSubmit`
- `SessionStart`
- `Setup`

### consumer

- `dt1(...)` 会把它转成一条 user meta message

### 持久化边界

- attachment 默认不进入普通 session log
- 只有 `hook_additional_context` 在 `CLAUDE_CODE_SAVE_HOOK_ADDITIONAL_CONTEXT` 打开时例外保留

所以它主要是“当前轮/当前会话上下文桥”，不是稳定 transcript 一等结构。

## `dynamic_skill` 与 `skill_listing`

两者名字接近，但本页只记录 `dt1(...)` 的消费差异；更细职责、增量发送与还原边界见 [../../03-ecosystem/05-skill-system.md](../../03-ecosystem/05-skill-system.md)。

### `dynamic_skill`

- `dt1(...)`：直接丢弃

### `skill_listing`

- `dt1(...)`：转成 “The following skills are available...” meta message

因此不能把二者都粗写成“skills attachment”。

## `team_context` 与 `teammate_mailbox`

### `team_context`

由 `z8z(...)` 产出，至少包含：

```ts
{
  type: 'team_context'
  agentId: string
  agentName: string
  teamName: string
  teamConfigPath: string
  taskListPath: string
}
```

`dt1(...)` 会把它 materialize 成一整段 `<system-reminder>`，内容包括：

- 当前 teammate 身份
- team config 路径
- task list 路径
- leader 固定名 `team-lead`
- 必须按名字而不是 UUID 发消息

### `teammate_mailbox`

- 不自己拼固定模板
- 直接走 `I0z().formatTeammateMessages(A.messages)`

因此它更接近 mailbox 内容投递。

## usage attachments

### `token_usage`

- producer：`Y8z(...)`
- payload：`{ used, total, remaining }`
- 受 `CLAUDE_CODE_ENABLE_TOKEN_USAGE_ATTACHMENT` 开关控制

### `budget_usd`

- producer：`$8z(maxBudgetUsd)`
- payload：`{ used, total, remaining }`
- 只有 `maxBudgetUsd !== undefined` 时生成

### `output_token_usage`

- `dt1(...)` 明确支持 payload：
  - `turn`
  - `budget | null`
  - `session`
- 但当前本地 producer `w8z()` 直接返回 `[]`

因此它应视为：

- 消费分支已接好
- 本地 producer 未启用
- 当前本地 bundle 内更接近死分支

## 本页结论

1. 高价值 attachment 已能字段化到足以支持还原运行时语义。
2. `dt1(...)` 的 materialize 规则已经能区分：
   - 延迟 user turn
   - 可还原 plan/skill/runtime 状态
   - 纯本地侧带
3. `output_token_usage` 当前不能被写成稳定运行时信号。

## 证据落点

- `cli.js`
  - `plan_file_reference`、`invoked_skills`、`task_status`、`async_hook_response`、usage / delta attachments 的 materialize
  - `queued_command`、`task_status`、`async_hook_response`
- [../../03-ecosystem/03-plan-system.md](../../03-ecosystem/03-plan-system.md)
- [../../03-ecosystem/05-skill-system.md](../../03-ecosystem/05-skill-system.md)

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
