# TUI 系统

## 本页用途

- 作为 TUI 主题的导航页，固定子页分工、阅读顺序和跨专题边界。
- 不再在父页重复维护 root、状态轴、dialog、renderer 的正文分析；这些内容统一下沉到子页。

## 相关文件

- [../01-runtime/01-product-cli-and-modes.md](../01-runtime/01-product-cli-and-modes.md)
- [02-remote-persistence-and-bridge.md](./02-remote-persistence-and-bridge.md)
- [03-plan-system.md](./03-plan-system.md)
- [04-mcp-system.md](./04-mcp-system.md)
- [05-skill-system.md](./05-skill-system.md)
- [06-plugin-system.md](./06-plugin-system.md)
- [07-tui-system/01-repl-root-and-render-pipeline.md](./07-tui-system/01-repl-root-and-render-pipeline.md)
- [07-tui-system/02-transcript-and-rewind.md](./07-tui-system/02-transcript-and-rewind.md)
- [07-tui-system/03-input-footer-and-voice.md](./07-tui-system/03-input-footer-and-voice.md)
- [07-tui-system/04-dialogs-and-approvals.md](./07-tui-system/04-dialogs-and-approvals.md)
- [07-tui-system/05-help-settings-model-theme-and-diff.md](./07-tui-system/05-help-settings-model-theme-and-diff.md)
- [07-tui-system/06-tool-permission-dispatch.md](./07-tui-system/06-tool-permission-dispatch.md)
- [07-tui-system/07-message-row-and-subtype-renderers.md](./07-tui-system/07-message-row-and-subtype-renderers.md)
- [07-tui-system/08-tool-result-renderers.md](./07-tui-system/08-tool-result-renderers.md)

## 专题拆分

### 1. REPL root 与 render pipeline

- [07-tui-system/01-repl-root-and-render-pipeline.md](./07-tui-system/01-repl-root-and-render-pipeline.md)：覆盖两套 root、`screen === "transcript"` 分支、`Obz(...)` 四槽位、`Aj6` render pipeline 与 keybinding context。

### 2. transcript 与 rewind

- [07-tui-system/02-transcript-and-rewind.md](./07-tui-system/02-transcript-and-rewind.md)：覆盖 transcript 专门视图、搜索/导出/show-all、message selector 与 rewind/summarize/restore 分支。

### 3. 输入区、footer 与 voice

- [07-tui-system/03-input-footer-and-voice.md](./07-tui-system/03-input-footer-and-voice.md)：覆盖 `PF4/oLz(...)` 输入枢纽、attachments/footer 焦点、`chat:cycleMode` 与 permission mode 轴、stash/external editor/image paste 和 voice。

### 4. dialogs、approvals 与 callouts

- [07-tui-system/04-dialogs-and-approvals.md](./07-tui-system/04-dialogs-and-approvals.md)：覆盖 `focusedInputDialog` 优先级链、permission/sandbox/worker approvals，以及各类 elicitation/callout。

### 5. Help、Settings、Model/Theme 与 Diff

- [07-tui-system/05-help-settings-model-theme-and-diff.md](./07-tui-system/05-help-settings-model-theme-and-diff.md)：覆盖 help overlay、settings registry 与子对话框、ThemePicker/ModelPicker、local JSX 面板，以及 `option` 不再当作主 screen 的当前判断。

### 6. tool-specific 审批分派

- [07-tui-system/06-tool-permission-dispatch.md](./07-tui-system/06-tool-permission-dispatch.md)：覆盖 `zB4(...)` 总分发器、`mVz(...)` switch 表、path-aware 审批壳和 plan/skill/AskUserQuestion 等专用审批组件。

### 7. message row 与 subtype renderer

- [07-tui-system/07-message-row-and-subtype-renderers.md](./07-tui-system/07-message-row-and-subtype-renderers.md)：覆盖顶层 `message.type` 分派、user/assistant/system block subtype、grouped/collapsed render 与 transcript 可见性差异。

### 8. tool result renderer 家族

- [07-tui-system/08-tool-result-renderers.md](./07-tui-system/08-tool-result-renderers.md)：覆盖 `Abq(...)` 与 `toolUseResult` sidecar、lookup/progress/rejection/error 特判，以及高价值结果渲染器家族。

## 建议阅读顺序

1. 先看 [01-repl-root-and-render-pipeline.md](./07-tui-system/01-repl-root-and-render-pipeline.md)，建立 root、布局槽位和消息区主渲染链。
2. 再看 [03-input-footer-and-voice.md](./07-tui-system/03-input-footer-and-voice.md) 与 [04-dialogs-and-approvals.md](./07-tui-system/04-dialogs-and-approvals.md)，补齐输入层、permission mode、dialog 优先级和交互前台。
3. 接着看 [02-transcript-and-rewind.md](./07-tui-system/02-transcript-and-rewind.md)，把 transcript 视图和 rewind 主链接上。
4. 然后看 [05-help-settings-model-theme-and-diff.md](./07-tui-system/05-help-settings-model-theme-and-diff.md) 与 [06-tool-permission-dispatch.md](./07-tui-system/06-tool-permission-dispatch.md)，补齐 local JSX 面板和 tool-specific 审批树。
5. 最后看 [07-message-row-and-subtype-renderers.md](./07-tui-system/07-message-row-and-subtype-renderers.md) 与 [08-tool-result-renderers.md](./07-tui-system/08-tool-result-renderers.md)，完成消息行和 tool result 的最末级渲染还原。

## 与其它专题的边界

### 运行模式与入口

- TUI / Headless 的入口分流、命令树与非交互判定不在本页展开。
- 这组边界优先看 [../01-runtime/01-product-cli-and-modes.md](../01-runtime/01-product-cli-and-modes.md)。

### remote / bridge / transcript 持久化

- remote-control、bridge、远端 transcript 持久化与冲突还原不在 TUI 父页展开。
- 这组边界优先看 [02-remote-persistence-and-bridge.md](./02-remote-persistence-and-bridge.md)。

### permissions / ask backend / tool execution

- TUI 只处理本地前台状态机与渲染，不重复展开工具执行器、permission 决策核心和 ask backend transport。
- 这组边界优先看：
  - [../02-execution/01-tools-hooks-and-permissions.md](../02-execution/01-tools-hooks-and-permissions.md)
  - [../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md](../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md)
  - [../02-execution/01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md](../02-execution/01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md)

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
