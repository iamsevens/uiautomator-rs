//! 截图功能示例
//!
//! 演示如何使用 uiautomator 进行屏幕截图
//!
//! 运行方式:
//! ```bash
//! cargo run --example screenshot_demo
//! ```

use uiautomator::{Device, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();

    println!("=== uiautomator 截图功能示例 ===\n");

    // 连接到设备
    println!("正在连接到设备...");
    let device = Device::connect(None).await?;
    println!("设备连接成功: {}\n", device.serial());

    // 获取设备信息
    let info = device.info().await?;
    println!("设备信息:");
    println!("  屏幕尺寸: {}x{}", info.display_width, info.display_height);
    println!("  SDK 版本: {}", info.sdk_int);
    println!("  当前应用: {}\n", info.current_package_name);

    // 示例 1: 获取截图
    println!("示例 1: 获取截图");
    println!("正在截图...");
    let image = device.screenshot().await?;
    println!("截图成功");
    println!("  图片尺寸: {}x{}", image.width(), image.height());
    println!("  图片格式: {:?}\n", image.color());

    // 示例 2: 保存为 PNG 格式
    println!("示例 2: 保存为 PNG 格式");
    let png_path = "screenshot_demo.png";
    device.screenshot_to_file(png_path).await?;
    println!("截图已保存到: {}\n", png_path);

    // 示例 3: 保存为 JPEG 格式
    println!("示例 3: 保存为 JPEG 格式");
    let jpeg_path = "screenshot_demo.jpg";
    device.screenshot_to_file(jpeg_path).await?;
    println!("截图已保存到: {}\n", jpeg_path);

    // 示例 4: 连续截图
    println!("示例 4: 连续截图 (3 张)");
    for i in 1..=3 {
        let path = format!("screenshot_demo_{}.png", i);
        device.screenshot_to_file(&path).await?;
        println!("  第 {} 张截图已保存到: {}", i, path);

        // 添加延迟
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    println!();

    // 示例 5: 图片处理
    println!("示例 5: 图片处理");
    let image = device.screenshot().await?;

    // 转换为 RGB8 格式
    let _rgb_image = image.to_rgb8();
    println!("已转换为 RGB8 格式");

    // 转换为 RGBA8 格式
    let _rgba_image = image.to_rgba8();
    println!("已转换为 RGBA8 格式");

    // 裁剪图片(左上角 100x100)
    let cropped = image.crop_imm(0, 0, 100, 100);
    println!("已裁剪图片: {}x{}", cropped.width(), cropped.height());

    // 保存裁剪后的图片
    cropped.save("screenshot_demo_cropped.png")?;
    println!("裁剪后的图片已保存\n");

    // 示例 6: 验证截图尺寸
    println!("示例 6: 验证截图尺寸");
    let (screen_width, screen_height) = device.window_size().await?;
    let screenshot = device.screenshot().await?;

    if screenshot.width() == screen_width && screenshot.height() == screen_height {
        println!("截图尺寸与屏幕尺寸一致");
    } else {
        println!("截图尺寸与屏幕尺寸不一致");
    }
    println!("  屏幕: {}x{}", screen_width, screen_height);
    println!("  截图: {}x{}\n", screenshot.width(), screenshot.height());

    println!("=== 所有示例完成 ===");
    println!("\n生成的文件:");
    println!("  - screenshot_demo.png");
    println!("  - screenshot_demo.jpg");
    println!("  - screenshot_demo_1.png");
    println!("  - screenshot_demo_2.png");
    println!("  - screenshot_demo_3.png");
    println!("  - screenshot_demo_cropped.png");

    Ok(())
}
