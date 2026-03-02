//! 应用管理示例
//!
//! 演示如何使用 uiautomator 管理 Android 应用

use std::time::Duration;
use uiautomator::{Device, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();

    println!("=== 应用管理示例 ===\n");

    // 连接到设备
    println!("正在连接到设备...");
    let device = Device::connect(None).await?;
    println!("设备连接成功: {}\n", device.serial());

    // 示例 1: 启动应用(仅包名)
    println!("示例 1: 启动应用(仅包名)");
    println!("正在启动应用...");
    device.app_start("com.android.settings", None).await?;
    println!("应用启动成功\n");

    // 等待应用启动
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 示例 2: 获取当前应用信息
    println!("示例 2: 获取当前应用信息");
    let current = device.app_current().await?;
    println!("当前应用:");
    println!("  包名: {}", current.package);
    println!("  Activity: {}", current.activity);
    if let Some(pid) = current.pid {
        println!("  PID: {}", pid);
    }
    println!();

    // 示例 3: 启动应用(指定 Activity)
    println!("示例 3: 启动应用(指定 Activity)");
    println!("正在启动应用到设置页...");
    device
        .app_start("com.android.settings", Some(".Settings"))
        .await?;
    println!("应用启动成功\n");

    tokio::time::sleep(Duration::from_secs(2)).await;

    // 示例 4: 等待应用启动
    println!("示例 4: 等待应用启动");
    println!("等待设置应用启动...");
    let pid = device
        .app_wait("com.android.settings", Some(Duration::from_secs(10)))
        .await?;
    println!("应用已启动, PID: {}\n", pid);

    // 示例 5: 停止应用
    println!("示例 5: 停止应用");
    println!("停止设置应用...");
    device.app_stop("com.android.settings").await?;
    println!("应用停止成功\n");

    tokio::time::sleep(Duration::from_secs(1)).await;

    // 示例 6: 清除应用数据
    println!("示例 6: 清除应用数据");
    println!("清除设置应用数据...");
    match device.app_clear("com.android.settings").await {
        Ok(_) => println!("应用数据清除成功\n"),
        Err(e) => println!("应用数据清除失败: {:?}\n", e),
    }

    // 示例 7: 完整的应用操作流程
    println!("示例 7: 完整的应用操作流程");
    println!("1. 停止应用");
    device.app_stop("com.android.settings").await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("2. 清除应用数据");
    let _ = device.app_clear("com.android.settings").await;

    println!("3. 启动应用");
    device.app_start("com.android.settings", None).await?;

    println!("4. 等待应用启动");
    let pid = device
        .app_wait("com.android.settings", Some(Duration::from_secs(10)))
        .await?;
    println!("   应用已启动, PID: {}", pid);

    println!("5. 验证应用在前台");
    let current = device.app_current().await?;
    assert_eq!(current.package, "com.android.settings");
    println!("   验证成功: {} 在前台", current.package);

    println!("\n=== 所有示例完成 ===");

    Ok(())
}
