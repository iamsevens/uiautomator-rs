//! Device 核心功能示例
//!
//! 本示例展示如何使用 Device 的核心功能:
//! - 连接到设备
//! - 获取设备信息
//! - 获取屏幕尺寸
//! - 查找 UI 元素

use uiautomator::{Device, Selector, ServerMode};

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志系统
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("=== Device 核心功能示例 ===\n");

    // 1. 连接到设备(自动选择)
    println!("1. 连接到设备...");
    let device = Device::connect(None).await?;
    println!("   已成功连接到设备: {}", device.serial());
    println!("   使用服务器模式: {:?}", device.server_mode());
    println!();

    // 2. 获取设备信息
    println!("2. 获取设备信息...");
    let info = device.info().await?;
    println!(
        "   屏幕尺寸: {}x{}",
        info.display_width, info.display_height
    );
    println!("   屏幕旋转: {}度", info.display_rotation * 90);
    println!("   当前应用: {}", info.current_package_name);
    println!("   SDK 版本: {}", info.sdk_int);
    println!(
        "   屏幕状态: {}",
        if info.screen_on { "开启" } else { "熄灭" }
    );
    println!();

    // 3. 获取屏幕尺寸(简化方法)
    println!("3. 获取屏幕尺寸(简化方法)...");
    let (width, height) = device.window_size().await?;
    println!("   宽度: {} 像素", width);
    println!("   高度: {} 像素", height);
    println!();

    // 4. 查找 UI 元素
    println!("4. 查找 UI 元素(选择器)...");

    // 通过文本查找
    let selector1 = Selector::new().text("Settings");
    let element1 = device.find(selector1);
    println!("   已创建通过文本查找的元素: {:?}", element1.selector());

    // 通过资源 ID 查找
    let selector2 = Selector::new().resource_id("com.android.settings:id/search");
    let element2 = device.find(selector2);
    println!("   已创建通过资源 ID 查找的元素: {:?}", element2.selector());

    // 组合多个条件
    let selector3 = Selector::new()
        .text("Settings")
        .class_name("android.widget.TextView")
        .clickable(true);
    let element3 = device.find(selector3);
    println!("   已创建组合条件查找的元素: {:?}", element3.selector());
    println!();

    // 5. 演示不同的连接方法
    println!("5. 演示不同的连接方法...");

    // 快速连接(Direct 模式)
    println!("   - 快速连接(Direct 模式)");
    let device_quick = Device::connect_quick(None).await?;
    println!("     已连接: {}", device_quick.serial());

    // 指定模式连接
    println!("   - 指定模式连接(Direct 模式)");
    let device_with_mode = Device::connect_with_mode(None, ServerMode::Direct).await?;
    println!("     已连接: {}", device_with_mode.serial());
    println!();

    // 6. 查看设备配置
    println!("6. 查看设备配置...");
    println!("   - 设备序列号: {}", device.serial());
    println!("   - 服务器模式: {:?}", device.server_mode());

    // 获取配置
    let settings = device.settings();
    let settings_guard = settings.read().unwrap();
    println!("   - 等待超时: {:?}", settings_guard.wait_timeout);
    println!("   - 最大重试次数: {}", settings_guard.max_retry);
    println!();

    println!("=== 所有示例完成 ===");

    Ok(())
}
