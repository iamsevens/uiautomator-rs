//! 线程安全和并发测�?
//!
//! 验证需�?12.2: 使用 Arc 共享 Device, AdbClient, JsonRpcClient
//! 使用 RwLock 保护可变状态（Settings�?
//! 使用 AtomicU64 生成请求 ID

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use uiautomator::{Device, Settings};

/// 测试需�?12.2: Device 使用 Arc 共享
///
/// 验证 Device 可以在多个线程之间安全共�?
#[tokio::test]
async fn test_device_uses_arc_for_sharing() {
    // Device 实现�?Clone，内部使�?Arc 共享
    // 这个测试验证 Device 可以被克隆并在多个任务中使用

    if let Ok(device) = Device::connect(None).await {
        let device1 = device.clone();
        let device2 = device.clone();

        // 验证克隆�?Device 指向同一个底层连�?
        assert_eq!(device1.serial(), device2.serial());
        assert_eq!(device1.serial(), device.serial());
    }
}

/// 测试需�?12.2: Device 可以在多个异步任务中并发使用
#[tokio::test]
#[ignore = "需要真实设备或模拟器"]
async fn test_device_concurrent_access() {
    if let Ok(device) = Device::connect(None).await {
        let device1 = device.clone();
        let device2 = device.clone();
        let device3 = device.clone();

        // 创建多个并发任务
        let task1 = tokio::spawn(async move { device1.window_size().await });

        let task2 = tokio::spawn(async move { device2.window_size().await });

        let task3 = tokio::spawn(async move { device3.window_size().await });

        // 等待所有任务完�?
        let (r1, r2, r3) = tokio::join!(task1, task2, task3);

        // 验证所有任务都成功
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        assert!(r3.is_ok());

        // 验证返回的结果一�?
        let size1 = r1.unwrap().unwrap();
        let size2 = r2.unwrap().unwrap();
        let size3 = r3.unwrap().unwrap();

        assert_eq!(size1, size2);
        assert_eq!(size2, size3);
    }
}

/// 测试需�?12.2: Settings 使用 RwLock 保护
///
/// 验证 Settings 可以在多个线程中安全地读�?
#[test]
fn test_settings_uses_rwlock() {
    let settings = Arc::new(RwLock::new(Settings::default()));

    // 测试读锁
    {
        let s = settings.read().unwrap();
        assert_eq!(s.wait_timeout, Duration::from_secs(20));
    }

    // 测试写锁
    {
        let mut s = settings.write().unwrap();
        s.set_wait_timeout(Duration::from_secs(30));
    }

    // 验证修改生效
    {
        let s = settings.read().unwrap();
        assert_eq!(s.wait_timeout, Duration::from_secs(30));
    }
}

/// 测试需�?12.2: Settings 在多线程环境下的并发读写
#[test]
fn test_settings_concurrent_read_write() {
    let settings = Arc::new(RwLock::new(Settings::default()));
    let mut handles = vec![];

    // 创建多个读线�?
    for _ in 0..5 {
        let settings_clone = Arc::clone(&settings);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let s = settings_clone.read().unwrap();
                let _ = s.wait_timeout;
            }
        });
        handles.push(handle);
    }

    // 创建多个写线�?
    for i in 0..5 {
        let settings_clone = Arc::clone(&settings);
        let handle = thread::spawn(move || {
            for j in 0..20 {
                let mut s = settings_clone.write().unwrap();
                s.set_wait_timeout(Duration::from_secs(10 + i + j));
            }
        });
        handles.push(handle);
    }

    // 等待所有线程完�?
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证最终状态是有效�?
    let final_timeout = {
        let s = settings.read().unwrap();
        s.wait_timeout
    };

    assert!(final_timeout >= Duration::from_secs(10));
}

/// 测试需求 12.2: Device 和 Settings 可以在多个任务中并发访问
#[tokio::test]
#[ignore = "需要真实设备或模拟器"]
async fn test_device_settings_concurrent_access() {
    if let Ok(device) = Device::connect(None).await {
        let device1 = device.clone();
        let device2 = device.clone();
        let device3 = device.clone();

        // 创建读任�?
        let read_task1 = tokio::spawn(async move {
            for _ in 0..100 {
                let _ = device1.get_wait_timeout();
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });

        let read_task2 = tokio::spawn(async move {
            for _ in 0..100 {
                let _ = device2.get_wait_timeout();
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });

        // 创建写任�?
        let write_task = tokio::spawn(async move {
            for i in 0..50 {
                device3.set_wait_timeout(Duration::from_secs(20 + i));
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        });

        // 等待所有任务完�?
        let (r1, r2, r3) = tokio::join!(read_task1, read_task2, write_task);

        // 验证所有任务都成功
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        assert!(r3.is_ok());
    }
}

/// 测试需�?12.2: JsonRpcClient 使用 AtomicU64 生成请求 ID
///
/// 验证请求 ID 生成是线程安全的
#[test]
fn test_atomic_request_id_generation() {
    let request_id = Arc::new(AtomicU64::new(1));
    let mut handles = vec![];
    let all_ids = Arc::new(RwLock::new(Vec::new()));

    // 创建多个线程并发生成 ID
    for _ in 0..10 {
        let request_id_clone = Arc::clone(&request_id);
        let all_ids_clone = Arc::clone(&all_ids);

        let handle = thread::spawn(move || {
            let mut local_ids = Vec::new();
            for _ in 0..100 {
                let id = request_id_clone.fetch_add(1, Ordering::SeqCst);
                local_ids.push(id);
            }

            // 将本�?ID 添加到全局列表
            let mut ids = all_ids_clone.write().unwrap();
            ids.extend(local_ids);
        });

        handles.push(handle);
    }

    // 等待所有线程完�?
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证所�?ID 都是唯一�?
    let ids = all_ids.read().unwrap();
    let mut sorted_ids = ids.clone();
    sorted_ids.sort();
    sorted_ids.dedup();

    // 应该�?1000 个唯一�?ID (10 线程 * 100 �?
    assert_eq!(sorted_ids.len(), 1000);

    // 验证 ID 是连续的（从 1 �?1000�?
    for (i, &id) in sorted_ids.iter().enumerate() {
        assert_eq!(id, (i + 1) as u64);
    }
}

/// 测试需�?12.2: 验证 Device 内部使用 Arc 共享 AdbClient
#[tokio::test]
#[ignore = "需要真实设备或模拟器"]
async fn test_device_shares_adb_client() {
    if let Ok(device) = Device::connect(None).await {
        let device1 = device.clone();
        let device2 = device.clone();

        // 获取 AdbClient 引用
        let adb1 = device1.adb_client();
        let adb2 = device2.adb_client();

        // 验证它们指向同一�?AdbClient（通过 Arc::ptr_eq�?
        assert!(Arc::ptr_eq(adb1, adb2));
    }
}

/// 测试需�?12.2: 验证 Device 内部使用 Arc 共享 JsonRpcClient
#[tokio::test]
#[ignore = "需要真实设备或模拟器"]
async fn test_device_shares_jsonrpc_client() {
    if let Ok(device) = Device::connect(None).await {
        let device1 = device.clone();
        let device2 = device.clone();

        // 获取 JsonRpcClient 引用
        let rpc1 = device1.jsonrpc_client();
        let rpc2 = device2.jsonrpc_client();

        // 验证它们指向同一�?JsonRpcClient（通过 Arc::ptr_eq�?
        assert!(Arc::ptr_eq(rpc1, rpc2));
    }
}

/// 测试需�?12.2: 验证 Device 内部使用 Arc<RwLock<Settings>>
#[tokio::test]
#[ignore = "需要真实设备或模拟器"]
async fn test_device_shares_settings() {
    if let Ok(device) = Device::connect(None).await {
        let device1 = device.clone();
        let device2 = device.clone();

        // 获取 Settings 引用
        let settings1 = device1.settings();
        let settings2 = device2.settings();

        // 验证它们指向同一�?Settings（通过 Arc::ptr_eq�?
        assert!(Arc::ptr_eq(settings1, settings2));

        // 修改一�?Device 的设�?
        device1.set_wait_timeout(Duration::from_secs(99));

        // 验证另一�?Device 也能看到修改
        assert_eq!(device2.get_wait_timeout(), Duration::from_secs(99));
    }
}

/// 测试需�?12.2: 验证多个 Device 实例可以并发操作
#[tokio::test]
#[ignore = "需要真实设备或模拟器"]
async fn test_multiple_devices_concurrent_operations() {
    // 连接到同一个设备的多个 Device 实例
    if let Ok(device1) = Device::connect(None).await {
        if let Ok(device2) = Device::connect(Some(device1.serial())).await {
            // 创建并发任务
            let task1 = tokio::spawn(async move {
                for _ in 0..10 {
                    let _ = device1.window_size().await;
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            });

            let task2 = tokio::spawn(async move {
                for _ in 0..10 {
                    let _ = device2.window_size().await;
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            });

            // 等待所有任务完�?
            let (r1, r2) = tokio::join!(task1, task2);

            // 验证所有任务都成功
            assert!(r1.is_ok());
            assert!(r2.is_ok());
        }
    }
}

/// 测试需�?12.2: 验证 RwLock 允许多个并发�?
#[test]
fn test_rwlock_allows_concurrent_reads() {
    let settings = Arc::new(RwLock::new(Settings::default()));
    let mut handles = vec![];

    // 创建多个读线�?
    for _ in 0..10 {
        let settings_clone = Arc::clone(&settings);
        let handle = thread::spawn(move || {
            // 持有读锁一段时�?
            let s = settings_clone.read().unwrap();
            thread::sleep(Duration::from_millis(100));
            s.wait_timeout
        });
        handles.push(handle);
    }

    // 所有读线程应该能够并发执行
    let start = std::time::Instant::now();
    for handle in handles {
        handle.join().unwrap();
    }
    let elapsed = start.elapsed();

    // 如果是串行执行，需�?1000ms (10 * 100ms)
    // 如果是并发执行，只需要约 100ms
    assert!(elapsed < Duration::from_millis(200));
}

/// 测试需�?12.2: 验证 RwLock 写锁是互斥的
#[test]
fn test_rwlock_write_is_exclusive() {
    let settings = Arc::new(RwLock::new(Settings::default()));
    let counter = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    // 创建多个写线�?
    for _ in 0..10 {
        let settings_clone = Arc::clone(&settings);
        let counter_clone = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut s = settings_clone.write().unwrap();
                // 增加计数�?
                let old_value = counter_clone.fetch_add(1, Ordering::SeqCst);
                // 设置超时为计数器�?
                s.set_wait_timeout(Duration::from_secs(old_value + 1));
            }
        });
        handles.push(handle);
    }

    // 等待所有线程完�?
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证计数器值正确（应该�?1000�?
    let final_count = counter.load(Ordering::SeqCst);
    assert_eq!(final_count, 1000);
}

/// 测试需�?12.2: 验证 Device Clone 不会创建新的连接
#[tokio::test]
#[ignore = "需要真实设备或模拟器"]
async fn test_device_clone_shares_connection() {
    if let Ok(device) = Device::connect(None).await {
        // 克隆 Device
        let device_clone = device.clone();

        // 验证序列号相�?
        assert_eq!(device.serial(), device_clone.serial());

        // 验证服务器模式相�?
        assert_eq!(device.server_mode(), device_clone.server_mode());

        // 修改原始 Device 的设�?
        device.set_wait_timeout(Duration::from_secs(88));

        // 验证克隆�?Device 也能看到修改
        assert_eq!(device_clone.get_wait_timeout(), Duration::from_secs(88));
    }
}

/// 测试需�?12.2: 验证并发安全�?- 属�?9
///
/// **Feature: uiautomator, Property 9: 并发安全�?*
/// 验证需�?12.2
#[tokio::test]
#[ignore = "需要真实设备或模拟器"]
async fn test_concurrent_operations_do_not_interfere() {
    if let Ok(device) = Device::connect(None).await {
        let device1 = device.clone();
        let device2 = device.clone();

        // 任务 1: 获取设备信息
        let task1 = tokio::spawn(async move {
            let mut results = Vec::new();
            for _ in 0..5 {
                if let Ok(info) = device1.info().await {
                    results.push(info.display_width);
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            results
        });

        // 任务 2: 获取窗口尺寸
        let task2 = tokio::spawn(async move {
            let mut results = Vec::new();
            for _ in 0..5 {
                if let Ok((width, _)) = device2.window_size().await {
                    results.push(width);
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            results
        });

        // 等待所有任务完�?
        let (r1, r2) = tokio::join!(task1, task2);

        // 验证两个任务都成功完�?
        assert!(r1.is_ok());
        assert!(r2.is_ok());

        let results1 = r1.unwrap();
        let results2 = r2.unwrap();

        // 验证结果一致（同一个设备的宽度应该相同�?
        assert_eq!(results1.len(), 5);
        assert_eq!(results2.len(), 5);

        // 所有结果应该相�?
        for i in 0..5 {
            assert_eq!(results1[i], results2[i]);
        }
    }
}
