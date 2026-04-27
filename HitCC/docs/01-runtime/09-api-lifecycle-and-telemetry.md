# API 请求生命周期、Telemetry 初始化与开关矩阵

## 本页用途

- 用来把请求级 telemetry、3P OpenTelemetry 初始化、startup telemetry、remote managed settings gating 放到同一页里。
- 用来回答“什么时候会初始化 telemetry、什么时候会延迟、什么时候只是关闭外围流量但不会彻底停掉 telemetry runtime”。

## 相关文件

- [05-model-adapter-provider-and-auth.md](./05-model-adapter-provider-and-auth.md)
- [06-stream-processing-and-remote-transport.md](./06-stream-processing-and-remote-transport.md)
- [../02-execution/06-context-runtime-and-tool-use-context.md](../02-execution/06-context-runtime-and-tool-use-context.md)
- [../03-ecosystem/02-remote-persistence-and-bridge.md](../03-ecosystem/02-remote-persistence-and-bridge.md)
- [../05-appendix/02-evidence-map.md](../05-appendix/02-evidence-map.md)

## 一句话结论

Claude Code CLI 里的 telemetry 不是一个单开关系统，而是至少分成四层：

- request lifecycle telemetry：`lj4 / Es1 / ij4`
- startup telemetry：`tengu_startup_telemetry`
- 3P OTEL metrics / logs / traces：`initializeTelemetry()`
- 1P event logging / GrowthBook event logger：独立于 `initializeTelemetry()` 的另一条链

因此 `DISABLE_TELEMETRY` 与 `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` 的效果并不相同，也不等于“所有 telemetry 都停了”。

## 初始化入口：`Rg8 -> H4A -> qvz -> initializeTelemetry`

当前本地 bundle 里，3P telemetry 初始化主入口已经可以写成：

```text
startup init
  -> Rg8()
    -> if remote managed settings gating active:
         maybe eager-init beta tracing
         wait tL8()
         -> SQ()
         -> H4A()
       else:
         -> H4A()

H4A()
  -> guard UI4
  -> qvz()
    -> import { initializeTelemetry } from Nl1
    -> meter = await initializeTelemetry()
    -> if meter:
         Fd8(meter, createCounter)
```

这里能直接确认：

- `H4A()` 只是一次性初始化 guard，不负责具体 exporter 组装
- `qvz()` 才是真正调用 `initializeTelemetry()` 的地方
- `initializeTelemetry()` 返回的是 meter；只有 meter 存在时，`Fd8(...)` 才会注册 counters

## `Rg8()` 在启动期何时触发

这一点现在也可以写得更实，不必只停留在“startup init”。

- TUI 主启动路径里，会先做 `SQ()`，再调用 `Rg8()`
- headless / `--print` / `stream-json` 主路径里，同样会先做 `SQ()`，再调用 `Rg8()`

因此更稳的结论是：

- telemetry 初始化属于启动早期动作
- 发生在 settings 已经同步到本地运行态之后
- 不依赖主对话循环已经开始

另外 `Rg8()` 在 remote managed settings 分支里还有一个容易漏掉的细节：

- 若 `CF1()` 为真，且 `lA() && rj()` 成立，会先尝试一次 `H4A()`
- 这次 eager init 失败只记日志，不会阻止后续 `tL8() -> SQ() -> H4A()` 的正式路径

所以 beta tracing 在特定入口下并不一定严格等到 remote settings 完成后才首次初始化。

## remote managed settings 为什么会阻塞 telemetry init

这部分现在已经不再只是现象描述。

### `Tu()`：是否需要 remote managed settings

`CF1()` 只是 `Tu()` 的薄包装。  
`Tu()` 当前至少要求：

- provider 是 `firstParty`
- 当前 host 仍是 Anthropic first-party host
- 不是 `local-agent` 入口
- 且满足下列任一条件：
  - OAuth token 存在，但 `subscriptionType === null`
  - OAuth token 存在，且 scopes 包含 enterprise 相关 scope，并且 `subscriptionType` 是 `enterprise / team`
  - 存在 API key

因此更稳的理解不是“登录了就一定等 remote settings”，而是：

- 只有 first-party 且具备 enterprise/team/待判定账户语义时，才进入 remote managed settings 路径

### `EBq()` / `tL8()` / `yBq()`

一旦 `CF1()` 为真，启动期会先执行 `EBq()`：

- 若还没有 loading promise，则创建 `C$6`
- 同时保存 resolver `TC`
- 超时后会强制 resolve，避免无限阻塞

之后：

- `tL8()` 只是 `await C$6`
- `yBq()` / `eL8()` 在 remote settings 刷新完成后 resolve `TC`
- `policySettings` 改变时会触发 `qX.notifyChange("policySettings")`

所以 telemetry 被“延迟初始化”的真实原因不是初始化函数内部主动等待，而是：

- 外层 `Rg8()` 先等待 remote settings loading promise
- promise resolve 后调用 `SQ()` 重刷 `process.env`
- 然后才进入 `H4A()`

### `SQ()` 在这里的作用

`SQ()` 当前可直接确认会：

- 把 `P8().env` 合回 `process.env`
- 把 `$A()?.env` 合回 `process.env`
- 然后刷新若干依赖 `process.env` 的运行态缓存

所以 remote managed settings 对 telemetry 初始化的影响，不是“传一个 settings 对象进去”，而是：

- 先把远端/本地 settings 合并进 `process.env`
- 再让 `initializeTelemetry()` 读取已经更新后的 OTEL / auth / proxy 相关环境变量

## `initializeTelemetry()`：实际做了什么

`initializeTelemetry()` 对应 `Wb_()`。  
它至少做四组事：

### 1. 预处理 env

- `bootstrapTelemetry()` / `_14()` 会默认把 `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE` 设成 `delta`
- 若 `B$A()` 条件成立，会把 console exporter 从 `OTEL_*_EXPORTER` 中剔掉
- 安装 OTEL diag logger：`YO6.diag.setLogger(new bQ1, ERROR)`

### 2. 构造 meter provider

无论 `CLAUDE_CODE_ENABLE_TELEMETRY` 是否开启，`Wb_()` 都会创建 `MeterProvider`。

区别只在 reader/exporter 来源：

- `q = z14()` 只看 `CLAUDE_CODE_ENABLE_TELEMETRY`
- `q === true`
  - 加入 `Jb_()` 产出的标准 metrics exporters
- `Db_() === true`
  - 再额外加入 `Xb_()` reader

这里最重要的结论是：

- `CLAUDE_CODE_ENABLE_TELEMETRY` 控制的是 OTEL exporter 组装
- 不是“是否创建 telemetry runtime 对象”的总开关

### 3. 条件性创建 event logger

标准路径里，只有 `q === true` 且 `Mb_()` 返回了至少一个 log exporter，才会：

- 创建 `LoggerProvider`
- `Dq8(loggerProvider)`
- `fq8(logger)` 设置全局 event logger

而 `GO(eventName, attrs)` 是否真的能发出事件，完全取决于：

- `ld8()` 是否返回已初始化的 event logger

如果没有 logger：

- `GO(...)` 会直接丢事件，并打印一次 `Event dropped (no event logger initialized)`

### 4. 条件性创建 tracer provider

traces 有两条独立路径：

- 标准 OTEL traces
  - 条件：`q === true && HS1()`
- beta tracing 旁路
  - 条件：`rj() === true`
  - 依赖 `ENABLE_BETA_TRACING_DETAILED + BETA_TRACING_ENDPOINT`

beta tracing 旁路 `fb_(resource)` 会直接：

- 建 `OTLPTraceExporter`
- 建 `OTLPLogExporter`
- 设置全局 tracer provider
- 设置全局 logger provider
- `fq8(logger)` 直接装 event logger

因此更稳的理解是：

- beta tracing 可以绕过标准 `CLAUDE_CODE_ENABLE_TELEMETRY` 开关，单独把 tracer/logger 拉起来

## `Fd8(...)`：counter 注册点

`Fd8(meter, createCounter)` 当前已直接确认注册这些 counters：

- `claude_code.session.count`
- `claude_code.lines_of_code.count`
- `claude_code.pull_request.count`
- `claude_code.commit.count`
- `claude_code.cost.usage`
- `claude_code.token.usage`
- `claude_code.code_edit_tool.decision`
- `claude_code.active_time.total`

这说明 session 级统计不是散落在业务代码里随意打点，而是：

- 先在 telemetry init 成功后统一注册
- 再由全局状态对象 `V8` 保存 counter 引用
- 后续业务路径只做 `counter.add(...)`

## startup telemetry 不是 `initializeTelemetry()` 的一部分

`amz()` 会打：

- `d("tengu_startup_telemetry", {...})`

但它有自己独立的 gating：

- 若 `NB()` 为真，直接返回

而 `NB()` 又等于：

- 使用 3P provider
- 或 `cl8() === true`

而 `cl8()` 来源于：

- `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`
- `DISABLE_TELEMETRY`

所以 startup telemetry 的结论应写成：

- 它不是 OTEL exporter 的一部分
- 它属于 1P internal event logging
- 但会被 3P provider、`DISABLE_TELEMETRY`、`CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` 一起压掉

### `tengu_startup_telemetry` 字段表

`amz()` 会并行读取 `RH()`、`bP6()`、`rp8()`，再拼上 sandbox/runtime/env 侧标志位后调用 `d("tengu_startup_telemetry", ...)`。  
当前能直接写实的字段如下，见 `cli.js`。

| 字段 | 来源 | 当前可确认语义 |
| --- | --- | --- |
| `is_git` | `RH()` | 当前工作目录是否处于 Git 仓库语境 |
| `worktree_count` | `bP6()` | 当前仓库可见的 worktree 数 |
| `gh_auth_status` | `rp8()` | 本地 `gh` CLI 鉴权状态；当前可确认枚举为 `not_installed / authenticated / not_authenticated` |
| `sandbox_enabled` | `_A.isSandboxingEnabled()` | sandbox 总开关是否开启 |
| `are_unsandboxed_commands_allowed` | `_A.areUnsandboxedCommandsAllowed()` | 是否允许执行非 sandbox 命令 |
| `is_auto_bash_allowed_if_sandbox_enabled` | `_A.isAutoAllowBashIfSandboxedEnabled()` | sandbox 开启时是否自动放行 bash |
| `auto_updater_disabled` | `qg()` | auto updater 是否被禁用 |
| `prefers_reduced_motion` | `TA().prefersReducedMotion ?? false` | UI reduced motion 偏好 |
| `has_node_extra_ca_certs` | `process.env.NODE_EXTRA_CA_CERTS` | 是否配置额外 CA 证书环境变量 |
| `has_client_cert` | `process.env.CLAUDE_CODE_CLIENT_CERT` | 是否配置 client cert |
| `has_use_system_ca` | `LJ6("--use-system-ca")` | 是否显式传入 `--use-system-ca` |
| `has_use_openssl_ca` | `LJ6("--use-openssl-ca")` | 是否显式传入 `--use-openssl-ca` |

这里还有一个边界要写清：

- `omz()` 只上报这些 CA / cert 开关是否存在
- 没有把证书路径、证书内容或其他敏感值直接塞进 startup telemetry

### `gh_auth_status` 的枚举值已经可以钉死

`gh_auth_status` 对应 `rp8()`，当前本地实现非常具体，见 `cli.js`：

- 先 `Qw("gh")` 检查本机是否存在 `gh` 可执行文件
- 若不存在，返回 `not_installed`
- 若存在，执行 `gh auth token`
  - `timeout=5000`
  - `stdout/stderr` 都忽略
  - `reject=false`
- `exitCode === 0` 时返回 `authenticated`
- 否则返回 `not_authenticated`

因此这不是一个泛化的“GitHub 登录状态”，而是更窄的：

- 本地 `gh` CLI 是否安装
- 以及 `gh auth token` 是否能在 5 秒内成功返回 token

当前没看到它会：

- 区分多 host / 多 profile
- 区分 token scope
- 复用 Claude 自己的 GitHub import-token / remote session 鉴权状态

## 1P event logging 配置来源：`Qg7()` / `gg7()`

1P event logging 不是写死常量初始化，而是显式吃两组动态配置，见 `cli.js`。

### `tengu_event_sampling_config`

- `gg7()` 读取 `TZ("tengu_event_sampling_config", {})`
- `vZ1(eventName)` 再从结果里按事件名取 `sample_rate`
- `sample_rate`
  - 不是数字、或小于 `0`、或大于 `1`：视为无效，返回 `null`
  - 大于等于 `1`：等价于不抽样
  - 小于等于 `0`：等价于完全丢弃
  - `0~1`：按概率采样

### `tengu_1p_event_batch_config`

- `Fg7()` 读取 `TZ("tengu_1p_event_batch_config", {})`
- `Qg7()` 会把这组配置映射到 `BatchLogRecordProcessor` 与 `ZZ1` exporter：
  - `scheduledDelayMillis`
    - 优先 `q.scheduledDelayMillis`
    - 否则回退到 `parseInt(process.env.OTEL_LOGS_EXPORT_INTERVAL || "10000")`
  - `maxExportBatchSize`
    - 优先 `q.maxExportBatchSize`
    - 默认 `200`
  - `maxQueueSize`
    - 优先 `q.maxQueueSize`
    - 默认 `8192`
  - `skipAuth`
    - 透传到 `new ZZ1({ skipAuth })`
  - `maxAttempts`
    - 透传到 `new ZZ1({ maxAttempts })`
  - `path`
    - 透传到 `new ZZ1({ path })`
  - `baseUrl`
    - 透传到 `new ZZ1({ baseUrl })`

因此更稳的结论是：

- 1P event logging 的 flush cadence 既受 GrowthBook/remote config 影响，也受 `OTEL_LOGS_EXPORT_INTERVAL` 兜底
- batch exporter 的 endpoint、认证策略与重试上限都不是固定编译时常量

## 哪些设置会修改遥测数据或上报行为

这一层当前需要明确拆开，不要再混成“telemetry 开关”一个词。

本地已能正证的入口主要有三类：

- 通过 settings `env` 注入、最后进入 `process.env` 的环境变量
- remote config / GrowthBook 下发的 1P event logging 配置
- 业务设置本身，它们不是 telemetry 专用开关，但会直接改写 telemetry payload 里的字段值

### A. 产品运行态模式位

这组变量主要决定“哪些 telemetry / 外围流量路径会被旁路”，而不等于“telemetry runtime 一定不初始化”。

| 设置 | 当前作用 |
| --- | --- |
| `DISABLE_TELEMETRY` | 把运行态切到 `no-telemetry`，通过 `cl8()/NB()` 压掉 startup telemetry 和部分外围路径；当前本地 bundle 没看到它直接短路 `initializeTelemetry()` |
| `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` | 把运行态切到 `essential-traffic`，影响面比 `DISABLE_TELEMETRY` 更宽，会压掉更多非必要出网/上报链路 |
| `DISABLE_ERROR_REPORTING` | 只直接作用于错误上报 sink `O6(...)`，不等于 OTEL / 1P event logging / startup telemetry 全停 |

### B. 标准 3P OpenTelemetry 组装与 exporter 配置

这组变量主要决定“是否组装 3P telemetry exporter、发到哪里、多久 flush 一次”。

| 设置 | 当前作用 |
| --- | --- |
| `CLAUDE_CODE_ENABLE_TELEMETRY` | 标准 3P OTEL metrics/logs/traces exporter 总开关 |
| `OTEL_METRICS_EXPORTER` | metrics exporter 类型集合 |
| `OTEL_LOGS_EXPORTER` | logs exporter 类型集合；决定 `GO(...)` 事件是否可能真正发出 |
| `OTEL_TRACES_EXPORTER` | traces exporter 类型集合 |
| `OTEL_METRIC_EXPORT_INTERVAL` | metrics 导出间隔 |
| `OTEL_LOGS_EXPORT_INTERVAL` | logs 导出间隔；同时也是 1P event logging batch delay 的 env fallback |
| `OTEL_TRACES_EXPORT_INTERVAL` | traces 导出间隔 |
| `OTEL_EXPORTER_OTLP_ENDPOINT` 与 `OTEL_EXPORTER_OTLP_*` | OTLP exporter endpoint、headers、protocol、TLS/client cert 等连接参数 |
| `CLAUDE_CODE_OTEL_SHUTDOWN_TIMEOUT_MS` | telemetry shutdown 时的 flush timeout |
| `CLAUDE_CODE_OTEL_FLUSH_TIMEOUT_MS` | 显式 flush 时的 timeout |

这里要强调：

- 这组 `OTEL_*` 大量来自 vendored OpenTelemetry SDK
- 但它们确实会改变客户端遥测的上报后端、批处理节奏和连接参数
- 因此不能因为它们“不是产品自定义键名”就把它们排除在遥测配置面之外

### C. beta tracing 旁路

这组变量不是标准 OTEL 主开关，而是另一条 debug / tracing 旁路。

| 设置 | 当前作用 |
| --- | --- |
| `ENABLE_BETA_TRACING_DETAILED` | 打开 beta tracing 条件之一 |
| `BETA_TRACING_ENDPOINT` | beta tracing 的 log/trace OTLP endpoint |

当前更稳的边界是：

- `ENABLE_BETA_TRACING_DETAILED + BETA_TRACING_ENDPOINT` 可以绕过标准 `CLAUDE_CODE_ENABLE_TELEMETRY`
- 因此它们属于独立的 telemetry 配置面

### D. 会直接改写事件 payload 内容的 env

这组变量不是“是否上报”的开关，而是会改变每条事件带什么字段、字段是否脱敏。

| 设置 | 当前作用 |
| --- | --- |
| `OTEL_LOG_USER_PROMPTS` | 决定 `user_prompt` 事件里 `prompt` 是原文还是固定 `<REDACTED>` |
| `OTEL_METRICS_INCLUDE_SESSION_ID` | 决定通用 OTEL 事件属性里是否带 `session.id` |
| `OTEL_METRICS_INCLUDE_VERSION` | 决定是否带 `app.version` |
| `OTEL_METRICS_INCLUDE_ACCOUNT_UUID` | 决定是否带 `user.account_uuid` 与 `user.account_id` |
| `CLAUDE_CODE_ACCOUNT_TAGGED_ID` | 改写 `user.account_id` 的值 |
| `CLAUDE_CODE_WORKSPACE_HOST_PATHS` | 为通用 OTEL 事件属性补 `workspace.host_paths` |

因此当前可直接写死：

- `user_prompt` 默认不是明文上报
- session/account/version 这几类公共维度是否出现，受独立 env 控制

### E. 1P event logging 的 remote config

这组不是本地一级 settings 键，而是通过 `TZ(...)` 读取的远端/实验配置。

| 设置 | 当前作用 |
| --- | --- |
| `tengu_event_sampling_config` | 按事件名配置 `sample_rate`，决定 1P internal event 是否抽样、全发或直接丢弃 |
| `tengu_1p_event_batch_config` | 控制 1P event logging 的 batch delay、队列大小、导出批大小、认证策略、重试次数、path、baseUrl |

因此 1P event logging 的“采样率”和“批处理发送策略”当前不是写死常量。

### F. 不是 telemetry 专用开关，但会改写 telemetry 字段值的业务设置

这些设置不应被归类成“telemetry settings”，但它们会直接进入 request/startup/internal event payload：

- `model`
- `effortLevel`
- `fastMode`
- provider 切换
- permissions / sandbox 相关模式

例如：

- request telemetry 会记录 `model`、`effortValue`、`fastMode`、`permissionMode`、`provider`
- internal event core metadata 也会带 `model`、`betas`、`client_type` 等上下文

因此文档层更稳的分类应该是：

- “控制遥测 runtime / sink 的设置”
- “控制事件内容是否脱敏/是否附带维度的设置”
- “会间接改写遥测字段值的业务设置”

## `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` 与 `DISABLE_TELEMETRY`：当前可确认的差异

### 模式判定

```text
i$A():
  CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC -> "essential-traffic"
  DISABLE_TELEMETRY -> "no-telemetry"
  else -> "default"

BO()  = mode === "essential-traffic"
cl8() = mode !== "default"
```

因此：

- `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`
  - `BO() === true`
  - `cl8() === true`
- `DISABLE_TELEMETRY`
  - `BO() === false`
  - `cl8() === true`

这意味着两者不是同义词。

还能进一步补一层 helper 关系：

```text
NB() =
  CLAUDE_CODE_USE_BEDROCK
  || CLAUDE_CODE_USE_VERTEX
  || CLAUDE_CODE_USE_FOUNDRY
  || cl8()
```

因此：

- `NB()` 不是“telemetry disabled”判定
- 而是一个更宽的“某些外围能力需要关掉/绕开”的 helper
- provider 切到 3P，或进入 `essential-traffic / no-telemetry` 模式，都会让 `NB() === true`

### 当前已能直接确认的 gating 矩阵

#### `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`

先要区分两个层次：

- `BO()` helper 直接命中的分支
- 同一个“essential-traffic”模式下、但不是通过 `BO()` helper 守卫的分支

如果只按 `BO()` 这个 helper 本身看，当前命中点已经能基本列全。

##### A. `BO()` helper 直接命中的分支

**直接压掉出网/上报函数：**

- `O6(...)` 错误上报
- `jg7()` model capabilities 拉取
- `l28()` fast mode prefetch
- `$Tq()` rate-limit / quota 相关预拉取
- `Zu()` account settings 读取
- `dA6()` `claude_code_grove` 读取
- `vG_()` org metrics opt-out API
- `U1z()` bug/feedback 上报
- `J1A()` changelog 拉取
- `Zbz()` `/api/claude_cli/bootstrap`

这里还能再写细一点：

- `jg7()`
  - 通过 `client.models.list(...)` 拉模型能力
- `$Tq()`
  - 通过 `y1_()` 发一个 `source: "quota_check"` 的轻量模型请求，读取 quota / rate-limit 头
- `Zu()`
  - `GET /api/oauth/account/settings`
- `dA6()`
  - `GET /api/claude_code_grove`
- `vG_()`
  - `GET /api/claude_code/organizations/metrics_enabled`
- `U1z()`
  - `POST /api/claude_cli_feedback`
- `Zbz()`
  - `GET /api/claude_cli/bootstrap`

**不直接出网，但会提前关掉产品能力入口：**

- `p$("allow_product_feedback")`
  - 当本地没有 policy limits 配置时，`BO() && qM_.has("allow_product_feedback")` 会直接返回 `false`
- `feedback/bug` 命令的 `isEnabled`
  - 显式把 `BO()` 作为禁用条件之一

**只阻止后台刷新，不等于绝对阻止底层 endpoint：**

- `vk4()`
  - 只负责触发 `T1A()` 的后台 passes eligibility 刷新
  - 这里的 `BO()` 命中说明“启动/后台刷新”会被压掉
  - 但底层 `C$z()` referral eligibility endpoint 本身在当前可见代码里没有单独 `BO()` 守卫

##### B. 同一模式下，但不是 `BO()` helper 直接命中的分支

下面这些路径同样会被 `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` 影响，但实现不是 `if (BO())`：

- `amz()` startup telemetry
  - 通过 `NB() -> cl8()` 被压掉
- `Wg7()` MCP registry 拉取
  - 直接检查 `process.env.CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`

因此如果未来继续补证据，必须避免把“`BO()` helper 命中列表”和“整个 essential-traffic mode 的全部影响面”混成一张表。

##### C. `DISABLE_TELEMETRY` 不会一起压掉的东西

这一点现在也能说得更硬：

- `DISABLE_TELEMETRY`
  - 只会让 `i$A() === "no-telemetry"`
  - 因此 `cl8() === true`
  - 但 `BO() === false`
- 直接后果是：
  - 那些用 `BO()` 守卫的“非必要流量”不会自动一起停
  - 例如 `Wg7()` 这类直接看 `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` 的路径，不会因为 `DISABLE_TELEMETRY` 被压掉

所以更稳的工程结论是：

- `DISABLE_TELEMETRY`
  - 更像“停 telemetry 相关面 + 触发 `cl8()` 族旁路”
- `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`
  - 才是“真正的非必要出网总开关”

当前没看到它会阻断的主链：

- `_I4(...)` / `VN8(...)` / `/v1/messages`
- auth token / API key 主鉴权
- remote managed settings 获取

### 一个更实用的本地判断口径

如果未来重写时只想快速判断“这个功能会不会被模式开关压掉”，当前更稳的顺序是：

1. 先看它是不是直接用 `BO()`
2. 再看它是不是直接读 `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`
3. 再看它是不是通过 `NB()/cl8()` 被间接旁路
4. 最后再看它是否单独依赖 `CLAUDE_CODE_ENABLE_TELEMETRY` 或 beta tracing env

不要再把这些路径压缩成一个单布尔 `TelemetryEnabled`。

所以它更准确的语义是：

- 关闭“外围增强与遥测相关流量”
- 不是把 inference data plane 关掉

#### `DISABLE_TELEMETRY`

当前在 bundle 里能直接看到的作用更窄：

- 让模式变成 `no-telemetry`
- 因而 `cl8() === true`
- 进而让 `NB()` 为真，压掉 `tengu_startup_telemetry`

但当前本地 bundle **没有直接看到** 它：

- 让 `BO()` 变真
- 或直接短路 `initializeTelemetry()`
- 或直接传入 `z14()`

所以当前更稳的判断应写成：

- `DISABLE_TELEMETRY` 在本地 bundle 里主要是一个“外层模式位”
- 它不会像 `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` 那样批量压掉所有 `BO()` 守卫路径
- 它也不等同于“彻底不初始化 telemetry runtime”

#### 本地 bundle 中，`DISABLE_TELEMETRY` 只有一个直接读点

这一点现在可以收紧得更硬，不必继续写成“暂时没看到更多”。

当前本地 bundle 对 `DISABLE_TELEMETRY` 的直接读取只看到一处，见 `cli.js`：

- `i$A()`
  - `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC -> "essential-traffic"`
  - `DISABLE_TELEMETRY -> "no-telemetry"`
  - 否则 `"default"`
- `cl8()` 只是 `i$A() !== "default"`
- `NB()` 再把 `cl8()` 并入 startup telemetry / 若干外围路径的 gating

与之相对，3P OpenTelemetry 初始化主链 `Wb_()` 读的是另一套变量，见 `cli.js`：

- `z14()` 只看 `CLAUDE_CODE_ENABLE_TELEMETRY`
- `Jb_()` / `Mb_()` / `Pb_()` 从 `OTEL_*` 环境变量组装 metrics/logs/traces exporter
- shutdown 提示里明确写的是“Disable OpenTelemetry by unsetting `CLAUDE_CODE_ENABLE_TELEMETRY`”

因此当前能正证的本地结论应改成：

- `DISABLE_TELEMETRY` 在本地 bundle 里不是 3P OTEL exporter 的主开关
- 标准 3P OTEL logs/metrics/traces 是否组装，直接取决于 `CLAUDE_CODE_ENABLE_TELEMETRY`
- `DISABLE_TELEMETRY` 当前可见作用仍主要是把运行态切到 `no-telemetry`，从而通过 `cl8()/NB()` 压掉 startup telemetry 和部分外围流量

#### `OTEL_*` 命中很多，但大多属于 vendored OpenTelemetry

继续按字面搜索后，这个边界也可以写得更清楚：

- bundle 里确实有大量 `OTEL_*` 环境变量命中
- 但其中相当一部分来自 vendored OpenTelemetry SDK 自身
- 对产品侧行为最关键的本地 wrapper 仍主要集中在：
  - `z14()`
    - 读 `CLAUDE_CODE_ENABLE_TELEMETRY`
  - `fb_()`
    - 读 `BETA_TRACING_ENDPOINT`
  - `Wb_()`
    - 组装 readers / log exporters / trace exporters
  - `i$A()`
    - 读 `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`
    - 读 `DISABLE_TELEMETRY`

因此不要把“bundle 内存在很多 `OTEL_*` 读点”误解成“产品层自己维护了一大套额外 gating”。  
更稳的工程划分是：

- `OTEL_*`
  - 主要属于 exporter/SDK 配置面
- `CLAUDE_CODE_ENABLE_TELEMETRY`
  - 标准 3P OTEL 组装总开关
- `BETA_TRACING_ENDPOINT` / `ENABLE_BETA_TRACING_DETAILED`
  - beta tracing 旁路
- `DISABLE_TELEMETRY` / `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`
  - 产品运行态模式位

#### 当前还能直接确认的间接影响面

除了 `tengu_startup_telemetry`，`DISABLE_TELEMETRY -> cl8() -> NB()` 在本地还会命中这些路径：

- 1P event logging 总开关
  - `b_6() = !NB()`
  - `vU6(...)`、`TZ1(...)`、`Qg7()`、`EA9()` 都先检查 `b_6()`
  - 因此 `DISABLE_TELEMETRY` 会把 first-party internal event logging 与 GrowthBook experiment event logging 一并压掉
- `tengu_context_size`
  - `tb4(...)` 开头就是 `if (NB()) return`
  - 所以这条上下文规模统计也会被压掉
- feedback survey
  - `W_8() = cl8()`
  - session quality / transcript ask 这组 survey 展示逻辑里会直接 `if (W_8()) return`
  - 因此 `DISABLE_TELEMETRY` 也会关闭产品反馈 survey 弹出

这里要特别区分：

- `/bug` 命令是否可用，当前仍主要受 `BO()` 和 `allow_product_feedback` 控制
- feedback survey 则会被 `W_8()` 直接关掉

也就是说，`DISABLE_TELEMETRY` 会关 survey，但当前本地并不能证明它会像 `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` 一样把 `/bug` 命令入口也一并关掉

#### `DISABLE_ERROR_REPORTING`

`O6(error)` 的 return-early 条件当前已经能直接写死，见 `cli.js`：

- `CLAUDE_CODE_USE_BEDROCK`
- `CLAUDE_CODE_USE_VERTEX`
- `CLAUDE_CODE_USE_FOUNDRY`
- `DISABLE_ERROR_REPORTING`
- `BO() === true`

因此这里要避免继续写成“`O6(...)` 只会被 `BO()` 压掉”。

更准确的说法是：

- `DISABLE_ERROR_REPORTING` 只直接作用于错误上报 sink
- 它和 `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`、provider 切换开关会在 `O6(...)` 这一层汇合
- 这不等同于 `initializeTelemetry()`、`tengu_startup_telemetry`、1P event logging 都被同一变量关闭

## 一个反直觉但现在已能确认的点

从本地代码直读，下面这件事是成立的：

- 即使 `DISABLE_TELEMETRY=1`
- 只要启动链仍调用 `Rg8() -> H4A() -> initializeTelemetry()`
- 并且别的条件满足
- telemetry runtime 仍可能被初始化

特别是：

- beta tracing 旁路 `rj()` 不读取 `DISABLE_TELEMETRY`
- 它只看 `ENABLE_BETA_TRACING_DETAILED + BETA_TRACING_ENDPOINT`

所以如果未来要重写，这里必须明确拆成：

- 非必要流量模式
- 1P startup telemetry
- 标准 OTEL logs/metrics/traces
- beta tracing debug path

不能继续混成一个 `TelemetryEnabled` 布尔值。

## 与请求生命周期页的关系

本页负责回答：

- telemetry 何时初始化
- event logger / meter / tracer 何时存在
- 哪些开关会关掉哪些外围流量

而请求级字段与 `lj4 / Es1 / ij4` 的闭环，见：

- [06-stream-processing-and-remote-transport.md](./06-stream-processing-and-remote-transport.md)

## 当前仍保留的边界

- `DISABLE_TELEMETRY` 是否还会被远端服务端侧额外解释，本地 bundle 无法证明
- 1P event logging 的完整服务端协议未在 bundle 中展开，只能确认本地初始化与调用点
- 若只统计 `BO()` helper 直接命中的分支，当前已基本覆盖
- 但若按“essential-traffic mode 的全部影响面”统计，仍需继续补直接读 env 或经 `cl8()/NB()` 间接生效的外围路径

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
