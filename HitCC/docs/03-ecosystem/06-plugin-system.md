# Plugin 系统深挖

## 本页用途

- 单独整理 plugin 的 marketplace、安装缓存、启停状态、运行时装配与对 skills/MCP/hooks/LSP 的注入边界。
- 把 plugin 从导航页里拆出来，避免继续和 MCP/skills/Plan/TUI 混写。

## 相关文件

- [04-mcp-system.md](./04-mcp-system.md)
- [05-skill-system.md](./05-skill-system.md)
- [../01-runtime/01-product-cli-and-modes.md](../01-runtime/01-product-cli-and-modes.md)
- [../02-execution/02-instruction-discovery-and-rules.md](../02-execution/02-instruction-discovery-and-rules.md)

## 一句话结论

plugin 在 Claude Code CLI 中不是“附带几条 skill 的目录”，而是贯穿 **marketplace 分发、安装缓存、settings 启停、运行时装配、能力注入、策略管控** 的扩展控制面。

---

## 结构总览

当前更稳的 plugin 分层应写成：

```text
CLI / management layer
  -> plugin validate/list/install/uninstall/enable/disable/update
  -> plugin marketplace add/list/remove/update

marketplace / distribution layer
  -> known_marketplaces.json
  -> marketplace cache / refresh / seed / official fallback
  -> plugin entry source resolution

installation / state layer
  -> installed_plugins.json
  -> enabledPlugins across managed/user/project/local/flag
  -> versioned cache / zip cache / plugin data dir

runtime materialization layer
  -> session --plugin-dir
  -> marketplace plugins
  -> builtin plugins
  -> dependency demotion / managed blocking / precedence merge

capability injection layer
  -> commands / skills / agents / output styles / hooks
  -> mcpServers / lspServers / settings / channels / userConfig
```

## 命令面已经能单独成系统

`plugin` 命令树当前已能直接确认包括：

- `validate`
- `list`
- `install`
- `uninstall` / `remove` / `rm`
- `enable`
- `disable`
- `update`
- `marketplace add`
- `marketplace list`
- `marketplace remove`
- `marketplace update`

其中有几个边界很关键：

- `validate <path>` 明确是校验 **plugin 或 marketplace manifest**
- `list --available --json` 会同时列出“已安装”和“marketplace 中可安装”的插件
- `uninstall --keep-data` 说明 plugin 还有独立持久化数据目录
- `marketplace` 子树证明 plugin 的正式分发单位不是单个 repo，而是 **marketplace**

因此 CLI 对 plugin 的定位不是“扫描本地扩展目录”，而是**内建完整安装与分发管理面**。

## plugin 至少有三层状态，而不是一张表

当前本地 bundle 已能把 plugin 状态拆成三层：

### 1. marketplace 声明层

- `known_marketplaces.json`
- `marketplaces/<name>.json` 或 marketplace 目录缓存
- seed marketplace / official marketplace / settings-sourced marketplace

这层解决的是：

- 从哪里发现 plugin
- 每个 marketplace 的 source 是什么
- 本地 cache 安装位置在哪里
- 是否允许 autoUpdate

### 2. 安装记录层

- `installed_plugins.json`
- 记录 `pluginId -> [installations...]`
- 每条 installation 至少带：
  - `scope`
  - `installPath`
  - `version`
  - `installedAt`
  - `lastUpdated`
  - `gitCommitSha`
  - `projectPath`（非 user scope 时）

这层解决的是：

- 哪些 plugin 已经物化到本地磁盘
- 不同 scope 是否有各自安装副本
- 当前缓存路径和版本是什么

### 3. 启停选择层

- `settings.json` 的 `enabledPlugins`
- 来源至少包括：
  - `policySettings`
  - `userSettings`
  - `projectSettings`
  - `localSettings`
  - `flagSettings`

这层解决的是：

- 哪些已安装 plugin 当前应视为 enabled
- scope 优先级如何覆盖
- builtin plugin 是否被显式关闭

因此更稳的理解不是“plugin 安装后就生效”，而是：

```text
marketplace declaration
-> installation ledger
-> enabledPlugins decision
-> runtime load
```

## runtime 真实入口：`AM / mD / bJ4 / SJ4`

plugin 运行时装配主链已经比较清楚：

- `SJ4({ cacheOnly })`
  - 从 `enabledPlugins + flag settings` 收集目标 plugin id
  - 先按 marketplace / policy / cache 解析每个 plugin
- `bJ4(loadMarketplacePlugins)`
  - 再并入 `--plugin-dir` session plugins
  - 再并入 builtin plugins
  - 再做 managed blocking、session override、dependency demotion
- `AM()`
  - 走 `SJ4({ cacheOnly: false })`
  - 允许下载/缓存真实 plugin
- `mD()`
  - 默认走 `SJ4({ cacheOnly: true })`
  - 只使用现有 cache
  - 若 `CLAUDE_CODE_SYNC_PLUGIN_INSTALL` 开启则直接退回 `AM()`

这说明 plugin 在启动期至少分成两种装配模式：

- **cache-only 启动路径**
- **允许同步安装的全量路径**

## source precedence：session plugin 会盖掉已安装 plugin

`qe_ / Ke_ / bJ4` 这一组当前已经能把优先级收紧为：

```text
session --plugin-dir plugins
-> installed / marketplace-backed plugins
-> builtin plugins
```

更具体地说：

- `--plugin-dir` 产物会被标成 `name@inline`
- builtin plugin 会被标成 `name@builtin`
- 若 session plugin 与已安装 plugin 同名：
  - session plugin 覆盖 marketplace-backed 版本
- 若 managed settings 锁住该插件名：
  - `--plugin-dir` 同名副本会被直接挡掉

因此 `--plugin-dir` 不是简单“额外追加”，而是**session scope overlay**。

## marketplace 不是简单仓库地址，而是正式 schema

marketplace schema 当前已能直接确认至少包括：

- `name`
- `owner`
- `plugins[]`
- `forceRemoveDeletedPlugins`
- `metadata`
- `allowCrossMarketplaceDependenciesOn`

source 类型也已比较完整：

- `github`
- `git`
- `url`
- `file`
- `directory`
- `npm`
- `settings`
- policy only:
  - `hostPattern`
  - `pathPattern`

其中几个边界很关键：

- `settings` source 不是远端拉取，而是把 settings 内联 marketplace 写成 synthetic `marketplace.json`
- `inline` 名字保留给 `--plugin-dir`
- `builtin` 名字保留给内建插件
- 一批官方 marketplace 名字被保留，并要求 source 必须来自 `anthropics/*`

因此 marketplace 不是“下载一份列表”那么简单，而是**受命名规则、source 类型、enterprise policy 共同约束的分发目录**。

## plugin manifest 比 skills/mcp 页面里的“plugin skill”要大得多

plugin manifest schema 当前已能直接确认至少覆盖：

- `name`
- `version`
- `description`
- `author`
- `homepage`
- `repository`
- `license`
- `keywords`
- `dependencies`
- `commands`
- `agents`
- `skills`
- `outputStyles`
- `hooks`
- `mcpServers`
- `lspServers`
- `channels`
- `userConfig`
- `settings`

因此 plugin 更稳的定义应是：

- **一组可被统一分发、安装、启停、配置与治理的扩展能力包**

而不是：

- skill 的目录
- MCP 的包装壳
- 单独一份 hooks 配置

## manifest 路径与兼容模式

当前读取路径至少有两层：

- 首选：`.claude-plugin/plugin.json`
- 兼容：根目录 `plugin.json`

而且 manifest 之外还存在一套默认目录约定：

- `commands/`
- `agents/`
- `skills/`
- `output-styles/`
- `hooks/hooks.json`

也就是说，plugin 即使不在 manifest 里显式列出所有组件，也可能通过默认目录被自动装载。

## marketplace entry 与 plugin.json 不是简单二选一

`CJ4(...)` 当前已经能把两者的关系拆成三种：

### 1. 只有 marketplace entry

- 允许直接由 marketplace entry 提供 `commands/agents/skills/hooks/outputStyles`
- 即使 plugin 根里没有 `.claude-plugin/plugin.json` 也能装起来

### 2. 有 plugin.json，且 marketplace entry 保持默认 / 显式 `strict: true`

- 两边的组件定义会做增量合并
- marketplace entry 可继续补充 paths / hooks / metadata

### 3. 有 plugin.json，且 marketplace entry `strict: false` 同时再声明组件

- 会被判成冲突
- 文案直接提示这是 conflicting manifests
- 报错还会明确提示：
  - 要么把 marketplace entry 改成 `strict: true`
  - 要么删除其中一侧的组件声明

因此 plugin 的真实语义不是“manifest 只有一份”，而是：

- **plugin.json 是主 manifest**
- **marketplace entry 在某些模式下还能当 overlay / fallback manifest**

## 安装与缓存不是平铺目录，而是 versioned cache

plugin 安装路径当前已能直接确认至少经过：

1. 解析 source
2. 拉取或复制到临时目录
3. 解析 manifest
4. 计算 version
5. 物化到 versioned cache
6. 写入 `installed_plugins.json`

version 的优先顺序也已经可写实：

- manifest `version`
- 调用方提供的 version
- git SHA / pre-resolved SHA
- `unknown`

本地缓存形状则至少包括：

- `~/.claude/plugins/cache/...`
- 可选 zip cache
- `npm-cache`
- plugin data dir：`~/.claude/plugins/data/{id}/`

这说明 plugin 的安装结果不是“把仓库 clone 到某个目录”，而是**带版本名、可迁移、可清理、可回退的本地缓存对象**。

## source 类型已经超出“本地目录 + GitHub”

plugin source 当前已能直接确认包括：

- 相对路径字符串
- `github`
- `url`
- `git-subdir`
- `npm`
- `pip`

但实现状态并不完全对称：

- `github / url / git-subdir / 相对路径` 已有明确装载链
- `npm` 已有安装到本地 cache 的实现
- `pip` schema 已存在，但当前直接报 `not yet supported`

因此 schema 支持面大于当前完整实现面，这点在重写时要保留。

更具体一点：

- `pip` 不是“暂时没走到”。
- `se6(...)` 在 source switch 里有明确 `case "pip"`，但分支主体直接抛：
  - `Python package plugins are not yet supported`
- UI 安装详情页仍会把 `pip` 视为 remote plugin source，与 `github/url/npm` 一样显示“组件摘要需安装后发现”

因此当前更稳的结论不是“pip 可能有隐藏实现”，而是：

- **schema / UI 已知晓 `pip`**
- **实际安装器仍未落地**

## 依赖解析不是建议，而是硬约束

plugin 依赖链当前已经闭环到安装与运行时两层：

### 安装期

- `Lt1(...)` 会先做 dependency closure 解析
- 再把整条闭包一起写入 `enabledPlugins`
- 然后按闭包顺序安装依赖

### 运行期

- `JJ4(...)` 会复检 enabled plugin 的依赖是否满足
- 若缺依赖、依赖未启用、跨 marketplace 不被允许：
  - 当前 plugin 会被 demote 成 disabled
  - 并生成 `dependency-unsatisfied` 错误

跨 marketplace 依赖还有一条硬边界：

- 只有**根 marketplace** 的 `allowCrossMarketplaceDependenciesOn` 生效
- 不存在“传递信任”

因此 plugin dependency 不是 UI 提示，而是**正式的启用前提与运行时 gate**。

## 启停规则是 scope-aware 的

plugin enable/disable/uninstall 当前不是只改一个布尔值，而是带 scope 语义：

- `user`
- `project`
- `local`
- `managed`
- `flag`
- builtin 走 `userSettings`

其中几个行为已经能直接写实：

- `enable/disable` 若不指定 scope，会自动探测已有可编辑 scope
- `uninstall` 会校验当前安装副本是否真的在目标 scope
- 若插件在 project scope 启用，用户想“只对自己关闭”，会被引导改用 local scope
- `disable --all` 是独立批量路径

因此 plugin scope 不是安装时附带标签，而是**设置层与磁盘层共同参与的状态维度**。

## plugin 会向运行时注入多类能力

基于 `RJ4 / G26 / zt1 / K68 / mF / ZA6 / bt6`，plugin 当前至少可注入：

- commands
- skills
- agents
- output styles
- hooks
- MCP servers
- LSP servers
- settings overlay

其中：

- commands / skills / agents / hooks / MCP / LSP 都有独立 loader
- `plugin settings` 会在 enabled plugin 集合上做 merge
- 若多个 plugin 覆盖同一 setting key，日志会记 override

因此 plugin 是真正的**多能力装配容器**。

## 与 skills 的接线点

plugin 对 skills 的影响当前至少分两层：

- plugin commands：进入 `G26()`
- plugin skills：进入 `zt1()`

然后在 skills 总注册表里，顺序可确认是：

```text
bundled skills
-> builtin plugin skills
-> skill dir commands
-> RC4
-> plugin commands
-> plugin skills
-> built-in local command set
```

因此 plugin 不只是“给 skill 提供额外目录”，而是直接占据 skills registry 的两个独立注入槽位。  
skills 更细执行与公告边界见 [05-skill-system.md](./05-skill-system.md)。

## 与 MCP 的接线点

plugin manifest 中的 `mcpServers`、`channels`、`userConfig` 共同构成了 plugin->MCP 的装配桥：

- plugin 可直接携带 MCP server config
- plugin 可声明 channel 绑定到某个 MCP server
- channel 注册要求该 server 来自 **plugin-sourced 且 marketplace 可识别** 的插件
- plugin 的 userConfig 会参与 `${user_config.KEY}` 替换

因此 plugin 在 MCP 侧不是“发现来源”这么简单，而是：

- **MCP server producer**
- **channel capability carrier**
- **MCP 配置 UI 的上游**

MCP 连接、resource、channel push 更细行为见 [04-mcp-system.md](./04-mcp-system.md)。

## `userConfig` 与 plugin settings 说明 plugin 自带配置面

plugin 配置当前至少有两条线：

### 1. `userConfig`

- manifest 可声明用户可配置项
- sensitive 值不会和普通值走同一持久化位置
- 非敏感值可注入 skills / agents / hooks / MCP env

### 2. `settings`

- plugin 可直接提供 allowlisted settings overlay
- enabled plugin 集合最终会把这些设置 merge 进运行态

这说明 plugin 不只是“静态能力包”，而是**带配置 schema 的扩展模块**。

## channels 证明 plugin 还承担消息入口角色

`channels` 字段当前至少能确认：

- 每个 channel 绑定某个 plugin 内的 MCP server
- 可附带 channel 级 `userConfig`
- 运行时 `channel_enable` 会验证该 server 是否真来自 marketplace plugin
- 通过验证后，channel notification 会被回注成 queued prompt

因此某些 plugin 的本质不是“给模型多一个工具”，而是**提供一条入站消息通道**。

## builtin plugin 不是普通已安装 plugin

builtin plugin 当前已有独立 sentinel：

- source：`name@builtin`
- marketplace：`builtin`

其特征至少包括：

- 来自内存里的 `ht1` 注册表
- 不走 marketplace 下载
- 不走普通 uninstall/update
- 可以通过 `enabledPlugins` 显式 enable/disable
- 还能额外产出 builtin plugin skills

从 `Rt1 / ZJ4 / WJ4` 反推，内建 registry 条目至少可带：

- `description`
- `version`
- `defaultEnabled`
- `isAvailable`
- `hooks`
- `mcpServers`
- `skills`

因此 builtin plugin 更像：

- **内建扩展包 registry**

而不是：

- 放在某个 cache 目录里的普通插件

但这里已经能把“不确定点”继续收紧：

- `ht1` 的消费侧非常清楚：
  - `Rt1()` 把条目转成 `name@builtin`
  - `ZJ4()` 从启用的 builtin 条目里再抽 `skills`
  - `WJ4(name)` 提供按名读取
- `ht1` 的初始化侧在本地 bundle 中只看到：
  - `ee6()` 里 `ht1 = new Map()`
- 对整份 bundle 继续追踪，没有找到：
  - `ht1.set(...)`
  - `ht1 = new Map([...])`
  - 其他显式 producer

因此本地发行版能正证的是：

- **builtin plugin 机制存在且消费链完整**

但还不能正证的是：

- **这一版 readable bundle 里到底注册了哪些具体 builtin plugin**
- **producer 是被裁掉了，还是这版实际上没有启用内建 plugin 条目**

## `plugin validate` 已基本闭环

`plugin validate` 现在已经可以写到输出格式和分支差异级别。

### 输入分流

- 不带参数时直接输出 usage/example 文案
- 目录输入时优先检查：
  - `.claude-plugin/marketplace.json`
  - 若不存在，再查 `.claude-plugin/plugin.json`
- 普通文件输入时：
  - 文件名是 `marketplace.json` -> marketplace 校验
  - 文件名是 `plugin.json` -> plugin 校验
  - 其他文件会先尝试按 marketplace 形状判断：
    - 若顶层存在 `plugins[]`，按 marketplace
    - 否则按 plugin

### CLI 输出样式

CLI handler `Mmz(...)` 的固定输出形状是：

```text
Validating <fileType> manifest: <filePath>

✗ Found N errors:
  • <path>: <message>

⚠ Found N warnings:
  • <path>: <message>

✓ Validation passed
```

退出码边界也已明确：

- 成功：`0`
- 校验失败：`1`
- 意外异常：`2`

### plugin manifest 校验

`w8A(...)` 当前至少会检查：

- 文件是否存在、是否可读、是否是合法 JSON
- `e36().strict()` schema
- `commands/agents/skills` 里的字符串路径是否包含 `..`
- `category/source/tags/strict/id` 这类 marketplace 字段是否误写进 `plugin.json`
  - 这类情况只报 warning
  - 文案明确说明运行时会忽略
- `name` 是否 kebab-case
  - 当前 CLI 接受
  - 但 claude.ai marketplace sync 要求 kebab-case
- 是否缺 `version/description/author`

### marketplace manifest 校验

`$8A(...)` 当前至少会检查：

- 文件存在性 / JSON 语法
- `Ve().extend({ plugins: R.array(jj1().strict()) }).strict()` schema
- 每个 plugin entry 的 `source`/`source.path` 是否包含 `..`
- 相对路径 source 的告警文案会明确提示：
  - 路径是相对 marketplace root
  - 不是相对 `marketplace.json` 自身
- marketplace 内是否存在重复 plugin name
- 本地 `./...` plugin entry 的 `version` 是否与目标 plugin 的 `.claude-plugin/plugin.json` 一致
  - 不一致只报 warning
  - 文案明确写出 install 时 **plugin.json version 优先**
- `metadata.description` 缺失 warning

### 目录内附属内容校验

若校验目标是 `.claude-plugin/plugin.json`，CLI 还会继续跑 `AG4(...)`：

- 递归检查 `skills/` 下的 `SKILL.md`
- 递归检查 `agents/`、`commands/` 下的 markdown 文件
- 单独检查 `hooks/hooks.json`

其中 markdown frontmatter 校验 `Z9z(...)` 至少覆盖：

- 是否存在 frontmatter
- YAML 是否能解析
- frontmatter 顶层是否是 mapping
- `description`
- `name`
- `allowed-tools`
- `shell`

并且文案会明确区分两类后果：

- 某些字段类型错：
  - runtime 会 silent drop
- hooks JSON 错：
  - runtime 会直接破坏整个 plugin load

因此 `plugin validate` 这一块已经不只是“能校验 schema”，而是：

- **manifest 识别**
- **plugin / marketplace 差异化检查**
- **附属 markdown / hooks 内容检查**
- **明确的 CLI 文本输出与退出码**

## `/plugin` 管理 UI 已能拆出明确状态机

本地 bundle 已经能把 `/plugin` overlay 的主状态写实成一组 view state，而不只是“有个 plugin 菜单”。

### 主列表态

管理面主列表当前会把条目混排成三类：

- plugin
- child MCP
- failed plugin / flagged plugin

并按 scope 分组显示：

- `flagged`
- `project`
- `local`
- `user`
- `enterprise`
- `managed`
- `builtin`
- `dynamic`

同时支持：

- `/` 或直接打字进入搜索
- `Space` toggle
- `Enter` 看详情
- `Esc` 返回

### 详情/子页态

当前已能直接确认的 state 包括：

- `plugin-list`
- `plugin-details`
- `plugin-options`
- `configuring-options`
- `configuring`
- `failed-plugin-details`
- `flagged-detail`
- `confirm-project-uninstall`
- `confirm-data-cleanup`
- `mcp-detail`
- `mcp-tools`
- `mcp-tool-detail`

### 已确认的交互分支

- plugin 明细页可进入：
  - enable
  - disable
  - update
  - uninstall
  - configure
- 若插件带 `userConfig`，启用后会进入 `plugin-options / configuring-options`
- 若插件的 MCP 配置需要单独走 `MCPB` 文件，则会进入 `configuring`
- 若 project scope 卸载会影响团队共享配置，会先进入 `confirm-project-uninstall`
- 若 data dir 非空，会进入 `confirm-data-cleanup`
- failed plugin 详情页提供 remove
- flagged plugin 详情页提供 dismiss
- child MCP 可以继续下钻到：
  - `mcp-detail`
  - `mcp-tools`
  - `mcp-tool-detail`

因此 `/plugin` 不是单页列表，而是：

- **一个把 plugin、plugin-sourced MCP、failed state、flagged state、config UI 串起来的 overlay 状态机**

## 安全与下架处理也是 plugin 子系统的一部分

plugin 当前还有一条明确的安全治理链：

- `blocklist.json`
- `flagged-plugins.json`
- 官方 security feed
- marketplace 的 `forceRemoveDeletedPlugins`

行为至少包括：

- 拉取官方安全消息并本地缓存
- 对被 marketplace 删除、且要求强制移除的 plugin 自动卸载
- 把已下架/已标记 plugin 记入 flagged 状态
- `/plugin` UI 中可单独展示 flagged plugin

因此 plugin 系统并不只负责“安装更多扩展”，还负责**下架、封禁、清理与用户提示**。

## 更稳的工程结论

基于当前本地 bundle，plugin 子系统已经可以收敛成：

1. 有独立命令树与 marketplace 子命令树。
2. 有 marketplace 声明层、安装记录层、enabledPlugins 选择层三层状态。
3. 有 `cacheOnly` 与全量安装两套 runtime 装配路径。
4. 有 session inline、marketplace、builtin 三类 source 与明确覆盖优先级。
5. 有 plugin manifest、marketplace entry，以及“`strict: true` 可 overlay、`strict: false` 会在双声明时冲突”的组合规则。
6. 有 versioned cache、zip cache、plugin data dir 与 installed ledger。
7. 有 dependency closure 安装与运行时 dependency demotion。
8. 能向 commands、skills、agents、hooks、MCP、LSP、settings 注入能力。
9. 自带 userConfig、channel、security/delist 这些治理层能力。

## 当前仍未完全钉死

- `ht1` 的 consumer 侧已清楚，但 producer 与具体内建条目仍无法从本地 bundle 正证。
- `/plugin` overlay 的主状态与主要交互已能写实，但个别组件内部展示细节还没必要继续穷举。
- `pip` source 的“不对称支持”已经明确，但未来版本是否会补上安装器，本地 bundle 无法判断。

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
