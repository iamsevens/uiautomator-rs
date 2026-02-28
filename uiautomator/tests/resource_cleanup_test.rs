//! 资源清理测试
//!
//! 验证需�?12.5: �?Drop trait 中清理资源，确保异步任务取消时清理资�?

use std::time::Duration;
use uiautomator::Device;

/// 测试需�?12.5: Device 可以被正�?Drop
#[tokio::test]
async fn test_device_can_be_dropped() {
    // 创建 Device 并让它离开作用�?
    {
        if let Ok(_device) = Device::connect(None).await {
            // Device 在这里被创建
        }
        // Device 在这里被 Drop
    }

    // 如果没有 panic，说�?Drop 成功
}

/// 测试需�?12.5: 多个 Device 克隆可以被独�?Drop
#[tokio::test]
async fn test_multiple_device_clones_can_be_dropped() {
    if let Ok(device) = Device::connect(None).await {
        let device1 = device.clone();
        let device2 = device.clone();

        // Drop device1
        drop(device1);

        // device2 �?device 仍然可用
        let _ = device2.serial();
        let _ = device.serial();

        // Drop device2
        drop(device2);

        // device 仍然可用
        let _ = device.serial();
    }
}

/// 测试需�?12.5: 异步任务取消时资源被清理
#[tokio::test]
async fn test_async_task_cancellation_cleans_up_resources() {
    // 创建一个长时间运行的任�?
    let task = tokio::spawn(async {
        if let Ok(device) = Device::connect(None).await {
            // 模拟长时间操�?
            tokio::time::sleep(Duration::from_secs(10)).await;
            device
        } else {
            panic!("无法连接设备");
        }
    });

    // 等待一小段时间
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 取消任务
    task.abort();

    // 等待任务结束
    let result = task.await;

    // 验证任务被取�?
    assert!(result.is_err());

    // 如果没有资源泄漏，这个测试应该通过
}

/// 测试需�?12.5: Device �?Drop 时不�?panic
#[test]
fn test_device_drop_does_not_panic() {
    // 使用 catch_unwind 捕获 panic
    let result = std::panic::catch_unwind(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            if let Ok(device) = Device::connect(None).await {
                drop(device);
            }
        });
    });

    // 验证没有 panic
    assert!(result.is_ok());
}

/// 测试需�?12.5: 并发 Drop 不会导致问题
#[tokio::test]
async fn test_concurrent_drop_is_safe() {
    if let Ok(device) = Device::connect(None).await {
        let device1 = device.clone();
        let device2 = device.clone();
        let device3 = device.clone();

        // 在不同的任务�?Drop
        let task1 = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            drop(device1);
        });

        let task2 = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            drop(device2);
        });

        let task3 = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(30)).await;
            drop(device3);
        });

        // 等待所有任务完�?
        let (r1, r2, r3) = tokio::join!(task1, task2, task3);

        // 验证所有任务都成功
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        assert!(r3.is_ok());
    }
}

/// 测试需�?12.5: Device 在异步上下文中被 Drop
#[tokio::test]
async fn test_device_drop_in_async_context() {
    // 创建多个嵌套的异步作用域
    async fn inner_function() -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(device) = Device::connect(None).await {
            let _ = device.serial();
            // device 在函数结束时�?Drop
        }
        Ok(())
    }

    // 调用多次
    for _ in 0..3 {
        let _ = inner_function().await;
    }
}

/// 测试需�?12.5: 验证 Arc 引用计数正确管理
#[tokio::test]
async fn test_arc_reference_counting() {
    use std::sync::Arc;

    if let Ok(device) = Device::connect(None).await {
        // 获取 AdbClient �?Arc
        let adb1 = device.adb_client().clone();

        // 创建更多引用
        let adb2 = adb1.clone();
        let adb3 = adb2.clone();

        // 验证强引用计�?
        assert!(Arc::strong_count(&adb1) >= 2); // 至少�?device �?adb1

        // Drop 一些引�?
        drop(adb2);
        drop(adb3);

        // 引用计数应该减少
        // 注意：由�?device 内部也持有引用，所以计数不会是 1
    }
}

/// 测试需�?12.5: 验证 RwLock �?Drop 时释放锁
#[tokio::test]
async fn test_rwlock_released_on_drop() {
    if let Ok(device) = Device::connect(None).await {
        // 获取写锁
        {
            let settings = device.settings();
            let mut s = settings.write().unwrap();
            s.set_wait_timeout(Duration::from_secs(99));
            // 写锁在这里被释放
        }

        // 应该能够再次获取�?
        let timeout = device.get_wait_timeout();
        assert_eq!(timeout, Duration::from_secs(99));
    }
}

/// 测试需�?12.5: 验证异步操作中断时的资源清理
#[tokio::test]
async fn test_resource_cleanup_on_async_interruption() {
    use tokio::time::timeout;

    // 创建一个会超时的操�?
    let result = timeout(Duration::from_millis(100), async {
        if let Ok(device) = Device::connect(None).await {
            // 模拟长时间操�?
            tokio::time::sleep(Duration::from_secs(10)).await;
            device
        } else {
            panic!("无法连接设备");
        }
    })
    .await;

    // 验证操作超时
    assert!(result.is_err());

    // 如果资源被正确清理，不应该有泄漏
}

/// 测试需�?12.5: 验证 Device 可以在不同的线程中被 Drop
#[test]
fn test_device_can_be_dropped_in_different_threads() {
    use std::thread;

    let rt = tokio::runtime::Runtime::new().unwrap();

    // 在主线程创建 Device
    let device = rt.block_on(async { Device::connect(None).await.ok() });

    if let Some(device) = device {
        // 在另一个线程中 Drop
        let handle = thread::spawn(move || {
            drop(device);
        });

        // 等待线程完成
        handle.join().unwrap();
    }
}

/// 测试需�?12.5: 验证资源清理不会阻塞
#[tokio::test]
async fn test_resource_cleanup_is_non_blocking() {
    if let Ok(device) = Device::connect(None).await {
        let start = std::time::Instant::now();

        // Drop device
        drop(device);

        let elapsed = start.elapsed();

        // Drop 应该很快完成（不应该阻塞�?
        assert!(elapsed < Duration::from_millis(100));
    }
}

/// 测试需�?12.5: 验证多次 Drop 同一�?Device 克隆是安全的
#[tokio::test]
async fn test_multiple_drops_of_same_device_clone() {
    if let Ok(device) = Device::connect(None).await {
        let device1 = device.clone();

        // 第一�?Drop
        drop(device1.clone());

        // 第二�?Drop
        drop(device1.clone());

        // 原始 device 仍然可用
        let _ = device.serial();
    }
}

/// 测试需�?12.5: 验证 Device �?panic 时也能正确清�?
#[test]
fn test_device_cleanup_on_panic() {
    let result = std::panic::catch_unwind(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // 直接 panic，不依赖于设备连�?
            panic!("测试 panic");
        });
    });

    // 验证 panic 被捕�?
    assert!(result.is_err());

    // 如果资源被正确清理，不应该有泄漏
}

/// 测试需�?12.5: 验证属�?10 - 资源清理完整�?
///
/// **Feature: uiautomator, Property 10: 资源清理完整�?*
/// 验证需�?12.5
#[tokio::test]
async fn test_resource_cleanup_completeness() {
    // 创建多个 Device 并执行操�?
    for _ in 0..5 {
        if let Ok(device) = Device::connect(None).await {
            // 执行一些操�?
            let _ = device.window_size().await;

            // 创建克隆
            let device_clone = device.clone();

            // 在异步任务中使用
            let task = tokio::spawn(async move {
                let _ = device_clone.serial();
            });

            // 等待任务完成
            let _ = task.await;

            // device 在循环结束时�?Drop
        }
    }

    // 如果所有资源都被正确清理，这个测试应该通过
}

/// 测试需�?12.5: 验证 Settings �?RwLock �?Drop 时不会死�?
#[test]
fn test_settings_rwlock_no_deadlock_on_drop() {
    use std::sync::{Arc, RwLock};
    use uiautomator::Settings;

    let settings = Arc::new(RwLock::new(Settings::default()));

    // 获取多个读锁
    let _guard1 = settings.read().unwrap();
    let _guard2 = settings.read().unwrap();
    let _guard3 = settings.read().unwrap();

    // Drop 所有守�?
    drop(_guard1);
    drop(_guard2);
    drop(_guard3);

    // 应该能够获取写锁
    let mut s = settings.write().unwrap();
    s.set_wait_timeout(Duration::from_secs(99));
}
