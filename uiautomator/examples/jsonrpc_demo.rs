//! JSON-RPC 客户端示例
//!
//! 本示例展示如何使用 JsonRpcClient 与设备通信

use std::sync::Arc;
use uiautomator::adb::AdbClient;
use uiautomator::jsonrpc::JsonRpcClient;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    println!("=== JSON-RPC 客户端示例 ===\n");

    // 1. 创建 ADB 客户端
    println!("1. 创建 ADB 客户端...");
    let adb_client = Arc::new(AdbClient::new().await?);
    println!("   ADB 客户端已创建\n");

    // 2. 获取设备列表
    println!("2. 获取设备列表...");
    let devices = adb_client.devices().await?;

    if devices.is_empty() {
        println!("   未找到设备");
        println!("\n请确保:");
        println!("  - Android 设备已连接或模拟器正在运行");
        println!("  - ADB 服务器正在运行(adb devices)");
        println!("  - 设备已启用 USB 调试");
        return Ok(());
    }

    println!("   找到 {} 个设备", devices.len());
    for device in &devices {
        println!("     - {}", device);
    }
    println!();

    // 3. 选择第一个设备
    let device_serial = devices[0].clone();
    println!("3. 使用设备: {}", device_serial);

    // 4. 创建 JSON-RPC 客户端
    println!("4. 创建 JSON-RPC 客户端...");
    println!("   这将:");
    println!("   - 检查并推送 u2.jar 到设备");
    println!("   - 启动 UiAutomator 服务");
    println!("   - 建立端口转发");
    println!("   - 等待服务就绪");
    println!();

    let jsonrpc_client = JsonRpcClient::new(device_serial.clone(), adb_client.clone()).await?;
    println!("   JSON-RPC 客户端已就绪\n");

    // 5. 测试 ping
    println!("5. 测试服务连接...");
    let is_alive = jsonrpc_client.ping().await?;
    if is_alive {
        println!("   服务正常运行\n");
    } else {
        println!("   服务未响应\n");
        return Ok(());
    }

    println!("=== 示例完成 ===");
    println!("\nJSON-RPC 客户端已成功建立与设备的连接!");
    println!("现在可以使用 jsonrpc_client.call() 来调用 UiAutomator 服务的各种功能.");

    Ok(())
}
