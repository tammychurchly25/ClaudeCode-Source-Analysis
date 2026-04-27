# 主循环与压缩路径

## 本页用途

- 这页不再承载全部细节，而改成 `01-runtime` 下 agent loop / compact 主题的总览与导航。
- 原先混在一页里的内容，已经拆成主循环状态与 `yield` 面、压缩链、无工具分支、工具轮与终止返回四个专题页。

## 相关文件

- [04-agent-loop-and-compaction/01-main-loop-state-caches-and-yield-surface.md](./04-agent-loop-and-compaction/01-main-loop-state-caches-and-yield-surface.md)
- [04-agent-loop-and-compaction/02-compaction-pipeline-and-auto-compact-tracking.md](./04-agent-loop-and-compaction/02-compaction-pipeline-and-auto-compact-tracking.md)
- [04-agent-loop-and-compaction/03-no-tool-branch-recovery-stop-and-reactive-compact.md](./04-agent-loop-and-compaction/03-no-tool-branch-recovery-stop-and-reactive-compact.md)
- [04-agent-loop-and-compaction/04-tool-round-next-turn-and-terminal-reasons.md](./04-agent-loop-and-compaction/04-tool-round-next-turn-and-terminal-reasons.md)
- [03-input-compilation.md](./03-input-compilation.md)
- [05-model-adapter-provider-and-auth.md](./05-model-adapter-provider-and-auth.md)
- [06-stream-processing-and-remote-transport.md](./06-stream-processing-and-remote-transport.md)
- [../02-execution/01-tools-hooks-and-permissions.md](../02-execution/01-tools-hooks-and-permissions.md)

## 拆分后的主题边界

### 主循环状态 / `J` / turn 内缓存 / `yield` 面

见：

- [04-agent-loop-and-compaction/01-main-loop-state-caches-and-yield-surface.md](./04-agent-loop-and-compaction/01-main-loop-state-caches-and-yield-surface.md)

这一页集中放：

- `CC / po_` 的最小状态机骨架
- 长生命周期状态 `J`
- turn 内局部缓存与跨 turn 槽位
- `stream_request_start`、`assistant`、`attachment`、`tombstone` 等 `yield` 面

### 压缩链 / compact producer / `autoCompactTracking`

见：

- [04-agent-loop-and-compaction/02-compaction-pipeline-and-auto-compact-tracking.md](./04-agent-loop-and-compaction/02-compaction-pipeline-and-auto-compact-tracking.md)

这一页集中放：

- `microcompact` 与 `DEq(...)`
- `compactionResult -> hn(...) -> transcript rebuild` 合同
- `compact_boundary`、`preservedSegment`
- full / partial / session-memory compact 差异
- `autoCompactTracking` 的熔断与 telemetry 语义

### 无工具分支：reactive compact / `max_output_tokens` / stop hook

见：

- [04-agent-loop-and-compaction/03-no-tool-branch-recovery-stop-and-reactive-compact.md](./04-agent-loop-and-compaction/03-no-tool-branch-recovery-stop-and-reactive-compact.md)

这一页集中放：

- `z6 === false` 时的 turn 尾部分支
- reactive compact 的接口合同与未接线结论
- `max_output_tokens` 续写还原
- `Rj4(...)` stop hook 子状态机与 `stopHookActive`

### 有工具分支 / 下一 turn / 中断与错误返回

见：

- [04-agent-loop-and-compaction/04-tool-round-next-turn-and-terminal-reasons.md](./04-agent-loop-and-compaction/04-tool-round-next-turn-and-terminal-reasons.md)

这一页集中放：

- `Re6 / Zx8` 工具轮
- `hook_stopped_continuation`
- 延后一轮消费的 tool summary
- `aborted_streaming`、`aborted_tools`、`model_error` 等终止返回

## 建议阅读顺序

1. 先看 [01-main-loop-state-caches-and-yield-surface.md](./04-agent-loop-and-compaction/01-main-loop-state-caches-and-yield-surface.md)，建立 `po_` 的状态机骨架和对外事件面。
2. 再看 [02-compaction-pipeline-and-auto-compact-tracking.md](./04-agent-loop-and-compaction/02-compaction-pipeline-and-auto-compact-tracking.md)，理解 compact producer、重建合同和 tracking。
3. 然后看 [03-no-tool-branch-recovery-stop-and-reactive-compact.md](./04-agent-loop-and-compaction/03-no-tool-branch-recovery-stop-and-reactive-compact.md)，补齐无工具 turn 的还原与停止逻辑。
4. 最后看 [04-tool-round-next-turn-and-terminal-reasons.md](./04-agent-loop-and-compaction/04-tool-round-next-turn-and-terminal-reasons.md)，把工具阶段、下一轮续转和各种退出原因补齐。

## 与其它专题的边界

- 输入如何被编译成主循环入口前的消息与上下文，见 [03-input-compilation.md](./03-input-compilation.md)。
- `callModel`、provider 选择、鉴权与模型门面，见 [05-model-adapter-provider-and-auth.md](./05-model-adapter-provider-and-auth.md)。
- 流事件如何被适配成 UI / SDK 可消费输出，以及 fallback transport，见 [06-stream-processing-and-remote-transport.md](./06-stream-processing-and-remote-transport.md)。
- 工具执行器、hook 事件与权限系统细节，见 [../02-execution/01-tools-hooks-and-permissions.md](../02-execution/01-tools-hooks-and-permissions.md)。

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
