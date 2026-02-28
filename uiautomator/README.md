# uiautomator

[English](./README.md) | [简体中文](./README.zh-CN.md)

Rust async Android UI automation library with a Python `uiautomator2`-style API.

## What This Crate Provides

- Device connection and mode routing (`Auto`, `AtxAgent`, `Direct`)
- Selector-based element lookup
- UiObject operations (click/input/clear/wait/info/text/bounds)
- Gestures, key events, screenshots, app lifecycle operations
- Structured errors with retry support for unstable device/RPC scenarios

## Installation

```toml
[dependencies]
uiautomator = "0.1"
tokio = { version = "1", features = ["full"] }
```

Optional ATX-Agent installer support:

```toml
[dependencies]
uiautomator = { version = "0.1", features = ["atx-agent-install"] }
```

Note:
- The `uiautomator` crate package does not embed multi-arch `atx-agent` binaries to keep publish size within crates.io limits.
- Recommended setup path is `uiautomator-cli init` (or equivalent external provisioning) before using `AtxAgent` mode.

## Minimal Example

```rust
use uiautomator::{Device, Selector};

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    let d = Device::connect(None).await?;

    let settings = d.find(Selector::new().text("Settings"));
    if settings.exists(None).await? {
        settings.click(None, None).await?;
    }

    Ok(())
}
```

## Modes

- `Auto` (default): try ATX-Agent first, fallback to Direct.
- `AtxAgent`: use ATX-Agent transport (recommended for long-running/stable automation).
- `Direct`: direct JSON-RPC transport (fast setup, lower robustness).

## Common APIs

- `Device`: `connect`, `info`, `find`, `click`, `swipe`, `press`, `screenshot`, `app_start`, `app_stop`
- `UiObject`: `exists`, `wait`, `wait_gone`, `click`, `long_click`, `set_text`, `clear_text`, `get_text`, `info`
- `Selector`: text/resource-id/class/description plus regex and hierarchy (`child`/`sibling`)

## Testing

```bash
cargo test
cargo test -- --ignored --nocapture --test-threads=1
```

For full device regression, use the repository script from root:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/device-full-test.ps1 -Serial <serial>
```

## Documentation

- Public requirements/design/tasks/release docs:
  - `../docs/public/REQUIREMENTS.md`
  - `../docs/public/DESIGN.md`
  - `../docs/public/TASKS.md`
  - `../docs/public/TESTING_RELEASE.md`

Note: `tests/**` and `examples/**` stay in GitHub and are not included in the crates package by default.

## License

MIT.
