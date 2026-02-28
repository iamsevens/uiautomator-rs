//! 并发操作示例
//!
//! 本示例展示如何使用 uiautomator 同时控制多个设备并发操作:
//! - 并发连接多个设备
//! - 同时执行操作
//! - 使用 tokio 的并发原语
//! - 处理并发错误和资源
//!
//! 注意: 需要连接多个设备才能运行此示例
//!
//! 运行方式:
//! ```bash
//! cargo run --example concurrent
//! ```

use std::time::Duration;
use tokio::time::Instant;
use uiautomator::{AdbClient, Device, Key};

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志系统
    uiautomator::init_logger();

    println!("=== 并发操作示例 ===\n");

    // 1. 列出所有连接的设备
    println!("1. 列出所有设备...");
    let adb = AdbClient::new().await?;
    let devices: Vec<String> = adb.devices().await?;

    if devices.is_empty() {
        println!("没有找到连接的设备");
        println!("\n请确保:");
        println!("- 至少连接一个 Android 设备或启动模拟器");
        println!("- ADB 服务器正在运行");
        println!("- 设备已启用 USB 调试");
        return Ok(());
    }

    println!("找到 {} 个设备", devices.len());
    for (i, serial) in devices.iter().enumerate() {
        println!("  {}. {}", i + 1, serial);
    }
    println!();

    // 2. 并发连接所有设备
    println!("2. 并发连接所有设备...");
    let start = Instant::now();

    let mut connect_futures = vec![];
    for serial in devices.iter() {
        let serial_clone = serial.clone();
        connect_futures.push(async move { Device::connect(Some(&serial_clone)).await });
    }

    // 使用 futures::future::join_all 等待所有连接完成
    let results = futures::future::join_all(connect_futures).await;

    let mut connected_devices: Vec<(String, Device)> = vec![];
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(device) => {
                println!("  设备 {} 连接成功", devices[i]);
                connected_devices.push((devices[i].clone(), device));
            }
            Err(e) => {
                println!("  设备 {} 连接失败: {}", devices[i], e);
            }
        }
    }

    let elapsed = start.elapsed();
    println!("连接完成, 耗时: {:?}\n", elapsed);

    if connected_devices.is_empty() {
        println!("没有成功连接的设备");
        return Ok(());
    }

    // 3. 并发获取设备信息
    println!("3. 并发获取所有设备信息...");
    let start = Instant::now();

    let mut info_futures = vec![];
    for (serial, device) in connected_devices.iter() {
        let serial_clone = serial.clone();
        let device_clone = device.clone();
        info_futures.push(async move {
            let info = device_clone.info().await?;
            Ok::<_, uiautomator::Error>((serial_clone, info))
        });
    }

    let results = futures::future::join_all(info_futures).await;

    for result in results {
        match result {
            Ok((serial, info)) => {
                println!("  设备 {}:", serial);
                println!("    - 屏幕: {}x{}", info.display_width, info.display_height);
                println!("    - SDK: {}", info.sdk_int);
                println!("    - 当前应用: {}", info.current_package_name);
            }
            Err(e) => {
                println!("  获取信息失败: {}", e);
            }
        }
    }

    let elapsed = start.elapsed();
    println!("信息获取完成, 耗时: {:?}\n", elapsed);

    // 4. 并发执行相同操作
    println!("4. 并发在所有设备上按 Home 键...");
    let start = Instant::now();

    let mut press_futures = vec![];
    for (serial, device) in connected_devices.iter() {
        let serial_clone = serial.clone();
        let device_clone = device.clone();
        press_futures.push(async move {
            device_clone.press(Key::Home).await?;
            Ok::<_, uiautomator::Error>(serial_clone)
        });
    }

    let results = futures::future::join_all(press_futures).await;

    for result in results {
        match result {
            Ok(serial) => {
                println!("  设备 {} 已按键", serial);
            }
            Err(e) => {
                println!("  按键失败: {}", e);
            }
        }
    }

    let elapsed = start.elapsed();
    println!("按键完成, 耗时: {:?}\n", elapsed);

    // 5. 并发截图
    println!("5. 并发在所有设备上截图...");
    let start = Instant::now();

    let mut screenshot_futures = vec![];
    for (i, (serial, device)) in connected_devices.iter().enumerate() {
        let serial_clone = serial.clone();
        let device_clone = device.clone();
        let filename = format!("concurrent_device_{}.png", i + 1);
        screenshot_futures.push(async move {
            device_clone.screenshot_to_file(&filename).await?;
            Ok::<_, uiautomator::Error>((serial_clone, filename))
        });
    }

    let results = futures::future::join_all(screenshot_futures).await;

    for result in results {
        match result {
            Ok((serial, filename)) => {
                println!("  设备 {} 截图已保存到: {}", serial, filename);
            }
            Err(e) => {
                println!("  截图失败: {}", e);
            }
        }
    }

    let elapsed = start.elapsed();
    println!("截图完成, 耗时: {:?}\n", elapsed);

    // 6. 演示错误处理
    println!("6. 演示错误处理...");
    println!("  尝试在所有设备上查找不存在的元素");

    let mut exists_futures = vec![];
    for (serial, device) in connected_devices.iter() {
        let serial_clone = serial.clone();
        let device_clone = device.clone();
        exists_futures.push(async move {
            let selector = uiautomator::Selector::new().text("ThisElementDoesNotExist12345");
            let exists = device_clone
                .find(selector)
                .exists(Some(Duration::from_secs(2)))
                .await?;
            Ok::<_, uiautomator::Error>((serial_clone, exists))
        });
    }

    let results = futures::future::join_all(exists_futures).await;

    for result in results {
        match result {
            Ok((serial, exists)) => {
                if exists {
                    println!("  设备 {} 找到元素(意外)", serial);
                } else {
                    println!("  设备 {} 未找到元素(预期)", serial);
                }
            }
            Err(e) => {
                println!("  设备检查失败: {}", e);
            }
        }
    }
    println!();

    // 7. 使用 tokio::join! 同时等待多个操作
    if connected_devices.len() >= 2 {
        println!("7. 使用 tokio::join! 同时等待两个操作...");
        let device1 = &connected_devices[0].1;
        let device2 = &connected_devices[1].1;

        let start = Instant::now();
        let (result1, result2) = tokio::join!(device1.window_size(), device2.window_size());

        match (result1, result2) {
            (Ok(size1), Ok(size2)) => {
                println!("  设备 1 屏幕: {}x{}", size1.0, size1.1);
                println!("  设备 2 屏幕: {}x{}", size2.0, size2.1);
            }
            _ => {
                println!("  部分操作失败");
            }
        }

        let elapsed = start.elapsed();
        println!("完成, 耗时: {:?}\n", elapsed);
    }

    println!("=== 示例完成 ===");
    println!("\n总结:");
    println!("成功演示了并发控制 {} 个设备", connected_devices.len());
    println!("展示了 futures::future::join_all 和 tokio::join! 的使用");
    println!("展示了处理并发错误和资源");
    println!("\n提示:");
    println!("- Device 实现了 Clone, 可以安全地在多个任务间共享");
    println!("- 使用 futures::future::join_all 并发执行多个操作");
    println!("- 使用 tokio::join! 同时等待少量操作");
    println!("- 注意处理每个操作可能的错误");

    Ok(())
}
