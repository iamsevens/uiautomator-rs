# Requirements Baseline

[English](./REQUIREMENTS.md) | [简体中文](./REQUIREMENTS.zh-CN.md)

## 1. Purpose

This file defines the public requirements baseline for release decisions and implementation traceability.
It is a normalized view of internal specs, not a lightweight summary.

## 2. Source Mapping

| Public Domain | Internal Source | Status |
|---|---|---|
| Public docs and examples coverage | `docs-examples-quality/tasks.md` | Done |
| Core library capabilities | `uiautomator/requirements.md` (1-12) | Done |
| CLI environment management | `uiautomator-cli/requirements.md` (1-7) | Done |
| Selector compatibility fixes | `bugfix/requirements.md` (1-5) | Done |
| Phase2 enhancements | `uiautomator-phase2/requirements.md` (1-7) | Done |

## 3. Glossary

- `Device`: Android device/emulator connected through ADB
- `Selector`: query condition set for locating UI elements
- `UiObject`: element operation object bound to a selector
- `Direct`: direct JSON-RPC transport to device service
- `ATX-Agent`: device-side daemon exposing REST + JSON-RPC forwarding
- `Auto`: prefer ATX-Agent, fallback to Direct

## 4. Functional Requirements (Release Blocking)

### FR-UA-01 Device Connection and Bootstrap

Acceptance:

1. Connect by explicit serial when provided.
2. Auto-select only when exactly one device is online.
3. Return explicit error when multiple devices exist without serial.
4. Validate/bring up required device-side runtime after connect.

### FR-UA-02 Device Information

Acceptance:

1. Provide resolution, rotation, SDK version, foreground package, and screen state.
2. Return diagnosable errors for failures.

### FR-UA-03 Element Location

Acceptance:

1. Support base conditions (`text`, `resourceId`, `className`, `description`).
2. Support multi-condition composition.
3. Support `instance` and `index`.
4. Support timeout-based existence/non-existence semantics.

### FR-UA-04 Element Operations

Acceptance:

1. Support click, long-click, set/clear/get text.
2. Support `exists`, `wait`, `wait_gone`.
3. Support `info`, `bounds`, `center` access.
4. Return explicit element-not-found/timeout class errors.

### FR-UA-05 Gestures and Key Events

Acceptance:

1. Support click/long-click/swipe/drag/double-click by coordinates.
2. Support percent-to-absolute coordinate conversion.
3. Support common keys (Home/Back/Power/navigation/volume).

### FR-UA-06 Screenshot and File Output

Acceptance:

1. Return image bytes/object and file-save modes.
2. Return explicit errors on capture/save failures.

### FR-UA-07 App Lifecycle

Acceptance:

1. Provide `app_start`, `app_stop`, `app_wait`, `app_current`, `app_clear`.
2. Provide diagnosable timeout/failure errors.

### FR-UA-08 JSON-RPC Reliability

Acceptance:

1. Parse `result`/`error` correctly.
2. Handle timeout/network/service failure modes.
3. Support retry and recovery with clear stop conditions.

### FR-UA-09 Async and Concurrency

Acceptance:

1. Public I/O APIs are async-compatible.
2. Multi-device concurrent usage is safe.
3. Cancellation/timeout does not leak runtime resources.

### FR-CLI-01 `init`

Acceptance:

1. Validate device, install resources, start and verify service.
2. Support `--serial` targeting.
3. Support `--force` reinstall behavior.

### FR-CLI-02 `status`/`restart`/`uninstall`

Acceptance:

1. `status` exposes health/version/port basics.
2. `restart` is stop-then-start with final status.
3. `uninstall` performs stop/uninstall/cleanup flow.

### FR-CLI-03 Embedded Offline Resources

Acceptance:

1. Build embeds required ATX/APK/JAR assets.
2. Runtime init does not depend on online downloads.
3. Missing resources fail early with actionable guidance.

### FR-CLI-04 Cross-platform and Error Readability

Acceptance:

1. Consistent behavior on Windows/Linux/macOS.
2. Complete help output and actionable error messages.
3. Avoid false-failure interpretation in text logs.

### FR-BUG-01 Selector Compatibility

Acceptance:

1. `to_params()` always includes correct mask bits.
2. Include extended bool/regex/index fields.
3. Support `child`/`sibling` hierarchy selectors.
4. Match Python uiautomator2 serialization semantics.

### FR-BUG-02 ATX Compatibility

Acceptance:

1. Correctly decode ADB `shell_v2` output.
2. Accept `/version` plain-text response.
3. Support architecture fallback chain (e.g. `amd64 -> 386`).

## 5. Phase2 Enhancements (Non-blocking by Default)

### PH2-01 Error Type Consolidation

1. End-to-end consistent error types and context.
2. Update docs/examples to reflect final error semantics.

### PH2-02 API Consistency

1. Normalize timeout parameters (`Option<Duration>` semantics).
2. Normalize coordinate argument model and behavior.

### PH2-03 Mock Coverage Systematization

1. Stable mock test infrastructure.
2. Repeatable validation for retry/error paths.

### PH2-04 Optional Caching and Performance

1. Configurable device-info cache TTL.
2. Preserve API semantic compatibility by default.

## 6. Non-functional Requirements

### NFR-01 Compatibility

- OS: Windows/Linux/macOS
- Android: API 21+
- CPU ABI: arm64, x86_64 (with fallback paths where needed)

### NFR-02 Stability

- Retries for transient failures
- Recovery for service-side instability
- Long-running reliability with ATX-Agent preference

### NFR-03 Observability

- Structured full regression outputs: `summary.json`, `summary.junit.xml`
- API coverage outputs: `api-coverage.json`, `api-coverage.md`
- Docs/examples coverage outputs: `docs-coverage-summary.json`, `docs-coverage-summary.md`

### NFR-04 Maintainability

- Public API documentation coverage
- Public API example coverage at release quality
- Test coverage on critical paths
- Scripted and auditable release checks

## 7. Release Acceptance Gates

A release-ready state requires all of the following:

1. All release-blocking requirements are satisfied.
2. Scripted `clean -> rebuild -> full regression` passes.
3. At least emulator + real-device evidence is available.
4. Docs/examples coverage evidence is generated and reviewed.
5. Package checks and publish order are valid (`uiautomator` then `uiautomator-cli`).

Current tracked Phase2 enhancements are complete; future enhancements can be added as new task streams in `TASKS.md`.
