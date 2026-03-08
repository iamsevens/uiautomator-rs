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

Publishing and release gates are defined in repository root `PUBLISHING.md`.
Release-grade validation and evidence live in `../docs/public/TESTING_RELEASE.md`.

## Relationship to `uiautomator`

`uiautomator-cli` depends on the `uiautomator` crate and should be published after `uiautomator`.

## Documentation

- Public testing and release baseline: `../docs/public/TESTING_RELEASE.md`
- Public tasks ledger: `../docs/public/TASKS.md`

## License

MIT.
