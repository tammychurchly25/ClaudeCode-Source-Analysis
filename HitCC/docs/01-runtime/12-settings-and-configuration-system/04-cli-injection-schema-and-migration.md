# 配置与用户设置系统：CLI 注入、schema 与迁移兼容层

## CLI 注入与运行时注入

### `--settings`

`--settings` 不是单纯接收文件路径。  
当前支持两种输入：

1. 文件路径
2. inline JSON 字符串

若是 inline JSON：

- 启动时先校验 JSON 语法
- 再落到临时 `.json` 文件
- 把该临时文件当作 `flagSettingsPath`

因此 `flagSettings` 的文件来源并不一定是用户已有文件。

### `--setting-sources`

如前所述，它只决定基础磁盘 source 是否纳入：

- user
- project
- local

不会移除：

- `policySettings`
- `flagSettings`

### 运行时内存 overlay：`apply_flag_settings`

除了 CLI 启动参数，settings 还可以在运行期收到 `apply_flag_settings` 请求。  
这条路径会：

1. 读取当前 inline flag settings
2. 与新 patch 做对象 merge
3. 写回 `flagSettingsInline`
4. 触发 `qX.notifyChange("flagSettings")`

因此 `flagSettings` 不是纯启动期概念，而是一个可热更新的 source。

### settings 变更后的运行时反应不是统一“立即生效”

高价值链路当前可以收成下表：

| 触发路径 | 立即动作 | 后续链路 | 关键边界 |
| --- | --- | --- | --- |
| `wA(user/project/local, patch)` | 直接写盘并调用 `BX()` 清 settings cache | 后续通常还要靠 `S8z()` watcher 检到文件变化，再由 `Cu8(source)` 发事件 | `wA()` 自己不会统一调用 `qX.notifyChange(...)`；很多 UI 会同步改本地 state，避免等 watcher |
| 文件 watcher：`S8z()`；MDM/注册表轮询：`u8z()` | `Cu8(source)` -> `BX()` + `Ye1.emit(source)` | `RY6()` 订阅后调用 `XU8()`，刷新 `appState.settings`、重建 `toolPermissionContext`；`Tm8()` 也会刷新 settings error 提示 | source 粒度保留到 `user/project/local/policy`；drop-ins 会统一归到 `policySettings` |
| `appState.settings` 变化后的全局观察器：`ja(newState, oldState)` | 比较新旧 `settings` | 若 `settings.env` 变化，则调用 `SQ()` 重刷 `process.env` 与相关缓存 | 不是每个 key 都触发运行态副作用；`SQ()` 主要对应 env 侧 |
| remote managed settings：`yBq()` / `eL8()` / `BJ_()` | 显式 `qX.notifyChange("policySettings")` | 走与 watcher 相同的 subscriber 链；另外 `q4_()` 还会二次比较 plugin-affecting snapshot，必要时重载 plugin hooks | 当前至少可以确认 policy/managed 侧变动会触发条件式 plugin hook reload |
| 运行期 `apply_flag_settings` | 更新 inline flag overlay，并 `qX.notifyChange("flagSettings")` | headless / SDK 路径会跟着刷新 `appState.settings`；remote/headless 侧还会重算由 settings 派生的 fast mode 态 | `flagSettings` 不是只在启动读一次 |
| `/voice` toggle | `wA("userSettings", { voiceEnabled })` 后立即 `qX.notifyChange("userSettings")` | 语音相关 UI/运行态无需等待磁盘 watch | 这是少数明确“写盘 + 显式 notify”的用户入口 |

## schema 轮廓

### schema 是统一的大模型，不是分散小片段

settings schema `_D()` 当前覆盖范围很大。  
至少已经能直接确认的主分组包括：

- auth/provider
  - `apiKeyHelper`
  - AWS/GCP/XAA 相关 auth helper
- runtime/env
  - `env`
  - `language`
  - `defaultShell`
  - `outputStyle`
- persistence
  - `cleanupPeriodDays`
  - `plansDirectory`
  - `autoMemoryEnabled`
  - `autoMemoryDirectory`
  - `autoDreamEnabled`
- model
  - `model`
  - `availableModels`
  - `modelOverrides`
  - `advisorModel`
  - `effortLevel`
  - `fastMode`
- permissions/sandbox
  - `permissions.*`
  - `sandbox.*`
  - `skipDangerousModePermissionPrompt`
  - `skipAutoPermissionPrompt`
  - `useAutoModeDuringPlan`
  - `autoMode.*`
- hooks / MCP / plugins
  - `hooks`
  - `allowedMcpServers`
  - `deniedMcpServers`
  - `enabledPlugins`
  - `extraKnownMarketplaces`
  - `strictKnownMarketplaces`
  - `blockedMarketplaces`
  - `pluginConfigs`
  - `strictPluginOnlyCustomization`
- remote / channels / ssh
  - `remote.defaultEnvironmentId`
  - `channelsEnabled`
  - `allowedChannelPlugins`
  - `sshConfigs`
- UI/UX
  - `spinnerTipsEnabled`
  - `spinnerVerbs`
  - `spinnerTipsOverride`
  - `syntaxHighlightingDisabled`
  - `prefersReducedMotion`
  - `defaultView`

因此 settings system 已经承担：

- core runtime config
- enterprise policy
- plugin/marketplace control plane
- 一部分 UI preference

而不是“零散偏好项容器”。

## 迁移与兼容层

### 不是 schema version 升级器，而是“启动迁移 + 宽容 schema”

当前本地实现里还没有看到单独的 `settingsVersion` 字段，也没有一套“按版本号升级 settings 文件”的统一 migration runner。  
更准确的还原应是：

- 一部分历史偏好保存在 app state / 旧存储里
- 启动时执行若干一次性迁移
- 迁移结果再写回 `userSettings` 或 `localSettings`
- schema 本身再保留少量 deprecated / forward-compatible 入口

所以这里的兼容层不是“文件格式版本演进器”，而更像“产品历史包袱清理层”。

### 已确认的启动迁移

启动链当前已确认会按固定顺序调用：

```text
dr4 -> lr4 -> nr4 -> Oo4 -> Ko4 -> ar4 -> zo4 -> wo4 -> tr4 -> Ao4
```

其中 settings 相关迁移可以整理成下表：

| 迁移 | 触发条件 | 写入位置 | 清理/哨兵 | 当前可确认语义 |
| --- | --- | --- | --- | --- |
| `dr4()` | `appState.autoUpdates === false`，且 `autoUpdatesProtectedForNative !== true` | `userSettings.env.DISABLE_AUTOUPDATER = "1"` | 删除 app state 里的 `autoUpdates / autoUpdatesProtectedForNative`；同时立刻同步 `process.env.DISABLE_AUTOUPDATER` | 把旧 app state 的“关闭自动更新”迁成 settings/env 语义 |
| `lr4()` | `appState.bypassPermissionsModeAccepted === true` | 若 `Gf6()` 还没命中任何来源的 `skipDangerousModePermissionPrompt`，则写 `userSettings.skipDangerousModePermissionPrompt = true` | 删除旧 app state 字段 | 旧 bypass acceptance 迁入新版 dangerous-mode opt-in |
| `nr4()` | 旧 MCP 审批字段存在：`enableAllProjectMcpServers / enabledMcpjsonServers / disabledMcpjsonServers` | `localSettings` | 从旧存储移除上述 3 个字段 | `enableAllProjectMcpServers` 只在 local 尚未显式设置时补入；数组字段会与现有 local 值去重合并 |
| `Oo4()` | `opusProMigrationComplete` 尚未完成 | app state | 写 `opusProMigrationComplete`；在 `firstParty + pro` 且当前没有自定义 model 时，再写 `opusProMigrationTimestamp` | 当前不是“强制改写 settings.model”，而是为新的默认模型语义留下 sentinel/timestamp |
| `Ko4()` | `sonnet1m45MigrationComplete` 尚未完成 | `userSettings.model`；运行态 `mainLoopModelOverride`；app state | 写 `sonnet1m45MigrationComplete` | 只把 `sonnet[1m]` 从旧 4.5 显式标识迁到 `sonnet-4-5-20250929[1m]`，并同步运行态 override |
| `ar4()` | `firstParty` 且 `s28()` remap gate 开启，`userSettings.model` 命中旧 `claude-opus-*` 标识 | `userSettings.model = "opus"` | 记录 `legacyOpusMigrationTimestamp` | 旧的显式 Opus 型号名折叠成新的 alias |
| `zo4()` | `firstParty`，且账户档位命中 `pro / max / 特定 team tier`，`userSettings.model` 命中旧 `claude-sonnet-4-5-*` 或 `sonnet-4-5-*` | `userSettings.model = "sonnet"` 或 `"sonnet[1m]"` | `numStartups > 1` 时记录 `sonnet45To46MigrationTimestamp` | 把 Sonnet 4.5 的显式型号迁成新的 Sonnet 4.6 alias 族 |
| `wo4()` | `uH()` 为真，且 `userSettings.model === "opus"` | `userSettings.model = "opus[1m]"`，或在它已等于当前默认模型时删掉显式值 | 无单独哨兵 | 这条迁移的目标不是“永远保留显式值”，而是尽量把旧 `opus` 对齐到新的 1M 默认语义；若显式值已与默认等价，就直接删 key |
| `tr4()` | app state 中存在 `replBridgeEnabled`，且还没有 `remoteControlAtStartup` | app state | 把 `replBridgeEnabled` 折叠写入 `remoteControlAtStartup = Boolean(replBridgeEnabled)`，再删除旧字段 | 这是 app state / global 偏好字段迁移，不是写入 user/project settings 文件，但也不只是单纯删除旧键 |
| `Ao4()` | `hasResetAutoModeOptInForDefaultOffer` 尚未完成，且 `C68() === "enabled"` | 可能删除 `userSettings.skipAutoPermissionPrompt` | 写 `hasResetAutoModeOptInForDefaultOffer` | 只在用户之前接受过 auto opt-in、但 `permissions.defaultMode !== "auto"` 时清掉旧 opt-in，避免默认 offer 切换后把历史接受态误当成当前意图 |

因此 settings migration 的真实图景是：

- 一部分迁移写 settings 文件
- 一部分迁 app state
- 两者在启动阶段一起完成
- 同一轮启动里还会顺手改运行态 override 或 `process.env`

这里还有几个 guard 语义现在也已经足够稳定：

- `LI()` -> `subscriptionType === "pro"`
- `XR()` -> `subscriptionType === "max"`
- `Ce()` -> `subscriptionType === "team"` 且 `rateLimitTier === "default_claude_max_5x"`
- `C68()` -> 远端 `tengu_auto_mode_config.enabled`，当前枚举是 `enabled / disabled / opt-in`
- `Gf6()` -> 任一 settings source 已经显式接受 dangerous mode prompt
- `s28()` -> legacy model remap gate

### schema 自身的兼容入口

除了启动迁移，schema 还保留了几条更“软”的兼容手段：

- `includeCoAuthoredBy` 仍保留在 schema 中，但已标记为 deprecated，推荐改用 `attribution`
- `strictPluginOnlyCustomization` 先做 preprocess，再用 `.catch(void 0)` 吞掉不兼容值
- `effortLevel` 也用 `.catch(void 0)` 丢弃无法识别的持久化值

其中 `strictPluginOnlyCustomization` 的兼容语义还多一层：

- schema 会过滤掉当前客户端不认识的 surface
- 安装/诊断链会额外给出“被静默忽略”的提醒

这说明兼容策略偏向：

- 尽量继续启动
- 尽量忽略未知值
- 用 warning 替代 hard fail

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
