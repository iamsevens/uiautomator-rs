[English](./CONTRIBUTING.md) | [简体中文](./CONTRIBUTING.zh-CN.md)

# Contributing Guide

感谢你为 `uiautomator-rs` 做贡献。

本仓库包含两个发布项目和一个测试夹具目录（无 workspace）：
- `uiautomator/`：核心 Rust 库
- `uiautomator-cli/`：CLI 工具
- `test-app/`：Android 测试 APK 工程（仅仓库内测试用途，不对外发布）

## 开发原则

- 提交前确保改动最小、可验证、可回滚。
- 公共 API 变更必须补充文档与测试。
- 优先使用约定式提交前缀：`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`, `perf:`, `ci:`

## 本地检查

### uiautomator

```bash
cd uiautomator
cargo fmt --check
cargo clippy
cargo test --lib
```

### uiautomator-cli

```bash
cd uiautomator-cli
cargo fmt --check
cargo clippy
cargo test --lib
```

## 发布门禁检查

发布流程与门禁以 [PUBLISHING.md](./PUBLISHING.md) 为唯一入口。
本指南不再重复发布命令。

## 文档覆盖率检查

仓库根目录执行：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\docs-coverage-report.ps1
```

如需启用阈值失败（用于 CI 守门）：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\docs-coverage-report.ps1 -FailOnThreshold -MinDocsPercent 99.0 -MinExamplesPercent 55.0
```

## Pull Request 要求

- 说明变更背景和目标。
- 列出验证方式（命令、设备/模拟器环境）。
- 若涉及行为变化，补充迁移说明。
- 不要将临时报告、任务记录等内部文件放入 crate 根目录。

## 行为规范

参与本项目即表示同意遵守 `CODE_OF_CONDUCT.md`。



