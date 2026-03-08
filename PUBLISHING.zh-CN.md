[English](./PUBLISHING.md) | [简体中文](./PUBLISHING.zh-CN.md)

# Publishing Process

This repository keeps `.kiro/` in git on purpose, but crate packages must stay clean.
Use this file as the fixed release process for `uiautomator` and `uiautomator-cli`.

## Rules

1. Internal progress files belong to `internal/archive/`, not crate roots.
2. Never keep `TASK_*`, `*_REPORT.md`, `*_SUMMARY.md`, `MANUAL_TEST_*.md` inside `uiautomator/` or `uiautomator-cli/`.
3. Crate content is controlled by `[package].include` in:
`uiautomator/Cargo.toml`
`uiautomator-cli/Cargo.toml`
4. `tests/**` 和 `examples/**` 保留在 GitHub 仓库，不进入 crates 发布包。
5. `test-app/` 仅作为仓库内测试夹具，不参与 crates 发布。
6. If you add files outside included paths and want them published, update the corresponding `include` list.

## Pre-Release Checklist

1. Bump versions and keep dependency versions aligned:
   - `uiautomator/Cargo.toml` -> `[package].version`
   - `uiautomator-cli/Cargo.toml` -> `[package].version`
   - `uiautomator-cli/Cargo.toml` -> `dependencies.uiautomator.version`
2. Ensure local internal records are in `internal/archive/`.
3. 通过统一脚本串行执行 GitHub 发布门禁（唯一入口）：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\trigger-gh-release-gate.ps1 -Repo iamsevens/uiautomator-rs -Ref main
```

该脚本会按顺序执行 `Release Check` 和 `Publish Dry Run`，任一失败即立即返回失败。

4. 运行文档/示例覆盖率检查：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\docs-coverage-report.ps1 -FailOnThreshold -MinDocsPercent 99.0 -MinExamplesPercent 55.0
```

5. 需要时可手动检查打包内容：

```bash
cd uiautomator && cargo package --list
cd ../uiautomator-cli && cargo package --list
```

说明：
- 由于 `tests/**` 与 `examples/**` 不进入发布包，`Publish Dry Run` 可能提示 “ignoring example/test ... not included in the published package”，这是预期行为。

## Publish Order

1. Publish `uiautomator` first.
2. Wait until crates.io index resolves the new version.
3. Publish `uiautomator-cli`.

## 固化发布流程（务必严格执行）

1. 版本与文档同步：
   - 同时升级 `uiautomator` 与 `uiautomator-cli` 版本号，并同步依赖版本。
   - 更新所有公开 README 的安装示例版本号。
   - 两个 `CHANGELOG.md` 都要补齐新版本条目。
   - 先提交并推送，再跑门禁。
2. 发布前检查：
   - 执行 `docs-coverage-report.ps1`（文档 ≥99%，示例 ≥55%）。
   - 执行 `trigger-gh-release-gate.ps1`（Release Check + Publish Dry Run）。
3. 按顺序发布：
   - 先发布 `uiautomator`。
   - 用 `cargo info uiautomator@X.Y.Z` 确认 crates.io 索引可见。
   - 确认后再发布 `uiautomator-cli`。
4. 打 tag 与 Release：
   - 打 `vX.Y.Z` 标签并推送。
   - crates 发布完成后再创建 GitHub Release（确保链接可用）。

## 已踩坑汇总与规避方法

- **CLI dry-run 报错**：`failed to select a version for uiautomator`  
  原因是 `uiautomator` 版本尚未出现在 crates.io 索引。必须先发布核心库并等待索引刷新。
- **crates 发布报 HTTP/2 stream error**  
  使用 `CARGO_HTTP_MULTIPLEXING=false` 重试。
- **PowerShell 不支持 `&&`**  
  Windows 下用两条命令分开执行。
- **仅文档改动仍需升版**  
  crates.io/docs.rs 不允许覆盖同版本内容。

## When You Add New Files

1. `tests/**` 与 `examples/**` 默认不进入发布包；如果要发布这些内容，需要显式修改对应 crate 的 `include`。
2. 新增运行时必需文件（例如 `assets/**`、`src/**` 下的新文件）后，确认已被 `include` 覆盖。
3. 新增顶层发布文档（例如 `MIGRATION.md`）需要显式加入对应 crate 的 `include`。



