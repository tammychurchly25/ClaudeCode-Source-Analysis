# Tool Permission 分派树

## 本页用途

- 把 `zB4(...)` 从“tool permission 总分发器”继续拆成可还原的工具类型到审批 UI 映射。
- 固定哪些审批 UI 是专用组件，哪些走通用 wrapper。

## 相关文件

- [../07-tui-system.md](../07-tui-system.md)
- [04-dialogs-and-approvals.md](./04-dialogs-and-approvals.md)
- [../../02-execution/01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md](../../02-execution/01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md)

## 总分发器：`zB4(...)`

`tool-permission` overlay 的核心仍是 `zB4(...)`。

它的固定步骤已经很清楚：

1. `BVz(...)` 生成顶部审批文案。
2. 注册 `app:interrupt`，中断时会 `onReject + onDone + queue reject`。
3. `mVz(toolUseConfirm.tool)` 选择具体审批组件。
4. 透传：
   - `toolUseConfirm`
   - `toolUseContext`
   - `onDone`
   - `onReject`
   - `workerBadge`
   - `setStickyFooter`

因此 `zB4(...)` 不拥有审批细节，真正的语义在 `mVz(...)`。

## `mVz(...)`：当前可直接还原的分派表

### 1. `eD -> pu4(...)`

这是单文件字符串替换编辑审批。

已确认：

- 解析 `old_string / new_string / replace_all`
- 标题是 `Edit file`
- 用 `eg8(...)` 预览 edit patch
- 走 `IQ` 这一类可 diff 的文件编辑审批壳

也就是典型：

```text
str_replace / edit
```

### 2. `Af -> Tm4(...)`

这是写文件或覆盖文件审批。

已确认：

- 解析 `file_path / content`
- 文件存在时标题是 `Overwrite file`
- 不存在时标题是 `Create file`
- 用 `Gm4(...)` 展示完整文件内容或 patch 预览
- 同样走 `IQ` 壳

### 3. `yq -> fm4(...)`

这是 Bash 命令审批。

已确认：

- 普通路径走 `xNz(...)`
- 若 `YE6(command)` 命中 sed/编辑型模式，则转 `du4(...)` 的专用分支
- 支持 destructive warning
- 支持把命令前缀保存成 allow rule
- 标题会区分：
  - `Bash command`
  - `Bash command (unsandboxed)`

这说明 Bash 审批不是单一确认框，而是：

- 普通命令 UI
- 编辑型 shell 命令 UI
- rule suggestion / feedback mode

三层叠加。

### 4. `eU -> KB4(...)`

这是 PowerShell 命令审批。

它与 Bash 很像，但有独立组件：

- 标题固定 `PowerShell command`
- 用 `tm4(...)` 生成选项
- 用 `am4(...)` 推断可保存的 PowerShell 命令前缀
- 有自己的 destructive command warning 规则集

因此 PowerShell 不是 Bash 文案替换，而是独立分支。

### 5. `_f -> Em4(...)`

这是 fetch 类 URL 访问审批。

当前已确认：

- 从输入里取 `url`
- 提取 hostname
- 标题是 `Fetch`
- 选项包括：
  - `Yes`
  - `Yes, and don't ask again for <domain>`
  - `No`

其“don't ask again”写入的是：

- `toolName + domain:<hostname>`

这说明它是面向 URL/domain 级规则的专用审批 UI。

### 6. `Ko -> Rm4(...)`

这是 notebook 编辑审批。

已确认：

- 解析 `notebook_path / cell_id / new_source / cell_type / edit_mode`
- 标题固定 `Edit notebook`
- 预览组件是 `Lm4(...)`
- `edit_mode` 至少包括：
  - `insert`
  - `delete`
  - `replace`

它仍走 `IQ` 壳，但内容预览已 notebook-aware。

### 7. `zf -> Cm4(...)`

这是 `ExitPlanMode` 审批，也就是“计划完成，是否开始执行”。

它不是普通工具审批，而是计划态的专用大面板。

已确认：

- 会展示 plan 内容
- 会展示 requested permissions
- 会根据上下文生成多种“同意后进入哪种 mode”的选项
- `No` 不是简单拒绝，而是“继续 planning 并带反馈”

它本质上是：

```text
plan approval / ready-to-code gate
```

### 8. `je6 -> bm4(...)`

这是进入 plan mode 的审批。

标题固定：

- `Enter plan mode?`

文案会明确说明：

- 先探索代码
- 识别现有模式
- 设计实现方案
- 在获得批准前不改代码

因此它是 plan mode 入口确认框。

### 9. `m76 -> xm4(...)`

这是 Skill 使用审批。

已确认：

- 从输入里解析 `skill`
- 标题格式是 `Use skill "<name>"?`
- 允许：
  - 精确允许某个 skill
  - 允许 `<prefix>:*`
- 使用通用 `HF8(...)` 选项组件，但语义是 skill allowlist

### 10. `Oy6 -> nm4(...)`

这是 `AskUserQuestion` / 多问题问卷审批与作答 UI。

它不是简单 prompt，而是完整问卷系统：

- 多问题分页
- 单选 / 多选 / `__other__`
- 文本输入
- option preview
- 图片粘贴
- review answers
- submit / cancel

在 plan mode 下还额外支持：

- `Respond to Claude`
- `Finish plan interview`

这是当前最重的一类 tool-specific 审批 UI。

### 11. `rU / ru / __ -> Nm4(...)`

这三类工具共用一个通用文件路径审批器。

已确认：

- `rU`
  - `FindFiles`
- `ru`
  - `Grep / Search`
- `__`
  - `Read`
- 若工具提供 `getPath()`，会抽出 path
- 用 `tool.userFacingName()` 生成可读名称
- 根据 `tool.isReadOnly()` 决定标题：
  - `Read file`
  - `Edit file`
- 继续走 `IQ` 壳

这说明 bundle 里至少有三种 path-aware 工具共享这套 UI。  
它们现在已经可以直接点名为：

```text
rU = FindFiles
ru = Grep/Search
__ = Read
```

### 12. `Xa1` 说明 `Nm4(...)` 不是“只服务这三种工具”

另外还能直接看到：

- `Xa1`
  - 即 LSP/代码智能工具
  - `checkPermissions()` 同样走 `n76(...)`
  - 也提供 `getPath({ filePath })`

但 `mVz(...)` 的静态 `switch` 里没有直接出现 `Xa1`。  
这说明更稳的判断是：

- `Nm4(...)` 确实是通用 path-aware 审批壳
- `rU / ru / __` 是当前静态表里已点名的三种
- `Xa1` 至少证明：`n76(...)` 这条 path-aware 权限核心并不只服务 `Read / Grep / FindFiles`

但这一段现在还能再收紧一层：

- 对当前整份 bundle 继续全局搜索后，`hVz / SVz / bVz / xVz / RVz / CVz / IVz / uVz` 只在 `mVz(...)` 与变量声明处出现
- 本地没有看到任何赋值站点
- 因而就当前 bundle 而言，**不存在把 `Xa1` 直接接到某个专用审批组件上的硬证**

因此当前不能再把“`Xa1` 很可能对应这四个槽位之一”写得太实。  
更稳的说法应是：

- `Xa1` 复用了 path-aware permission core
- 但它最终走哪套审批 UI，在本地可见 bundle 里没有闭环
- 若未来存在对应关系，更可能发生在：
  - bundle 外注入
  - build-time 裁剪后缺失的分支
  - 或保留别名/占位槽位

### 13. `hVz / SVz / bVz / xVz`

这组槽位现在可以再收紧一层，不该继续写成“本地未点名工具类型”：

- `hVz / SVz / bVz / xVz`
  - 是 `mVz(...)` `switch` 里的四个 tool-side case label
- `RVz / CVz / IVz / uVz`
  - 是与之成对的 component-side 变量
- 八个变量在同一处一起初始化为 `null`
- 全 bundle 搜索下，本地只看到：
  - `mVz(...)` 里的读取
  - 以及这组 `var ... = null` 声明
- 本地未见任何赋值站点

这里最关键的运行时边界是：

- `switch (A)` 比较的是对象身份
- 当前若 `hVz / SVz / bVz / xVz` 仍为 `null`，这些 `case` 标签在本地就是不可达分支
- 也就是说，当前本地 bundle 里，它们不只是“缺少专用 UI”，而是连对应 tool slot 都没有接线

因此更稳的表述应改成：

- 这不是“四种本地还没追平的活工具”
- 而是四对保留的 dispatch hook：
  - tool slot
  - component slot
- 当前本地 bundle 内，两侧都未接线

若未来存在真实语义，更像来自：

- bundle 外 wiring
- build-time 条件编译/裁剪
- 更完整发行物里的可选 feature slice

“延迟注入”仍然可以作为候选解释之一，但不能再写成当前本地运行时的既成事实。

## 两类通用审批壳

### `IQ(...)`

这类壳主要服务“有路径、可预览 diff、可套 IDE diff 支持”的工具：

- 文件编辑
- 写文件
- notebook 编辑
- 通用 path-aware 工具

特点：

- 标题 / 副标题 / 问句 / content 区块分离
- 能接 IDE diff support
- 能按 path / operationType 记录 completion telemetry

### `HF8(...)`

这是通用选项确认器。

当前明确服务于：

- skill 使用
- 以及若干非文件型工具确认

特点：

- option 可带 `feedbackConfig`
- 可进入 accept/reject feedback mode
- 可绑定 keybinding

## Bash / PowerShell 的“建议规则”不是附属信息，而是一级交互

`xNz(...)` 和 `KB4(...)` 都会从 `permissionResult.suggestions` 中生成：

- `Yes`
- `Yes, and don't ask again ...`
- `No`

并支持：

- 直接应用建议规则
- 手动修改 command prefix
- 在 accept/reject 时附带 instructions

这意味着命令审批 UI 已经把“本次放行”和“未来规则写回”合到同一层。

## `H46(...)`：保留槽位的最终兜底不是空白，而是完整通用审批器

继续把 `hVz / SVz / bVz / xVz` 这组黑箱往下追，当前还能再钉死一个更精确的边界：

- 只有在外部 wiring 先给 `hVz / SVz / bVz / xVz` 之一赋入真实 tool 对象时，对应 case 才可能被命中
- 此时若 `RVz / CVz / IVz / uVz` 没有同步接上专用组件，才会 fallback 到 `H46(...)`
- 对当前本地 bundle 本身来说，由于 tool-side slot 也是 `null`，这四个 case 事实上是 dormant dispatch hook，不是活 fallback 分支

`H46(...)` 本身不是空壳，它至少会做这些事：

- 用 `tool.userFacingName(input)` 生成标题
- 打 `tool_use_single` telemetry
- 展示 `tool.renderToolUseMessage(...)`
- 展示 `permissionResult`
- 通过 `HF8(...)` 提供标准选项

当前能直接确认的标准选项至少包括：

- `Yes`
- `Yes, and don't ask again ...`
- `No`

其中“don't ask again”分支会写入：

- `addRules`
- `behavior: "allow"`
- `destination: "localSettings"`
- 规则主体只带 `toolName`

拒绝分支则会走：

- `toolUseConfirm.onReject(...)`

因此对当前本地 bundle 来说，即便 `RVz / CVz / IVz / uVz` 永远没有外部赋值，审批主链也不会失效。  
更稳的运行时语义是：

```text
当前本地 bundle
-> 四对 slot 默认全是 null
-> 这些 case 本地不可达
-> 真正未命中的工具统一走 default -> H46(...)

若未来有外部 wiring:
-> 命中保留 tool slot
-> 取不到专用组件
-> fallback 到 H46(...)
```

所以这组黑箱更像：

- 专用审批体验增强槽位

而不是：

- 审批流程能否运行的必要依赖

## 当前已钉死的结论

- `tool-permission` 不是单一弹窗，而是一组专用审批组件的分派树。
- 文件编辑、文件写入、notebook 编辑、通用 path 工具共用 `IQ` 壳。
- Bash 与 PowerShell 是两条独立命令审批链，都支持 destructive warning 与 allow rule 写回。
- `ExitPlanMode`、`EnterPlanMode`、`AskUserQuestion`、`Skill` 都有各自专用 UI，不适合被简化成普通 `confirm(prompt)`。
- `rU / ru / __` 现在已经能点名为 `FindFiles / Grep / Read`。

## 仍未完全钉死

- 当前 ask-capable built-in 余集已经明显收缩：
  - `WebSearch`
    - `checkPermissions() -> passthrough + allow rule suggestion`
    - 但不在 `mVz(...)` 静态分发表中
  - generic `MCPTool`
    - 静态 `mcp` singleton 与动态 MCP tool wrapper 都是 `checkPermissions() -> passthrough`
    - 同样不在 `mVz(...)` 静态分发表中
  - `Xa1`
    - `isLsp`
    - `checkPermissions() -> n76(...)`
    - 有独立 `renderToolUseMessage`
    - 但也不在 `mVz(...)` 静态分发表中
- 因而若要对 `hVz / SVz / bVz / xVz` 给出“当前最佳候选”，更稳的排序是：
  - `WebSearch`
  - `MCP`
  - `LSP / code-intel`
  - 以及一个当前发行版里已整片裁掉、无法再从 surviving singleton 直接点名的额外 slice
- `ListMcpResourcesTool` 与 `ReadMcpResourceTool` 当前都能直接看到 `checkPermissions() -> allow`。
  - 这说明 MCP 的 resource/read 支线至少不是当前这 4 对 hook 的 surviving ask-family。
  - 因而若第 4 个 slice 最终仍落在 MCP 大类内部，也更像 generic MCP tool、channel/外接审批，或更完整发行物里的可选实现，而不是 resource/read/polling 这一支。
- `Xa1(LSP)` 与 `hVz / SVz / bVz / xVz` 之间在当前本地 bundle 内没有直接对应证据；若要继续追，更像是在追 bundle 外或 build-time 被裁剪掉的实现。
- `hVz / SVz / bVz / xVz` 在原始实现里分别对应什么 tool family，当前仍无法仅靠本地 bundle 定性。
- `RVz / CVz / IVz / uVz` 这四个专用组件变量在本地 bundle 内未见任何赋值；若未来还要继续追，重点已经不是 JSX 活路径，而是 bundle 外注入、构建期常量折叠，或更完整发行物里的 feature slice。
- `H46(...)` 本身已经会渲染：
  - `tool.userFacingName(input)`
  - `tool.renderToolUseMessage(...)`
  - `permissionResult`
  - `HF8(...)` 的标准选项/反馈壳
  因而像 `WebSearch`、generic `MCP` 这类 surviving family，即便没有专用组件，也不会在本地审批链上缺掉核心交互。
  这进一步说明这 4 对 hook 更像 optional enhancement slot，而不是当前本地审批主链的必需拼图。

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
