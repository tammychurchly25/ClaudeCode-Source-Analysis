# Transcript 与 Rewind

## 本页用途

- 把 transcript 视图自身的快捷键、搜索、导出和 show-all 行为单独写清。
- 把 `message-selector` 从“一个状态名”提升到可还原的 rewind / restore / summarize 工作流。

## 相关文件

- [../07-tui-system.md](../07-tui-system.md)
- [01-repl-root-and-render-pipeline.md](./01-repl-root-and-render-pipeline.md)
- [04-dialogs-and-approvals.md](./04-dialogs-and-approvals.md)
- [../../02-execution/04-non-main-thread-prompt-paths.md](../../02-execution/04-non-main-thread-prompt-paths.md)

## transcript 是专用视图，不是展开聊天历史

进入 transcript 视图的主要入口仍是全局快捷键：

- `app:toggleTranscript`

切入后：

- `screen` 变成 `transcript`
- `R5A(...)` 会启用 transcript 专属快捷键
- 主体直接渲染 `Aj6(..., screen: "transcript")`
- 底部接 `Pbz(...)` transcript 状态条

退出方式当前至少能稳定确认：

- 再次触发 `app:toggleTranscript`
- `transcript:exit`

`q / / / n / N / v` 这组 transcript 内快捷键还有一个重要前提：

- 代码里对应 handler 只会在 `B6 === "transcript" && rL` 时注册
- 而当前主 REPL 分支里 `rL` 被直接写成 `false`

因此对这份本地 bundle 来说，更稳的表述应是：

- 这组快捷键的**实现逻辑仍在**
- 但当前 transcript 主路径并没有把这组 handler 接活

## transcript 状态条：`Pbz(...)`

`Pbz(...)` 当前已经能确认至少展示三类状态：

- transcript 是否是 detailed transcript
- `showAllInTranscript` 是否打开
- 搜索状态或渲染状态

提示文案会按条件切换：

- 有 `searchBadge` 时：显示 `current / count`
- 有 `status` 字符串时：右侧优先显示状态文本
- virtual scroll 活跃时：显示上下/Home/End 提示
- 否则显示 `Ctrl+E` 的 show-all / collapse 提示

因此 `Pbz(...)` 不是只会显示固定帮助文案，它还承担：

- transcript 导出反馈
- 其他短时运行状态回显

## transcript 搜索不是普通字符串过滤

搜索接口层当前能直接写实为：

1. `/`：在 transcript 视图里启动搜索模式
2. `setQuery(...)`：把 query 写给终端 renderer
3. `scanElement(...)`：扫描已渲染节点树里的命中位置
4. `setPositions(...)`：把命中几何位置回灌给 renderer
5. `n / N`：在命中间跳转
6. `q`：退出搜索

这里的关键点是：

- 搜索不是对 message 数组做纯文本过滤
- 而是借助终端 renderer 的 `scanElementSubtree(...)` 在**渲染后的节点树**里取命中位置

因此它更接近“终端视图内查找”，不是 transcript 结构层面的简单匹配。

但当前还要补一层非常关键的实现边界：

- `Lx4()` 确实把 renderer 的 `setSearchHighlight / scanElementSubtree / setSearchPositions` 暴露给上层
- renderer 的 `scanElementSubtree(...)` 也确实会：
  - 取目标节点的 `computedWidth / computedHeight / left / top`
  - 离屏重绘该子树
  - 在重绘后的字符网格里找 query 命中
- 但 transcript 主路径里的 `/ / n / N / q` handler 当前受 `rL = false` 限制，没有真正激活

所以更精确的还原应当拆成两层：

- 搜索几何接口和 renderer 侧扫描逻辑是存在的
- 当前这份 bundle 的 transcript 主路径没有把这条交互链正式打开

## 列宽变化会重置 transcript 搜索

当前能直接看到：

- 当 terminal columns 改变时
- 如果 transcript 搜索或相关状态活跃

会执行：

- 关闭搜索栏
- 清空 query
- 清空命中计数
- `disarmSearch()`
- `setQuery("")`

这说明搜索命中位置强依赖当前终端几何，不能跨宽度变化直接复用。

## transcript 内导出：`v`

`v` 这条逻辑本身已经完整写在 transcript 分支里，会：

1. 把当前 transcript 用 `Ep8(...)` 渲染为纯文本
2. 写到系统临时目录下的 `cc-transcript-<timestamp>.txt`
3. 尝试调用外部打开逻辑
4. 在底部状态栏短暂显示结果

但它和搜索一样，也挂在：

- `B6 === "transcript" && rL && !searchBarOpen`

这意味着：

- 导出实现是存在的
- 当前 build 的 transcript 主路径里，这个快捷键同样没有被真正接活

所以本地 bundle 里稳定存在的导出路径更稳的是两层：

- dormant 的 transcript 内 `v`
- 可独立工作的 `Ep8(...) / OPz(...)` 文本导出链

## `message-selector`：rewind / restore / summarize 入口

`focusedInputDialog === "message-selector"` 时，会渲染 `r4A(...)`。

它不是简单选择一个 message，而是一个带还原策略的工作流。

### 候选消息不是所有 user message

`r4A(...)` 只挑选可 rewind 的 user message。  
会排除：

- `tool_result` user
- meta user
- compact summary
- visible-in-transcript-only
- 若干特殊 prompt tag

另外还会额外塞一个“当前空 prompt”占位。

### 进入确认后有四类动作

若目标消息关联到文件还原信息，则选项包括：

- `both`
- `conversation`
- `code`
- `summarize`
- `nevermind`

若没有代码可还原，则不会提供 `code/both`。

### code restore 与 conversation restore 是两条独立链

`r4A(...)` 会分开调用：

- `onRestoreCode`
- `onRestoreMessage`

两者可单独失败，因此 UI 里专门有：

- 只还原代码
- 只还原会话
- 两者一起还原

### summarize from here 不是普通还原

选择 `summarize` 后会走：

- `onSummarize`

主 REPL 里这条分支会进一步：

1. 找到选中消息在 transcript 中的边界位置。
2. 构造 toolUseContext / system prompt / userContext / systemContext。
3. 调用 `fVq(...)` 一类 compact summarize 核心。
4. 生成：
   - boundary marker
   - messagesToKeep
   - summaryMessages
   - attachments
   - hookResults
5. 用新数组替换当前消息尾部。
6. 重新生成 `conversationId`。
7. 必要时把被 rewind 的原始 prompt 文本重新回填输入框。

所以 `message-selector` 其实是：

```text
rewind picker
-> restore code / restore conversation / summarize from here
-> 可能改写 file history
-> 可能改写 transcript 边界
-> 可能生成新 conversationId
```

### 文件 diff 不是现算 git diff，而是从 transcript/fileHistory 重建

`r4A(...)` 会通过：

- `wt6(...)`
- `LC8(...)`
- `Mkz(...)`

推导出：

- `filesChanged`
- `insertions`
- `deletions`

并在 UI 里显示 restore 后果。  
这说明 rewind 的“代码变化提示”来自会话内部状态，不依赖当前 git 工作树实时计算。

## 当前仍未完全钉死

- transcript 搜索命中的坐标对象结构，目前仍只还原到 renderer 接口层。
- `rL` 为什么在当前 build 被硬钉成 `false`，还没追到更上游的 build-time 判断。
- `message-selector` 中 `onRestoreMessage` 更下游的 transcript fork 细节，仍需和 session/resume 文档继续对齐。
- transcript 导出有两条链：dormant 的视图内 `v` 与可独立调用的 `Ep8/OPz` 文本导出链；入口与交互不同。

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
