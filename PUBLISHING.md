# Publishing Process

[English](./PUBLISHING.md) | [简体中文](./PUBLISHING.zh-CN.md)

This repository keeps internal planning files out of the public package surface.
Use this as the fixed release process for `uiautomator` and `uiautomator-cli`.

## Rules

1. Internal progress records belong to `internal/archive/`, not crate roots.
2. Do not keep `TASK_*`, `*_REPORT.md`, `*_SUMMARY.md`, `MANUAL_TEST_*.md` inside `uiautomator/` or `uiautomator-cli/`.
3. Crate package content is controlled by `[package].include` in:
- `uiautomator/Cargo.toml`
- `uiautomator-cli/Cargo.toml`
4. `tests/**` and `examples/**` stay in GitHub but are excluded from crates packages by default.
5. `test-app/` is a repository test fixture and is not published to crates.io.
6. If new files must be shipped, update the corresponding crate `include` list.

## Pre-release Checklist

1. Bump versions and keep dependency versions aligned:
- `uiautomator/Cargo.toml` -> `[package].version`
- `uiautomator-cli/Cargo.toml` -> `[package].version`
- `uiautomator-cli/Cargo.toml` -> `dependencies.uiautomator.version`

2. Ensure internal records are archived under `internal/archive/`.

3. Run package checks from repository root:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\release-check.ps1
```

If intentionally running before commit:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\release-check.ps1 -AllowDirty
```

```bash
bash ./scripts/release-check.sh
```

If intentionally running before commit:

```bash
bash ./scripts/release-check.sh --allow-dirty
```

4. Inspect package content if needed:

```bash
cd uiautomator && cargo package --list
cd ../uiautomator-cli && cargo package --list
```

5. Run docs/examples coverage report:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\docs-coverage-report.ps1 -FailOnThreshold -MinDocsPercent 99.0 -MinExamplesPercent 55.0
```

6. Run the GitHub release gate workflows (recommended single entrypoint):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\trigger-gh-release-gate.ps1 -Repo iamsevens/uiautomator-rs -Ref main
```

This script runs `Release Check` then `Publish Dry Run` in sequence and fails fast on the first non-success run.

7. (Optional local verification) Run dry-run publish in order:

```bash
cd uiautomator && cargo publish --dry-run
cd ../uiautomator-cli && cargo publish --dry-run
```

Note:
`cargo publish --dry-run` may report ignored tests/examples not included in package; this is expected by design.

## Publish Order

1. Publish `uiautomator` first.
2. Wait until crates.io index resolves the new version.
3. Publish `uiautomator-cli`.

## When Adding New Files

1. `tests/**` and `examples/**` are excluded by default.
2. Runtime-required files (for example under `assets/**` or `src/**`) must be covered by `include`.
3. New publish-facing docs (for example `MIGRATION.md`) must be explicitly added to `include`.


