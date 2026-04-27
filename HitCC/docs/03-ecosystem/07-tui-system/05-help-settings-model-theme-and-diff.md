# Help、Settings、Model/Theme 与 Diff

## 本页用途

- 把之前还停留在“是正式焦点域”的 `Help / Settings / ThemePicker / ModelPicker / DiffDialog` 继续拆到组件树和打开路径层。
- 固定这些 UI 大多不是独立 `screen`，而是 local JSX / 局部面板 / 子对话框。

## 相关文件

- [../07-tui-system.md](../07-tui-system.md)
- [03-input-footer-and-voice.md](./03-input-footer-and-voice.md)
- [04-dialogs-and-approvals.md](./04-dialogs-and-approvals.md)

## `Help`：不是 slash-only，而是输入层原生 overlay

当前已能把帮助层拆成两条明确入口：

- 输入区里输入 `?` 时，`PF4/oLz(...)` 直接 `setHelpOpen(...)`
- 也存在 local JSX 命令 `help`

真正的帮助组件是 `WW4(...)`。

它有这些稳定结构：

- 注册 `context: "Help"`
- `help:dismiss -> Esc`
- 顶层容器是带 tab 的 `iC`
- 默认 tab 是 `general`
- 另外两块稳定 tab：
  - `commands`
  - `custom-commands`

帮助层里不是只展示快捷键，还会：

- 把默认命令与自定义命令分开展示
- 给出版本号
- 给出文档主页链接

这说明帮助层不是“一段静态帮助文本”，而是一个正式的 tabbed dialog。

## `ThemePicker`：既可独立打开，也可作为 Settings 子面板复用

主题选择器本体是 `wL6(...)`。

已能直接确认：

- local JSX 命令 `theme` 会直接加载它
- settings 面板内，`X6 === "Theme"` 时也复用同一个组件

其内部状态和能力已经足够具体：

- 注册 `context: "ThemePicker"`
- `ctrl+t -> theme:toggleSyntaxHighlighting`
- 选项至少包括：
  - `dark`
  - `light`
  - `dark-daltonized`
  - `light-daltonized`
  - `dark-ansi`
  - `light-ansi`
- `onFocus` 会预览主题
- `onChange` 会保存预览并提交选中主题
- `onCancel` 会撤销 preview

更关键的是，它内建了一个 demo patch 预览：

- 用 `lU(...)` 渲染示例 diff
- 同时展示 syntax highlighting 开关状态

所以 ThemePicker 不只是一个 select list，而是“选项 + 预览 + syntax highlighting 开关”的组合面板。

## `ModelPicker`：是输入层局部面板，不是顶层 screen

模型选择器核心组件是 `x26(...)`。

当前已能确认三条打开路径：

- Chat 上下文快捷键 `chat:modelPicker`
- settings 里 `Model` 条目进入 `X6 === "Model"`
- settings 里 `teammateDefaultModel` 进入 `X6 === "TeammateModel"`

它的关键结构：

- 注册 `context: "ModelPicker"`
- `left/right` 调整 effort
- 既维护当前 model，也维护当前 effort
- 会根据模型能力判断：
  - 是否支持 effort
  - 是否支持 `max`
  - 是否需要把 `max` 降到 `high`

它不是只改当前会话模型，还会按场景决定是否写回：

- 主模型路径会更新 `mainLoopModel`
- 常规配置路径会写 `userSettings.effortLevel`
- teammate 默认模型路径会单独写 `teammateDefaultModel`

因此 `ModelPicker` 本质上是：

```text
model choice
+ effort choice
+ settings/session write-back
```

不是简单的单列表选择器。

## `Settings`：本体是 `config` local JSX，而不是顶层 `screen`

设置面板的 local JSX 命令名是：

- `config`
- alias: `settings`

命令最终打开 `jL6(...)`，默认 tab 为 `Config`。

其外层结构已能写实为：

- `Status`
- `Config`
- `Usage`

其中 `Config` tab 的主体是 `BD4(...)`。

## `BD4(...)`：可搜索的设置注册表，而不是硬编码表单

`BD4(...)` 目前已经能还原成一台小状态机：

- `X6`
  - 当前子对话框类型
- `x`
  - 是否进入 settings search
- `N / h`
  - 当前选中索引与滚动窗口

主列表不是静态文本，而是由一组 setting descriptor 动态生成。  
当前能直接确认的条目至少包括：

- `thinkingEnabled`
- `fastMode`
- `promptSuggestionEnabled`
- `defaultPermissionMode`
- `useAutoModeDuringPlan`
- `theme`
- `outputStyle`
- `language`
- `model`
- `teammateDefaultModel`
- `diffTool`
- `editorMode`
- `defaultView`
- `notifChannel`
- `autoUpdatesChannel`
- `showExternalIncludesDialog`
- 以及多项 IDE / remote / privacy / teammate 相关设置

同时它支持：

- `Settings` context 内上下移动
- `/` 进入搜索
- `left/right/tab` 对 enum 做循环切换
- `boolean` 直接原地 toggle
- 某些 managed 条目转入子对话框

所以 Settings 不是“一个 JSON 映射到表单”的扁平页面，而是：

```text
searchable settings registry
-> inline toggle / enum cycle
-> child dialog
-> localSettings/userSettings/appState write-back
```

## Settings 下游子对话框已经基本能枚举

`BD4(...)` 里当前可直接看到这些 `X6` 子状态：

- `Theme`
- `Model`
- `TeammateModel`
- `ExternalIncludes`
- `OutputStyle`
- `Language`
- `EnableAutoUpdates`
- `ChannelDowngrade`

对应组件分别是：

- `Theme` -> `wL6(...)`
- `Model` / `TeammateModel` -> `x26(...)`
- `ExternalIncludes` -> `J6A(...)`
- `OutputStyle` -> `bD4(...)`
- `Language` -> `xD4(...)`
- `EnableAutoUpdates` -> 内联确认框
- `ChannelDowngrade` -> `RD4(...)`

这说明 settings 本身更像一个“配置总控台”，而不是单一页面。

## `Status` tab：环境快照 + 诊断列表

`jL6(...)` 里的 `Status` tab 不是空壳，而是：

- `VD4(...)`
  - 主体组件
- `b7z(...)`
  - 同步拼装基础状态块
- `p7z(...)`
  - 异步诊断区

当前已能直接确认 `Status` 至少展示两层内容：

### 1. 同步状态块

`b7z(...)` 会把这些运行态信息组织成 label/value 列表：

- 当前主模型
- theme
- MCP 状态摘要
- 以及与当前 context 直接相关的若干环境值

这些值经过 `x7z/u7z/I7z(...)` 统一渲染，支持：

- 单值
- 数组值
- React 节点型值

### 2. System Diagnostics

`VD4(...)` 还会挂一个异步 `diagnosticsPromise`：

- `jL6(...)` 初始化时通过 `_qz() -> ND4().catch(zqz)` 生成
- `p7z(...)` 用 `Suspense` 消费
- 若返回非空数组，则以 `System Diagnostics` 标题展示 warning 列表

因此 `Status` 不只是“当前设置回显”，更像：

```text
runtime snapshot
+ async diagnostics
```

## `Usage` tab：订阅配额与额外额度视图

`Usage` tab 的主体是 `FD4(...)`。

它的行为已经比较清楚：

- mount 后立即 `aS8()` 拉 usage 数据
- 失败时进入 error 态
- 支持 `settings:retry`
- 成功后渲染多个 usage bucket

当前已能直接确认至少有三段标准配额：

- `Current session`
  - `five_hour`
- `Current week (all models)`
  - `seven_day`
- `Current week (Sonnet only)`
  - `seven_day_sonnet`

每个 bucket 都用 `gD4(...)` 渲染成：

- 标题
- 利用率条
- `used %`
- reset 时间

另外若存在 `extra_usage`，还会继续进：

- `Kqz(...)`

它至少区分三种状态：

- 未启用
- unlimited
- 按月额度计费

所以 `Usage` 不是简单跳到 `/usage`，而是 settings 内嵌的一套 usage dashboard。

## `DiffDialog`：是 local JSX 命令，不是聊天 transcript 的一部分

diff 对话框对应 local JSX：

- `diff`

描述文案已经写死为：

- 查看未提交改动
- 查看按 turn 切分的 diff

真正组件是 `$4z(...)`。

它的状态机至少有三层：

- source 维度
  - `current`
  - `turn`
- 内部模式
  - `list`
  - `detail`
- 当前文件索引

## `DiffDialog` 的数据源不是只有 working tree

`$4z(...)` 同时支持两类来源：

- 当前工作区的 `git diff HEAD`
- 从 transcript 里按 turn 反推出的结构化 patch

其中按 turn 的聚合器是：

- `Zf4(...)`

它会把 user/tool result 里的结构化 patch 聚成：

- 每个 turn 的文件集
- 增删行统计
- hunk map

因此 DiffDialog 不是一个纯 git wrapper，而是：

```text
working tree diff
or
per-turn reconstructed patch set
```

## `DiffDialog` 的交互已可稳定还原

它注册 `context: "DiffDialog"`，支持：

- `Esc` 关闭
- `left/right`
  - 切 source
- `up/down`
  - 在 list 模式切文件
- `Enter`
  - 进入 detail
- `left`
  - detail 回 list

视图结构也已足够明确：

- 顶部 source 条
- 变更统计
- list 模式下的文件列表 `vf4(...)`
- detail 模式下的单文件 diff `kf4(...)`

`kf4(...)` 还区分：

- untracked
- binary
- large file
- truncated diff

这已经足够支撑后续重写。

## 当前已钉死的结论

- `Help / ThemePicker / ModelPicker / DiffDialog` 都是正式焦点域，但不必对应顶层 `screen`。
- `Settings` 的真实入口是 `config/settings` local JSX，内部再切 tab 与子对话框。
- `Status` tab 是运行态快照 + 异步诊断，不只是静态说明页。
- `Usage` tab 是内嵌 usage dashboard，不是单纯外链或命令跳转。
- `ThemePicker` 和 `ModelPicker` 都被 settings 复用，而不是各自重复造 UI。
- `DiffDialog` 同时服务当前 working tree 与 transcript turn diff，两种来源共用一套列表/详情壳层。

## 仍未完全钉死

- 早期 `option` 命中字符串的原始来源还没回溯完，但当前已缺少它是主 screen 的正证。
- `Status` 里 `b7z(...)` 输出的每个具体字段来源，还能继续逐项点名。
- `Usage` 的远端数据结构 `aS8()` 返回 schema 还没完全拆平。
- `privacy-settings`、`chrome` 等设置类 local JSX 还没逐个展开。

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
