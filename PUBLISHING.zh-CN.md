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
3. Run package checks from repo root:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\release-check.ps1
```

If you intentionally run checks before committing, use:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\release-check.ps1 -AllowDirty
```

```bash
bash ./scripts/release-check.sh
```

If you intentionally run checks before committing, use:

```bash
bash ./scripts/release-check.sh --allow-dirty
```

4. Inspect package content manually when needed:

```bash
cd uiautomator && cargo package --list
cd ../uiautomator-cli && cargo package --list
```

5. Run dry-run publish in order:

```bash
cd uiautomator && cargo publish --dry-run
cd ../uiautomator-cli && cargo publish --dry-run
```

说明：
- 由于 `tests/**` 与 `examples/**` 不进入发布包，`cargo publish --dry-run` 可能提示 “ignoring example/test ... not included in the published package”，这是预期行为。

## Publish Order

1. Publish `uiautomator` first.
2. Wait until crates.io index resolves the new version.
3. Publish `uiautomator-cli`.

## When You Add New Files

1. `tests/**` 与 `examples/**` 默认不进入发布包；如果要发布这些内容，需要显式修改对应 crate 的 `include`。
2. 新增运行时必需文件（例如 `assets/**`、`src/**` 下的新文件）后，确认已被 `include` 覆盖。
3. 新增顶层发布文档（例如 `MIGRATION.md`）需要显式加入对应 crate 的 `include`。

