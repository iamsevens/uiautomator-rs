# Quality Baseline

[English](./QUALITY_BASELINE.md) | [简体中文](./QUALITY_BASELINE.zh-CN.md)

## 1. Purpose

This document captures the stable outcomes of a validated full-project code review pass.

It records long-lived quality boundaries, hardened behavior, and maintenance follow-ups. It does not track rolling batch progress, temporary counters, or local scanning workflow details.

## 2. Current Baseline

- The full-project review sweep has been completed.
- Confirmed correctness issues found during that review have either been fixed or explicitly reclassified as design boundaries.
- There are no remaining known blocking correctness defects from that validated pass.
- The maintainability items identified during the same review have also been closed out and are not left as open action items.

## 3. Verified Hardening Outcomes

The current baseline includes these durable improvements:

- ADB timeout-shell paths now have bounded worker creation.
- `UiObject::click` and `long_click` now honor wait timeout semantics.
- Pointer-style actions validate JSON-RPC boolean results instead of silently accepting failed operations.
- `Settings` now defensively clamps `max_retry=0` and `polling_interval=0`.
- JSON-RPC and ATX-Agent temp-file handling now uses unique filenames and fallible path conversion instead of unsafe `unwrap()`.
- Both Direct-mode and ATX-Agent client teardown now clean up local port forwards.
- CLI uninstall no longer reports success after critical cleanup failures.
- Real-device installation/service tests now restore device state and wait for readiness instead of relying on fixed sleeps.
- The Android test app has been stabilized for automation usage by fixing unbounded memory retention, overlapping callbacks, reset-message overwrite, and unreliable main-menu scroll layout.

## 4. Explicit Design Boundaries

### 4.1 Selector Semantics Boundary

- Device-side `objInfo(selector)` remains the authority for full selector semantics.
- The client only performs defensive revalidation for selector dimensions that `ElementInfo` can represent directly, such as exact match, contains, starts_with, boolean flags, and selected regex-backed fields.
- `index`, `instance`, and `child`/`sibling` hierarchy semantics are not replayed on the client.

Reasoning:

- `ElementInfo` does not contain enough structure to reconstruct hierarchy-sensitive selector semantics without loss.
- Replaying those rules in Rust would create a second selector engine that could drift from Android and Python `uiautomator2` behavior.

### 4.2 Policy-Choice Items

The following are treated as current policy choices rather than hidden correctness bugs:

- `Rect::width()` / `height()` use saturating arithmetic as defensive normalization.
- `build.rs` handling of missing install assets is a packaging/release-policy concern rather than a runtime correctness failure.
- Some serialization and mutex constraints in integration tests remain intentional test-design tradeoffs for real-device compatibility.

## 5. Maintainability Closure

The following maintainability issues were closed without changing public behavior:

- Large inline unit-test modules were extracted out of `device.rs`.
- Large inline unit-test modules were extracted out of `selector.rs`.
- `Key` name/keycode conversion now relies on shared metadata rather than separate hand-maintained tables.
- Test-app main-menu navigation wiring now uses a helper instead of repeated button boilerplate.
- CLI embedded-resource helpers are now exercised by the `version` command instead of existing only in tests/docs.

## 6. Verification Baseline

This baseline assumes the following checks pass:

- `uiautomator/`: `cargo test --lib`
- `uiautomator-cli/`: `cargo test`
- `test-app/`: `gradlew.bat assembleDebug` or `./gradlew assembleDebug`

Notes:

- Real-device `ignored` tests still require external device availability and are not replaced by local no-device runs.
- Release-grade verification still follows [`TESTING_RELEASE.md`](./TESTING_RELEASE.md).

## 7. How To Use

Read this document when you need to:

- check whether a concern was already validated and closed during the full-project review
- decide whether a behavior difference is a bug or an intentional design boundary
- continue quality work without rereading local review-process files

For deeper architectural rationale, continue with [`DESIGN.md`](./DESIGN.md).
