# 配置与用户设置系统

## 本页用途

- 这页不再承载全部细节，而改成 `01-runtime` 下 settings 主题的总览与导航。
- 原先混在一页里的内容，已经拆成 source/path、加载与 merge、缓存与写回、CLI 注入与 schema、消费索引与重写边界五个专题页。

## 相关文件

- [12-settings-and-configuration-system/01-source-model-and-paths.md](./12-settings-and-configuration-system/01-source-model-and-paths.md)
- [12-settings-and-configuration-system/02-loading-policy-and-merge.md](./12-settings-and-configuration-system/02-loading-policy-and-merge.md)
- [12-settings-and-configuration-system/03-cache-refresh-and-writeback.md](./12-settings-and-configuration-system/03-cache-refresh-and-writeback.md)
- [12-settings-and-configuration-system/04-cli-injection-schema-and-migration.md](./12-settings-and-configuration-system/04-cli-injection-schema-and-migration.md)
- [12-settings-and-configuration-system/05-key-consumers-and-rewrite-boundaries.md](./12-settings-and-configuration-system/05-key-consumers-and-rewrite-boundaries.md)
- [01-product-cli-and-modes.md](./01-product-cli-and-modes.md)
- [02-session-and-persistence.md](./02-session-and-persistence.md)
- [05-model-adapter-provider-and-auth.md](./05-model-adapter-provider-and-auth.md)
- [09-api-lifecycle-and-telemetry.md](./09-api-lifecycle-and-telemetry.md)
- [../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md](../02-execution/01-tools-hooks-and-permissions/03-permission-mode-and-classifier.md)
- [../02-execution/01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md](../02-execution/01-tools-hooks-and-permissions/04-policy-sandbox-and-approval-backends.md)
- [../03-ecosystem/04-mcp-system.md](../03-ecosystem/04-mcp-system.md)
- [../03-ecosystem/06-plugin-system.md](../03-ecosystem/06-plugin-system.md)

## 拆分后的主题边界

### source 模型 / 路径体系

见：

- [12-settings-and-configuration-system/01-source-model-and-paths.md](./12-settings-and-configuration-system/01-source-model-and-paths.md)

这一页集中放：

- 5 个正式 settings source
- `--setting-sources` 的真实边界
- 用户根目录、enterprise policy 根目录
- `userSettings / projectSettings / localSettings / flagSettings / policySettings` 路径族

### 读取 / 校验 / merge / remote managed settings

见：

- [12-settings-and-configuration-system/02-loading-policy-and-merge.md](./12-settings-and-configuration-system/02-loading-policy-and-merge.md)

这一页集中放：

- `ye(path)` 与 `_D()` 校验链
- `C28(...)` 的 permission rules 预清洗
- `PZ3()` effective settings 装配
- `policySettings` fallback 链
- remote managed settings、policy limits、平台托管差异与 merge 规则

### 缓存 / 热刷新 / 写回语义

见：

- [12-settings-and-configuration-system/03-cache-refresh-and-writeback.md](./12-settings-and-configuration-system/03-cache-refresh-and-writeback.md)

这一页集中放：

- 两级 cache 与 plugin overlay cache
- `BX()` 失效入口
- 文件 watcher 与 MDM/registry poll
- `wA(source, patch)` 写回语义
- config 面板与 app state / global store 的分流

### CLI 注入 / schema / 迁移兼容层

见：

- [12-settings-and-configuration-system/04-cli-injection-schema-and-migration.md](./12-settings-and-configuration-system/04-cli-injection-schema-and-migration.md)

这一页集中放：

- `--settings`、`--setting-sources`
- 运行期 `apply_flag_settings`
- settings change 的运行时反应链
- schema 主分组
- 启动迁移矩阵与兼容入口

### 键族消费索引 / 未决边界 / 重写结论

见：

- [12-settings-and-configuration-system/05-key-consumers-and-rewrite-boundaries.md](./12-settings-and-configuration-system/05-key-consumers-and-rewrite-boundaries.md)

这一页集中放：

- 高价值键族到消费点索引
- `strictPluginOnlyCustomization` surface 边界
- `sshConfigs` 的当前负证与剩余不确定性
- 当前仍未完全钉死的点
- 可直接作为重写输入的骨架

## 建议阅读顺序

1. 先看 [01-source-model-and-paths.md](./12-settings-and-configuration-system/01-source-model-and-paths.md)，建立 source 和路径边界。
2. 再看 [02-loading-policy-and-merge.md](./12-settings-and-configuration-system/02-loading-policy-and-merge.md)，把读取、校验、远端托管和 merge 主链接起来。
3. 然后看 [03-cache-refresh-and-writeback.md](./12-settings-and-configuration-system/03-cache-refresh-and-writeback.md)，理解热刷新和持久化入口。
4. 接着看 [04-cli-injection-schema-and-migration.md](./12-settings-and-configuration-system/04-cli-injection-schema-and-migration.md)，补齐 flag、schema 和历史兼容层。
5. 最后看 [05-key-consumers-and-rewrite-boundaries.md](./12-settings-and-configuration-system/05-key-consumers-and-rewrite-boundaries.md)，把消费面、未决点与重写边界收束到一起。

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
