# Hook Schema、`InstructionsLoaded` 与事件输入

## 本页用途

- 单独承接 hook 配置层 schema、`InstructionsLoaded` 触发链和 HookEvent 输入字段。
- 把“有哪些 hook”“输入长什么样”“哪些 attachment 会进 transcript”固定到同一页。

## 相关文件

- [../02-hook-system.md](../02-hook-system.md)
- [../../02-instruction-discovery-and-rules.md](../../02-instruction-discovery-and-rules.md)
- [../../06-context-runtime-and-tool-use-context.md](../../06-context-runtime-and-tool-use-context.md)
- [../../../05-appendix/02-evidence-map.md](../../../05-appendix/02-evidence-map.md)

## Hook 系统

### Hook 配置层 schema

可确认的 hook event：

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

可确认的 hook def 类型：

- `command`
- `prompt`
- `http`
- `agent`

并支持：

- `timeout`
- `statusMessage`
- `once`
- `async`
- `asyncRewake`
- `allowedEnvVars`
- `model`
- `shell`

### `PreToolUse` 运行时返回

已确认可能产生：

- `message`
- `hookPermissionResult`
- `preventContinuation`
- `stopReason`
- `updatedInput`
- `additionalContexts`
- `stop`

### Hook 结果的 transcript 显示

attachment/type 至少包含：

- `async_hook_response`
- `hook_blocking_error`
- `hook_non_blocking_error`
- `hook_error_during_execution`
- `hook_success`
- `hook_stopped_continuation`
- `hook_system_message`
- `hook_permission_decision`

### 结论

Hook 不是后台机制，而是会进入 transcript 和 UI 呈现。

### `InstructionsLoaded` payload 与触发点

这条现在已经不再是“只有名字，没有形状”。

### 已确认 payload

`InstructionsLoaded` 的 schema 至少包含：

- `hook_event_name: "InstructionsLoaded"`
- `file_path`
- `memory_type: "User" | "Project" | "Local" | "Managed"`
- `load_reason: "session_start" | "nested_traversal" | "path_glob_match" | "include" | "compact"`
- `globs?`
- `trigger_file_path?`
- `parent_file_path?`

### 已确认运行时入口

- `Xi6()`：检查是否存在 `InstructionsLoaded` hook
- `Di6(...)`：执行 `InstructionsLoaded` hooks
- `WQ9()`：一次性读取当前 load reason，并在消费后重置为 `session_start`
- `Mi6(reason)`：设置下一次主扫描应使用的 load reason，并清空 `sj()` cache

### 已确认触发面

1. `sj()` 主扫描结束后
2. nested memory / opened-file 相关追加装载时

更具体地说：

- 主扫描路径里，会在收集完成后对每个 `User / Project / Local / Managed` memory file 调 `Di6(...)`
- 若该文件是通过 `@include` 引入，则 `load_reason = "include"`
- 否则主扫描路径使用 `WQ9()` 的结果；这一点现在已经可以更具体地确认：
  - `WQ9()` 读取当前 `sC1`
  - 然后把 `sC1` 重置回 `session_start`
  - 这个消费动作发生在 `Xi6()` 判断之前，因此即使当前没有 `InstructionsLoaded` hook，reason 槽位也会被读走
  - 本地活代码里只直接看到 `Mi6("compact")`
  - 调用点是 `Cn(querySource)`，且只在 `querySource === undefined`、`querySource.startsWith("repl_main_thread")` 或 `querySource === "sdk"` 时触发
  - 也就是说它更像主线程 / SDK query 在 compact 后，为下一次 `sj()` 主扫描预置 `load_reason = "compact"`
  - 它不是 compact 完成时立刻发 hook，而是下一次 `sj()` 扫描结束后才真正 `Di6(...)`
- nested memory 路径里，`load_reason` 会落在：
  - `nested_traversal`
  - `path_glob_match`
  - `include`

还能继续收紧 nested 路径的时序：

- 追加装载逻辑会先判断 `readFileState` 里是否还没有该 memory 文件
- 然后先把它写进 `nested_memory` attachment 与 `readFileState`
- 最后才在 `Xi6()` 命中且类型允许时调用：

```text
let reason =
  file.globs ? "path_glob_match"
  : file.parent ? "include"
  : "nested_traversal"

Di6(file.path, file.type, reason, {
  globs: file.globs,
  triggerFilePath,
  parentFilePath: file.parent
})
```

因此这里的三个 reason 不是主扫描 `WQ9()` 分出来的，而是 nested loader 当场按文件来源判定的。

### HookEvent 输入 schema：当前已可直接写实

除了 `PreToolUse` 与 `InstructionsLoaded`，下面这些事件的输入字段也已能从运行时代码直接读出：

- `PostToolUseFailure`
  - `session_id`
  - `transcript_path`
  - `cwd`
  - `permission_mode`
  - `agent_id?`
  - `agent_type`
  - `hook_event_name: "PostToolUseFailure"`
  - `tool_name`
  - `tool_input`
  - `tool_use_id`
  - `error`
  - `is_interrupt`
- `Notification`
  - `session_id`
  - `transcript_path`
  - `cwd`
  - `hook_event_name: "Notification"`
  - `message`
  - `title`
  - `notification_type`
- `SessionStart`
  - `session_id`
  - `transcript_path`
  - `cwd`
  - `hook_event_name: "SessionStart"`
  - `source`
  - `agent_type`
  - `model`
- `StopFailure`
  - `session_id`
  - `transcript_path`
  - `cwd`
  - `PERMISSION_MODE?`
  - `agent_id?`
  - `agent_type`
  - `hook_event_name: "StopFailure"`
  - `error`
  - `error_details`
  - `last_assistant_message?`
- `ConfigChange`
  - `session_id`
  - `transcript_path`
  - `cwd`
  - `hook_event_name: "ConfigChange"`
  - `source`
  - `file_path`
- `FileChanged`
  - `session_id`
  - `transcript_path`
  - `cwd`
  - `hook_event_name: "FileChanged"`
  - `file_path`
  - `event`

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
