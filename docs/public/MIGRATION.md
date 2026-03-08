# Migration Guide

[English](./MIGRATION.md) | [简体中文](./MIGRATION.zh-CN.md)

## 1. Current Status

There is currently no published breaking migration that requires user action.

The public `1.x` line is being kept backward compatible where practical, and recent additions such as:

- `Coord`
- `Device::click_coord`
- `Device::long_click_coord`
- `Device::double_click_coord`
- `Device::swipe_coord`
- `Device::drag_coord`
- `Device::set_cache_ttl`
- `Device::clear_cache`
- `Device::disable_cache`

were added as non-breaking extensions.

## 2. What To Expect In Future Versions

If a future release introduces a user-visible breaking change, this document will record:

1. The source version and target version.
2. The affected API surface.
3. The exact replacement pattern.
4. Whether the change is source-breaking, behavior-breaking, or environment-breaking.
5. Whether an automated migration or compatibility shim exists.

## 3. Current Upgrade Guidance

For upgrades within the current published line:

1. Re-run your normal device regression after dependency bump.
2. If you use ATX setup flows, re-check `uiautomator-cli init` on at least one real device and one emulator.
3. If you want coordinate APIs with percent support, prefer the `*_coord` methods instead of rewriting existing pixel-based calls.
4. Keep device-info caching opt-in unless your call path explicitly benefits from it.

## 4. Related Documents

- `REQUIREMENTS.md`
- `DESIGN.md`
- `API_DOCS.md`
- `TESTING_RELEASE.md`
