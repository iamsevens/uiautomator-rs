# Error Handling Best Practices

This guide documents the recommended way to consume `uiautomator` errors in production code.

## 1. Treat Errors by Category

Use concrete variants for control flow, then fallback to `code()`/`category()` for telemetry.

```rust
use uiautomator::Error;

fn classify(err: &Error) -> (&'static str, u32) {
    (err.category(), err.code())
}
```

## 2. Match Known Recovery Paths

```rust
use std::time::Duration;
use uiautomator::{Device, Error};

async fn wait_app(device: &Device) -> Result<(), Error> {
    match device.app_wait("com.example.app", Some(Duration::from_secs(10))).await {
        Ok(_) => Ok(()),
        Err(Error::AppNotInstalled(pkg)) => {
            eprintln!("app not installed: {pkg}");
            Err(Error::AppNotInstalled(pkg))
        }
        Err(Error::AppCrashed(pkg)) => {
            eprintln!("app crashed: {pkg}");
            Err(Error::AppCrashed(pkg))
        }
        Err(Error::Timeout) => {
            eprintln!("startup timeout");
            Err(Error::Timeout)
        }
        Err(other) => Err(other),
    }
}
```

## 3. Recommended Retry Boundaries

- Retry on: `Error::Timeout`, transient `Error::Http`, transient `Error::JsonRpc`.
- Do not blind-retry on: `Error::AppNotInstalled`, `Error::DeviceNotFound`, `Error::MultipleDevicesFound`.
- Escalate immediately on: repeated `Error::UiAutomatorNotConnected` after environment bootstrap.

## 4. Logging Rules

- Always log `error.code()` and `error.category()`.
- Include device serial/package/selector in surrounding logs.
- Keep raw command outputs truncated when surfacing user-facing messages.

## 5. API Notes

1. Element waits may return `Error::ElementNotFound` or `Error::ElementTimeout`.
2. App lifecycle methods may return `Error::AppNotInstalled`, `Error::AppCrashed`, `Error::AppStartFailed`, `Error::Timeout`.
3. Device connect may return `Error::DeviceNotFound`, `Error::MultipleDevicesFound`, `Error::DeviceOffline`, `Error::UiAutomatorNotConnected`.
