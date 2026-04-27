# Attachment 与 ContextModifier

## 本页用途

- 作为 attachment 主题的导航页，固定这一组子页的阅读入口和边界。
- 不再在父页重复维护 producer、materialize、`contextModifier` 的正文细节；这些内容统一下沉到子页。

## 相关文件

- [05-attachments-and-context-modifiers/01-attachment-lifecycle-and-producers.md](./05-attachments-and-context-modifiers/01-attachment-lifecycle-and-producers.md)
- [05-attachments-and-context-modifiers/02-high-value-attachment-payloads-and-materialization.md](./05-attachments-and-context-modifiers/02-high-value-attachment-payloads-and-materialization.md)
- [05-attachments-and-context-modifiers/03-context-modifier-and-executor-consumers.md](./05-attachments-and-context-modifiers/03-context-modifier-and-executor-consumers.md)
- [01-tools-hooks-and-permissions/01-tool-execution-core.md](./01-tools-hooks-and-permissions/01-tool-execution-core.md)
- [03-prompt-assembly-and-context-layering.md](./03-prompt-assembly-and-context-layering.md)
- [04-non-main-thread-prompt-paths.md](./04-non-main-thread-prompt-paths.md)
- [06-context-runtime-and-tool-use-context.md](./06-context-runtime-and-tool-use-context.md)
- [../01-runtime/03-input-compilation.md](../01-runtime/03-input-compilation.md)
- [../03-ecosystem/03-plan-system.md](../03-ecosystem/03-plan-system.md)
- [../03-ecosystem/05-skill-system.md](../03-ecosystem/05-skill-system.md)

## 专题拆分

### 1. attachment 生命周期与 producer

- [05-attachments-and-context-modifiers/01-attachment-lifecycle-and-producers.md](./05-attachments-and-context-modifiers/01-attachment-lifecycle-and-producers.md)：覆盖 `KE6(...) / Nq(...) / P6z(...)`、loading gate、transcript/UI 边界与 compact/resume 生命周期。

### 2. 高价值 attachment payload 与 materialize

- [05-attachments-and-context-modifiers/02-high-value-attachment-payloads-and-materialization.md](./05-attachments-and-context-modifiers/02-high-value-attachment-payloads-and-materialization.md)：覆盖 `dt1(...)` 的类型级消费矩阵，以及 `queued_command`、`plan_file_reference`、`invoked_skills`、`task_status`、`async_hook_response` 等高价值 payload。

### 3. `contextModifier` 与执行器 consumer

- [05-attachments-and-context-modifiers/03-context-modifier-and-executor-consumers.md](./05-attachments-and-context-modifiers/03-context-modifier-and-executor-consumers.md)：覆盖 `contextModifier` 运行时接口、串行/并发 consumer、`SkillTool` concrete producer 与 remote/bridge 反证。

## 建议阅读顺序

1. 先看 [01-attachment-lifecycle-and-producers.md](./05-attachments-and-context-modifiers/01-attachment-lifecycle-and-producers.md)，建立 attachment 的 producer、包装、loading gate 与 compact/resume 生命周期。
2. 再看 [02-high-value-attachment-payloads-and-materialization.md](./05-attachments-and-context-modifiers/02-high-value-attachment-payloads-and-materialization.md)，理解 `dt1(...)` 如何决定哪些 payload 会真正进入 prompt。
3. 最后看 [03-context-modifier-and-executor-consumers.md](./05-attachments-and-context-modifiers/03-context-modifier-and-executor-consumers.md)，把执行后运行态如何被修改补齐。

## 与其它专题的边界

### compact / resume

- attachment 的 keep 顺序、plan file 还原、invoked skill 还原，不在父页展开。
- 具体细节优先看：
  - [05-attachments-and-context-modifiers/01-attachment-lifecycle-and-producers.md](./05-attachments-and-context-modifiers/01-attachment-lifecycle-and-producers.md)
  - [../01-runtime/04-agent-loop-and-compaction.md](../01-runtime/04-agent-loop-and-compaction.md)
  - [../03-ecosystem/03-plan-system.md](../03-ecosystem/03-plan-system.md)
  - [../03-ecosystem/05-skill-system.md](../03-ecosystem/05-skill-system.md)

### prompt / context 黑箱

- attachment 最终如何进入 request-level prompt，不应和 prompt/context 黑箱混写。
- 这组边界优先看：
  - [03-prompt-assembly-and-context-layering.md](./03-prompt-assembly-and-context-layering.md)
  - [04-non-main-thread-prompt-paths.md](./04-non-main-thread-prompt-paths.md)
  - [06-context-runtime-and-tool-use-context.md](./06-context-runtime-and-tool-use-context.md)

### tool execution / permission

- attachment / `contextModifier` 的执行期消费面与工具执行器强耦合，但正文已拆开。
- 具体链路优先看：
  - [05-attachments-and-context-modifiers/03-context-modifier-and-executor-consumers.md](./05-attachments-and-context-modifiers/03-context-modifier-and-executor-consumers.md)
  - [01-tools-hooks-and-permissions/01-tool-execution-core.md](./01-tools-hooks-and-permissions/01-tool-execution-core.md)

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
