# Request-level 注入分层与本地/服务端边界

## 本页用途

- 单独承接 request-level 注入需要如何分层理解，以及本地 request build 与服务端黑箱之间的边界。
- 把 `prompt-text`、`schema/request options`、`transport` 三层拆开，并收口 `_I4 / AI4 / hN(...)` 之后本地还能证明什么。

## 相关文件

- [../03-prompt-assembly-and-context-layering.md](../03-prompt-assembly-and-context-layering.md)
- [../04-non-main-thread-prompt-paths.md](../04-non-main-thread-prompt-paths.md)
- [../../01-runtime/06-stream-processing-and-remote-transport.md](../../01-runtime/06-stream-processing-and-remote-transport.md)
- [../../04-rewrite/02-open-questions-and-judgment.md](../../04-rewrite/02-open-questions-and-judgment.md)

## request-level 注入与黑箱边界

### request-level 注入要分成 `prompt`、`schema/request options`、`transport` 三层

“request-level 注入”如果不分层，很容易把显式 request 字段和黑箱 context 混在一起。  
当前 `_I4(...)` 的 builder 已足够把这三层拆开：

```text
payload = {
  model: af(model),
  messages: LZz(k, ...),
  system: hZz(q, ...),
  tools,
  tool_choice,
  betas,
  metadata: n16(),
  max_tokens,
  thinking,
  temperature?,
  context_management?,
  output_config?,
  speed?
}
-> client.beta.messages.create(payload)
```

因此更稳的分层应是：

#### 1. prompt-text 注入

这才是本页真正讨论的 prompt layering 本体：

- `messages = LZz(_X(...), ...)`
- `system = hZz(WK(...), ...)`

其中 `_I4(...)` 还能直接确认两类“进入 system 文本块本体”的 request-level 补段：

- `lX8(...)`
- `cX8(...)`
- 以及若条件命中时，`...H ? [N2q] : []`、`...h ? [vu8] : []` 这类 **在 `WK(...)` 之前就显式拼入 `system sections` 的本地段**

也就是说：

- 本地确实存在 request build 阶段的附加 system section
- 但它们仍是 **`_I4(...)` 内显式可见的本地拼装**
- 不是“发送后又被某个本地黑箱二次追加”

#### 2. schema / request-options 注入

这些字段会改变一次请求的能力与行为，但**不属于 prompt 文本**：

- `tools`
- `tool_choice`
- `betas`
- `metadata`
- `max_tokens`
- `thinking`
- `temperature`
- `context_management`
- `output_config`
- `speed`

这里最容易误写的是两项：

- `advisor_20260301` 是通过 `tools` 注入的额外 server tool schema，不是 system 文本段
- `context_management` / `output_config` 是 request options，不是额外 `context` 文本

#### 3. transport / headers

再往外一层是 provider client 与 transport：

- `_y(...)` 负责 provider client、默认 headers、auth 与 request wrapper
- `sdk-url / remote-control / bridge` 负责 transport/ingress

它们会影响**请求如何被发送**，但从当前本地可见代码看：

- 不会在 `_I4(...)` 之后再改写 `messages/system`
- 也没有看到第二个本地 prompt builder 在 transport 层重新拼 context

### 本地 request build 与服务端黑箱的边界

围绕“是否还会额外追加 `context / compat / verification`”，这一页现在可以把本地边界写得更硬。

当前本地可见的 request-level payload producer 只剩：

- `_I4(...)`
- `AI4(...)`
- 少量 side query helper `hN(...)`

其中主链 `_I4(...)` 的最后落体已经是：

```text
payload = {
  messages: LZz(k, ...),
  system: hZz(q, ...),
  tools,
  ...
}
-> client.beta.messages.create(payload)
```

而 `AI4(...)` 复用的是同一套 builder callback，只把 `stream: true` 切成 `stream: false`，不是第二套 prompt assembly。

`hN(...)` 也能进一步提供一个旁证：它虽然是 side-query helper，但它同样是在本地显式构造：

```text
system = [
  lX8(...),
  skipSystemPromptPrefix ? [] : [cX8(...)],
  ...customSystem
]

payload = {
  model,
  system,
  messages,
  tools?,
  tool_choice?,
  output_config?,
  temperature?,
  stop_sequences?,
  thinking?,
  betas,
  metadata: n16()
}
-> client.beta.messages.create(...)
```

这说明即使是少量专用 side-query helper，本地也仍然是“先显式组 payload，再直接落到 `beta.messages.create(...)`”，没有再看到另一层隐藏本地注入。

因此本地 bundle 当前能直接确认：

- `_I4/AI4` 之后，没有看到第二次本地 request-level `system/messages` 注入
- `hN(...)` 这类 helper 也是本地显式拼 payload 后直接调用 provider，不是隐藏 prompt proxy
- `sdk-url / remote-control` 在本地可见代码里只换 transport / ingress，不改 `userContext / systemContext / system`
- 如果还存在额外的 `context / compat / verification` 拼装，更可能位于：
  - `client.beta.messages.create(...)` 之后的 provider / first-party 服务端黑箱
  - 而不是当前 CLI 本地链路

但这一页仍应保留一个克制边界：

- 本地代码只能证明 **发送前最后一个可见 payload producer 到这里为止**
- 不能反证服务端绝对不会再做：
  - server-side system augmentation
  - server tool wiring
  - verification / compat / policy prompt wrapping
- 因此“服务端是否还会追加黑箱 context”目前仍然属于 **本地 bundle 无法正证或反证的剩余未知点**

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
