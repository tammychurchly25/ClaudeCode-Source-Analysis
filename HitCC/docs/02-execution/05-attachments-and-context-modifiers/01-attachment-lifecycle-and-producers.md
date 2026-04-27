# Attachment 生命周期与 Producer/Consumer

## 本页用途

- 单独说明 attachment 如何被生产、包装、进入 transcript、参与 compact / resume，以及最后如何进入或不进入 prompt。
- 把 attachment 运行时主链从 payload 细节中拆开。

## 总体链路

```text
input compilation / runtime state
  -> P6z(...)
  -> payload objects
  -> KE6(...)
  -> Nq(payload)
  -> transcript attachment items
  -> dt1(...)
     -> tool IO pair / user meta / []
  -> _X(...)
  -> request messages
```

## 包装层：`KE6(...)` / `Nq(...)`

这层职责很薄，但决定了 attachment 的真实身份：

```text
KE6(...)
  -> await P6z(...)
  -> for each payload:
       yield Nq(payload)
```

`Nq(...)` 产物已经是标准 transcript item：

```ts
interface TranscriptAttachmentEntry {
  type: 'attachment'
  attachment: AttachmentPayload
  uuid: string
  timestamp: string
}
```

因此 attachment 不是一开始就塞进 `user.message.content`，而是先独立落进 transcript。

## `P6z(...)`：主 producer

`P6z(...)` 先受两层总开关约束：

- `CLAUDE_CODE_DISABLE_ATTACHMENTS`
- `CLAUDE_CODE_SIMPLE`

命中任一都直接返回空数组。

之后它按三大类 producer 聚合 attachment。

## attachment loading gate：不是每轮都跑

普通输入链里，`ihz(...)` 只有在下面条件成立时才会调用 `KE6(...) / P6z(...)`：

```text
!isMeta
&& W !== null
&& (
     mode !== "prompt"
     || x
     || !W.startsWith("/")
   )
```

这意味着至少有三条硬边界：

- meta 输入不做 attachment loading
- 没有主文本时不做 attachment loading
- `prompt` 模式下，本地 slash command 不走普通 attachment loading

因此 attachment 不是“收到输入统一跑一遍”的全局前置步骤。

## 第一组：由用户文本触发的 attachment

### `x6z(...)`：`@"path"` / `@path`

会生成：

- `directory`
- `file`
  - `text`
  - `image`
  - `notebook`
  - `pdf`
- `pdf_reference`
- `already_read_file`

当前已能确认：

- 目录转目录 listing
- 大 PDF 退化成 `pdf_reference`
- 已读且未变化文件退化成 `already_read_file`
- `file#Lx-Ly` 会进入 `offset/limit` 路径

### `m6z(...)`：`@server:resource`

会生成：

- `mcp_resource`

### `u6z(...)`：`@agent-xxx`

会生成：

- `agent_mention`

这里只做点名声明，不在这一层直接发起 subagent。

## 第二组：按运行时状态自动附加的 attachment

这是 `P6z(...)` 的常驻 producer 区。

### 状态/提醒类

- `date_change`
- `ultrathink_effort`
- `critical_system_reminder`
- `output_style`
- `token_usage`
- `budget_usd`
- `output_token_usage`
- `verify_plan_reminder`
- `todo_reminder`
- `task_reminder`

### discovery / registry delta 类

- `deferred_tools_delta`
- `agent_listing_delta`
- `mcp_instructions_delta`
- `dynamic_skill`
- `skill_listing`

### 文件/记忆/诊断类

- `edited_text_file`
- `edited_image_file`
- `nested_memory`
- `relevant_memories`
- `diagnostics`
- `lsp_diagnostics`

### plan / auto / team 类

- `plan_mode`
- `plan_mode_reentry`
- `plan_mode_exit`
- `auto_mode`
- `auto_mode_exit`
- `teammate_mailbox`
- `team_context`

这里已有两个稳定边界：

- `team_context` / `teammate_mailbox` 只有 team runtime 开启时出现
- `teammate_mailbox` 在 `querySource === "session_memory"` 时会被跳过

## 第三组：仅主线程附加的 attachment

`P6z(...)` 会把一组 producer 限制在 `agentId` 为空的主线程。

### IDE / 输入侧

- `selected_lines_in_ide`
- `opened_file_in_ide`

### 任务/异步状态

- `queued_command`
- `task_status`
- `async_hook_response`

这一点要和“看起来像运行时状态类”区分开：

- `task_status`
- `async_hook_response`

在当前本地 bundle 中都属于主线程专属 producer，不是 subagent 常驻 attachment。

## attachment 到 prompt：`dt1(...)` 只是 consumer，不是原始来源

`dt1(...)` 不会无差别放行 attachment，而是按类型做三种处理：

1. 合成 synthetic tool transcript
2. 转成 user meta message
3. 直接丢弃 `[]`

payload 细节见：

- [02-high-value-attachment-payloads-and-materialization.md](./02-high-value-attachment-payloads-and-materialization.md)

## attachment 与 transcript / UI 的边界

### transcript 中先有 attachment

当前轮顺序可收紧成：

```text
current-turn user
-> P6z(...) attachments
-> UserPromptSubmit hook attachments
```

也就是说：

- 输入编译阶段 attachment
- hook 合并阶段 attachment

要分开看，不能混成一个来源。

### UI 中很多 attachment 会被隐藏或弱显示

bundle 内存在一组特殊隐藏/折叠类型，例如：

- `hook_success`
- `hook_additional_context`
- `command_permissions`
- `agent_mention`
- `output_style`
- `plan_mode`
- `structured_output`
- `team_context`
- `todo_reminder`
- `deferred_tools_delta`
- `mcp_instructions_delta`
- `token_usage`
- `ultrathink_effort`
- `output_token_usage`
- `verify_plan_reminder`
- `date_change`

这再次说明 attachment 是运行时控制层对象，不等于用户稳定可见正文。

## compact / resume：attachment 的 keep 与 restore

### compact keep 顺序

`jk6(...)` 与 `fVq(...)` 的保留 attachment 顺序同构：

```text
restored_files
-> task_status
-> plan_file_reference
-> plan_mode
-> invoked_skills
-> deferred_tools_delta
-> agent_listing_delta
-> mcp_instructions_delta
```

两者外层返回骨架不同：

- full compact：`boundaryMarker -> summaryMessages -> attachments -> hookResults`
- partial compact：`boundaryMarker -> summaryMessages -> messagesToKeep -> attachments -> hookResults`

这说明 compact 不只是保留“摘要 + 少量消息”，而是明确维护一组高价值 attachment 状态。

### resume restore 边界

当前至少有三条已闭环还原链：

1. `plan_file_reference`
   - `mN8(...)` 在 resume 时负责还原当前 slug 对应 plan file
   - fork 还原时则走 `AVq(...)` 复制旧 plan 文件到新 slug
2. `invoked_skills`
   - `Su_(messages)` 会扫描历史 attachment
   - 遇到 `invoked_skills` 就重新 `NJ6(...)` 装回运行态
3. compact keep 的其它 attachment
   - 由 `dt1(...)` 在后续 prompt materialize 时继续生效

当前 resume 主链可以概括成：

```text
I76(...)
  -> mN8(...) restore plan file when needed
  -> Su_(messages) restore invoked skills runtime
  -> hq4(...) fix interrupted turn state
  -> BD("resume", ...)
```

而 UI 还原路径还能进一步确认：

```text
fork resume
  -> AVq(d8, FM(sessionId))

non-fork resume
  -> mN8(d8, FM(sessionId))
```

因此 attachment 还原边界不能简单写成“resume 后消息里还在”，而应拆成三类：

- 还原磁盘状态
- 还原运行态 Map / registry
- 还原后续 prompt 可见 meta context

## 当前仍未完全钉死

- 低频/UI-only attachment 的完整字段族还能继续补表。
- compact / resume 保留 attachment 的少量边缘类型，当前仍更适合按专题继续追，而不是继续堆回总览页。

## 证据落点

- `cli.js`
  - `P6z(...)`
  - `D6z(...) / f6z(...)`
  - `Su_(...)`、`I76(...)`
  - resume UI 主链中的 `AVq(...) / mN8(...)`
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
