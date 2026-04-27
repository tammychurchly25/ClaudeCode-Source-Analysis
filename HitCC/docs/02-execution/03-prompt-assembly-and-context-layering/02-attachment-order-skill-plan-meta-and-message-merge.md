# Attachment 顺序、skill/plan 元信息与消息合并

## 本页用途

- 单独承接 attachment 在主线程 prompt 里的顺序问题，以及 `skill_listing / plan_mode / invoked_skills` 这组元信息的落点。
- 把普通当前轮、compact/resume 保留附件、`ClaudeMd` 前缀链与 compat 当前边界收敛到同一页。

## 相关文件

- [../03-prompt-assembly-and-context-layering.md](../03-prompt-assembly-and-context-layering.md)
- [../05-attachments-and-context-modifiers.md](../05-attachments-and-context-modifiers.md)
- [../06-context-runtime-and-tool-use-context.md](../06-context-runtime-and-tool-use-context.md)
- [../../03-ecosystem/03-plan-system.md](../../03-ecosystem/03-plan-system.md)
- [../../03-ecosystem/05-skill-system.md](../../03-ecosystem/05-skill-system.md)

## Attachment 顺序与消息合并

### `invoked_skills`：这里只保留顺序结论

关于 `invoked_skills` 的职责、生成与还原链路，详见 [../../03-ecosystem/05-skill-system.md](../../03-ecosystem/05-skill-system.md)。  
本页只记录它在 compact / resume 里的顺序位置。

#### full / partial compact

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

两者的区别只在外层返回骨架：

- full compact：`boundaryMarker -> summaryMessages -> attachments -> hookResults`
- partial compact：`boundaryMarker -> summaryMessages -> messagesToKeep -> attachments -> hookResults`

因此更稳的顺序结论可以直接写成：

- `invoked_skills` 明确位于 `plan_file_reference / plan_mode` 之后
- 位于 `deferred_tools_delta / agent_listing_delta / mcp_instructions_delta` 之前
- 位于 `hookResults` 之前

#### resume

- resume 不会额外制造新的 `invoked_skills` attachment
- 因此这里不存在“resume 二次注入后的重新排序问题”

### 普通当前轮里已能钉死的内容顺序

如果只看主线程普通当前轮，且聚焦：

- `Lx8(userContext)`
- `skill_listing`
- `plan_mode`
- `critical_system_reminder`
- 当前轮 user 输入

那么当前更稳的请求级线性顺序应写成：

```text
Lx8(userContext)
-> 历史 transcript
-> 上一个 assistant / tool_result
-> skill_listing
-> plan_mode_reentry (optional)
-> plan_mode
-> critical_system_reminder
-> 当前轮 user 输入
```

这里有两个必须一起记住的实现细节：

1. `BU4(...)` 会先把“当前轮 user 文本 + pasted image blocks”包成**一个 user message**，再把 `P6z(...)` 产出的 attachment transcript entries 接在后面。  
2. 随后 `u0z(...)` 会把这些尾随 attachment 向左移动到更早的 assistant / `tool_result` 边界；再经过 `_X(...) / Mg8(...)` 后，它们往往不再保留成独立 message，而是合并进相邻 user message。  

### `P6z(...)` 的 attachment 生成顺序现在已能直接写实

这一层此前常被写成“若干 attachment”，但实际代码里是**固定数组顺序**，不是按异步完成时间乱序插入。  
原因是 `P6z(...)` 虽然内部大量用 `Promise.all(...)`，但最终会按声明顺序做 `[..., ...J.flat(), ...X.flat(), ...D.flat()]`。

如果只看普通当前轮，attachment 生成顺序当前可写成：

```text
at_mentioned_files
-> mcp_resources
-> agent_mentions
-> date_change
-> ultrathink_effort
-> deferred_tools_delta
-> agent_listing_delta
-> mcp_instructions_delta
-> changed_files
-> nested_memory
-> dynamic_skill
-> skill_listing
-> plan_mode_reentry (optional, inside plan_mode producer)
-> plan_mode
-> plan_mode_exit
-> auto_mode
-> auto_mode_exit
-> todo_reminder / task_reminder
-> teammate_mailbox (non-session_memory only)
-> team_context
-> agent_pending_messages
-> critical_system_reminder
-> ide_selection
-> ide_opened_file
-> output_style
-> diagnostics
-> lsp_diagnostics
-> unified_tasks
-> async_hook_responses
-> token_usage
-> budget_usd
-> output_token_usage
-> verify_plan_reminder
-> queued_commands
```

因此如果只讨论这一页最关心的四类 attachment，则当前本地 bundle 里的生成顺序可以进一步收紧为：

- `skill_listing`
- `plan_mode_reentry`（若存在）
- `plan_mode`
- `critical_system_reminder`

### `invoked_skills` 的位置要单独看

`invoked_skills` 不在普通当前轮的 `P6z(...)` 生成链里。  
当前直接能确认的来源是：

- compact 路径里由 `vVq(...)` 生成保留 attachment
- resume 路径里由 `Su_(...)` 还原回运行态

因此：

- 普通当前轮里，讨论 `skill_listing / plan_mode / critical_system_reminder / 当前轮 user` 的顺序时，不应把 `invoked_skills` 混在一起
- `invoked_skills` 更应归入 **compact/resume 保留附件** 的顺序问题

skills 三类 attachment 的职责差异、`discoveredSkillNames` 的状态、以及 skills 的三层结构，已迁移到 [../../03-ecosystem/05-skill-system.md](../../03-ecosystem/05-skill-system.md)。本页不再重复。

### 当前最可靠的合并近似

现在更稳妥的写法应拆成两层：

```text
system chain
  lX8
  -> cX8
  -> bC(
       overrideSystemPrompt
       > active agent prompt
       > customSystemPrompt
       > $X(...) default system sections
       > appendSystemPrompt
     )
  -> dj4(..., systemContext)

message-prefix chain
  userContext.ClaudeMd
    = eC1([
        managed CLAUDE.md
        -> managed .claude/rules/*
        -> user CLAUDE.md
        -> user .claude/rules/*
        -> ancestor directories from root -> cwd:
             project CLAUDE.md
             -> project .claude/CLAUDE.md
             -> project .claude/rules/*
             -> local claude.local.md
        -> additionalDirectoriesForClaudeMd:
             CLAUDE.md
             -> .claude/CLAUDE.md
             -> .claude/rules/*
        -> AutoMem
        -> TeamMem
      ])
  -> currentDate
  -> Lx8(...): prepend as meta user message
```

这里要特别修正两件事：

1. `dynamic_skill` 更像 **attachment/meta message 阶段**，不是 `sj()` 的普通 memory file。  
2. `sj()` 的扫描顺序已较稳，但它对应的是 **userContext.ClaudeMd 的内部顺序**，不应再直接表述成 request-level `system` 顺序。

还要再补一个关键结论：

3. compat 文件当前更像 **`/init` 的迁移输入**，不是 `sj()` 或 `$X(...)` 的并行 runtime source。  
4. `subagent / compact / hook agent / bridge` 本地路径当前也没有显示出另一套 compat 注入逻辑。  
   更准确地说，compat 进入 non-main-thread 只有三类载体：`BN(...)` fresh build、`lZ(...)` snapshot reuse、以及显式 override / 父 prompt 复用；专用 `hook_agent` 与 dedicated compact summarize 路径当前看不到独立 compat 注入面。

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
