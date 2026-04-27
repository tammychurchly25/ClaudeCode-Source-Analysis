# `EnterPlanMode`、`ExitPlanMode` 与 `/plan`

## 本页用途

- 单独承接 plan mode 的正式状态迁移和 `/plan` 入口。
- 把工具层状态机与本地命令入口拆开写实。

## 相关文件

- [../03-plan-system.md](../03-plan-system.md)
- [01-runtime-objects-and-plan-file-lifecycle.md](./01-runtime-objects-and-plan-file-lifecycle.md)
- [../07-tui-system.md](../07-tui-system.md)

## 3. Enter / Exit 是正式状态迁移，不是提示词

## 3.1 `EnterPlanMode`

目前已能直接确认：

- 是正式工具，不是 slash command
- 主线程可用；agent context 会被限制
- 进入前会先把当前 `toolPermissionContext` 经过 `Hy6(...)`
- 然后通过 session 级 mode 切到 `plan`
- `checkPermissions()` 会走专用审批框，标题固定为 `Enter plan mode?`

因此：

- plan mode 是本地 permission state change
- 不是“模型自己尽量先规划一下”

## 3.2 `ExitPlanMode`

`ExitPlanMode` 的工具层逻辑现在已经能写成更细的状态机：

```text
validate: 必须当前 mode === plan
-> checkPermissions: ask("Exit plan mode?")
-> 读取 input.plan 或 AP(agentId)
-> 若 teammate: 发 plan_approval_request 给 team-lead
-> 若 main thread: 还原 prePlanMode/default
-> 设置 hasExitedPlanMode / needsPlanModeExitAttachment
-> 返回 tool result
```

工具输出 schema 当前可直接钉死为：

```ts
{
  plan: string | null
  isAgent: boolean
  filePath?: string
  hasTaskTool?: boolean
  planWasEdited?: boolean
  awaitingLeaderApproval?: boolean
  requestId?: string
}
```

补充边界：

- `allowedPrompts[]` 当前只公开 `Bash + semantic prompt`
- 但当前本地 bundle 里，这条能力默认未真正接通
- `prePlanMode === "auto"` 且 auto gate 已失效时，会降级回 `default`
- `hasTaskTool` 代表当前上下文是否可用 Agent/Task 类工具，不是 plan file 自身属性

## 4. `/plan` 不是另一套系统，而是 plan file 的本地入口

本地 `local-jsx` 命令 `plan` 现在可以直接写实：

- 当前不在 `plan` mode
  - `/plan`
  - 直接切 mode 到 `plan`
  - 如果带额外参数且不是 `open`，会让输入继续作为 query
- 当前已在 `plan` mode 且无 plan file
  - 显示 `Already in plan mode. No plan written yet.`
- 当前已在 `plan` mode 且已有 plan
  - `/plan`
  - 打开本地 JSX 预览器展示当前 plan
- `/plan open`
  - 调 `GL(planPath)` 打开真实 plan file 到外部编辑器

因此 `/plan` 的更稳定义是：

```text
mode entry + viewer + external-editor launcher
```

不是另一种“计划协议”。

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
