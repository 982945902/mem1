# mem1 对标执行看板

> 目标：在 2 周内达到可运行、可评测、可对比的 mem1 对标版本。

## 使用方式

- 状态：`TODO` / `DOING` / `DONE` / `BLOCKED`
- 优先级：`P0`（必须）/ `P1`（重要）/ `P2`（优化）

## 任务列表

| 状态 | 优先级 | 任务 | 负责人 | 验收标准 |
|---|---|---|---|---|
| TODO | P0 | API 契约冻结（/memories, /memories/search, /memories/:id） | TBD | 契约文档合并，示例请求/响应可复制运行 |
| TODO | P0 | 存储层 schema/migration | TBD | 新库启动可自动建表，回归测试通过 |
| TODO | P0 | Add/Search handler 单测 | TBD | 覆盖正常流与 user_id 缺失场景 |
| TODO | P0 | `make sample` 评测流程打通 | TBD | sample 数据可产出 results 文件 |
| TODO | P1 | Embedding provider 抽象（off/local/openai） | TBD | 三种模式可通过配置切换 |
| TODO | P1 | local 模型目录校验 + 默认模型下载 | TBD | 默认路径缺模型可自动下载并启动 |
| TODO | P1 | Search 输出 formatted_context（FACTS/ENTITIES） | TBD | 返回字段可直接给下游 LLM 使用 |
| TODO | P2 | CI 整合 format/lint/test | TBD | PR 自动校验并阻断失败提交 |
| TODO | P2 | 检索优化（重排/混合检索/时间衰减） | TBD | 离线评测分数相比基线有提升 |

## 里程碑

- M1（第 1 周末）：P0 全部 DONE。
- M2（第 2 周末）：P1 全部 DONE，完成一轮 LOCOMO 对比。
- M3（持续）：P2 逐步推进，建立稳定迭代节奏。
