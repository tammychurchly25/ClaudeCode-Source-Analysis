# 术语索引

> 本页用于统一“压缩名”“职责名”“文档里反复出现的运行时术语”。
>
> 原则是优先按职责理解，而不是执着于压缩变量名本身。

## 使用说明

- “压缩名”表示从 bundle 里看到的符号名。
- “建议职责名”表示重写时更适合采用的工程命名。
- “当前判断”表示基于现有证据对它的职责总结，不等于 1:1 原始实现名。

## 主执行链

### `jBz`

- 建议职责名：顶层快速分流入口
- 当前判断：处理 `--version`、bridge、chrome/native-host、MCP 等 fast-path，再懒加载主 CLI

### `_Bz`

- 建议职责名：主入口 `main`
- 当前判断：负责信号、deep-link、非交互判定、预加载 settings，再进入 Commander 程序

### `YBz`

- 建议职责名：CLI 程序装配器
- 当前判断：构建顶层命令树，并在 `preAction` 中统一做初始化接线

### `AU8`

- 建议职责名：输入编译器入口
- 当前判断：把原始输入编译成可进入主循环的 `messages + options` 结构

### `ihz`

- 建议职责名：输入预处理器
- 当前判断：处理图片、附件、slash command、remote 限制、本地命令分流

### `BU4`

- 建议职责名：普通输入消息构造器
- 当前判断：把普通用户输入包装成最终 user message，并返回 `shouldQuery`

### `CC`

- 建议职责名：主循环外壳
- 当前判断：创建 queued command 容器，调用 `po_`，在结束时统一收尾

### `po_`

- 建议职责名：多轮 agent 状态机
- 当前判断：真正的 turn loop，负责 compact、callModel、tool use、stop hook、还原与下一轮拼接

### `Jk6`

- 建议职责名：高层模型调用适配器
- 当前判断：把 `messages + systemPrompt + tools + options` 转成统一流事件，供主循环消费

### `VN8`

- 建议职责名：模型调用重试/降级包装层
- 当前判断：处理 retry、fallback、流式失败还原等模型调用防护逻辑

## 工具与权限链

### `he6`

- 建议职责名：单工具调度入口
- 当前判断：按 tool name / alias 找到工具定义，做基础分发与错误兜底

### `Mo_`

- 建议职责名：单工具执行包装器
- 当前判断：执行输入校验、PreToolUse、permission merge、tool.call、PostToolUse

### `Re6`

- 建议职责名：流式并发工具执行器
- 当前判断：在 `streamingToolExecution` 开启时管理并发安全工具的流式执行

### `Zx8`

- 建议职责名：传统批次工具执行器
- 当前判断：按并发安全性分块执行，并控制 `contextModifier` 的提交时机

## Session 与还原

### `I76`

- 建议职责名：Resume 还原器
- 当前判断：还原 transcript、plan、file-history backups，并处理 interrupted turn

### `hq4`

- 建议职责名：中断回补处理器
- 当前判断：识别 interrupted turn，必要时插入 continuation，并修正最后消息边界

## Prompt、Skill 与压缩

### `sj`

- 建议职责名：CLAUDE.md / memory 扫描主链
- 当前判断：扫描 `CLAUDE.md`、`.claude/rules/`、`CLAUDE.local.md`、AutoMem、TeamMem 等来源
- 当前更稳的落点：主线程里更像 `userContext.ClaudeMd` 的来源，而不是最终 request `system` 字段本身

### `_$(...)`

- 建议职责名：userContext 生成器
- 当前判断：读取 `sj()` 结果并装配主线程 `userContext`
- 当前已直接确认的字段：
  - `claudeMd?`
  - `currentDate`

### `vO`

- 建议职责名：systemContext 生成器
- 当前判断：主线程 `systemContext` 生成器
- 当前已直接确认的字段：
  - `gitStatus?`
- 当前更精确的判断：
  - `gitStatus` 不是对象，而是多行字符串快照
  - `vO()` 内部还残留一个未启用的第二注入槽位（`has_injection` / `K = null`）

### `wb1`

- 建议职责名：git 状态快照构造器
- 当前判断：生成 `systemContext.gitStatus`
- 当前已直接确认的组成：
  - 当前分支
  - 主分支
  - `git status --short`
  - 最近 5 条提交

### `Lx8`

- 建议职责名：userContext 前置注入器
- 当前判断：把 `userContext` 包成 `<system-reminder>` user meta message，前插到消息链

### `dj4`

- 建议职责名：systemContext 末尾拼接器
- 当前判断：对 `Object.entries(systemContext)` 做 `key: value` 串接后，追加到 system prompt sections 末尾

### `d6z`

- 建议职责名：动态 skill 扫描器
- 当前判断：扫描触发目录下的 `SKILL.md`，产出 `dynamic_skill` attachment

### `c6z`

- 建议职责名：skill 列表生成器
- 当前判断：根据当前 registry 生成 `skill_listing` attachment

### `Su_`

- 建议职责名：已调用 skill 还原器
- 当前判断：在 resume 时把 `invoked_skills` attachment 重新装回运行态

### `Mi6`

- 建议职责名：compact 指令加载入口
- 当前判断：与 compact 相关的 instructions 加载有关，但完整覆盖面仍待验证

## 运行时术语

### Transcript

- 当前判断：不是聊天记录，而是事件日志
- 包含内容：message、summary、title、task summary、mode、queue operation、content replacement、file-history snapshot 等

### Session State Store

- 当前判断：全局 app state 与 per-session state 的混合存储
- 作用：共享当前 cwd、sessionId、缓存、模型消耗、invoked skills、prompt cache 等信息

### ContextModifier

- 当前判断：工具执行后的上下文修改描述，不只是 UI 附件
- 已确认影响面：权限规则、模型选择、推理强度

### Sidechain

- 当前判断：与主会话分开的执行链路，可复用主循环，但 transcript 独立落盘

### Subagent

- 当前判断：Sidechain 的具体实例形态之一，落盘到 `subagents/agent-<id>.jsonl`

### Headless

- 当前判断：一等运行模式，不是 TUI 的简化输出
- 典型用途：`print/json/stream-json`
- 边界：可与 Bridge 共用部分主执行链，但不应把 `--sdk-url` 或 `remote-control` 直接并入 Headless 术语

### Bridge

- 当前判断：`remote-control`、`--remote`、`--sdk-url + stream-json` 这一组远程控制 / SDK 接入 / 远程 transport 模式族
- 边界：和本地 TUI/Headless 共享核心循环，但 transport、接入方与产品语义不同

## 命名建议

- 文档与代码中优先使用职责名，压缩名只作为注释或证据引用保留。
- 如果一个压缩名职责仍有争议，先在本页标成“待验证”，不要直接在正文里硬编码为稳定接口名。

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
