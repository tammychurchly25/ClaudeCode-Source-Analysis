# 对话框、审批与 Callout

## 本页用途

- 把 `focusedInputDialog` 从“弹层枚举”还原成精确优先级链。
- 把 tool permission、sandbox permission、prompt、elicitation、cost、idle-return 等分支各自的行为和副作用写清。

## 相关文件

- [../07-tui-system.md](../07-tui-system.md)
- [03-input-footer-and-voice.md](./03-input-footer-and-voice.md)
- [05-help-settings-model-theme-and-diff.md](./05-help-settings-model-theme-and-diff.md)
- [06-tool-permission-dispatch.md](./06-tool-permission-dispatch.md)
- [../../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md](../../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md)
- [../../02-execution/01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md](../../02-execution/01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md)

## `focusedInputDialog` 是单一优先级选择链

主 REPL 通过 `UA8()` 统一计算 `focusedInputDialog`。  
当前能直接写出的优先级顺序是：

1. `message-selector`
2. `sandbox-permission`
3. `tool-permission`
4. `prompt`
5. `worker-sandbox-permission`
6. `elicitation`
7. `cost`
8. `idle-return`
9. `ide-onboarding`
10. `effort-callout`
11. `remote-callout`
12. `lsp-recommendation`
13. `plugin-hint`
14. `desktop-upsell`

这条链很关键，因为它说明：

- 同一时刻底部并不是所有提示都能出现
- 而是先选出唯一焦点弹层，再决定渲染哪个分支

## `onCancel` 不是统一 close，而是按分支执行副作用

主 REPL 的 `IX()` 至少会按当前 dialog 做这些差异化处理：

- `tool-permission`
  - 调用 `onAbort()`
  - 清空 tool use confirm queue
- `prompt`
  - reject 当前 prompt 请求
  - abort 对应控制器
- 其他分支
  - 清流式状态
  - 可能把当前输入暂存回消息流

所以“Esc 取消”在 TUI 里不是纯 UI 事件，而是会真正影响执行态。

## `tool-permission`：`zB4(...)`

`tool-permission` 走 overlay 槽位，由 `zB4(...)` 承担。

它做的不是自己渲染一种固定 UI，而是：

1. 根据 `toolUseConfirm.tool` 调 `mVz(K.tool)`
2. 选择具体工具类型对应的 permission 组件
3. 注册 `app:interrupt`
4. 记录 permission telemetry

因此 `zB4(...)` 更像工具审批的总分发器，不是最终 UI 本体。

## `sandbox-permission`：`e5A(...)`

这是最明确的一类弹层之一。  
它面向的是：

- network access outside sandbox

可选项当前能直接写实为：

- `Yes`
- `Yes, and don't ask again for <host>`
- `No, and tell Claude what to do differently`

返回值是：

- `allow`
- `persistToSettings`

主 REPL 收到后若选择持久化，会把规则写进：

- `localSettings`

## `worker-sandbox-permission`

这条分支和普通 sandbox permission 类似，但有两个差异：

- 请求来自 worker / teammate
- 持久化时当前只对 allow 分支写规则

也就是说它不是复用普通 UI 文案那么简单，背后还有 team 协作语义。

## `prompt`：`XB4(...)`

这条分支对应的是工具或 MCP 需要用户在一组选项里做选择。

当前已确认：

- `request.options` 会被映射成选择项
- `app:interrupt` 会直接 abort
- UI 用单选 list 展示，不是自由输入

因此 `prompt` 更接近：

```text
question + fixed options
```

不是 `elicitation` 那种 schema form。

## `elicitation`：`MB4(...)`

`elicitation` 已经能明确拆成两种模式。

### 1. `url` 模式：`iVz(...)`

行为是：

- 告诉用户某个 MCP server 想打开 URL
- 可接受/拒绝
- 接受后进入 waiting 状态
- waiting 态下还能 reopen / continue / cancel

所以这不是单步确认框，而是一个带 waiting 子状态的两阶段 UI。

### 2. schema form 模式：`lVz(...)`

这条路径更重，已经是完整表单：

- 读取 `requestedSchema.properties`
- 生成字段列表
- 跟踪 required / validation errors
- 支持 enum / oneOf / array multi-select
- 支持 date/date-time 自动解析

其中 date/date-time 自动解析会调用：

- `UZ(...)`
- `querySource: "mcp_datetime_parse"`

也就是**本地再借助模型**把自然语言日期解析成 ISO 8601，再回填表单值。

这说明 elicitation 不只是 UI 渲染 schema，而是把：

- 表单状态
- 本地验证
- 日期自动解析
- abort/cancel

都放进了同一条用户输入闭环。

## `cost`：`Rx4(...)`

`cost` 当前是很轻的一次性确认框：

- 文案固定围绕 session 花费阈值
- 唯一动作是确认

但它的重要性在于：它是 `focusedInputDialog` 正式成员，说明“成本提醒”是 TUI 主状态机的一部分，而不是旁路通知。

## `idle-return`：`Cx4(...)`

这条分支对应用户长时间离开后再回来。  
当前选项至少包括：

- `continue`
- `clear`
- `never`

对应动作：

- `continue`
  - 继续当前会话
- `clear`
  - 调 `clearConversation(...)`
  - 作为新对话发送当前输入
- `never`
  - 写入本地持久设置，不再提示

因此 idle-return 不只是提醒“你离开太久了”，而是显式优化：

- token 开销
- 会话上下文长度

## 其他 callout / hint / onboarding 分支

当前 bundle 已可直接确认这些正式分支存在：

- `ide-onboarding`
- `effort-callout`
- `remote-callout`
- `lsp-recommendation`
- `plugin-hint`
- `desktop-upsell`

其中目前已经能写实的行为包括：

### `effort-callout`

- 提供 `low / medium / high`
- 会把选择写入 `userSettings.effortLevel`

### `lsp-recommendation`

- 会基于文件扩展名和本地二进制可用性推荐插件
- 推荐过或忽略计数过高后会停止继续提示

### `plugin-hint`

- 在命中相关命令时提示用户安装/启用插件

### `remote-callout`

这一块现在已经不能再只写一句“允许在 REPL 中开启 remote bridge 相关状态”。

当前本地可见行为至少包括：

- 只在这些条件同时满足时才出现：
  - `remoteDialogSeen` 还没写过
  - 当前是支持该体验的交互态
  - 本地已有 OAuth access token
- `/remote-control` 首次命中 preflight 通过后
  - 若满足上面的条件
  - 不会立刻 connect
  - 而是先把 `showRemoteCallout = true`
- callout 打开后会立刻把：
  - `remoteDialogSeen = true`
  - 写进持久状态
- 文案直接写明：
  - 当前 CLI session 可从 `claude.ai/code` 或 Claude app 访问
  - 随时可再用 `/remote-control` 断开
- 选项当前只有两类：
  - `Enable Remote Control for this session`
  - `Never mind`
- 若用户选 `enable`
  - 清掉 `showRemoteCallout`
  - 打开 `replBridgeEnabled`
  - 同时写 `replBridgeExplicit = true`
- 若用户选 `dismiss`
  - 只关闭 callout
  - 不自动 connect

因此 `remote-callout` 的定位更准确地说是：

- **第一次启用 Remote Control 时的产品教育 + 安全确认层**
- 而不是 bridge transport 本身的一部分

它在 `focusedInputDialog` 里的优先级也已经可确定：

- 低于 `ide-onboarding / effort-callout`
- 高于 `lsp-recommendation / plugin-hint / desktop-upsell`

## 当前仍未完全钉死

- `tool-permission` 下层具体工具类型 UI 组件还没逐个追平。
- `ide-onboarding / plugin-hint / desktop-upsell` 的内部文案和完整组件树还没有单独展开。
- `Help / Settings / ModelPicker / ThemePicker / DiffDialog` 虽然已经能确定是正式焦点域，但对应的实际弹层树还未在本组文档中落地。

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
