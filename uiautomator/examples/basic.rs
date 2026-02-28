//! 基础使用示例
//!
//! 这个示例展示了 uiautomator 的基本功能，包括：
//! - 连接到设备
//! - 获取设备信息
//! - 元素定位和操作
//! - 手势操作
//! - 按键操作
//! - 截图
//!
//! 运行方式：
//! ```bash
//! cargo run --example basic
//! ```

use std::time::Duration;
use uiautomator::{Device, Key, Selector};

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志系统
    uiautomator::init_logger();

    println!("=== uiautomator 基础使用示例 ===\n");

    // 1. 连接到设备
    println!("1. 连接到设备...");
    let device = Device::connect(None).await?;
    println!("   ✓ 设备连接成功\n");

    // 2. 获取设备信息
    println!("2. 获取设备信息...");
    let info = device.info().await?;
    println!(
        "   屏幕尺寸: {}x{}",
        info.display_width, info.display_height
    );
    println!("   旋转角度: {}°", info.display_rotation);
    println!("   SDK 版本: {}", info.sdk_int);
    println!(
        "   屏幕状态: {}",
        if info.screen_on { "点亮" } else { "熄灭" }
    );
    println!("   当前应用: {}\n", info.current_package_name);

    // 3. 按键操作
    println!("3. 按键操作...");
    println!("   按下 Home 键返回主屏幕");
    device.press(Key::Home).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("   ✓ 完成\n");

    // 4. 坐标点击
    println!("4. 坐标点击...");
    let (width, height) = device.window_size().await?;
    let center_x = width / 2;
    let center_y = height / 2;
    println!("   点击屏幕中心: ({}, {})", center_x, center_y);
    device.click(center_x, center_y).await?;
    println!("   ✓ 完成\n");

    // 5. 滑动操作
    println!("5. 滑动操作...");
    let start_x = width / 2;
    let start_y = height * 3 / 4;
    let end_x = width / 2;
    let end_y = height / 4;
    println!(
        "   从 ({}, {}) 滑动到 ({}, {})",
        start_x, start_y, end_x, end_y
    );
    device
        .swipe(
            start_x,
            start_y,
            end_x,
            end_y,
            Some(Duration::from_millis(500)),
        )
        .await?;
    println!("   ✓ 完成\n");

    // 6. 元素定位和操作
    println!("6. 元素定位和操作...");
    println!("   查找文本为 'Settings' 或 '设置' 的元素");

    // 尝试查找 Settings 按钮
    let settings_selector = Selector::new().text("Settings");
    let settings_exists = device
        .find(settings_selector.clone())
        .exists(Some(Duration::from_secs(2)))
        .await?;

    if settings_exists {
        println!("   找到 Settings 元素");
        let info = device.find(settings_selector.clone()).info().await?;
        println!("   元素信息:");
        println!("     - 文本: {}", info.text);
        println!("     - 类名: {}", info.class_name);
        println!("     - 可点击: {}", info.clickable);
        println!("     - 边界: {:?}", info.bounds);
    } else {
        println!("   未找到 Settings 元素(这是正常的,取决于当前界面)");
    }
    println!("   ✓ 完成\n");

    // 7. 截图
    println!("7. 截图...");
    let screenshot_path = "basic_example_screenshot.png";
    device.screenshot_to_file(screenshot_path).await?;
    println!("   ✓ 截图已保存到: {}\n", screenshot_path);

    // 8. 等待超时设置
    println!("8. 配置等待超时...");
    device.set_wait_timeout(Duration::from_secs(10));
    let timeout = device.get_wait_timeout();
    println!("   当前等待超时: {:?}\n", timeout);

    println!("=== 示例完成 ===");
    println!("\n提示:");
    println!("- 更多元素定位示例请参考 examples/uiobject_demo.rs");
    println!("- 更多手势操作示例请参考 examples/gesture_demo.rs");
    println!("- 更多应用管理示例请参考 examples/app_demo.rs");

    Ok(())
}
