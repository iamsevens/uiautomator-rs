# Release Notes

[English](./RELEASE_NOTES.md) | [简体中文](./RELEASE_NOTES.zh-CN.md)

## 2026-03-03: CI Regression Matrix Stability

### Summary

This release focuses on CI/device-regression stability and test determinism.
No public API breaking changes.

### Fixed

- Fixed `Device Regression Matrix` manual dispatch payload handling on PowerShell.
  - Use `gh workflow run --json` to avoid quote loss in `targets_json`.
  - Prevents JSON parse failures in `prepare-targets`.

- Fixed non-functional workflow failures on self-hosted runner setup.
  - Removed `actions/upload-artifact@v4` from matrix workflow.
  - Avoids transient action-download timeout failures during `Set up job`.

- Stabilized dialog coverage test under emulator timing variance.
  - Hardened `integration_testapp_coverage_test::test_dialog_flows_and_wait_gone`.
  - Added conditional wait/retry checks for Bottom Sheet result confirmation.

### Validation Evidence

- Matrix run: `22583901973` (success)
  - `mumu-16384` (`127.0.0.1:16384`) passed
  - `ld-emulator-5554` (`emulator-5554`) passed

### Impact

- Runtime library behavior: unchanged.
- Main impact: better CI reliability and fewer false-negative regression failures.
