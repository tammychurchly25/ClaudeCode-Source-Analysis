# Plan 系统

## 本页用途

- 这页不再承载全部细节，而改成 `03-ecosystem` 下 plan 主题的总览与导航。
- 原先混在一页里的内容，已经拆成文件与状态对象、enter/exit 与 `/plan`、审批 UI 与 edited-plan 回传、attachments/持久化/团队审批四个专题页。

## 相关文件

- [03-plan-system/01-runtime-objects-and-plan-file-lifecycle.md](./03-plan-system/01-runtime-objects-and-plan-file-lifecycle.md)
- [03-plan-system/02-enter-exit-and-plan-command.md](./03-plan-system/02-enter-exit-and-plan-command.md)
- [03-plan-system/03-exit-approval-ui-planwasedited-and-ultraplan-bridge.md](./03-plan-system/03-exit-approval-ui-planwasedited-and-ultraplan-bridge.md)
- [03-plan-system/04-attachments-persistence-and-team-approval.md](./03-plan-system/04-attachments-persistence-and-team-approval.md)
- [01-resume-fork-sidechain-and-subagents.md](./01-resume-fork-sidechain-and-subagents.md)
- [01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md](./01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md)
- [../01-runtime/02-session-and-persistence.md](../01-runtime/02-session-and-persistence.md)
- [../02-execution/05-attachments-and-context-modifiers.md](../02-execution/05-attachments-and-context-modifiers.md)
- [07-tui-system.md](./07-tui-system.md)

## 拆分后的主题边界

### 运行时对象模型 / plan file 生命周期

见：

- [03-plan-system/01-runtime-objects-and-plan-file-lifecycle.md](./03-plan-system/01-runtime-objects-and-plan-file-lifecycle.md)

这一页集中放：

- plan runtime object 模型
- `plansDirectory`、slug、per-agent plan file 路径
- plan file 的读写、还原、snapshot 与 fork 复制链

### `EnterPlanMode` / `ExitPlanMode` / `/plan`

见：

- [03-plan-system/02-enter-exit-and-plan-command.md](./03-plan-system/02-enter-exit-and-plan-command.md)

这一页集中放：

- plan mode 的正式状态迁移
- `ExitPlanMode` 工具输出 schema
- `/plan` 作为 mode entry、viewer 与 external-editor launcher 的本地入口

### Exit 审批 UI / `planWasEdited` / CCR-ultraplan 回传

见：

- [03-plan-system/03-exit-approval-ui-planwasedited-and-ultraplan-bridge.md](./03-plan-system/03-exit-approval-ui-planwasedited-and-ultraplan-bridge.md)

这一页集中放：

- `Cm4(...)` 审批 UI 状态机
- clear-context 与 keep-context 后继链
- `planWasEdited` 的真实判定条件
- `Ctrl+G`、CCR/web、`ultraplanPendingChoice` 与 `needsAttention` 链

### attachments / compact-resume / teammate approval

见：

- [03-plan-system/04-attachments-persistence-and-team-approval.md](./03-plan-system/04-attachments-persistence-and-team-approval.md)

这一页集中放：

- `plan_mode*` 与 `plan_file_reference`
- compact / resume / clear-context 的保留链
- teammate -> leader plan approval mailbox 协议
- 当前已钉死与仍未完全钉死的边界

## 建议阅读顺序

1. 先看 [01-runtime-objects-and-plan-file-lifecycle.md](./03-plan-system/01-runtime-objects-and-plan-file-lifecycle.md)，建立 plan file 与 mode state 的核心模型。
2. 再看 [02-enter-exit-and-plan-command.md](./03-plan-system/02-enter-exit-and-plan-command.md)，补齐正式状态迁移与 `/plan` 入口。
3. 然后看 [03-exit-approval-ui-planwasedited-and-ultraplan-bridge.md](./03-plan-system/03-exit-approval-ui-planwasedited-and-ultraplan-bridge.md)，理解退出审批和 edited-plan 回传链。
4. 最后看 [04-attachments-persistence-and-team-approval.md](./03-plan-system/04-attachments-persistence-and-team-approval.md)，把 attachment、还原与 team approval 合回同一条运行链。

## 与其它专题的边界

- teammate runtime、mailbox 与 approval backend 更广义的 team 机制，见 [01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md](./01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md)。
- TUI 里的 tasks、dialogs、plan detail 与 remote ultraplan 呈现，见 [07-tui-system.md](./07-tui-system.md)。
- attachment payload 的 materialize 细节，见 [../02-execution/05-attachments-and-context-modifiers.md](../02-execution/05-attachments-and-context-modifiers.md)。

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
