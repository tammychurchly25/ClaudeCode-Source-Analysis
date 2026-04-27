# Plan 运行时对象与文件生命周期

## 本页用途

- 单独承接 plan 的 runtime object 模型和 plan file 生命周期。
- 把“plan 到底是什么对象”“文件怎么命名、还原、复制”固定到一页。

## 相关文件

- [../03-plan-system.md](../03-plan-system.md)
- [../01-resume-fork-sidechain-and-subagents.md](../01-resume-fork-sidechain-and-subagents.md)
- [../../01-runtime/02-session-and-persistence.md](../../01-runtime/02-session-and-persistence.md)
- [../../02-execution/05-attachments-and-context-modifiers.md](../../02-execution/05-attachments-and-context-modifiers.md)

## 一句话结论

Plan 不是“模型先想想再写代码”的软约束，而是一套正式的运行时子系统：

```text
permission mode(plan)
-> per-session/per-agent plan file
-> /plan viewer + external-editor bridge
-> ExitPlanMode approval UI
-> plan attachments(reentry / reminder / exit / file reference)
-> compact / resume recovery
-> teammate -> leader mailbox approval
```

其中真正的主对象不是对话里的某段文本，而是 **plan file + mode state + attachment state**。

## 1. 运行时对象模型

当前 bundle 已能把 Plan 相关 runtime object 收紧到下面这些核心对象：

- `jX(agentId)`
  - 返回当前 session 或 subagent 对应的 plan file path
- `AP(agentId)`
  - 直接从磁盘读取 plan file 内容
- `uF(sessionId?)`
  - 维护当前 session 对应的 plan slug
- `toolPermissionContext.mode === "plan"`
  - 表示当前处于 plan mode
- `toolPermissionContext.prePlanMode`
  - 记录进入 plan 前的 permission mode，用于退出时还原
- `V8.hasExitedPlanMode`
  - 由 `yS6()/wE(...)` 维护，驱动后续 `plan_mode_reentry`
- `V8.needsPlanModeExitAttachment`
  - 由 `fc8()/Nb(...)` 维护，驱动后续 `plan_mode_exit`

因此更稳的理解不是：

```text
plan = 一条消息
```

而是：

```text
plan = file on disk
     + session mode state
     + transcript-visible attachments
     + exit/reentry bookkeeping bits
```

## 2. Plan File 生命周期

### 2.1 路径与命名

plan file 默认不放在 transcript 目录里，而是走独立的 plans 目录：

- `Aw()`
  - 优先读 settings 里的 `plansDirectory`
  - 若配置越出 project root，会告警并回退
  - 默认目录为 `<stateRoot>/plans`
- `uF()`
  - 给当前 session 生成并缓存一个 slug
  - slug 形状是随机短名，不是 session id 本身
- `jX(agentId)`
  - 主线程：`<plansDir>/<slug>.md`
  - subagent：`<plansDir>/<slug>-agent-<agentId>.md`

这说明每个 agent 都可以拥有自己的 plan file，而且 filename 不是稳定的人类命名，而是 session-scoped slug。

### 2.2 读写入口

当前直接可见的 plan file 入口很少，但职责很硬：

- `AP(agentId)`
  - `readFileSync(..., "utf-8")`
  - 文件不存在时返回 `null`
- `ExitPlanMode.call(...)`
  - 若 input 里带 `plan`，先写回磁盘，再做后续退出逻辑
- `/plan open`
  - 直接把 `jX()` 返回的真实 plan path 打开到外部编辑器

### 2.3 还原与补救

Plan 不是“文件丢了就算了”。本地 bundle 有明确的还原链：

- `mN8(transcript, sessionId?)`
  - 根据 transcript 中的 slug 还原 plan file
  - 若文件丢失，会按优先级尝试补回：
  - `file_snapshot(key="plan")`
  - `oq_(messages)` 提取出的历史 plan 内容
- `oq_(messages)` 当前可从三类来源反推 plan：
  - assistant 的 `ExitPlanMode` tool input 里的 `plan`
  - user message 上的 `planContent`
  - `plan_file_reference.planContent`
- `BN8()`
  - 把当前 plan file 记成 `system/file_snapshot`
  - snapshot key 固定为 `plan`

因此 plan file 的可靠语义更接近：

```text
disk file
<-> file snapshot
<-> transcript fallback extraction
```

而不是普通临时文件。

### 2.4 迁移/复制辅助函数

还能看到一个独立 helper：

- `AVq(transcript, newSessionId)`
  - 从旧 slug 对应文件复制到新 slug 对应文件

这条调用链现在已经可以收紧：

- session 还原主链 `lL(...)` 里：
  - `w1 === "fork"` 时调用 `AVq(d8, FM(Y8))`
  - 非 fork 时调用 `mN8(d8, FM(Y8))`

因此 `AVq(...)` 不是泛化的“以后也许会用到的复制 helper”，而是当前本地 bundle 里 **fork 新 session 时复制旧 plan file 到新 session slug** 的正式路径。

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
