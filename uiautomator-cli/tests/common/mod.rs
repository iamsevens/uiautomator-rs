#![allow(dead_code)]

use anyhow::Result;
use uiautomator_cli::{commands, installer::Installer};

pub fn resolve_test_serial() -> Option<String> {
    for key in ["TEST_DEVICE_SERIAL", "ANDROID_SERIAL"] {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

pub async fn new_installer() -> Result<Installer> {
    Installer::new(resolve_test_serial()).await
}

pub async fn execute_init(force: bool) -> Result<()> {
    commands::execute_init(resolve_test_serial(), force).await
}

pub async fn execute_status() -> Result<()> {
    commands::execute_status(resolve_test_serial()).await
}

pub async fn execute_restart() -> Result<()> {
    commands::execute_restart(resolve_test_serial()).await
}

pub async fn execute_uninstall() -> Result<()> {
    commands::execute_uninstall(resolve_test_serial()).await
}
