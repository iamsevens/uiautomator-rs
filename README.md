# uiautomator-rs

[![crates.io - uiautomator](https://img.shields.io/crates/v/uiautomator.svg)](https://crates.io/crates/uiautomator)
[![docs.rs - uiautomator](https://docs.rs/uiautomator/badge.svg)](https://docs.rs/uiautomator)
[![crates.io - uiautomator-cli](https://img.shields.io/crates/v/uiautomator-cli.svg)](https://crates.io/crates/uiautomator-cli)
[![docs.rs - uiautomator-cli](https://docs.rs/uiautomator-cli/badge.svg)](https://docs.rs/uiautomator-cli)

[English](./README.md) | [简体中文](./README.zh-CN.md)

Rust implementation of Android UI automation with a Python `uiautomator2`-style API.

## Overview

`uiautomator-rs` contains two publishable Rust crates and one in-repo test fixture app:

- `uiautomator/`: core async library (`Device`, `Selector`, `UiObject`, JSON-RPC/ATX transport)
- `uiautomator-cli/`: CLI for ATX-Agent setup and lifecycle management
- `test-app/`: Android APK fixture for integration and regression testing (repo-only, not published)

## Current Status

- Core library APIs are implemented and validated on emulator + real devices.
- ATX-Agent compatibility fixes are in place (shell_v2 decoding, `/version` text response compatibility).
- Selector compatibility fixes are in place (mask bits, extended fields, child/sibling serialization).
- Full device regression is script-based with machine-readable summaries.

## Key Capabilities

- Device connect and mode routing (`Auto`, `AtxAgent`, `Direct`)
- Element lookup with rich selectors
- UiObject operations (`click`, `long_click`, `set_text`, `wait`, `wait_gone`, etc.)
- Gestures, keys, screenshots, and app lifecycle operations
- Built-in retry and structured error mapping
- Multi-device async usage

## Repository Layout

```text
.
├── uiautomator/           # core crate
├── uiautomator-cli/       # CLI crate
├── test-app/              # Android test fixture APK project
├── scripts/               # regression/release scripts
├── docs/public/           # public spec/design/task/release docs
└── internal/              # local artifacts and internal records
```

Note: this repository is not a Cargo workspace; run Cargo commands inside each crate directory.

## Quick Start

### 1) Prerequisites

- Android device or emulator with ADB connectivity
- Rust toolchain
- `adb` available in `PATH`

### 2) Build

```bash
cd uiautomator
cargo build

cd ../uiautomator-cli
cargo build
```

### 3) Initialize device-side environment (CLI)

```bash
cd uiautomator-cli
cargo run -- init --serial <serial> --force
cargo run -- status --serial <serial>
```

### 4) Basic library usage

```rust
use uiautomator::{Device, Selector};

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    let d = Device::connect(None).await?;
    d.find(Selector::new().text("Settings")).click(None, None).await?;
    Ok(())
}
```

## Testing

### Per-crate tests

```bash
cd uiautomator-cli
cargo test
cargo test -- --ignored --nocapture --test-threads=1

cd ../uiautomator
cargo test
cargo test -- --ignored --nocapture --test-threads=1
```

### Full device regression (recommended)

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/device-full-test.ps1 -Serial <serial>
```

Outputs include logs plus machine-readable artifacts (`summary.json`, `summary.junit.xml`).

### GitHub Actions regression (sequential)

```cmd
scripts\run-gh-device-regression.cmd -Serial <serial> -TargetName <name> -ExpectedAbi <abi> -ExpectedAndroidMajor <major>
```

If exactly one ADB device is online, `-Serial` can be omitted and the script auto-selects it.

The script dispatches `Install Smoke` first, waits for completion, then dispatches `Device Regression Matrix`.  
It uses `gh api workflow_dispatch` payload files (UTF-8, no BOM) to avoid `targets_json` quote-loss in PowerShell.

## Public Documentation

- [Public Docs Index](docs/public/README.md)
- [Requirements Baseline](docs/public/REQUIREMENTS.md)
- [Design Baseline](docs/public/DESIGN.md)
- [Tasks Ledger](docs/public/TASKS.md)
- [Testing and Release Baseline](docs/public/TESTING_RELEASE.md)

Chinese mirrors are available as `*.zh-CN.md` files in the same directory.

## Crates Relationship

- `uiautomator` is the base library crate.
- `uiautomator-cli` depends on `uiautomator`.
- Release order on crates.io: `uiautomator` first, then `uiautomator-cli`.

## Release

Use the unified release gate script before any crates.io publish attempt:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\trigger-gh-release-gate.ps1 -Repo iamsevens/uiautomator-rs -Ref main
```

It runs `Release Check` then `Publish Dry Run` in sequence and fails fast on the first non-success run.  
See [PUBLISHING.md](./PUBLISHING.md) for the full fixed release process.

## Support and Policies

- [Contributing](./CONTRIBUTING.md)
- [Security Policy](./SECURITY.md)
- [Support](./SUPPORT.md)
- [Code of Conduct](./CODE_OF_CONDUCT.md)

## License

MIT. See [LICENSE](./LICENSE).

