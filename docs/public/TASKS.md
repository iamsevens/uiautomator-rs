# Tasks Ledger

[English](./TASKS.md) | [简体中文](./TASKS.zh-CN.md)

## 1. Purpose

Public task ledger for release accountability.
It tracks what is complete, what remains open, and what actually blocks release.

## 2. Sources

- `.kiro/specs/docs-examples-quality/tasks.md`
- `.kiro/specs/uiautomator/tasks.md`
- `.kiro/specs/uiautomator-cli/tasks.md`
- `.kiro/specs/uiautomator-phase2/tasks.md`
- `.kiro/specs/bugfix/tasks.md`

## 3. Status Model

- `Done`: implemented and verified
- `Open`: not completed yet
- `Release-Blocking`: must be done before release
- `Post-Release`: strongly recommended but not blocking current release

## 4. Consolidated Status

| Domain | Current State | Release Impact |
|---|---|---|
| docs/examples quality | Done (coverage tooling + CI guard + 100% docs/examples) | Satisfied |
| `uiautomator` Phase1 + ATX | Done (1-19, 18A/18B) | Satisfied |
| `uiautomator-cli` base + engineering | 1-25 done | Satisfied |
| `bugfix` selector fixes | Done (1-3) | Satisfied |
| `phase2` enhancements | Done (1/2/3/4 + 5.1 + 5.2 + 5.3 + 6.1 + 6.2 + 6.3 + 7.1-7.4 complete) | Satisfied |

## 5. Done Ledger (Evidence Eligible)

### 5.1 Core Library (`uiautomator`)

Done groups:

- 1-4: project foundation, ADB, JSON-RPC, data model
- 5-14: selector, UiObject, Device APIs, exports/docs
- 15-17: unit/integration checkpoints
- 18A: ATX connection mode
- 18B: ATX install mode
- 19: docs and release prep

### 5.2 CLI (`uiautomator-cli`)

Done groups:

- 1-20: command framework, installer, tests, build/release
- 21: API coverage accounting automation
- 22: test log normalization (UTF-8 + JSON/JUnit summary)
- 23: multi-arch/multi-version regression matrix
- 24: post-install smoke (`cargo install` path)
- 25: nightly regression guardrail

Evidence highlights:

- Device regression matrix validated on both emulator targets with success:
  - Run `22583901973`
  - `mumu-16384` (`127.0.0.1:16384`)
  - `ld-emulator-5554` (`emulator-5554`)

### 5.3 Bugfix Stream

Done groups:

- 1: selector mask correction (P0)
- 2: extended selector fields (P1)
- 3: child/sibling hierarchy selector support (P2)

### 5.4 Docs and Examples Quality

Done groups:

- 1-5: coverage tooling, parser, missing docs/examples closure, CI guard, release checklist integration

Evidence highlights:

- Coverage entrypoint: `scripts/docs-coverage-report.ps1`
- Latest summary artifacts:
  - `internal/testlogs/docs/latest-summary.json`
  - `internal/testlogs/docs/latest-summary.md`
- Verified aggregate coverage:
  - docs `331/331` (`100%`)
  - examples `187/187` (`100%`)
- CI guard workflow:
  - `.github/workflows/docs-coverage.yml`

## 6. Open Ledger

### 6.1 Phase2 Enhancements

From `uiautomator-phase2/tasks.md`:

1. Task 1 (error type refinement)
- 1.1 Done
- 1.2 Done
- 1.3 Done

2. Task 5 (API consistency, done)
- 5.1 Done
- 5.2 Done (public `Coord` model + `*_coord` helpers)
- 5.3 Done

3. Task 6 (mock coverage expansion, done)
- 6.1 Done (mockito + `mock_uiobject_test` + CI guard)
- 6.2 Done (device info, element info, click, set_text, exists, element-not-found)
- 6.3 Done (network, timeout, invalid response, server error, retry)

4. Task 7 (device info cache/performance, optional)
- 7.1 Done (cache entry + shared cache state)
- 7.2 Done (cache hit/expiry behavior + mock regression)
- 7.3 Done (`set_cache_ttl`, `clear_cache`, `disable_cache`)
- 7.4 Done (`criterion` benchmark: `benches/device_info_cache.rs`)

No explicit open item remains in the current public task ledger.

## 7. Release Blocking Decision

### Current Blocking Items

No explicit open `Release-Blocking` item remains.

### Baseline Checks

1. Core FR requirements are implemented.
2. Full regression script path exists and is runnable.
3. Structured test artifacts are available.
4. API coverage artifacts are available.
5. Docs/examples coverage artifacts are available.
6. Publish order and package checks are defined.

Conclusion: release baseline is met; no tracked open item remains in the current ledger.

## 8. Recommended Next Order

### P0

No P0 item remains.

### P1

No P1 item remains.

### P2

No P2 item remains.

## 9. Cross-Doc Consistency Rule

When status changes, update all three baselines together:

- `REQUIREMENTS.md`
- `DESIGN.md`
- `TESTING_RELEASE.md`

This avoids drift between implementation status and public documentation.
