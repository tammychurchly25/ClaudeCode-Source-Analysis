# 非模型网络通道

## 本页用途

- 把所有“不走 `/v1/messages` 主推理请求、但确实会出网”的运行路径收束到同一页。
- 明确这些路径与 inference/data plane、control plane、telemetry 初始化页之间的边界。
- 避免后续继续把 voice、plugin stats、GrowthBook、transcript share 这类外围流量散写到各自语境里。

## 相关文件

- [05-model-adapter-provider-and-auth.md](./05-model-adapter-provider-and-auth.md)
- [06-stream-processing-and-remote-transport.md](./06-stream-processing-and-remote-transport.md)
- [09-api-lifecycle-and-telemetry.md](./09-api-lifecycle-and-telemetry.md)
- [10-control-plane-api-and-auxiliary-services.md](./10-control-plane-api-and-auxiliary-services.md)
- [../03-ecosystem/02-remote-persistence-and-bridge.md](../03-ecosystem/02-remote-persistence-and-bridge.md)
- [../03-ecosystem/06-plugin-system.md](../03-ecosystem/06-plugin-system.md)

## 一句话结论

Claude Code CLI 的“网络通信模块”不能只看模型请求。  
当前本地 bundle 已能确认至少还有四组独立的外围出网链：

- voice stream WebSocket
- plugin install counts 拉取
- GrowthBook remote eval / streaming refresh / experiment event logging
- transcript share、remote transcript persistence、1P event logging failed-batch 缓存这组三类外围同步链

它们与 `/v1/messages` 主链共享认证或全局状态，但不属于同一条 data plane。

## 总体结构

当前更稳的非模型网络图应写成：

```text
non-LLM network paths
  -> voice dictation path
     -> local recording
     -> remote STT websocket

  -> plugin ecosystem peripheral fetch
     -> install counts JSON
     -> local cache

  -> feature flag / experiment path
     -> GrowthBook remote eval
     -> EventSource refresh
     -> GrowthbookExperimentEvent -> 1P event logging

  -> session/peripheral sync paths
     -> transcript share upload
     -> remote transcript persistence
     -> 1P event logging failed-batch cache
     -> local-only UI stats store (boundary, not remote)
```

这里最重要的不是“还有几条 HTTP 请求”，而是：

- 它们各自有独立 transport
- 触发时机和失败语义不同
- 有些只做装饰性增强
- 有些会影响实时交互能力
- 有些只是本地统计，不应误记成远端同步

## 1. voice stream WebSocket

### 结论

voice 模式不是本地离线语音识别。  
当前可见实现是：

- 本地录音
- 远端 WebSocket STT
- 本地按 transcript 增量回填输入框

因此它属于一条独立的实时媒体网络通道。

### 本地录音与前置条件

录音层当前已能直接确认：

- 优先 native audio module
- Linux 可退回 `arecord`
- 其他平台可退回 `sox rec`
- 默认采样参数固定为：
  - `sample_rate = 16000`
  - `channels = 1`
  - `encoding = linear16`

这说明 voice 的音频采集在本地完成，网络侧只负责 STT。

### WebSocket 连接链

`connectVoiceStream` / `ip8(...)` 的主链当前可以写成：

```text
useVoice()
  -> startRecording()
  -> connectVoiceStream(ip8)
  -> WebSocket ${BASE_API_URL -> ws}/api/ws/speech_to_text/voice_stream
```

URL 参数已能直接确认包括：

- `encoding=linear16`
- `sample_rate=16000`
- `channels=1`
- `endpointing_ms=300`
- `utterance_end_ms=1000`
- `language=<normalized language>`
- 可选 `keyterms[]`

认证头也已可写实：

- `Authorization: Bearer <oauth access token>`
- `User-Agent`
- `x-app: cli`

因此这条链依赖 claude.ai OAuth token，而不是 API key。

### 连接内协议

当前可见消息协议至少包括：

- client -> server
  - 二进制 audio chunk
  - `KeepAlive`
  - `CloseStream`
- server -> client
  - `TranscriptText`
  - `TranscriptEndpoint`
  - `TranscriptError`
  - `error`

并且客户端还实现了：

- 周期性 `KeepAlive`
- finalize safety timeout
- `no_data_timeout`
- 早期 `voice_stream` 错误时单次重试
- silent-drop replay

所以这不是普通“发一个音频文件再等结果”，而是持续连接的流式会话协议。

### feature flag 与分支

voice stream 里还能直接看到一条 GrowthBook 相关分支：

- `tengu_cobalt_frost`
  - 开启后附加：
    - `use_conversation_engine=true`
    - `stt_provider=deepgram-nova3`

因此 voice 本身也受远端 feature flag 影响，不是完全静态实现。

### 边界

当前能正证的是：

- 客户端录音链
- WebSocket endpoint
- 认证头
- wire protocol
- retry / keepalive / finalize 行为

当前不能正证的是：

- 服务端 STT 引擎内部实现
- `deepgram-nova3` 之外的全部 provider 分支

## 2. plugin install counts 拉取

### 结论

plugin install counts 不属于 marketplace manifest 主链。  
它是 `plugin list --available --json` 时附加的一条装饰性远端拉取路径。

它失败不会破坏 plugin discovery，只会丢失 `installCount` 字段。

### 主链

当前实现可以直接收束成：

```text
plugin list --available --json
  -> EL6()
    -> C3z() read local cache
    -> else I3z() fetch remote JSON
    -> b3z() save cache
    -> Map(pluginId -> unique_installs)
  -> merge into available plugin list as installCount
```

远端 URL 当前已可直接写死为：

```text
https://raw.githubusercontent.com/anthropics/claude-plugins-official/refs/heads/stats/stats/plugin-installs.json
```

### 本地缓存

缓存行为也已基本闭环：

- 文件名：`install-counts-cache.json`
- cache version：`1`
- TTL：24 小时
- 过期、结构错误、版本不匹配都会视为 cache miss

缓存 entry 形状至少包括：

```text
{
  version,
  fetchedAt,
  counts: [
    { plugin, unique_installs }
  ]
}
```

因此这条链不是 session 级缓存，而是 plugin 生态自己的轻量本地缓存。

### 对 plugin 系统的真正意义

更稳的判断不是“plugin 系统会远端查询安装统计”，而是：

- plugin 主发现链来自 marketplace manifest
- install counts 只是列表 UI / JSON 输出的附加增强
- 它不参与 enable/install/update 决策

所以它应归入“外围网络通道”，不应混到 plugin 安装主逻辑里。

## 3. GrowthBook / feature flag 远端拉取与事件记录

### 结论

当前 bundle 中的 feature flag 不是纯本地配置表。  
它至少同时覆盖三条独立链：

- remote eval 拉取 features/experiments
- EventSource 增量刷新
- experiment exposure -> 1P event logging

因此它本身就是一套独立的外围网络子系统。

### remote eval 主链

`initializeGrowthBook` / `tl()` 当前可还原成：

```text
tl()
  -> VZ1()
    -> new GrowthBook-like client D_8({
         apiHost: "https://api.anthropic.com/",
         clientKey: "sdk-zAZezfDKGoZuXXKe",
         remoteEval: true,
         cacheKeyAttributes: ["id", "organizationUUID"],
         apiHostRequestHeaders?: BH().headers
       })
    -> client.init()
    -> refresh / cache / publish to AppState.cachedGrowthBookFeatures
```

远端路径已能直接确认包括：

- `GET /api/features/<clientKey>`
- `POST /api/eval/<clientKey>`
- `EventSource /sub/<clientKey>`

因此 GrowthBook 在这里不是“只读 feature JSON”，而是远端评估型接入。

### 本地缓存与刷新

客户端当前还维护：

- `cachedGrowthBookFeatures`
- 内存 `Map`：当前活跃 features
- 周期性 refresh timer
- auth change 后的强制 refresh

并会在 refresh 成功后：

- 更新 `cachedGrowthBookFeatures`
- 唤醒 `onGrowthBookRefresh()` 订阅者

这说明 feature flag 的结果是 session 运行态的一部分，不是一次性启动配置。

### EventSource 背景同步

若后端声明支持 streaming，会进一步进入：

```text
EventSource /sub/<clientKey>
  -> listen "features"
  -> listen "features-updated"
  -> refresh in background
```

并带有：

- reconnect/backoff
- visibility / idle listener
- 本地 stale cache 策略

因此它已经不是简单 polling。

### experiment event logging

当 feature 来自 experiment 时，客户端会把曝光转成：

- `GrowthbookExperimentEvent`

随后进入 1P event logging sink。  
本页在这里主要固定一条跨系统边界：

- GrowthBook 不只影响功能 gate
- experiment exposure 还会进入 1P event logging
- 因此它会和 telemetry/event logging 配置发生联动

更细的 event logger 初始化、batch 配置、endpoint 与 payload 形状，分别见：

- [09-api-lifecycle-and-telemetry.md](./09-api-lifecycle-and-telemetry.md)
- [10-control-plane-api-and-auxiliary-services/05-github-telemetry-and-mcp-proxy.md](./10-control-plane-api-and-auxiliary-services/05-github-telemetry-and-mcp-proxy.md)

### 非必要流量边界

结合 telemetry 页，更稳的写法应是：

- GrowthBook / 1P event logging 属于外围流量
- 但它们不等同于 OTEL `initializeTelemetry()`
- 也不应再被表述成 provider 主请求附带行为

## 4. stats / usage 相关远端或缓存同步链路

### 先划清边界

这一组最容易写乱。  
当前必须先拆成两类：

- 真正出网或远端同步的链路
- 纯本地统计对象

如果不先拆开，后面很容易把 UI stats、session usage、remote persistence 混成一个“stats 模块”。

### 4.1 transcript share

反馈调查里的 transcript share 当前已经可写实成独立上传路径：

```text
feedback survey
  -> user chooses share transcript
  -> POST https://api.anthropic.com/api/claude_code_shared_session_transcripts
```

请求体至少包括：

- `content`
- `appearance_id`

固定请求头至少包括：

- `Content-Type: application/json`
- `User-Agent`

因此它不是 `/bug` 命令，也不是 event logging batch，而是单独的 transcript share endpoint。

### 4.2 remote transcript persistence

session/transcript writer 当前还有一条独立远端同步链：

```text
appendEntry()
  -> persistToRemote(sessionId, entry)
```

其行为边界已经比较清楚：

- 优先写 internal event writer
- 否则若 remote ingress URL 存在，则走 remote persistence
- 失败只记 `tengu_session_persistence_failed`

因此它更准确的定位不是“stats/usage 上传”，而是：

- transcript/workspace state 的远端副本持久化
- 需要和远端 session ingress 保持链式一致

因此 transcript 本身不只是本地 JSONL，也可能被同步到远端 session ingress。

这一块更细闭环见：

- [../03-ecosystem/02-remote-persistence-and-bridge.md](../03-ecosystem/02-remote-persistence-and-bridge.md)

### 4.3 1P event logging failed-batch 本地缓存

1P event logging 失败时，不会只丢日志。  
当前可见实现会把失败批次落到：

```text
<Claude root>/telemetry/1p_failed_events.<session>.<uuid>.json
```

这条链的意义很关键：

- 它是“远端统计上传失败后的本地补偿缓存”
- 不是 session transcript
- 也不是普通 debug log

因此它属于 stats/usage 相关的外围同步机制。

这里还能再补两条关键边界，见 `cli.js`：

- `ZZ1` 构造时就会调用 `retryPreviousBatches()`，因此启动期会扫描 `telemetry/` 下同 session 的历史 `1p_failed_events.*.json`，并在后台回捞重试
- 回捞时会排除当前活动批次文件；若达到 `maxAttempts` 仍失败，对应 failed-batch 文件会被直接删除

默认重试参数当前也能直接写死：

- `batchDelayMs=100`
- `baseBackoffDelayMs=500`
- `maxBackoffDelayMs=30000`
- `maxAttempts=8`

其中 backoff 公式是 `min(baseBackoffDelayMs * attempts^2, maxBackoffDelayMs)`，所以它不是固定延迟重试，而是有上限的平方退避。

### 4.4 本地 UI stats store 不是远端网络链

当前 `statsStore` / `getFpsMetrics` / `frame_duration_ms` 这一组需要明确排除：

- `statsStore` 保存在全局 `AppState`
- TUI render path 会 `hd8(stats)`
- frame timing 通过 `stats.observe("frame_duration_ms", ...)` 记录

这说明它是：

- 本地 UI/性能统计对象

而不是：

- 独立远端 stats API
- usage sync endpoint

因此在本页里应把它写成“边界提醒”，而不是算进非模型出网主表。

### 4.5 与 request usage 的区别

这里还要明确与主模型 usage 的边界：

- `input_tokens / output_tokens / cache_* / costUSD`
  - 属于 `/v1/messages` 请求生命周期
- transcript share / remote transcript persistence / 1P event logging failed-batch
  - 属于 session/peripheral sync 链

两者不能合并称为同一种“usage 模块”。

## 5. 与其他页面的分工

本页负责回答：

- 除模型主请求外，CLI 还会在哪些地方出网
- 这些路径各自的 transport 是什么
- 哪些只是装饰性增强，哪些会影响交互能力
- 哪些统计是远端同步，哪些只是本地状态

其他页面负责：

- provider/data plane 主链
  - [05-model-adapter-provider-and-auth.md](./05-model-adapter-provider-and-auth.md)
- stream/fallback/sdk-url/remote ingress transport
  - [06-stream-processing-and-remote-transport.md](./06-stream-processing-and-remote-transport.md)
- telemetry 初始化与开关矩阵
  - [09-api-lifecycle-and-telemetry.md](./09-api-lifecycle-and-telemetry.md)
- control plane endpoints 总表
  - [10-control-plane-api-and-auxiliary-services.md](./10-control-plane-api-and-auxiliary-services.md)
- remote transcript / bridge 持久化
  - [../03-ecosystem/02-remote-persistence-and-bridge.md](../03-ecosystem/02-remote-persistence-and-bridge.md)
- plugin 主系统
  - [../03-ecosystem/06-plugin-system.md](../03-ecosystem/06-plugin-system.md)

## 当前已经能稳下来的结论

1. voice 是本地录音 + 远端 STT WebSocket，不属于 `/v1/messages` 主模型请求。
2. plugin install counts 是 GitHub raw JSON 拉取 + 本地 24h 缓存，只影响可用插件列表的装饰字段。
3. GrowthBook 是 remote eval + EventSource refresh + experiment event logging 的组合子系统。
4. transcript share、remote transcript persistence、1P event logging failed-batch 缓存都属于 session/peripheral sync 链；其中 remote transcript persistence 更偏 transcript 一致性，而不是统计上传。
5. `statsStore` / FPS / frame timing 是本地 UI stats，不应误记成远端 stats API。

## 当前仍未完全钉死

- voice 服务端 STT 的内部 provider 与后端实现细节，本地 bundle 无法正证。
- plugin install counts 上游 `stats` 分支的生成逻辑不可见，本地只能确认消费端。
- `statsStore` 与更大范围 perf/telemetry exporter 的全部对应关系，不适合在本页继续展开。

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
