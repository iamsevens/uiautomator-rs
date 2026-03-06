# API Documentation Guide

[English](./API_DOCS.md) | [简体中文](./API_DOCS.zh-CN.md)

## 1. Purpose

This document is the public API guide for `uiautomator-rs`.
It complements generated rustdoc with a stable overview of crate relationships, API conventions, and example locations.

## 2. Generated API References

- Core library: [docs.rs/uiautomator](https://docs.rs/uiautomator)
- CLI library: [docs.rs/uiautomator-cli](https://docs.rs/uiautomator-cli)

Use the generated docs for complete item-level signatures and examples.
Use this guide for the higher-level contract across crates.

## 3. Crate Relationship

- `uiautomator`: async library for Android UI automation through ADB, JSON-RPC, and optional ATX-Agent transport
- `uiautomator-cli`: operational companion for ATX-Agent environment setup and service lifecycle management

Recommended usage:

1. Provision the target device with `uiautomator-cli init`.
2. Use the `uiautomator` library in application code.
3. Prefer `Auto` mode unless you have a strong reason to force `Direct` or `AtxAgent`.

## 4. Core Library API Surface

### `Device`

Primary entry point for:

- device connection and mode selection
- element lookup through `find`
- gestures and key input
- screenshots and app lifecycle operations

Common entry methods:

- `Device::connect`
- `Device::connect_with_mode`
- `Device::info`
- `Device::find`
- `Device::click`
- `Device::click_coord`
- `Device::long_click_coord`
- `Device::double_click_coord`
- `Device::swipe`
- `Device::swipe_coord`
- `Device::drag_coord`
- `Device::set_cache_ttl`
- `Device::clear_cache`
- `Device::disable_cache`
- `Device::app_start`
- `Device::app_stop`
- `Device::app_wait`

### `Selector`

Builder-style query description for UI lookup.

Supported selector families include:

- text and regex text matching
- resource id, class name, description
- boolean state flags
- `index` / `instance`
- hierarchy selectors such as `child` and `sibling`

### `Coord`

Unified coordinate value for public APIs that need either:

- absolute pixels: `Coord::pixel(200)`
- relative percentages: `Coord::percent(0.5)`

Coordinate-aware helpers keep existing pixel APIs intact and add explicit alternatives such as:

- `Device::click_coord`
- `Device::long_click_coord`
- `Device::double_click_coord`
- `Device::swipe_coord`
- `Device::drag_coord`

### `UiObject`

Lazy element handle derived from a `Selector`.

Common operations:

- `exists`
- `wait`
- `wait_gone`
- `click`
- `long_click`
- `set_text`
- `clear_text`
- `get_text`
- `info`
- `bounds`
- `center`

## 5. CLI API Surface

`uiautomator-cli` exposes both a command-line interface and reusable installer helpers.

Operational commands:

- `init`
- `status`
- `restart`
- `uninstall`
- `version`

Reusable library entry points:

- `Installer::new`
- `Installer::install`
- `Installer::status`
- `Installer::restart`
- `Installer::uninstall`

## 6. API Conventions

### Async

- Device and UiObject operations are async-first.
- Use `tokio` or another compatible async runtime.

### Return Values

- Fallible operations return `uiautomator::Result<T>` or CLI `Result<T>`.
- Bool-returning methods such as `exists` use `Ok(false)` for expected negative states and reserve `Err(...)` for transport, protocol, or semantic failures.

### Timeouts

- Timeout-bearing library APIs use `Option<Duration>`.
- `None` means "use configured/default timeout".

### Modes

- `Auto`: try ATX-Agent first, then fallback to Direct
- `AtxAgent`: require ATX-Agent path
- `Direct`: bypass ATX-Agent and talk to the JSON-RPC runtime directly

### Caching

- `Device::info()` is uncached by default.
- `Device::set_cache_ttl(...)` enables opt-in device-info caching.
- `Device::clear_cache()` forces the next `info()` call to re-fetch.
- `Device::disable_cache()` restores always-live reads.

### Errors

Public APIs use structured error variants instead of broad string failures.
Important categories include:

- device selection / offline errors
- RPC and transport errors
- element-not-found / timeout errors
- app lifecycle errors

## 7. Examples and Test References

Published crate packages do not include the full repository `tests/` and `examples/` trees.
Use the GitHub repository for complete reference material:

- `uiautomator/examples/`
- `uiautomator/tests/`
- `uiautomator-cli/tests/`
- `test-app/`

Generated rustdoc examples are maintained at full coverage and serve as the quickest per-item reference.

## 8. Documentation Quality Status

Current baseline:

- public API docs coverage: `100%`
- public API example coverage: `100%`

Verification path:

- script: `scripts/docs-coverage-report.ps1`
- latest summaries:
  - `internal/testlogs/docs/latest-summary.json`
  - `internal/testlogs/docs/latest-summary.md`
- latest verified aggregate:
  - docs `331/331`
  - examples `187/187`
- CI guard:
  - `.github/workflows/docs-coverage.yml`

## 9. Related Documents

- `REQUIREMENTS.md`
- `DESIGN.md`
- `TASKS.md`
- `TESTING_RELEASE.md`
- `RELEASE_NOTES.md`
