# 配置与用户设置系统：缓存、热刷新与写回

## 缓存与失效

### 两级缓存

settings 当前至少有两层缓存：

1. effective settings 总缓存
2. per-source settings 缓存

对应接口边界当前已可写成：

- `HYA() / JYA()` -> effective settings cache
- `MYA() / PYA()` -> per-source cache

### `BX()` 是统一失效入口

`BX()` 当前会同时清掉：

- effective settings cache
- per-source settings cache

因此它不是某个子模块的局部 helper，而是整个 settings system 的全局 invalidate。

### plugin settings 还有一层独立 cache

plugin enabled 集会先汇总出一份 plugin settings overlay，并缓存到独立槽位。  
只要 plugin affecting settings 改变，也会触发：

- plugin cache 失效
- settings cache 失效

这说明最终 effective settings 实际上是：

```text
plugin settings overlay
  + settings sources
  -> effective settings
```

而不是“plugin 是 settings 的下游消费者”。

## 变更检测与热刷新

### settings 文件不是靠重启生效

settings runtime 会启动专门的变更监听器。  
`S8z()` 当前会：

1. 收集各 source 实际存在的 settings 目录
2. 额外把 `managed-settings.d` 目录纳入 watch
3. 监听 `change / add / unlink`
4. 经过 `ConfigChange` hook 判定后触发 `BX()`
5. 再发出对应 source 的 change 事件

这里有两个容易漏掉的实现边界：

- `flagSettings` 不参与普通文件监听列表
- drop-ins 目录内任意 `*.json` 变化都会被归类为 `policySettings`

### `policySettings` 还有 MDM/注册表轮询链

平台托管配置并不完全依赖文件系统事件。  
`u8z()` 会周期性重读 machine-managed 配置快照，并比较：

- `mdm.settings`
- `hkcu.settings`

若序列化结果变化，就会：

1. 更新内存里的 `kj1 / Nj1`
2. 触发 `Cu8("policySettings")`
3. 由 `BX()` 清空 settings cache

默认轮询周期是 30 分钟。  
所以 `policySettings` 实际上有两条热刷新链：

- 文件 watch
- MDM / registry poll

## 写回语义

### 只有 3 个 source 可直接写回

`wA(source, patch)` 当前只对以下 source 真正写盘：

- `userSettings`
- `projectSettings`
- `localSettings`

对这两个 source 不做写盘：

- `policySettings`
- `flagSettings`

这点很关键，因为它把 settings source 分成了：

- 用户可编辑持久化 source
- 只读注入 source

### 写回不是“覆盖整个文件”

写回时会：

1. 先读已有 settings
2. 若已有 settings 因 schema 校验失败而不可作正式 source，仍尝试把 raw object 当作基底
3. 再把 patch merge 进去
4. `undefined` 值会删除 key
5. 数组在写回阶段按新值替换
6. 最后格式化写回 JSON

因此 `wA(...)` 的语义更接近：

- 结构化 patch merge

而不是：

- 重写整份 settings 模型

这也解释了为什么不少 UI/命令会把“改一个设置”实现成对 `wA(...)` 的小 patch。

### 写回入口不是单一 UI，而是多条入口共用 `wA(...)`

当前已经能把高价值写回入口收成一张“入口 -> key -> source”表：

| 入口 | 写入位置 | 典型 key | 关键语义 |
| --- | --- | --- | --- |
| `/advisor`：`pMz(...)` | `userSettings` | `advisorModel` | 同时更新 app state 里的 `advisorModel`；`unset/off` 会写 `undefined` 删除持久化值 |
| `/effort` 与 effort 选择对话框：`lPz()` / `iPz()` / `RQ4(...)` | `userSettings` | `effortLevel` | `auto/unset` 会删 key；若存在 `CLAUDE_CODE_EFFORT_LEVEL`，settings 只保留持久化偏好，当前 session 仍受 env 覆盖 |
| 模型选择器 + 全局状态观察器：`x6(...)` / `ja(...)` | `userSettings` | `model` | picker 先改 `mainLoopModel`；真正持久化由 `ja(newState, oldState)` 在状态变化后补写 `userSettings.model` |
| settings 面板：`J6` 条目与关闭时的 `o6()` flush | `localSettings` / `userSettings` | `spinnerTipsEnabled`、`prefersReducedMotion`、`defaultView`、`outputStyle`；`alwaysThinkingEnabled`、`fastMode`、`promptSuggestionEnabled`、`language`、`useAutoModeDuringPlan`、`permissions.defaultMode`、`autoUpdatesChannel` | 面板不是每个 item 都立即写盘；一部分先改内存快照，关闭时再批量写回两个 source |
| remote environment picker：`WR4()` | `localSettings` | `remote.defaultEnvironmentId` | 只写 local，不写 user/project；`DR4()` 读取时若环境不存在会回退到默认候选 |
| plugin options / plugin MCP user config：`Dg8()` / `Wg8()` / `cG8()` | `userSettings` + secure storage | `pluginConfigs.*` | sensitive key 会分流到 secure storage；settings 内同名明文会被 scrub |
| `.mcp.json` 审批对话框：`WL4()` / `Zl4()` | `localSettings` | `enableAllProjectMcpServers`、`enabledMcpjsonServers`、`disabledMcpjsonServers` | 这是 project-local 准入偏好，不进入 user settings |
| `mcp xaa setup/clear`：`gr4()` | `userSettings` | `xaaIdp.issuer / clientId / callbackPort` | client secret 与 id token 不进 settings，仍走 keychain / token cache |
| `/voice` | `userSettings` | `voiceEnabled` | 写盘后会显式 `qX.notifyChange("userSettings")`，不等待文件 watcher |
| install / update 命令 | `userSettings` | `autoUpdatesChannel` | 只有 `latest/stable` 路径会落持久化 channel |
| 启动迁移：`dr4 / lr4 / nr4 / ar4 / Ko4 / zo4 / wo4 / Ao4` | `userSettings` / `localSettings` | `env.DISABLE_AUTOUPDATER`、`skipDangerousModePermissionPrompt`、MCP 审批字段、`model`、`skipAutoPermissionPrompt` | migration 不是旁路逻辑，本质上也是正式写回入口 |

### 另一条写回面是 app state / 本地 store，而不是 settings 文件

并不是所有“设置项”都会走 `wA(...)`。

当前已直接看到，settings 面板里这类项主要经 `g8()` / `J()` 改 app state：

- `autoCompactEnabled`
- `verbose`
- `terminalProgressBarEnabled`
- `showTurnDuration`
- `respectGitignore`
- `copyFullResponse`
- `editorMode`
- `remoteControlAtStartup`

这说明当前产品里的“设置面板”其实同时覆盖：

- 真正的 settings source 持久化
- app state / 本地 UI 偏好

两者不能混成一类。

`nr4()` 也能说明这一点：  
它会先把旧 MCP 审批字段迁入 `localSettings`，再用 `S$()` 从旧 store 清理原字段，而不是只做 settings file patch。

### config 面板注册表本身也显式标注了 source

`My6()` 这层 descriptor registry 不是运行时临时猜测“这个设置该写哪”。  
每个条目在注册表里就已经显式带了 `source`：

- `source: "settings"`
  - `model`
  - `alwaysThinkingEnabled`
  - `permissions.defaultMode`
  - `language`
  - `voiceEnabled`
  - `autoMemoryEnabled`
  - `autoDreamEnabled`
- `source: "global"`
  - `preferredNotifChannel`
  - `autoCompactEnabled`
  - `fileCheckpointingEnabled`
  - `showTurnDuration`
  - `terminalProgressBarEnabled`
  - `todoFeatureEnabled`
  - `teammateMode`
  - `remoteControlAtStartup`

所以 config 面板的真实模型不是“一个统一 settings 表单”，而是：

- 一部分 descriptor 对应正式 settings source
- 一部分 descriptor 对应 app state / 本地 store

`J6` 运行时生成的可见条目，再按 entry type 分成几类：

- 立即写 settings
  - `spinnerTipsEnabled`
  - `prefersReducedMotion`
  - `thinkingEnabled`
  - `defaultPermissionMode`
  - `useAutoModeDuringPlan`
  - `defaultView`
- 只改 app state / global store
  - `autoCompactEnabled`
  - `notifChannel`
  - `teammateMode`
  - `remoteControlAtStartup`
  - `editorMode`
  - `fileCheckpointingEnabled`
  - `terminalProgressBarEnabled`
  - `showTurnDuration`
- 先改快照、关闭时再由 `o6()` flush
  - `outputStyle`
  - `language`
  - `autoUpdatesChannel`
  - 以及 `user/local` 两份暂存 patch
- 不做 inline 写盘，而是跳到子对话框 / managedEnum 分支
  - `theme`
  - `model`
  - `teammateDefaultModel`
  - `showExternalIncludesDialog`

这也解释了为什么 config 面板里的“看起来像设置项”的条目，真正落盘路径会明显分叉。

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
