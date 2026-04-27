# 文档风格与结构约定

## 本页用途

- 用来固定 `docs/` 的信息架构、父页/子页职责和基础文风，避免后续继续把目录页写回混合正文。
- 用来给后续拆分、补证据、重组章节提供统一判断标准。

## 适用范围

- 本页适用于 `docs/` 下所有主题文档。
- 若本页与旧文档写法冲突，以本页为准，并在后续整理时逐步回收旧写法。

## 核心原则

### 1. 父页只做导航，不承载新正文

目录父页统一定义为“导航页”，例如：

- `01-runtime/10-control-plane-api-and-auxiliary-services.md`
- `01-runtime/12-settings-and-configuration-system.md`
- `02-execution/01-tools-hooks-and-permissions.md`
- `02-execution/05-attachments-and-context-modifiers.md`
- `03-ecosystem/01-resume-fork-sidechain-and-subagents.md`
- `03-ecosystem/07-tui-system.md`

这类页面只允许承载：

- 本页用途
- 子页地图
- 每个子页的一句话边界
- 建议阅读顺序
- 与相邻专题的边界说明

这类页面不再承载：

- 长篇运行时分析
- 大段伪代码主链
- “当前最稳的结论”一类正文总结
- 证据落点、bundle 行号、压缩名细节判断
- “当前仍未完全钉死”这类正文级未决项

若某段内容需要解释“怎么运作”“证据在哪”“哪里还不确定”，它应进入子页，而不是继续留在父页。

### 2. 子页才是正文与证据页

子页负责承载：

- 具体机制
- 关键调用链
- 证据落点
- 反证与边界
- 当前未决点
- 可直接服务重写的结论

判断标准很简单：

- 需要写函数名、压缩名、事件链、schema、证据点时，写到子页。
- 只是在回答“这个专题分成哪几块、先读哪块、和别的专题怎么分界”时，才写在父页。

### 3. 总索引只做全局入口，不复制正文

`00-overview/00-index.md` 只负责：

- 全局导航
- 阅读顺序
- 维护入口

不负责重复各主题页中的详细判断。

### 4. 未知点集中收口

未知点不要在多个父页重复维护。

统一入口保持为：

- `00-overview/01-scope-and-evidence.md`
- `04-rewrite/02-open-questions-and-judgment.md`

若子页存在局部未决项，可以在子页末尾保留，但不要再把同一组未知点抄回父页。

## 页面类型约定

### A. 总索引页

特征：

- 面向整套知识库
- 负责全局阅读入口

当前对应：

- `00-overview/00-index.md`

### B. 父页 / 导航页

特征：

- 对应一个目录或一组拆分页
- 负责主题地图和边界，不负责展开细节

建议长度：

- 优先控制在 `60-120` 行内

### C. 子页 / 正文页

特征：

- 对应一个稳定问题域
- 允许展开调用链、证据、反证、判断

建议长度：

- 超过 `500` 行时，应主动评估是否还能继续按问题域拆分
- 超过 `800` 行时，默认视为应拆分候选，除非确实无法再拆

### D. 附录页

特征：

- 不承载主题主叙事
- 用来统一术语、证据索引、速查表

当前对应：

- `05-appendix/01-glossary.md`
- `05-appendix/02-evidence-map.md`

## 父页推荐模板

父页优先采用以下结构：

1. `## 本页用途`
2. `## 相关文件`
3. `## 拆分后的主题边界` 或 `## 专题拆分`
4. `## 建议阅读顺序`
5. `## 与其它专题的边界`

其中：

- `相关文件` 只列本页真正依赖的相邻主题，不要把整组目录全抄一遍
- `拆分后的主题边界` 每个子页只写一句话边界，不写长段正文
- `与其它专题的边界` 只写跨目录分工，不展开子系统细节

## 子页推荐模板

子页可按需要自由组织，但建议至少覆盖：

1. `## 本页用途`
2. `## 相关文件`
3. `## 一句话结论` 或直接进入正文
4. 机制主体
5. 证据与边界
6. `## 当前仍未完全钉死`

子页允许保留证据落点、压缩名、bundle 行号、伪代码链和反证。

## 命名与分层约定

### 目录命名

- 目录名使用稳定主题名，不使用临时整理术语。
- 父页文件名与子目录名保持同名主干，便于直觉跳转。

### 标题命名

- 父页标题优先写“主题名”或“主题名 + 总览语义”。
- 子页标题优先写“机制名 + 结论/边界”。
- 不用“杂项”“补充”“继续整理”这类历史过程命名。

### 编号语义

- 编号表示阅读顺序与主题组织，不表示证据强弱。
- 新增子页时，优先插入到对应主题簇，而不是把不相关内容塞进已有大页。

## 文风约定

- 优先写判断边界，不写空泛总结。
- 优先写“已确认 / 高可信推断 / 待验证”的区别，不把推断写成定论。
- 优先按职责命名，不迷信压缩名。
- 父页语气保持克制，避免写成正文摘要。
- 正文页可以有结论，但要能回到证据或边界。

## 迁移规则

当发现父页已经长成混合页时，按下面顺序处理：

1. 先判断哪些段落其实属于具体问题域。
2. 把这些段落下沉到最合适的现有子页。
3. 若没有合适子页，再新增子页。
4. 父页只保留导航、边界和阅读顺序。

迁移时不要做：

- 在父页保留一份长摘要，同时在子页再保留一份完整正文
- 为了省事把不相关内容塞进“最接近”的子页
- 只删父页内容，不给出新的落点

## 当前优先执行的结构修复

当前最优先按本约定回收的父页包括：

- `02-execution/05-attachments-and-context-modifiers.md`
- `03-ecosystem/07-tui-system.md`

这两页后续应继续收缩到纯导航页形态，把正文判断下沉到各自子页。

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
