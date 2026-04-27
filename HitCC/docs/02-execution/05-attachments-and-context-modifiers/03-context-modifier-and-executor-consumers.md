# `contextModifier` 与执行器 Consumer

## 本页用途

- 单独梳理 `contextModifier` 的运行时接口、应用时机、并发提交规则，以及当前本地 bundle 的 concrete producer 边界。

## 运行时接口

执行器侧已经能把 `contextModifier` 收成：

```ts
interface ToolContextModifierEnvelope {
  toolUseID: string
  modifyContext(ctx: ToolUseContext): ToolUseContext
}
```

工具返回后，执行器会把它包在结果对象里往上送。

## consumer：串行路径

串行执行时，modifier 是即时应用的：

```text
Go_(...)
  -> for each tool result:
       if contextModifier:
         ctx = modifyContext(ctx)
```

也就是：

- 当前工具结束后
- 下一工具开始前
- 立刻更新 `ToolUseContext`

## consumer：并发路径

并发块里不会“谁先完成谁先提交”。

实际语义是：

1. 先按 `toolUseID` 收集 `contextModifier`
2. 等并发块全部完成
3. 再按原始 tool block 顺序依次应用

因此并发安全块的语义是“块结束后按声明顺序提交”，不是“按完成时间提交”。

## consumer：执行器实例内的非并发工具

执行器对象内部还维护一份 `contextModifiers` 数组：

- 工具完成后先把 modifier 暂存进去
- 若该工具不是 concurrency-safe，则立即更新本地 `toolUseContext`

这和外层批执行路径的规则一致，只是粒度更靠近 executor 内部。

## 当前本地 bundle 内唯一确认的 concrete producer：`SkillTool`

`SkillTool` 返回结果时，会直接构造 `contextModifier(...)`。

当前已正证它至少能改三类状态：

1. `toolPermissionContext.alwaysAllowRules.command`
2. `options.mainLoopModel`
3. `getAppState().effortValue`

因此它不是“只附加一段说明文字”的 skill helper，而是真的会改后续运行态。

## 本地 bundle 边界

重新扫完后，当前能收紧成：

- 本地 bundle 中，作为 tool return 值显式暴露 `contextModifier` 的 concrete producer，目前只看到 `SkillTool`
- 其它命中点都属于 generic consumer
- 没有看到 tool-returned `contextModifier` 直接去修改：
  - `readFileState`
  - `contentReplacementState`
  - `additionalDirectoriesForClaudeMd`
  - `active agent definitions`

因此当前更稳的结论不是“肯定只有一个 producer 永远存在”，而是：

- **在当前本地 bundle 可见代码中，只正证到 `SkillTool`**

## remote / bridge 的反证

remote / bridge 确实会影响本地上下文状态，但当前可见路径采用的是直接写缓存，而不是复用同一套 tool-returned `contextModifier` 协议。

已看到的代表性反证是：

- `seed_read_state`
  - 直接把 read state 写入缓存
  - 不是先返回一个 `contextModifier` 再由执行器应用

因此本地当前更稳的判断是：

- “远端能改本地上下文”成立
- “远端通过同一套 `contextModifier` 协议改上下文”当前没有直接证据

## 与 `ToolUseContext` 的关系

`contextModifier` 的真正价值，不是 UI 展示，而是把工具结果转成下一步运行态。

更准确地说：

```text
tool.call(...)
  -> may return contextModifier
  -> executor applies modifyContext(ctx)
  -> next tool / next side-path sees updated ToolUseContext
```

因此它和 [06-context-runtime-and-tool-use-context.md](../06-context-runtime-and-tool-use-context.md) 是一组上下游关系：

- `ToolUseContext` 是被改写的对象
- `contextModifier` 是改写协议

## 当前仍未完全钉死

- 远端/服务端侧是否还有 bundle 外 concrete producer，当前不能 100% 排除。
- 少量灰度/已裁剪工具若历史上支持 `contextModifier`，当前本地 bundle 无法正证。

## 证据落点

- `cli.js`
  - generic consumer
  - `SkillTool` concrete producer
  - `seed_read_state` 反证
- [06-context-runtime-and-tool-use-context.md](../06-context-runtime-and-tool-use-context.md)

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
