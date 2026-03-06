# Testing and Release Baseline

[English](./TESTING_RELEASE.md) | [简体中文](./TESTING_RELEASE.zh-CN.md)

## 1. Purpose

Defines release-grade validation and evidence standards.

Goals:

1. Reproduce `clean environment -> rebuilt runtime -> full regression` deterministically.
2. Distinguish real product failures from environment noise.
3. Produce auditable release artifacts.

## 2. Sources and Applicability

Sources:

- `.kiro/specs/uiautomator*/tasks.md`
- `.kiro/specs/bugfix/tasks.md`
- `scripts/device-full-test.ps1`
- `scripts/run-validation-gate.ps1`
- `scripts/api-coverage-report.ps1`
- `scripts/docs-coverage-report.ps1`
- `scripts/unit-coverage-report.ps1`
- `scripts/release-check.ps1`, `scripts/release-check.sh`
- `scripts/trigger-gh-device-regression.ps1`

Applies to:

- `uiautomator` release verification
- `uiautomator-cli` release verification

## 3. Test Layering

- `unit`: pure logic/model/serialization/error mapping
- `integration`: real device/emulator end-to-end
- `ignored`: environment-heavy or slow suites

### `non-ignored` vs `ignored`

- `non-ignored`: default daily signal path
- `ignored`: additional deep verification path

Release validation should include both.

## 4. Full Regression Script Contract

### Standard Command

```powershell
C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/run-validation-gate.ps1 -Mode full -Serial <serial>
```

### Fixed Steps

1. Run gate preflight (`PowerShell`, `adb`, Rust toolchain, device readiness).
2. Clean ATX/uiautomator runtime state and stale processes.
3. Rebuild environment via `init -f`.
4. Install/verify `test-app` (`com.uiautomator.testapp`).
5. Run four suites:
- `uiautomator-cli` non-ignored
- `uiautomator-cli` ignored
- `uiautomator` non-ignored
- `uiautomator` ignored

### Hard Constraints

- Serial pinning is mandatory.
- Hard timeout is mandatory.
- Structured summary output is mandatory.

## 5. Regression Matrix Template

### Minimum Matrix (Recommended)

- 1 real `arm64` device
- 1 `x86_64` emulator

### Expanded Matrix (Recommended)

- At least 2 Android major versions
- Multiple emulator vendors/types
- Both initial states:
- clean environment rebuild
- preinstalled ATX runtime

### Record Template

| RunID | Device | Type | Arch | Android | Initial State | CLI non-ignored | CLI ignored | Lib non-ignored | Lib ignored | Final |
|---|---|---|---|---|---|---|---|---|---|---|
| `<run-id>` | `<serial>` | `real/emulator` | `arm64/x86_64` | `<ver>` | `clean/preinstalled` | pass/fail | pass/fail | pass/fail | pass/fail | pass/fail |

## 6. Artifact and Log Standards

### Output Paths

- Gate orchestrator: `internal/testlogs/validation-gate/<run-id>/`
- Full regression: `internal/testlogs/full-device/<run-id>/`
- API coverage: `internal/testlogs/api-coverage/<run-id>/`
- Docs coverage: `internal/testlogs/docs/<run-id>/`
- Unit coverage baseline: `internal/testlogs/unit-coverage/<run-id>/`

### Required Artifacts

- Human logs: `*.log`, `*.err.log`
- Machine-readable: `summary.json`, `summary.junit.xml`, `api-coverage.json`, `api-coverage.md`, `docs-coverage-summary.json`, `docs-coverage-summary.md`
- Internal baseline: `unit-coverage-summary.json`, `unit-coverage-summary.md`

### Encoding Rule

- UTF-8 end-to-end for console/process/log parsing.
- Encoding anomalies are treated as environment faults and require rerun.

## 7. Failure Taxonomy (Anti-Misread)

### Codes

- `E-ENV`: environment issues (offline device, adb failure, permission popup, port conflict)
- `E-ATX`: ATX runtime issues (install/start/probe/version mismatch)
- `E-APP`: test-app mismatch issues
- `E-TEST`: test-case design/assertion/timeout issues
- `E-LIB`: core library regressions
- `E-CLI`: CLI flow/output regressions

### Gate Failure Codes (Machine Readable)

- `env_missing_powershell`
- `env_missing_adb`
- `env_missing_rust_toolchain`
- `adb_device_unavailable`
- `test_app_build_failed`
- `step_timeout`
- `manifest_or_summary_invalid`
- `test_or_runtime_failure`

### Handling Rules

1. `E-ENV` / `E-ATX`: fix environment first, then rerun.
2. `E-APP` / `E-TEST`: fix test assets/cases and compare before/after.
3. `E-LIB` / `E-CLI`: bugfix required; full rerun after fix.

### Failure Record Template

| Time | RunID | Device | Step | Code | Symptom | Root Cause | Action | Rerun |
|---|---|---|---|---|---|---|---|---|
| `<time>` | `<run-id>` | `<serial>` | `<step>` | `E-*` | `<symptom>` | `<cause>` | `<action>` | pass/fail |

## 8. Coverage Accounting

### API Coverage Command

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/api-coverage-report.ps1
```

### API Coverage Expectations

- Public API to test-case mapping generated
- Uncovered APIs flagged explicitly
- Coverage artifacts included in release evidence

### Docs And Examples Coverage Command

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/docs-coverage-report.ps1 -FailOnThreshold -MinDocsPercent 100 -MinExamplesPercent 100
```

### Docs And Examples Expectations

- Public API docs coverage remains at `100%`
- Public API examples coverage remains at `100%`
- Latest docs coverage summaries are generated under `internal/testlogs/docs/`
- CI blocks regressions below the configured threshold

### Unit Coverage Baseline Command

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/unit-coverage-report.ps1 -Crates uiautomator
```

### Unit Coverage Baseline Expectations

- Produce a reproducible unit-coverage baseline for the core crate.
- Use it to identify weak modules before adding more tests.
- Do not treat a single global percentage as the only release-quality signal for device-bound code.

## 9. Release Gates

### Hard Gates

1. `release-check` passes.
2. Clean rebuild + full regression passes on real device + emulator.
3. `summary.json` and `summary.junit.xml` are present and valid.
4. API coverage artifacts are generated and reviewed.
5. Docs/examples coverage artifacts are generated and reviewed.
6. `cargo publish --dry-run` passes for both crates.

### Soft Gates

1. Keep Task 23 matrix expansion maintained (baseline completed).
2. Keep Task 24 post-install smoke path maintained (baseline completed).
3. Keep Task 25 nightly guardrail healthy (baseline completed).
4. Re-run optional perf benchmark when cache-related code changes: `cargo bench --bench device_info_cache -- --sample-size 10`.

## 10. Release Execution Order

### Commands (Windows)

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/release-check.ps1
C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/run-validation-gate.ps1 -Mode full -Serial <serial>
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/api-coverage-report.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/docs-coverage-report.ps1 -FailOnThreshold -MinDocsPercent 100 -MinExamplesPercent 100
# optional perf evidence when cache code changes
cargo bench --bench device_info_cache -- --sample-size 10
cargo publish --dry-run
```

### GitHub Actions (Manual Sequential Trigger)

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/trigger-gh-device-regression.ps1 -Serial <serial> -TargetName <name> -ExpectedAbi <abi> -ExpectedAndroidMajor <major>
```

Behavior:

1. Dispatch `Install Smoke` and wait for `success`.
2. Dispatch `Device Regression Matrix` and wait for `success`.
3. Fail fast on timeout or non-success conclusion with direct run URL.

### Commands (Linux/macOS)

```bash
bash scripts/release-check.sh
# run pwsh device/coverage scripts or equivalent wrappers
```

### crates.io Order

1. Publish `uiautomator`
2. Wait for index visibility
3. Publish `uiautomator-cli`

## 11. Release Evidence Template

```markdown
## Release Evidence - <version>

- Date: <YYYY-MM-DD>
- Reviewer: <name>
- Commit: <sha>

### Device Matrix
- <run-id-1>: <serial> / <arch> / pass
- <run-id-2>: <serial> / <arch> / pass

### Artifacts
- summary.json: <path>
- summary.junit.xml: <path>
- api-coverage.json: <path>
- api-coverage.md: <path>
- docs-coverage-summary.json: <path>
- docs-coverage-summary.md: <path>

### Gate Check
- release-check: pass/fail
- full regression: pass/fail
- coverage review: pass/fail
- docs/examples coverage review: pass/fail
- cargo publish --dry-run (uiautomator): pass/fail
- cargo publish --dry-run (uiautomator-cli): pass/fail

### Exceptions
- <none / details>

### Final Decision
- Release: yes/no
- Notes: <text>
```

## 12. Post-release Guardrails

- Schedule nightly full regression.
- Keep structured artifacts as CI artifacts.
- Route failures using the `E-*` taxonomy.

Related tasks: see `TASKS.md` (Task 23/24/25 completed; current open items are Phase2 follow-ups).
