[English](./README.md) | [简体中文](./README.zh-CN.md)

# Public Docs

本目录是内部 `.kiro/specs` 的公开化整编版本，用于 GitHub 对外阅读、发布对账和审计留痕。

## 文档来源

归并来源：

- `uiautomator`（Phase1 + ATX 扩展）
- `uiautomator-cli`
- `uiautomator-phase2`（已完成的增强任务流）
- `bugfix`（Selector/兼容性修复）

## 文档清单

- `REQUIREMENTS.md`：完整需求基线（含需求编号、验收标准、发布门槛）
- `DESIGN.md`：完整设计基线（含模块设计、关键流程、错误与可观测性）
- `API_DOCS.md`：公开 API 指南（crate 关系、调用约定、docs.rs/示例入口）
- `MIGRATION.md`：迁移状态与未来破坏性变更说明入口
- `TASKS.md`：任务台账（含已完成、未完成、阻塞性、执行优先级）
- `TESTING_RELEASE.md`：测试与发布流程（脚本化回归、结构化摘要、发布顺序）
- `RELEASE_NOTES.md`：发布说明（含关键修复点与可追溯验证证据）

## 使用方式

1. 先读 `REQUIREMENTS.md` 确认功能边界与验收口径。
2. 再读 `DESIGN.md` 对应实现架构与关键决策。
3. 用 `API_DOCS.md` 查看公开 API 结构、docs.rs 和示例入口。
4. 用 `TASKS.md` 对账当前进度与后续计划。
5. 按 `TESTING_RELEASE.md` 执行发布前验证。

## 状态约定

- `已完成`：已实现并有验证证据
- `发布工程`：不改变核心 API 的工程增强
- `Phase2`：增强任务，默认不阻塞当前发布

