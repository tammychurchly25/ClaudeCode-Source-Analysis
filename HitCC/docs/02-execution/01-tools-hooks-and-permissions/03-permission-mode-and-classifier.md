# Permission Mode、状态机与 Auto Classifier

## 本页用途

- 单独整理本地 permission core、mode 状态机与 auto classifier 的运行边界。
- 把 `D0z / YP`、`za / Hy6 / oy6 / LqA / Qs6` 这条状态链从 sandbox / backend 合流部分拆开。

## 相关文件

- [../01-tools-hooks-and-permissions.md](../01-tools-hooks-and-permissions.md)
- [../04-non-main-thread-prompt-paths.md](../04-non-main-thread-prompt-paths.md)
- [../05-attachments-and-context-modifiers.md](../05-attachments-and-context-modifiers.md)
- [../06-context-runtime-and-tool-use-context.md](../06-context-runtime-and-tool-use-context.md)
- [../../03-ecosystem/03-plan-system.md](../../03-ecosystem/03-plan-system.md)
- [../../05-appendix/01-glossary.md](../../05-appendix/01-glossary.md)

## 权限系统与工具许可决策

### permission mode

可确认至少有：

- `default`
- `plan`
- `acceptEdits`
- `dontAsk`
- `bypassPermissions`
- `auto`

### 语义提醒

`dontAsk` 不是“全部允许”，而更接近“未预授权的直接拒绝，不弹问询”。

### 规则来源

权限决策至少综合以下来源：

1. 静态 allow/deny rules
2. skill/contextModifier 写入的 alwaysAllowRules
3. PreToolUse hook 决策
4. `canUseTool()` transport/UI/SDK 决策

### 工具级规则形态

至少支持：

- `Bash(git:*)`
- `Edit(docs/**)`
- `WebFetch(domain:github.com)`
- `mcp__server__tool`

说明规则是“工具名 + 参数子集/路径/域名”层面的细粒度权限，不是简单 tool name 列表。

### 本地 permission core 不是一个函数，而是两层

当前更稳的本地骨架应拆成：

```text
D0z(...)
  -> 静态 deny 规则
  -> 静态 ask 规则
  -> tool.checkPermissions(...)
  -> bypass/rule allow shortcut
  -> ask fallback

YP(...)
  -> 接 D0z 结果
  -> dontAsk 拒绝 ask
  -> auto / plan(auto-active) classifier
  -> 无法弹 prompt 的异步场景降级
  -> 返回 allow / ask / deny 给外层 prompt backend
```

因此“权限系统”不只是 `canUseTool()` 的一个布尔判断，而是：

- 先做本地规则和工具级 `checkPermissions()`
- 再叠加 mode 语义
- 再在需要时进入 auto classifier
- 最后才交给具体 prompt backend 或 transport

### permission mode 不是枚举，而是带副作用的状态机

当前本地 bundle 中，真正执行 mode 切换的是 `za(oldMode, newMode, ctx)`。

它不是简单改 `ctx.mode`，还会同步处理：

- `prePlanMode`
- auto-mode active 标记
- dangerous allow rules 的剥离与还原
- 一部分 UI / telemetry 侧副作用

#### 初始 mode 选择

启动时不是只看 `--permission-mode`。

`LqA(...)` 当前至少会综合：

1. `--dangerously-skip-permissions`
2. `--permission-mode`
3. settings 里的 `permissions.defaultMode`

然后按顺序挑第一个当前可用的候选。

几个明确约束：

- `bypassPermissions` 若被 Statsig gate 或 settings 禁用，会跳过并附带 notification
- `auto` 若 gate 不可用，会直接降回 `default`
- remote 场景下，settings 的 `defaultMode` 只接受 `acceptEdits / plan / default`

#### 已确认的 mode 语义

- `default`
  - 普通 ask/allow/deny 流程
- `acceptEdits`
  - 比 `default` 更宽，但仍不是“全放行”
  - auto mode 会先尝试判断“若切到 acceptEdits 是否可放行”
- `dontAsk`
  - 遇到 ask 直接 deny
  - 不进入交互审批
- `bypassPermissions`
  - 本地 permission core 直接 allow
  - 但前提是 session 启动时显式允许该模式，且没有被组织策略禁用
- `auto`
  - ask 不一定进入人工审批，而是先过 classifier
- `plan`
  - 不是独立于 auto 的平行态
  - 它会和 `prePlanMode`、hidden auto-active 状态一起组成复合状态

#### 进入 / 离开 auto 的真实副作用

`za(...)` 在 `default/acceptEdits/dontAsk/... <-> auto` 切换时会做两件关键事：

1. `yG.setAutoModeActive(true/false)`
2. `Xm(...) / _a(...)`

其中：

- 进入 `auto`
  - 要求 `SV() === true`
  - 会剥离危险的 broad allow 规则，避免 classifier 被“预授权”绕过
- 离开 `auto`
  - 会还原之前剥离掉的规则

这说明 auto mode 不是“在 ask 前多跑一个分类器”这么简单，而是会主动改写当前 permission context。

### dangerous allow rules 在 auto mode 下会被主动剥离

`Xm(...)` 的目标不是一般 allow rule，而是那些会直接绕过 classifier 的危险预授权。

当前已能直接确认至少覆盖：

- 过宽的 `Bash(...)` allow
- 过宽的 `PowerShell(...)` allow
- task 类危险权限

并且它不是只看一个来源，而会扫描 `alwaysAllowRules` 中多个 destination。

当前更稳的还原应写成：

- 进入 auto 时，把这类危险 allow 从当前 context 临时拿掉
- 记录到 `strippedDangerousRules`
- 退出 auto 时，再由 `_a(...)` 还原

这也是为什么 auto mode 与普通 allow/deny rules 不是简单并列关系。

### plan mode 与 auto mode 是耦合状态机，不是两个互不相干的开关

`plan` 的关键不是 `mode==="plan"` 这一个字段，而是：

- `mode`
- `prePlanMode`
- `isAutoModeActive()`
- `strippedDangerousRules`

#### 进入 plan：`Hy6(...)`

从普通模式切入 `plan` 时，`za(...)` 会直接走 `Hy6(ctx)`，而不是只改 mode。

`Hy6(...)` 的当前逻辑可以收敛成：

- 若当前就是 `plan`：原样返回
- 若当前是 `auto`
  - 总会记录 `prePlanMode: "auto"`
  - 若 plan 仍允许沿用 auto，则保留危险规则已剥离态
  - 若 plan 不再允许 auto，则关闭 auto active，并还原已剥离规则
- 若当前不是 `auto`
  - 若 `shouldPlanUseAutoMode()` 成立，进入“plan + hidden auto-active”
  - 同时把当前模式记到 `prePlanMode`
- 若不满足上述条件
  - 进入普通 `plan`
  - `prePlanMode` 记录进入前模式

因此更准确的状态图应理解为：

```text
default / acceptEdits / dontAsk / bypassPermissions
  -> enter plan
     -> plan(prePlanMode=oldMode)
     -> 若 shouldPlanUseAutoMode()，再叠加 hidden auto-active
```

#### plan 内部还会继续同步 auto gate：`oy6(...)`

即使已经处于 `plan`，本地仍会通过 `oy6(...)` 持续修正：

- 当前是否应该保持 hidden auto-active
- 当前是否应还原/剥离 dangerous rules

这说明 plan 不是进入时一次性决策，运行中也会随 auto gate 可用性变化而重算。

#### `shouldPlanUseAutoMode()` 的真实条件

`IqA()` 当前等价于：

```text
hasAutoModeOptInAnySource()
&& isAutoModeGateEnabled()
&& useAutoModeDuringPlan !== false
```

其中：

- opt-in 不是抽象概念，而是 `skipAutoPermissionPrompt` 持久化接受态或 CLI flag
- `useAutoModeDuringPlan` 可被 user/local/flag/policy 任一来源显式关掉

#### 离开 plan 的还原语义

从 `plan` 退出时，`za(...)` 会：

- 关闭 hidden auto-active，若有
- 还原剥离的 dangerous rules
- 清掉 `prePlanMode`

而真正“退出 plan 后回到哪个 mode”，则由 `ExitPlanMode` 路径决定。  
这一点与 plan file / leader approval 的结合，见 [../../03-ecosystem/03-plan-system.md](../../03-ecosystem/03-plan-system.md)。

### auto mode gate 不是单一布尔值，而是多路可用性合成

`SV()` 当前至少同时受三类条件约束：

1. circuit breaker 没有关闭
2. settings 没有显式 `disableAutoMode`
3. 当前模型支持 auto mode

而 `Qs6(...)` 还会再把远端配置 `tengu_auto_mode_config` 合进来，区分：

- `enabled`
- `opt-in`
- `disabled`

并进一步计算：

- 当前模型是否支持
- fast-mode breaker 是否命中
- 当前 session 是否真的可以进入 auto
- classifier UI/manager 是否应认为“available”

#### kick-out 不是提示，而是真正改 context

当 `auto` 或 `plan + hidden auto-active` 在运行中变得不可用时，`Qs6(...)` 返回的 `updateContext(...)` 会直接：

- 把 `mode:"auto"` 踢回 `default`
- 或把 `prePlanMode:"auto"` 改成 `default`
- 同时还原之前剥离的 dangerous rules
- 把 `isAutoModeAvailable` 置 `false`

因此 auto 不可用不是“灰掉一个按钮”，而是真正的运行时状态回退。

### auto mode classifier 不是黑盒：输入、默认规则和 fast-path 都已可写实

#### 当前 action 如何送进 classifier

本地不会把“待执行动作”当普通字符串描述，而是先构造成一个 synthetic assistant `tool_use`：

```text
Kyq(toolName, input)
  -> assistant message with tool_use block
```

随后再与已有 transcript 一起送入 classifier。

#### classifier 看到的 transcript 不是原始 transcript

`nEq(...) / aEq(...) / oEq(...) / vV8(...)` 当前至少会抽取：

- user 文本
- assistant `tool_use`
- queued command attachment
- 当前 action 的 `tool_use`
- 可选的 `tool_result`
- 用户 `CLAUDE.md`

其中一个很关键的当前事实是：

- 默认 `includeToolResults = false`

也就是说 classifier 默认并不会把已存在的 tool output 全量并入判断上下文。

#### 默认规则来源

`GV8()` 直接从内嵌 classifier prompt 模板中解析三段默认列表：

- `allow`
- `soft_deny`
- `environment`

`lEq()` 则返回展开后的完整 classifier system prompt。

#### 自定义规则不是 append，而是“按 section 替换默认段”

`_p6()` 会跨这些来源聚合：

- `userSettings`
- `localSettings`
- `flagSettings`
- `policySettings`

然后把各来源收集到的：

- `autoMode.allow`
- `autoMode.soft_deny`
- `autoMode.environment`

拼成新的 section，替换默认模板里的对应段落。

因此这里更稳的结论不是“用户规则追加到 defaults 后面”，而是：

```text
同类 custom rules 先跨来源合并
-> 再整体替换默认 section
```

#### 一个很容易漏掉的细节：schema 里有 `deny`，但当前聚合链不消费

`autoMode` 的 schema 允许：

- `allow`
- `soft_deny`
- `deny`
- `environment`

但当前 `_p6()` 实际只收集：

- `allow`
- `soft_deny`
- `environment`

没有看到 `deny` 被继续注入 classifier prompt 的本地路径。  
因此文档里不能把 `deny` 当成已确认生效的 classifier section。

#### 进入 classifier 之前的 fast-path

auto mode 并不是所有 ask 都一律送 classifier。

当前至少有这些短路：

- tool 声明 `toAutoClassifierInput(...) === ""`
  - 直接 allow
- 若把同一个动作放到 `acceptEdits` 下会直接 allow
  - 直接 allow
- 命中本地 safe allowlist 的工具
  - 直接 allow
- `requiresUserInteraction()` 的工具
  - 不走 classifier，保留人工审批
- `PowerShell`
  - 显式跳过 classifier，要求人工审批
- `safetyCheck` 类型 ask
  - 保留原 ask，不直接交给 classifier

因此 classifier 只处理“仍然不确定、且适合自动判定”的那一段 ask。

#### classifier backend 还有两套形态

当前 bundle 可见两种实现：

1. 旧的 tool-based `classify_result`
2. 新的 XML classifier
   - `fast`
   - `thinking`
   - `both/xml_2stage`

选择受 `tengu_auto_mode_config.twoStageClassifier` 控制。

#### classifier 失败时不是统一 fail-open

当前至少有四种不同失败语义：

- parse failure
  - 直接 block，偏 fail-closed
- classifier unavailable
  - 可能 fail-open 回到人工审批
  - 也可能在 `tengu_iron_gate_closed` 命中时 fail-closed
- transcript too long
  - 交互场景下回退到普通人工审批
  - 无 prompt 的 headless/async 场景可直接 deny
- request aborted
  - 记为 unavailable / interrupted

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
