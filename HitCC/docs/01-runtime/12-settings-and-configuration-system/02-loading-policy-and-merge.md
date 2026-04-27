# 配置与用户设置系统：读取、远端托管与 merge

## 读取、校验与 merge

### 单文件读取：`ye(path)`

单个 settings 文件的读取链已经比较清楚：

1. 先 resolve 实际路径
2. 读取原始文本
3. 空文件视为空对象
4. JSON parse
5. 经过统一 settings schema `_D()`
6. 返回 `{ settings, errors }`

几个明确边界：

- 文件不存在或坏 symlink 不会直接炸整个 CLI，会走错误记录路径
- schema 校验失败时，该文件不会变成有效 settings，但错误会被收集
- 这条链是所有磁盘 settings source 的共同底层

### `C28(...)` 是 permission rules 的预清洗层

`ye(path)` 和平台托管分支下的 `Tj1(...)` 在正式跑 `_D()` 之前，都会先调用 `C28(...)`。  
它当前不是通用 migration，也不是第二套 schema，而是一个很窄的“permission rules 宽容清洗层”：

- 只处理 `permissions.allow / deny / ask`
- 数组里非字符串项会被直接移除，并记录错误
- 字符串 permission rule 会先走 `qj1(...)` 语法检查
- 无效 rule 会被跳过，并把解析错误/建议拼进错误消息

因此更准确的语义是：

- 配置文件可以在 permission rules 层“部分坏掉但继续加载”
- 它会尽量保住剩余合法 rule
- 它不会修复其他字段，也不会把修复结果写回原文件

### effective settings 不是现读现算，而是统一汇总

`PZ3()` 是当前更稳的“从磁盘和运行态来源组装 effective settings”的主装配器。  
它会：

1. 先并入 cached plugin settings overlay
2. 再按 `nc()` 给出的有效 source 集逐个加载
3. 为每个 source 收集 schema/读取错误
4. 做最终 merge

最终得到：

- `effective settings`
- `errors[]`

并写入全局缓存。

### `policySettings` 有独立 fallback 链

`policySettings` 不是走普通 `m2(source) -> ye(path)` 直线读取。  
当前更稳的顺序应写成：

```text
policySettings
  -> remote-settings.json 缓存
  -> 平台 machine-managed 配置
  -> managed-settings.json + managed-settings.d/*.json
  -> Windows HKCU user fallback
```

这里有几个当前已经可以写死的边界：

- `remote-settings.json` 只有在 remote managed settings 模式被判定开启后才会参与
- 平台 machine-managed 配置优先于本地 `managed-settings.json` 文件族
- `managed-settings.json` 与 drop-ins 不是互斥，而是按文件名排序后 merge
- `HKCU` 只作为 Windows 用户级 fallback，不是与 `HKLM` 并列同优先级 source

### remote managed settings 的完整链不是“读缓存”，而是独立的远端配置通道

`policySettings` 里的 remote managed settings 当前已经可以还原成一条单独的时序链：

```text
Tu() 判定是否启用 remote managed settings
  -> 读取 <appRoot>/remote-settings.json 作为当前缓存
  -> 把缓存内容做 canonical sort + sha256，生成 checksum
  -> GET /api/claude_code/settings
       If-None-Match: "<checksum>"
  -> 本地校验响应 schema
  -> 必要时弹安全确认
  -> 接受后写回 remote-settings.json
  -> 作为 policySettings 家族成员参与 merge
```

当前能写死的启用条件不是“只要登录就拉远端配置”，而是 `Tu()` 这组更窄的 gate：

- 必须是 `firstParty` provider
- `gP()` 必须为真
- `process.env.CLAUDE_CODE_ENTRYPOINT !== "local-agent"`
- 还要满足以下任一条件：
  - 有 OAuth access token，但 `subscriptionType === null`
  - 有 OAuth access token，且 scope 包含企业相关权限、`subscriptionType` 为 `enterprise` 或 `team`
  - 本地能取到 first-party API key

这说明 remote managed settings 更接近：

- first-party 官方运行态下的 enterprise / team / 待判定账户远端托管配置

而不是所有鉴权形态共享的一般设置接口。

### 远端请求、响应与缓存校验

当前远端 fetch 入口是：

- `GET ${BASE_API_URL}/api/claude_code/settings`

鉴权头来源有两种：

- `x-api-key`
- `Authorization: Bearer <accessToken>`，并附带 `anthropic-beta`

另外总会带：

- `User-Agent`

本地不会直接拿文件字节做 cache validator，而是先把 settings 对象 canonicalize：

- 对象 key 递归排序
- 再 `JSON.stringify`
- 再做 `sha256`
- 最终形成 `sha256:<hex>`

这个值会作为本地缓存版本，并写进：

- `If-None-Match: "<checksum>"`

因此这里更准确是：

- 客户端自算 checksum，借 `If-None-Match` 做远端协商缓存

而不是：

- 直接把服务端原始 ETag 原封不动持久化

远端返回的本地期望 schema 当前可直接写死为：

- `uuid: string`
- `checksum: string`
- `settings: Record<string, unknown>`

随后还会继续过两层本地校验：

1. `HBq()` 校验远端包装结构
2. `_D().safeParse(...)` 校验 settings 本体

所以服务端即使返回了 JSON，也不代表会直接进 effective settings。

### 状态语义、失败退化与 stale cache

remote managed settings 的状态语义现在已经比较完整：

- `304`
  - 直接保留缓存
  - 记为 `Cache still valid`
- `204` 或 `404`
  - 视为“远端没有 settings”
  - 本地结果退化为空对象
  - 并删除 `remote-settings.json`
- `200`
  - 校验通过后进入应用链
- auth 错误
  - 直接 `skipRetry`
- timeout / network / 其他错误
  - 进入重试

重试不是无限循环。  
`xJ_()` 当前会在初次请求之外，再按退避策略最多补 5 次重试；若仍失败：

- 有旧缓存 -> 使用 stale cache
- 无旧缓存 -> 返回 `null`

因此当前真实语义是：

- remote managed settings 是 best-effort
- 但只要本地已有缓存，就明显偏向“继续用旧值启动”

### 用户确认不是对所有远端改动都弹框

`vBq(...)` 当前不是“远端 settings 一变就弹确认框”。  
只有同时满足以下条件才会要求用户确认：

1. 新 settings 里包含高风险子集
2. 该高风险子集相对旧缓存发生了变化
3. 当前环境允许弹交互确认 UI

这里的“高风险子集”目前已能直接写成：

- shell 相关设置
- `env` 中不在内建 allowlist 里的环境变量
- `hooks`

也就是说，若远端变化只涉及普通布尔项、枚举项、模型偏好等非高风险字段：

- 不会触发这条安全确认链

而若用户拒绝：

- 新远端 settings 不会写入 `remote-settings.json`
- 当前进程继续沿用旧缓存

因此确认框的真实语义更接近：

- 是否接受远端下发的 shell/env/hook 级安全敏感改动

而不是：

- 是否接受任意 enterprise policy 改动

### 启动 gating、后台轮询与 auth change

remote managed settings 还有一条容易漏掉的“加载 promise”链：

- `EBq()` 创建 `C$6`
- `TC` 是 resolver
- 30 秒后若仍未完成，会强制 resolve，避免启动无限阻塞
- `tL8()` 只是等待这条 promise

启动期的主顺序当前可写成：

```text
preAction
  -> smz() 启动迁移
  -> yBq() 刷 remote managed settings
  -> gF1() 刷 policy limits
```

其中 `yBq()` 有两个关键边界：

- 若启动时已经有 `remote-settings.json` 缓存，会先提前 resolve loading promise，避免外层长时间阻塞
- 刷新完成后若拿到 settings，会发 `qX.notifyChange("policySettings")`

后台刷新也不是只在启动做一次：

- `pJ_()` 会启动 1 小时轮询
- `BJ_()` 每轮重新请求远端 settings，并与当前缓存比较
- 若序列化结果不同，就触发 `policySettings` 变更

auth 变化时还会走单独刷新链：

- `bF1()` 清 remote settings cache、loading promise 和本地缓存文件
- `eL8()` 再重新拉取
- 完成后发 `policySettings` change

因此 remote managed settings 既有：

- 启动期 gating
- 运行期后台 poll
- auth change 强制刷新

而不是单次冷启动读取。

### `policySettings` 与 `policy limits` 是两条不同的远端 enterprise 通道

这一点非常容易混淆，但当前已经能明确拆开：

- remote managed settings
  - endpoint: `/api/claude_code/settings`
  - 本地缓存: `<appRoot>/remote-settings.json`
  - 产物进入 `policySettings`
  - 最终参与 settings merge
- policy limits
  - endpoint: `/api/claude_code/policy_limits`
  - 本地缓存: `<appRoot>/policy-limits.json`
  - 产物是 `restrictions`
  - 通过 `isPolicyAllowed(...)` 直接做 capability gate

所以像这些能力开关：

- `allow_remote_control`
- `allow_product_feedback`

当前更可能来自：

- policy limits

而不是：

- `policySettings` merge 后的普通 settings key

这说明 enterprise 远端控制面至少分成两类：

- 进入 settings merge 的 remote managed settings
- 不进入 settings merge、直接做 allow/deny gate 的 policy limits

### remote managed settings 与 policy limits 的资格条件并不相同

这两条远端链虽然都挂在 first-party enterprise 控制面下，但本地 eligibility gate 并不完全一样。

`remote managed settings` 走 `Tu()`，当前至少要求：

- `provider === firstParty`
- `gP()` 为真
- `process.env.CLAUDE_CODE_ENTRYPOINT !== "local-agent"`
- 再满足以下任一条件：
  - OAuth token 存在，且 `subscriptionType === null`
  - OAuth token 存在，且 scope 含 `Gh`，并且 `subscriptionType` 是 `enterprise / team`
  - 可取到 first-party API key

`policy limits` 走 `ku()`，当前至少要求：

- `provider === firstParty`
- `gP()` 为真
- 再满足以下任一条件：
  - 可取到 first-party API key
  - OAuth token 存在，且 scope 含 `Gh`，并且 `subscriptionType` 是 `enterprise / team`

这带来一个很关键的差异：

- `subscriptionType === null` 的 OAuth 账户，可以进入 remote managed settings
- 但同一账户在本地证据下不会进入 policy limits

另外，两条 endpoint 的“空结果”语义也不完全相同：

- remote managed settings：接受 `200 / 204 / 304 / 404`
- policy limits：接受 `200 / 304 / 404`

其中 remote settings 的 `204/404` 都会被本地视作“远端没有 settings”；policy limits 的 `404` 则被视作“没有 restrictions”。

`isPolicyAllowed(...)` 的默认态也还要再收紧一点理解：

- 当本地没有 restrictions 对象时，大多数 capability 默认仍是 allow
- 但若处于 `BO()` 对应的 essential-traffic / no-telemetry 模式，`allow_product_feedback` 会被额外压成 `false`
- 当前本地没有看到 `allow_remote_control`、`allow_remote_sessions` 也走同样的 closed-default 特判

### 平台托管差异表

| 平台 | machine-managed 来源 | 文件侧来源 | 用户级 fallback | 关键边界 |
| --- | --- | --- | --- | --- |
| macOS | `/Library/Managed Preferences/<username>/com.anthropic.claudecode.plist`，找不到再看 `/Library/Managed Preferences/com.anthropic.claudecode.plist` | `/Library/Application Support/ClaudeCode/managed-settings.json` 与 `managed-settings.d/*.json` | 无独立 user fallback | plist 命中后直接作为 machine-managed 结果 |
| Windows | `HKLM\SOFTWARE\Policies\ClaudeCode\Settings` | `C:\Program Files\ClaudeCode\managed-settings.json` 与 `managed-settings.d/*.json` | `HKCU\SOFTWARE\Policies\ClaudeCode\Settings` | 只在本地 managed file family 为空时才看 `HKCU` |
| Linux / other | 无额外 OS-managed 入口 | `/etc/claude-code/managed-settings.json` 与 `managed-settings.d/*.json` | 无 | `policySettings` 主要来自文件族与 remote cache |

其中 Windows 这条“只在文件族为空时才看 `HKCU`”很关键。  
`HZ3()` 会先检查 `managed-settings.json` 和 drop-ins 里是否已经存在非空对象；若存在，就直接禁止 `HKCU` fallback 继续生效。

### merge 不是纯覆盖

核心 merge helper 是 `ic(...)`，并带自定义数组合并策略 `A96(...)`。

当前已确认的规则：

- 普通对象字段按层 merge
- 数组不是后者整体覆盖，而是去重合并

因此像这些 settings 会天然跨 source 累积：

- permission rules 数组
- `enabledPlugins` 里的对象项
- 各类 allow/deny list
- `autoMode` 下的规则数组

但也要注意：

- 不是所有逻辑上的“最终效果”都等于简单 merge 后直接使用
- 某些子系统会在消费前再做 managed-only 过滤或二次规整

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
