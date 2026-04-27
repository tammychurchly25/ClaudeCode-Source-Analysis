# Exit 审批 UI、`planWasEdited` 与 ultraplan 回传

## 本页用途

- 单独承接 `ExitPlanMode` 的审批 UI 状态机，以及 `planWasEdited` 和 CCR/web ultraplan 回传链。
- 把本地 TUI、外部编辑器、远端 marker 回传和弱消费状态拆到一页里。

## 相关文件

- [../03-plan-system.md](../03-plan-system.md)
- [02-enter-exit-and-plan-command.md](./02-enter-exit-and-plan-command.md)
- [../07-tui-system.md](../07-tui-system.md)
- [../01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md](../01-resume-fork-sidechain-and-subagents/03-agent-team-mailbox-and-approval.md)

## 5. Exit 审批 UI 与编辑器状态机

`ExitPlanMode` 的 UI 组件是 `Cm4(...)`，它不是普通确认框，而是带 plan 读取、外部编辑、反馈和 mode 选择的专用状态机。

### 5.1 输入源

审批组件拿 plan 的顺序是：

```text
A.input.plan
-> AP()
-> "No plan found. Please write your plan to the plan file first."
```

也就是说：

- 如果调用侧已经把 plan 文本注入 tool input，UI 优先用注入值
- 否则直接回读磁盘 plan file

### 5.2 本地 TUI 里没有内嵌大编辑器，编辑依赖外部 editor

当前本地 bundle 可直接确认的编辑动作是 `Ctrl+G`：

- 如果当前就是 `ExitPlanMode`
  - `Ctrl+G` 对真实 plan file path 调 `GL(planPath)`
- 否则
  - 用 `ZV(currentText)` 把内容写到临时文件，再调外部 editor
- editor 关闭后重新读回文本
- 若内容变化：
  - 刷新当前 plan 文本
  - 显示 `Plan saved!`
  - 把“已修改”状态置真

因此至少在本地 TUI 可见范围内：

- 没看到一个复杂的内嵌 plan editor
- `ExitPlanMode` 面板里展示 plan 的组件是只读渲染，不是文本编辑器
- 真正的编辑桥是 `GL(...) / ZV(...)`
- plan approval 面板主要负责 **展示 + 外部编辑跳转 + 结果分流**

### 5.3 审批选项不是单一 Yes/No

`KVz(...)` 生成的选项会受这些状态影响：

- `showClearContextOnPlanAccept`
- auto mode 是否可用
- bypassPermissions 是否可用
- 是否开启 web/Ultraplan 相关路径

因此 “接受计划” 至少会分成：

- clear context + auto
- clear context + bypassPermissions
- clear context + acceptEdits
- keep context + auto
- keep context + bypassPermissions / acceptEdits
- keep context + default(manual approve edits)
- `No`
- 可选的 `ultraplan`

### 5.4 clear-context 与 keep-context 是两条不同后继链

#### clear context

UI 会把一条新的 `initialMessage` 写进 app state：

- message content 以 `Implement the following plan:` 开头
- 会带 `planContent`
- 会带 `clearContext: true`
- 会带目标 `mode`
- 会带原 `allowedPrompts`

后续 effect 真正执行时会：

- 清空当前会话消息和 readFileState
- 若存在 `planContent`，先取当前 slug，再在清上下文后用 `zu1(y8(), slug)` 还原 plan slug 绑定
- 用 `pN(toolPermissionContext, MF8(mode, allowedPrompts))` 重建 permission context

这说明 clear-context 不是“丢掉旧上下文后自由发挥”，而是把 plan 当作新的启动消息重新灌入主循环。

#### keep context

UI 不走 `initialMessage`，而是直接：

- `A.onAllow(input, MF8(mode, allowedPrompts), feedback)`

其中 `MF8(...)` 的真实行为需要分成“schema 设计”和“当前 bundle 实装”两层：

- 始终生成：
  - `setMode`
- 只有 `a16() === true` 时才会额外生成：
  - `addRules`
  - rule 形状是：
    - `toolName: "Bash"`
    - `ruleContent: "prompt: <semantic prompt>"`

但当前本地 bundle 里：

- `a16()` 直接返回 `false`
- `ukq(prompt)` 虽然会把 prompt 编成 `prompt: ...`
- Bash 侧语义匹配相关 `_N8(...) / mkq(...) / oT6(...)` 也都是 stub / 空实现
- `zN8(...)` 直接返回：
  - `matches: false`
  - `confidence: "high"`
  - `reason: "This feature is disabled"`
- `gkq(...)` 是空实现
- auto classifier prompt builder `JK_(...)` 里，对 semantic prompt rules 的总开关已经被内联成：
  - `let w = !1`
  - 随后 `oT6(A) / _N8(A)` 根本不会进入 classifier prompt 模板

因此更稳的结论不是“`allowedPrompts` 已完整生效，只是还没追完”，而是：

```text
schema / UI / rule shape 已存在
但当前本地发行版里 semantic prompt permission 不只是 runtime gate 未打开
而是已经在多条链路上被常量 false / stub 化
```

因此 keep-context 路径是：

```text
ExitPlanMode approval
-> immediate permission context mutation
-> 原对话继续运行
```

### 5.5 `No` 不是简单 reject

`No` 分支会继续留在 planning：

- 可携带文字反馈
- 可附带粘贴图片
- 最终调用 `A.onReject(feedback, imageBlocks?)`

因此它更接近：

```text
继续 planning，并把用户反馈回灌给模型
```

而不是“纯取消”。

## 6. `planWasEdited` 的更精确语义

这是当前这页最需要补硬的一点。

## 6.1 工具层的真实赋值条件

`ExitPlanMode.call(...)` 最终返回：

```ts
planWasEdited: z !== void 0 || void 0
```

其中：

- `z = "plan" in A && typeof A.plan === "string" ? A.plan : void 0`

也就是说本地工具层真正判断的不是“文件是否比之前不同”，而是：

```text
这次 ExitPlanMode input 里是否显式带了 plan 字段
```

## 6.2 本地 TUI 可直接确认的来源

在 `Cm4(...)` 里，提交给 `A.onAllow(...)` 的 input 是：

```ts
x && !K6 ? {} : { plan: Q }
```

其中：

- `x` 表示当前审批的是 `ExitPlanMode`
- `K6` 是本地“plan 内容被编辑过”的 dirty flag
- 当前可直接确认会把 `K6` 置真的路径是：
  - `Ctrl+G` 打开外部编辑器后，读回内容与旧值不同

因此在本地 TUI 可直接确认的结论应写成：

- **未编辑**
  - 不提交 `input.plan`
  - `planWasEdited` 缺省
- **经 `Ctrl+G` 外部编辑且内容变化**
  - 提交 `input.plan`
  - `planWasEdited: true`

## 6.3 关于 “CCR web UI or Ctrl+G”

`sdk-tools.d.ts` 与 output schema 描述里明确写着：

- `CCR web UI or Ctrl+G`

结合本地 bundle 里另外两组证据：

- 本地 `Cm4(...)` 只看到 `Ctrl+G` 外部编辑链，没有看到 web 内嵌编辑器
- 远端 `ultraplan`/CCR 轮询器会显式解析：
  - `## Approved Plan:`
  - `## Approved Plan (edited by user):`

因此更稳的表述是：

- `Ctrl+G` 是本地 bundle 可直接看到的已确认来源
- `CCR web UI` 是 schema 注释直接声明的来源
- 本地可见远端证据更像在说明：
  - cloud / web 侧确实存在“用户可改 plan 后再批准”的路径
  - 且它会把结果编码成 `Approved Plan (edited by user)` marker 回传
- 但本地 bundle 里没有把该 web 端组件树与 dirty-state 细节全部暴露出来

## 6.4 CCR / web edited-plan 的回传链已经可以写成明确链路

当前本地 bundle 已能把 `edited-plan` 从远端回到本地的桥接链写成：

```text
remote ExitPlanMode success
-> tool_result markdown:
   ## Approved Plan:
   or
   ## Approved Plan (edited by user):
-> ultraplan poller 扫 transcript
-> `B2z(...)` 从 marker 后截出 plan body
-> 本地 app state:
   ultraplanPendingChoice = { plan, sessionId, taskId }
```

更具体地说：

- `ExitPlanMode.mapToolResultToToolResultBlockParam(...)`
  - 在 main-thread / non-agent / 非空 plan 分支里，会把 plan 文本拼进 `tool_result`
  - 标题取决于 `planWasEdited`
- `wV4.ingest(...)`
  - 轮询远端 transcript
  - 追踪最近一次 `ExitPlanMode` 的 `tool_use_id`
  - 再去配对对应的 `tool_result`
- `B2z(...)`
  - 只认两种 marker：
    - `## Approved Plan:`
    - `## Approved Plan (edited by user):`
  - 找到后直接截取后面的 plan 文本
- `c2z(...)`
  - 拿到 plan 后把本地 task 置 `needsAttention`
  - 再把结果写入：
    - `ultraplanPendingChoice.plan`
    - `ultraplanPendingChoice.sessionId`
    - `ultraplanPendingChoice.taskId`

这说明当前可直接确认的远端回传语义不是：

```text
structured payload { plan, edited: true }
```

而是：

```text
markdown marker
-> 本地字符串解析
-> 归一化成 raw plan text
```

这也带来一个很关键的边界：

- “edited by user” 这个事实在远端 `tool_result` 层是可见的
- 但 `B2z(...)` 提取后，本地当前可见状态里只保留 plan 文本
- 也就是说在已暴露的本地回传链上，**编辑来源标记不会继续以结构化状态向后传递**

## 6.4.1 本地还有一条 `plan file -> tool_use.input.plan` 回填链

除了远端 `tool_result` marker 回传外，本地 bundle 里还能直接确认一条更靠近 transcript/transport 的补强路径：

```text
assistant message serialization
-> `S2z(...)`
-> 若命中 `ExitPlanMode`
-> 读 `AP()`
-> 把当前 plan file 回填进 `tool_use.input.plan`
```

更具体地说：

- `AAA(...)`
  - 在把本地消息转成结构化 assistant message 时，会先过 `S2z(...)`
- `S2z(...)`
  - 只改 `tool_use`
  - 只在 `tool_use.name === ExitPlanMode` 时生效
  - 若 `AP()` 能读到当前 plan file，则把它并进：
    - `tool_use.input.plan`

因此对于“`input.plan` 从哪里来”这个问题，现在至少可以再缩小一层：

- **本地 TUI 编辑链**：`Ctrl+G` dirty 后，`Cm4(...)` 会显式提交 `input.plan`
- **本地结构化消息/转运链**：`S2z(...)` 会按当前磁盘 plan file 回填 `ExitPlanMode.input.plan`
- **web/CCR 前端 dirty-state**：仍缺组件树与状态提升细节，但问题已经缩小到 web 端，而不是整个本地 runtime 都没有 `input.plan` producer

## 6.4.2 CCR / web 侧目前只暴露了 `plan/ultraplan` 模式合同，没有暴露 dirty payload

在“web dirty-state 怎么变成 `input.plan`”这个问题上，本地 bundle 现在还能再补一层明确边界：

```text
local launch `zAA(...)`
-> `FC8(...)` create remote session
-> seed event #1:
   `control_request.set_permission_mode`
   { mode: "plan", ultraplan: true }
-> seed event #2:
   initial user message
```

这条启动链当前可直接确认：

- `zAA(...)`
  - 本地选择 `ultraplan` 后，调用 `FC8(...)`
  - 传入：
    - `permissionMode: "plan"`
    - `ultraplan: true`
    - `initialMessage: d2z(blurb, seedPlan)`
- `FC8(...)`
  - 在真正的 user message 前，先塞一条 session seed event：
    - `control_request`
    - `request.subtype === "set_permission_mode"`
    - payload 只有：
      - `mode`
      - `ultraplan`
- 这说明本地启动 remote/web ultraplan 时，**明确可见的控制面合同只是“进入 plan mode，并把本 session 标成 ultraplan”**

同一条合同在 session state 同步侧还能再看到一次：

- `ja(...)`
  - 当 app state 的 mode 发生跨模式族变化时
  - 会向外发：
    - `permission_mode`
    - `is_ultraplan_mode`
- `yx4(...)`
  - 会把外部 session state 里的：
    - `permission_mode`
    - `is_ultraplan_mode`
  - 回灌进本地 app state
- 进入本地 REPL 后，真正处理 `set_permission_mode` 的地方是：
  - `nuz(...)` 只更新 `toolPermissionContext.mode`
  - 调用点另外把：
    - `isUltraplanMode : G6.ultraplan ?? x6.isUltraplanMode`
    - 独立写进 app state

因此这里可以得出一个比“还没追完”更硬的边界判断：

- 当前本地 bundle **已经暴露**：
  - `plan/ultraplan` 的 CCR session bootstrap 合同
  - `permission_mode / is_ultraplan_mode` 的 session-state 同步合同
- 当前本地 bundle **没有暴露**：
  - web editor subtree
  - plan diff / dirty flag / `planWasEdited`
  - 任何经 `control_request` 或 session state 明文传输的 plan 文本

也就是说，`web dirty-state -> ExitPlanMode.input.plan` 这段目前最稳的归属是：

- **发生在远端会话自身的 plan editor / approval UI 内部**
- **不是通过本地 REPL 可见的 CCR 控制面字段来完成状态提升**

## 6.4.3 `ultraplanPendingChoice` 与本地 UI 的关系：已看到弱消费，未看到专属确认器

现在可以把 `ultraplanPendingChoice` 与真正可见的本地 UI 消费拆成两层：

### 第一层：稳定可见的 UI 消费其实是 `task.needsAttention`

`c2z(...)` 在写入 `ultraplanPendingChoice` 之前，会先把对应 remote task 置成：

- `needsAttention: true`

这条状态随后会进入已经能直接命中的本地 UI 链：

```text
`c2z(...)`
-> remote task `needsAttention = true`
-> background task summary `jC8(...)`
-> 单个 remote ultraplan task 文案:
   `ultraplan ready`
-> footer/tasks 入口 `PF4(oLz)`
-> background tasks dialog `oB8(...)`
-> remote session detail `_V4(...)`
-> ultraplan detail `b2z(...)`
```

这里已经能写死的点是：

- `jC8(...)`
  - 当 background task 只有一个 `remote_agent`
  - 且 `isUltraplan === true`
  - 若 `needsAttention === true`
    - 文案从 `ultraplan` 变成 `ultraplan ready`
- `PF4(oLz)`
  - 负责 footer / background tasks 入口
  - 打开后会进入 `oB8(...)`
- `oB8(...)`
  - 把 `remote_agent` 作为 `Remote agents` 分组项列出
  - 选中后进入 `_V4(...)`
- `_V4(...)`
  - 若 `session.isUltraplan === true`
  - 不走普通 remote session detail，而是直接切到 `b2z(...)`
- `b2z(...)`
  - 是 ultraplan 专用 detail UI
  - 当前可见动作是：
    - 打开 web session
    - stop ultraplan
    - back

也就是说，**当前已确认的本地“最终可见提醒”是 task badge / task list / ultraplan detail 这一条链，而不是 `ultraplanPendingChoice` 本身直接渲染出的独立面板。**

### 第二层：`ultraplanPendingChoice` 自身目前只看到弱消费证据

当前本地 bundle 里，对 `ultraplanPendingChoice` 的直接读取证据非常有限：

- REPL 主体 `f3A(...)`
  - 通过 selector 读取 `state.ultraplanPendingChoice`
  - 但在当前可见代码里，唯一直接命中的副作用只是：
    - 若 `ultraplanPendingChoice` 已存在且 `showBashesDialog === true`
    - 则关闭 background tasks dialog

因此目前最稳的表述应改成：

- `ultraplanPendingChoice` **不是完全无 consumer**
- 但当前能直接命中的 direct consumer 还只有：
  - REPL 顶层的一个“关掉 bashes dialog”联动
- 尚未看到一个明确的：
  - `Approve ultraplan result`
  - `Use this approved plan`
  - `clear/keep context`
  - `mode choice`
  这种专属本地确认组件直接读取 `ultraplanPendingChoice`

换句话说，当前本地 bundle 已经足够说明：

- **用户可见提醒主链** 已落在 `needsAttention -> tasks UI`
- **`ultraplanPendingChoice` 自身的最终落地器** 仍未完全暴露

## 6.5 `planWasEdited` 影响什么

schema 注释已直接点明：

- 它决定 plan 是否在 `tool_result` 中被 echo back

因此它不是纯 telemetry 位，也不是纯 UI 提示位，而是影响 assistant/tool_result 展示内容的控制字段。

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
