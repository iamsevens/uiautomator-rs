# Public Docs

[English](./README.md) | [简体中文](./README.zh-CN.md)

Public-facing documentation baseline derived from internal specifications.

## Scope

These documents consolidate internal spec streams into a publishable set:

- `uiautomator` (core + ATX extensions)
- `uiautomator-cli`
- `uiautomator-phase2` (completed enhancement stream)
- `bugfix` (selector compatibility and ATX reliability fixes)

## Documents

- `REQUIREMENTS.md`: release baseline requirements with IDs and acceptance criteria
- `DESIGN.md`: architecture, module design, key decisions, and risk handling
- `QUALITY_BASELINE.md`: stable quality conclusions from the validated review pass, including hardened behavior, explicit boundaries, and verification expectations
- `API_DOCS.md`: public API guide, crate relationships, conventions, and example entry points
- `MIGRATION.md`: migration status and future breaking-change guidance
- `TASKS.md`: completion ledger, open items, release blocking status, priorities
- `TESTING_RELEASE.md`: test matrix, failure taxonomy, release gates, evidence templates
- `RELEASE_NOTES.md`: curated release notes with evidence-backed CI/device regression outcomes

Chinese mirrors are available as `*.zh-CN.md` in the same directory.

## How to Use

1. Start with `REQUIREMENTS.md` for scope and acceptance.
2. Read `DESIGN.md` for implementation and operational decisions.
3. Use `QUALITY_BASELINE.md` for stable review-derived conclusions and closed issue boundaries.
4. Use `API_DOCS.md` for public API structure and docs.rs entry points.
5. Check `TASKS.md` for current progress and next priorities.
6. Follow `TESTING_RELEASE.md` for release-grade verification.

## Status Labels

- `Done`: implemented with verifiable evidence
- `Release Engineering`: non-API engineering hardening tasks
- `Phase2`: enhancement track, typically non-blocking for current release
