# uiautomator-cli

[English](./README.md) | [简体中文](./README.zh-CN.md)

CLI tool for initializing and managing ATX-Agent on Android devices.

## Overview

`uiautomator-cli` provides a one-command workflow to set up device-side automation runtime.
It is designed for consistent, scriptable environment bootstrap across Windows/Linux/macOS.

## Core Commands

- `uiautomator init` - install/start/verify ATX-Agent resources
- `uiautomator status` - check service health/version/port
- `uiautomator restart` - restart service
- `uiautomator uninstall` - remove installed components
- `uiautomator version` - show CLI version information

Common options:

- `-s, --serial <SERIAL>` target a specific device
- `-f, --force` force reinstall during `init`

## Quick Usage

Install from crates.io:

```bash
cargo install uiautomator-cli
```

Then initialize a device:

```bash
# initialize a target device
uiautomator init --serial <serial> --force

# check status
uiautomator status --serial <serial>

# restart service
uiautomator restart --serial <serial>

# uninstall
uiautomator uninstall --serial <serial>
```

If exactly one ADB device is online, `--serial` can be omitted.

## Build From Source

```bash
cd uiautomator-cli
cargo build
cargo test --lib
```

Run ignored/integration tests (device required):

```bash
cargo test -- --ignored --nocapture --test-threads=1
```

## Release and Verification

From repository root:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/trigger-gh-release-gate.ps1 -Repo iamsevens/uiautomator-rs -Ref main
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-validation-gate.ps1 -Mode full -Serial <serial>
```

The first command is the unified release gate entrypoint (`Release Check` + `Publish Dry Run`).

## Relationship to `uiautomator`

`uiautomator-cli` depends on the `uiautomator` crate and should be published after `uiautomator`.

## Documentation

- Public testing and release baseline: `../docs/public/TESTING_RELEASE.md`
- Public tasks ledger: `../docs/public/TASKS.md`

## License

MIT.
