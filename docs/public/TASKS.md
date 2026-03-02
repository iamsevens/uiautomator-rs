# Tasks Ledger

[English](./TASKS.md) | [简体中文](./TASKS.zh-CN.md)

## 1. Purpose

Public task ledger for release accountability.
It tracks what is complete, what remains open, and what actually blocks release.

## 2. Sources

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
| `uiautomator` Phase1 + ATX | Done (1-19, 18A/18B) | Satisfied |
| `uiautomator-cli` base + engineering | 1-25 done | Satisfied |
| `bugfix` selector fixes | Done (1-3) | Satisfied |
| `phase2` enhancements | partial (1/2/3/4 + 5.1 done, others open) | Non-blocking by default |

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

## 6. Open Ledger

### 6.1 Phase2 Enhancements

From `uiautomator-phase2/tasks.md`:

1. Task 1 (error type refinement)
- 1.1 Done
- 1.2 Done
- 1.3 Done

2. Task 5 (API consistency)
- 5.1 Done
- 5.2 Open (optional)
- 5.3 Open

3. Task 6 (mock coverage expansion)
- 6.1 Open
- 6.2 Open
- 6.3 Open

4. Task 7 (device info cache/performance, optional)
- 7.1-7.4 Open

## 7. Release Blocking Decision

### Current Blocking Items

No explicit open `Release-Blocking` item remains.

### Baseline Checks

1. Core FR requirements are implemented.
2. Full regression script path exists and is runnable.
3. Structured test artifacts are available.
4. Publish order and package checks are defined.

Conclusion: release baseline is met; open items are sustainability hardening.

## 8. Recommended Next Order

### P0

1. Phase2 Task 5.3 (API docs consistency review)

### P1

1. Phase2 Task 6 (mock systematization)

### P2

1. Phase2 Task 5.2 (optional coordinate model unification)
2. Phase2 Task 7 (optional cache/perf)

## 9. Cross-Doc Consistency Rule

When status changes, update all three baselines together:

- `REQUIREMENTS.md`
- `DESIGN.md`
- `TESTING_RELEASE.md`

This avoids drift between implementation status and public documentation.
