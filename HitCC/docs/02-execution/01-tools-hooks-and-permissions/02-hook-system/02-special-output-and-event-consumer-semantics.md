# `hookSpecificOutput` 与特殊事件消费语义

## 本页用途

- 单独承接 `hookSpecificOutput`、`SubagentStart` 和多类特殊事件的 JSON output 消费语义。
- 把“schema 支持什么”和“调用方真正消费什么”拆开写实。

## 相关文件

- [../02-hook-system.md](../02-hook-system.md)
- [../01-tool-execution-core.md](../01-tool-execution-core.md)
- [../03-permission-mode-and-classifier.md](../03-permission-mode-and-classifier.md)
- [../04-policy-sandbox-and-approval-backends.md](../04-policy-sandbox-and-approval-backends.md)
- [../../04-non-main-thread-prompt-paths/02-hook-and-compact-special-paths.md](../../04-non-main-thread-prompt-paths/02-hook-and-compact-special-paths.md)

### `hookSpecificOutput`：当前已能直接确认的分支

`Nv6()` 现在已经能直接确认这些 `hookSpecificOutput.hookEventName` 分支：

- `PreToolUse`
  - `permissionDecision`
  - `permissionDecisionReason`
  - `updatedInput`
  - `additionalContext`
- `UserPromptSubmit`
  - `additionalContext`
- `SessionStart`
  - `additionalContext`
  - `initialUserMessage`
  - `watchPaths`
- `Setup`
  - `additionalContext`
- `SubagentStart`
  - `additionalContext`
- `PostToolUse`
  - `additionalContext`
  - `updatedMCPToolOutput`
- `PostToolUseFailure`
  - `additionalContext`
- `Notification`
  - `additionalContext`
- `PermissionRequest`
  - `decision`
- `Elicitation`
  - `action`
  - `content`
- `ElicitationResult`
  - `action`
  - `content`
- `CwdChanged`
  - `watchPaths`
- `FileChanged`
  - `watchPaths`
- `WorktreeCreate`
  - `worktreePath`

### `SubagentStart`：本地活消费链与顺序

`SubagentStart` 现在不该只记成“支持 `additionalContext`”，而应写成一条更具体的链。

#### 输入形状

`Ri1(agentId, agentType, signal)` 传给 `FC(...)` 的 hook input 当前可直接写成：

```text
{
  ...hY(undefined),
  hook_event_name: "SubagentStart",
  agent_id,
  agent_type
}
```

也就是说，这一事件当前稳定带：

- `session_id`
- `transcript_path`
- `cwd`
- `hook_event_name: "SubagentStart"`
- `agent_id`
- `agent_type`

#### 匹配与执行顺序

`et1(...)` 的匹配阶段本身是有顺序的：

- 先收集 `V8z(...)` 返回的 matcher 列表
- 再按 hook 类型重排成：
  - `command`
  - `prompt`
  - `agent`
  - `http`
  - `callback`
  - `function`

但真正执行时，`FC(...)` 会：

- 把每个匹配 hook 变成一个 async generator
- 用 `MC8(...)` 并发跑
- 通过 `Promise.race(...)` 按**完成先后**吐回结果

所以需要明确区分两层：

- matcher/hook 列表顺序：稳定
- hook 返回结果顺序：**完成顺序，不保证等于 matcher 顺序**

#### 当前调用点只消费 `additionalContext`

`SubagentStart` 最大的实现边界在调用方，不在 schema。

`BN(...)` 对 `Ri1(...)` 的消费现在只有：

```text
for await (let r of Ri1(...)) {
  if (r.additionalContexts?.length > 0) z6.push(...r.additionalContexts)
}
if (z6.length > 0) {
  I.push(Nq({
    type: "hook_additional_context",
    content: z6,
    hookName: "SubagentStart",
    ...
  }))
}
```

因此当前本地活语义应写成：

- `additionalContext`
  - 会被收集
  - 会被打包成一个 `hook_additional_context` attachment
  - 会并入 subagent sidechain messages 尾部

而这些东西当前**没有看到活消费者**：

- `blockingError`
- `preventContinuation`
- `systemMessage`
- 通用 `hook_success` / `hook_non_blocking_error` message

换句话说：

- `FC(...)` 通用能力比 `SubagentStart` 当前调用点更强
- 但 `SubagentStart` 的本地活消费面只有 **附加上下文注入**

### Hook 事件全集：本地 bundle 内已基本闭环

运行时事件枚举 `Mp` 与内嵌帮助元数据合起来，当前本地 bundle 可见的事件至少包括：

- `PreToolUse`
- `PostToolUse`
- `PostToolUseFailure`
- `Notification`
- `UserPromptSubmit`
- `SessionStart`
- `SessionEnd`
- `Stop`
- `StopFailure`
- `SubagentStart`
- `SubagentStop`
- `PreCompact`
- `PostCompact`
- `PermissionRequest`
- `Setup`
- `TeammateIdle`
- `TaskCreated`
- `TaskCompleted`
- `Elicitation`
- `ElicitationResult`
- `ConfigChange`
- `WorktreeCreate`
- `WorktreeRemove`
- `InstructionsLoaded`
- `CwdChanged`
- `FileChanged`

### `Mi6("compact")`：本地活入口已经收紧

这一点现在可以写得更硬：

- `WQ9()` 只在 `sj()` 主扫描末尾消费一次当前 `sC1`
- 消费后立刻把 `sC1` 重置回 `session_start`
- 当前本地 bundle 内，唯一直接命中的 setter 仍是 `Cn(querySource) -> Mi6("compact")`
- `Cn(querySource)` 又只在：
  - `querySource === undefined`
  - `querySource.startsWith("repl_main_thread")`
  - `querySource === "sdk"`
  时真的调用 `Mi6("compact")`

因此就当前本地 bundle 可见代码而言，`Mi6("compact")` 的活语义已经可以收紧为：

- compact 成功后，为下一次主线程 / SDK 的 `sj()` 主扫描预置 `load_reason = "compact"`

目前没再看到第二个本地活入口。  
剩余不确定性只在 bundle 外或服务端黑箱。

### `Stop / SessionEnd / SubagentStop / TaskCreated / TaskCompleted / WorktreeRemove` 的 JSON output 消费语义

这 6 个点现在可以分两组写死。

#### 第一组：走 `FC(...)`，但没有 event-specific `hookSpecificOutput`

- `Stop`
- `SubagentStop`
- `TaskCreated`
- `TaskCompleted`

这些事件都会进 `FC(...)`，但它们最终走的 JSON 解析器 `tt1(...)` 没有对应的 event-specific case。  
`tt1(...)` 只显式消费：

- `PreToolUse`
- `UserPromptSubmit`
- `SessionStart`
- `Setup`
- `SubagentStart`
- `PostToolUse`
- `PostToolUseFailure`
- `PermissionRequest`
- `Elicitation`
- `ElicitationResult`

所以对这 4 个事件来说，JSON output 里真正有运行时效果的，只剩顶层通用字段：

- `continue: false`
  - 变成 `preventContinuation: true`
  - 若带 `stopReason`，会一并上浮
- `decision: "block"`
  - 变成 `blockingError`
- `systemMessage`
  - 变成 `hook_system_message` attachment
- `suppressOutput`
  - 只影响成功时是否额外生成通用 `"hook completed"` 提示，不改变控制流

更具体地说：

- `Stop`
  - 调用方会消费 `blockingError` 与 `preventContinuation`
  - `blockingError` 会被包装成 stop feedback 再塞回模型继续一轮
  - `preventContinuation` 会产出 `hook_stopped_continuation`，并让当前 stop 路径停止继续
- `SubagentStop`
  - 与 `Stop` 同骨架，只是作用对象换成 subagent
- `TaskCreated`
  - 当前调用方只检查 `blockingError`
  - 因此真正能阻止创建任务的，是 blocking path
  - `preventContinuation` / `systemMessage` 对 task create 本身没有控制效果
- `TaskCompleted`
  - 显式 task status 变更路径里，当前调用方只检查 `blockingError`
  - turn 结束时的 teammate/task 收尾路径里，会同时消费 `blockingError` 与 `preventContinuation`

因此这组事件可以直接下结论：

- 没有 event-specific JSON branch
- 真正有控制效果的是通用 `blocking / preventContinuation`
- 其余最多只影响 transcript/UI 提示

#### 第二组：走 `UC(...)`，JSON 基本不进入业务语义

- `SessionEnd`
- `WorktreeRemove`

这两个事件都不走 `FC(...)`，而是直接走 `UC(...)`。

`UC(...)` 对 command/http hook 的 JSON 只会做两类底层处理：

- `decision === "block"` -> `blocked: true`
- 把 `systemMessage / watchPaths` 之类挂回结果对象

但它们的调用方当前都没有继续消费这些字段：

- `SessionEnd`
  - 只对失败 hook 的 stderr 做 `process.stderr.write(...)`
  - 然后清理 session hook 状态
  - 成功 hook 的 JSON 不会改 session end 流程
- `WorktreeRemove`
  - 只看“是否配置了 hook、是否有结果、哪些 hook 失败”
  - 失败会打日志
  - 不消费 `blocked / systemMessage / watchPaths`

所以这组事件现在可以直接写成：

- JSON 可以被底层解析
- 但当前业务调用方不消费其语义字段
- 实际效果基本只剩“命令是否成功/失败”与 stderr 日志

### `WorktreeCreate` 要单独记

`WorktreeCreate` 是 worktree 族里唯一必须保留特例的事件：

- command hook：当前仍以 stdout 路径为准
- callback/http hook：可通过 `hookSpecificOutput.worktreePath` 产出 worktree 路径

这也是为什么 `WorktreeCreate` 在 schema 与 `UC(...)` 中有专门分支，而 `WorktreeRemove` 没有。

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
