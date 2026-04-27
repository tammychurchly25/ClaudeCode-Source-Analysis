# 全局状态、Session 与 Transcript 持久化

## 本页用途

- 用来理解运行时状态放在哪里，以及 Session/Transcript 为什么本质上是“工作现场”而不是聊天记录。
- 用来支撑后续重写中的持久化层、resume 逻辑和 file-history 设计。

## 相关文件

- [01-product-cli-and-modes.md](./01-product-cli-and-modes.md)
- [04-agent-loop-and-compaction.md](./04-agent-loop-and-compaction.md)
- [../03-ecosystem/01-resume-fork-sidechain-and-subagents.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents.md)
- [../03-ecosystem/02-remote-persistence-and-bridge.md](../03-ecosystem/02-remote-persistence-and-bridge.md)
- [../05-appendix/02-evidence-map.md](../05-appendix/02-evidence-map.md)

## 全局应用状态与 Session State Store

### 全局状态模型

已确认存在一个中心化状态单例，至少维护：

- `originalCwd`
- `cwd`
- `projectRoot`
- `sessionId`
- `parentSessionId`
- `displayName`
- `modelUsage`
- `totalCostUSD`
- `totalAPIDuration`
- `totalToolDuration`
- `totalTurnHookDuration`
- `lastInteractionTime`
- `inlinePlugins`
- `sessionCronTasks`
- `invokedSkills`
- `systemPromptSectionCache`
- `planSlugCache`
- `cachedClaudeMdContent`
- `additionalDirectoriesForClaudeMd`
- `sessionPersistenceDisabled`
- `sessionTrustAccepted`

### 设计判断

这是一个**全局 app state + per-session state 混合 Store**。

优点：

- 任意模块都能拿到当前会话、模型、工具、缓存、设置
- TUI 和 Headless 可以共享状态逻辑

缺点：

- 强全局可变状态
- 并发与测试复杂度高
- 模块间耦合明显

### 重写建议

重写时建议保留“单例状态中心”概念，但用更清晰的分层：

```ts
AppState
  - ui
  - settings
  - mcp
  - plugins
  - permission
  - sessions

SessionState
  - transcript
  - tool/usage stats
  - plan
  - file history
  - invoked skills
```

---

## Session 与 Transcript 持久化体系

### 根目录 `U1`

已确认规则：

- 优先 `process.env.CLAUDE_CONFIG_DIR`
- 否则 `~/.claude`
- 路径会做 Unicode NFC 归一化

因此根目录可以直接还原为：

```ts
const appRoot = process.env.CLAUDE_CONFIG_DIR ?? path.join(os.homedir(), '.claude')
```

### 路径结构

已确认或高可信：

```text
<appRoot>/
  projects/
    <normalized-project-id>/
      <session-id>.jsonl
      <current-session-id>/
        subagents/
          agent-<agentId>.jsonl
```

相关函数语义：

- `Pm()` -> `join(U1(), "projects")`
- `O2(projectPath)` -> `join(Pm(), normalizeProjectId(projectPath))`
- `PG(sessionId)` -> `join(O2(currentProject), `${sessionId}.jsonl`)`
- `$0(agentId)` -> 当前 session 目录下 `subagents/agent-<id>.jsonl`

### Writer 内核：`class iC4`

已确认字段：

- `sessionFile`
- `pendingEntries`
- `writeQueues: Map`
- `flushTimer`
- `activeDrain`
- `pendingWriteCount`
- `FLUSH_INTERVAL_MS = 100`
- `MAX_CHUNK_BYTES = 104857600`

### 行为

- `appendEntry()` 不直接写盘，而是 `enqueueWrite()`
- 每 100ms 批量 drain
- 按目标文件聚合
- 拼成 JSONL 块批量写入
- 超过 100MB 再切块
- `flush()` 等待 active drain + pending writes 清零

### 设计意义

这是一个小型的 **批量 JSONL 写入调度器**，不是简单的 `appendFile`。

### 惰性 materialize

主 session transcript 文件不会在 session 创建时立刻落盘。只有出现需要写入主链的消息/entry 时才 materialize。

在此之前产生的 metadata/queue op 会先放在 `pendingEntries`，待 sessionFile 确定后回灌。

### 这样设计的原因

- 避免空会话文件
- 避免 sidechain/subagent 抢先建主 session 文件
- 避免 metadata 落入错误 session

### Transcript schema：不是只有消息

从还原解析器、writer 分流与 attachment/UI 渲染可确认，jsonl 里至少包括：

- user/assistant/progress/system/attachment 等消息实体
- `summary`
- `custom-title`
- `ai-title`
- `last-prompt`
- `task-summary`
- `tag`
- `agent-name`
- `agent-color`
- `agent-setting`
- `pr-link`
- `file-history-snapshot`
- `attribution-snapshot`
- `content-replacement`
- `context-collapse-commit`
- `context-collapse-snapshot`
- `mode`
- `worktree-state`
- `queue-operation`

### 结论

Transcript 实际上是一个“事件日志”，而不是单纯的聊天记录。

### `recordTranscript` / `jV`

语义已确认：

- 会对消息做 UUID 去重
- 只写新消息
- 维护 `parentUuid`
- progress 类实体不一定作为主链 parent 候选

这对 resume/replay/remote 同步非常关键。

### Resume 不只是读 transcript

`I76(...)` 统一“读会话”入口，支持：

- 最近会话
- sessionId
- jsonl 路径
- 已加载 session 对象

还原时还会做：

- 还原 plan 文件
- 还原/复制 file-history 备份
- 清理 interrupted turn
- 注入 resume hook 结果

因此 Resume 还原的是“工作现场”，不是普通聊天历史。

---

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
