# 非主线程 Prompt 路径

## 本页用途

- 这页不再承载全部细节，而改成 `02-execution` 下非主线程 prompt 主题的总览与导航。
- 原先混在一页里的内容，已经拆成共享 merge 骨架、hook/compact 专用路径、fork-family snapshot 复用、compat 与 agent definitions 四个专题页。

## 相关文件

- [04-non-main-thread-prompt-paths/01-shared-merge-skeleton-and-overrides.md](./04-non-main-thread-prompt-paths/01-shared-merge-skeleton-and-overrides.md)
- [04-non-main-thread-prompt-paths/02-hook-and-compact-special-paths.md](./04-non-main-thread-prompt-paths/02-hook-and-compact-special-paths.md)
- [04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md](./04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md)
- [04-non-main-thread-prompt-paths/04-compat-agent-definitions-and-instruction-entry.md](./04-non-main-thread-prompt-paths/04-compat-agent-definitions-and-instruction-entry.md)
- [02-instruction-discovery-and-rules.md](./02-instruction-discovery-and-rules.md)
- [03-prompt-assembly-and-context-layering.md](./03-prompt-assembly-and-context-layering.md)
- [06-context-runtime-and-tool-use-context.md](./06-context-runtime-and-tool-use-context.md)
- [../01-runtime/04-agent-loop-and-compaction.md](../01-runtime/04-agent-loop-and-compaction.md)
- [../03-ecosystem/01-resume-fork-sidechain-and-subagents.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents.md)

## 拆分后的主题边界

### 1. 共享 merge 骨架与前置裁剪

- [04-non-main-thread-prompt-paths/01-shared-merge-skeleton-and-overrides.md](./04-non-main-thread-prompt-paths/01-shared-merge-skeleton-and-overrides.md)：覆盖非主线程三分类里的前两类，重点整理 `BN(...)`、`CC(...)`、`omitClaudeMd`、`Explore/Plan` 裁剪，以及 `SubagentStart` hook 注入位置。

### 2. hook / compact 的专用 prompt 路径

- [04-non-main-thread-prompt-paths/02-hook-and-compact-special-paths.md](./04-non-main-thread-prompt-paths/02-hook-and-compact-special-paths.md)：覆盖 `hook_prompt`、`hook_agent`、verification 残留资产、compact summarize 旁路与 shared-prefix/fallback 分支。

### 3. `lZ(...)`、`cacheSafeParams` 与 request snapshot

- [04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md](./04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md)：覆盖 `ML / Cj4 / xe6 / I1z` 生命周期、fork-family 的 snapshot producer，以及 fresh-build 与 reuse 的边界。

### 4. compat、agent definitions 与指令进入载体

- [04-non-main-thread-prompt-paths/04-compat-agent-definitions-and-instruction-entry.md](./04-non-main-thread-prompt-paths/04-compat-agent-definitions-and-instruction-entry.md)：覆盖 compat 在非主线程里的三种 carrier、verification 本地反证闭环、agent definitions 来源与 winner 规则、祖先目录/worktree/external include。

## 建议阅读顺序

1. 先看 [01-shared-merge-skeleton-and-overrides.md](./04-non-main-thread-prompt-paths/01-shared-merge-skeleton-and-overrides.md)，建立“哪些仍走主骨架、哪些只是前置裁剪”的最小框架。
2. 再看 [02-hook-and-compact-special-paths.md](./04-non-main-thread-prompt-paths/02-hook-and-compact-special-paths.md)，把 hook 与 compact 这些专用 prompt 路径从主骨架里剥离出来。
3. 然后看 [03-fork-family-cache-safe-params-and-snapshot-reuse.md](./04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md)，补齐 `lZ(...)` 与 request snapshot 的真实复用边界。
4. 最后看 [04-compat-agent-definitions-and-instruction-entry.md](./04-non-main-thread-prompt-paths/04-compat-agent-definitions-and-instruction-entry.md)，把 compat、agent definitions 与目录扫描规则收口。

## 与其它专题的边界

### 主线程 prompt 装配

- 主线程 `system/messages` layering、`$X / dj4 / Lx8 / bC / WK` 的标准装配流程，优先看 [03-prompt-assembly-and-context-layering.md](./03-prompt-assembly-and-context-layering.md)。
- 本组只回答这些装配产物如何在非主线程里被复用、裁剪或旁路，不再重复主线程装配细节。

### instruction discovery / compat 扫描

- `sj()`、`CLAUDE.md`、`@include` 与 rules loader 的主扫描链，以及 compat 与 `/init` 的边界，优先看 [02-instruction-discovery-and-rules.md](./02-instruction-discovery-and-rules.md)。
- 本组只补“这些产物如何进入非主线程 prompt”与“哪些路径不会重新 discovery”。

### subagent / resume / agent runtime

- subagent、fork、sidechain、resume、teammate runtime 的生命周期，优先看 [../03-ecosystem/01-resume-fork-sidechain-and-subagents.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents.md) 及其子页。
- 本组只讨论它们发起模型请求时的 prompt 载体与上下文继承，不展开 agent runtime 主流程。

### compact / turn 状态机

- compact 的 turn 内状态迁移、autocompact/reactive compact 与 transcript rebuild，优先看 [../01-runtime/04-agent-loop-and-compaction.md](../01-runtime/04-agent-loop-and-compaction.md)。
- 本组只拆 compact 在 prompt 层的 shared-prefix 与 dedicated summarize 路径。

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
