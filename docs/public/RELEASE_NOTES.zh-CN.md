# 发布说明

[English](./RELEASE_NOTES.md) | [简体中文](./RELEASE_NOTES.zh-CN.md)

## 2026-03-03：CI 回归矩阵稳定性修复

### 摘要

本次发布聚焦于 CI/设备回归稳定性与测试可重复性，
不包含对外 API 的破坏性变更。

### 修复内容

- 修复 PowerShell 环境下手动触发 `Device Regression Matrix` 时参数序列化问题。
  - 统一使用 `gh workflow run --json` 传递输入。
  - 避免 `targets_json` 引号丢失导致 `prepare-targets` JSON 解析失败。

- 修复 self-hosted Runner 在作业初始化阶段的非业务失败。
  - 从矩阵工作流中移除 `actions/upload-artifact@v4`。
  - 避免 `Set up job` 阶段因 Action 下载超时造成误失败。

- 修复模拟器时序抖动导致的对话框覆盖测试偶发失败。
  - 加固 `integration_testapp_coverage_test::test_dialog_flows_and_wait_gone`。
  - 为 Bottom Sheet 结果增加条件等待与重试确认。

### 验证证据

- 矩阵运行 `22583901973`：成功
  - `mumu-16384`（`127.0.0.1:16384`）通过
  - `ld-emulator-5554`（`emulator-5554`）通过

### 影响范围

- 运行时库行为：无变化。
- 主要收益：CI 更稳定，回归测试误报失败显著减少。
