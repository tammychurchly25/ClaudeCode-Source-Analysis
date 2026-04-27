# 产品形态、入口、命令树与运行模式

## 本页用途

- 用来快速理解 Claude Code CLI 是什么产品，以及 CLI 如何从顶层入口分流到不同运行模式。
- 用来建立 `jBz -> _Bz -> YBz` 这一段启动链的整体认识。

## 相关文件

- [../00-overview/01-scope-and-evidence.md](../00-overview/01-scope-and-evidence.md)
- [02-session-and-persistence.md](./02-session-and-persistence.md)
- [03-input-compilation.md](./03-input-compilation.md)
- [04-agent-loop-and-compaction.md](./04-agent-loop-and-compaction.md)
- [../05-appendix/01-glossary.md](../05-appendix/01-glossary.md)

## 产品形态总览

从发行版可以明确看出，Claude Code CLI 不是“问答命令行”，而是一个完整的 **Agentic Coding Shell**。

其能力至少包括：

- Commander 风格 CLI 程序
- React/Ink 风格 TUI
- Headless/SDK/JSON 流模式
- 会话持久化与还原
- 分叉会话 / 子 Agent / Sidechain
- 工具执行与权限系统
- MCP server/client 管理
- 插件管理与 Marketplace
- Chrome/native-host / computer-use / remote-control bridge
- Hook 系统
- Rules / Skills / Memory / Plan / File History
- 远程 transcript 同步

### 产品本质

其产品本质可以理解为：

```text
CLI/TUI 外壳
  -> 输入编译器
    -> 多轮 Agent Loop
      -> 模型调用适配层
      -> 工具执行器
      -> Hook/Permission
      -> Session/Transcript/Plan/FileHistory
```

### 与一般聊天 CLI 的根本差异

Claude Code CLI 与普通聊天 CLI 的最大区别：

1. **会话不是单一消息数组**，而是带 file-history、content-replacement、plan、attribution、queued commands 的“工作现场”。
2. **工具不是插件式附加物**，而是主循环的一等公民。
3. **Prompt 不是固定字符串**，而是多源、多层、带缓存的 system sections。
4. **Resume/Fork** 还原的是“工作状态”，不仅是聊天记录。
5. **Headless 与 TUI 共享核心循环**，只是外层 transport/render 不同。

---

## 顶层入口与启动分流

### 顶层入口：`jBz()`

最外层入口不是直接进主 CLI，而是先做 fast-path 分流。

### 已确认分支

按大致顺序：

1. `--version`：直接输出版本并退出
2. `--claude-in-chrome-mcp`：启动 Chrome MCP server
3. `--chrome-native-host`：启动 Chrome native host
4. `--computer-use-mcp`：启动 computer-use MCP server
5. `remote-control | rc | remote | sync | bridge`：进入 remote bridge 路径
6. `--tmux + --worktree`：命中 tmux/worktree 快速路径
7. `--update / --upgrade`：改写成 `update` 子命令
8. `--bare`：设置 `CLAUDE_CODE_SIMPLE=1`
9. `startCapturingEarlyInput()`：提前捕获输入
10. 懒加载 `main()`（即 `_Bz()`）

### 设计意图

这是典型的“**薄入口 + 重模式提前分流 + 主模块延迟加载**”设计。

优点：

- 启动更快
- Bridge/Chrome/Native Host 模式不必加载整套 TUI/Agent Loop
- 某些特殊子进程可以保持较小内存占用

### 重写建议

重写时保留这个分层：

```ts
entry.ts
  -> fastPathDispatch(argv)
  -> mainCli()
```

---

## 主 CLI 程序与命令树

### `main()` / `_Bz()`

`_Bz()` 的职责是：

- 初始化信号处理
- 处理 deep-link/deeplink URI
- 判断是否非交互模式
- 预加载 settings
- 进入 Commander 程序构建与执行

### 非交互模式判定

至少会触发 non-interactive 的条件：

- `-p` / `--print`
- `--init-only`
- `--sdk-url`
- `!stdout.isTTY`

### 推论

Claude Code CLI 的非交互模式不只是一种“纯文本打印”，而是一个独立的一等运行模式，Headless/SDK/Bridge 都依赖此路径。

### `YBz()`：真正的 CLI 主控

`YBz()` 负责两件事：

1. 构建 Commander 命令树
2. 设置 `preAction` 初始化逻辑

### 已确认的顶层命令

- `claude [prompt]`
- `auth`
- `agents`
- `auto-mode`
- `doctor`
- `install`
- `mcp`
- `plugin` / `plugins`
- `setup-token`
- `update` / `upgrade`
- 隐藏：`remote-control` / `rc`

### 已确认的子命令

#### auth
- `login`
- `logout`
- `status`

#### mcp
- `add`
- `add-from-claude-desktop`
- `add-json`
- `get`
- `list`
- `remove`
- `reset-project-choices`
- `serve`

#### plugin
- `install`
- `uninstall` / `remove`
- `enable`
- `disable`
- `update`
- `list`
- `marketplace`
- `validate`

### 关键选项

从主 action 入口与 help 文本可确认常见选项至少包括：

- `--print`
- `--output-format`
- `--input-format`
- `--json-schema`
- `--continue`
- `--resume`
- `--fork-session`
- `--session-id`
- `--name`
- `--model`
- `--effort`
- `--fallback-model`
- `--tools`
- `--allowedTools`
- `--disallowedTools`
- `--permission-mode`
- `--dangerously-skip-permissions`
- `--system-prompt`
- `--append-system-prompt`
- `--settings`
- `--setting-sources`
- `--mcp-config`
- `--plugin-dir`
- `--strict-mcp-config`
- `--worktree`
- `--tmux`
- `--ide`
- `--bare`
- `--sdk-url`
- `--teleport`
- `--remote`
- `--remote-control`

### `preAction`

统一初始化层，而不是每个命令各自初始化。已确认其至少会做：

- terminal title
- log sink
- migration
- plugin-dir 注入
- settings 同步
- remote settings / managed settings 处理

### 设计特点

这说明原工程非常强调：

- **命令层统一生命周期**
- 配置、日志、插件、远端设置在 action 前完成接线

---

## 运行模式：Headless / TUI / Bridge / Chrome / MCP

### 模式总览

可以明确分出：

1. **TUI 模式**：交互式终端 UI
2. **Headless 模式**：非交互/print/json/stream-json
3. **Bridge 模式**：remote-control / sdk-url / stream-json
4. **Chrome/Native Host 模式**
5. **MCP serve 模式**
6. **computer-use-mcp 模式**

### Headless

Headless 由 `runHeadless()`（对应逆向中提到的 `cuz(...)`）承担。

其职责：

- 构造 IO bridge
- 初始化 sandbox
- 统一处理 continue/resume/teleport
- 构造工具集与 permission gate
- 调用 `luz(...)`
- 根据 `text/json/stream-json` 输出结果

### TUI

TUI 最终会进 `hA8(...)` 一类 React/Ink root。

典型入口：

- 新会话
- resume/continue/teleport 后进入主 App
- 仅 `--resume` 无 session 时进入 ResumeConversation 选择器

### Bridge

已确认支持：

- `--input-format stream-json`
- `--output-format stream-json`
- `--sdk-url`
- `remote-control`

说明 Headless path 可以被外部前端/控制器嵌套调用，作为“子 agent 进程”或本地执行后端。

但 `remote-control`、`--remote`、`--sdk-url + stream-json` 三条入口的产品语义并不相同：

- `remote-control`
  - 把当前本地会话开放给远端接入
- `--remote`
  - 让 CLI 充当远端 session 前端
- `--sdk-url + stream-json`
  - 让 CLI 充当可嵌入的 agent backend

更细的工作流矩阵、bridge transport、远程 transcript 持久化与冲突还原，统一见：

- [../03-ecosystem/02-remote-persistence-and-bridge.md](../03-ecosystem/02-remote-persistence-and-bridge.md)

### Chrome / Native Host / computer-use

这几条都是启动时 fast-path，独立于主 CLI；因此可以被视为“旁路模式”。

---

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
