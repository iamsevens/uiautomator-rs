//! 应用测试示例
//!
//! 本示例展示如何使用 uiautomator 进行应用自动化测试, 包括:
//! - 启动和停止应用
//! - 等待应用启动
//! - 元素定位和交互
//! - 验证测试状态
//! - 截图记录
//!
//! 示例使用 Android 设置应用作为演示
//!
//! 运行方式:
//! ```bash
//! cargo run --example app_test
//! ```

use std::time::Duration;
use uiautomator::{Device, Key, Selector};

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志系统
    uiautomator::init_logger();

    println!("=== 应用自动化测试示例 ===\n");

    // 连接到设备
    println!("连接到设备...");
    let device = Device::connect(None).await?;
    println!("设备连接成功\n");

    // 测试配置
    let package_name = "com.android.settings";
    let test_name = "设置应用测试";

    println!("开始测试: {}", test_name);
    println!("目标应用: {}\n", package_name);

    // 步骤 1: 确保应用已停止
    println!("步骤 1: 停止应用(清理测试环境)");
    device.app_stop(package_name).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("应用已停止\n");

    // 步骤 2: 启动应用
    println!("步骤 2: 启动应用");
    device.app_start(package_name, None).await?;
    println!("启动命令已发送\n");

    // 步骤 3: 等待应用启动
    println!("步骤 3: 等待应用启动完成(超时 10 秒)");
    match device
        .app_wait(package_name, Some(Duration::from_secs(10)))
        .await
    {
        Ok(pid) => {
            println!("应用已启动, PID: {}\n", pid);
        }
        Err(e) => {
            println!("应用启动超时: {}\n", e);
            return Err(e);
        }
    }

    // 步骤 4: 验证当前应用
    println!("步骤 4: 验证当前应用");
    let current_app = device.app_current().await?;
    println!("当前应用:");
    println!("  - 包名: {}", current_app.package);
    println!("  - Activity: {}", current_app.activity);
    if let Some(pid) = current_app.pid {
        println!("  - PID: {}", pid);
    }

    if current_app.package == package_name {
        println!("应用验证成功\n");
    } else {
        println!("当前应用不匹配\n");
    }

    // 步骤 5: 截图记录初始状态
    println!("步骤 5: 截图记录初始状态");
    device.screenshot_to_file("app_test_initial.png").await?;
    println!("截图已保存到: app_test_initial.png\n");

    // 步骤 6: 查找和交互元素
    println!("步骤 6: 查找界面元素");

    // 尝试查找搜索按钮(多种方式)
    let search_selectors = vec![
        Selector::new().description("Search settings"),
        Selector::new().description("搜索设置"),
        Selector::new().text("Search"),
        Selector::new().text("搜索"),
        Selector::new().resource_id("com.android.settings:id/search_action_bar"),
    ];

    let mut found_search = false;
    for selector in search_selectors {
        if device
            .find(selector.clone())
            .exists(Some(Duration::from_secs(2)))
            .await?
        {
            println!("找到搜索元素");
            let info = device.find(selector.clone()).info().await?;
            println!("  元素信息:");
            println!("    - 文本: {}", info.text);
            println!("    - 描述: {}", info.content_description);
            println!("    - 类名: {}", info.class_name);
            println!("    - 资源ID: {}", info.resource_id);
            found_search = true;
            break;
        }
    }

    if !found_search {
        println!("  未找到搜索元素, 可能取决于设备和系统版本");
    }
    println!();

    // 步骤 7: 执行滑动
    println!("步骤 7: 执行滑动操作");
    let (width, height) = device.window_size().await?;
    let start_x = width / 2;
    let start_y = height * 3 / 4;
    let end_x = width / 2;
    let end_y = height / 4;

    println!("  向上滑动...");
    device
        .swipe(
            start_x,
            start_y,
            end_x,
            end_y,
            Some(Duration::from_millis(500)),
        )
        .await?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("滑动完成\n");

    // 步骤 8: 截图记录滑动后状态
    println!("步骤 8: 截图记录滑动后状态");
    device.screenshot_to_file("app_test_scrolled.png").await?;
    println!("截图已保存到: app_test_scrolled.png\n");

    // 步骤 9: 按返回键
    println!("步骤 9: 按返回键");
    device.press(Key::Back).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("已按键\n");

    // 步骤 10: 返回主屏幕
    println!("步骤 10: 返回主屏幕");
    device.press(Key::Home).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("已按键\n");

    // 步骤 11: 清理 - 停止应用
    println!("步骤 11: 清理 - 停止应用");
    device.app_stop(package_name).await?;
    println!("应用已停止\n");

    println!("=== 测试完成 ===");
    println!("\n测试总结:");
    println!("所有测试步骤执行成功");
    println!("生成了 2 张截图");
    println!("\n提示:");
    println!("- 可以根据实际应用修改包名和元素选择器");
    println!("- 使用 Selector 的各种方法组合定位元素");
    println!("- 使用 wait() 方法等待元素出现");
    println!("- 使用截图记录测试过程");

    Ok(())
}
