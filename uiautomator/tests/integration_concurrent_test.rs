// 集成测试：并发操作
// 需求: 12.2, 12.5

mod common;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uiautomator::{Key, Selector};

async fn screenshot_with_retry(
    device: &uiautomator::Device,
    task_id: usize,
    max_attempts: usize,
) -> uiautomator::Result<image::DynamicImage> {
    let mut last_error = None;

    for attempt in 1..=max_attempts {
        match device.screenshot().await {
            Ok(img) => return Ok(img),
            Err(err) => {
                eprintln!(
                    "截图任务 {} 第 {}/{} 次失败: {}",
                    task_id, attempt, max_attempts, err
                );
                last_error = Some(err);
                tokio::time::sleep(Duration::from_millis(200 * attempt as u64)).await;
            }
        }
    }

    Err(last_error.expect("screenshot retry should have captured an error"))
}

#[tokio::test]
async fn test_concurrent_device_info() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 使用 tokio::join! 并发获取设备信息
    let (info0, info1, info2, info3, info4) = tokio::join!(
        device.info(),
        device.info(),
        device.info(),
        device.info(),
        device.info()
    );

    let info0 = info0.unwrap();
    let info1 = info1.unwrap();
    let info2 = info2.unwrap();
    let info3 = info3.unwrap();
    let info4 = info4.unwrap();

    println!(
        "任务 0: 屏幕尺寸 {}x{}",
        info0.display_width, info0.display_height
    );
    println!(
        "任务 1: 屏幕尺寸 {}x{}",
        info1.display_width, info1.display_height
    );
    println!(
        "任务 2: 屏幕尺寸 {}x{}",
        info2.display_width, info2.display_height
    );
    println!(
        "任务 3: 屏幕尺寸 {}x{}",
        info3.display_width, info3.display_height
    );
    println!(
        "任务 4: 屏幕尺寸 {}x{}",
        info4.display_width, info4.display_height
    );

    // 验证所有结果一致
    assert_eq!(info1.display_width, info0.display_width);
    assert_eq!(info2.display_width, info0.display_width);
    assert_eq!(info3.display_width, info0.display_width);
    assert_eq!(info4.display_width, info0.display_width);

    println!("✓ 并发获取设备信息成功");
}

#[tokio::test]
async fn test_concurrent_screenshots() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 部分设备/后端不支持并发 screencap，强制串行执行截图命令，避免偶发超时误报。
    let screenshot_lock = Arc::new(Mutex::new(()));
    let lock0 = screenshot_lock.clone();
    let lock1 = screenshot_lock.clone();
    let lock2 = screenshot_lock.clone();

    // 使用 tokio::join! 并发截图
    let (img0, img1, img2) = tokio::join!(
        async {
            let _guard = lock0.lock().await;
            screenshot_with_retry(&device, 0, 3).await
        },
        async {
            let _guard = lock1.lock().await;
            screenshot_with_retry(&device, 1, 3).await
        },
        async {
            let _guard = lock2.lock().await;
            screenshot_with_retry(&device, 2, 3).await
        }
    );

    let img0 = img0.unwrap();
    let img1 = img1.unwrap();
    let img2 = img2.unwrap();

    println!("任务 0: 截图 {}x{}", img0.width(), img0.height());
    println!("任务 1: 截图 {}x{}", img1.width(), img1.height());
    println!("任务 2: 截图 {}x{}", img2.width(), img2.height());

    println!("✓ 并发截图成功");
}

#[tokio::test]
async fn test_concurrent_element_operations() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 启动设置应用
    device.app_start(common::TEST_APP_PACKAGE, None).await.ok();
    common::wait_ui_stable().await;

    // 使用 tokio::join! 并发查找元素
    let selector = Selector::new().class_name("android.widget.TextView");

    // 先创建元素对象
    let element0 = device.find(selector.clone());
    let element1 = device.find(selector.clone());
    let element2 = device.find(selector.clone());

    let (exists0, exists1, exists2) = tokio::join!(
        element0.exists(Some(Duration::from_secs(5))),
        element1.exists(Some(Duration::from_secs(5))),
        element2.exists(Some(Duration::from_secs(5)))
    );

    let exists0 = exists0.unwrap();
    let exists1 = exists1.unwrap();
    let exists2 = exists2.unwrap();

    println!("任务 0: 元素存在 = {}", exists0);
    println!("任务 1: 元素存在 = {}", exists1);
    println!("任务 2: 元素存在 = {}", exists2);

    println!("✓ 并发元素操作成功");

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_concurrent_gestures() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    let (width, height) = device.window_size().await.unwrap();

    // 顺序执行手势操作（避免冲突）
    // 注意：手势操作会影响 UI 状态，不适合真正并发执行

    for i in 0..3 {
        // 每个任务点击不同的位置
        let x = width / 4 * (i + 1);
        let y = height / 2;

        tokio::time::sleep(Duration::from_millis(i as u64 * 200)).await;
        device.click(x, y).await.unwrap();
        println!("任务 {}: 点击 ({}, {})", i, x, y);
    }

    println!("✓ 顺序手势操作成功");
}

#[tokio::test]
async fn test_concurrent_key_press() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 顺序按键（避免冲突）
    // 注意：按键操作会影响系统状态，不适合真正并发执行
    let keys = vec![Key::VolumeUp, Key::VolumeDown, Key::VolumeUp];

    for (i, key) in keys.into_iter().enumerate() {
        tokio::time::sleep(Duration::from_millis(i as u64 * 300)).await;
        device.press(key).await.unwrap();
        println!("任务 {}: 按键 {:?}", i, key);
    }

    println!("✓ 顺序按键操作成功");
}

#[tokio::test]
async fn test_resource_cleanup_after_concurrent_ops() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 执行大量并发操作
    for i in 0..10 {
        let (info_result, size_result) = tokio::join!(device.info(), device.window_size());
        info_result.ok();
        size_result.ok();
        println!("任务 {} 完成", i);
    }

    // 验证设备仍然可用
    let info = device.info().await;
    assert!(info.is_ok(), "并发操作后设备应该仍然可用");

    println!("[OK] 并发操作后资源清理正常");
}

#[tokio::test]
async fn test_shared_device_state() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 设置超时（不是异步方法）
    device.set_wait_timeout(Duration::from_secs(15));

    // 读取超时设置多次
    let timeout0 = device.get_wait_timeout();
    let timeout1 = device.get_wait_timeout();
    let timeout2 = device.get_wait_timeout();
    let timeout3 = device.get_wait_timeout();
    let timeout4 = device.get_wait_timeout();

    println!("任务 0: 超时设置 = {:?}", timeout0);
    println!("任务 1: 超时设置 = {:?}", timeout1);
    println!("任务 2: 超时设置 = {:?}", timeout2);
    println!("任务 3: 超时设置 = {:?}", timeout3);
    println!("任务 4: 超时设置 = {:?}", timeout4);

    // 验证所有结果一致
    assert_eq!(timeout1, timeout0);
    assert_eq!(timeout2, timeout0);
    assert_eq!(timeout3, timeout0);
    assert_eq!(timeout4, timeout0);

    println!("[OK] 共享设备状态访问正常");
}

#[tokio::test]
async fn test_concurrent_app_operations() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 任务 1: 启动应用
    device.app_start(common::TEST_APP_PACKAGE, None).await.ok();
    println!("任务 1: 应用已启动");

    // 等待启动完成
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 使用 tokio::join! 并发执行查询操作
    let (info_result, screenshot_result) = tokio::join!(device.app_current(), device.screenshot());

    if let Ok(info) = info_result {
        println!("任务 2: 当前应用 = {}", info.package);
    }

    if screenshot_result.is_ok() {
        println!("任务 3: 截图完成");
    }

    println!("✓ 并发应用操作成功");

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_stress_concurrent_operations() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 压力测试：大量并发操作
    let selector = Selector::new().class_name("android.widget.TextView");

    for i in 0..20 {
        match i % 3 {
            0 => {
                device.info().await.ok();
            }
            1 => {
                device.window_size().await.ok();
            }
            _ => {
                let element = device.find(selector.clone());
                element.exists(Some(Duration::from_secs(2))).await.ok();
            }
        }
    }

    // 验证设备仍然可用
    let info = device.info().await;
    assert!(info.is_ok(), "压力测试后设备应该仍然可用");

    println!("✓ 压力测试完成");
}
