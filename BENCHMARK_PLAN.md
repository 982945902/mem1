# 对标 mem1 的最小可交付方案（MVP）

本文给出一个可执行的、分阶段的对标方案，目标是在 1~2 周内交付可用版本，并可用 LOCOMO 流水线评估。

## 1. 目标定义

- 对齐能力：Memory CRUD + Search + 可选 Embedding + 评测闭环。
- 对齐接口：`/memories`、`/memories/search`、`/memories/:id`。
- 对齐结果：可返回结构化上下文（FACTS + ENTITIES）供下游 LLM 使用。

## 2. 分阶段计划

### P0（第 1 周）

1. **服务基础能力**
   - 启动配置：`MEM1_BIND`、`MEM1_DB_PATH`。
   - API：新增/搜索/获取/删除。
   - `user_id` 维度隔离。
2. **基础检索能力**
   - 无向量时关键词检索。
   - 返回 `score` 字段（关键词模式可为空或规则分）。
3. **验收标准**
   - `curl` 可完整跑通 CRUD + Search。
   - 可运行小样本评测（sample）。

### P1（第 2 周）

1. **Embedding 能力对齐**
   - `off/local/openai` 三种 provider。
   - 本地模型目录校验与自动下载默认模型。
2. **搜索结果增强**
   - `formatted_context` 组织为 FACTS / ENTITIES。
   - 支持 `limit` 与默认值。
3. **验收标准**
   - 在同一数据集上，向量检索结果优于纯关键词检索。

### P2（持续迭代）

1. 质量：重排、混合检索、时间衰减。
2. 工程：压测、监控、错误预算。
3. 生态：SDK 扩展（TS/Go）、部署模板（Docker/K8s）。

## 3. 建议的任务拆解（可直接建 issue）

- [ ] API 契约冻结（OpenAPI 或 markdown）
- [ ] 存储层 schema 与 migration
- [ ] Add/Search handler 单测
- [ ] Embedding provider 抽象
- [ ] Local embed 加载/下载逻辑
- [ ] Evaluation 一键脚本
- [ ] CI：format/lint/test

## 4. 风险与规避

- 向量模型下载失败：降级关键词检索，保持可用。
- 评测不稳定：固定模型与温度，记录提示词版本。
- 数据隔离风险：所有读写都强制携带 `user_id`。

## 5. 最小验收命令

```bash
# 1) 启动服务
cd mem1-server && cargo run

# 2) 运行评测（另开终端）
cd evaluation && make sample
```

若 sample 跑通并输出结果文件，则 MVP 达标。


## 6. 直接执行入口

- 任务看板：`TODO_BENCHMARK.md`
- 快速评测命令：
  - `just benchmark-sample`（小样本）
  - `just benchmark-full`（完整数据）

说明：上述命令依赖 `mem1-server` 先启动（默认 `127.0.0.1:8080`）。
