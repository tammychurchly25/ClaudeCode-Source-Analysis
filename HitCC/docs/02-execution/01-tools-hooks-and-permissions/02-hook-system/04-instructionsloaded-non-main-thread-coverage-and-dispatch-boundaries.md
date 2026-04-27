# `InstructionsLoaded` 在非主线程的覆盖面与 dispatcher 边界

## 本页用途

- 单独承接 `InstructionsLoaded` 在非主线程路径里的覆盖面，以及 hook producer / dispatcher 的当前闭环边界。
- 把这部分与纯时序图拆开，避免继续长成混合页。

## 相关文件

- [../02-hook-system.md](../02-hook-system.md)
- [03-runtime-order-and-cross-stage-timing.md](./03-runtime-order-and-cross-stage-timing.md)
- [../../04-non-main-thread-prompt-paths.md](../../04-non-main-thread-prompt-paths.md)
- [../../04-non-main-thread-prompt-paths/01-shared-merge-skeleton-and-overrides.md](../../04-non-main-thread-prompt-paths/01-shared-merge-skeleton-and-overrides.md)
- [../../04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md](../../04-non-main-thread-prompt-paths/03-fork-family-cache-safe-params-and-snapshot-reuse.md)

### `InstructionsLoaded` 在非主线程路径里的覆盖面现在还能再收紧一层

之前把非主线程大致分成“默认会 / override 不一定 / 旁路不会”还不够。  
现在更稳的写法应该把 `BN(...)` 与 `lZ(...)` 分开，因为两者决定 `InstructionsLoaded` 的方式并不一样。

#### 1. `BN(...)` 默认构造：**通常会出现**

`BN(...)` 的默认形状里，当前是：

```text
userContext   = override.userContext   ?? _$()
systemContext = override.systemContext ?? vO()
```

而 `_$()` 的主路径会调用 `sj()`。  
因此只要没有显式 override `userContext`：

- 普通 subagent
- skill fork
- 大多数 built-in / custom agent 分支

当前仍会重新跑 `_$() -> sj()`，从而具备再次触发 `InstructionsLoaded` 的本地条件。

#### 2. `BN(...)` 显式 override `userContext`：**不保证出现**

如果调用方直接把现成的 `userContext` 塞给 `BN(...)`：

- 就不会再走默认的 `_$()`
- 也就不会因为这一步再次 fresh-run `sj()`

所以对 `BN(...)` 家族，更准确的说法是：

- **默认构造会**
- **override 复用父上下文时不一定会**

#### 3. `lZ(...)` 家族：要看 `cacheSafeParams` 从哪来

这一类以前挂得太笼统。  
`lZ(...)` 本体不会调用 `_$()` / `vO()`，它只会消费传进来的：

- `systemPrompt`
- `userContext`
- `systemContext`
- `forkContextMessages`

再交给 `CC(...)`。  
因此 `lZ(...)` 是否再次触发 `InstructionsLoaded`，取决于 **`cacheSafeParams` 的 producer**。

##### 3.1 复用现成 snapshot：**当前不会再次出现**

如果 `cacheSafeParams` 来自：

- `ML(ctx)`
- `xe6()`

那么 `lZ(...)` 本次只是复用已有 `userContext/systemContext`。  
当前本地直接看到的复用路径包括：

- `prompt_suggestion`
- `speculation`
- `extract_memories`
- `auto_dream`
- `session_memory`
- `agent_summary`
- 命中 `xe6()` 的 `/btw` side question
- 命中 `xe6()` 的 SDK `side_question`

这些路径当前都不应再写成“也许会触发”。更稳的本地判断是：

- **它们复用旧 snapshot 时，不会因为这一步再次触发 `InstructionsLoaded`**

##### 3.2 fresh-build `cacheSafeParams`：**会再次具备条件**

如果进入 `lZ(...)` 之前，调用方先自己重建：

- `userContext = _$()`
- `systemContext = vO()`

那么就又回到了 fresh-run `sj()` 的条件。  
当前本地已直接看到：

- `/btw` 的 `I1z(...)` fallback 分支
- `fD4(...)`

因此这一类应写成：

- **不是 `lZ(...)` 自己触发**
- **而是它前面的 fresh-build helper 再次触发了 `InstructionsLoaded` 条件**

##### 3.3 SDK side question 的空快照分支：**当前看不到 fallback**

SDK `side_question` 还有一个本地可见细节：

- 若 `xe6()` 为空
- 当前直接返回 `response: null`
- 没有看到像 `/btw` 那样再走 `_$() / vO()` 的 fallback

因此它不应再挂成“未知”，而更应写成：

- **SDK side question 命中 snapshot 时复用旧上下文**
- **未命中时当前直接放弃，不会补跑一遍 `sj()`**

#### 4. 明确旁路主扫描的路径：**当前看不到**

这两条本地路径现在仍然可以写得很硬：

- `hook_agent`
  - 直接把 `userContext:{}`、`systemContext:{}` 传进 `CC(...)`
  - 当前看不到 `_$() / sj()` 入口
- `compact` summarize 核心
  - 真实 summarize 调用直接走专用 prompt
  - 当前也看不到 fresh-run `sj()` 的本地活入口

因此这两类路径不应再挂成“也许会触发”，而更应写成：

- **当前本地可见实现里，`hook_agent` 与 compact summarize 核心都旁路了 `InstructionsLoaded` 主触发链**

### 本地 hook producer / dispatcher 当前已基本闭环

这个点现在也可以明显收紧，不必继续只写成“可能还有很多没追到”。

#### registry/source 层

`et1(...)` 当前合并的本地 hook 来源已能直接收成三类：

1. `dg()`
   - settings / managed hooks 主表
2. `wh()`
   - plugin hooks
3. `sessionHooks`
   - 当前 session / agent runtime 动态挂入的 hooks

其中：

- plugin hooks 由 `mF() / eq_()` 装入
- `allowManagedHooksOnly` 会在合并时直接改变是否并入 plugin / 非 managed hooks

#### dispatch 层

本地真正执行 hook 的 dispatcher 也只看到两套：

1. `FC(...)`
   - 负责大多数会进入 transcript / permission / continuation 语义的 hook
2. `UC(...)`
   - 负责 fire-and-forget / 简化返回语义的一组 hook

而所有 `execute*Hooks`：

- `qe1 / qs1 / E26 / Di6 / Ju1 / Mu1 / ...`

本质上都只是：

- 组装 `hookInput`
- 选择 `FC` 或 `UC`

没有再看到第三套本地 hook runtime / hidden dispatcher。

#### 事件集合也已闭环

当前本地 bundle 的 hook 事件集合由：

- `Mp` 运行时枚举
- `qp8(...)` 的元数据表
- `eq_(plugin)` 的装载表

共同闭环。  
没有看到第四个独立事件注册器，也没有看到某个额外本地模块偷偷扩展新 event family。

### 目前仍未完全钉死

- 少数事件的 `hookSpecificOutput` 是否在 bundle 外还有额外扩展分支
- `InstructionsLoaded` 多文件装载后的绝对顺序，是否还有 bundle 外特例

### 待验证点

- bundle 外或服务端侧是否还存在本地不可见的 hook 事件扩展
- bundle 外是否还存在会改变 `InstructionsLoaded` 绝对顺序的额外 wiring

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
