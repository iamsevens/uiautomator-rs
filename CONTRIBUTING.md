# Contributing Guide

[English](./CONTRIBUTING.md) | [简体中文](./CONTRIBUTING.zh-CN.md)

Thanks for contributing to `uiautomator-rs`.

## Repository Components

This repository contains two publishable crates and one test fixture project (no workspace):

- `uiautomator/`: core Rust library
- `uiautomator-cli/`: CLI tool
- `test-app/`: Android fixture APK project (repo-only test asset)

## Contribution Principles

- Keep changes small, verifiable, and reversible.
- Public API changes must include docs and tests.
- Use conventional commit prefixes when possible:
  - `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`, `perf:`, `ci:`

## Local Checks

### `uiautomator`

```bash
cd uiautomator
cargo fmt --check
cargo clippy
cargo test --lib
```

### `uiautomator-cli`

```bash
cd uiautomator-cli
cargo fmt --check
cargo clippy
cargo test --lib
```

## Release Gate Check

Release workflow and gates are centralized in [PUBLISHING.md](./PUBLISHING.md).
This guide intentionally avoids duplicating release commands.

## Docs Coverage Check

From repository root:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\docs-coverage-report.ps1
```

Optional threshold enforcement:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\docs-coverage-report.ps1 -FailOnThreshold -MinDocsPercent 99.0 -MinExamplesPercent 55.0
```

## Pull Request Requirements

- Explain motivation and scope.
- List validation commands and device/emulator context.
- Include migration notes when behavior changes.
- Do not add temporary reports/internal logs into crate roots.

## Conduct

By participating, you agree to follow [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md).


