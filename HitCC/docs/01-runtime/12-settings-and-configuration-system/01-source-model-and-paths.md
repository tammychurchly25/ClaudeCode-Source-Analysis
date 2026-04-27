# 配置与用户设置系统：source 模型与路径体系

## 总体判断

Claude Code CLI 的 settings 不是“读一个 JSON 文件”。

当前更稳的还原应是：

```text
schema-defined settings model
  -> 多 source 读入
  -> 逐 source 校验/报错
  -> 按 source 顺序 merge
  -> 缓存 effective settings + per-source settings
  -> 由 qX.notifyChange(...) 驱动运行时刷新
  -> 被 permissions / sandbox / plugins / MCP / telemetry / UI 等子系统消费
```

因此 settings system 的真实角色更接近：

- 全局运行时配置总线
- enterprise policy 入口
- CLI flag 注入面
- 一部分用户偏好的持久化层

## source 模型

### 已确认的 5 个正式 source

settings source 枚举当前已直接可写实：

- `userSettings`
- `projectSettings`
- `localSettings`
- `flagSettings`
- `policySettings`

CLI 内部还给这些 source 准备了稳定的人类可读名称：

- `userSettings` -> user / User settings
- `projectSettings` -> project / Shared project settings
- `localSettings` -> project, gitignored / Project local settings
- `flagSettings` -> cli flag / Command line arguments
- `policySettings` -> managed / Enterprise managed settings

这说明 source 不是实现细节，而是对用户可见的正式概念。

### 有效 source 集不是随便拼的

`allowedSettingSources` 的默认全集就是上述 5 个 source。  
但启动时真正参与加载的是 `nc()`，其逻辑不是“原样返回 allowedSettingSources”，而是：

```text
selected sources from allowedSettingSources
  + policySettings
  + flagSettings
```

因此更准确的语义是：

- `user/project/local` 可以被裁剪
- `policySettings` 与 `flagSettings` 会被强制并入有效 source 集

### `--setting-sources` 只裁 user/project/local

`--setting-sources` 当前只接受：

- `user`
- `project`
- `local`

若传入其他值会直接报错。  
它的作用不是决定“是否加载 policy/flag”，而只是重设 `allowedSettingSources` 里的可选基础集。

因此启动链应写成：

```text
--setting-sources=user,local
  -> allowedSettingSources = [userSettings, localSettings]
  -> effective load sources = [userSettings, localSettings, policySettings, flagSettings]
```

这是当前文档里最容易漏掉的一条边界。

## 路径体系

### 根目录

settings 相关的用户根目录仍继承应用根：

- 优先 `process.env.CLAUDE_CONFIG_DIR`
- 否则 `~/.claude`

这条根规则与 session/transcript 持久化共用同一条主根。

### enterprise policy 根目录是另一套平台根

`policySettings` 的本地 machine-managed 根目录不走 `~/.claude`，而是单独按平台分叉：

- macOS: `/Library/Application Support/ClaudeCode`
- Windows: `C:\Program Files\ClaudeCode`
- Linux / other: `/etc/claude-code`

这套根目录下面再派生：

- `managed-settings.json`
- `managed-settings.d/*.json`

而 remote managed settings 的本地缓存则仍落回用户配置根：

- `<appRoot>/remote-settings.json`

### 各 source 的主要路径

当前可以直接收敛为：

```text
userSettings
  -> <appRoot>/settings.json
  -> 某些分支下改为 <appRoot>/cowork_settings.json

projectSettings
  -> <projectRoot>/.claude/settings.json

localSettings
  -> <projectRoot>/.claude/settings.local.json

policySettings
  -> <policyRoot>/managed-settings.json 主文件
  -> <policyRoot>/managed-settings.d/*.json
  -> <appRoot>/remote-settings.json 远端托管缓存
  -> 平台托管 plist / 注册表
  -> Windows 用户级注册表 fallback

flagSettings
  -> --settings 指向的文件
  -> 或 --settings 提供的 inline JSON 落成的临时文件
  -> 再叠加运行时内存里的 inline flag overlay
```

其中几个点需要单独强调：

### `userSettings` 文件名不是永远固定

用户级默认文件名通常是：

- `settings.json`

但在 cowork 分支下会切到：

- `cowork_settings.json`

因此不能把“用户设置文件恒等于 `~/.claude/settings.json`”写死成唯一实现。

### `policySettings` 不是单文件

本地文件侧至少有两层：

- `managed-settings.json`
- managed settings drop-ins 目录下的多个 `*.json`

另外还有两类非普通文件来源：

- remote managed settings
- 平台托管配置 / 注册表

所以 `policySettings` 更准确是一个 source family，不是一份文件。

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
