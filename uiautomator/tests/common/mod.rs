// 集成测试通用辅助函数和配�?

#![allow(dead_code)]

use std::env;
use std::sync::Once;
use uiautomator::{Device, Result};

static INIT: Once = Once::new();

/// 初始化测试环境（日志等）
pub fn init_test_env() {
    INIT.call_once(|| {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .ok();
    });
}

/// 获取测试设备序列号（从环境变量或使用默认值）
pub fn get_test_device_serial() -> Option<String> {
    env::var("TEST_DEVICE_SERIAL").ok()
}

/// 解析测试设备序列号：
/// 1) 优先使用 TEST_DEVICE_SERIAL
/// 2) 未设置时自动选择可用设备（多设备优先 emulator-*）
async fn resolve_test_device_serial() -> Option<String> {
    if let Some(serial) = get_test_device_serial() {
        return Some(serial);
    }

    let adb = match uiautomator::AdbClient::new().await {
        Ok(adb) => adb,
        Err(err) => {
            log::warn!("无法连接 ADB，无法解析测试设备: {:?}", err);
            return None;
        }
    };

    let devices = match adb.devices().await {
        Ok(devices) => devices,
        Err(err) => {
            log::warn!("获取设备列表失败: {:?}", err);
            return None;
        }
    };

    match devices.len() {
        0 => None,
        1 => Some(devices[0].clone()),
        _ => {
            if let Some(emulator) = devices
                .iter()
                .find(|serial| serial.starts_with("emulator-"))
            {
                log::warn!(
                    "检测到多个设备且未设置 TEST_DEVICE_SERIAL，自动选择模拟器: {}",
                    emulator
                );
                Some(emulator.clone())
            } else {
                log::warn!(
                    "检测到多个设备且未设置 TEST_DEVICE_SERIAL，自动选择第一个设备: {}",
                    devices[0]
                );
                Some(devices[0].clone())
            }
        }
    }
}

/// 检查是否有可用的测试设备
pub async fn check_device_available() -> bool {
    let serial = resolve_test_device_serial().await;
    Device::connect(serial.as_deref()).await.is_ok()
}

/// 连接到测试设备
pub async fn connect_test_device() -> Result<Device> {
    init_test_env();

    let serial = resolve_test_device_serial().await;
    log::info!("尝试连接到测试设备: {:?}", serial);

    let device = Device::connect(serial.as_deref()).await?;
    prepare_device_for_ui_tests(&device).await?;
    Ok(device)
}

/// 跳过测试如果没有设备可用
#[macro_export]
macro_rules! skip_if_no_device {
    () => {
        if !common::check_device_available().await {
            eprintln!("⚠️  跳过测试：没有可用的 Android 设备");
            eprintln!("提示：连接 Android 设备或启动模拟器后重新运行测试");
            eprintln!("可以通过环境变量 TEST_DEVICE_SERIAL 指定设备序列号");
            return;
        }
    };
}

/// 测试应用包名（仓库内 test-app）
pub const TEST_APP_PACKAGE: &str = "com.uiautomator.testapp";
pub const TEST_APP_ACTIVITY: &str = "com.uiautomator.testapp.MainActivity";

/// 等待一小段时间（用于 UI 稳定）
pub async fn wait_ui_stable() {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
}

/// 清理测试环境（返回主屏幕）
pub async fn cleanup_test_env(device: &Device) -> Result<()> {
    use uiautomator::Key;

    // 按 Home 键返回主屏幕
    device.press(Key::Home).await?;
    wait_ui_stable().await;

    Ok(())
}

/// 统一测试前设备预处理：确保屏幕点亮、解锁并回到主页。
async fn prepare_device_for_ui_tests(device: &Device) -> Result<()> {
    let serial = device.serial();
    let adb = device.adb_client();

    // Best-effort: 某些模拟器/ROM 可能不支持其中个别命令。
    let _ = adb
        .shell(serial, "input keyevent KEYCODE_WAKEUP", None)
        .await;
    let _ = adb.shell(serial, "wm dismiss-keyguard", None).await;
    let _ = adb.shell(serial, "input keyevent 82", None).await; // MENU

    // 回到主页，避免继承上个测试页面状态。
    device.press(uiautomator::Key::Home).await?;
    wait_ui_stable().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_test_env() {
        init_test_env();
        // 应该不会 panic
    }

    #[tokio::test]
    async fn test_get_test_device_serial() {
        let serial = get_test_device_serial();
        // 可能是 Some 或 None，都是有效的
        println!("测试设备序列号: {:?}", serial);
    }
}
