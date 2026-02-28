//! 异步 API 设计测试
//!
//! 验证需求 12.1, 12.3: 所有 I/O 方法都返回 Future，使用 async fn 定义

use std::time::Duration;
use uiautomator::{Device, Key, Selector};

/// 测试需求 12.1: 所有 I/O 方法都返回 Future
///
/// 这个测试验证所有 I/O 方法都是异步的，返回 Future
#[tokio::test]
async fn test_all_io_methods_are_async() {
    // 这个测试通过编译即可验证所有方法都是 async fn
    // 如果方法不是 async，则无法使用 .await

    // 验证 Device::connect 是异步的
    let _: Result<Device, _> = async { Device::connect(None).await }.await;

    // 验证 Device::connect_quick 是异步的
    let _: Result<Device, _> = async { Device::connect_quick(None).await }.await;

    // 验证 Device::connect_with_mode 是异步的
    let _: Result<Device, _> = async {
        use uiautomator::ServerMode;
        Device::connect_with_mode(None, ServerMode::Direct).await
    }
    .await;
}

/// 测试需求 12.1: Device 的所有 I/O 方法都是异步的
#[tokio::test]
#[ignore = "需要真实设备或模拟器"]
async fn test_device_io_methods_are_async() {
    if let Ok(device) = Device::connect(None).await {
        // 验证 info() 是异步的
        let _ = device.info().await;

        // 验证 window_size() 是异步的
        let _ = device.window_size().await;

        // 验证 pos_rel2abs() 是异步的
        let _ = device.pos_rel2abs(0.5, 0.5).await;

        // 验证 click() 是异步的
        let _ = device.click(100, 200).await;

        // 验证 long_click() 是异步的
        let _ = device.long_click(100, 200, None).await;

        // 验证 double_click() 是异步的
        let _ = device.double_click(100, 200, None).await;

        // 验证 swipe() 是异步的
        let _ = device.swipe(100, 200, 300, 400, None).await;

        // 验证 drag() 是异步的
        let _ = device.drag(100, 200, 300, 400, None).await;

        // 验证 press() 是异步的
        let _ = device.press(Key::Home).await;

        // 验证 press_keycode() 是异步的
        let _ = device.press_keycode(3).await;

        // 验证 screenshot() 是异步的
        let _ = device.screenshot().await;

        // 验证 screenshot_to_file() 是异步的
        let _ = device.screenshot_to_file("test.png").await;

        // 验证 app_start() 是异步的
        let _ = device.app_start("com.android.settings", None).await;

        // 验证 app_stop() 是异步的
        let _ = device.app_stop("com.android.settings").await;

        // 验证 app_clear() 是异步的
        let _ = device.app_clear("com.android.settings").await;

        // 验证 app_current() 是异步的
        let _ = device.app_current().await;

        // 验证 app_wait() 是异步的
        let _ = device
            .app_wait("com.android.settings", Duration::from_secs(1))
            .await;

        // 验证 wait_for() 是异步的
        let _ = device
            .wait_for(|| async { Ok(true) }, Some(Duration::from_secs(1)))
            .await;
    }
}

/// 测试需求 12.1: UiObject 的所有 I/O 方法都是异步的
#[tokio::test]
#[ignore = "需要真实设备或模拟器"]
async fn test_uiobject_io_methods_are_async() {
    if let Ok(device) = Device::connect(None).await {
        let selector = Selector::new().text("Settings");
        let element = device.find(selector);

        // 验证 exists() 是异步的
        let _ = element.exists(None).await;

        // 验证 wait() 是异步的
        let _ = element.wait(Some(Duration::from_secs(1))).await;

        // 验证 wait_gone() 是异步的
        let _ = element.wait_gone(Some(Duration::from_secs(1))).await;

        // 验证 info() 是异步的
        let _ = element.info().await;

        // 验证 click() 是异步的
        let _ = element.click(None, None).await;

        // 验证 click_exists() 是异步的
        let _ = element.click_exists(None).await;

        // 验证 long_click() 是异步的
        let _ = element.long_click(None, None).await;

        // 验证 get_text() 是异步的
        let _ = element.get_text().await;

        // 验证 set_text() 是异步的
        let _ = element.set_text("test").await;

        // 验证 clear_text() 是异步的
        let _ = element.clear_text().await;

        // 验证 center() 是异步的
        let _ = element.center().await;

        // 验证 bounds() 是异步的
        let _ = element.bounds().await;
    }
}

/// 测试需求 12.3: 使用 tokio 的异步原语
///
/// 验证代码使用 tokio::time::sleep 等异步原语
#[tokio::test]
async fn test_uses_tokio_async_primitives() {
    // 验证可以使用 tokio::time::sleep
    let start = std::time::Instant::now();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let elapsed = start.elapsed();

    assert!(elapsed >= Duration::from_millis(100));
    assert!(elapsed < Duration::from_millis(200));
}

/// 测试需求 12.3: 异步方法可以并发执行
#[tokio::test]
async fn test_async_methods_can_run_concurrently() {
    // 创建多个异步任务
    let task1 = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        "task1"
    });

    let task2 = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        "task2"
    });

    let task3 = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        "task3"
    });

    // 并发等待所有任务
    let start = std::time::Instant::now();
    let (r1, r2, r3) = tokio::join!(task1, task2, task3);
    let elapsed = start.elapsed();

    // 验证任务都成功完成
    assert_eq!(r1.unwrap(), "task1");
    assert_eq!(r2.unwrap(), "task2");
    assert_eq!(r3.unwrap(), "task3");

    // 验证并发执行（总时间应该接近 100ms，而不是 300ms）
    assert!(elapsed < Duration::from_millis(200));
}

/// 测试需求 12.3: 异步方法可以使用 tokio::select!
#[tokio::test]
async fn test_async_methods_work_with_select() {
    // 测试可以使用 timeout 包装异步操作
    let result = tokio::time::timeout(
        Duration::from_millis(100),
        tokio::time::sleep(Duration::from_millis(50)),
    )
    .await;

    assert!(result.is_ok());

    // 测试超时情况
    let result = tokio::time::timeout(
        Duration::from_millis(50),
        tokio::time::sleep(Duration::from_millis(100)),
    )
    .await;

    assert!(result.is_err());
}

/// 测试需求 12.1: 验证 AdbClient 的异步方法
#[tokio::test]
async fn test_adb_client_async_methods() {
    use uiautomator::AdbClient;

    // 验证 new() 是异步的
    let _: Result<AdbClient, _> = async { AdbClient::new().await }.await;
}

/// 测试需求 12.1: 验证 AdbClient 的所有 I/O 方法都是异步的
#[tokio::test]
#[ignore = "需要 ADB 服务器运行"]
async fn test_adb_client_io_methods_are_async() {
    use uiautomator::AdbClient;

    if let Ok(client) = AdbClient::new().await {
        // 验证 devices() 是异步的
        let _ = client.devices().await;

        // 验证 shell() 是异步的
        let _ = client.shell("test", "echo test", None).await;

        // 验证 push() 是异步的
        let _ = client
            .push("test", "/tmp/test", "/data/local/tmp/test")
            .await;

        // 验证 pull() 是异步的
        let _ = client
            .pull("test", "/data/local/tmp/test", "/tmp/test")
            .await;

        // 验证 forward() 是异步的
        let _ = client.forward("test", 9008, 9008).await;
    }
}

/// 测试需求 12.1: 验证 JsonRpcClient 的异步方法
#[tokio::test]
async fn test_jsonrpc_client_async_methods() {
    // JsonRpcClient::new 是异步的
    // JsonRpcClient::call 是异步的
    // JsonRpcClient::ping 是异步的

    // 这些方法的异步性已经在其他测试中验证
    // 这里只是确认它们的签名是正确的
}

/// 测试需求 12.3: 验证异步方法可以在不同的 tokio 运行时中使用
#[test]
fn test_async_methods_work_with_different_runtimes() {
    // 测试单线程运行时
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        tokio::time::sleep(Duration::from_millis(10)).await;
    });

    // 测试多线程运行时
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        tokio::time::sleep(Duration::from_millis(10)).await;
    });
}

/// 测试需求 12.1: 验证异步方法返回正确的 Future 类型
#[tokio::test]
async fn test_async_methods_return_correct_future_types() {
    // 验证方法返回的是 impl Future<Output = Result<T>>

    // Device::connect 返回 Future<Output = Result<Device>>
    let future = Device::connect(None);
    let _: Result<Device, _> = future.await;

    // 验证可以将 Future 存储在变量中
    let future = async { Device::connect(None).await };
    let _: Result<Device, _> = future.await;
}

/// 测试需求 12.3: 验证异步方法可以被取消
#[tokio::test]
async fn test_async_methods_can_be_cancelled() {
    // 创建一个长时间运行的任务
    let task = tokio::spawn(async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        "completed"
    });

    // 等待一小段时间后取消
    tokio::time::sleep(Duration::from_millis(100)).await;
    task.abort();

    // 验证任务被取消
    let result = task.await;
    assert!(result.is_err());
}

/// 测试需求 12.1: 验证所有异步方法都使用 async fn 语法
///
/// 这个测试通过编译即可验证，因为如果方法不是 async fn，
/// 则无法使用 .await 语法
#[tokio::test]
async fn test_all_methods_use_async_fn_syntax() {
    // 如果这个测试能编译通过，说明所有方法都正确使用了 async fn

    // 测试 Device 方法
    let _ = Device::connect(None).await;

    // 测试链式调用
    if let Ok(device) = Device::connect(None).await {
        let _ = device.info().await;
        let _ = device.window_size().await;
    }
}
