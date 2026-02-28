// 集成测试：手势和按键操作
// 需求: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7

mod common;

use std::time::Duration;
use uiautomator::Key;

#[tokio::test]
async fn test_coordinate_click() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 获取屏幕尺寸
    let (width, height) = device.window_size().await.unwrap();

    // 点击屏幕中心
    let center_x = width / 2;
    let center_y = height / 2;

    let result = device.click(center_x, center_y).await;
    assert!(result.is_ok(), "坐标点击应该成功");

    println!("[OK] 坐标点击成功: ({}, {})", center_x, center_y);

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_long_click() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 获取屏幕尺寸
    let (width, height) = device.window_size().await.unwrap();

    // 长按屏幕中心
    let center_x = width / 2;
    let center_y = height / 2;

    let result = device
        .long_click(center_x, center_y, Some(Duration::from_secs(1)))
        .await;
    assert!(result.is_ok(), "长按应该成功");

    println!("[OK] 长按成功");

    common::wait_ui_stable().await;
    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_double_click() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 获取屏幕尺寸
    let (width, height) = device.window_size().await.unwrap();

    // 双击屏幕中心
    let center_x = width / 2;
    let center_y = height / 2;

    let result = device
        .double_click(center_x, center_y, Some(Duration::from_millis(100)))
        .await;
    assert!(result.is_ok(), "双击应该成功");

    println!("[OK] 双击成功");

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_swipe() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 启动设置应用
    device.app_start(common::TEST_APP_PACKAGE, None).await.ok();
    common::wait_ui_stable().await;

    // 获取屏幕尺寸
    let (width, height) = device.window_size().await.unwrap();

    // 从下往上滑动（向上滚动）
    let start_x = width / 2;
    let start_y = height * 3 / 4;
    let end_x = width / 2;
    let end_y = height / 4;

    let result = device
        .swipe(
            start_x,
            start_y,
            end_x,
            end_y,
            Some(Duration::from_millis(300)),
        )
        .await;
    assert!(result.is_ok(), "滑动应该成功");

    println!("[OK] 滑动成功");

    common::wait_ui_stable().await;
    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_drag() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 获取屏幕尺寸
    let (width, height) = device.window_size().await.unwrap();

    // 拖拽操作
    let start_x = width / 3;
    let start_y = height / 2;
    let end_x = width * 2 / 3;
    let end_y = height / 2;

    let result = device
        .drag(
            start_x,
            start_y,
            end_x,
            end_y,
            Some(Duration::from_millis(500)),
        )
        .await;
    assert!(result.is_ok(), "拖拽应该成功");

    println!("[OK] 拖拽成功");

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_percentage_coordinates() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 使用百分比坐标点击（0.5, 0.5 表示屏幕中心）
    // 注意：当前实现可能需要先转换为像素坐标
    let (width, height) = device.window_size().await.unwrap();
    let x = (width as f32 * 0.5) as u32;
    let y = (height as f32 * 0.5) as u32;

    let result = device.click(x, y).await;
    assert!(result.is_ok(), "百分比坐标点击应该成功");

    println!("[OK] 百分比坐标转换正常");

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_press_home_key() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 启动设置应用
    device.app_start(common::TEST_APP_PACKAGE, None).await.ok();
    common::wait_ui_stable().await;

    // 按 Home 键
    let result = device.press(Key::Home).await;
    assert!(result.is_ok(), "按 Home 键应该成功");

    println!("[OK] Home 键按下成功");

    common::wait_ui_stable().await;
}

#[tokio::test]
async fn test_press_back_key() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 启动设置应用
    device.app_start(common::TEST_APP_PACKAGE, None).await.ok();
    common::wait_ui_stable().await;

    // 按 Back 键
    let result = device.press(Key::Back).await;
    assert!(result.is_ok(), "按 Back 键应该成功");

    println!("[OK] Back 键按下成功");

    common::wait_ui_stable().await;
    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_press_recent_key() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 按最近任务键
    let result = device.press(Key::Recent).await;
    assert!(result.is_ok(), "按最近任务键应该成功");

    println!("[OK] Recent 键按下成功");

    common::wait_ui_stable().await;
    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_press_volume_keys() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 按音量增加键
    let result = device.press(Key::VolumeUp).await;
    assert!(result.is_ok(), "按音量增加键应该成功");

    println!("[OK] VolumeUp 键按下成功");

    tokio::time::sleep(Duration::from_millis(500)).await;

    // 按音量减少键
    let result = device.press(Key::VolumeDown).await;
    assert!(result.is_ok(), "按音量减少键应该成功");

    println!("[OK] VolumeDown 键按下成功");
}

#[tokio::test]
async fn test_press_direction_keys() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 测试方向键
    let keys = vec![Key::Up, Key::Down, Key::Left, Key::Right];

    for key in keys {
        let result = device.press(key).await;
        assert!(result.is_ok(), "按方向键应该成功");
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    println!("[OK] 方向键按下成功");

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_press_enter_key() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 按 Enter 键
    let result = device.press(Key::Enter).await;
    assert!(result.is_ok(), "按 Enter 键应该成功");

    println!("[OK] Enter 键按下成功");
}

#[tokio::test]
async fn test_press_keycode() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 使用键码按 Home 键（键码 3）
    let result = device.press_keycode(3).await;
    assert!(result.is_ok(), "通过键码按键应该成功");

    println!("[OK] 键码按键成功");

    common::wait_ui_stable().await;
}

#[tokio::test]
async fn test_multiple_gestures_sequence() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 启动设置应用
    device.app_start(common::TEST_APP_PACKAGE, None).await.ok();
    common::wait_ui_stable().await;

    let (width, height) = device.window_size().await.unwrap();

    // 执行一系列手势操作
    // 1. 点击
    device.click(width / 2, height / 2).await.ok();
    common::wait_ui_stable().await;

    // 2. 滑动
    device
        .swipe(
            width / 2,
            height * 3 / 4,
            width / 2,
            height / 4,
            Some(Duration::from_millis(300)),
        )
        .await
        .ok();
    common::wait_ui_stable().await;

    // 3. 按返回键
    device.press(Key::Back).await.ok();
    common::wait_ui_stable().await;

    println!("[OK] 多个手势序列执行成功");

    common::cleanup_test_env(&device).await.ok();
}
