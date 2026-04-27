# 配置与用户设置系统：键族消费索引与重写边界

## 键族到消费点索引

这一节不追求把每个 key 全量平铺，而是把当前已能钉死的高价值键族，按“谁在消费、何时消费、是否带 gating”收成导航索引。

### 1. `permissions.*` / auto mode

| 键族 | 主要消费点 | 消费方式 | 关键边界 |
| --- | --- | --- | --- |
| `permissions.defaultMode` | `LqA(...)`、settings UI 的 `defaultPermissionMode` 条目 | 启动时参与初始 permission mode 选择；设置面板可写回 `userSettings.permissions.defaultMode` | remote 场景下只接受 `acceptEdits / plan / default`，不接受完整本地枚举 |
| `skipDangerousModePermissionPrompt` | `Gf6()`、`lr4()` | 主要作为“是否已接受 dangerous mode prompt”的跨 source 持久化位 | 当前检查 user/local/flag/policy 四个 source，不看 project |
| `skipAutoPermissionPrompt` | `Al()`、`IqA()`、`hY6()` | 作为 auto mode opt-in 事实源；决定 plan mode 能否进入 hidden auto-active | 任一 source 为真都可视作已 opt-in |
| `useAutoModeDuringPlan` | `xP7()`、`IqA()`、`oy6(...)`、settings UI 对应 toggle | 运行期 gate；切换后会立即重算当前 `toolPermissionContext` | 任一 source 显式为 `false` 都会阻断 plan 内 auto 语义 |
| `disableBypassPermissionsMode` | `LqA()`、`fo()` | 启动时过滤 `bypassPermissions` 候选 mode | settings 可直接把 bypass mode 从候选集中移除，即使 CLI 显式请求 |
| `disableAutoMode` | `RqA()`、`SV()`、`qS6()` | 启动 gate + 运行期持续校验；不可用时会把当前 context 从 `auto` 或 `plan+auto` 踢回普通态 | 不是只灰 UI，而是真正修改 permission context |
| `autoMode.allow / soft_deny / environment` | `_p6()`、classifier prompt builder | 跨 `user/local/flag/policy` 聚合后，替换 classifier prompt 中对应 section | schema 虽有 `deny`，但当前本地消费链没有把它继续注入 classifier prompt |

这部分更细的 mode 状态机、classifier 输入和审批队列，仍以 permission 专题页为主。

### 2. `sandbox.*` / `env` / `defaultShell` / `hooks`

| 键族 | 主要消费点 | 消费方式 | 关键边界 |
| --- | --- | --- | --- |
| `sandbox.network.*` / `sandbox.filesystem.*` | `RG8(settings)` | 运行期组装真正下发给 sandbox runtime 的网络/文件系统策略 | 它会和 `WebFetch(domain:...)`、`Edit(...)`、`Read(...)` 等 permission rules 合流，不是独立权限表 |
| `defaultShell` | `FU4()` | 作为命令执行层的默认 shell 选择 | 缺省时回退到 `"bash"` |
| `env` | `II4()`、`SQ()` | `II4()` 在启动时把 app state 与 settings env 写入 `process.env`；`SQ()` 在运行中重刷 `process.env` 并刷新依赖缓存 | `policySettings.env` 会在 `Tu()` 判定后参与；`SQ()` 不只是赋值，还会联动 `coA / loA / F_7 / C$8` |
| remote managed settings 里的 shell/env/hook 高风险子集 | `sN6()`、`fbq()`、`vBq()`、`zBq()` | 远端 settings 落盘前先做安全确认 | 只有 shell settings、allowlist 外 env、`hooks` 变化才会弹框，不是所有远端改动都确认 |
| `hooks` / `disableAllHooks` / `allowManagedHooksOnly` | `nzq()`、`bS()`、`Tl6()`、`BD()`、`dN8()` | 先求 effective hooks，再决定 session/setup/plugin hooks 是否运行 | managed-only 可以让 user/project/local hooks 失效；`disableAllHooks` 还能直接短路 hook 执行链 |

因此 `env` 和 `hooks` 不是“偏好项”，而是直接进入运行时执行面与出网/遥测初始化面。

这里还要单独补一个当前最容易误判的点：

- 本地 bundle 里没有看到稳定的一组一级 `telemetry.*` settings 键
- 遥测相关配置当前主要是通过 `env` 注入，再由 telemetry init / 1P event logging / beta tracing 读取
- 因此如果要追“哪个 settings 改了 telemetry”，第一落点应先看 `settings.env`、`policySettings.env` 与 remote managed settings 写入的 env，而不是去找独立 telemetry schema

换句话说，当前 settings 系统对 telemetry 的主要暴露面是：

- `env`
- remote managed settings 对 `process.env` 的重刷
- 少量远端 experiment/config key 对 1P event logging 的再配置

而不是一个显式的 `telemetry` 顶层对象。

### 3. `model` / `availableModels` / `modelOverrides` / `effortLevel` / `advisorModel` / `fastMode`

| 键族 | 主要消费点 | 消费方式 | 关键边界 |
| --- | --- | --- | --- |
| `availableModels` | `Se(model)` | 作为模型选择 gate，决定某个 model 是否允许被选中 | 支持 alias、版本前缀、完整 model id；空数组不是“无限制”，而是“只有默认模型可用” |
| `modelOverrides` | `BP7()`、`Q28()`、`N3()` | 在 provider 模型表构建时做正向 override；在显示/比较时做反向映射 | 它不是 UI label rewrite，而是 provider model id 映射层 |
| `effortLevel` | `Mv1()`、`lPz()` / `iPz()`、`ModelPicker` | 既是持久化默认值，也是 `/effort` 与模型选择器的写回目标 | `CLAUDE_CODE_EFFORT_LEVEL` 可覆盖当前 session，使 settings 中的 effort 只保留为持久化偏好 |
| `advisorModel` | `T2q()`、`/advisor` 命令实现 `pMz(...)` | 运行期从 app state 读取，并写回 `userSettings.advisorModel` | advisor 还受当前主模型能力约束，不是设置了就一定可用 |
| `fastMode` / `fastModePerSessionOptIn` | `Rj1()`、settings UI fast mode toggle | 参与启动时的 fast mode 默认态，并可由设置面板写回 `userSettings.fastMode` | 若 `fastModePerSessionOptIn === true`，settings 中的 `fastMode` 不会跨 session 自动开启 |

这里能看出 model family 的设置不只决定“显示哪个模型名”，还会改写 provider 映射、推理强度和 session 默认态。

### 4. `enabledPlugins` / marketplaces / `pluginConfigs` / `strictPluginOnlyCustomization`

| 键族 | 主要消费点 | 消费方式 | 关键边界 |
| --- | --- | --- | --- |
| `enabledPlugins` | `$3z()`、`Rq6()`、`SJ4()`、`bJ4()`、`M9z()` | 先按 source/scope 合成 enabled set，再驱动 plugin runtime 真实装配 | `policySettings.enabledPlugins` 不只“优先级更高”，还能把同名 plugin 直接锁死为 disabled |
| `extraKnownMarketplaces` | `Qvq()`、marketplace resolve 链 | 把 settings-sourced marketplace 注册进运行期 marketplace 集合 | 它解决的是“注册/发现”，不是 strict policy gate 本身 |
| `strictKnownMarketplaces` / `blockedMarketplaces` | `d16()`、`QI1()`、`DW6()` | 在 marketplace add/fetch 前做 source 级准入/阻断 | 检查发生在下载前，阻断的 marketplace 不会落磁盘 |
| `pluginConfigs` | `GY6()`、`Wg8()`、`bZ()`、`cG8()` | 存储 plugin options 与 plugin MCP server user config；敏感项拆到 secure storage | 它不是普通 UI 缓存，后续还会通过 `${user_config.KEY}` 进入 skills / MCP / hooks 插值链 |
| `strictPluginOnlyCustomization` | `IZ(surface)`、`nzq()`、`sx_(...)`、MCP resolve 链 | 作为 surface 级 capability gate，按 `hooks / mcp / skills / agents` 等面切断非 plugin 来源 | 它不是简单 merge 优先级，而是“该 surface 只允许 plugin/managed/builtin 来源继续生效” |

这一组说明 plugin system 与 settings system 是双向耦合：

- settings 决定哪些 plugin / marketplace / plugin config 能进入 runtime
- plugin 又反过来提供 settings overlay、MCP、hooks、skills、agents

另外，plugin hook 热刷新并不是盲重载。`$u1()` 当前只跟踪：

- `enabledPlugins`
- `extraKnownMarketplaces`
- `strictKnownMarketplaces`
- `blockedMarketplaces`

而 `q4_()` 只在这些 plugin-affecting settings 真的变化时，才触发 plugin hook reload。

#### `strictPluginOnlyCustomization` 的 surface 边界已经可以收紧

当前 schema 允许的 surface 不是开放集合，而是固定四个：

- `skills`
- `agents`
- `hooks`
- `mcp`

`true` 等价于锁住全部四个 surface；数组形式只锁指定项。未知值会在 schema preprocess 阶段被过滤，再由 `.catch(void 0)` 宽容吞掉。

| surface | 主要 gate | 被切掉的非 plugin 来源 | 仍保留的来源 | 关键边界 |
| --- | --- | --- | --- | --- |
| `hooks` | `IZ("hooks")`、`nzq()`、`R74()`、`BN(...)` | `user/project/local` 合成后的普通 settings hooks；来自非 plugin source 的 command/agent frontmatter hooks | `policySettings.hooks`、plugin hooks、builtin/bundled/plugin-source command/agent hooks | 它不只是“settings hooks 不生效”；连 command/agent 自带 hooks 也会按 `vl6(source)` 做 source gate |
| `mcp` | `IZ("mcp")`、`IN()`、`Uw6()`、`sx_(...)` | user/project/local MCP config、`.mcp.json` 进入的普通手工 server、非 plugin agent 声明的 `mcpServers` | enterprise / managed MCP config、plugin MCP servers、plugin agent 自带 MCP | `IN(name)` 在锁定时只回看 enterprise；`Uw6()` 也会把 user/project/local MCP source 缩成空集 |
| `skills` | `IZ("skills")`、`pE6()`、`_R1()` | `~/.claude/skills`、项目 `.claude/skills`、动态 skill 目录发现 | managed skill dir、plugin skills、builtin plugin skills、bundled skills | 当前本地证据只显示它切 skill-dir 来源；不是“所有 prompt command 都没了” |
| `agents` | `IZ("agents")`、`Ao("agents", ...)` | `~/.claude/agents`、项目 `.claude/agents` | managed agents、plugin agents、builtin agents | `Ao(...)` 只对 `agents` 特判 user/project 目录，不会顺手影响别的目录型 surface |

这里有两个很容易误判的边界：

- 它不是“plugin 优先级更高”，而是对指定 surface 直接切断非 plugin source。
- 它也不是“所有自定义目录都锁死”。当前本地版本里，`commands / output-styles / workflows` 不在 `Jf6` 内，目录 loader 也没有同类 special-case，不能把它们误写成同样受这个 key 控制。

### 5. `allowedMcpServers` / `remote.defaultEnvironmentId` / channels

| 键族 | 主要消费点 | 消费方式 | 关键边界 |
| --- | --- | --- | --- |
| `allowedMcpServers` / `deniedMcpServers` | `oLq()`、`nk6()`、`va6()`、`Uw6()`、`TA6()` | 在 MCP add、resolve、连接前三个阶段持续做 allow/deny 判定 | 既支持 `serverName`，也支持 stdio command / URL pattern；deny 永远先于 allow |
| `allowManagedMcpServersOnly` | `I3_()`、`S3_()` | 只把 allowlist 读取源收缩到 `policySettings` | denylist 仍继续读 `TA()`，所以用户仍可“额外 deny 自己不想用的 server” |
| `remote.defaultEnvironmentId` | remote session 创建链、`DR4()`、`WR4()` | 远端会话创建时作为默认环境选择；settings UI 可把它写回 `localSettings.remote.defaultEnvironmentId` | 若配置的 environment id 不存在，会回退到 `anthropic_cloud` / 首个非 bridge 环境 |
| `channelsEnabled` / `allowedChannelPlugins` | `D68()` | channel registration 前的 org policy gate | team / enterprise 账户下，未启用 `channelsEnabled` 时直接拒绝；allowlist 还会校验 plugin 与 marketplace 是否命中批准列表 |

这一组说明：

- MCP 不是“读取配置后直接连”，而是持续受 enterprise policy gate 约束
- remote settings 里的 `remote.defaultEnvironmentId` 已经有明确运行期消费者，不只是 UI 预留字段
- channel 入口和普通 MCP tool 暴露是两套不同的准入面

### 6. `outputStyle` / `language` / spinner / `defaultView` / `promptSuggestionEnabled`

| 键族 | 主要消费点 | 消费方式 | 关键边界 |
| --- | --- | --- | --- |
| `outputStyle` | `At6()`、`R6z()`、config 面板 `OutputStyle` 对话框 | 既决定 REPL 主线程的 output-style querySource / prompt 标识，也决定最终渲染风格；设置面板写回 `localSettings.outputStyle` | 它不是纯视觉主题，而会进入 prompt/runtime 选择面 |
| `language` | config 面板 `Language` 对话框、`t88(TA().language)` | 作为响应语言偏好持久化到 `userSettings.language`；语音听写路径也会直接消费它选择 dictation language | 它不是只影响文案显示，voice path 也会读 |
| `spinnerTipsEnabled` / `spinnerTipsOverride` | `Uj_()`、`lRz()`、`XU8()` | 前者控制是否展示 spinner tip；后者可增补或替换默认 tip 集 | `spinnerTipsOverride.excludeDefault` 会把默认 tip 集整体切掉 |
| `prefersReducedMotion` | `Uj_()`、`qj_()`、若干 `lj(...)` 动画调用点 | 作为动画/闪烁/过渡的总开关 | 它不是 theme 选项，而是明确的 accessibility/runtime gate |
| `defaultView` | 启动期 `TA().defaultView === "chat"` 分支 | 启动时决定是否默认进入 brief/chat 视图，并联动 `isBriefOnly` | 它影响的是启动视图选择，不只是 config 面板展示态 |
| `promptSuggestionEnabled` | `bg4()`、initialize 请求处理链 | 控制输入框 placeholder / proactive suggestion 是否出现；初始化请求可在未显式禁用时把运行态 prompt suggestion 打开 | 显式 `false` 是硬禁用；不是单纯“默认值为空” |

### 7. `skipWebFetchPreflight` / `autoMemory*` / `autoDream*` / `sshConfigs`

| 键族 | 主要消费点 | 写回入口 | 热生效特征 | 关键边界 |
| --- | --- | --- | --- | --- |
| `skipWebFetchPreflight` | `co1(...)`、startup telemetry `wBz(...)` | 当前未见本地 UI / 命令写回入口 | `WebFetch` 调用时即时读取 | 它只跳过 `bY4(hostname)` 这一步 domain preflight，不会绕过 `WebFetch(domain:...)` 权限规则，也不会绕过网络出口代理的 403 阻断 |
| `autoMemoryEnabled` | `r5()`、`fT8()`、`kj4()/yo_()`、memory path 分类链 `eT6()/m7_()/dx1()/KNq()/ _Nq()`、UI toggle | auto-memory 面板 toggle -> `wA("userSettings", { autoMemoryEnabled })` | 运行时按 `r5()` 热读取 | 关闭后不只是“少一个目录”：auto-memory prompt 注入、memory path 识别、背景 memory extraction 都会一起停掉；`CLAUDE_CODE_DISABLE_AUTO_MEMORY` 会优先压过 settings |
| `autoMemoryDirectory` | `mA9()`、`rO()` | 当前未见本地 UI / 命令写回入口 | 当前只确认在 memory dir 解析时读取 | source 优先级是 `policy -> flag -> local -> user`，明确不看 `projectSettings`；支持 `~/` 展开，但要求通过绝对路径与安全归一化校验；未设置时回退到 `~/.claude/projects/<sanitized-cwd>/memory/` |
| `autoDreamEnabled` | `Gx8()`、`Ro_()`、`Lj4()` auto-dream 调度器、UI toggle | auto-memory 面板 toggle -> `wA("userSettings", { autoDreamEnabled })` | 调度前按 `Gx8()` 热读取 | 它不是 auto-dream 的唯一 gate；真正运行还要同时满足非 headless/非禁用、`autoMemoryEnabled` 打开，以及最小时间/会话数阈值 |
| `sshConfigs` | schema `_D()`；另有独立 SSH session runtime `useSSHSession` | 当前未见本地 UI / 命令写回入口 | 当前已见 SSH runtime，但未见它从 settings 取值 | 当前更稳的结论不是“完全没有 SSH”，而是“bundle 里既有 `sshConfigs` schema，也有 SSH runtime；但全文搜索仍未见 `sshConfigs / sshHost / sshPort / sshIdentityFile / startDirectory` 在 schema 之外的消费链，也未见本地 `ssh` CLI 命令注册；并且 `sshSession` 只出现在 REPL 参数位与 `useSSHSession` 消费位，主启动链传给 REPL 的配置对象里并未携带它”，因此不能把它写成已接通的 SSH 配置面 |

### `sshConfigs` 当前更准确的边界

这一项现在已经不能再简单写成“只有 schema，没有任何 SSH 相关实现”。

本地 bundle 里已经能直接看到独立 SSH session runtime：

- `useSSHSession` 会创建 manager
- 支持消息收发、permission request、interrupt
- 支持掉线重连
- 会回看 SSH stderr tail 并拼进失败消息

但另一边，针对 settings schema 的全文搜索当前仍然只命中：

- `sshConfigs`
- `sshHost`
- `sshPort`
- `sshIdentityFile`
- `startDirectory`

的 schema 定义与描述文案，没有看到：

- 从 effective settings 读取这些字段并构造 SSH session 参数
- 本地 `claude ssh ...` 命令注册
- `sshConfigs` 与 remote environment picker / teleport / bridge 创建链的直接接线

而且现在还能补上一条更强的负证：

- `sshSession` 这个运行时对象名，当前只命中两处
  - `f3A({ ..., sshSession: E, ... })` 的 REPL 参数位
  - `VF4({ session: E, ... })` 的 `useSSHSession` 消费位
- `createManager`、`getStderrTail`、`proxy.stop` 这三个 SSH runtime 特征点在全文也都各只命中一次，而且全部落在同一条 `useSSHSession` 消费链上，没有看到第二套 SSH manager/runtime
- `claude ssh <config> [dir]` 这段文案在全文只命中一次，而且仅出现在 `sshConfigs` schema 描述里
- CLI 入口的 argv 快速分流只显式识别 `remote-control | rc | remote | sync | bridge`
- Commander 命令树里当前也未见 `command("ssh")` 一类子命令注册
- 启动主链真正传给 REPL 的配置对象 `lq` 里，当前只看到：
  - `debug`
  - `commands`
  - `initialTools`
  - `mcpClients`
  - `autoConnectIdeFlag`
  - `mainThreadAgentDefinition`
  - `disableSlashCommands`
  - `dynamicMcpConfig`
  - `strictMcpConfig`
  - `systemPrompt`
  - `appendSystemPrompt`
  - `taskListId`
  - `thinkingConfig`
  - 可选 `onTurnComplete`
- 也就是说，当前本地启动主链并不会把任何 SSH session object 注入 REPL

因此就当前 bundle 内可见代码来说，还可以再收紧成：

- 没看到第二个 SSH runtime 实现
- 没看到隐藏 `ssh` fast path
- 没看到本地 `ssh` 子命令或别名命令注册
- 剩余不确定性更像 bundle 外能力、灰度入口或服务端配套，而不是本地 bundle 内还有另一条可见主链

所以当前最稳的写法应是：

- SSH runtime 存在
- `sshConfigs` schema 也存在
- 但这份 bundle 内，至少主启动链仍未见两者打通
- 当前真正已接通的 remote 入口仍是 `--remote` / `--teleport` / `remote.defaultEnvironmentId`，不是 `sshConfigs`

## 当前仍未完全钉死的点

- remote managed settings 与 policy limits 的本地链已经基本闭环，但服务端是否还会对 payload 做二次裁剪、按账号形态下发不同最终态，本地 bundle 不能完全证明。
- 启动迁移主链已经能还原到函数级矩阵，但仍有少量外围 feature-gate / offer 细枝末节没有必要在本页继续铺平。
- `sshConfigs` 现在已经能比“schema 与 SSH runtime 同时存在”更进一步：当前 bundle 的主启动链没有把 `sshSession` 注入 REPL，因此至少本地主链并未接线。剩余不确定性主要只在 bundle 外能力、灰度入口或未命中的旁路，而不是 settings 主 merge / 写回链本身。

## 当前可作为重写输入的结论

如果重写配置子系统，当前已经足够稳定的骨架是：

```ts
type SettingsSource =
  | "userSettings"
  | "projectSettings"
  | "localSettings"
  | "flagSettings"
  | "policySettings"

type LoadedSettings = {
  effective: Settings
  sources: Array<{ source: SettingsSource; settings: Partial<Settings> }>
  errors: SettingsError[]
}

loadSettings():
  pluginOverlay
  -> load enabled source set
  -> always append policy + flag
  -> validate each source
  -> merge with array dedupe
  -> cache effective + per-source
  -> expose notify/invalidate hooks
```

这已经不是“猜一个 settings.json 格式”的层级，而是能支撑独立模块化重写的程度。

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
