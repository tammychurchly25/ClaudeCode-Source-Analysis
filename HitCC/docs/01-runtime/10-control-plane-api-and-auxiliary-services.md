# 控制面 API 与外围服务

## 本页用途

- 这页不再承载全部细节，而改成 `01-runtime` 下 control plane 主题的总览与导航。
- 原先混在一页里的内容，已经拆成 host/auth 与数据面、OAuth/account、remote environment/session、bridge credential 生命周期、GitHub/telemetry/MCP proxy 五个专题页。

## 相关文件

- [10-control-plane-api-and-auxiliary-services/01-hosts-auth-and-data-plane.md](./10-control-plane-api-and-auxiliary-services/01-hosts-auth-and-data-plane.md)
- [10-control-plane-api-and-auxiliary-services/02-oauth-and-account-control-plane.md](./10-control-plane-api-and-auxiliary-services/02-oauth-and-account-control-plane.md)
- [10-control-plane-api-and-auxiliary-services/03-remote-environments-and-sessions.md](./10-control-plane-api-and-auxiliary-services/03-remote-environments-and-sessions.md)
- [10-control-plane-api-and-auxiliary-services/04-bridge-credentials-and-worker-lifecycle.md](./10-control-plane-api-and-auxiliary-services/04-bridge-credentials-and-worker-lifecycle.md)
- [10-control-plane-api-and-auxiliary-services/05-github-telemetry-and-mcp-proxy.md](./10-control-plane-api-and-auxiliary-services/05-github-telemetry-and-mcp-proxy.md)
- [05-model-adapter-provider-and-auth.md](./05-model-adapter-provider-and-auth.md)
- [06-stream-processing-and-remote-transport.md](./06-stream-processing-and-remote-transport.md)
- [09-api-lifecycle-and-telemetry.md](./09-api-lifecycle-and-telemetry.md)
- [12-settings-and-configuration-system/02-loading-policy-and-merge.md](./12-settings-and-configuration-system/02-loading-policy-and-merge.md)
- [../03-ecosystem/02-remote-persistence-and-bridge.md](../03-ecosystem/02-remote-persistence-and-bridge.md)
- [../03-ecosystem/04-mcp-system.md](../03-ecosystem/04-mcp-system.md)

## 拆分后的主题边界

### Host / 认证形态 / 数据面 supporting endpoints

见：

- [10-control-plane-api-and-auxiliary-services/01-hosts-auth-and-data-plane.md](./10-control-plane-api-and-auxiliary-services/01-hosts-auth-and-data-plane.md)

这一页集中放：

- first-party 网络面的四层划分
- `api.anthropic.com`、`platform.anthropic.com`、`claude.ai / claude.com` 相关 OAuth/web host、`mcp-proxy.anthropic.com`
- 数据面、org-scoped 控制面、MCP proxy 的认证差异
- `remote managed settings` 这条 first-party 远端配置通道与其鉴权差异
- `/v1/messages`、`/v1/models`、`/v1/files` 等 supporting endpoints

### OAuth / account control plane

见：

- [10-control-plane-api-and-auxiliary-services/02-oauth-and-account-control-plane.md](./10-control-plane-api-and-auxiliary-services/02-oauth-and-account-control-plane.md)

这一页集中放：

- authorize / token exchange
- `/api/oauth/profile`
- `/api/oauth/claude_cli/roles`
- `/api/oauth/claude_cli/create_api_key`
- OAuth scope 族

### remote environment / code session control plane

见：

- [10-control-plane-api-and-auxiliary-services/03-remote-environments-and-sessions.md](./10-control-plane-api-and-auxiliary-services/03-remote-environments-and-sessions.md)

这一页集中放：

- environment 枚举与创建
- environment 列表的本地选择与消费面
- `/v1/sessions*` REST 与 WebSocket 主链

### bridge credentials / worker 生命周期 / 失败边界

见：

- [10-control-plane-api-and-auxiliary-services/04-bridge-credentials-and-worker-lifecycle.md](./10-control-plane-api-and-auxiliary-services/04-bridge-credentials-and-worker-lifecycle.md)

这一页集中放：

- `/v1/environments/bridge*` 辅助接口
- `environment_secret -> work secret -> session_ingress_token`
- `/v1/code/sessions/{id}/bridge` 响应字段
- `worker_jwt` 刷新链、`worker_epoch` 失败语义
- `environment_secret` 轮换边界与当前不能过度推断的点

### GitHub 接入 / telemetry / MCP proxy

见：

- [10-control-plane-api-and-auxiliary-services/05-github-telemetry-and-mcp-proxy.md](./10-control-plane-api-and-auxiliary-services/05-github-telemetry-and-mcp-proxy.md)

这一页集中放：

- GitHub App / token-sync / import-token
- `/api/event_logging/batch`
- `ClaudeCodeInternalEvent` 与 `GrowthbookExperimentEvent`
- `claudeai-proxy` MCP transport
- 当前仍未完全钉死的外围控制面边界

## 建议阅读顺序

1. 先看 [01-hosts-auth-and-data-plane.md](./10-control-plane-api-and-auxiliary-services/01-hosts-auth-and-data-plane.md)，建立 host、认证与 supporting API 的总图。
2. 再看 [02-oauth-and-account-control-plane.md](./10-control-plane-api-and-auxiliary-services/02-oauth-and-account-control-plane.md)，补齐 account / org metadata 获取链。
3. 然后看 [03-remote-environments-and-sessions.md](./10-control-plane-api-and-auxiliary-services/03-remote-environments-and-sessions.md)，建立 remote environment 与 session 主接口。
4. 接着看 [04-bridge-credentials-and-worker-lifecycle.md](./10-control-plane-api-and-auxiliary-services/04-bridge-credentials-and-worker-lifecycle.md)，理解 bridge worker 控制面的 credential 分层和还原语义。
5. 最后看 [05-github-telemetry-and-mcp-proxy.md](./10-control-plane-api-and-auxiliary-services/05-github-telemetry-and-mcp-proxy.md)，把 GitHub、遥测和 MCP 代理这些外围控制面补齐。

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
