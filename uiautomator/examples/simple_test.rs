//! 简化测试 - 直接连接已运行的服务

use std::time::Duration;
use uiautomator::{Device, Selector};

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    println!("🔌 连接设备...");

    // 使用 connect_quick 跳过 ATX-Agent
    let device = Device::connect_quick(None).await?;
    println!("✅ 设备连接成功: {}", device.serial());

    // 获取设备信息
    println!("\n📱 获取设备信息...");
    let info = device.info().await?;
    println!("  屏幕: {}x{}", info.display_width, info.display_height);
    println!("  SDK: {}", info.sdk_int);
    println!("  当前应用: {}", info.current_package_name);

    // 点击 BASIC CONTROLS 按钮
    println!("\n🎯 点击 BASIC CONTROLS 按钮...");
    device
        .find(Selector::new().resource_id("com.uiautomator.testapp:id/btn_basic_controls"))
        .click(None, None)
        .await?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    // 点击普通按钮
    println!("🎯 点击普通按钮...");
    device
        .find(Selector::new().resource_id("com.uiautomator.testapp:id/btn_normal"))
        .click(None, None)
        .await?;

    tokio::time::sleep(Duration::from_millis(500)).await;

    // 获取结果
    println!("📝 获取结果文本...");
    let result = device
        .find(Selector::new().resource_id("com.uiautomator.testapp:id/tv_result"))
        .get_text()
        .await?;
    println!("✅ 结果: {}", result);

    println!("\n🎉 测试完成！");

    Ok(())
}
