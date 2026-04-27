# 主线程 Prompt 装配与 Context Layering

## 本页用途

- 这页不再承载主线程 prompt 装配的全部正文，而改成 `02-execution` 下 prompt layering 主题的总览与导航。
- 原先混在一页里的内容，已经拆成 system 链与默认 sections、attachment 顺序与 message merge、API payload/cache 与最终边界、request-level 注入层级与本地/服务端黑箱边界四个专题页。

## 相关文件

- [03-prompt-assembly-and-context-layering/01-system-chain-default-sections-and-context-sources.md](./03-prompt-assembly-and-context-layering/01-system-chain-default-sections-and-context-sources.md)
- [03-prompt-assembly-and-context-layering/02-attachment-order-skill-plan-meta-and-message-merge.md](./03-prompt-assembly-and-context-layering/02-attachment-order-skill-plan-meta-and-message-merge.md)
- [03-prompt-assembly-and-context-layering/03-api-payload-order-prompt-caching-and-final-boundary.md](./03-prompt-assembly-and-context-layering/03-api-payload-order-prompt-caching-and-final-boundary.md)
- [03-prompt-assembly-and-context-layering/04-request-level-injection-layers-and-local-server-boundary.md](./03-prompt-assembly-and-context-layering/04-request-level-injection-layers-and-local-server-boundary.md)
- [02-instruction-discovery-and-rules.md](./02-instruction-discovery-and-rules.md)
- [04-non-main-thread-prompt-paths.md](./04-non-main-thread-prompt-paths.md)
- [05-attachments-and-context-modifiers.md](./05-attachments-and-context-modifiers.md)
- [06-context-runtime-and-tool-use-context.md](./06-context-runtime-and-tool-use-context.md)
- [../03-ecosystem/05-skill-system.md](../03-ecosystem/05-skill-system.md)
- [../04-rewrite/02-open-questions-and-judgment.md](../04-rewrite/02-open-questions-and-judgment.md)

## 拆分后的主题边界

### 1. system 链、默认 sections 与上下文来源

- [03-prompt-assembly-and-context-layering/01-system-chain-default-sections-and-context-sources.md](./03-prompt-assembly-and-context-layering/01-system-chain-default-sections-and-context-sources.md)：覆盖 `bC(...)`、`dj4(...)`、`vO()`、`Mi6("compact")`、`$X(...)`、skills 在 prompt 里的最小落点，以及 `deferred_tools_delta` 与真实 tool schema 注入的边界。

### 2. attachment 顺序、skill/plan 元信息与消息合并

- [03-prompt-assembly-and-context-layering/02-attachment-order-skill-plan-meta-and-message-merge.md](./03-prompt-assembly-and-context-layering/02-attachment-order-skill-plan-meta-and-message-merge.md)：覆盖 `invoked_skills` 在 compact/resume 里的顺序、普通当前轮 attachment 生成顺序、`skill_listing / plan_mode / critical_system_reminder` 的位置，以及 `ClaudeMd` 前缀链和 compat 的当前边界。

### 3. API payload 顺序、prompt caching 与最终 `system/messages` 边界

- [03-prompt-assembly-and-context-layering/03-api-payload-order-prompt-caching-and-final-boundary.md](./03-prompt-assembly-and-context-layering/03-api-payload-order-prompt-caching-and-final-boundary.md)：覆盖 `_I4(...) -> _X(...) -> LZz(...) -> hZz(...)` 的对象级顺序、cache breakpoint、`cache_reference`、`cache_edits` 的当前停用状态、system prompt 分块缓存，以及最终 `payload.system / payload.messages` 边界。

### 4. request-level 注入分层与本地/服务端黑箱边界

- [03-prompt-assembly-and-context-layering/04-request-level-injection-layers-and-local-server-boundary.md](./03-prompt-assembly-and-context-layering/04-request-level-injection-layers-and-local-server-boundary.md)：覆盖 `prompt-text`、`schema/request options`、`transport` 三层注入拆分，以及 `_I4 / AI4 / hN(...)` 之后本地还能证明什么、不能证明什么。

## 建议阅读顺序

1. 先看 [01-system-chain-default-sections-and-context-sources.md](./03-prompt-assembly-and-context-layering/01-system-chain-default-sections-and-context-sources.md)，建立主线程 prompt 的 `system` 链和默认来源。
2. 再看 [02-attachment-order-skill-plan-meta-and-message-merge.md](./03-prompt-assembly-and-context-layering/02-attachment-order-skill-plan-meta-and-message-merge.md)，把 attachment、skill/plan 元信息和 `ClaudeMd` 前缀链接到一起。
3. 然后看 [03-api-payload-order-prompt-caching-and-final-boundary.md](./03-prompt-assembly-and-context-layering/03-api-payload-order-prompt-caching-and-final-boundary.md)，补齐最终发给 `beta.messages.create(...)` 的 payload 形状。
4. 最后看 [04-request-level-injection-layers-and-local-server-boundary.md](./03-prompt-assembly-and-context-layering/04-request-level-injection-layers-and-local-server-boundary.md)，收口 request-level 注入分层与服务端黑箱边界。

## 与其它专题的边界

### instruction discovery / compat 扫描

- `sj()`、`CLAUDE.md`、`@include` 与 rules loader 的主扫描链，以及 compat 与 `/init` 的边界，优先看 [02-instruction-discovery-and-rules.md](./02-instruction-discovery-and-rules.md)。
- 本组只回答这些产物如何进入主线程 prompt，以及哪些内容不应再误写成 request-level `system`。

### 非主线程 prompt 路径

- `BN(...)`、`CC(...)`、fork-family、hook/compact summarize 与 compat agent 的旁路路径，优先看 [04-non-main-thread-prompt-paths.md](./04-non-main-thread-prompt-paths.md) 及其子页。
- 本组只固定主线程 prompt 装配本体，不重复展开非主线程如何复用或裁剪这些产物。

### attachment / context runtime / skill runtime

- attachment 生命周期、`contextModifier`、`ToolUseContext` 与 skills 的 registry/runtime 还原，不在本页展开。
- 这组边界优先看：
  - [05-attachments-and-context-modifiers.md](./05-attachments-and-context-modifiers.md)
  - [06-context-runtime-and-tool-use-context.md](./06-context-runtime-and-tool-use-context.md)
  - [../03-ecosystem/05-skill-system.md](../03-ecosystem/05-skill-system.md)

### 剩余未决点

- “服务端是否还会追加黑箱 context” 这类剩余判断，统一参考 [../04-rewrite/02-open-questions-and-judgment.md](../04-rewrite/02-open-questions-and-judgment.md)。

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
