# Claude Code CLI 重建知识库

## 使用方式

- 目录化版本：`docs/`
- 维护原则：优先维护拆分后的文档；补证据、纠错和重写规划都在这里持续推进

## 入口导航

### 00-overview

- [01-scope-and-evidence.md](./01-scope-and-evidence.md)
  - 文档边界、当前结论、证据来源、置信度定义
- [02-document-style-and-structure-conventions.md](./02-document-style-and-structure-conventions.md)
  - 文档风格、父页/子页职责、拆分与迁移约定

### 01-runtime

- [../01-runtime/01-product-cli-and-modes.md](../01-runtime/01-product-cli-and-modes.md)
  - 产品形态、入口分流、命令树、运行模式
- [../01-runtime/02-session-and-persistence.md](../01-runtime/02-session-and-persistence.md)
  - 全局状态、Session、Transcript、持久化策略
- [../01-runtime/03-input-compilation.md](../01-runtime/03-input-compilation.md)
  - 输入本地编译、附件与 slash command 预处理
- [../01-runtime/04-agent-loop-and-compaction.md](../01-runtime/04-agent-loop-and-compaction.md)
  - 主循环与 compact 主题的总览与拆分导航
- [../01-runtime/04-agent-loop-and-compaction/01-main-loop-state-caches-and-yield-surface.md](../01-runtime/04-agent-loop-and-compaction/01-main-loop-state-caches-and-yield-surface.md)
  - 主循环骨架、运行态缓存与对外产出面
- [../01-runtime/04-agent-loop-and-compaction/02-compaction-pipeline-and-auto-compact-tracking.md](../01-runtime/04-agent-loop-and-compaction/02-compaction-pipeline-and-auto-compact-tracking.md)
  - compact 管线、触发来源与跟踪状态
- [../01-runtime/04-agent-loop-and-compaction/03-no-tool-branch-recovery-stop-and-reactive-compact.md](../01-runtime/04-agent-loop-and-compaction/03-no-tool-branch-recovery-stop-and-reactive-compact.md)
  - 无工具分支、reactive compact 与 stop 相关收尾
- [../01-runtime/04-agent-loop-and-compaction/04-tool-round-next-turn-and-terminal-reasons.md](../01-runtime/04-agent-loop-and-compaction/04-tool-round-next-turn-and-terminal-reasons.md)
  - 工具轮续转、延后摘要与终止原因
- [../01-runtime/05-model-adapter-provider-and-auth.md](../01-runtime/05-model-adapter-provider-and-auth.md)
  - 模型门面、provider 选择与鉴权来源
- [../01-runtime/06-stream-processing-and-remote-transport.md](../01-runtime/06-stream-processing-and-remote-transport.md)
  - 流事件映射、fallback 与远端传输
- [../01-runtime/07-web-search-tool.md](../01-runtime/07-web-search-tool.md)
  - Web 搜索工具包装、服务端调用链与返回边界
- [../01-runtime/08-web-fetch-tool.md](../01-runtime/08-web-fetch-tool.md)
  - Web 获取工具、本地抓取链与服务端边界
- [../01-runtime/09-api-lifecycle-and-telemetry.md](../01-runtime/09-api-lifecycle-and-telemetry.md)
  - telemetry 初始化、remote settings gating、OTEL/event logger 与开关矩阵
- [../01-runtime/10-control-plane-api-and-auxiliary-services.md](../01-runtime/10-control-plane-api-and-auxiliary-services.md)
  - data plane / control plane 分层、OAuth/account、remote environment/session、GitHub 接入、1P telemetry、MCP proxy
- [../01-runtime/11-non-llm-network-paths.md](../01-runtime/11-non-llm-network-paths.md)
  - voice WebSocket、plugin install counts、GrowthBook、transcript share 与其他非模型出网链
- [../01-runtime/12-settings-and-configuration-system.md](../01-runtime/12-settings-and-configuration-system.md)
  - settings source、路径、优先级、缓存、写回、CLI 注入与 schema 轮廓

### 02-execution

- [../02-execution/01-tools-hooks-and-permissions.md](../02-execution/01-tools-hooks-and-permissions.md)
  - 工具执行器、Hook、权限系统的总览与拆分导航
- [../02-execution/01-tools-hooks-and-permissions/01-tool-execution-core.md](../02-execution/01-tools-hooks-and-permissions/01-tool-execution-core.md)
  - 工具调度、执行包装、结果回写与并发路径
- [../02-execution/01-tools-hooks-and-permissions/02-hook-system.md](../02-execution/01-tools-hooks-and-permissions/02-hook-system.md)
  - Hook 主题的总览与拆分导航
- [../02-execution/01-tools-hooks-and-permissions/02-hook-system/01-schema-instructionsloaded-and-event-inputs.md](../02-execution/01-tools-hooks-and-permissions/02-hook-system/01-schema-instructionsloaded-and-event-inputs.md)
  - hook schema、InstructionsLoaded 与事件输入
- [../02-execution/01-tools-hooks-and-permissions/02-hook-system/02-special-output-and-event-consumer-semantics.md](../02-execution/01-tools-hooks-and-permissions/02-hook-system/02-special-output-and-event-consumer-semantics.md)
  - 特殊输出分支与关键事件消费语义
- [../02-execution/01-tools-hooks-and-permissions/02-hook-system/03-runtime-order-and-cross-stage-timing.md](../02-execution/01-tools-hooks-and-permissions/02-hook-system/03-runtime-order-and-cross-stage-timing.md)
  - hook 与权限相关阶段的顺序图
- [../02-execution/01-tools-hooks-and-permissions/02-hook-system/04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md](../02-execution/01-tools-hooks-and-permissions/02-hook-system/04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md)
  - InstructionsLoaded 在非主线程的覆盖面与派发边界
- [../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md](../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md)
  - permission mode 状态机、auto mode gate 与 classifier
- [../02-execution/01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md](../02-execution/01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md)
  - managed policy、sandbox 合流与审批后端
- [../02-execution/02-instruction-discovery-and-rules.md](../02-execution/02-instruction-discovery-and-rules.md)
  - instruction 扫描链、rules、compat 与排除规则
- [../02-execution/03-prompt-assembly-and-context-layering.md](../02-execution/03-prompt-assembly-and-context-layering.md)
  - 主线程 prompt 主题的总览与拆分导航
- [../02-execution/03-prompt-assembly-and-context-layering/01-system-chain-default-sections-and-context-sources.md](../02-execution/03-prompt-assembly-and-context-layering/01-system-chain-default-sections-and-context-sources.md)
  - system sections、上下文来源与 compact 相关失效边界
- [../02-execution/03-prompt-assembly-and-context-layering/02-attachment-order-skill-plan-meta-and-message-merge.md](../02-execution/03-prompt-assembly-and-context-layering/02-attachment-order-skill-plan-meta-and-message-merge.md)
  - attachment 顺序、skill/plan 注入与消息合并
- [../02-execution/03-prompt-assembly-and-context-layering/03-api-payload-order-prompt-caching-and-final-boundary.md](../02-execution/03-prompt-assembly-and-context-layering/03-api-payload-order-prompt-caching-and-final-boundary.md)
  - prompt caching 与最终 API payload 边界
- [../02-execution/03-prompt-assembly-and-context-layering/04-request-level-injection-layers-and-local-server-boundary.md](../02-execution/03-prompt-assembly-and-context-layering/04-request-level-injection-layers-and-local-server-boundary.md)
  - request 级注入层与本地/服务端边界
- [../02-execution/04-non-main-thread-prompt-paths.md](../02-execution/04-non-main-thread-prompt-paths.md)
  - 非主线程 prompt 主题的总览与拆分导航
- [../02-execution/04-non-main-thread-prompt-paths/01-shared-merge-skeleton-and-overrides.md](../02-execution/04-non-main-thread-prompt-paths/01-shared-merge-skeleton-and-overrides.md)
  - 共用 merge 骨架、模式裁剪与子代理注入位置
- [../02-execution/04-non-main-thread-prompt-paths/02-hook-and-compact-special-paths.md](../02-execution/04-non-main-thread-prompt-paths/02-hook-and-compact-special-paths.md)
  - hook/compact 特殊路径与 verification 残留边界
- [../02-execution/04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md](../02-execution/04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md)
  - fork-family snapshot 复用与 cacheSafeParams 边界
- [../02-execution/04-non-main-thread-prompt-paths/04-compat-agent-definitions-and-instruction-entry.md](../02-execution/04-non-main-thread-prompt-paths/04-compat-agent-definitions-and-instruction-entry.md)
  - compat 载体、agent definitions 来源与优先级
- [../02-execution/05-attachments-and-context-modifiers.md](../02-execution/05-attachments-and-context-modifiers.md)
  - attachment 与 ContextModifier 主题导航
- [../02-execution/06-context-runtime-and-tool-use-context.md](../02-execution/06-context-runtime-and-tool-use-context.md)
  - 运行时上下文分层、继承与裁剪规则

### 03-ecosystem

- [../03-ecosystem/01-resume-fork-sidechain-and-subagents.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents.md)
  - Resume/Fork/Sidechain、Subagent、Agent Team 的总览与拆分导航
- [../03-ecosystem/01-resume-fork-sidechain-and-subagents/01-resume-fork-sidechain-and-subagent-core.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents/01-resume-fork-sidechain-and-subagent-core.md)
  - Resume、Fork、Sidechain、Subagent 核心路径
- [../03-ecosystem/01-resume-fork-sidechain-and-subagents/02-agent-team-and-task-model.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents/02-agent-team-and-task-model.md)
  - Agent Team、TaskList、Roster 与协作模型
- [../03-ecosystem/01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md)
  - mailbox 协议、审批流与生命周期控制
- [../03-ecosystem/01-resume-fork-sidechain-and-subagents/04-teammate-runtime-and-backends.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents/04-teammate-runtime-and-backends.md)
  - teammate runtime、backend 分流与任务认领
- [../03-ecosystem/02-remote-persistence-and-bridge.md](../03-ecosystem/02-remote-persistence-and-bridge.md)
  - remote-control、bridge、远程 transcript 持久化与冲突还原
- [../03-ecosystem/03-plan-system.md](../03-ecosystem/03-plan-system.md)
  - Plan 主题的总览与拆分导航
- [../03-ecosystem/03-plan-system/01-runtime-objects-and-plan-file-lifecycle.md](../03-ecosystem/03-plan-system/01-runtime-objects-and-plan-file-lifecycle.md)
  - plan runtime object、plan file 生命周期与 fork 复制
- [../03-ecosystem/03-plan-system/02-enter-exit-and-plan-command.md](../03-ecosystem/03-plan-system/02-enter-exit-and-plan-command.md)
  - plan 模式的进入、退出与命令入口
- [../03-ecosystem/03-plan-system/03-exit-approval-ui-planwasedited-and-ultraplan-bridge.md](../03-ecosystem/03-plan-system/03-exit-approval-ui-planwasedited-and-ultraplan-bridge.md)
  - 退出审批 UI、编辑状态与 bridge 回传
- [../03-ecosystem/03-plan-system/04-attachments-persistence-and-team-approval.md](../03-ecosystem/03-plan-system/04-attachments-persistence-and-team-approval.md)
  - plan 相关 attachment、持久化与 team approval
- [../03-ecosystem/04-mcp-system.md](../03-ecosystem/04-mcp-system.md)
  - MCP 配置源、连接层、指令增量、resource、deferred tool
- [../03-ecosystem/05-skill-system.md](../03-ecosystem/05-skill-system.md)
  - skills 注册源、动态发现、运行时修改与 fork 语义
- [../03-ecosystem/06-plugin-system.md](../03-ecosystem/06-plugin-system.md)
  - plugin 的 marketplace、安装缓存、启停状态、依赖闭包与运行时注入
- [../03-ecosystem/07-tui-system.md](../03-ecosystem/07-tui-system.md)
  - TUI root、状态轴、对话框体系与 keybinding context
- [../03-ecosystem/07-tui-system/01-repl-root-and-render-pipeline.md](../03-ecosystem/07-tui-system/01-repl-root-and-render-pipeline.md)
  - 主 REPL root 与渲染管线
- [../03-ecosystem/07-tui-system/02-transcript-and-rewind.md](../03-ecosystem/07-tui-system/02-transcript-and-rewind.md)
  - transcript 搜索/导出、show-all、message-selector 与 rewind/summarize
- [../03-ecosystem/07-tui-system/03-input-footer-and-voice.md](../03-ecosystem/07-tui-system/03-input-footer-and-voice.md)
  - 输入区、attachments、footer、history/help 与 voice
- [../03-ecosystem/07-tui-system/04-dialogs-and-approvals.md](../03-ecosystem/07-tui-system/04-dialogs-and-approvals.md)
  - 对话框优先级、审批弹层与 callout 分支
- [../03-ecosystem/07-tui-system/05-help-settings-model-theme-and-diff.md](../03-ecosystem/07-tui-system/05-help-settings-model-theme-and-diff.md)
  - help、settings、theme/model picker 与 diff 对话框
- [../03-ecosystem/07-tui-system/06-tool-permission-dispatch.md](../03-ecosystem/07-tui-system/06-tool-permission-dispatch.md)
  - tool-specific 审批分派表
- [../03-ecosystem/07-tui-system/07-message-row-and-subtype-renderers.md](../03-ecosystem/07-tui-system/07-message-row-and-subtype-renderers.md)
  - 消息行渲染链、message type 与 block subtype
- [../03-ecosystem/07-tui-system/08-tool-result-renderers.md](../03-ecosystem/07-tui-system/08-tool-result-renderers.md)
  - tool result 渲染、sidecar 与错误/拒绝分支

### 04-rewrite

- [../04-rewrite/01-rewrite-architecture.md](../04-rewrite/01-rewrite-architecture.md)
  - 候选架构、职责边界、接口还原目标与落地顺序
- [../04-rewrite/02-open-questions-and-judgment.md](../04-rewrite/02-open-questions-and-judgment.md)
  - 能否开工的判断、阻塞级未决项与后续补证方向

### 05-appendix

- [../05-appendix/01-glossary.md](../05-appendix/01-glossary.md)
  - 压缩名、运行时术语、重写时建议命名
- [../05-appendix/02-evidence-map.md](../05-appendix/02-evidence-map.md)
  - 主题到证据来源的对应关系，方便后续补强

## 建议阅读顺序

1. 先看 [01-scope-and-evidence.md](./01-scope-and-evidence.md)，确认当前结论和可信度边界。
2. 再读 runtime 与 execution 两组文件，建立主执行链。
3. 接着读 ecosystem，补齐 Resume、Remote、MCP、TUI 等外围系统。
4. 最后读 rewrite 与 appendix，把“知道什么”和“该怎么重写”接起来。

## 维护约定

- 新增证据时，优先改对应主题文件，再在 evidence map 中登记。
- 如果某个压缩名的职责判断发生变化，先改 glossary，再回改正文引用。
- 结构调整、父页收缩和新增拆分页时，先遵守 [02-document-style-and-structure-conventions.md](./02-document-style-and-structure-conventions.md)。
- 不把历史整理过程和旧目录痕迹重新带回知识库首页。

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
