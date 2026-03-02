// 集成测试：截图和应用管理
// 需求: 7.1, 7.2, 7.3, 7.4, 7.5, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6

mod common;

use std::path::Path;
use std::time::Duration;

#[tokio::test]
async fn test_screenshot() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 截图
    let result = device.screenshot().await;
    assert!(result.is_ok(), "截图应该成功");

    if let Ok(image) = result {
        println!("✓ 截图成功: {}x{}", image.width(), image.height());
        assert!(image.width() > 0, "图像宽度应该大于 0");
        assert!(image.height() > 0, "图像高度应该大于 0");
    }
}

#[tokio::test]
async fn test_screenshot_to_file_png() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 保存截图为 PNG
    let path = "test_screenshot.png";
    let result = device.screenshot_to_file(path).await;
    assert!(result.is_ok(), "保存截图应该成功");

    // 验证文件存在
    assert!(Path::new(path).exists(), "截图文件应该存在");

    println!("✓ 截图保存为 PNG 成功");

    // 清理
    std::fs::remove_file(path).ok();
}

#[tokio::test]
async fn test_screenshot_to_file_jpg() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 保存截图为 JPEG
    let path = "test_screenshot.jpg";
    let result = device.screenshot_to_file(path).await;
    assert!(result.is_ok(), "保存截图应该成功");

    // 验证文件存在
    assert!(Path::new(path).exists(), "截图文件应该存在");

    println!("✓ 截图保存为 JPEG 成功");

    // 清理
    std::fs::remove_file(path).ok();
}

#[tokio::test]
async fn test_app_start_package_only() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 只使用包名启动应用
    let result = device.app_start(common::TEST_APP_PACKAGE, None).await;
    assert!(result.is_ok(), "启动应用应该成功");

    println!("✓ 通过包名启动应用成功");

    common::wait_ui_stable().await;
    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_app_start_with_activity() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 使用包名和 Activity 启动应用
    let result = device
        .app_start(common::TEST_APP_PACKAGE, Some(common::TEST_APP_ACTIVITY))
        .await;
    assert!(result.is_ok(), "启动应用应该成功");

    println!("✓ 通过包名和 Activity 启动应用成功");

    common::wait_ui_stable().await;
    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_app_stop() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 先启动应用
    device.app_start(common::TEST_APP_PACKAGE, None).await.ok();
    common::wait_ui_stable().await;

    // 停止应用
    let result = device.app_stop(common::TEST_APP_PACKAGE).await;
    assert!(result.is_ok(), "停止应用应该成功");

    println!("✓ 停止应用成功");

    common::wait_ui_stable().await;
}

#[tokio::test]
async fn test_app_current() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 启动设置应用
    device.app_start(common::TEST_APP_PACKAGE, None).await.ok();
    common::wait_ui_stable().await;

    // 获取当前应用信息
    let result = device.app_current().await;
    assert!(result.is_ok(), "获取当前应用信息应该成功");

    if let Ok(info) = result {
        println!("当前应用:");
        println!("  包名: {}", info.package);
        println!("  Activity: {}", info.activity);
        if let Some(pid) = info.pid {
            println!("  PID: {}", pid);
        }

        assert!(!info.package.is_empty(), "包名不应该为空");
        assert!(!info.activity.is_empty(), "Activity 不应该为空");
    }

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_app_wait() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 先停止应用（如果正在运行）
    device.app_stop(common::TEST_APP_PACKAGE).await.ok();
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 启动应用
    device.app_start(common::TEST_APP_PACKAGE, None).await.ok();

    // 等待应用启动
    let result = device
        .app_wait(common::TEST_APP_PACKAGE, Some(Duration::from_secs(10)))
        .await;
    assert!(result.is_ok(), "等待应用启动应该成功");

    if let Ok(pid) = result {
        println!("✓ 应用启动成功，PID: {}", pid);
        assert!(pid > 0, "PID 应该大于 0");
    }

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_app_wait_timeout() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 等待一个不存在的应用
    let result = device
        .app_wait("com.nonexistent.app.12345", Some(Duration::from_secs(2)))
        .await;
    assert!(result.is_err(), "等待不存在的应用应该超时");

    println!("✓ 应用等待超时功能正常");
}

#[tokio::test]
async fn test_app_clear() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 先停止应用
    device.app_stop(common::TEST_APP_PACKAGE).await.ok();
    common::wait_ui_stable().await;

    // 清除应用数据
    let result = device.app_clear(common::TEST_APP_PACKAGE).await;

    // 注意：清除系统应用数据可能需要特殊权限，可能会失败
    match result {
        Ok(_) => println!("✓ 清除应用数据成功"),
        Err(e) => println!("⚠️  清除应用数据失败（可能需要特殊权限）: {}", e),
    }
}

#[tokio::test]
async fn test_app_lifecycle() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 完整的应用生命周期测试

    // 1. 停止应用
    device.app_stop(common::TEST_APP_PACKAGE).await.ok();
    println!("1. 应用已停止");
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 2. 启动应用
    device
        .app_start(common::TEST_APP_PACKAGE, None)
        .await
        .unwrap();
    println!("2. 应用已启动");
    common::wait_ui_stable().await;

    // 3. 等待应用就绪
    let pid = device
        .app_wait(common::TEST_APP_PACKAGE, Some(Duration::from_secs(10)))
        .await
        .unwrap();
    println!("3. 应用就绪，PID: {}", pid);

    // 4. 获取当前应用信息
    let info = device.app_current().await.unwrap();
    println!("4. 当前应用: {}", info.package);

    // 5. 停止应用
    device.app_stop(common::TEST_APP_PACKAGE).await.unwrap();
    println!("5. 应用已停止");

    println!("✓ 应用生命周期测试完成");
}

#[tokio::test]
async fn test_screenshot_during_app_operation() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 启动应用
    device.app_start(common::TEST_APP_PACKAGE, None).await.ok();
    common::wait_ui_stable().await;

    // 在应用运行时截图
    let result = device.screenshot().await;
    assert!(result.is_ok(), "应用运行时截图应该成功");

    if let Ok(image) = result {
        println!("✓ 应用运行时截图成功: {}x{}", image.width(), image.height());
    }

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_multiple_screenshots() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 连续截图多次
    for i in 0..3 {
        let result = device.screenshot().await;
        assert!(result.is_ok(), "第 {} 次截图应该成功", i + 1);

        if let Ok(image) = result {
            println!("第 {} 次截图: {}x{}", i + 1, image.width(), image.height());
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    println!("✓ 多次截图成功");
}
