# Tool Result Renderer

## 本页用途

- 专门拆开 `user.tool_result -> Abq(...)` 这一支，不再把它混在 subtype 总表里。
- 固定 `tool_result` 从执行器、transcript、lookup 到最终 UI 的两层形态。
- 记录当前已经能钉死的几类高价值结果布局：`Read`、`Bash/PowerShell`、`Edit/Write`、`NotebookEdit`、`Grep/Search/Glob`、`WebFetch/WebSearch`。

## 相关文件

- [../07-tui-system.md](../07-tui-system.md)
- [07-message-row-and-subtype-renderers.md](./07-message-row-and-subtype-renderers.md)
- [../../02-execution/01-tools-hooks-and-permissions/01-tool-execution-core.md](../../02-execution/01-tools-hooks-and-permissions/01-tool-execution-core.md)
- [../../01-runtime/07-web-search-tool.md](../../01-runtime/07-web-search-tool.md)
- [../../01-runtime/08-web-fetch-tool.md](../../01-runtime/08-web-fetch-tool.md)

## 核心边界：UI 正常显示的不是 transcript 里的 `tool_result.content`

当前最重要的一条结论是：

- `tool_result` 在 transcript 里有一份回灌给模型的 block content
- 在 TUI 里还有一份并行保存的 `toolUseResult`
- `Abq(...)` 的正常结果分支吃的是后者，不是前者

可以还原成这条链：

```text
tool.call(...)
-> structured output (S.data)
-> mapToolResultToToolResultBlockParam(...)
-> H2q(...) 统一结果压缩/落盘
-> Q8({
     message.content = [tool_result block],
     toolUseResult = structured output or transformed output
   })
-> lookups.toolResultByToolUseID / toolUseByToolUseID
-> Abq(...)
-> tool.renderToolResultMessage(message.toolUseResult, progressMessages, ...)
```

这意味着：

1. transcript 里的 `tool_result.content` 更偏向“给模型继续消费/持久化”的文本或 block。
2. TUI 正常渲染更偏向“工具自己的结构化结果对象”。
3. 这两层不能混写成同一个概念。

## `Abq(...)` 的真实分支表

`Abq(...)` 先用 `tool_use_id` 去 `lookups.toolUseByToolUseID` 找回原始 tool 与 tool_use。找不到就直接返回 `null`。

之后是四类主分支。

### 1. 全局中断提示：`$$6`

如果 `tool_result.content` 以：

- `The user doesn't want to take this action right now...`

开头，直接走 `FCq(...)`，显示：

- `Interrupted · What should Claude do instead?`

这一支不再区分具体工具。

### 2. 工具拒绝提示：`O$6` 或 `UW`

如果 `tool_result.content`：

- 以 `The user doesn't want to proceed with this tool use...` 开头
- 或恰好等于 `"[Request interrupted by user for tool use]"`

就走 `nCq(...)`。

这一支会：

- 取回原始 `toolUse.input`
- 调 tool 自己的 `renderToolUseRejectedMessage(...)`
- 把 `progressMessagesForMessage`、`style`、`theme`、`isTranscriptMode` 一并传进去

如果工具没有自定义 rejected renderer，就退回通用 `AU(...)`，也就是：

- `Interrupted · What should Claude do instead?`

### 3. 错误态：`param.is_error === true`

错误态统一走 `cCq(...)`，但里面不是单一 fallback，而是继续拆成几类特殊字符串：

- `UW`
  - 直接显示通用 `Interrupted · What should Claude do instead?`
- `Lp1 + rejected plan`
  - 走 `Gy8(...)`
  - 显示 plan rejected 的专门框
- `vy8 + user feedback`
  - 走 `QCq(...)`
  - 表示用户拒绝了工具调用并给出后续说明
- `gb4`
  - 识别为 permission denial / auto-mode classifier denial
  - 显示 `Denied by auto mode classifier · /feedback if incorrect`
- 其他错误
  - 优先调用 tool 自己的 `renderToolUseErrorMessage(...)`
  - 再退回通用 `VO(...)`

所以 `tool_result.is_error` 不是“统一红字打印错误文本”，而是：

- 先识别几类运行时约定字符串
- 再交给工具自定义错误 renderer
- 最后才落回通用错误块

### 4. 正常结果：`aCq(...)`

只有到了这里，才进入真正的“结果展示”路径。

`aCq(...)` 的关键调用是：

```text
tool.renderToolResultMessage(
  message.toolUseResult,
  progressMessages,
  {
    input: original tool input,
    style,
    theme,
    tools,
    verbose,
    isTranscriptMode,
    isBriefOnly
  }
)
```

也就是说，正常结果 renderer 能同时拿到：

- 结构化结果
- 原始输入
- 进度消息
- transcript/brief/style/theme 上下文

这也是为什么 `tool_result` 在 UI 层能做很多与 transcript 文本无关的专门布局。

## lookup 不是装饰，而是 `tool_result` 渲染的必要前提

lookup builder 会维护至少这些索引：

- `toolUseByToolUseID`
- `toolResultByToolUseID`
- `progressMessagesByToolUseID`
- `resolvedToolUseIDs`
- `erroredToolUseIDs`

`Abq(...)`、`$bq(...)`、`W74(...)` 都依赖这组索引。

这说明消息区不是“顺序扫 message.content 即可渲染”，而是：

```text
normalized messages
-> build lookups
-> row renderer 回查 tool_use / tool_result / progress / hook 状态
-> tool-specific render
```

## 超长结果不是简单截断，而是统一落盘再回填 preview

`mapToolResultToToolResultBlockParam(...)` 之后还会统一经过：

- `iv6(...)`
- `H2q(...)`
- `lv8(...)`
- `lv6(...)`

这条链的行为已经能固定为：

- 空结果改写成 `(<tool> completed with no output)`
- 含图片的结果不走文本落盘
- 过大的文本结果写到 session 下的 `tool-results/`
- transcript 内只保留 `<persisted-output>...</persisted-output>` 包装的 preview 文本

后面 message budget 阶段还会再次把旧结果替换成 persisted preview。  
因此 tool result 至少有两层压缩：

1. 单次结果大小压缩
2. 历史消息预算压缩

## 当前已钉死的结果布局家族

### 1. `Read`

`Read.renderToolResultMessage = AM4(...)` 已经能稳定分成五类：

- `text`
  - `Read N lines`
- `image`
  - `Read image (<size>)`
- `pdf`
  - `Read PDF (<size>)`
- `parts`
  - `Read N pages (<size>)`
- `notebook`
  - `Read N cells`

这很重要，因为它说明 `Read` 的 TUI 结果不是把文件内容原样打出来。  
真正的正文内容主要还是通过：

- transcript block content
- 搜索抽取
- verbose 展开

来消费，而结果行本身更像结构化摘要。

### 2. `Bash` / `PowerShell`

`Bash.renderToolResultMessage = qq4(...)`，真实主体是 `kO6(...)`。  
它当前至少会处理：

- `stdout`
- `stderr`
- `isImage`
- `returnCodeInterpretation`
- `noOutputExpected`
- `backgroundTaskId`
- `timeoutMs`

已确认行为：

- 如果检测到 image data，只显示：
  - `[Image data detected and sent to Claude]`
- `stdout` 与 `stderr` 分开显示
- 会剥掉 `sandbox_violations` 包装
- 会单独抽出 `Shell cwd was reset to ...` warning
- 无输出时显示：
  - `Done`
  - `(No output)`
  - 或 `Running in the background`
- timeout 会挂一个单独提示块

所以 shell result 在 UI 层不是单一文本框，而是：

```text
stdout block
+ stderr block
+ cwd reset warning
+ background / no-output state
+ timeout hint
```

### 3. `Edit` / `Write`

这两类结果当前已经能确认是 patch/diff 主导，而不是简单成功提示。

`Edit.renderToolResultMessage = D34(...)`：

- 直接交给 `Eb8(...)`
- 使用 `structuredPatch + originalFile`
- plan 文件还会额外挂：
  - `/plan to preview`

`Write.renderToolResultMessage = u34(...)`：

- `create`
  - 新文件写入摘要
  - condensed 下只显示 `Wrote N lines to <file>`
  - 否则走 `rg_(...)` 展示写入预览
- `update`
  - 也交给 `Eb8(...)`
  - 说明 write update 与 edit 在 UI 层共享 patch preview 思路

这说明代码编辑类工具的结果 renderer，核心对象是：

- structured patch
- first line / preview
- 文件路径

不是 transcript 里的那句 “file updated successfully”。

### 4. `NotebookEdit`

`NotebookEdit.renderToolResultMessage = w94(...)` 当前很明确：

- 成功
  - `Updated cell <id>:`
  - 下方直接给 cell source code block
- 失败
  - 直接显示错误文本

而 rejected/error 也有单独 renderer：

- `z94(...)`
- `Y94(...)`

所以 notebook 路径并没有退回普通 file edit diff，而是保留了 notebook/cell 语义。

### 5. `Grep` / `Search` / `Glob`

这组工具的结果更接近：

- `count + content`
- `count + filenames`
- `files list`

其中：

- `Grep.renderToolResultMessage = i34(...)`
  - `content` 模式显示 `N lines`
  - `count` 模式显示 `N matches / M files`
  - `files_with_matches` 模式显示 `N files`
- `Glob`
  - 结果文本主要是文件名列表
- `Search`
  - 复用类似的 count/list 视觉策略

因此搜索类结果的 UI 核心不是 rich card，而是：

- 数量标签
- 内容或文件列表

### 6. `WebFetch` / `WebSearch`

这一组在 TUI 层反而很克制。

`WebFetch.renderToolResultMessage = BY4(...)`：

- 默认只显示：
  - `Received <size> (<code> <codeText>)`
- verbose 才把 `result` 正文带上

`WebSearch.renderToolResultMessage = Hw4(...)`：

- 只显示 search 次数
- 总耗时

这说明 server-side / network-like 工具的结果，在 TUI 里很多时候只给摘要，不把全文塞进行消息行。

## `tool_result` 还会反过来影响 transcript 搜索与默认可见性

这一块现在也能单独钉死。

### 截断结果会强制走 verbose 展示

`Aj6(...)` 会检查：

- 当前行是不是 user `tool_result`
- 是否非 error
- 是否有 `toolUseResult`
- 对应 tool 的 `isResultTruncated(toolUseResult)` 是否为真

如果为真，这一行会被当作需要展开/verbose 的特殊对象。  
因此“结果是否截断”不是纯 tool 内部逻辑，也会回流影响 transcript 可见性。

### 搜索文本优先用工具自定义抽取

transcript 搜索文本默认走 `tk4(...)`。  
但如果当前 user 行带有 `toolUseResult`，并且对应工具实现了：

- `extractSearchText(toolUseResult)`

就会覆盖默认文本提取。

这意味着：

- UI 搜索不一定扫 transcript 里的回灌文本
- 很多工具实际上是用结构化结果的关键信息参与搜索

## 当前已经能稳下来的结论

- `tool_result` 在 UI 层至少分成两层：transcript block content 与 `toolUseResult` sidecar。
- `Abq(...)` 正常结果分支并不直接渲染 `tool_result.content`，而是回查 tool metadata 与结构化结果。
- 错误态/拒绝态存在一组固定协议字符串，不应误写成“所有错误都走通用 renderer”。
- `Read`、`Bash`、`Edit/Write`、`NotebookEdit`、`WebFetch/WebSearch` 的结果布局已经足够稳定，可直接指导重写。
- `isResultTruncated(...)` 与 `extractSearchText(...)` 说明 tool result renderer 不只影响显示，也影响 transcript 默认展开与搜索语义。

## 仍未完全钉死

- 不是每个工具的 `renderToolResultMessage(...)` 都已逐项拆到同样深度；当前最值钱的大类已经闭环，但仍有少量工具只知道摘要轮廓。
- `aCq(...)` 里 `Allowed by auto mode classifier` 旁路提示所依赖的局部状态来源还不够完整，本地只看到 `OEq(toolUseId)` 的读取点。
- `toolUseResult` 在 remote / bridge / bundle 外路径上是否还会再被二次裁剪，目前只能确认本地可见链路。

## 证据落点

- `cli.js`
  - `lv8 / lv6 / iv6 / H2q / ll6`
  - `VO(...)`
  - `FCq / cCq / nCq`
  - `aCq(...)`
  - `Abq(...)`
  - `kO6(...)`
  - `qq4 / Kq4`
  - `X34 / D34 / f34 / W34`
  - `u34(...)`
  - `NotebookEdit` 的 `q94 / z94 / Y94 / w94`
  - `BY4(...)`
  - `Hw4(...)`
  - 执行器把 `toolUseResult` 与 `message.content` 同时写入 `Q8(...)`
  - headless / remote 输出侧单独携带 `tool_use_result`
  - `isResultTruncated(...)` 与 `extractSearchText(...)` 对 transcript 可见性/搜索的回流
  - `iCq(...)`
  - lookup 结构
  - `$$6 / O$6 / UW / vy8 / Lp1 / gb4`

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
