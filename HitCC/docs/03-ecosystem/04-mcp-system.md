# MCP 系统深挖

## 本页用途

- 单独收拢 MCP 的配置源、命令面、运行时连接、prompt 注入、resource attachment 与 deferred tool 机制。
- 把 MCP 从总览页里拆出来，避免继续和 plugin/plan/TUI 混写。

## 相关文件

- [../01-runtime/01-product-cli-and-modes.md](../01-runtime/01-product-cli-and-modes.md)
- [../02-execution/01-tools-hooks-and-permissions.md](../02-execution/01-tools-hooks-and-permissions.md)
- [../02-execution/03-prompt-assembly-and-context-layering.md](../02-execution/03-prompt-assembly-and-context-layering.md)
- [../02-execution/05-attachments-and-context-modifiers.md](../02-execution/05-attachments-and-context-modifiers.md)

## 一句话结论

MCP 在 Claude Code CLI 中不是附属插件，而是贯穿 **CLI 配置层、连接层、工具层、resource 层、prompt 层** 的一等系统。

---

## 结构总览

当前更稳的 MCP 分层应写成：

```text
CLI config / command layer
  -> mcp add/list/get/remove/serve/add-json/add-from-claude-desktop/reset-project-choices

startup resolve / connect layer
  -> 读取多源 MCP config
  -> 过滤 enterprise policy / strict mode
  -> 去重、连接、产出 clients/tools/commands

request / tool layer
  -> mcp__server__tool 命名
  -> deferred schema + ToolSearch
  -> mcp_instructions_delta

resource / transcript layer
  -> mcp_resource attachment
  -> list/read/subscribe/unsubscribe/polling
```

## 命令面已经基本闭环

`mcp` 子命令树当前已能直接确认包括：

- `serve`
- `add`
- `add-json`
- `add-from-claude-desktop`
- `get`
- `list`
- `remove`
- `reset-project-choices`

其中：

- `add-from-claude-desktop` 明确是把桌面端配置导入 CLI
- `reset-project-choices` 明确是重置当前项目 `.mcp.json` 的已批准/已拒绝 server 选择
- `list/get` 文案明确提醒：会跳过 workspace trust dialog，并可能为了 health check 启动 `.mcp.json` 里的 stdio server

这说明 CLI 对 MCP 的定位不是“读静态配置”，而是**内建完整管理面**。

## 配置源与 scope

从启动主链和 `.mcp.json` 读写代码看，MCP config 至少分成：

- `project`：当前项目 `.mcp.json`
- `user`
- `local`
- `dynamic`
- `enterprise`
- `claudeai`

其中 project scope 的核心事实已经可以直接写实：

- 文件名就是项目根的 `.mcp.json`
- 写入采用 `tmp file -> datasync -> rename` 的原子替换方式
- 读写时会保留已有文件 mode
- project scope 的增删最终都是改写 `.mcp.json` 的 `mcpServers`

这说明 `.mcp.json` 不是只读兼容文件，而是 MCP 的正式 project-level source of truth。

## `--strict-mcp-config` 与 enterprise 约束

启动链里已经能直接确认两条硬约束：

1. 当 enterprise MCP config 存在时，不能再使用 `--strict-mcp-config`
2. 当 enterprise MCP config 存在时，不能再动态配置额外 MCP servers

更具体地说：

- `--strict-mcp-config` 的语义是“只使用 `--mcp-config` 指定的 servers，忽略其他 MCP 配置源”
- enterprise 存在时，MCP 控制权会上卷，CLI 会直接拒绝这类本地重组

因此 MCP config 不是简单 merge，而是**受 policy/enterprise gate 约束的多源合成系统**。

## 去重不是按名称，而是按“连接签名”

`Fw6(...)` 已直接暴露出 MCP server 去重的核心签名规则：

- stdio server -> `stdio:${JSON.stringify([command, ...args])}`
- URL server -> `url:${normalizedUrl}`

基于这个签名，至少存在两条去重链：

- `h3_(pluginServers, manualServers)`：压掉与手工配置重复的 plugin MCP servers
- `Zr6(claudeaiConnectors, manualServers)`：压掉与手工配置重复的 claude.ai connector

因此更稳的结论是：

- MCP server 名字不是唯一性依据
- 真正用于 suppress duplicate 的，是 **stdio command signature / normalized URL**

## 连接层：配置解析与 client/tool/command 产物

启动期当前已能直接确认：

1. 先 resolve MCP configs
2. 再把 config 分成：
   - `sdk` 类型
   - 常规 MCP server config
3. 常规 config 走 `Ao6(...)`
4. `Ao6(...)` 通过 `XN6(...)` 聚合每个 server 的：
   - `client`
   - `tools`
   - `commands`

`Ao6(...)` 的职责已经比较明确：

- 输入是一批 server configs
- 输出是 `clients[] + tools[] + commands[]`
- 某个 server 失败不会阻止整体返回
- 返回前会统计 `tools_count / commands_count / commands_metadata_length`

因此可以把 `Ao6/XN6` 理解成：

- **MCP config -> connected client set -> tool/command registry** 的主装配器

## `XC(...)`：transport / 握手 / 超时路径已经可以继续写实

`XN6(...)` 真正逐个连 server 时，核心连接函数是 `XC(name, config, stats?)`。  
当前本地 bundle 已能把 transport 分支拆到这一层：

- `sse`
  - 建 `aw6(...)` authProvider
  - 走 `nV8(new URL(url), ...)`
  - `eventSourceInit.fetch` 会补 `Authorization`、`User-Agent`、`Accept: text/event-stream`
- `http`
  - 同样走 `aw6(...)` authProvider
  - 走 `rV8(new URL(url), ...)`
  - `requestInit` 会带 `User-Agent`、代理选项、额外 headers
  - `Wp1(...)` 会为非 `GET` 请求补 `accept` 并附加超时 signal
- `ws`
  - 走 `GE8(...)`
  - headers 至少包括 `User-Agent`
  - 若存在 session auth，也会带 `Authorization`
- `sse-ide`
  - 也是 `nV8(...)`
  - 重点是把 IDE transport 所需 fetch/dispatcher 包进去
- `ws-ide`
  - 也是 `GE8(...)`
  - 会带 `X-Claude-Code-Ide-Authorization`
- `claudeai-proxy`
  - 走 `rV8(...)`
  - 请求入口会包 `ow_(fetch)`，对 `401` 做 token 变化后的重试
  - 额外带 `X-Mcp-Client-Session-Id`
- `stdio`
  - 默认走 `tu1({ command, args, env, stderr: "pipe" })`
  - 若设置 `CLAUDE_CODE_SHELL_PREFIX`，会把原 command+args 折成单条 shell 命令
- 特殊内建 stdio
  - Chrome MCP 与 Computer Use MCP 不一定起外部进程
  - 命中内建 server 名时，会直接创建 in-process server，再用 linked transport pair 对接 client

这意味着 `XN6` 自己只是并发装配器，**真正的 transport 选择、认证方式、内建/外部 server 分流都在 `XC(...)`**。

## 四个 transport 类已经能写到协议粒度

此前只知道 `nV8 / rV8 / GE8 / tu1` 是 transport 名字。  
现在可以继续收紧到“各自到底怎么收发”的程度。

### `tu1`：stdio JSON-RPC 直连

- 启动外部进程时固定：
  - `stdin: pipe`
  - `stdout: pipe`
  - `stderr: inherit | pipe | overlapped`
- 只透传一小组白名单环境变量，再叠加 server 自己的 env
- 读路径：
  - 从 `stdout` 累积到 `_readBuffer`
  - 反复 `readMessage()` 解出 MCP message
- 写路径：
  - `send()` 直接把编码后的 message 写进子进程 `stdin`
- 关停路径：
  - 先 `stdin.end()`
  - 超时再 `SIGTERM`
  - 仍不退出再 `SIGKILL`

因此 `tu1` 本质上就是：

- **stdio 上的 framed JSON-RPC transport**

### `nV8`：经典 SSE + 独立 POST endpoint

- 连接阶段先起 `EventSource`
- 服务端必须额外发一个 `event: endpoint`
  - data 里给出后续 POST 的真正 endpoint URL
- 普通 message event 的 `data` 会被当作 JSON parse 成 MCP message
- 发送阶段不走 SSE 回写：
  - 而是对 `endpoint` 做 `POST application/json`
- 若收到 `401 + www-authenticate`
  - 会解析 resource metadata / scope
  - 调 authProvider 完成授权
  - 再重试

因此 `nV8` 不是“单纯订阅 SSE”，而是：

- **SSE 下行 + 独立 HTTP POST 上行**

### `rV8`：streamable HTTP / session 化 HTTP transport

- `start()` 只建 `AbortController`
- 真正的下行流在 `_startOrAuthSse(...)` 中：
  - 对主 URL 发 `GET`
  - `Accept: text/event-stream`
  - 若有 resumption token，放进 `Last-Event-ID`
- 上行发送在 `send(...)` 中：
  - 对同一 URL 发 `POST`
  - `Accept: application/json, text/event-stream`
  - `Content-Type: application/json`
- 服务端若回 `mcp-session-id`
  - 客户端会缓存为 session 头
- 若返回 `202`
  - 对 notification 类消息会另起 SSE 继续等后续结果
- 若返回 `text/event-stream`
  - 直接把 body 交给 SSE parser
- 若返回 `application/json`
  - 直接 parse 成 MCP response
- `terminateSession()` 会对同一 URL 发 `DELETE`

重连逻辑也已明确：

- 记录 server 给的 `retry`
- 否则按：
  - 初始 `1000ms`
  - 增长因子 `1.5`
  - 最大 `30000ms`
  - 最多 `2` 次
- 会带着 `resumptionToken` 继续追流

因此 `rV8` 更准确的定义是：

- **同 URL 上的 session 化 streamable HTTP transport**

### `GE8`：WebSocket 上的 JSON message transport

- 只在 socket open 后才允许 `start()`
- 接收端把每条 websocket message 直接 `JSON.parse -> Sb.parse`
- 发送端把 message `JSON.stringify` 后直接 `ws.send`
- Bun 和 Node 的事件绑定分别处理

因此 `GE8` 没有额外 framing 逻辑，本质上就是：

- **WebSocket 直传 JSON message**

## 连接后的初始化、能力探测与清理策略

`XC(...)` 在 transport 建好后，还会继续做一组固定初始化：

- 创建 `SV8(...)` MCP client
- 安装 `ListRoots` handler，返回当前 cwd 的 `file://` root
- 按 `Xy8()` 做连接超时控制
  - 默认来自 `MCP_TIMEOUT`
  - 缺省值为 `30000ms`
- 连接成功后读取：
  - `getServerCapabilities()`
  - `getServerVersion()`
  - `getInstructions()`
- `instructions` 长度过大时会截断到本地上限再放进运行态

清理路径也已经相当具体：

- `stdio` server 清理会尝试：
  - `SIGINT`
  - 不退出再 `SIGTERM`
  - 仍不退出再 `SIGKILL`
- in-process server 会先关内建 server，再关 client
- 连接关闭时会清：
  - `iy.cache`
  - `Un.cache`
  - `z$6.cache`
  - `XC.cache`

因此 MCP client 不是一次性连上就算了，而是带 **超时、能力探测、缓存失效、分 transport cleanup** 的完整生命周期对象。

## 错误还原不是一刀切

`XC(...)` 的错误还原逻辑也可以继续收紧：

- `sse/http/claudeai-proxy` 若命中 needs-auth 条件，会返回 `type: "needs-auth"`
- `http/claudeai-proxy` 若服务端回的是 session-not-found 风格 `404`，会认定为 session expired
- `sse/http/claudeai-proxy` 对以下 terminal error 会累计计数：
  - `ECONNRESET`
  - `ETIMEDOUT`
  - `EPIPE`
  - `EHOSTUNREACH`
  - `ECONNREFUSED`
  - `Body Timeout Error`
  - `terminated`
  - `SSE stream disconnected`
  - `Failed to reconnect SSE stream`
- 连续达到阈值后会主动关闭 transport
- `onclose` 会清缓存，为下一次重连留出干净状态

所以当前更稳的理解不是“连失败就 failed”，而是：

- 先区分 `needs-auth / session-expired / terminal transport errors / clean close`
- 再决定缓存复用还是强制重建

## MCP prompt command 也是正式产物

`Ao6/XN6` 输出的并不只有 tools。

`z$6(connectedClient)` 已能直接确认：

- 只有 `capabilities.prompts` 存在时才会请求 `prompts/list`
- 每个 prompt 会被包装成 `type: "prompt"` 命令对象
- `source: "mcp"`
- 名字规范是：

```text
mcp__<server>__<prompt>
```

- `getPromptForCommand(args, ctx)` 最终会调用 `client.getPrompt(...)`

这说明 MCP server 不只向 CLI 暴露 tool，还能暴露 **prompt command**，并且这些对象后续会进入 skills/command 侧的执行链。

## `mcp__server__tool`：命名与权限规则

MCP tool 的规范命名现在可以直接写成：

```text
mcp__<serverName>__<toolName>
```

辅助函数边界也已比较清楚：

- `Ov(name)`：把 MCP tool name 解析成 `{ serverName, toolName? }`
- `t21(tool)`：若带 `mcpInfo`，输出规范化 MCP tool 名

权限规则方面有一条很关键的限制：

- MCP permission rule **不支持** 普通工具那种括号 pattern 写法
- 合法形式更接近：
  - `mcp__server`
  - `mcp__server__*`
  - `mcp__server__tool`

这说明 MCP 工具在权限系统里是**特殊语法族**，而不是普通 tool rule 的平移版本。

## deferred MCP tool：默认只先暴露名字

当前更稳的结论是：

- MCP tools 默认就是 deferred 候选
- 初始不会全量把 schema 注入模型
- 先通过 `deferred_tools_delta` 告诉模型有哪些名字
- 需要时由 `ToolSearch` 用 `select:mcp__server__tool` 触发 schema 装载

这里的关键区别必须保留：

```text
conversation attachment
  -> deferred_tools_delta
  -> 只是名字列表

request tools array
  -> 真正 callable 的 schema
  -> 由 query builder 根据历史 tool_reference 再注入
```

因此：

- 看到 `mcp__...` 名字，不等于 schema 已经在 request body 里
- MCP 与 deferred tools 机制是深度耦合的

## MCP instructions：有 system section，也有 delta attachment

MCP 指令当前至少存在两种落点：

### 1. 默认 system section

`$X(...)` 会通过 `C8_(mcpClients)` 注入 MCP instructions section。  
其文本来源是 `U8_(clients)`，只收集：

- `type === "connected"` 的 client
- 且该 client 提供了 `instructions`

也就是说，MCP server 可以把“如何使用本 server”的指令挂进 system prompt。

### 2. attachment 侧增量更新

`nN8(...)` 会生成 `mcp_instructions_delta`。  
其控制逻辑已可直接写成：

- `ZT6()` gate 开启时才启用
- `IHq(mcpClients, transcriptMessages, clientSideExtraBlocks)` 比较：
  - 当前已连接 clients 中带 `instructions` 的集合
  - 历史 transcript 里已经宣布过的 `mcp_instructions_delta`
- 只发新增 server 的 block，或移除已经不存在的 server 名

另外还有一条很关键的补充：

- 当 deferred tools 开启、模型支持、并且 MCP tool search 可用时，会额外插入一个 client-side block
- 这说明 `mcp_instructions_delta` 不只是 server 原生 instructions，还可能叠加**客户端使用提醒**

## `mcp_resource`：resource 是 transcript 一等类型

MCP resource 当前已经不是“工具调用返回字符串”这么简单。

至少已确认：

- attachment 类型里有 `mcp_resource`
- `@server:uri` 形式会走 `m6z(...)`
- `m6z(...)` 会：
  - 从当前 `mcpClients` 找 server
  - 从 `options.mcpResources[server]` 找 uri 元数据
  - 调 `client.readResource({ uri })`
  - 生成 `{ type: "mcp_resource", server, uri, name, description, content }`

这说明 MCP resource 在运行时有独立对象形状，不是临时文本拼接。

同时要注意它的 prompt 侧语义也已经可以写实：

- `dt1(...)` 不会只把它还原成一个 URI 引用
- 而是会把 `readResource` 返回的正文快照直接展开进 prompt
- `text` 内容会连同“不要重复读取该资源”的提醒一起写入
- `blob` 内容则退化成二进制占位说明

因此 `mcp_resource` 更接近“当前轮已读资源正文快照”，而不是“资源引用记录”。

更细的 attachment payload/materialize 规则，见：

- [../02-execution/05-attachments-and-context-modifiers/02-high-value-attachment-payloads-and-materialization.md](../02-execution/05-attachments-and-context-modifiers/02-high-value-attachment-payloads-and-materialization.md)

## SDK tool schema：resource / subscribe / polling 已直接暴露

`sdk-tools.d.ts` 直接证明 MCP resource 相关能力至少包括：

- `ListMcpResources`
- `ReadMcpResource`
- `SubscribeMcpResource`
- `UnsubscribeMcpResource`
- `SubscribePolling`
- `UnsubscribePolling`

而且 schema 已经细到：

- `subscriptionId`
- `reason`
- `type: "tool" | "resource"`
- `intervalMs`
- `blobSavedTo`

因此资源订阅/轮询不只是 UI 概念，而是**正式工具面能力**。

## channel push notification：至少有一条正式入站链

此前只知道“channel / push notification 可能存在”。  
现在可以把其中一条主链写实：

- 需要先发 `channel_enable`
- 只允许 **plugin-sourced 且 marketplace 可识别** 的 MCP server 注册 channel
- 注册成功后，会给该 server 安装 `notifications/claude/channel` handler
- 收到通知后会把内容包装成：
  - `mode: "prompt"`
  - `priority: "next"`
  - `isMeta: true`
  - `origin.kind: "channel"`
  - `origin.server: <server>`
  - `skipSlashCommands: true`
- 然后塞回本地 queued prompt 队列
- reconnect 后，`l3A(...)` 会重新注册同一 handler

因此至少对 channel 这一支，MCP push 已经不是抽象概念，而是：

- **server notification -> local prompt injection -> 后续主循环处理**

当前仍未正证的，是：

- 资源订阅更新是否也走同样 transcript/prompt 落点
- 非 channel 的其他通知类型是否还有第二条 UI 链

## resource / polling update：当前只正证到“可渲染”，还没正证到“谁在生产”

这块现在可以比之前更精确。

本地 bundle 里已经能直接看到三层事实：

### 1. 协议层支持

- schema 里有：
  - `resources/subscribe`
  - `resources/unsubscribe`
  - `notifications/resources/updated`
- client capability 检查也会校验：
  - server 是否支持 resources
  - server 是否支持 subscribe
- bundle 内嵌 MCP 基类还暴露了：
  - `sendResourceUpdated(...)`
  - `sendResourceListChanged()`

但这里要特别区分：

- 这是 **协议/SDK 面可发送**
- 不是已经证明 CLI 产品主链真的在发送

### 2. 连接层支持

- `resources/list_changed` 已有本地 notification handler
- 收到后会：
  - `Un.cache.delete(server)`
  - 重新跑 `Un(client)`
  - 刷新本地 `resources`
- 这一支当前本地只看到：
  - refresh runtime snapshot
  - 没看到写 transcript
  - 没看到回注 prompt
- 当前本地没看到 `notifications/resources/updated` 的对应 runtime handler
- `prompts/tools/resources` 三类 `list_changed`
  - 本地都只看到“删 cache -> 重新 list -> 更新 runtime snapshot”
  - 没看到它们直接生成 `<mcp-resource-update>` / `<mcp-polling-update>` 这类 transcript 文本

### 3. transcript / renderer 侧支持

- UI renderer 会专门识别：
  - `<mcp-resource-update ...>`
  - `<mcp-polling-update ...>`
- 还有单独的 parser，把它们显示成：
  - server
  - target
  - reason

但继续追本地 bundle 后，当前没有找到：

- 这两种 tag 的生产者
- 把 resource update / polling update 回注为 prompt 或 transcript 的本地写入点

而且这点现在可以写得更硬：

- 全 bundle 对 `<mcp-resource-update` / `<mcp-polling-update` 的字面命中，当前只落在：
  - parser 正则
  - renderer 分支
- `notifications/resources/updated`
  - 当前只正证到 schema / capability assert / generic `sendResourceUpdated(...)`
  - 没看到 CLI 主运行时对它注册 handler
- `sendResourceUpdated(...)` / `sendResourceListChanged()`
  - 当前也没看到任何产品层 call site
- 全 bundle 对 `setNotificationHandler(...)` 的主运行时命中
  - 当前能确认的是：
    - channel notification
    - `tools/prompts/resources` 的 `list_changed`
  - 没看到 `notifications/resources/updated` 被注册到产品级 handler
- 同一 bundle 里虽然有：
  - `notifications/resources/updated`
  - `notifications/resources/list_changed`
  - `resources/subscribe`
- 但当前本地可见代码并没有把这些 notification 映射成上述 tag 字符串

换句话说，静态证据当前更像：

- parser / renderer 预留了 update tag 的显示能力
- generic MCP SDK 预留了 resources updated/list_changed 的协议能力
- CLI 产品运行时主链实际只把 `resources/list_changed` 当成一次资源缓存刷新
- 没有把它提升成用户可见的 update 消息

因此这块更稳的结论应改成：

- **本地 bundle 已确认 update tag 的 renderer / parser 存在**
- **generic MCP 基类具备发送 update notification 的协议能力**
- **CLI 产品主链里，`resources/updated` 的 consumer 仍未找到**
- **CLI 产品主链里，`<mcp-resource-update>` / `<mcp-polling-update>` 的 producer 仍未找到**
- **若真实更新提示存在，当前更可能来自 bundle 外路径、服务端注入，或尚未命中的更高层工具实现**

## `options.mcpTools` 的非空主路径已确认

此前 `mcpTools` 只知道是 request option 里的字段，但本地常见路径大多传空数组。  
现在已经能确认它的主用法：

- 主 agent loop 调 `callModel(...)` 时，会把 `appState.mcp.tools` 直接塞进 `options.mcpTools`
- 同时还会带：
  - `hasPendingMcpServers`
  - `queryTracking`
- compact / tool summary / hook prompt / agent creation 等辅助查询，基本都传 `mcpTools: []`

因此更稳的结论应改成：

- `mcpTools` 非空主路径已经确认存在
- 它主要属于**主对话采样/模型请求路径**
- 辅助小查询、compact、summary 往往不带这组 runtime MCP tool 列表

## 更稳的工程结论

基于当前本地 bundle，MCP 子系统已经可以收敛成：

1. 有独立命令树和配置源
2. 有 enterprise / strict mode / allowlist-denylist 约束
3. 有基于连接签名的去重层
4. 有 `client -> tools/commands/resources` 的运行时装配层
5. 有独立 prompt 注入与 delta attachment 机制
6. 有 resource 读取、订阅、轮询与 transcript 表达
7. 有按 transport 分流的连接、超时、重连与 cleanup 生命周期
8. MCP prompt commands 会进入 command/skill 执行面
9. channel push 至少有一条已确认的 prompt 注入链
10. 与 deferred tool / ToolSearch 深度耦合

## 当前仍未完全钉死

- 资源订阅更新、polling 更新的本地 producer 仍未找到；当前更强的说法是“本地只正证到 parser / renderer，不足以证明 CLI 本地链会主动生产这些 tag”
- 服务端侧是否还会对 MCP 指令做额外拼装，目前不能从本地 bundle 直接证明

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
