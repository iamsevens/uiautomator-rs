//! 按键操作示例
//!
//! 演示如何使用 uiautomator 进行按键操作
//!
//! 运行方式：
//! ```bash
//! cargo run --example key_demo
//! ```

use std::time::Duration;
use uiautomator::{Device, Key};

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志
    uiautomator::init_logger();

    println!("=== 按键操作示例 ===\n");

    // 连接到设备
    println!("正在连接到设备...");
    let device = Device::connect(None).await?;
    println!("设备连接成功\n");

    // 示例 1: 使用 Key 枚举按键
    println!("示例 1: 使用 Key 枚举按键");
    println!("按下 Home 键...");
    device.press(Key::Home).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("按下 Back 键...");
    device.press(Key::Back).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 示例 2: 按下方向键
    println!("\n示例 2: 按下方向键");
    println!("按下 Up 键...");
    device.press(Key::Up).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("按下 Down 键...");
    device.press(Key::Down).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("按下 Left 键...");
    device.press(Key::Left).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("按下 Right 键...");
    device.press(Key::Right).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 示例 3: 按下 Enter 键
    println!("\n示例 3: 按下 Enter 键");
    device.press(Key::Enter).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 示例 4: 使用键码按键
    println!("\n示例 4: 使用键码按键");
    println!("按下键码 3 (Home)...");
    device.press_keycode(3).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("按下键码 4 (Back)...");
    device.press_keycode(4).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 示例 5: 音量键
    println!("\n示例 5: 音量键");
    println!("按下音量增加键...");
    device.press(Key::VolumeUp).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("按下音量减少键...");
    device.press(Key::VolumeDown).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 示例 6: 媒体控制键
    println!("\n示例 6: 媒体控制键");
    println!("按下播放/暂停键...");
    device.press(Key::MediaPlayPause).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 示例 7: 其他常用键
    println!("\n示例 7: 其他常用键");
    println!("按下菜单键...");
    device.press(Key::Menu).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("按下最近任务键...");
    device.press(Key::Recent).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 示例 8: Key 枚举和键码的转换
    println!("\n示例 8: Key 枚举和键码的转换");
    let home_key = Key::Home;
    println!("Home 键的键码: {}", home_key.to_keycode());
    println!("Home 键的名称: {}", home_key.to_name());

    // 从名称创建 Key
    if let Some(key) = Key::from_name("back") {
        println!("从名称 'back' 创建的键: {:?}", key);
        println!("对应的键码: {}", key.to_keycode());
    }

    println!("\n=== 示例完成 ===");

    Ok(())
}
