# Publishing Process

[English](./PUBLISHING.md) | [简体中文](./PUBLISHING.zh-CN.md)

This repository keeps internal planning files out of the public package surface.
Use this as the fixed release process for `uiautomator` and `uiautomator-cli`.
Treat this document as the single source of truth for releases.

## Prerequisites

1. Working tree is clean (`git status -sb` shows no changes).
2. GitHub CLI is authenticated (`gh auth status`).
3. crates.io credentials are available:
   - `cargo login` done locally, or `CARGO_REGISTRY_TOKEN` set.
4. If your network is flaky, plan to set `CARGO_HTTP_MULTIPLEXING=false` for publish retries.

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

3. Run the GitHub release gate workflows (single entrypoint):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\trigger-gh-release-gate.ps1 -Repo iamsevens/uiautomator-rs -Ref main
```

This script runs `Release Check` then `Publish Dry Run` in sequence and fails fast on the first non-success run.

4. Run docs/examples coverage report:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\docs-coverage-report.ps1 -FailOnThreshold -MinDocsPercent 99.0 -MinExamplesPercent 55.0
```

5. Inspect package content if needed:

```bash
cd uiautomator && cargo package --list
cd ../uiautomator-cli && cargo package --list
```

Note:
`Publish Dry Run` may report ignored tests/examples not included in package; this is expected by design.

## Publish Order

1. Publish `uiautomator` first.
2. Wait until crates.io index resolves the new version.
3. Publish `uiautomator-cli`.

## Hardened Release Flow (Do Not Skip)

1. Update versions and docs:
   - Bump `uiautomator` + `uiautomator-cli` versions and dependency version.
   - Update all public README install snippets to the new version.
   - Add the new version entry to both `CHANGELOG.md` files.
   - Commit and push before running any release gate.
2. Run pre-release checks:
   - `docs-coverage-report.ps1` (must be 99%+ docs and 55%+ examples).
   - `trigger-gh-release-gate.ps1` (Release Check + Publish Dry Run).
3. Publish crates in order:
   - Publish `uiautomator` first.
   - Confirm the version is visible in the crates.io index:
     - `cargo info uiautomator@X.Y.Z`
   - Publish `uiautomator-cli` only after the above succeeds.
4. Tag and release:
   - Create git tag `vX.Y.Z` and push.
   - Create GitHub Release after crates publish (so links are live).

## Post-release Verification

1. Confirm crates.io visibility:
   - `cargo info uiautomator@X.Y.Z`
   - `cargo info uiautomator-cli@X.Y.Z`
2. Confirm docs.rs pages respond for the new version.

## Known Pitfalls and Fixes

- **CLI dry-run fails** with `failed to select a version for uiautomator`:  
  `uiautomator` is not yet visible in crates.io index. Publish core first and wait for index propagation.
- **Crates publish fails with HTTP/2 stream errors**:  
  Retry with `CARGO_HTTP_MULTIPLEXING=false` in the environment.
- **PowerShell command chaining**:  
  Use separate commands instead of `&&` in Windows PowerShell.
- **Docs-only changes still require version bump**:  
  crates.io/docs.rs do not allow overwriting an existing version.

## When Adding New Files

1. `tests/**` and `examples/**` are excluded by default.
2. Runtime-required files (for example under `assets/**` or `src/**`) must be covered by `include`.
3. New publish-facing docs (for example `MIGRATION.md`) must be explicitly added to `include`.


