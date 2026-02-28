//! ADB 客户端使用示例
//!
//! 本示例展示如何使用 AdbClient 连接设备, 执行命令和传输文件
//!
//! 注意: 此示例需要在 Windows 环境下运行, 且 ADB 服务器已启动

use std::time::Duration;
use uiautomator::adb::AdbClient;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志系统
    uiautomator::init_logger();

    println!("=== ADB 客户端示例 (Windows 版本) ===\n");

    // 1. 创建 ADB 客户端
    println!("1. 创建 ADB 客户端...");
    let client = AdbClient::new().await?;
    println!("   已成功创建\n");

    // 2. 列出连接的设备
    println!("2. 获取设备列表...");
    let devices = client.devices().await?;
    println!("   找到 {} 个设备", devices.len());
    for device in &devices {
        println!("   - {}", device);
    }
    println!();

    if devices.is_empty() {
        println!("没有找到设备, 请确保设备已连接并且 ADB 服务器已启动");
        return Ok(());
    }

    // 使用第一个设备进行后续操作
    let serial = &devices[0];
    println!("使用设备: {}\n", serial);

    // 3. 执行 shell 命令
    println!("3. 执行 shell 命令...");

    // 获取 Android 版本
    let sdk_version = client
        .shell(
            serial,
            "getprop ro.build.version.sdk",
            Some(Duration::from_secs(5)),
        )
        .await?;
    println!("   SDK 版本: {}", sdk_version.trim());

    // 获取设备型号
    let model = client
        .shell(
            serial,
            "getprop ro.product.model",
            Some(Duration::from_secs(5)),
        )
        .await?;
    println!("   设备型号: {}", model.trim());

    // 获取屏幕尺寸
    let wm_size = client
        .shell(serial, "wm size", Some(Duration::from_secs(5)))
        .await?;
    println!("   屏幕尺寸: {}", wm_size.trim());
    println!();

    // 4. 文件传输示例
    println!("4. 文件传输示例...");

    // 创建一个测试文件 (Windows 路径)
    let test_content = "Hello from uiautomator on Windows!";
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test.txt");
    std::fs::write(&test_file, test_content)?;
    println!("   已创建测试文件: {}", test_file.display());

    // 推送文件到设备
    client
        .push(
            serial,
            test_file.to_str().unwrap(),
            "/data/local/tmp/test.txt",
        )
        .await?;
    println!("   文件已推送到设备");

    // 验证文件内容
    let content = client
        .shell(
            serial,
            "cat /data/local/tmp/test.txt",
            Some(Duration::from_secs(5)),
        )
        .await?;
    println!("   设备上的文件内容: {}", content.trim());

    // 拉取文件回来
    let pulled_file = temp_dir.join("test_pulled.txt");
    client
        .pull(
            serial,
            "/data/local/tmp/test.txt",
            pulled_file.to_str().unwrap(),
        )
        .await?;
    println!("   文件已从设备拉取");

    // 验证拉取的内容
    let pulled_content = std::fs::read_to_string(&pulled_file)?;
    println!("   拉取的文件内容: {}", pulled_content.trim());
    println!();

    // 5. 端口转发示例
    println!("5. 端口转发示例...");
    client.forward(serial, 9008, 9008).await?;
    println!("   端口转发已建立: localhost:9008 -> device:9008");
    println!();

    println!("=== 示例完成 ===");

    Ok(())
}
