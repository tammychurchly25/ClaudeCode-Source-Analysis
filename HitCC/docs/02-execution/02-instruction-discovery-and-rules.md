# Instruction 发现与扫描规则

## 本页用途

- 用来单独整理 `sj()` 主扫描链、`CLAUDE.md` 系列文件、`.claude/rules/` 与 compat 相关判断。
- 用来把“哪些东西是 runtime source，哪些东西只是 `/init` 迁移输入”从主线程 prompt 装配链里拆开。

## 相关文件

- [01-tools-hooks-and-permissions.md](./01-tools-hooks-and-permissions.md)
- [03-prompt-assembly-and-context-layering.md](./03-prompt-assembly-and-context-layering.md)
- [04-non-main-thread-prompt-paths.md](./04-non-main-thread-prompt-paths.md)
- [../05-appendix/01-glossary.md](../05-appendix/01-glossary.md)

## Instruction 发现、扫描与 Rules/Skills 来源

这一条已经从“仅还原骨架”推进到“**主扫描顺序与本地 compat 结论基本钉死**，剩余边界主要落在 attachment 顺序细节和 bundle 外行为”。

### 明确存在的 prompt/instruction sources

source 枚举至少包括：

- `local`
- `user`
- `project`
- `dynamic`
- `enterprise`
- `claudeai`
- `managed`

说明 prompt 不是单一来源，而是分层拼装。

### 一个关键修正：`sj()` 不应再直接归入 system prompt 主链

补完直接证据后，必须修正此前文档里的一个核心判断：

- `sj()` 仍然是最重要的 instructions / memory 扫描器
- 但在**主线程运行时**，它的产出首先进入 `_$(...) -> userContext.ClaudeMd`
- 随后再经 `Lx8(...)` 变成前置 `<system-reminder>` user meta message
- 也就是说，`sj()` 主链更接近 **userContext/messages 注入链**
- 而不是最终 API request 的 `system` 字段主链

当前更稳的职责分层应改写为：

- `system` 链：运行时 header + `bC(...)` 外层 system prompt 合成 + `systemContext`
- `messages` 链：`userContext`、attachment/meta message、transcript 正常消息一起归一化后下发

### 已确认的 `sj()` 主扫描文件与目录

本轮已从 `sj()` 主扫描函数直接确认，它至少会扫描：

- `Managed` 级 `CLAUDE.md`
- `Managed` 级 `.claude/rules/`
- `User` 级 `CLAUDE.md`
- `User` 级 `.claude/rules/`
- 沿项目祖先目录向上查找的 `<dir>/CLAUDE.md`
- 沿项目祖先目录向上查找的 `<dir>/.claude/CLAUDE.md`
- 沿项目祖先目录向上查找的 `<dir>/.claude/rules/`
- 沿项目祖先目录向上查找的 `<dir>/claude.local.md`
- `additionalDirectoriesForClaudeMd` 指向目录下的 `CLAUDE.md / .claude/CLAUDE.md / .claude/rules/`
- 追加到末尾的 `AutoMem`
- 追加到末尾的 `TeamMem`

但这里必须再次强调：

- 这条顺序是 **`sj()` 的扫描顺序**
- 不应直接等同于“最终 API request 的 system sections 顺序”

### 关于 compat 文件

此前文档写到：

- `AGENTS.md`
- `.cursor/rules`
- `.cursorrules`
- `.github/copilot-instructions.md`
- `.windsurfrules`
- `.clinerules`
- `.mcp.json`

现在可以把 compat 相关结论再收紧一层。

本轮**没有在 `sj()` 这条主扫描链里直接看到这些文件名常量**；它们目前更像是：

- `/init`/初始化类 prompt 明确提及的“应参考文件”
- 某些 IDE/tool ecosystem 兼容逻辑中的概念来源

额外可确认的是：

- `AGENTS.md / .cursor/rules / .cursorrules / .github/copilot-instructions.md / .windsurfrules / .clinerules` 目前看到的命中基本都落在 `/init` 类提示词文案里
- `.mcp.json` 确实有独立的 MCP 配置读写路径，但这属于 MCP config 子系统证据，**不是 prompt 注入证据**
- `init` builtin prompt 明确要求子代理在初始化时读取这些 compat 文件，并把其中重要部分写入生成的 `CLAUDE.md`
- `init` 本身只是一次性 prompt command：其 `getPromptForCommand()` 返回的长文本会在执行 `/init` 的那一轮被包成 meta user message 送入模型，而不是注册成常驻 system/userContext 来源
- `subagent / fork / compact / hook agent / sdk-url / bridge` 当前也**没有看到第二套 compat discovery**
- 但它们进入 non-main-thread 的载体不能再一概写成“复用现成上下文”
  - `BN(...)` 默认分支会 fresh-build `userContext/systemContext`
  - `lZ(...)` 只消费外部给的 `cacheSafeParams`，是否复用旧 snapshot 取决于 producer
  - `hook_agent` / dedicated compact summarize 这类专用路径则是旁路主线程 layering

因此当前更稳的本地闭环应改写为：

- `compat 文件 -> /init 读取 -> 写入 CLAUDE.md / claude.local.md -> sj() 扫描 -> userContext.ClaudeMd -> 运行时`
- 而不是：`compat 文件 -> CLI 本地主线程自动兼容注入 prompt`

因此更稳妥的结论应是：

- `CLAUDE.md / .claude/rules / claude.local.md / AutoMem / TeamMem` 的扫描已确认
- compat 文件**不会在当前已确认的 CLI 本地链路中被自动注入** `system` 或 `messages`
- 若 `remote-control / sdk-url` 对接的远端服务端在服务端侧再做 compat 拼装，本地 bundle 无法直接证明或反证

### 已确认的缓存、函数与状态

- `cachedClaudeMdContent`
- `systemPromptSectionCache`
- `additionalDirectoriesForClaudeMd`
- `sj()`：主 instructions/memory 文件收集器
- `xy()`：单文件处理与递归 include
- `v16()`：`.claude/rules/` 规则文件收集器
- `JI6()`：conditional rule 过滤层
- `HQ9() / aC1()`：文件读取、注释剥离、`@include` 解析
- `_$(...)`：把 `sj()` 扫描结果装配进 `userContext`
- `vO(...)`：systemContext 生成器
- `Lx8(...)`：把 `userContext` 前插为 `<system-reminder>` user meta message
- `dj4(...)`：把 `systemContext` 追加到 system prompt sections 末尾
- `$X(...)`：默认 system sections 生成器
- `_X(...)`：消息归一化与 attachment -> message 线性化入口
- `invokedSkills`
- `planSlugCache`

### 直接可得的结论

这说明 prompt/discovery 系统：

1. 有 section 级缓存
2. 支持从额外目录加载 `CLAUDE.md`
3. `.claude/rules/` 至少区分 unconditional 与 conditional 两类
4. Skills 可以被调用后纳入 prompt/context
5. 主扫描函数不是平铺读目录，而是带递归 include、去重、深度限制与路径排除的收集器
6. `sj()` 的结果在主线程里首先表现为 `userContext.ClaudeMd`，不是直接写进 `system`

### `systemPromptSectionCache`：现在可以写成真正的 section cache

这块以前只写到“有 section 级缓存”，现在可以再收紧。

当前本地 bundle 直接可见：

- `systemPromptSectionCache` 是全局 `AppState` 上的一份 `Map`
- 统一入口是：
  - `Lq8()`：取整张 cache map
  - `Vc8(name, value)`：按 section 名写入
  - `Ec8()`：整张清空
- `$X(...)` 不直接逐段裸算，而是先构造一组 section descriptor，再交给 `kHq(...)`

`kHq(...)` 的本地语义现在可以直接写成：

```text
for each section descriptor:
  if !cacheBreak && cache.has(name)
    -> return cached value
  else
    -> recompute via compute()
    -> cache.set(name, value)
```

因此这里不是“整段 system prompt 做一个字符串缓存”，而是：

- **按 section name 做独立缓存**
- 命中粒度是 section，不是整份 prompt
- 即使本轮是 cache-break section，算完后的最新值仍会回写到 map

### `$X(...)` 当前已直接可见的 section 名单

`$X(...)` 现在可直接确认会交给 `kHq(...)` 的 section 包括：

- `memory`
- `ant_model_override`
- `env_info_simple`
- `language`
- `output_style`
- `mcp_instructions`
- `scratchpad`
- `frc`
- `summarize_tool_results`
- `brief`

其中当前本地 bundle 里，只有一项明确被标成 `cacheBreak: true`：

- `mcp_instructions`

也就是说，当前更稳的说法应是：

- **绝大多数默认 system sections 都允许按 name 复用**
- **MCP instructions 被视为跨 turn 易变段，每轮强制重算**

### 各 section 的当前内容边界

这一项现在也不必继续只写“名单”，可以把当前本地 bundle 已直接可见的 section 语义再压实。

- `memory`
  - 由 `fT8()` 生成
  - 内容不是单一文件正文，而是 auto/team memory 的组合提示块
  - 会按：
    - team memory 启用与否
    - auto memory 是否启用
    - extract mode 是否启用
    - 额外 guidelines
    这些条件切换不同模板
- `ant_model_override`
  - 由 `h8_()` 生成
  - 当前本地直接返回 `null`
  - 因而现在更像保留 section 名，而不是活注入源
- `env_info_simple`
  - 由 `Tvq(model, additionalWorkingDirs)` 生成
  - 当前是环境摘要块：工作目录、git repo 状态、平台、OS、模型家族信息、fast mode 说明等
- `language`
  - 只在 settings 里存在 `language` 时生成
  - 内容是“始终用某语言回复”的固定说明块
- `output_style`
  - 由当前 output style prompt 生成
  - 若没有选中的 style，则为 `null`
- `mcp_instructions`
  - 由连接中 MCP server 的 instructions 拼成
  - 当前唯一明确的 `cacheBreak: true` section
- `scratchpad`
  - 由 `d8_()` 生成
  - 只在 scratchpad 功能可用时出现
  - 内容是“必须使用 session scratchpad 目录而不是 `/tmp`”的说明块
- `frc`
  - 由 `c8_(model)` 生成
  - 当前本地直接返回 `null`
- `summarize_tool_results`
  - 当前是固定提醒文本
  - 核心语义是：tool result 之后可能被清空，重要信息要尽快写回自己输出
- `brief`
  - 只有 brief entitlement 存在且运行态 brief mode 已启用时才非空
  - 内容来自 `BRIEF_PROACTIVE_SECTION`

#### 重点 section 变化条件表

| section | builder / selector | 何时非空 | 主要变化条件 | 额外说明 |
| --- | --- | --- | --- | --- |
| `memory` | `fT8()` | 只有 memory 功能链有可用来源时才非空 | `tengu_moth_copse` gate；team memory 是否启用；auto memory 是否启用；extract mode `hU6()`；`CLAUDE_COWORK_MEMORY_EXTRA_GUIDELINES` 是否存在 | team memory 开启时走 combined prompt；否则只走 auto memory；两者都不可用时返回 `null` |
| `output_style` | `yvq()` -> `S8_(style)` | 只有选中的 style 不是 `default/null` 时才非空 | plugin forced output style；`settings.outputStyle`；当前 cwd 下 output-style registry 变化 | `output_style` section 只负责 style prompt 本身；`keepCodingInstructions` 影响的是 `x8_()` 是否保留，不是这个 section 的非空条件 |
| `brief` | `i8_()` | 只有 `BRIEF_PROACTIVE_SECTION` 已装入且 `isBriefEnabled()` 为真时才非空 | brief entitlement 是否存在；运行态 brief mode toggle | 当前更像“运行态开关段”，不是固定 prompt 常量 |
| `language` | `R8_(settings.language)` | 只有 settings 里设置了 `language` 时才非空 | 用户设置变更 | 与 `brief/output_style` 不同，它不依赖插件 registry 或会话 mode |
| `mcp_instructions` | `C8_(mcpClients)` | 只有存在 `connected` 且带 `instructions` 的 MCP client 时才非空 | MCP client connect/disconnect；server instructions 变化 | 当前唯一明确 `cacheBreak: true` 的 section |

因此当前对 section cache 的更稳理解应改写为：

- **不是所有 section 都是“重内容动态块”**
- 其中有些其实是：
  - 恒为 `null` 的保留槽位
  - 固定文案提示
  - 受运行态开关控制的条件段

### `systemPromptSectionCache` 当前缓存什么，不缓存什么

把 section 名单与实现一起看后，当前本地可以直接分成三类：

1. **缓存且常常有实质内容**
   - `memory`
   - `env_info_simple`
   - `language`
   - `output_style`
   - `scratchpad`
   - `brief`
2. **缓存，但当前常量/空值化**
   - `ant_model_override`
   - `frc`
   - `summarize_tool_results`
3. **每轮强制重算**
   - `mcp_instructions`

这点很关键，因为它说明当前 section cache 的收益主要不在：

- `summarize_tool_results`
- `ant_model_override`
- `frc`

而主要在：

- memory prompt
- env / output style / language 这类稳定段
- scratchpad / brief 这类按会话状态变化、但不需要每轮重算的段

### `systemPromptSectionCache` 的失效边界

当前本地已直接看到的清理链有两类：

1. 显式清空 section cache
   - `Yn() -> Ec8()`
2. 与 instruction/user context 联动的上层失效
   - `Mi6(loadReason)` 会把 `sj.cache` 清掉
   - `Cn(querySource)` 在 compact 后会：
     - 主线程 / SDK 路径下清 `_$.cache`
     - 调 `Mi6("compact")`
     - 再调 `Yn()` 清 `systemPromptSectionCache`

另外，session reset / clear conversation 路径本地也直接能看到：

- `_$.cache.clear?.()`
- `vO.cache.clear?.()`
- `wb1.cache.clear?.()`
- `mjq(null)`
- `Cn()`
- `Mi6("session_start")`

因此当前更准确的边界是：

- `systemPromptSectionCache` 是**会话内 section 复用层**
- 但不是长期稳定缓存
- compact、clear/reset、session-start 这类边界都会把它打掉

另外还有一个容易误判的点，现在也可以一起钉死：

- `currentDate` 不在 `systemPromptSectionCache` 这层
- 它来自 `_$(...) -> userContext.currentDate`
- 日期切换由 `y6z()` / `lastEmittedDate` 这套 attachment 链单独处理

因此当前没有看到：

- “跨天自动清 `systemPromptSectionCache`” 的专门逻辑
- 也没有必要用它来解释日期变化

更稳的理解应是：

- **日期变化属于 `userContext` / attachment 层问题**
- **不是 default system sections cache 的失效触发器**

### `Mi6(...)` 不是立即触发 hook，而是为“下一次主扫描”预置 load reason

这一点现在也可以从 bundle 里写得更硬。

`Mi6(reason)` 当前本地只有三件事：

```text
sC1 = reason
tC1 = true
sj.cache.clear?.()
```

也就是说它不会直接执行 `Di6(...)`。  
真正消费这个 reason 的地方，在 `sj()` 主扫描尾部：

```text
let f = WQ9()
if (f !== undefined && Xi6()) {
  for (W of K) {
    if (!fQ9(W.type)) continue
    let G = W.parent ? "include" : f
    Di6(W.path, W.type, G, { globs: W.globs, parentFilePath: W.parent })
  }
}
```

这能直接推出三条关键时序结论：

1. `compact` 不是 compact 完成时立即发 `InstructionsLoaded`
   - 它只是先通过 `Mi6("compact")` 预置“下一次主扫描的默认 reason”
2. 真正发 hook，要等下一次普通 `sj()` 主扫描完整跑完
   - 收集完 `K` 之后才统一逐文件 `Di6(...)`
3. 即使默认 reason 已是 `compact`
   - 只要某个文件是通过 `@include` 进入
   - 它最后发出的 `load_reason` 仍会被覆写成 `include`

这里还要再补一个容易漏掉的细节：

- `WQ9()` 的调用发生在 `Xi6()` 判断之前
- 也就是当前 reason 槽位是**一次性消费**
- 即使当前根本没有 `InstructionsLoaded` hook，`WQ9()` 也会把它读走并重置回 `session_start`

因此当前本地更稳的说法应是：

- **`Mi6(...)` 是“下一次主扫描的 load-reason setter”**
- **不是 hook dispatcher**
- **`compact` / `session_start` 这类主 reason 都是 `sj()` 末尾一次性消费的**

### `_$()` / `vO()`：字段现在已直接钉死

现在已经可以直接从 bundle 写出两者定义，而不再只是调用点反推。

#### `_$()` 当前已直接确认

`_$()` 的实际返回形状现在可以写成：

```text
CLAUDE_CODE_DISABLE_CLAUDE_MDS || (simple-mode && additionalDirectoriesForClaudeMd is empty)
  -> ClaudeMd = null
else
  -> ClaudeMd = eC1(Pi6(await sj()))

return {
  ...(ClaudeMd ? { ClaudeMd } : {}),
  currentDate: `Today's date is ${eU6()}.`
}
```

因此当前已直接确认：

- `userContext` 固定至少有 `currentDate`
- `ClaudeMd` 是可选字段，不一定存在
- `cachedClaudeMdContent` 会在这里被更新

### `cachedClaudeMdContent` 不是扫描缓存，而是“渲染后快照”

这项以前容易被误读成：

- `sj()` 的文件级结果缓存
- 或 `CLAUDE.md` 原始内容缓存

现在可以收紧为：

- `_$()` 在得到 `K = eC1(Pi6(await sj()))` 之后
- 直接执行 `$c8(K || null)`
- 也就是把**最终渲染完成、准备用于 `userContext.ClaudeMd` 的整段字符串**写入全局状态

因此它缓存的不是：

- 文件数组
- section 列表
- 未拼装的原始 markdown

而是：

- **主线程最近一次生成出来的完整 CLAUDE.md 渲染文本**

当前本地直接可见的消费点也只有一条核心链：

- `HK_()` 读取 `Oc8()`
- 包成一段 auto-mode classifier 的 `<user_claude_md>` user message
- 并给这段 classifier 输入再打上 `cache_control`

因此更稳的结论是：

- `cachedClaudeMdContent` 更像**给 auto-mode / classifier 旁路复用的用户指令快照**
- 不是 `sj()` 的替代缓存层

#### `vO()` 当前已直接确认

`vO()` 的实际返回形状目前可以写得更精确：

```text
gitStatus =
  remote mode or git instructions disabled or unsupported repo state
    ? null
    : await wb1()

injection = null

return {
  ...(gitStatus ? { gitStatus } : {}),
  ...(injection ? { ??? } : {})
}
```

因此当前已直接确认：

- `systemContext` 当前本地 bundle 里唯一直接实锤的字段就是 `gitStatus`
- `vO()` 内部仍残留一个“第二动态槽位”
  - 局部变量形态是 `K = null`
  - 完成日志里仍会上报 `has_injection`
  - 但当前返回时对应 spread 是空对象
- 因此更严格的结论不是“设计上永远只有 `gitStatus`”，而是“**当前本地可执行路径只会产出 `gitStatus`；第二槽位目前没有活代码**”
- 就这一页最关心的 discovery 语义来说，还能再补一句负面证据：
  - 当前没有看到 compat 文件扫描结果、`sj()` 产物，或其他 instruction source 流向这个 `K`
  - 因此 `vO()` 的预留槽位目前**看不出与 compat discovery 有直接关系**
- `Explore / Plan` 在 `BN(...)` 里裁掉的也正是这个 `gitStatus`
- 本地 bundle 中 `currentDate / gitStatus / ClaudeMd` 的后续命中已经基本穷尽
  - `currentDate` 只看到 `_$()` 生成，没有看到后续专门裁剪
  - `gitStatus` 只看到 `vO()` 生成与 `BN(...)` 中 `Explore / Plan` 裁剪
  - `ClaudeMd` 只看到 `_$()` 生成与 `BN(...)` 中 `omitClaudeMd` 裁剪

#### `wb1()` / `gitStatus` 当前已直接确认

`wb1()` 返回的不是结构化对象，而是一段多行字符串快照。其固定骨架现在可以写成：

```text
This is the git status at the start of the conversation. Note that this status is a snapshot in time, and will not update during the conversation.
Current branch: <current branch>

Main branch (you will usually use this for PRs): <main branch>

Status:
<git status --short output or "(clean)">

Recent commits:
<git log --oneline -n 5 output>
```

因此 `gitStatus` 当前更精确的语义应改写为：

- 它是 **systemContext 里的单个字符串字段**
- 但这个字符串本身封装了 4 段 git 快照信息

目前已确认的 4 段来源为：

- `Current branch`
  - `vM() -> NkA() -> QGK()`
  - 读取当前 HEAD 所在分支；失败时回退成 `"HEAD"`
- `Main branch`
  - `EE() -> ykA() -> lGK()`
  - 优先级为：
    - `refs/remotes/origin/HEAD`
    - `origin/main`
    - `origin/master`
    - 最后回退 `"main"`
- `Status`
  - 直接执行 `git --no-optional-locks status --short`
- `Recent commits`
  - 直接执行 `git --no-optional-locks log --oneline -n 5`

额外还可直接确认：

- `Status` 文本超过 `40000` 字符会被截断
- 截断后会附加“如需更多信息请自己运行 `git status`”的提示
- 因此 `tb4(...)` 一类统计代码读取 `Y.gitStatus?.length` 时，拿到的是**字符串长度**，不是结构化对象大小

### `claudeMdExcludes`

设置中存在：

- `ClaudeMdExcludes: string[]`

语义明确：

- 对绝对路径或 glob 做排除
- 只影响 `User / Project / Local`
- 不影响 `Managed / policy`
- 匹配基于归一化后的绝对路径，并会补充 realpath 变体

### 更精确的作用点

`claudeMdExcludes` 是在 `xy()` 递归处理单个 memory 文件时生效的。  
因此它不只是“最终展示前过滤”，而是会直接阻止某些 `CLAUDE.md` / `CLAUDE.local.md` / `.claude/rules/*` 进入收集链。

---

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
