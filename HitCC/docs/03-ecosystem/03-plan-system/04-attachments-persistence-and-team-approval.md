# Plan Attachments、持久化保留链与团队审批

## 本页用途

- 单独承接 plan attachments、compact/resume/clear-context 的保留链，以及 teammate -> leader 的审批协议。
- 把已钉死和未完全钉死的边界集中收口。

## 相关文件

- [../03-plan-system.md](../03-plan-system.md)
- [01-runtime-objects-and-plan-file-lifecycle.md](./01-runtime-objects-and-plan-file-lifecycle.md)
- [../01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md](../01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md)
- [../../02-execution/05-attachments-and-context-modifiers/02-high-value-attachment-payloads-and-materialization.md](../../02-execution/05-attachments-and-context-modifiers/02-high-value-attachment-payloads-and-materialization.md)

## 7. Plan Attachment 协议现在已经可以表化

当前至少确认 4 种 plan attachment：

| attachment type | producer | payload 形状 | 主要用途 |
| --- | --- | --- | --- |
| `plan_mode` | `v6z(...)` / `TVq(...)` | `{ type, reminderType: "full" \| "sparse", isSubAgent: boolean, planFilePath: string, planExists: boolean }` | 在 plan mode 中给主循环注入 planning 约束与 plan file 信息 |
| `plan_mode_reentry` | `v6z(...)` | `{ type, planFilePath: string }` | 再次进入 plan mode 时提醒先读旧 plan |
| `plan_mode_exit` | `T6z(...)` | `{ type, planFilePath: string, planExists: boolean }` | 刚退出 plan mode 后提醒现在可执行 |
| `plan_file_reference` | `cN8(...)` | `{ type, planFilePath: string, planContent: string }` | compact / keep 后把 plan 作为可还原上下文保留下来 |

### 7.1 产生时机

`plan_mode` / `plan_mode_reentry`：

- 仅在当前 mode 仍是 `plan` 时生成
- 如果最近已经出现过 `plan_mode` 或 `plan_mode_reentry`，且 user turn 数不足 5，则不重复注入
- reminder cadence：
  - `PM4.TURNS_BETWEEN_ATTACHMENTS = 5`
  - `PM4.FULL_REMINDER_EVERY_N_ATTACHMENTS = 5`

`plan_mode_exit`：

- 由 `V8.needsPlanModeExitAttachment` 驱动
- 只在“已经退出 plan mode”且该位为真时生成一次

`plan_file_reference`：

- 当前只在 compact 相关保留链里生成
- 已直接看到的调用点包括：
  - 全量 compact `jk6(...)`
  - partial compact
  - session-memory compact
- 只要当前还能读到 plan file，就会把完整 `planContent` 带上
- 它不是普通当前轮提醒，而是 compact / 还原链保留的 plan file 快照

### 7.2 attachment 与状态位的关系

状态位之间的关系当前已能写实：

- `Hd(oldMode, newMode)`
  - `old !== "plan" && new === "plan"` 时清空 `needsPlanModeExitAttachment`
  - `old === "plan" && new !== "plan"` 时置 `needsPlanModeExitAttachment = true`
- `ExitPlanMode.call(...)`
  - 退出成功后：
  - `wE(true)` 记录“本 session 已退出过 plan mode”
  - `Nb(true)` 标记应产生 `plan_mode_exit`
- `v6z(...)`
  - 若 `yS6() === true` 且 plan file 仍存在
  - 先发 `plan_mode_reentry`
  - 然后 `wE(false)`，避免反复重复
- `T6z(...)`
  - 发出 `plan_mode_exit` 后 `Nb(false)`

这说明 reentry/exit attachment 不是通过扫描 transcript 临时猜出来的，而是有显式 session bit 驱动。

### 7.3 消费侧不是单一展示

同一种 attachment 至少有两类消费面：

- transcript/TUI 列表渲染
  - `plan_file_reference` 在消息列表里只显示 `Plan file referenced (<path>)`
- prompt 注入
  - `dt1(...)` 会把 plan attachment 统一 materialize 成可还原的 planning context

因此 attachment 不能只记“名字有哪些”，还必须区分：

```text
transcript renderer
vs
prompt materializer
```

更细的 payload 字段与 `dt1(...)` materialize 规则，见：

- [../../02-execution/05-attachments-and-context-modifiers/02-high-value-attachment-payloads-and-materialization.md](../../02-execution/05-attachments-and-context-modifiers/02-high-value-attachment-payloads-and-materialization.md)

本页只保留 plan 主题独有的结论：

- 哪些 attachment 由 plan runtime 状态位驱动
- compact / resume / clear-context 如何保住 plan file
- teammate approval 如何把 plan mode 真正迁移到执行态

## 8. Compact / Resume / Clear-Context 三条保留链

## 8.1 Compact keep

手动 compact 与自动 compact 之后，plan 不会自然消失：

- `cN8(...)` 会把当前 plan file 归档成 `plan_file_reference`
- `TVq(...)` 在仍处于 `plan` mode 时会额外生成 `plan_mode(full)`

这说明 compact 的 keep attachments 明确把 plan 当成高优先级状态，而不是普通历史文本。

## 8.2 Resume

resume 时 plan file 缺失，会从：

1. `file_snapshot(plan)`
2. `assistant ExitPlanMode.input.plan`
3. `user.planContent`
4. `plan_file_reference.planContent`

里反推还原。

## 8.3 clear-context after plan approval

clear-context 接受计划时，并不会丢掉 plan file：

- 清上下文前取当前 slug
- 清空消息后再把 slug 重新绑回当前 session

因此 clear-context 的真实语义是：

```text
conversation reset
!=
plan file reset
```

另外还有一条之前没钉死、现在可以收紧的边界：

- clear-context effect 里虽然出现了 `pendingPlanVerification`
- 但对应赋值条件是：
  - `let cA = w1.message.planContent && !1`

也就是当前本地 bundle 里，这条 plan verification 后处理分支是常假分支。  
因此不能把“clear-context 之后还会进入额外 plan verification 流程”写成已确认事实。

## 9. Teammate -> Leader Plan Approval

team 场景下，`ExitPlanMode` 不直接切到实施，而会走 mailbox 协议：

### 9.1 request schema

```ts
{
  type: "plan_approval_request"
  from: string
  timestamp: string
  planFilePath: string
  planContent: string
  requestId: string
}
```

### 9.2 response schema

```ts
{
  type: "plan_approval_response"
  requestId: string
  approved: boolean
  feedback?: string
  timestamp: string
  permissionMode?: PermissionMode
}
```

### 9.3 行为

teammate 调 `ExitPlanMode` 时：

- 读取当前 plan file
- 发 `plan_approval_request` 给 `team-lead`
- 当前 task 置 `awaitingPlanApproval: true`
- tool_result 返回：
  - `awaitingLeaderApproval: true`
  - `requestId`

leader 侧 approve 时：

- 生成 `plan_approval_response`
- `permissionMode` 取 leader 当前 mode
- 若 leader 当前本身在 `plan`，则降成 `default`
- 当前本地 inbox poller 可见实现里，leader 收到 request 后会直接：
  - `Found X plan approval request(s), auto-approving`
  - 没看到额外人工 gate

teammate 列表 UI 里也有专门状态：

- `awaitingPlanApproval === true`
- 直接显示 `[awaiting approval]`

teammate 收到 response 后，当前本地 bundle 里已能确认两条落地动作：

- foreground runtime：
  - inbox poller 解析 `plan_approval_response`
  - 若 `approved === true`
    - 立即按 `permissionMode ?? "default"` 做 `setMode`
    - 这才是 teammate 真正退出 `plan` 的时点
  - 若 `approved === false`
    - 保持 planning，日志/邮箱里展示 feedback
- task state：
  - `Ww4(...)` 会把对应 task 的 `awaitingPlanApproval` 清回 `false`
  - 它本身不做 mode 迁移，mode 迁移发生在 inbox poller 处理 response 时

因此 team plan approval 不是 UI 幻象，而是 task runtime 的真实状态位。

## 10. 目前已经钉死与仍未完全钉死

### 已经钉死

- plan file 是 per-session/per-agent 的独立磁盘对象，不是消息文本。
- `/plan`、`EnterPlanMode`、`ExitPlanMode`、attachments、compact/resume 在同一套 Plan runtime 里。
- `plan_mode / plan_mode_reentry / plan_mode_exit / plan_file_reference` 的 producer、主要 payload 与 prompt 语义已经明确。
- 本地 TUI 的 plan 编辑主链是外部 editor bridge，不是大型内嵌编辑器。
- `planWasEdited` 在本地工具层的直接判定条件是“本次 tool input 是否显式带 `plan`”。
- `CCR/web` 的 edited-plan 回传主桥已经明确：
  - 远端 `tool_result` 用 markdown marker 标记
  - 本地 `B2z(...)` 解析后写入 `ultraplanPendingChoice.plan`
- 本地还存在一条 `plan file -> ExitPlanMode.input.plan` 的补强链：
  - `AAA(...) -> S2z(...) -> AP() -> tool_use.input.plan`
- `CCR/web` 侧当前可见的本地启动合同也已经明确：
  - `zAA(...) -> FC8(...)`
  - 只把 `set_permission_mode { mode:"plan", ultraplan:true }` 与初始 user message 注入远端 session
  - `ja(...) / yx4(...)` 只同步 `permission_mode / is_ultraplan_mode`
- `ultraplan` 的本地可见提醒主链已经明确：
  - `c2z(...)` 先把 remote task 置 `needsAttention`
  - `jC8(...)` / `PF4(oLz)` / `oB8(...)` / `b2z(...)` 把它呈现为 `ultraplan ready` 与 tasks detail
- `AVq(...)` 当前只有一条已确认正式用途：fork 新 session 时复制 plan file。
- `allowedPrompts` 在 schema/UI 上存在，但当前本地发行版里不只是 feature-gated：
  - `a16()` 常量 `false`
  - `_N8 / mkq / oT6 / gkq / zN8` 全部 stub 化
  - classifier prompt builder 里还把相关开关内联成常假分支
- teammate 收到 `plan_approval_response` 后的 mode 切换与 `awaitingPlanApproval` 清理路径已经明确。

### 仍未完全钉死

- CCR/web plan editor 的完整组件树与更细 dirty-state 分支。
- `planWasEdited` 的 web 侧 dirty-state 到 tool input 注入之间，仍缺少完整前端状态树；当前新增证据已能说明 CCR 控制面只携带 `mode/ultraplan` 合同，不携带 dirty payload，因此问题进一步收缩在远端 plan editor / approval UI 内部。本地侧已有 `Cm4(...)` 与 `S2z(...)` 两条 producer。
- `ultraplanPendingChoice` 的**专属**本地确认器仍没直接命中；当前能直接命中的本地 UI 主链其实是 `needsAttention -> tasks/bashes -> ultraplan ready/detail`，而 `ultraplanPendingChoice` 本身只看到弱消费。
- semantic prompt permission 为什么在当前发行版里被 gate/stub 掉，仍缺少更上层产品开关或构建分支证据。

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
