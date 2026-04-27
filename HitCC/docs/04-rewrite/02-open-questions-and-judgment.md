# 重写判断、阻塞级未决项与后续补证

## 本页用途

- 用来回答：基于当前发行版分析，是否已经足以开始重写高相似版本。
- 用来只保留那些会直接影响重写策略、模块边界或扩展位设计的未决项。

## 相关文件

- [01-rewrite-architecture.md](./01-rewrite-architecture.md)
- [../00-overview/01-scope-and-evidence.md](../00-overview/01-scope-and-evidence.md)
- [../02-execution/03-prompt-assembly-and-context-layering.md](../02-execution/03-prompt-assembly-and-context-layering.md)
- [../01-runtime/05-model-adapter-provider-and-auth.md](../01-runtime/05-model-adapter-provider-and-auth.md)
- [../01-runtime/12-settings-and-configuration-system.md](../01-runtime/12-settings-and-configuration-system.md)
- [../03-ecosystem/02-remote-persistence-and-bridge.md](../03-ecosystem/02-remote-persistence-and-bridge.md)
- [../05-appendix/02-evidence-map.md](../05-appendix/02-evidence-map.md)

## 当前判断

### 已经可以做什么

在当前证据边界下，已经足以开始：

- 可运行替代品的重建
- 高相似版本的模块化重写

但仍然不足以承诺：

- 原始源码文件结构还原
- 私有服务端实现还原
- 1:1 原版复刻

### 这个判断成立的前提

当前已经还原到足以支撑重写的主干包括：

- CLI 启动分流与命令树
- Session / Transcript / Resume / Fork 主模型
- settings/config source、merge、cache 与 key consumer 主边界
- 输入编译链
- 主循环与工具轮
- prompt discovery 与主线程/非主线程 prompt 主分层
- model/provider/auth/stream fallback/remote ingress 的主结构
- 工具执行、permission merge、hook 主骨架
- remote transcript persistence 与 bridge 本地主工作流

### 需要避免的误读

- 这不等于“原版工程结构已经还原”。
- 这不等于“服务端黑箱行为已经完全知道”。
- 这也不等于“可以把工程实现选择写成还原结论”。

## 当前阻塞级或边界级未决项

这些问题不会阻止 Phase 1 和 Phase 2 开工，但会直接影响扩展位设计、后期行为对齐或某些模块边界是否要留出弹性。

### 1. Request-level prompt 最终顺序与本地/服务端边界

当前已确认主线程与非主线程 prompt 主骨架，但仍未完全钉死：

- request-level 的最终线性顺序
- `vO()` 预留注入槽位的原始用途
- 服务端收到本地 payload 后是否还会额外拼装 context / compat / verification

这意味着：

- `prompt` 层可以开工
- 但 compose pipeline 不宜写成“只有一条永远固定的最终序列”

### 2. Hook 扩展面与少量边角时序

当前本地已经足以还原 hook 主 schema 与跨阶段主顺序，但仍未完全钉死：

- bundle 外是否还有额外 hook 事件或输出分支
- 少量 command / hook 完成时序边角

这意味着：

- hook runtime 可以按主骨架实现
- 但事件枚举和输出通道应保留扩展位，而不是封死成当前 bundle 可见集合

### 3. ContextModifier 的额外 producer

当前本地能直接确认的 concrete producer 主要是 SkillTool。  
剩余未决点是：

- 是否还存在 bundle 外、远端路径或未启用分支中的第二类 producer

这意味着：

- `ToolExecutionOutput` 应保留 `contextModifier` 扩展能力
- 但不要把“当前只看到一个 producer”误写成“系统设计上只允许一个 producer”

### 4. bridge / worker 的服务端正式语义

当前本地已能区分多种 token/credential 与还原语义，但仍无法直接证明：

- `environment_secret` 的失效条件
- `worker_epoch` 的正式服务端含义
- `pending_action` 在远端 UI / 服务端协调层中的真实消费方式

这意味着：

- remote / bridge 主结构已足以实现
- 但相关状态对象与错误还原逻辑仍应允许后续补字段和补分支

### 5. 少量只影响高相似度对齐的行为边角

这类问题主要影响后期对齐，不影响主架构开工：

- `Mi6("compact")` 的 bundle 外或灰度路径覆盖面
- 少量 telemetry / cache / header 的边缘行为
- 低频工具、局部 system/informational producer 与 TUI 微观交互细节

## 已降级为非阻塞项的内容

下面这些点可以继续记录，但不该再当成“是否能开工”的关键阻塞项：

- 早期 `option` 命中字符串的历史来源
- 少量低频 protocol-like `system` subtype / producer 的调用表与边缘可见性
- transcript 搜索 / 导出 / virtual scroll 的 dormant 支线原因
- `hVz / SVz / bVz / xVz / RVz / CVz / IVz / uVz` 这组槽位在 bundle 外的真实来源
- `bridge-kick` 是否在上游曾有完整注册链
- plan 的 web editor / dirty-state，以及 `ultraplanPendingChoice` 专属本地落地器这类更偏前台整合的问题

这些问题值得继续补证据，但它们已经不再决定“主逻辑是否足够重写”。

## 建议的工程动作

### 1. 以候选架构开工，而不是等待全部未知点归零

当前更稳的做法是：

- 用 [01-rewrite-architecture.md](./01-rewrite-architecture.md) 里的候选分层启动工程
- 把未决区实现成可收缩的扩展位
- 避免为了等待 1:1 证据而阻塞主干重建

### 2. 在未决区主动保留弹性

建议保留扩展位的地方包括：

- prompt compose pipeline
- hook event / hook output runtime
- `ToolExecutionOutput` 与 context modifier
- bridge / worker 状态对象与错误分支

### 3. 继续补证据时，回到专题页而不是在本页继续堆正文

后续补证据应回到各自主题页：

- prompt 与 request boundary：`02-execution/03*`
- hook / permission：`02-execution/01*`
- remote / bridge：`03-ecosystem/02`
- TUI：`03-ecosystem/07*`
- model/provider/auth：`01-runtime/05` 与 `01-runtime/06`

本页只保留：

- 当前能否开工的判断
- 阻塞级未决项
- 下一步该去哪里补证据

## 一句话结论

当前发行版已经足以支撑高相似版本的模块化重写；剩余未知主要集中在服务端黑箱、bundle 外扩展面和少量后期对齐细节，不应继续把它们和“能否开始重写”混为一谈。

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
