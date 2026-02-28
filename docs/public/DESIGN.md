# Design Baseline

[English](./DESIGN.md) | [简体中文](./DESIGN.zh-CN.md)

## 1. Purpose

Public architecture and design baseline for `uiautomator-rs`, aligned with requirement IDs and release verification.

## 2. Scope and Boundaries

### In Scope

- Core library architecture (`uiautomator`)
- CLI architecture (`uiautomator-cli`)
- Selector compatibility and ATX reliability fixes
- Test/release observability design

### Out of Scope

- Business-level UI workflows (pagination/navigation heuristics)
- Device farm scheduling/orchestration platform
- Cloud execution control plane

## 3. Requirement-to-Design Traceability

| Requirement ID | Design Focus | Modules | Verification |
|---|---|---|---|
| FR-UA-01 | connect mode routing + idempotent bootstrap | `device.rs`, `adb.rs` | integration + full-device script |
| FR-UA-03 | selector DSL + mask serialization | `selector.rs`, `uiobject.rs` | unit + integration |
| FR-UA-04 | UiObject operation semantics | `uiobject.rs` | mock + integration |
| FR-UA-08 | RPC retry/recovery/error mapping | `jsonrpc.rs`, `error.rs` | failure-path tests |
| FR-CLI-01 | init flow state machine | `installer.rs`, `commands.rs` | ignored integration |
| FR-CLI-03 | embedded resources and build checks | `resources.rs`, `build.rs` | package/release checks |
| FR-BUG-01 | selector compatibility layer | `selector.rs` | bugfix regression |
| FR-BUG-02 | ATX compatibility layer | `atx_agent.rs`, `device.rs` | multi-device regression |

## 4. System Architecture

Logical path:

`User Code / CLI -> Device -> (JsonRpcClient | AtxAgentClient) -> AdbClient -> Android Device`

Design principles:

- Keep behavior close to Python `uiautomator2` expectations.
- Keep public I/O APIs async-first.
- Treat transient instability as normal: retry, fallback, recover.
- Prefer diagnosable errors over broad string guessing.

## 5. Mode Strategy

### Direct

- JSON-RPC directly to device-side service.
- Faster startup, lower long-run resilience.

### ATX-Agent

- REST + JSON-RPC forwarding path through ATX-Agent.
- Better long-run stability and service lifecycle handling.

### Auto

- ATX-Agent preferred, Direct fallback.
- Default for practical cross-environment reliability.

## 6. Module Design

### `device.rs`

- Unified API entry point.
- Mode routing and high-level operation orchestration.
- Aggregates element, gesture, key, screenshot, and app APIs.

### `jsonrpc.rs`

- Request/response construction and parsing.
- Retry policy integration with bounded attempts and delay strategy.
- Structured error translation.

### `atx_agent.rs`

- REST endpoints (`/version`, `/info`, `/jsonrpc/0`) abstraction.
- Install/start/restart/uninstall coordination.
- Compatibility handling for text responses and shell output formats.

### `adb.rs`

- Device listing, shell, push/pull, forwarding.
- Exit code + output-aware command failure reporting.
- Time-bounded execution to avoid hanging flows.

### `selector.rs`

- Builder DSL for query composition.
- Mask and field serialization parity with Python behavior.
- Hierarchy selectors (`child`/`sibling`).

### `uiobject.rs`

- Poll-based wait semantics (`exists`, `wait`, `wait_gone`).
- Element operations and metadata access.
- Operation result semantics validation (avoid silent success).

### `error.rs`

- Centralized error domains, codes, and context fields.
- Distinguishes device/protocol/semantic failures.

## 7. Key Flows

### CLI Init Flow

1. Validate target device availability.
2. Check existing ATX state.
3. Apply force/no-force branch.
4. Push/install required runtime assets.
5. Start service and verify health.
6. Return structured status.

### UiObject Wait Flow

1. Resolve effective timeout (operation override vs settings default).
2. Poll element state.
3. Return on first success.
4. Timeout returns explicit timeout-class error.

### Auto Fallback Flow

1. Probe ATX availability.
2. Use ATX when healthy.
3. Fallback to Direct when ATX probe fails.
4. Return aggregate diagnosable error when both fail.

## 8. Data Model Constraints

Core models include `DeviceInfo`, `ElementInfo`, `Rect`, `AppInfo`, `Settings`.

Constraints:

- `serde` mapping must remain protocol-compatible.
- Field semantics must match device-side payload semantics.
- Convenience methods must not alter source semantics.

## 9. Correctness Properties

1. Connection idempotence
2. Selector result consistency
3. Coordinate conversion correctness
4. Timeout boundary correctness
5. Request/response pairing correctness
6. Recovery operation idempotence
7. Concurrency safety
8. Resource cleanup completeness

## 10. Error and Observability Design

### Error Layers

- Device layer: offline/not found/permission/timeout
- Protocol layer: RPC parse/service response issues
- Semantic layer: element/app-state mismatch

### Logging

- Human-readable step logs
- Machine-readable summaries (`summary.json`, `summary.junit.xml`)
- Failure records include command, exit code, artifact paths

### Misread Prevention

- Avoid broad `contains("error")` fail logic.
- UTF-8 normalization across process output and log parsing.
- Explicit pass/fail statuses in summary output.

## 11. Testing Architecture

- Unit tests: logic/serialization/error mapping
- Integration tests: real device end-to-end
- Ignored tests: environment-heavy or long-running paths

Stability rules:

- Pin explicit device serial.
- Enforce hard timeout for long flows.
- Emit replayable logs and structured summaries.

API coverage accounting:

- `scripts/api-coverage-report.ps1` maps public API to tests.

## 12. Build and Release Design

- `build.rs` injects asset checksum metadata during compile.
- Missing required assets fail early.
- crates.io release order: `uiautomator` then `uiautomator-cli`.

## 13. Phase2 Design Scope

- Error type consolidation
- API argument consistency
- Systematic mock coverage
- Optional cache/perf improvements

These are tracked as enhancement milestones and do not automatically block current release.

## 14. Risks and Mitigations

1. Device/emulator variance
- Mitigation: release matrix includes real device + emulator.

2. Environment-generated false failures
- Mitigation: structured summary + failure taxonomy.

3. ABI/asset mismatch in ATX runtime
- Mitigation: ordered fallback chain and health verification.

4. Upstream response-format drift
- Mitigation: tolerant parsing and targeted regression tests.
