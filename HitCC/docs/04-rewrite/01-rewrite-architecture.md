# 重写候选架构、职责边界与落地顺序

## 本页用途

- 用来把前面已经还原出来的运行时职责边界，转成可执行的重写约束与候选架构。
- 用来说明哪些内容可以直接据此开工，哪些内容只能先预留扩展位，不能在这里提前写死。

## 相关文件

- [../00-overview/01-scope-and-evidence.md](../00-overview/01-scope-and-evidence.md)
- [../01-runtime/04-agent-loop-and-compaction.md](../01-runtime/04-agent-loop-and-compaction.md)
- [../01-runtime/05-model-adapter-provider-and-auth.md](../01-runtime/05-model-adapter-provider-and-auth.md)
- [../01-runtime/12-settings-and-configuration-system.md](../01-runtime/12-settings-and-configuration-system.md)
- [../02-execution/03-prompt-assembly-and-context-layering.md](../02-execution/03-prompt-assembly-and-context-layering.md)
- [../03-ecosystem/01-resume-fork-sidechain-and-subagents.md](../03-ecosystem/01-resume-fork-sidechain-and-subagents.md)
- [../03-ecosystem/02-remote-persistence-and-bridge.md](../03-ecosystem/02-remote-persistence-and-bridge.md)
- [02-open-questions-and-judgment.md](./02-open-questions-and-judgment.md)

## 使用边界

- 本页不是“原始源码目录结构还原页”。
- 本页给出的内容是**候选架构**，不是“已确认原版就这样分文件”。
- 若后续正文证据与本页候选设计冲突，以正文证据页为准，并回改本页。

## 当前可以直接继承的职责边界

基于现有 runtime、execution、ecosystem 三组正文，当前已经足以直接固定下面这些模块级职责：

- `cli`
  - 负责入口分流、Commander 命令树、非交互模式判定、启动前 settings 预加载
- `settings`
  - 负责 source/path、effective merge、cache/watcher、flag 注入、write-back，以及对 permission / plugin / MCP / model / env 的运行时配置分发
- `engine`
  - 负责 headless / repl 运行路径、输入编译、主 turn loop、模型调用门面
- `session`
  - 负责 transcript、resume、fork、sidechain、file history、统计与本地持久化
- `prompt`
  - 负责 instruction discovery、rules、skills、prompt compose、snapshot/cache
- `tools`
  - 负责 registry、执行器、并发工具调度、permission merge、tool result 回写
- `hooks`
  - 负责 hook schema、hook runtime、pre/post tool 与 session 级事件桥接
- `agents`
  - 负责 subagent / agent team 的 source load、winner 选择、launch 与约束裁剪
- `mcp / plugins`
  - 负责 MCP server/source 管理、plugin marketplace、安装缓存、能力注入
- `remote / bridge`
  - 负责 remote-control、remote session、direct connect、bridge transport 与冲突还原
- `ui`
  - 负责 TUI root、transcript、dialog、approval、tool result renderer 等前台表现

这些是当前能从发行版分析中稳定还原的**职责边界**。  
更细的文件拆分、类型命名与目录组织，仍属于工程实现选择。

## 候选分层

当前更稳的重写思路，不是先追求文件树长什么样，而是先保持下面这组层次分离：

### 1. 启动与命令层

- `cli` 只负责命令面、参数解析、启动分流与初始化时机。
- 不把 session、prompt、tool、remote 的实际运行逻辑继续塞回 CLI 命令处理函数。

### 2. 配置与策略层

- `settings` 应保持成独立层，而不是散落在 CLI、全局 store、UI toggle 和各个 consumer 里各自拼 effective config。
- permission、model、plugin、MCP、remote、env 这些消费面都很多，但 settings 仍应是单独模块，而不是 `shared/config.ts` 一类薄包装。

### 3. 主执行层

- `engine` 负责“输入进入系统后如何跑完整个 turn”。
- `headless` 与 `repl` 应共享 turn loop 和 model call 的核心运行骨架，只在 I/O 形态、前台 UI 和审批交互上分叉。

### 4. 持久化与工作现场层

- `session` 负责持久化模型和还原能力。
- 这一层应继续把 transcript、resume/fork、file-history、plan restore 等内容拆开，而不是重新塞成单个大 store。

### 5. Prompt 与上下文层

- `prompt` 负责 discovery、合成、cache、snapshot 和 request 边界。
- 与 `engine` 的边界应保持为：
  - `engine` 决定何时需要 prompt
  - `prompt` 决定 prompt 如何被构造

### 6. Tool / Permission / Hook 层

- `tools` 负责执行与结果提交。
- `permissions` 与 `hooks` 虽然强耦合 tool round，但不应直接埋在单一 tool executor 内。
- 保持这三者拆开，后续补齐 ask backend、managed policy、hook runtime 时才不会回到大函数。

### 7. 生态扩展层

- `agents`、`mcp`、`plugins`、`skills` 属于扩展能力面。
- 这些能力都需要接入 prompt、tool、settings、runtime，但不应反过来污染主循环的数据结构定义。

### 8. 远端与控制面层

- remote-control、bridge、direct connect、control-plane API 应独立成层。
- 它和 provider/model call 是正交关系，不应混为“模型请求的一个特殊分支”。

### 9. 表现层

- `ui` 只解决 TUI / dialog / renderer / input footer / transcript 这些前台表现问题。
- 不把状态机、tool 执行器、bridge 协议消费重新塞进组件树里。

## 一份可施工的目录骨架

下面这份目录骨架只是**一个可行拆法**，用来约束重写时的分层，不代表原始源代码文件树就是这样：

```text
src/
  cli/
    entry.ts
    program.ts
    commands/
  settings/
    sources.ts
    loader.ts
    merge.ts
    cache.ts
    writeback.ts
  engine/
    headless.ts
    repl.ts
    turn-loop.ts
    call-model.ts
    input-compiler.ts
  session/
    transcript.ts
    persistence.ts
    resume.ts
    fork.ts
    file-history.ts
    stats.ts
  prompt/
    discovery.ts
    compose.ts
    cache.ts
    rules.ts
    skills.ts
  tools/
    registry.ts
    executor.ts
    concurrent-runner.ts
    permissions.ts
    builtins/
  hooks/
    runtime.ts
    schema.ts
  agents/
    registry.ts
    launch.ts
  mcp/
    manager.ts
    client.ts
  plugins/
    manager.ts
    manifest.ts
  remote/
    control-plane.ts
    bridge.ts
    direct-connect.ts
  ui/
    app.tsx
    transcript/
    dialogs/
    renderers/
  state/
    app-state.ts
    session-state.ts
  shared/
    ids.ts
    paths.ts
    env.ts
    logger.ts
```

如果开始真正施工，建议优先保证：

- 单个文件只负责一个稳定问题域
- 大页里已经独立成专题的职责，不要在实现时再重新耦合
- 不要为了“像原版”而把已经拆清的边界重新揉回单文件

## 接口还原目标

这一节只描述**接口边界**，不把当前还没证死的字段细节提前写成最终类型定案。

### Transcript / Message 族

至少应保留：

- `user`
- `assistant`
- `system`
- `progress`
- `attachment`

并且要支持：

- block-based content
- tool use / tool result
- compact 与 retry 相关 system subtype
- tool use 关联关系

当前不宜提前写死的部分：

- 每个 `system` subtype 的完整字段族
- 远端协议型 system 行在不同前台路径中的最终可见性

### Session 持久化对象

至少应覆盖：

- session identity
- project/worktree 关联
- transcript 消息集合
- parent / fork 关系
- file-history / attribution / content replacement / compact snapshot 之类的工作现场

当前不宜提前写死的部分：

- 所有附属 snapshot 的完整字段结构
- 远端 session 相关扩展字段是否全部本地可见

### Compiled Input

至少应覆盖：

- 编译后的 messages
- 是否触发模型请求
- 本轮结果文本与输出模式
- 工具 allowlist / model override / plan 之类的请求级覆盖

当前不宜提前写死的部分：

- web/editor 侧 plan 注入的完整前端载荷
- 服务端收到 payload 后是否还会做额外 context 注入

### Tool Execution Output

至少应支持：

- 单条或多条 transcript 写回
- context modifier
- structured output / sidecar payload
- MCP 相关附加元数据
- 停止后续 continuation 的信号

当前不宜提前写死的部分：

- 除 SkillTool 之外是否还有 bundle 外 producer
- 一些低频工具的 sidecar 形状

### Hook Runtime Event

至少应支持：

- 直接消息输出
- permission 结果
- input 更新
- additional context 注入
- stop / preventContinuation 一类控制信号

当前不宜提前写死的部分：

- bundle 外是否还有额外事件分支
- 少量 command / hook 完成时序边角

### Turn Loop State

至少应跟踪：

- 当前消息集
- tool use context
- compact / recovery 状态
- pending tool summary

当前不宜提前写死的部分：

- `transition` 一类 branch marker 是否保留成调试/快照字段；就当前 bundle 而言，它不该被当成活控制流输入
- 所有 retry / telemetry / header 的边缘状态位
- 远端路径下服务端 sideband 行为的完整镜像

## 落地顺序

### Phase 1：可运行的 Headless 主干

目标：

- 跑通 `prompt -> model -> tool use -> transcript -> result`
- 支持 `json / jsonl / stream-json`
- 支持基础 resume / fork

优先实现的层：

- `cli`
- `settings`
- `engine`
- `session`
- `prompt`
- `tools`

### Phase 2：还原本地工作现场

补齐：

- file-history
- plan restore
- content replacement
- queued commands
- invoked skills
- attachment 体系

### Phase 3：还原外围能力

补齐：

- hooks
- MCP
- plugins
- agents / subagents
- remote-control / bridge
- TUI

### Phase 4：对齐边缘行为

逐步校对：

- request-level prompt 最终线性顺序
- `Mi6("compact")` 与相关失效链的完整覆盖面
- model fallback / retry 的全部边角
- 服务端或 bundle 外追加的 context / compat / verification 行为
- telemetry / cache / header 的边缘行为

## 暂不应提前定案的部分

下面这些点不妨碍开工，但不适合在本页里写成“最终设计已经确定”：

- request-level prompt 的最后顺序，以及本地/服务端边界后的残余注入点
- Hook 的 bundle 外扩展分支与少量边角时序
- 是否还有额外 ContextModifier producer
- bridge / worker 的服务端正式语义
- 少量只影响 1:1 复刻的 TUI 微观交互细节

更稳的工作方式是：

- 先按已确认职责边界拆模块
- 对未决区保留扩展位
- 等对应专题页继续补证据后，再收窄实现

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
