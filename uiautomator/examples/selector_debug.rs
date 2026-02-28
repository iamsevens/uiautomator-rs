// 选择器调试工具 - 查看服务端实际返回的数据
//
// 运行方式：
// cargo run --example selector_debug

use std::time::Duration;
use uiautomator::{Device, Selector};

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志（显示详细信息）
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("=== 选择器调试工具 ===\n");

    // 连接设备
    let device = Device::connect(None).await?;
    println!("✓ 已连接到设备: {}\n", device.serial());

    // 获取设备信息
    let info = device.info().await?;
    println!("设备信息:");
    println!("  分辨率: {}x{}", info.display_width, info.display_height);
    println!("  当前应用: {}", info.current_package_name);
    println!("  SDK 版本: {}\n", info.sdk_int);

    // 测试 1: 查找任意 TextView
    println!("--- 测试 1: 查找任意 TextView ---");
    let selector = Selector::new().class_name("android.widget.TextView");
    println!("选择器: {:?}", selector);

    match device.find(selector.clone()).info().await {
        Ok(element_info) => {
            println!("✓ 找到元素:");
            println!("  文本: '{}'", element_info.text);
            println!("  类名: {}", element_info.class_name);
            println!("  资源ID: {}", element_info.resource_id);
            println!(
                "  边界: ({}, {}) - ({}, {})",
                element_info.bounds.left,
                element_info.bounds.top,
                element_info.bounds.right,
                element_info.bounds.bottom
            );
            println!("  可点击: {}", element_info.clickable);
            println!("  已启用: {}", element_info.enabled);
        }
        Err(e) => {
            println!("✗ 未找到元素: {}", e);
        }
    }
    println!();

    // 测试 2: 使用 exists 方法
    println!("--- 测试 2: 使用 exists 方法 ---");
    let exists = device
        .find(selector.clone())
        .exists(Some(Duration::from_secs(2)))
        .await?;
    println!("元素存在: {}", exists);
    println!();

    // 测试 3: 查找 FrameLayout（可能是根元素）
    println!("--- 测试 3: 查找 FrameLayout ---");
    let frame_selector = Selector::new().class_name("android.widget.FrameLayout");
    println!("选择器: {:?}", frame_selector);

    match device.find(frame_selector).info().await {
        Ok(element_info) => {
            println!("✓ 找到元素:");
            println!("  文本: '{}'", element_info.text);
            println!("  类名: {}", element_info.class_name);
            println!("  资源ID: {}", element_info.resource_id);
            println!(
                "  边界: ({}, {}) - ({}, {})",
                element_info.bounds.left,
                element_info.bounds.top,
                element_info.bounds.right,
                element_info.bounds.bottom
            );

            // 检查是否可能是根元素
            let is_likely_root = element_info.class_name.contains("FrameLayout")
                && element_info.bounds.left == 0
                && element_info.bounds.top == 0
                && element_info.text.is_empty()
                && element_info.resource_id.is_empty();
            println!("  可能是根元素: {}", is_likely_root);
        }
        Err(e) => {
            println!("✗ 未找到元素: {}", e);
        }
    }
    println!();

    // 测试 4: 查找不存在的元素
    println!("--- 测试 4: 查找不存在的元素 ---");
    let nonexistent = Selector::new().text("这个元素绝对不存在_12345");
    println!("选择器: {:?}", nonexistent);

    match device.find(nonexistent).info().await {
        Ok(element_info) => {
            println!("⚠ 意外找到元素:");
            println!("  文本: '{}'", element_info.text);
            println!("  类名: {}", element_info.class_name);
            println!("  这可能表示服务端返回了根元素或其他元素");
        }
        Err(e) => {
            println!("✓ 正确返回错误: {}", e);
        }
    }
    println!();

    // 测试 5: 使用 instance 参数
    println!("--- 测试 5: 使用 instance 参数 ---");
    for i in 0..3 {
        let instance_selector = Selector::new()
            .class_name("android.widget.TextView")
            .instance(i);
        println!("查找第 {} 个 TextView:", i);

        match device.find(instance_selector).get_text().await {
            Ok(text) => println!("  文本: '{}'", text),
            Err(e) => println!("  错误: {}", e),
        }
    }
    println!();

    println!("=== 调试完成 ===");

    Ok(())
}
