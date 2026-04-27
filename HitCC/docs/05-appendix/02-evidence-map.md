# 证据索引

> 本页把“现有主题文档”与“证据来源”对齐，目的是减少重复逆向和重复扫大文件。

## 证据口径

证据来源、置信度标签与“已确认 / 高可信推断 / 待验证”的统一口径，统一以：

- [../00-overview/01-scope-and-evidence.md](../00-overview/01-scope-and-evidence.md)

为准，本页不再重复维护一套平行定义。

仍需外部补证、但会影响重写判断的未知点，统一以：

- [../04-rewrite/02-open-questions-and-judgment.md](../04-rewrite/02-open-questions-and-judgment.md)

作为入口。

本页只负责：

- 把主题文档和直接证据来源对应起来
- 在新增主题页后补登记对应证据入口
- 给强结论维护“断言 -> 直接证据点 -> 反证/边界 -> 校对入口”的硬索引

## 断言登记规则

后续如果正文里出现强结论，优先在本页登记成下面这种结构，而不是只补一个“相关主题页”链接：

- 断言：
  - 只写可被校对的判断，不写泛化口号
- 直接证据点：
  - 优先登记 A 级证据对应的函数、分支、schema、帮助输出或文件
  - 若正文页已经把 bundle 命中点整理好，这里直接引用正文页与关键符号
- 交叉佐证：
  - 只放支撑断言稳定性的 B 级归纳，不拿它冒充直接证据
- 反证/边界：
  - 明确写出当前不能证明什么、哪些路径只看到负证、哪些部分仍依赖 bundle 外或服务端黑箱
- 校对入口：
  - 给出复核时应先看的正文页，避免重复扫 bundle

如果一个结论暂时只能写出“主题页导航”，还写不出“直接证据点/反证边界”，说明它还不该被当成强断言。

## 上下文缓存 / request snapshot 专题速查

这一组是当前“上下文缓存”与“可复用 request snapshot”文档线的直达索引。

### 主题页索引

- [../01-runtime/04-agent-loop-and-compaction.md](../01-runtime/04-agent-loop-and-compaction.md)
  - 专题导航；具体直达 `01-main-loop-state-caches-and-yield-surface`、`02-compaction-pipeline-and-auto-compact-tracking`、`03-no-tool-branch-recovery-stop-and-reactive-compact`、`04-tool-round-next-turn-and-terminal-reasons`
- [../02-execution/02-instruction-discovery-and-rules.md](../02-execution/02-instruction-discovery-and-rules.md)
  - 负责 `systemPromptSectionCache`、`Mi6/WQ9`、`_$()/vO()`、section 变化条件
- [../02-execution/03-prompt-assembly-and-context-layering.md](../02-execution/03-prompt-assembly-and-context-layering.md)
  - 专题导航；具体直达 `01-system-chain-default-sections-and-context-sources`、`03-api-payload-order-prompt-caching-and-final-boundary`
- [../02-execution/04-non-main-thread-prompt-paths.md](../02-execution/04-non-main-thread-prompt-paths.md)
  - 专题导航；具体直达 `01-shared-merge-skeleton-and-overrides`、`02-hook-and-compact-special-paths`、`03-fork-family-cache-safe-params-and-snapshot-reuse`

### 关键函数索引

- section cache：
  - `Lq8 / Vc8 / Ec8 / kHq / Yn`
- default system sections：
  - `$X / fT8 / R8_ / S8_ / C8_ / d8_ / i8_ / h8_ / c8_`
- 主线程 request build / prompt cache：
  - `_I4 / hZz / lqA / LZz`
- instruction load reason：
  - `Mi6 / WQ9 / Di6`
- request snapshot 生命周期：
  - `ML / Cj4 / xe6 / I1z / lZ / ts6`
- compact 与失效：
  - `po_ / jk6 / ZVq / Jk6 / Cn / Sn`

## 主题到证据的对应关系

### [../00-overview/01-scope-and-evidence.md](../00-overview/01-scope-and-evidence.md)

- 主要证据：bundle 静态分析范围、运行时探测记录、各主题页的汇总结论
- 可信度：高
- 用途：确定哪些结论已经足以指导重写，哪些还只是待验证

### [../00-overview/02-document-style-and-structure-conventions.md](../00-overview/02-document-style-and-structure-conventions.md)

- 主要证据：知识库当前采用的父页/子页职责边界、拆分页维护规则、迁移与压缩约定
- 可信度：高
- 用途：校对目录结构是否仍与正文职责边界一致，避免把导航页重新写回直接证据页

### [../01-runtime/01-product-cli-and-modes.md](../01-runtime/01-product-cli-and-modes.md)

- 主要证据：顶层入口分支、Commander 命令树、`--help` 输出、非交互判定分支
- 可信度：高
- 后续补证据重点：bridge 子命令参数、Chrome/native-host 的更细启动参数

### [../01-runtime/02-session-and-persistence.md](../01-runtime/02-session-and-persistence.md)

- 主要证据：session 状态对象、path 计算函数、transcript writer、resume 解析路径
- 可信度：高
- 后续补证据重点：file-history backup 的更细时机、remote sync 下的持久化冲突边界

### [../01-runtime/03-input-compilation.md](../01-runtime/03-input-compilation.md)

- 主要证据：`AU8 -> ihz -> BU4` 调用链、block 归一化、pasted image 处理、remote slash gate、`processSlashCommand(...) / nx_(...) / r74(...)`、`UserPromptSubmit` hooks
- 可信度：高
- 后续补证据重点：bundle 外/灰度 producer 是否会把 `document` 等更低频 block 直接喂进 `ihz(...)`，以及 remote-control 之外是否还存在第二层命令过滤

### [../01-runtime/04-agent-loop-and-compaction.md](../01-runtime/04-agent-loop-and-compaction.md)

- 主要证据：导航页；负责主循环/compact 四个拆分页的入口、阅读顺序与专题边界，不单独承载直接证据
- 可信度：高
- 用途：把主循环状态、compact 管线、无工具分支、工具轮与终止返回组织到同一主题组

### [../01-runtime/04-agent-loop-and-compaction/01-main-loop-state-caches-and-yield-surface.md](../01-runtime/04-agent-loop-and-compaction/01-main-loop-state-caches-and-yield-surface.md)

- 主要证据：`CC / po_` 最小状态机骨架、长生命周期状态 `J`、turn 内缓存、`stream_request_start / assistant / attachment / tombstone` 等 `yield` 面
- 可信度：高
- 后续补证据重点：少量低频 `yield` subtype 的 UI/sidecar 可见性边界

### [../01-runtime/04-agent-loop-and-compaction/02-compaction-pipeline-and-auto-compact-tracking.md](../01-runtime/04-agent-loop-and-compaction/02-compaction-pipeline-and-auto-compact-tracking.md)

- 主要证据：`microcompact`、`DEq(...)`、`compactionResult -> hn(...) -> transcript rebuild` 合同、`compact_boundary / preservedSegment`、`autoCompactTracking`
- 可信度：高
- 后续补证据重点：partial/full/session-memory compact 在更边缘配置下的差异

### [../01-runtime/04-agent-loop-and-compaction/03-no-tool-branch-recovery-stop-and-reactive-compact.md](../01-runtime/04-agent-loop-and-compaction/03-no-tool-branch-recovery-stop-and-reactive-compact.md)

- 主要证据：`z6 === false` 的 turn 尾部分支、reactive compact 接口合同、`max_output_tokens` 还原、`Rj4(...)` stop hook 子状态机
- 可信度：中高
- 后续补证据重点：reactive compact 未接线结论在其他 build 里的稳定性

### [../01-runtime/04-agent-loop-and-compaction/04-tool-round-next-turn-and-terminal-reasons.md](../01-runtime/04-agent-loop-and-compaction/04-tool-round-next-turn-and-terminal-reasons.md)

- 主要证据：`Re6 / Zx8` 工具轮、`hook_stopped_continuation`、延后一轮消费的 tool summary、`aborted_streaming / aborted_tools / model_error` 等终止返回
- 可信度：高
- 后续补证据重点：少量错误码与终止原因在 remote/backend 变体下的映射差异

### [../01-runtime/05-model-adapter-provider-and-auth.md](../01-runtime/05-model-adapter-provider-and-auth.md)

- 主要证据：`Jk6 / hC1 / VN8` 包装层、provider factory、first-party/3P 分支、auth token / apiKey 来源优先级
- 可信度：高
- 后续补证据重点：少量 header/cache 边缘行为；控制面接口总表已拆到 `01-runtime/10-control-plane-api-and-auxiliary-services.md`；`WebSearchTool` 细节已拆到 `01-runtime/07-web-search-tool.md`

### [../01-runtime/06-stream-processing-and-remote-transport.md](../01-runtime/06-stream-processing-and-remote-transport.md)

- 主要证据：`_I4(...)` streaming 累积逻辑、fallback 分支、`sdk-url`/ingress transport
- 可信度：高
- 后续补证据重点：远端 ingress 的更细 header/sequence 行为

### [../01-runtime/07-web-search-tool.md](../01-runtime/07-web-search-tool.md)

- 主要证据：`WebSearchTool.call(...)`、`Kl_(...)`、`_l_(...)`、`Ow4/jw4/Hw4`、`Jk6(...)`、`client.beta.messages.create(...)`、`sdk-tools.d.ts` 里的 `WebSearchInput/WebSearchOutput`
- 可信度：高
- 后续补证据重点：服务端真实搜索供应商、`web_search_20260209` / `UserLocation` 是否在其他路径已启用

### [../01-runtime/08-web-fetch-tool.md](../01-runtime/08-web-fetch-tool.md)

- 主要证据：`WebFetch` 本地工具实现、`Qo1 / CY4 / bY4 / IY4 / do1 / co1 / lo1 / UZ`、`sdk-tools.d.ts` 里的 `WebFetchInput/WebFetchOutput`、generic streaming 对 `web_fetch_tool_result` 的支持、bundle 内嵌 `WebFetchTool20260209` 文档
- 可信度：中高
- 后续补证据重点：`mb8` 预批准列表全集、二进制内容落地保存的 content-type 条件、是否存在尚未命中的 server-side `web_fetch` 运行时包装器

### [../01-runtime/09-api-lifecycle-and-telemetry.md](../01-runtime/09-api-lifecycle-and-telemetry.md)

- 主要证据：`Rg8 / H4A / qvz / initializeTelemetry / Fd8`、`Tu / CF1 / EBq / tL8 / IF1 / yBq`、`i$A / BO / cl8`、`Wg7 / Zbz / vG_ / U1z / amz`
- 可信度：高
- 后续补证据重点：`DISABLE_TELEMETRY` 的服务端侧含义；OTEL exporter 与 1P event logging 的剩余配置边角

### [../01-runtime/10-control-plane-api-and-auxiliary-services.md](../01-runtime/10-control-plane-api-and-auxiliary-services.md)

- 主要证据：导航页；负责 host/auth、OAuth/account、remote environment/session、bridge credential、GitHub/telemetry/MCP proxy 五个拆分页的入口与边界
- 可信度：高
- 用途：把 first-party 网络面与控制面主题拆成稳定阅读顺序，避免把总览页误当成接口细节主证据

### [../01-runtime/10-control-plane-api-and-auxiliary-services/01-hosts-auth-and-data-plane.md](../01-runtime/10-control-plane-api-and-auxiliary-services/01-hosts-auth-and-data-plane.md)

- 主要证据：first-party 四层网络面划分、`api.anthropic.com / platform.anthropic.com / claude.ai / mcp-proxy.anthropic.com`、数据面与 org-scoped 控制面的鉴权差异、`/v1/messages / /v1/models / /v1/files`
- 可信度：高
- 后续补证据重点：少量 supporting endpoints 的字段级 schema

### [../01-runtime/10-control-plane-api-and-auxiliary-services/02-oauth-and-account-control-plane.md](../01-runtime/10-control-plane-api-and-auxiliary-services/02-oauth-and-account-control-plane.md)

- 主要证据：authorize / token exchange、`/api/oauth/profile`、`/api/oauth/claude_cli/roles`、`/api/oauth/claude_cli/create_api_key`、OAuth scope 族
- 可信度：高
- 后续补证据重点：role/profile 返回体的更多低频字段

### [../01-runtime/10-control-plane-api-and-auxiliary-services/03-remote-environments-and-sessions.md](../01-runtime/10-control-plane-api-and-auxiliary-services/03-remote-environments-and-sessions.md)

- 主要证据：environment 枚举/创建、environment 列表的本地选择与消费面、`/v1/sessions*` REST 与 WebSocket 主链
- 可信度：高
- 后续补证据重点：environment/session 返回 schema 的边角字段与错误还原分支

### [../01-runtime/10-control-plane-api-and-auxiliary-services/04-bridge-credentials-and-worker-lifecycle.md](../01-runtime/10-control-plane-api-and-auxiliary-services/04-bridge-credentials-and-worker-lifecycle.md)

- 主要证据：`/v1/environments/bridge*`、`environment_secret -> work secret -> session_ingress_token`、`/v1/code/sessions/{id}/bridge` 响应字段、`worker_jwt` 刷新链、`worker_epoch`
- 可信度：高
- 后续补证据重点：`environment_secret` 轮换边界与服务端正式 TTL 语义

### [../01-runtime/10-control-plane-api-and-auxiliary-services/05-github-telemetry-and-mcp-proxy.md](../01-runtime/10-control-plane-api-and-auxiliary-services/05-github-telemetry-and-mcp-proxy.md)

- 主要证据：GitHub App / token-sync / import-token、`/api/event_logging/batch`、`ClaudeCodeInternalEvent / GrowthbookExperimentEvent`、`claudeai-proxy` MCP transport
- 可信度：中高
- 后续补证据重点：GitHub beta 常量命名来源与少量外围控制面边界

### [../01-runtime/11-non-llm-network-paths.md](../01-runtime/11-non-llm-network-paths.md)

- 主要证据：`connectVoiceStream / ip8 / useVoice`、`EL6 / I3z / b3z`、`initializeGrowthBook / tl / D_8 / EventSource / TZ1 / ZZ1.sendBatchWithRetry`、transcript share endpoint、remote transcript persistence、`1p_failed_events.*.json` 本地补偿缓存、`statsStore / getFpsMetrics`
- 可信度：高
- 后续补证据重点：voice 服务端 STT provider 细节、plugin install counts 上游 stats 数据生成链、`statsStore` 与更大 perf/telemetry exporter 的完整对应关系

### [../01-runtime/12-settings-and-configuration-system.md](../01-runtime/12-settings-and-configuration-system.md)

- 主要证据：导航页；负责 source/path、加载与 merge、缓存与写回、CLI 注入与 schema、消费索引与重写边界五个拆分页的入口与边界
- 可信度：高
- 用途：把配置系统从单页大纲改造成按职责拆分的稳定入口，避免把总览页误写成实现细节全集

### [../01-runtime/12-settings-and-configuration-system/01-source-model-and-paths.md](../01-runtime/12-settings-and-configuration-system/01-source-model-and-paths.md)

- 主要证据：5 个正式 settings source、`--setting-sources` 真实边界、用户根目录与 enterprise policy 根目录、`userSettings / projectSettings / localSettings / flagSettings / policySettings` 路径族
- 可信度：高
- 后续补证据重点：不同平台上的目录 fallback 差异

### [../01-runtime/12-settings-and-configuration-system/02-loading-policy-and-merge.md](../01-runtime/12-settings-and-configuration-system/02-loading-policy-and-merge.md)

- 主要证据：`ye(path)` 与 `_D()` 校验链、`C28(...)` permission rules 预清洗、`PZ3()` effective settings 装配、`policySettings` fallback、remote managed settings 与 policy limits merge 规则
- 可信度：高
- 后续补证据重点：服务端二次裁剪与 org policy 变体边界

### [../01-runtime/12-settings-and-configuration-system/03-cache-refresh-and-writeback.md](../01-runtime/12-settings-and-configuration-system/03-cache-refresh-and-writeback.md)

- 主要证据：两级 cache 与 plugin overlay cache、`BX()` 失效入口、文件 watcher、MDM/registry poll、`wA(source, patch)` 写回语义、config 面板与 app state 分流
- 可信度：中高
- 后续补证据重点：更多热刷新源在非桌面环境中的触发差异

### [../01-runtime/12-settings-and-configuration-system/04-cli-injection-schema-and-migration.md](../01-runtime/12-settings-and-configuration-system/04-cli-injection-schema-and-migration.md)

- 主要证据：`--settings / --setting-sources`、`apply_flag_settings`、settings change 的运行时反应链、schema 主分组、启动迁移矩阵与兼容入口
- 可信度：高
- 后续补证据重点：少量历史迁移项的输入兼容边界

### [../01-runtime/12-settings-and-configuration-system/05-key-consumers-and-rewrite-boundaries.md](../01-runtime/12-settings-and-configuration-system/05-key-consumers-and-rewrite-boundaries.md)

- 主要证据：高价值键族到消费点索引、`strictPluginOnlyCustomization` surface 边界、`sshConfigs` 负证与剩余不确定性、可直接继承的重写骨架
- 可信度：高
- 后续补证据重点：`sshConfigs` 是否仍有 bundle 外或灰度入口

### [../02-execution/01-tools-hooks-and-permissions.md](../02-execution/01-tools-hooks-and-permissions.md)

- 主要证据：导航页；负责工具执行内核、Hook 系统、permission 状态机、managed policy/sandbox/审批 backend 四个子专题与 Hook 拆分页入口
- 可信度：高
- 用途：把执行层的工具、Hook、权限三条线收束成稳定阅读顺序，不单独承载直接证据

### [../02-execution/01-tools-hooks-and-permissions/01-tool-execution-core.md](../02-execution/01-tools-hooks-and-permissions/01-tool-execution-core.md)

- 主要证据：`he6 / Mo_ / Re6 / Zx8` 主执行链、deferred tools / `ToolSearch`、`tool_result` 两层形态、配对修复与落盘、`AskUserQuestion` 特例
- 可信度：高
- 后续补证据重点：`tool_reference` 到真实 tool schema 注入的最后桥接层

### [../02-execution/01-tools-hooks-and-permissions/02-hook-system.md](../02-execution/01-tools-hooks-and-permissions/02-hook-system.md)

- 主要证据：导航页；负责 hook schema、特殊输出分支、运行时时序、`InstructionsLoaded` 在非主线程的覆盖面四个拆分页入口
- 可信度：高
- 用途：把 Hook 系统从执行器正文里拆开，避免把父页误当作单一事件/函数的直接证据

### [../02-execution/01-tools-hooks-and-permissions/02-hook-system/01-schema-instructionsloaded-and-event-inputs.md](../02-execution/01-tools-hooks-and-permissions/02-hook-system/01-schema-instructionsloaded-and-event-inputs.md)

- 主要证据：hook schema、`InstructionsLoaded` 触发链、事件输入结构与参数边界
- 可信度：高
- 后续补证据重点：少量低频 event input 的字段覆盖面

### [../02-execution/01-tools-hooks-and-permissions/02-hook-system/02-special-output-and-event-consumer-semantics.md](../02-execution/01-tools-hooks-and-permissions/02-hook-system/02-special-output-and-event-consumer-semantics.md)

- 主要证据：`hookSpecificOutput` 分支、`Stop / SessionEnd / Worktree*` 等特殊事件的消费语义与保留行为
- 可信度：中高
- 后续补证据重点：更低频特殊输出类型的 transcript/UI 显示边界

### [../02-execution/01-tools-hooks-and-permissions/02-hook-system/03-runtime-order-and-cross-stage-timing.md](../02-execution/01-tools-hooks-and-permissions/02-hook-system/03-runtime-order-and-cross-stage-timing.md)

- 主要证据：Hook 与 permission 相关阶段的顺序图、跨阶段 timing 与 merge 点
- 可信度：高
- 后续补证据重点：流式工具与 hook 并行时的剩余时序边角

### [../02-execution/01-tools-hooks-and-permissions/02-hook-system/04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md](../02-execution/01-tools-hooks-and-permissions/02-hook-system/04-instructionsloaded-non-main-thread-coverage-and-dispatch-boundaries.md)

- 主要证据：`InstructionsLoaded` 在非主线程 prompt、fork-family、compact/hook 专用路径里的覆盖面与派发边界
- 可信度：中高
- 后续补证据重点：verification/compat 变体里是否还存在额外派发路径

### [../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md](../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md)

- 主要证据：`D0z / YP` permission core、`za / Hy6 / oy6 / LqA / Qs6 / SV / IqA`、dangerous allow rules 剥离与还原、auto classifier 输入与失败语义
- 可信度：高
- 后续补证据重点：classifier 在更极端 tool schema 下的判定残差

### [../02-execution/01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md](../02-execution/01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md)

- 主要证据：`allowManagedPermissionRulesOnly`、`RG8(...)` sandbox 合流、`--permission-prompt-tool`、`toolUseConfirmQueue`、remote/direct/ssh、headless/SDK/bridge 的 ask backend
- 可信度：高
- 后续补证据重点：`orphaned-permission` 与少量 approval backend 变体边界

### [../02-execution/02-instruction-discovery-and-rules.md](../02-execution/02-instruction-discovery-and-rules.md)

- 主要证据：`sj()` 扫描链、`_$() / vO() / wb1()`、`Lq8 / Vc8 / Ec8 / kHq / Yn`、`$X / fT8 / R8_ / S8_ / i8_`、compat 与 `/init` 的边界、`claudeMdExcludes`
- 可信度：中高
- 后续补证据重点：`vO()` 预留第二注入槽位的原始用途

### [../02-execution/03-prompt-assembly-and-context-layering.md](../02-execution/03-prompt-assembly-and-context-layering.md)

- 主要证据：导航页；负责主线程 prompt 的 system 链、attachment 顺序、payload/cache、request-level 注入四个拆分页入口
- 可信度：中高
- 用途：把主线程 prompt 装配拆成稳定阅读顺序，不单独承载直接证据

### [../02-execution/03-prompt-assembly-and-context-layering/01-system-chain-default-sections-and-context-sources.md](../02-execution/03-prompt-assembly-and-context-layering/01-system-chain-default-sections-and-context-sources.md)

- 主要证据：`bC(...)`、`dj4(...)`、`vO()`、`Mi6("compact")`、`$X(...)`、skills 在 prompt 里的最小落点、`deferred_tools_delta` 与真实 tool schema 注入边界
- 可信度：高
- 后续补证据重点：`vO()` 第二注入槽位的原始用途

### [../02-execution/03-prompt-assembly-and-context-layering/02-attachment-order-skill-plan-meta-and-message-merge.md](../02-execution/03-prompt-assembly-and-context-layering/02-attachment-order-skill-plan-meta-and-message-merge.md)

- 主要证据：`invoked_skills` 在 compact/resume 里的顺序、当前轮 attachment 生成顺序、`skill_listing / plan_mode / critical_system_reminder` 位置、`ClaudeMd` 前缀链与 compat 当前边界
- 可信度：中高
- 后续补证据重点：少量低频 attachment 在 merge 前后的排序差异

### [../02-execution/03-prompt-assembly-and-context-layering/03-api-payload-order-prompt-caching-and-final-boundary.md](../02-execution/03-prompt-assembly-and-context-layering/03-api-payload-order-prompt-caching-and-final-boundary.md)

- 主要证据：`_I4(...) -> _X(...) -> LZz(...) -> hZz(...)` 对象级顺序、cache breakpoint、`cache_reference`、`cache_edits` 当前停用状态、system prompt 分块缓存、最终 `payload.system / payload.messages` 边界
- 可信度：高
- 后续补证据重点：prompt cache 在更多 provider/transport 组合里的剩余差异

### [../02-execution/03-prompt-assembly-and-context-layering/04-request-level-injection-layers-and-local-server-boundary.md](../02-execution/03-prompt-assembly-and-context-layering/04-request-level-injection-layers-and-local-server-boundary.md)

- 主要证据：`prompt-text`、`schema/request options`、`transport` 三层注入拆分，以及 `_I4 / AI4 / hN(...)` 之后本地仍可证明的边界
- 可信度：中高
- 后续补证据重点：服务端是否还会追加黑箱 context

### [../02-execution/04-non-main-thread-prompt-paths.md](../02-execution/04-non-main-thread-prompt-paths.md)

- 主要证据：导航页；负责共享 merge 骨架、hook/compact 专用路径、fork-family snapshot 复用、compat/agent definitions 四个拆分页入口
- 可信度：中高
- 用途：把非主线程 prompt 相关证据拆成稳定子专题，不单独承载直接证据

### [../02-execution/04-non-main-thread-prompt-paths/01-shared-merge-skeleton-and-overrides.md](../02-execution/04-non-main-thread-prompt-paths/01-shared-merge-skeleton-and-overrides.md)

- 主要证据：非主线程三分类里的前两类共享 merge 骨架、`BN(...)`、`CC(...)`、`omitClaudeMd`、`Explore/Plan` 裁剪、`SubagentStart` hook 注入位置
- 可信度：高
- 后续补证据重点：更多模式裁剪组合的边界

### [../02-execution/04-non-main-thread-prompt-paths/02-hook-and-compact-special-paths.md](../02-execution/04-non-main-thread-prompt-paths/02-hook-and-compact-special-paths.md)

- 主要证据：`hook_prompt`、`hook_agent`、verification 残留资产、compact summarize 旁路、shared-prefix/fallback 分支
- 可信度：中高
- 后续补证据重点：verification 家族在更完整 build 中是否仍有活 wiring

### [../02-execution/04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md](../02-execution/04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md)

- 主要证据：`lZ(...)`、`cacheSafeParams`、`ML / Cj4 / xe6 / I1z` 生命周期、fork-family 的 snapshot producer、fresh-build 与 reuse 边界
- 可信度：高
- 后续补证据重点：reuse 失败时的回退条件与 cache invalidation 细节

### [../02-execution/04-non-main-thread-prompt-paths/04-compat-agent-definitions-and-instruction-entry.md](../02-execution/04-non-main-thread-prompt-paths/04-compat-agent-definitions-and-instruction-entry.md)

- 主要证据：compat 在非主线程里的三种 carrier、verification 本地反证闭环、agent definitions 来源与 winner 规则、祖先目录/worktree/external include
- 可信度：中高
- 后续补证据重点：agent definitions 在更多 host/runtime 变体下的来源优先级

### [../02-execution/05-attachments-and-context-modifiers.md](../02-execution/05-attachments-and-context-modifiers.md)

- 主要证据：导航页；负责 attachment 专题的子页入口与主题边界，不单独承载直接证据
- 可信度：高
- 用途：把 producer/lifecycle、payload/materialize、`contextModifier` consumer 三条正文线组织到同一目录

### [../02-execution/05-attachments-and-context-modifiers/01-attachment-lifecycle-and-producers.md](../02-execution/05-attachments-and-context-modifiers/01-attachment-lifecycle-and-producers.md)

- 主要证据：`P6z(...)` producer 矩阵、`KE6(...) / Nq(...)` attachment 包装、input compilation 的 loading gate、compact keep attachment 顺序、`Su_(...)` resume restore、`AVq(...) / mN8(...)` plan 还原分流
- 可信度：中高
- 后续补证据重点：少量 compact keep 边缘类型、UI 隐藏 attachment 与 transcript 渲染的更细边界

### [../02-execution/05-attachments-and-context-modifiers/02-high-value-attachment-payloads-and-materialization.md](../02-execution/05-attachments-and-context-modifiers/02-high-value-attachment-payloads-and-materialization.md)

- 主要证据：`dt1(...)` 的 prompt 归一化/丢弃规则、`queued_command` / `plan_file_reference` / `invoked_skills` / `relevant_memories` / `mcp_resource` / `task_status` / `async_hook_response` / `hook_additional_context` / usage attachments 的具体 materialize 逻辑
- 可信度：中高
- 后续补证据重点：低频 attachment payload 的完整字段族；`hook_*` 边缘类型在非主线程 prompt 中的精确覆盖面

### [../02-execution/05-attachments-and-context-modifiers/03-context-modifier-and-executor-consumers.md](../02-execution/05-attachments-and-context-modifiers/03-context-modifier-and-executor-consumers.md)

- 主要证据：`SkillTool` concrete `contextModifier`、执行器串行/并发 consumer、remote/bridge 直接缓存写入反证
- 可信度：中高
- 后续补证据重点：bundle 外/服务端侧是否存在第二个 concrete producer

### [../02-execution/06-context-runtime-and-tool-use-context.md](../02-execution/06-context-runtime-and-tool-use-context.md)

- 主要证据：`ts6(...)` 的 `ToolUseContext` 克隆规则、`BN(...)` 的 request/tool context 裁剪、`Cx / Zjq / Ly6 / av6` 的 `readFileState` 基线重建、`SkillTool` 的 concrete `contextModifier`、`discoveredSkillNames` 的 reset/clear 但未消费状态
- 可信度：中高
- 后续补证据重点：`vO()` 预留第二注入槽位的原始用途、远端服务端是否额外叠加 `systemContext`、远端是否存在本地 bundle 未暴露的额外 `contextModifier` producer

### [../03-ecosystem/01-resume-fork-sidechain-and-subagents.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents.md)

- 主要证据：`03-ecosystem` 这一组主题的导航性总结
- 可信度：高
- 用途：把 Resume/Fork/Sidechain、Agent Team 模型、mailbox 协议、teammate runtime 分页组织

### [../03-ecosystem/01-resume-fork-sidechain-and-subagents/01-resume-fork-sidechain-and-subagent-core.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents/01-resume-fork-sidechain-and-subagent-core.md)

- 主要证据：resume/fork/sidechain 路径、subagent transcript、`I76(...)` / `hq4(...)` / `CC`
- 可信度：高
- 后续补证据重点：subagent 边缘流程与 sidechain persistence 细节

### [../03-ecosystem/01-resume-fork-sidechain-and-subagents/02-agent-team-and-task-model.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents/02-agent-team-and-task-model.md)

- 主要证据：`TeamCreate / TeamDelete`、`AgentInput` teammate 分支、team config、task list 工具调用链
- 可信度：中高
- 后续补证据重点：team roster 字段全集、task list 更细状态机

### [../03-ecosystem/01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md)

- 主要证据：Agent Team mailbox / permission / plan approval、idle/shutdown 协议、inbox 存储格式
- 可信度：中高
- 后续补证据重点：team auto-approve 边界、mailbox message schema 全量枚举

### [../03-ecosystem/01-resume-fork-sidechain-and-subagents/04-teammate-runtime-and-backends.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents/04-teammate-runtime-and-backends.md)

- 主要证据：`in_process_teammate` task、`Du()` / `BA6()` backend 分流、task owner claim/unassign 路径
- 可信度：中高
- 后续补证据重点：pane backend tick/backoff、`F$_()` 是否为未启用预留分支

### [../03-ecosystem/02-remote-persistence-and-bridge.md](../03-ecosystem/02-remote-persistence-and-bridge.md)

- 主要证据：remote-control / `--remote` / `--sdk-url` 本地工作流矩阵、bridge credential handoff、remote transcript persistence、`409` 冲突还原、`bridge-kick` fault model 与其失联 handle 负证、`pending_action` 的只读不还原边界、web/app 与 IDE/desktop（Desktop config 导入、auto-connect、`ide_connected` 握手）配合证据
- 可信度：高
- 后续补证据重点：服务端语义边界（`environment_secret` TTL / `worker_epoch` 正式定义），而不是本地工作流主干

### [../03-ecosystem/03-plan-system.md](../03-ecosystem/03-plan-system.md)

- 主要证据：导航页；负责 plan file 生命周期、enter/exit 与 `/plan`、退出审批 UI、attachments/持久化/team approval 四个拆分页入口
- 可信度：中高
- 用途：把 plan 主题拆成稳定阅读顺序，不单独承载直接证据

### [../03-ecosystem/03-plan-system/01-runtime-objects-and-plan-file-lifecycle.md](../03-ecosystem/03-plan-system/01-runtime-objects-and-plan-file-lifecycle.md)

- 主要证据：`jX / AP / uF / Aw / mN8 / oq_ / BN8 / AVq`、`prePlanMode / hasExitedPlanMode / needsPlanModeExitAttachment`、plan file 的路径命名、还原链与 fork 复制链
- 可信度：中高
- 后续补证据重点：`plansDirectory` 越界保护的更多边界条件、subagent plan file 在更多还原分支里的命中覆盖面

### [../03-ecosystem/03-plan-system/02-enter-exit-and-plan-command.md](../03-ecosystem/03-plan-system/02-enter-exit-and-plan-command.md)

- 主要证据：`EnterPlanMode / ExitPlanMode` 的状态迁移、`prePlanMode` 还原分支、`/plan` local-jsx 子命令、external editor 打开链与 plan mode reentry/exit bookkeeping
- 可信度：中高
- 后续补证据重点：`/plan` 各低频子命令与异常分支的输出细节、外部编辑器失败时的产品级降级提示

### [../03-ecosystem/03-plan-system/03-exit-approval-ui-planwasedited-and-ultraplan-bridge.md](../03-ecosystem/03-plan-system/03-exit-approval-ui-planwasedited-and-ultraplan-bridge.md)

- 主要证据：`Cm4(...)` 审批 UI 状态机、`planWasEdited` 的本地判定、`S2z(...)` 的回填、`FC8(...)` 的 `set_permission_mode { mode:"plan", ultraplan:true }` 注入、`B2z / c2z` 的 ultraplan 回传与 `needsAttention` 链
- 可信度：中高
- 后续补证据重点：web 端 plan editor 的完整 dirty-state producer、`ultraplanPendingChoice` 的最终专属落地器

### [../03-ecosystem/03-plan-system/04-attachments-persistence-and-team-approval.md](../03-ecosystem/03-plan-system/04-attachments-persistence-and-team-approval.md)

- 主要证据：`plan_mode / plan_mode_reentry / plan_mode_exit / plan_file_reference` 的 producer 与 payload、compact/resume keep 链、teammate `plan_approval_request` mailbox 协议、remote ultraplan 的本地还原边界
- 可信度：中高
- 后续补证据重点：team approval 在更多 backend 变体下的分支差异、少量低频 attachment 的 transcript 可见性边界

### [../03-ecosystem/04-mcp-system.md](../03-ecosystem/04-mcp-system.md)

- 主要证据：`ZT6 / IHq / U8_ / C8_`、`Ao6 / XN6 / XC`、`.mcp.json` 原子写入、`Fw6 / nk6 / oLq`、`z$6 / prompts/list`、`notifications/claude/channel` 注册链、主循环 `options.mcpTools = appState.mcp.tools`、`sdk-tools.d.ts` 中的 resource/subscribe/polling schema
- 可信度：中高
- 后续补证据重点：resource subscribe/polling update 的本地 producer、以及服务端侧额外 MCP prompt 拼装

### [../03-ecosystem/05-skill-system.md](../03-ecosystem/05-skill-system.md)

- 主要证据：`IE6 / FH4 / pE6 / gE6 / d6z / c6z / ZC8 / NJ6 / Su_ / vVq`、`SkillTool` 的 `contextModifier`、`ts6(...)` 对 discovery triggers 的重置、`CC4 / A0 / rS / qn1 / pO6` 的多层 registry 合成、`_r1 / uE6 / tr / xb8` 的条件激活链
- 可信度：中高
- 后续补证据重点：bundle 外是否仍有额外 skill producer；`discoveredSkillNames / RC4` 在本地 bundle 中已更接近空壳/预留槽位

### [../03-ecosystem/06-plugin-system.md](../03-ecosystem/06-plugin-system.md)

- 主要证据：`e36 / Ve / aB6 / jj1` 的 plugin 与 marketplace schema、`installed_plugins.json / known_marketplaces.json` 读写链、`RJ4 / CJ4 / SJ4 / bJ4 / AM / mD` 的 runtime 装配链、`G26 / zt1 / K68 / mF / ZA6 / bt6` 的能力注入链、`Lt1 / HJ4 / JJ4` 的依赖解析与 demotion、`Rt1 / ZJ4 / qe_ / Ke_` 的 builtin 与 `--plugin-dir` 覆盖规则
- 可信度：中高
- 后续补证据重点：`ht1` 内建 plugin 注册表的 producer 与具体条目；其余 `plugin validate`、`/plugin` 主状态机、`pip` 非对称支持已基本钉死

### [../03-ecosystem/07-tui-system.md](../03-ecosystem/07-tui-system.md)

- 主要证据：导航页；负责 TUI 主题的子页入口、阅读顺序与跨专题边界，不单独承载直接证据
- 可信度：高
- 用途：把 root/render、transcript、input、dialogs、local JSX、permission dispatch、message renderer、tool-result renderer 八条正文线组织到同一目录

### [../03-ecosystem/07-tui-system/01-repl-root-and-render-pipeline.md](../03-ecosystem/07-tui-system/01-repl-root-and-render-pipeline.md)

- 主要证据：`f3A(...)` 主 REPL root、`screen === "transcript"` 的独立分支、`Obz(scrollable/bottom/overlay/modal)`、`Aj6/LOz` 的消息区标准化与 windowing、`TOz/scrollRef/renderRange` 的 dormant virtual-scroll 支线、主 REPL 中 `rL = false`
- 可信度：中高
- 后续补证据重点：brief/bash/collapsed read search 的更细视觉策略，以及 `rL` 被硬关闭的更上游原因

### [../03-ecosystem/07-tui-system/02-transcript-and-rewind.md](../03-ecosystem/07-tui-system/02-transcript-and-rewind.md)

- 主要证据：transcript 分支里的 `/ / n/N / q / v` handler、`Lx4()` 对 renderer search API 的桥接、renderer `scanElementSubtree(...)` 的离屏扫描、以及当前 `rL = false` 导致这组 handler 未接活；`r4A(...)` 的 rewind/restore/summarize 选项与 `fVq(...)` summarize-from-here 路径
- 可信度：中高
- 后续补证据重点：renderer 搜索命中的几何结构、`rL` 的 build-time 来源、`message-selector` 下游 restore/fork 对 session 状态的更细粒度影响

### [../03-ecosystem/07-tui-system/03-input-footer-and-voice.md](../03-ecosystem/07-tui-system/03-input-footer-and-voice.md)

- 主要证据：`PF4/oLz(...)` 输入枢纽、`chat:*` / `attachments:*` / `footer:*` / `help:dismiss` 快捷键、stash/external-editor/image-paste 路径、`pCz(...)`/`uc4(...)` voice 锚点与 push-to-talk 处理
- 可信度：中高
- 后续补证据重点：history search 面板的几何/候选细节

### [../03-ecosystem/07-tui-system/04-dialogs-and-approvals.md](../03-ecosystem/07-tui-system/04-dialogs-and-approvals.md)

- 主要证据：`UA8()` 的 `focusedInputDialog` 优先级链、`IX()` 的分支取消逻辑、`zB4/e5A/XB4/MB4/Rx4/Cx4/RQ4` 等 dialog/callout 组件
- 可信度：中高
- 后续补证据重点：`ide-onboarding/plugin-hint/desktop-upsell` 细节，以及非审批类 callout 的完整文案边界

### [../03-ecosystem/07-tui-system/05-help-settings-model-theme-and-diff.md](../03-ecosystem/07-tui-system/05-help-settings-model-theme-and-diff.md)

- 主要证据：`WW4(...)` help overlay、`wL6(...)` ThemePicker、`x26(...)` ModelPicker、`BD4(...)` settings registry、`$4z(...)` DiffDialog，以及 `config/settings`、`theme`、`help`、`diff` local JSX 注册
- 可信度：中高
- 后续补证据重点：`Status` 具体字段来源、`Usage` 返回 schema、更多 settings 类 local JSX，以及早期 `option` 命中字符串的原始来源

### [../03-ecosystem/07-tui-system/06-tool-permission-dispatch.md](../03-ecosystem/07-tui-system/06-tool-permission-dispatch.md)

- 主要证据：`mVz(...)` 的 switch 表、`pu4/Tm4/fm4/KB4/Em4/Rm4/Cm4/bm4/xm4/nm4/Nm4` 等 tool-specific 审批组件、`IQ/HF8` 两类审批壳，以及 `H46(...)` 通用 fallback 审批器
- 可信度：中高
- 后续补证据重点：若可能，继续确认 `hVz/SVz/bVz/xVz` 这四个保留槽位是否来自 bundle 外注入、build-time 裁剪，还是别的动态装配路径；审批主链本身已不再依赖这组黑箱存在

### [../03-ecosystem/07-tui-system/07-message-row-and-subtype-renderers.md](../03-ecosystem/07-tui-system/07-message-row-and-subtype-renderers.md)

- 主要证据：`OOz(...)`、`hC/Cx_(...)`、`bx_(...)`、`Ix_(...)`、`W74(...)`、`N74(...)`、`E74(...)`、`P74(...)`、`xo(...)`、`POz(...)` 的分派链，以及 `cj6(...)` / remote adapter 对 protocol-like system event 的放行边界
- 可信度：中高
- 后续补证据重点：是否还有其他工具实现 `renderGroupedToolUse(...)`；system/info 可见性主干已基本闭环

### [../03-ecosystem/07-tui-system/08-tool-result-renderers.md](../03-ecosystem/07-tui-system/08-tool-result-renderers.md)

- 主要证据：`Abq / aCq / cCq / nCq`、`toolUseResult` sidecar 写回链、`lookup` 构建、`H2q/lv8/lv6` 的结果落盘预览、以及 `Read/Bash/Edit/Write/NotebookEdit/WebFetch/WebSearch` 的 `renderToolResultMessage(...)`
- 可信度：中高
- 后续补证据重点：少量低频工具的 result renderer 细节，以及 bundle 外/remote 路径是否会再裁剪 `toolUseResult`

### [../04-rewrite/01-rewrite-architecture.md](../04-rewrite/01-rewrite-architecture.md)

- 主要证据：前面所有主题的归纳结果，不依赖单一函数
- 可信度：中高
- 用途：作为候选架构页，固定可直接继承的职责边界、接口还原目标与落地顺序

### [../04-rewrite/02-open-questions-and-judgment.md](../04-rewrite/02-open-questions-and-judgment.md)

- 主要证据：现有所有结论的汇总，以及尚未被 A 级证据完全钉死的部分
- 可信度：高
- 用途：决定“现在是否可以开工重写”，并集中维护阻塞级未决项与补证入口

## 关键结论复核入口

下面这些结论仍然重要，但它们各自已经有更合适的主承载页；本页只保留“先去哪里复核”的硬入口，不再在附录里重复展开主叙事。

### 1. 是否已经足以重写高相似版本

- 主判断页：
  - [../00-overview/01-scope-and-evidence.md](../00-overview/01-scope-and-evidence.md)
  - [../04-rewrite/02-open-questions-and-judgment.md](../04-rewrite/02-open-questions-and-judgment.md)
- 关键证据页：
  - [../01-runtime/01-product-cli-and-modes.md](../01-runtime/01-product-cli-and-modes.md)
  - [../01-runtime/02-session-and-persistence.md](../01-runtime/02-session-and-persistence.md)
  - [../01-runtime/04-agent-loop-and-compaction/01-main-loop-state-caches-and-yield-surface.md](../01-runtime/04-agent-loop-and-compaction/01-main-loop-state-caches-and-yield-surface.md)
  - [../01-runtime/04-agent-loop-and-compaction/02-compaction-pipeline-and-auto-compact-tracking.md](../01-runtime/04-agent-loop-and-compaction/02-compaction-pipeline-and-auto-compact-tracking.md)
  - [../02-execution/01-tools-hooks-and-permissions/01-tool-execution-core.md](../02-execution/01-tools-hooks-and-permissions/01-tool-execution-core.md)
  - [../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md](../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md)
  - [../03-ecosystem/01-resume-fork-sidechain-and-subagents/01-resume-fork-sidechain-and-subagent-core.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents/01-resume-fork-sidechain-and-subagent-core.md)
- 相关边界：
  - “不承诺还原原始源码文件结构 / 私有服务端实现 / 1:1 原版复刻” 统一看 `00-overview/01`
  - “当前阻塞级未决项与开工策略” 统一看 `04-rewrite/02`

### 2. 网络通信模块是否已经形成主分层闭环

- 主判断页：
  - [../00-overview/01-scope-and-evidence.md](../00-overview/01-scope-and-evidence.md)
- 关键证据页：
  - [../01-runtime/06-stream-processing-and-remote-transport.md](../01-runtime/06-stream-processing-and-remote-transport.md)
  - [../01-runtime/09-api-lifecycle-and-telemetry.md](../01-runtime/09-api-lifecycle-and-telemetry.md)
  - [../01-runtime/10-control-plane-api-and-auxiliary-services/01-hosts-auth-and-data-plane.md](../01-runtime/10-control-plane-api-and-auxiliary-services/01-hosts-auth-and-data-plane.md)
  - [../01-runtime/10-control-plane-api-and-auxiliary-services/03-remote-environments-and-sessions.md](../01-runtime/10-control-plane-api-and-auxiliary-services/03-remote-environments-and-sessions.md)
  - [../01-runtime/10-control-plane-api-and-auxiliary-services/04-bridge-credentials-and-worker-lifecycle.md](../01-runtime/10-control-plane-api-and-auxiliary-services/04-bridge-credentials-and-worker-lifecycle.md)
  - [../01-runtime/10-control-plane-api-and-auxiliary-services/05-github-telemetry-and-mcp-proxy.md](../01-runtime/10-control-plane-api-and-auxiliary-services/05-github-telemetry-and-mcp-proxy.md)
  - [../01-runtime/11-non-llm-network-paths.md](../01-runtime/11-non-llm-network-paths.md)
  - [../03-ecosystem/02-remote-persistence-and-bridge.md](../03-ecosystem/02-remote-persistence-and-bridge.md)
- 相关边界：
  - `environment_secret / session_ingress_token / worker_epoch` 等服务端语义黑箱，统一回到 `01-runtime/10` 与 `03-ecosystem/02`
  - telemetry/exporter、voice provider、stats 上游来源等剩余未知，统一回到对应 runtime 专题页

### 3. 是否还值得以“还原原源码”为主目标

- 主判断页：
  - [../00-overview/01-scope-and-evidence.md](../00-overview/01-scope-and-evidence.md)
  - [../04-rewrite/01-rewrite-architecture.md](../04-rewrite/01-rewrite-architecture.md)
  - [../04-rewrite/02-open-questions-and-judgment.md](../04-rewrite/02-open-questions-and-judgment.md)
- 复核重点：
  - 总览页负责说明“文档不承诺什么”
  - rewrite 页负责说明“为什么当前更适合按职责边界重写，而不是假设原始文件树已还原”

### 4. Prompt / Context 主链与未决边界

- 主判断页：
  - [../04-rewrite/02-open-questions-and-judgment.md](../04-rewrite/02-open-questions-and-judgment.md)
- 关键证据页：
  - [../02-execution/02-instruction-discovery-and-rules.md](../02-execution/02-instruction-discovery-and-rules.md)
  - [../02-execution/03-prompt-assembly-and-context-layering/01-system-chain-default-sections-and-context-sources.md](../02-execution/03-prompt-assembly-and-context-layering/01-system-chain-default-sections-and-context-sources.md)
  - [../02-execution/03-prompt-assembly-and-context-layering/03-api-payload-order-prompt-caching-and-final-boundary.md](../02-execution/03-prompt-assembly-and-context-layering/03-api-payload-order-prompt-caching-and-final-boundary.md)
  - [../02-execution/04-non-main-thread-prompt-paths/02-hook-and-compact-special-paths.md](../02-execution/04-non-main-thread-prompt-paths/02-hook-and-compact-special-paths.md)
  - [../02-execution/04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md](../02-execution/04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md)
  - [../01-runtime/05-model-adapter-provider-and-auth.md](../01-runtime/05-model-adapter-provider-and-auth.md)
- 相关边界：
  - `vO()` 第二注入槽位、`Mi6("compact")` 覆盖面、verification/compat/服务端二次注入等剩余问题，统一回到这些专题页与 `04-rewrite/02`

### 5. 上下文缓存 / request snapshot 是否已形成本地闭环

- 速查入口：
  - 先看本页前面的“上下文缓存 / request snapshot 专题速查”
- 关键证据页：
  - [../01-runtime/04-agent-loop-and-compaction/02-compaction-pipeline-and-auto-compact-tracking.md](../01-runtime/04-agent-loop-and-compaction/02-compaction-pipeline-and-auto-compact-tracking.md)
  - [../02-execution/02-instruction-discovery-and-rules.md](../02-execution/02-instruction-discovery-and-rules.md)
  - [../02-execution/03-prompt-assembly-and-context-layering/03-api-payload-order-prompt-caching-and-final-boundary.md](../02-execution/03-prompt-assembly-and-context-layering/03-api-payload-order-prompt-caching-and-final-boundary.md)
  - [../02-execution/04-non-main-thread-prompt-paths/01-shared-merge-skeleton-and-overrides.md](../02-execution/04-non-main-thread-prompt-paths/01-shared-merge-skeleton-and-overrides.md)
  - [../02-execution/04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md](../02-execution/04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md)
- 相关边界：
  - 这里的“闭环”只指本地 bundle 可见范围
  - 不等于服务端没有额外 cache layer

### 6. `contextModifier` / `ToolUseContext` 的真实地位

- 主判断页：
  - [../02-execution/06-context-runtime-and-tool-use-context.md](../02-execution/06-context-runtime-and-tool-use-context.md)
- 关键证据页：
  - [../02-execution/01-tools-hooks-and-permissions/01-tool-execution-core.md](../02-execution/01-tools-hooks-and-permissions/01-tool-execution-core.md)
  - [../02-execution/05-attachments-and-context-modifiers/03-context-modifier-and-executor-consumers.md](../02-execution/05-attachments-and-context-modifiers/03-context-modifier-and-executor-consumers.md)
  - [../03-ecosystem/05-skill-system.md](../03-ecosystem/05-skill-system.md)
- 相关边界：
  - 当前本地直接看到的 concrete producer 主要仍是 `SkillTool`
  - 是否存在 bundle 外、远端或服务端侧的第二类 producer，统一保持未决

## 后续补证据建议

- 若还要继续补 prompt/context 这条线，优先看 `vO()` 预留第二注入槽位与 verification 家族碎片的 build-time 来源。
- 再补 bundle 外 hook 扩展分支与少量流式取消边角。
- Prompt 相关补证据时，统一登记到本页，避免同一问题在多份正文里重复写。

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
