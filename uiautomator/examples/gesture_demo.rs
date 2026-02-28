//! 手势操作示例
//!
//! 演示如何使用 Device 的手势操作功能，包括�?
//! - 坐标转换（百分比和像素）
//! - 点击、长按、双�?
//! - 滑动和拖�?

use std::time::Duration;
use uiautomator::{Device, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日�?
    env_logger::init();

    println!("=== 手势操作示例 ===\n");

    // 连接到设�?
    println!("正在连接到设�?..");
    let device = Device::connect(None).await?;
    println!("设备连接成功: {}\n", device.serial());

    // 获取屏幕尺寸
    let (width, height) = device.window_size().await?;
    println!("屏幕尺寸: {}x{}\n", width, height);

    // ========== 坐标转换示例 ==========
    println!("=== 坐标转换示例 ===");

    // 百分比坐标转�?
    let (x, y) = device.pos_rel2abs(0.5, 0.5).await?;
    println!("屏幕中心（百分比 0.5, 0.5�? ({}, {})", x, y);

    let (x, y) = device.pos_rel2abs(0.25, 0.75).await?;
    println!("左下区域（百分比 0.25, 0.75�? ({}, {})", x, y);

    // 像素坐标直接返回
    let (x, y) = device.pos_rel2abs(100.0, 200.0).await?;
    println!("像素坐标�?00, 200�? ({}, {})\n", x, y);

    // ========== 点击操作示例 ==========
    println!("=== 点击操作示例 ===");

    // 基本点击
    println!("点击屏幕中心...");
    let (center_x, center_y) = device.pos_rel2abs(0.5, 0.5).await?;
    device.click(center_x, center_y).await?;
    println!("点击完成");

    tokio::time::sleep(Duration::from_secs(1)).await;

    // 长按
    println!("长按屏幕中心 1 �?..");
    device
        .long_click(center_x, center_y, Some(Duration::from_secs(1)))
        .await?;
    println!("长按完成");

    tokio::time::sleep(Duration::from_secs(1)).await;

    // 双击
    println!("双击屏幕中心...");
    device.double_click(center_x, center_y, None).await?;
    println!("双击完成\n");

    tokio::time::sleep(Duration::from_secs(1)).await;

    // ========== 滑动操作示例 ==========
    println!("=== 滑动操作示例 ===");

    // 从左向右滑动
    println!("从左向右滑动...");
    let (left_x, mid_y) = device.pos_rel2abs(0.2, 0.5).await?;
    let (right_x, _) = device.pos_rel2abs(0.8, 0.5).await?;
    device
        .swipe(
            left_x,
            mid_y,
            right_x,
            mid_y,
            Some(Duration::from_millis(300)),
        )
        .await?;
    println!("滑动完成");

    tokio::time::sleep(Duration::from_secs(1)).await;

    // 向上滑动（模拟滚动）
    println!("向上滑动（模拟滚动）...");
    let (mid_x, bottom_y) = device.pos_rel2abs(0.5, 0.8).await?;
    let (_, top_y) = device.pos_rel2abs(0.5, 0.2).await?;
    device
        .swipe(
            mid_x,
            bottom_y,
            mid_x,
            top_y,
            Some(Duration::from_millis(400)),
        )
        .await?;
    println!("滑动完成");

    tokio::time::sleep(Duration::from_secs(1)).await;

    // 对角线滑�?
    println!("对角线滑�?..");
    let (tl_x, tl_y) = device.pos_rel2abs(0.2, 0.2).await?;
    let (br_x, br_y) = device.pos_rel2abs(0.8, 0.8).await?;
    device
        .swipe(tl_x, tl_y, br_x, br_y, Some(Duration::from_millis(500)))
        .await?;
    println!("滑动完成\n");

    tokio::time::sleep(Duration::from_secs(1)).await;

    // ========== 拖拽操作示例 ==========
    println!("=== 拖拽操作示例 ===");

    // 拖拽操作（与滑动类似，但会保持按下状态）
    println!("拖拽元素...");
    let (start_x, start_y) = device.pos_rel2abs(0.3, 0.3).await?;
    let (end_x, end_y) = device.pos_rel2abs(0.7, 0.7).await?;
    device
        .drag(
            start_x,
            start_y,
            end_x,
            end_y,
            Some(Duration::from_millis(800)),
        )
        .await?;
    println!("拖拽完成\n");

    // ========== 组合操作示例 ==========
    println!("=== 组合操作示例 ===");

    // 模拟缩放手势（需要多点触控，这里只是示例�?
    println!("模拟放大手势（两次向外滑动）...");
    let (center_x, center_y) = device.pos_rel2abs(0.5, 0.5).await?;
    let (out_x, out_y) = device.pos_rel2abs(0.7, 0.7).await?;

    // 第一个手指向外滑�?
    device
        .swipe(
            center_x,
            center_y,
            out_x,
            out_y,
            Some(Duration::from_millis(300)),
        )
        .await?;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // 第二个手指向外滑动（反方向）
    let (in_x, in_y) = device.pos_rel2abs(0.3, 0.3).await?;
    device
        .swipe(
            center_x,
            center_y,
            in_x,
            in_y,
            Some(Duration::from_millis(300)),
        )
        .await?;
    println!("放大手势完成\n");

    println!("=== 示例完成 ===");

    Ok(())
}
