[English](./TESTING_RELEASE.md) | [简体中文](./TESTING_RELEASE.zh-CN.md)

# 发布说明

发布步骤以 `PUBLISHING.zh-CN.md` 为唯一入口。
本文档只描述测试范围与发布证据，不再重复发布流程。

# Testing And Release Baseline

## 1. 文档目的

本文件定义对外发布前的测试与发布基线，解决三个问题：

1. 如何确保环境从“无到有”搭建后，功能仍然可用。
2. 如何区分真实产品失败与环境噪声。
3. 如何形成可审计的发布证据。

## 2. 适用范围与输入

- `uiautomator` 核心库发布验证
- `uiautomator-cli` 工具链发布验证

输入来源：

- `.kiro/specs/uiautomator*/tasks.md`
- `.kiro/specs/bugfix/tasks.md`
- `scripts/device-full-test.ps1`
- `scripts/run-validation-gate.ps1`
- `scripts/api-coverage-report.ps1`
- `scripts/docs-coverage-report.ps1`
- `scripts/unit-coverage-report.ps1`
- `scripts/trigger-gh-device-regression.ps1`

## 3. 测试分层

- `unit`: 纯逻辑/模型/序列化/错误映射
- `integration`: 真实设备/模拟器端到端
- `ignored`: 环境依赖强或耗时较长的测试

### `non-ignored` vs `ignored`

- `non-ignored`: 日常 CI 和本地默认会运行
- `ignored`: 仅在明确需要时手动运行

## 4. 最小发布矩阵

发布前建议至少执行两条基线：

- `non-ignored` 全量（库 + CLI）
- `ignored` 全量（库 + CLI）

## 5. 证据与记录

- 记录设备型号、ABI、Android 版本
- 保存 `summary.json` / `summary.junit.xml`
- 记录发布门禁运行链接（参见 `PUBLISHING.zh-CN.md`）

## 6. 归档建议

- 内部记录放到 `internal/archive/`
- 对外发布只保留 README 与公共文档

## 7. 附：发布证据模板

```
## Release Evidence - <version>

- 发布门禁运行链接（参见 `PUBLISHING.zh-CN.md`）：<url>
- docs/examples coverage: pass/fail
- device regression: pass/fail
- Release: yes/no
```
