# 输入编译链

## 本页用途

- 用来说明用户输入如何在本地被编译成可进入主循环的消息、附件和控制选项。
- 用来把多模态 block、slash command、本地命令短路、`UserPromptSubmit` hook 合并点写成可复原的运行时说明。

## 相关文件

- [01-product-cli-and-modes.md](./01-product-cli-and-modes.md)
- [04-agent-loop-and-compaction.md](./04-agent-loop-and-compaction.md)
- [../02-execution/01-tools-hooks-and-permissions/02-hook-system.md](../02-execution/01-tools-hooks-and-permissions/02-hook-system.md)
- [../02-execution/05-attachments-and-context-modifiers.md](../02-execution/05-attachments-and-context-modifiers.md)
- [../05-appendix/01-glossary.md](../05-appendix/01-glossary.md)

## 总览

输入不是“原样塞进模型”，而是先经过一层本地编译：

```text
AU8(...)
  -> ihz(...)
     -> 多模态 block 归一化
     -> pasted image 归一化
     -> remote slash gate
     -> KE6(...) / P6z(...) 生成 attachment transcript entries
     -> bash / slash command / normal prompt 分流
     -> BU4(...)
     -> m5A(...) 追加图片尺寸类 meta message
  -> qe1(...) UserPromptSubmit
  -> 可能再注入 hook_additional_context / hook_success / blocking stop
  -> 返回给 CC(...)
```

这条链的直接结论：

- slash command 是本地解析和本地分流，不是交给模型猜。
- attachment 是本地 producer 生成的 transcript item，不是模型自己“发现”。
- 多模态输入的最终 `user.message.content` 不是纯字符串，而是 block 数组。
- `UserPromptSubmit` 不在输入最前面，而是在 `ihz(...)` 完成之后。

## `AU8(...)`：输入编译入口

### 入参形状

`AU8(...)` 直接接收当前轮最上层提交参数：

- `input`
- `preExpansionInput`
- `mode`
- `setToolJSX`
- `context`
- `pastedContents`
- `ideSelection`
- `messages`
- `setUserInputOnProcessing`
- `uuid`
- `isAlreadyProcessing`
- `querySource`
- `canUseTool`
- `skipSlashCommands`
- `bridgeOrigin`
- `isMeta`
- `skipAttachments`

其中最关键的几项是：

- `input`
  - 可能是字符串
  - 也可能是 block 数组
- `mode`
  - 决定是 `prompt`、`bash` 还是别的路径
- `pastedContents`
  - pasted image 单独走一条归一化链
- `skipSlashCommands`
  - 主要用于 remote-control 下的 slash command 限制
- `messages`
  - 传给 attachment producer，用于做 delta/reminder 类 attachment 判定

### 返回形状

当前更稳的写法不应只保留最小字段，而应写成“编译输出超集”：

```ts
interface CompiledInput {
  messages: TranscriptLikeMessage[]
  shouldQuery: boolean
  resultText?: string
  allowedTools?: string[]
  model?: string
  effort?: string
  nextInput?: string | ContentBlock[]
  submitNextInput?: boolean
}
```

原因是：

- 普通输入通常只返回 `messages + shouldQuery`
- slash command 路径还可能回填：
  - `allowedTools`
  - `model`
  - `effort`
  - `nextInput`
  - `submitNextInput`

### 顶层顺序

`AU8(...)` 的本地顺序已经可以直接写死：

```text
1. 若 mode=prompt 且 input 是普通字符串且不是 meta，先 setUserInputOnProcessing
2. 调 ihz(...)
3. 若 ihz 返回 shouldQuery=false，立即结束
4. 否则取文本 prompt（pU(A) || ""）
5. 执行 qe1(...): UserPromptSubmit hooks
6. 消费 hook 输出：
   - blockingError -> 直接停止
   - preventContinuation -> 写入停止消息并终止 query
   - additionalContexts -> 生成 hook_additional_context attachment
   - message -> 直接并入 messages
7. 返回给主循环
```

## `ihz(...)`：输入预处理器

`ihz(...)` 负责的不是单一“附件处理”，而是整段输入编译的主分流器。

### 第 1 段：把 `input` 归一成“前置 block + 主文本”

`ihz(...)` 先把输入拆成：

- `W`
  - 最终主文本字符串
- `G`
  - 主文本之前的原始 block
- `v`
  - 归一化后的完整输入
- `Z`
  - 额外的图片尺寸/来源说明文本

规则已经能直接确认：

1. 若 `input` 是字符串：
   - `W = input`
   - `G = []`
2. 若 `input` 是 block 数组：
   - 逐个遍历
   - `image` block 先走 `ai(...)` 做本地图像归一化
   - 成功拿到尺寸后，用 `Qv6(...)` 生成一条文本说明，塞进 `Z`
   - 图像 block 自身被替换成归一化后的 `u.block`
3. 最后一个 block 若是 `text`
   - 它会被视为主文本 `W`
   - 前面的 block 进入 `G`
4. 最后一个 block 若不是 `text`
   - `W = null`
   - 整个数组都留在 `G`

因此当前轮输入的真正语义是：

- “最后一个 text block” 更像主 prompt
- 它前面的 block 更像当前轮附带的多模态前置内容

### 第 2 段：pasted image 走独立归一化链

`pastedContents` 不和 `input` 里的 `image` block 复用同一套容器，而是单独处理：

1. 先从 `pastedContents` 里筛出图片项
2. 收集它们的 `id`，形成 `imagePasteIds`
3. 用 `mSq(...)` 取 source-path map
4. 每张 pasted image 都转成：
   - 一个真正的 `image` content block
   - 一条可选的尺寸/来源说明文本，继续塞进 `Z`
5. 最终这些 pasted image block 汇总到 `C`

这意味着 bundle 里已经明确区分了两类图像来源：

- 输入数组内直接携带的 `image` block
- 编辑器/剪贴板粘贴进来的 image

两者最后都会进入同一轮 `user` content，但 bookkeeping 不同：

- pasted image 会额外记录 `imagePasteIds`

### 第 3 段：remote-control 下的 slash command gate

`skipSlashCommands` 为真，且主文本 `W` 以 `/` 开头时，`ihz(...)` 会先做一次本地 gate：

1. 用 `JC8(W)` 解析 slash command
2. 用 `UU(commandName, commands)` 找命令定义
3. 若命令存在：
   - `sp8(command)` 为真：把 `x = false`
     - 这不是直接报错
     - 而是表示该 slash command 不能按普通本地 slash path 执行
   - `sp8(command)` 为假：
     - 直接短路返回
     - 结果是：
       - 一条当前输入的 user message
       - 一条 `<local-command-stdout>/${command} isn't available over Remote Control.</local-command-stdout>`
       - `shouldQuery = false`

因此这里不是“所有 remote slash command 都禁用”，而是：

- 一部分命令被本地彻底拦截
- 另一部分命令只是改变后续是否走 slash path

### `sp8(...)` 的当前实名命令族

`sp8(...)` 现在已经可以直接写成：

```ts
function sp8(command) {
  if (command.type === "local-jsx") return false
  if (command.type === "prompt") return true
  return AWz.has(command)
}
```

当前本地 bundle 里，`AWz` 命中的实名本地命令是：

- `compact`
- `clear`
- `cost`
- `release-notes`
- `files`
- `stub`
  - 隐藏且未启用

因此 remote-control 下的 slash gate 现在应收成：

- 所有 `prompt` command
  - 允许继续进入本地 slash 编译链
- 少量指定 `local` command
  - 也允许继续进入本地 slash 编译链
- 其余 `local` / `local-jsx` command
  - 直接被拦成 `isn't available over Remote Control`

这里还有一个产品层旁证：

- bridge system/init 会把 `commands:M.current.filter(sp8)` 发给远端

这说明 `sp8(...)` 不只是一个本地 if/else gate，它还是 remote-control 可见 slash universe 的筛选器。

### 第 4 段：attachment loading gate

attachment 不是永远加载，实际 gate 是：

```text
I = !isMeta
    && W !== null
    && (
         mode !== "prompt"
         || x
         || !W.startsWith("/")
       )
```

然后才会执行：

```text
PC8(KE6(W, toolUseContext, ideSelection, [], messages, querySource))
```

这几条非常关键：

- meta 输入不做 attachment loading
- 没有主文本时不做 attachment loading
- `prompt` 模式下，如果它是本地 slash command 且 `x === false`，也不会走普通 attachment loading

换句话说，普通用户 prompt 与 prompt slash command 的 attachment 生成时机并不相同。

## `ihz(...)` 的三条主分流

### 1. `mode === "bash"`

当 `W !== null && mode === "bash"` 时：

- 调 `processBashCommand(W, G, p, toolUseContext, setToolJSX)`
- 直接返回
- 不进入普通 `BU4(...)`

这里 `G` 会作为 `precedingInputBlocks` 带入 bash 输入包装，所以 bash 模式也支持前置多模态 block。

### 2. 本地 slash command

当 `W !== null && !x && W.startsWith("/")` 时：

- 调 `processSlashCommand(W, G, C, p, toolUseContext, setToolJSX, uuid, isAlreadyProcessing, canUseTool)`
- 返回 slash command 自己编译出的结果

这里四组输入各自职责很清楚：

- `G`
  - 原始输入数组里主文本前的 block
- `C`
  - pasted image block
- `p`
  - attachment transcript entries
- `W`
  - slash command 原始文本

### 3. 普通 prompt

其它情况最终都进入：

```text
m5A(BU4(v, C, imagePasteIds, p, uuid, permissionMode, isMeta), Z)
```

也就是：

1. 先由 `BU4(...)` 生成当前轮 user message
2. 再由 `m5A(...)` 把 `Z` 里的图片尺寸/来源说明包成额外一条 meta user message 追加进去

因此图片尺寸类说明不是塞进同一条 user content block，而是追加成后置 meta message。

## 输入 block family 的当前边界

### `ihz(...)` 活逻辑只直接特判 `text` / `image`

就输入编译本身，当前本地 bundle 能直接确认的活逻辑只有：

- `image`
  - 会被 `ai(...)` 归一化
- `text`
  - 只有最后一个 `text` block 会被提成主文本 `W`

也就是说，`ihz(...)` 本地真正依赖的规则仍是：

- “最后一个 `text` block = 主 prompt”
- 其它 block 只是前置内容

### 当前本地 producer 也主要只产出 `text` / `image`

目前能直接看到会把 block 数组喂进 `AU8(...)` / `ihz(...)` 的本地活 producer，主要也都落在这两类：

- 用户输入数组本身
  - 当前只看见 `text` / `image`
- pasted contents
  - 只会转成 `image`
- bridge inbound file attachments
  - 会先变成 `@"path"` 文本前缀，而不是 `document` block
- `queued_command`
  - 运行时还原成的也是 `text + image` 组合

### `document` 等 block 当前更像“下游兼容类型”，不是本地输入编译 producer

bundle 下游仍认识更多 content block：

- `document`
- `thinking`
- `tool_use`
- `tool_result`
- `server_tool_use`

但当前本地证据更稳地支持：

- 这些类型主要出现在 transcript / API payload / assistant runtime
- 没有直接看到本地输入侧 producer 把它们作为用户输入 block 喂给 `ihz(...)`

因此此页当前最稳的写法应是：

- **输入编译入口已正证的 block family：`text` / `image`**
- **`document` 等类型属于更大 message/content schema 的兼容面，不应直接写成当前本地输入编译的活 producer**

## `BU4(...)`：普通输入消息构造器

### 固定动作

`BU4(...)` 内部有几件已经能直接确认的动作：

1. 生成新的 promptId
2. 写入全局 promptId cache
3. 取首个 text block 做情绪/continue telemetry
4. 取最后一个 text block 做 `user_prompt` telemetry
5. 调 `Q8(...)` 生成本轮 user message

其中 telemetry 至少包含：

- `tengu_input_prompt`
  - `is_negative`
  - `is_keep_going`
- `user_prompt`
  - `prompt_length`
  - `prompt`
  - `prompt.id`

### 内容块组装规则

#### 有 pasted image 时

若 `q.length > 0`，`BU4(...)` 会把当前轮 user message 组装成：

```text
content = [
  ...当前文本或 block 数组,
  ...pasted image blocks
]
```

并额外设置：

- `imagePasteIds`
- `permissionMode`
- `isMeta`

注意这里的顺序是：

- 文本/已有 block 在前
- pasted image block 在后

#### 没有 pasted image 时

直接把 `input` 原样作为 `content` 传入 `Q8(...)`。

### 返回值结构

`BU4(...)` 返回的不是只有当前轮 user message，而是：

```text
{
  messages: [
    currentTurnUserMessage,
    ...attachmentMessages
  ],
  shouldQuery: true
}
```

这说明：

- 当前轮主 user message 与 attachment transcript item 仍是两类对象
- 它们在更后面的 prompt assembly 阶段才会进一步线性化/合并

## slash command 细分分支

### `processSlashCommand(...)` 的一级分流

#### 解析失败

若 `JC8(...)` 失败：

- 返回“Commands are in the form `/command [args]`”
- `shouldQuery = false`

#### 命令名不存在，但长得像 skill 名

若命令不存在，且 `i74(name)` 认为它是合法 slash token，且它又不是一个真实文件路径：

- 返回 `Unknown skill: ${name}`
- 如果有参数，还会额外给一条 warning system message
- `shouldQuery = false`

#### 命令名不存在，但不像合法 slash token

这种情况不会直接报错，而是退化成普通 user prompt：

- 直接构造一条 user message
- `shouldQuery = true`

因此并不是所有未知 `/xxx` 都当本地命令错误处理。

### `nx_(...)` 的二级分流

命令存在后，`nx_(...)` 会再按 command type 分流：

- `local-jsx`
  - 打开本地 UI 组件
  - 可选择直接 `skip`
- `local`
  - 只做本地命令执行
  - 返回 stdout/stderr
  - `shouldQuery = false`
- `prompt`
  - 若 `context === "fork"`：走 `lx_(...)`
  - 否则走 `r74(...)`

### prompt slash command 的编译结果

`r74(...)` 这条路径已经可以写成：

```text
messages = [
  1. 一个 user message，内容是 rx_(command, args)
  2. 一个 user meta message，内容是 command prompt block 数组
  3. prompt 文本触发出来的 attachment transcript items
  4. 一个 command_permissions attachment
]

shouldQuery = true
allowedTools = parsed command allowedTools
model = command.model
effort = command.effort
```

这里最重要的不是“slash command 也会 query”，而是：

- 它 query 的并不是原始 `/cmd args`
- 而是本地把 command prompt 展开后的 block 数组

## `UserPromptSubmit` hook 的精确位置

### hook 输入

`qe1(...)` 的 hook input 形状已经能写成：

```text
{
  ...hY(permissionMode),
  hook_event_name: "UserPromptSubmit",
  prompt: <plain text prompt>
}
```

其中 `hY(...)` 还会补：

- `session_id`
- `transcript_path`
- `cwd`
- `permission_mode`
- `agent_id?`
- `agent_type`

### hook 只在 `ihz(...)` 之后执行

`AU8(...)` 的顺序已经直接证明：

```text
ihz(...)
  -> shouldQuery?
  -> qe1(...)
```

所以 `UserPromptSubmit` 看见的已经是：

- 经过 slash command / bash / attachment 预处理之后的输入阶段
- 但还没真正进入模型调用

### hook 输出如何并入当前轮

`qe1(...)` 的产物在 `AU8(...)` 里按下面方式消费：

- `blockingError`
  - 直接终止
  - 生成 warning system message
  - 文本里会把 `Original prompt` 一并写出
- `preventContinuation`
  - 直接把一条普通 user message
    - `Operation stopped by hook`
    - 或 `Operation stopped by hook: ${stopReason}`
    塞进当前结果
  - 然后 `shouldQuery = false`
- `additionalContexts`
  - 生成一个 `hook_additional_context` attachment
  - `hookEvent = "UserPromptSubmit"`
  - `toolUseID = hook-${uuid}`
- `message`
  - 直接 push 到 `messages`
  - 若类型是 `hook_success`
    - 其 `content` 会先走 `cU4(...)` 截断

### 截断规则

`cU4(...)` 会把单条 hook 附加文本截到 `10000` 字符。

因此这里当前能写死的结论是：

- `UserPromptSubmit` 可以继续往当前轮注入上下文
- 但这个注入不是无限长原文直通

### matcher 在 `UserPromptSubmit` 上当前不起筛选作用

`et1(...)` 对不同 hook event 会先提一个 `matchQuery`。

但当前 switch 里：

- `PreToolUse / PostToolUse / PostToolUseFailure / PermissionRequest`
  - 用 `tool_name`
- `SessionStart`
  - 用 `source`
- `Setup / PreCompact / PostCompact`
  - 用 `trigger`
- `Notification`
  - 用 `notification_type`
- `InstructionsLoaded`
  - 用 `load_reason`
- `FileChanged`
  - 用 `basename(file_path)`

而 `UserPromptSubmit` 没有专门 case。  
结果是 `matchQuery` 留空，`et1(...)` 会直接采用该事件下的全部 hook matcher，而不是再做 `N8z(...)` 过滤。

因此当前本地实现里：

- `UserPromptSubmit` hook 的 `matcher`
  - **不会参与运行时筛选**
- 只要该事件下注册了 hook
  - 就都会进入候选集

### 多 hook 合并顺序是“完成顺序”，不是“配置顺序”

`FC(...)` 的关键结构是：

```text
matchedHooks = et1(...)
generators = matchedHooks.map(async function* (...) { ... })
for await (result of MC8(generators)) { ... }
```

而 `MC8(...)` 本身是：

- 同时拉起多路 async generator
- `Promise.race(...)`
- 谁先产出就先 `yield`

因此多条 `UserPromptSubmit` hook 并存时：

- 结果流入 `AU8(...)` 的顺序
  - 取决于各 hook 完成先后
- 不是 settings / plugin / session 里的静态声明顺序
- 也不是 `matcher` 列表顺序

### 去重规则只覆盖一部分 hook 类型

`et1(...)` 会对候选集做类型分桶后 dedupe：

- `command`
  - 按 `shell + command` 去重
- `prompt`
  - 按 `prompt` 去重
- `agent`
  - 按 `prompt` 去重
- `http`
  - 按 `url` 去重
- `callback`
  - 不 dedupe
- `function`
  - 不 dedupe

并且 dedupe key 还会拼上：

- `pluginRoot`
- 或 `skillRoot`

所以更精确的结论是：

- 同一 plugin / skill 内的重复 hook 更容易被折叠
- `callback` / `function` hook 则会按原候选数全部保留

### `AU8(...)` 只消费“先到达”的阻断结果

`AU8(...)` 对 `qe1(...)` 的消费是流式的：

- 一旦先看到 `blockingError`
  - 立即返回
- 一旦先看到 `preventContinuation`
  - 立即终止 query 并返回

因此在多 hook 并发时，当前轮真正生效的阻断结果应理解成：

- **先完成并先被 `AU8(...)` 读到的那一个**

这和“最后配置的 hook 覆盖前面”完全不是一回事。

## 当前最稳的还原结论

1. 输入编译不是一个小工具函数，而是一个本地状态机入口。
2. `AU8(...)` 的职责是“编译 + hook 合并”，不是“只做 message 包装”。
3. `ihz(...)` 才是真正的主分流器：
   - 多模态归一化
   - pasted image 归一化
   - attachment 生成
   - bash / slash / prompt 分流
4. 普通 prompt、bash、local slash command、prompt slash command，四条路径的输出形状并不相同。
5. `UserPromptSubmit` 的精确位置已经钉死在 `ihz(...)` 之后、query 之前。
6. `UserPromptSubmit` 多 hook 的当前轮合并顺序是完成顺序，不是配置顺序。
7. `UserPromptSubmit` 的 `matcher` 在当前本地实现里不参与筛选。

## 当前仍未完全钉死

- `Q8(...)` 的完整 user message helper 还有更多边缘字段，但对输入编译主逻辑已经不是阻塞项。
- `sp8(...)` 之外是否还存在 bundle 外或灰度下发的 remote 命令过滤层，当前本地不可见。
- 输入数组里虽然还不能 100% 排除 bundle 外 producer 喂入别的 block，但当前本地活 producer 已明显收敛在 `text` / `image`。

## 证据落点

- `cli.js`
  - `BU4(...)`
  - `AU8(...)`
  - `ihz(...)` 与 `m5A(...)`
  - bridge system/init 的 `commands.filter(sp8)`
  - `processSlashCommand(...) / nx_(...) / r74(...)`
  - `MC8(...)`
  - `N8z(...) / V8z(...) / J68(...) / et1(...)`
  - `qe1(...)`
  - `KE6(...)` / `Nq(...)`
  - `sp8(...)` 命中的实名命令对象
- [../02-execution/05-attachments-and-context-modifiers.md](../02-execution/05-attachments-and-context-modifiers.md)
  - attachment producer/consumer 细节

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
