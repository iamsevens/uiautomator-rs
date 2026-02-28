// 集成测试：设备连接和信息获取
// 需�? 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5

mod common;

use uiautomator::Device;

#[tokio::test]
async fn test_device_connect_with_serial() {
    common::init_test_env();
    skip_if_no_device!();

    let serial = common::get_test_device_serial();
    let device = Device::connect(serial.as_deref()).await;

    assert!(device.is_ok(), "应该能够连接到设备");

    if let Ok(device) = device {
        // 验证设备连接成功
        let info = device.info().await;
        assert!(info.is_ok(), "应该能够获取设备信息");
    }
}

#[tokio::test]
async fn test_device_connect_auto_select() {
    common::init_test_env();
    skip_if_no_device!();

    // 不提供序列号，让系统自动选择
    let device = Device::connect(None).await;

    // 如果只有一个设备，应该成功；如果有多个设备，应该失败
    match device {
        Ok(_) => {
            println!("[OK] 自动选择设备成功（只有一个设备）");
        }
        Err(e) => {
            // 如果有多个设备，应该返回明确的错误
            let err_msg = e.to_string();
            if err_msg.contains("多个设备") || err_msg.contains("multiple devices") {
                println!("[OK] 正确检测到多个设备");
            } else {
                panic!("意外的错误: {}", e);
            }
        }
    }
}

#[tokio::test]
async fn test_device_info_display_size() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();
    let info = device.info().await.unwrap();

    // 验证屏幕尺寸
    assert!(info.display_width > 0, "屏幕宽度应该大于 0");
    assert!(info.display_height > 0, "屏幕高度应该大于 0");

    println!(
        "设备屏幕尺寸: {}x{}",
        info.display_width, info.display_height
    );
}

#[tokio::test]
async fn test_device_info_rotation() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();
    let info = device.info().await.unwrap();

    // 验证旋转角度（应该是 0, 90, 180, 270 之一）
    assert!(
        info.display_rotation == 0
            || info.display_rotation == 90
            || info.display_rotation == 180
            || info.display_rotation == 270,
        "旋转角度应该是 0, 90, 180, 270 之一"
    );

    println!("设备旋转角度: {}", info.display_rotation);
}

#[tokio::test]
async fn test_device_info_current_package() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();
    let info = device.info().await.unwrap();

    // 验证当前包名不为空
    assert!(!info.current_package_name.is_empty(), "当前包名不应该为空");

    println!("当前前台应用: {}", info.current_package_name);
}

#[tokio::test]
async fn test_device_info_sdk_version() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();
    let info = device.info().await.unwrap();

    // 验证 SDK 版本（Android 5.0+ 是 21+）
    assert!(info.sdk_int >= 21, "SDK 版本应该至少是 21 (Android 5.0)");

    println!("Android SDK 版本: {}", info.sdk_int);
}

#[tokio::test]
async fn test_device_info_screen_on() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();
    let info = device.info().await.unwrap();

    // 屏幕状态是布尔值
    println!("屏幕是否点亮: {}", info.screen_on);

    // 这个测试不做断言，因为屏幕可能是开或关
}

#[tokio::test]
async fn test_window_size() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();
    let (width, height) = device.window_size().await.unwrap();

    assert!(width > 0, "窗口宽度应该大于 0");
    assert!(height > 0, "窗口高度应该大于 0");

    println!("窗口尺寸: {}x{}", width, height);
}

#[tokio::test]
async fn test_device_info_consistency() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 多次获取设备信息应该一致
    let info1 = device.info().await.unwrap();
    let info2 = device.info().await.unwrap();

    assert_eq!(info1.display_width, info2.display_width);
    assert_eq!(info1.display_height, info2.display_height);
    assert_eq!(info1.sdk_int, info2.sdk_int);

    println!("✓ 设备信息一致性验证通过");
}

#[tokio::test]
async fn test_multiple_device_instances() {
    common::init_test_env();
    skip_if_no_device!();

    let serial = common::get_test_device_serial();

    // 创建多个设备实例
    let device1 = Device::connect(serial.as_deref()).await.unwrap();
    let device2 = Device::connect(serial.as_deref()).await.unwrap();

    // 两个实例都应该能正常工作
    let info1 = device1.info().await.unwrap();
    let info2 = device2.info().await.unwrap();

    assert_eq!(info1.display_width, info2.display_width);

    println!("✓ 多个设备实例可以同时工作");
}
