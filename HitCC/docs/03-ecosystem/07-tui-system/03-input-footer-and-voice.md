# 输入区、Footer 与 Voice

## 本页用途

- 把 `PF4` 从“输入框组件”纠正为 TUI 的交互中枢。
- 把输入、历史、自动补全、attachments、footer 焦点、voice dictation 放到一个统一视角里理解。

## 相关文件

- [../07-tui-system.md](../07-tui-system.md)
- [01-repl-root-and-render-pipeline.md](./01-repl-root-and-render-pipeline.md)
- [04-dialogs-and-approvals.md](./04-dialogs-and-approvals.md)
- [05-help-settings-model-theme-and-diff.md](./05-help-settings-model-theme-and-diff.md)
- [../../01-runtime/03-input-compilation.md](../../01-runtime/03-input-compilation.md)

## `PF4` / `oLz(...)` 不是薄输入框

`PF4` 接收的输入已经能看出它跨了很多子系统：

- `input` / `onInputChange`
- `mode` / `onModeChange`
- `stashedPrompt`
- `submitCount`
- `pastedContents`
- `vimMode`
- `showBashesDialog`
- `isSearchingHistory`
- `helpOpen`
- `getToolUseContext`
- `onSubmit`
- `onAgentSubmit`
- `voiceInterimRange`
- `toolPermissionContext`
- `commands`
- `agents`
- `messages`
- `mcpClients`

所以 `PF4` 更接近：

```text
prompt editor
+ autocomplete
+ history search
+ attachments bar
+ footer focus
+ mode/model/thinking toggles
+ stash/external editor
+ image paste
+ voice dictation overlay
```

## 输入模型：文本、cursor、attachments 是一起维护的

当前输入区至少同时维护：

- 文本 `M`
- cursor offset `e`
- `insertTextRef`
- `pastedContents`

`insertTextRef` 暴露了：

- `insert(...)`
- `setInputWithCursor(...)`

这说明外部模块可以把文本或中间态内容插回输入框，而不是只能整体替换输入字符串。

## 提交链：不是回车就直接送模型

`X_(...)` 这条 submit 路径会先做多层短路：

1. 若 footer 正在选择，直接不提交。
2. 若正在 selecting-agent，也不提交。
3. 若 speculation/prompt suggestion 命中，可能先接收建议。
4. 若是 teammate 直发消息，走 agent/teammate 分支。
5. 若当前为空且没有图片，直接返回。
6. 若 autocomplete 正在显示，且不是目录补全，会先阻止提交。
7. 最后才进入真正的 `onSubmit`。

因此输入区不是“被动文本框”，而是本地编译和多路分流的前台门面。

## `?` 会直接切帮助层

在输入变更逻辑里，`?` 有专门分支：

- 记录 `tengu_help_toggled`
- `setHelpOpen(...)`

这说明帮助层不是 slash command，而是输入层原生交互的一部分。

## 输入区内建的主要快捷动作

当前 Chat 上下文下至少能直接确认：

- `chat:undo`
- `chat:newline`
- `chat:externalEditor`
- `chat:stash`
- `chat:modelPicker`
- `chat:thinkingToggle`
- `chat:cycleMode`
- `chat:imagePaste`
- `chat:messageActions`

这证明输入层已经绑定了：

- 文本编辑器式能力
- 模式/模型切换
- 多模态附件
- 本地工作流快捷入口

## `chat:cycleMode` 改的是独立 permission mode 轴

输入层拿到的不是一个抽象“页面 mode”，而是显式的：

- `toolPermissionContext`

因此 `chat:cycleMode` 更稳的职责不是“切 screen”，而是切：

- `toolPermissionContext.mode`

当前已直接钉死的 mode 值包括：

- `default`
- `acceptEdits`
- `bypassPermissions`
- `plan`
- `auto`
- `dontAsk`

这一组切换会经由 permission context 写回与副作用链生效，而不是经由 `setScreen(...)`。

所以从输入层就能确认：

- `screen`
  - 负责 `prompt / transcript` 这类可见视图
- `permission mode`
  - 负责 ask/allow/plan/auto 这类执行权限策略

这两条轴是并列的，不应再混写成同一条顶层状态。

## stash 不是历史搜索

`chat:stash` 的行为是：

- 若当前输入为空但有 stashed prompt，就还原
- 否则把当前输入与 pasted contents 一起存入 stash
- 再清空输入框

所以它更接近临时草稿缓存，而不是历史记录。

## external editor 是正式输入通道

`chat:externalEditor` 会：

- 打开外部编辑器
- 读取返回内容
- 用编辑后的文本替换当前输入
- 若失败则在通知区提示

这说明“在外部编辑长 prompt”不是附加功能，而是正式编辑路径。

## attachments 条不是消息区的一部分

输入层单独维护 image/text pasted contents，且有独立上下文：

- `Attachments`

对应动作至少包括：

- `attachments:next`
- `attachments:previous`
- `attachments:remove`
- `attachments:exit`

这意味着 attachment bar 有自己独立的焦点模型，不等同于消息列表或输入框本体。

## footer 也是正式焦点域

footer 当前至少会承载这些入口：

- `tasks`
- `teams`
- `bridge`
- 以及预留/条件开启的其他项

并且存在独立上下文：

- `Footer`

对应动作包括：

- `footer:up/down/next/previous`
- `footer:openSelected`
- `footer:clearSelection`

这证明 footer 不是纯展示状态，而是可导航、可激活的底部导航条。

## voice dictation：不是独立面板，而是“锚定到输入框里的中间态文本”

voice 集成主要分两层：

### 第一层：`pCz(...)`

负责：

- 在开始录音时记住输入框左右锚点
- 把 `voiceInterimTranscript` 实时插回输入框中间
- 在最终 transcript 到来时，把中间态文本固化进输入
- 计算 `voiceInterimRange`

这里的关键点是：

- 中间态语音文本不是另开面板显示
- 而是直接嵌进输入框内容，并用一个 range 标记

### 第二层：`uc4(...)`

负责：

- 解析 `voice:pushToTalk` 绑定
- 处理 warmup
- 在按住/连击触发时去掉激活按键字符
- 进入录音、结束录音、取消 warming 状态

代码里还专门考虑了一个约束：

- 某些按键绑定会在 warmup 期间把字符打进输入框

因此 voice 并不是“额外调用一个录音器”那么简单，而是和键盘系统、输入缓冲、光标定位强耦合。

## 当前仍未完全钉死

- help overlay 的内部组件树还没单独展开。
- `ModelPicker / ThemePicker / Settings` 的 UI 细节尚未从 `PF4` 继续向下追。
- history search 自身的候选面板与执行态细节还可继续补，但它已经能确定是输入区原生子系统，不是 transcript 的一部分。

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
