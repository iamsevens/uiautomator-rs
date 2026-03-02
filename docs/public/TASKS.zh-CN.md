[English](./TASKS.md) | [简体中文](./TASKS.zh-CN.md)

# Tasks Ledger

## 1. 文档目的

本文件用于把 `.kiro/specs` 里的任务状态公开化、可审计化，回答三个问题：

1. 哪些任务已经完成并可作为发布依据。
2. 哪些任务还没做，是否阻塞当前发布。
3. 接下来按什么顺序做，才能最低风险推进。

## 2. 来源与归并规则

任务来源：

- `.kiro/specs/uiautomator/tasks.md`
- `.kiro/specs/uiautomator-cli/tasks.md`
- `.kiro/specs/uiautomator-phase2/tasks.md`
- `.kiro/specs/bugfix/tasks.md`

状态定义：

- `Done`: 已实现并具备验证证据
- `Open`: 未完成
- `Release-Blocking`: 阻塞当前发布
- `Post-Release`: 建议发布后推进

## 3. 总览状态

| 任务域 | 状态概览 | 发布影响 |
|---|---|---|
| uiautomator Phase1 + ATX | 已完成（1~19, 18A/18B） | 已满足 |
| uiautomator-cli 基础 + 工程增强 | 1~25 已完成 | 已满足 |
| bugfix Selector 修复 | 1~3 全部完成 | 已满足 |
| phase2 核心增强 | 1/2/3/4 + 5.1 已完成，其余未完 | 默认不阻塞当前版本 |

## 4. 已完成任务台账（可作为发布证据）

### 4.1 核心库 `uiautomator`

已完成（来自 `uiautomator/tasks.md`）：

- 1~4：项目基础设施、ADB、JSON-RPC、数据模型
- 5~14：Selector、UiObject、Device 核心与 API 导出
- 15~17：单元测试、集成测试、阶段检查点
- 18A：ATX-Agent 连接模式全链路
- 18B：ATX-Agent 安装模式全链路
- 19：文档更新与发布准备

证据要点：

- 任务清单标记为已完成
- 对应功能已纳入回归脚本矩阵

### 4.2 工具链 `uiautomator-cli`

已完成（来自 `uiautomator-cli/tasks.md`）：

- 1~20：CLI 命令、安装器、错误处理、测试、构建发布
- 21：API 覆盖对账自动化
- 22：测试日志标准化（UTF-8 + JSON/JUnit）
- 23：多架构/多版本回归矩阵
- 24：安装后可用性烟测（`cargo install` 路径）
- 25：Nightly 回归守门

证据要点：

- CLI 与核心库全量测试链路已打通
- 结构化产物已作为回归输出标准
- 设备回归矩阵在两个模拟器目标上均成功：
- Run `22583901973`
- `mumu-16384`（`127.0.0.1:16384`）
- `ld-emulator-5554`（`emulator-5554`）

### 4.3 Bugfix 专项

已完成（来自 `bugfix/tasks.md`）：

- 1：mask 修复（P0）
- 2：扩展字段补齐（P1）
- 3：child/sibling 层级选择器（P2）

证据要点：

- 选择器序列化语义与 Python 对齐
- 相关测试任务全部勾选完成

## 5. 未完成任务台账

### 5.1 Phase2（核心增强，默认不阻塞当前发布）

来源：`uiautomator-phase2/tasks.md`

1. `Task 1` 细化错误类型系统
- `1.1` 已完成
- `1.2` 已完成：现有代码已迁移到新错误类型
- `1.3` 已完成：文档/示例已同步

2. `Task 5` API 一致性改进（Open）
- `5.1` 已完成：统一超时参数
- `5.2` Open：统一坐标参数（可选）
- `5.3` Open：文档审查

3. `Task 6` Mock 测试覆盖（Open）
- `6.1` Open：基础设施
- `6.2` Open：核心功能 mock
- `6.3` Open：错误场景 mock

4. `Task 7` 性能优化-设备信息缓存（Open，可选）
- `7.1` ~ `7.4` 均 Open

## 6. 发布阻塞判定

### 6.1 当前阻塞项

当前无明确 Release-Blocking Open 项。

### 6.2 发布基线确认清单

1. 核心能力（FR-UA/FR-CLI/FR-BUG）均已落地。
2. 多设备全量回归（含 ignored）已可执行并有日志。
3. API 覆盖与结构化摘要可生成。
4. 包内容与发布顺序已有流程约束。

结论：当前可发布；剩余项属于“可持续发布能力增强”。

## 7. 下一步执行顺序（建议）

### P0（发布后第一批）

1. 完成 Phase2 Task 5.3：API 文档一致性审查

理由：当前代码已完成错误类型与超时参数收敛，下一步优先补齐对外文档一致性。

### P1（发布后第二批）

1. 推进 Phase2 Task 6：Mock 测试体系化

理由：提升回归可信度并补齐错误/重试场景的可重复验证能力。

### P2（稳定性增强）

1. Phase2 Task 5.2：坐标参数统一（可选）
2. Phase2 Task 7：缓存优化（可选）

## 8. 任务到文档映射

- 需求基线：`docs/public/REQUIREMENTS.md`
- 设计基线：`docs/public/DESIGN.md`
- 测试与发布：`docs/public/TESTING_RELEASE.md`

保持原则：任务状态变更时，三份文档必须同步更新。

## 9. 维护规则

1. 新任务先写入内部 `.kiro/specs`，再归并到本公开台账。
2. 公开文档只保留“对外必要信息”，内部过程细节留在内部文档。
3. 每次发布前执行一次任务对账，避免“代码已变更、台账未更新”。

