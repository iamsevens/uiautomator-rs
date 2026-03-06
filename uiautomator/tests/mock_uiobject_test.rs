//! UiObject Mock 测试
//!
//! 使用 mockito 创建 mock HTTP 服务器，测试 UiObject 的核心方法

use mockito::{Matcher, Server};
use serde_json::json;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uiautomator::{Coord, Device, Error, Selector};

fn configure_fast_test_settings(device: &Device) {
    let mut settings = device.settings().write().expect("获取 Settings 写锁失败");
    settings.set_wait_timeout(Duration::from_millis(200));
    settings.set_polling_interval(Duration::from_millis(10));
    settings.set_max_retry(1);
    settings.retry_base_delay = Duration::from_millis(1);
}

fn element_info_payload(text: &str) -> serde_json::Value {
    json!({
        "text": text,
        "className": "android.widget.TextView",
        "bounds": {"left": 0, "top": 0, "right": 100, "bottom": 50},
        "visibleBounds": {"left": 0, "top": 0, "right": 100, "bottom": 50},
        "checkable": false,
        "checked": false,
        "clickable": true,
        "enabled": true,
        "focusable": false,
        "focused": false,
        "longClickable": false,
        "scrollable": false,
        "selected": false,
        "childCount": 0,
        "packageName": "com.example.app",
        "contentDescription": "",
        "resourceName": "com.example:id/item"
    })
}

fn device_info_payload(width: u32, height: u32, package: &str) -> serde_json::Value {
    json!({
        "displayWidth": width,
        "displayHeight": height,
        "displayRotation": 0,
        "currentPackageName": package,
        "sdkInt": 34,
        "screenOn": true,
        "naturalOrientation": true
    })
}

fn jsonrpc_success(result: serde_json::Value) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": result
    })
    .to_string()
}

fn jsonrpc_error(code: i32, message: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": code,
            "message": message
        }
    })
    .to_string()
}

/// 创建一个使用 mock 服务器的 Device
fn create_mock_device(server: &Server) -> Device {
    let rpc_url = format!("{}/jsonrpc/0", server.url());
    create_mock_device_from_url(&rpc_url)
}

fn create_mock_device_from_url(rpc_url: &str) -> Device {
    let device =
        Device::connect_with_rpc_url(Some("mock-device"), rpc_url).expect("创建 mock Device 失败");
    configure_fast_test_settings(&device);
    device
}

#[tokio::test]
async fn test_exists_returns_true_when_element_found() {
    let mut server = Server::new_async().await;

    // 创建 mock 服务器，返回元素信息
    let _m = server
        .mock("POST", "/jsonrpc/0")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "text": "Settings",
                "className": "android.widget.TextView",
                "bounds": {"left": 0, "top": 0, "right": 100, "bottom": 50},
                "visibleBounds": {"left": 0, "top": 0, "right": 100, "bottom": 50},
                "checkable": false,
                "checked": false,
                "clickable": true,
                "enabled": true,
                "focusable": false,
                "focused": false,
                "longClickable": false,
                "scrollable": false,
                "selected": false,
                "childCount": 0,
                "packageName": "com.android.settings",
                "contentDescription": "",
                "resourceName": ""
            }
        }"#,
        )
        .create_async()
        .await;

    let device = create_mock_device(&server);
    let element = device.find(Selector::new().text("Settings"));

    let exists = element.exists(Some(Duration::from_secs(5))).await.unwrap();
    assert!(exists);
}

#[tokio::test]
async fn test_exists_returns_false_when_element_not_found() {
    let mut server = Server::new_async().await;

    // 创建 mock 服务器，返回元素未找到错误
    let _m = server
        .mock("POST", "/jsonrpc/0")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32001,
                "message": "UiObjectNotFoundException: no such object"
            }
        }"#,
        )
        .create_async()
        .await;

    let device = create_mock_device(&server);
    let element = device.find(Selector::new().text("NonExistent"));

    let exists = element.exists(Some(Duration::from_secs(1))).await.unwrap();
    assert!(!exists);
}

#[tokio::test]
async fn test_wait_succeeds_when_element_appears() {
    let mut server = Server::new_async().await;

    // 创建 mock 服务器，返回元素信息
    let _m = server
        .mock("POST", "/jsonrpc/0")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "text": "Loading",
                "className": "android.widget.TextView",
                "bounds": {"left": 0, "top": 0, "right": 100, "bottom": 50},
                "visibleBounds": {"left": 0, "top": 0, "right": 100, "bottom": 50},
                "checkable": false,
                "checked": false,
                "clickable": false,
                "enabled": true,
                "focusable": false,
                "focused": false,
                "longClickable": false,
                "scrollable": false,
                "selected": false,
                "childCount": 0,
                "packageName": "com.example.app",
                "contentDescription": "",
                "resourceName": ""
            }
        }"#,
        )
        .create_async()
        .await;

    let device = create_mock_device(&server);
    let element = device.find(Selector::new().text("Loading"));

    let result = element.wait(Some(Duration::from_secs(5))).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_wait_timeout_returns_error() {
    let mut server = Server::new_async().await;

    // 创建 mock 服务器，始终返回元素未找到
    let _m = server
        .mock("POST", "/jsonrpc/0")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32001,
                "message": "UiObjectNotFoundException: no such object"
            }
        }"#,
        )
        .expect_at_least(1)
        .create_async()
        .await;

    let device = create_mock_device(&server);
    let element = device.find(Selector::new().text("NeverAppears"));

    let result = element.wait(Some(Duration::from_secs(2))).await;
    assert!(result.is_err());

    match result {
        Err(Error::ElementTimeout { selector, timeout }) => {
            assert!(selector.contains("NeverAppears"));
            assert_eq!(timeout, Duration::from_secs(2));
        }
        _ => panic!("Expected ElementTimeout error"),
    }
}

#[tokio::test]
async fn test_wait_gone_succeeds_when_element_disappears() {
    let mut server = Server::new_async().await;

    // 创建 mock 服务器，返回元素未找到
    let _m = server
        .mock("POST", "/jsonrpc/0")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32001,
                "message": "UiObjectNotFoundException: no such object"
            }
        }"#,
        )
        .create_async()
        .await;

    let device = create_mock_device(&server);
    let element = device.find(Selector::new().text("Loading"));

    let result = element.wait_gone(Some(Duration::from_secs(5))).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_wait_gone_timeout_returns_error() {
    let mut server = Server::new_async().await;

    // 创建 mock 服务器，始终返回元素存在
    let _m = server
        .mock("POST", "/jsonrpc/0")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "text": "Persistent",
                "className": "android.widget.TextView",
                "bounds": {"left": 0, "top": 0, "right": 100, "bottom": 50},
                "visibleBounds": {"left": 0, "top": 0, "right": 100, "bottom": 50},
                "checkable": false,
                "checked": false,
                "clickable": false,
                "enabled": true,
                "focusable": false,
                "focused": false,
                "longClickable": false,
                "scrollable": false,
                "selected": false,
                "childCount": 0,
                "packageName": "com.example.app",
                "contentDescription": "",
                "resourceName": ""
            }
        }"#,
        )
        .expect_at_least(1)
        .create_async()
        .await;

    let device = create_mock_device(&server);
    let element = device.find(Selector::new().text("Persistent"));

    let result = element.wait_gone(Some(Duration::from_secs(2))).await;
    assert!(result.is_err());

    match result {
        Err(Error::ElementTimeout { selector, timeout }) => {
            assert!(selector.contains("Persistent"));
            assert_eq!(timeout, Duration::from_secs(2));
        }
        _ => panic!("Expected ElementTimeout error"),
    }
}

#[tokio::test]
async fn test_info_returns_element_info_mock() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(r#""method":"objInfo""#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_success(element_info_payload("Profile")))
        .create_async()
        .await;

    let device = create_mock_device(&server);
    let element = device.find(Selector::new().text("Profile"));

    let info = element.info().await.expect("info() 应成功");
    assert_eq!(info.text, "Profile");
    assert_eq!(info.class_name, "android.widget.TextView");
}

#[tokio::test]
async fn test_device_info_mock() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(r#""method":"deviceInfo""#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_success(json!({
            "displayWidth": 1080,
            "displayHeight": 2400,
            "displayRotation": 1,
            "currentPackageName": "com.example.settings",
            "sdkInt": 34,
            "screenOn": true,
            "naturalOrientation": true
        })))
        .create_async()
        .await;

    let device = create_mock_device(&server);
    let info = device.info().await.expect("device.info() 应成功");

    assert_eq!(info.display_width, 1080);
    assert_eq!(info.display_height, 2400);
    assert_eq!(info.display_rotation, 90);
    assert_eq!(info.current_package_name, "com.example.settings");
}

#[tokio::test]
async fn test_info_returns_element_not_found_mock() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(r#""method":"objInfo""#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_error(
            -32001,
            "UiObjectNotFoundException: no such object",
        ))
        .create_async()
        .await;

    let device = create_mock_device(&server);
    let element = device.find(Selector::new().text("Missing"));

    let result = element.info().await;
    match result {
        Err(Error::ElementNotFound { selector }) => assert!(selector.contains("Missing")),
        other => panic!("Expected ElementNotFound error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_click_mock() {
    let mut server = Server::new_async().await;

    let _obj_info = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(r#""method":"objInfo""#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_success(element_info_payload("Submit")))
        .expect(2)
        .create_async()
        .await;

    let _click = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(r#""method":"click".*\[50,25\]"#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_success(json!(true)))
        .create_async()
        .await;

    let device = create_mock_device(&server);
    let element = device.find(Selector::new().text("Submit"));

    element.click(None, None).await.expect("click() 应成功");
}

#[tokio::test]
async fn test_click_coord_mock() {
    let mut server = Server::new_async().await;

    let _device_info = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(r#""method":"deviceInfo""#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_success(json!({
            "displayWidth": 1000,
            "displayHeight": 2000,
            "displayRotation": 0,
            "currentPackageName": "com.example.settings",
            "sdkInt": 34,
            "screenOn": true,
            "naturalOrientation": true
        })))
        .create_async()
        .await;

    let _click = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(
            r#""method":"click".*\[500,400\]"#.to_string(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_success(json!(true)))
        .create_async()
        .await;

    let device = create_mock_device(&server);
    device
        .click_coord(Coord::percent(0.5), Coord::pixel(400))
        .await
        .expect("click_coord() 应成功");
}

#[tokio::test]
async fn test_swipe_coord_mock() {
    let mut server = Server::new_async().await;

    let _device_info = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(r#""method":"deviceInfo""#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_success(json!({
            "displayWidth": 1000,
            "displayHeight": 2000,
            "displayRotation": 0,
            "currentPackageName": "com.example.settings",
            "sdkInt": 34,
            "screenOn": true,
            "naturalOrientation": true
        })))
        .create_async()
        .await;

    let _swipe = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(
            r#""method":"swipe".*\[100,400,800,1200,"#.to_string(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_success(json!(true)))
        .create_async()
        .await;

    let device = create_mock_device(&server);
    device
        .swipe_coord(
            Coord::percent(0.1),
            Coord::percent(0.2),
            Coord::percent(0.8),
            Coord::percent(0.6),
            Some(Duration::from_millis(300)),
        )
        .await
        .expect("swipe_coord() 应成功");
}

#[tokio::test]
async fn test_set_text_mock() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(
            r#""method":"setText".*"hello world""#.to_string(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_success(json!(true)))
        .create_async()
        .await;

    let device = create_mock_device(&server);
    let element = device.find(Selector::new().resource_id("com.example:id/input"));

    element
        .set_text("hello world")
        .await
        .expect("set_text() 应成功");
}

#[tokio::test]
async fn test_invalid_response_mock() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(r#""method":"objInfo""#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("this is not valid json")
        .create_async()
        .await;

    let device = create_mock_device(&server);
    let element = device.find(Selector::new().text("Broken"));

    let result = element.info().await;
    assert!(
        matches!(result, Err(Error::Serialization(_))),
        "expected serialization error, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_server_error_mock() {
    let mut server = Server::new_async().await;

    let _m = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(
            r#""method":"setText".*"server-error""#.to_string(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_error(-32010, "server busy"))
        .create_async()
        .await;

    let device = create_mock_device(&server);
    let element = device.find(Selector::new().resource_id("com.example:id/input"));

    let result = element.set_text("server-error").await;
    match result {
        Err(Error::JsonRpc(message)) => {
            assert!(message.contains("-32010") || message.contains("错误码 -32010"));
            assert!(message.contains("server busy"));
        }
        other => panic!("Expected JsonRpc error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_network_error_mock() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("绑定临时端口失败");
    let port = listener.local_addr().expect("读取临时端口失败").port();
    drop(listener);
    let rpc_url = format!("http://127.0.0.1:{port}/jsonrpc/0");

    let device = create_mock_device_from_url(&rpc_url);
    let element = device.find(Selector::new().text("Offline"));

    let result = element.info().await;
    assert!(
        matches!(result, Err(Error::Http(_)) | Err(Error::HttpTimeout)),
        "expected http-layer error, got {:?}",
        result
    );
}

#[tokio::test(start_paused = true)]
async fn test_timeout_mock() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("绑定超时测试端口失败");
    let addr = listener.local_addr().expect("读取超时测试端口失败");

    let server_task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("接受连接失败");
        let mut request_buf = vec![0_u8; 4096];
        let _ = stream.read(&mut request_buf).await;

        tokio::time::sleep(Duration::from_secs(3600)).await;
        let _ = stream.shutdown().await;
    });

    let device = create_mock_device_from_url(&format!("http://{addr}/jsonrpc/0"));
    let element = device.find(Selector::new().text("Slow"));

    let request = tokio::spawn(async move { element.info().await });
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(61)).await;

    let result = request.await.expect("等待 timeout 任务失败");
    match result {
        Err(Error::HttpTimeout) => {}
        Err(Error::Http(err)) if err.is_timeout() => {}
        other => panic!("expected timeout-class error, got {:?}", other),
    }

    server_task.abort();
}

#[tokio::test]
async fn test_retry_mechanism_mock() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("绑定重试测试端口失败");
    let addr = listener.local_addr().expect("读取重试测试端口失败");
    let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let attempt_count_clone = attempt_count.clone();

    let server_task = tokio::spawn(async move {
        let (mut first, _) = listener.accept().await.expect("第一次接受连接失败");
        attempt_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut first_buf = vec![0_u8; 4096];
        let _ = first.read(&mut first_buf).await;
        drop(first);

        let (mut second, _) = listener.accept().await.expect("第二次接受连接失败");
        attempt_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut second_buf = vec![0_u8; 4096];
        let _ = second.read(&mut second_buf).await;

        let body = jsonrpc_success(element_info_payload("Retry"));
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        second
            .write_all(response.as_bytes())
            .await
            .expect("写入重试成功响应失败");
        let _ = second.shutdown().await;
    });

    let device = create_mock_device_from_url(&format!("http://{addr}/jsonrpc/0"));
    {
        let mut settings = device
            .settings()
            .write()
            .expect("获取 retry Settings 写锁失败");
        settings.set_max_retry(2);
        settings.retry_base_delay = Duration::from_millis(1);
    }

    let element = device.find(Selector::new().text("Retry"));
    let info = element.info().await.expect("retry 后 info() 应成功");
    assert_eq!(info.text, "Retry");
    assert_eq!(
        attempt_count.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "expected exactly two attempts"
    );

    server_task.await.expect("等待 retry mock 服务器失败");
}

#[tokio::test]
async fn test_device_info_cache_uses_cache() {
    let mut server = Server::new_async().await;

    let _device_info = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(r#""method":"deviceInfo""#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_success(device_info_payload(
            1080,
            2400,
            "com.example.settings",
        )))
        .expect(1)
        .create_async()
        .await;

    let device = create_mock_device(&server);
    device.set_cache_ttl(Duration::from_secs(5));

    let first = device.info().await.expect("第一次 info() 应成功");
    let second = device.info().await.expect("第二次 info() 应成功");

    assert_eq!(first.display_width, 1080);
    assert_eq!(second.display_width, 1080);
}

#[tokio::test]
async fn test_device_info_cache_expires() {
    let mut server = Server::new_async().await;

    let _device_info = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(r#""method":"deviceInfo""#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_success(device_info_payload(
            1080,
            2400,
            "com.example.settings",
        )))
        .expect(2)
        .create_async()
        .await;

    let device = create_mock_device(&server);
    device.set_cache_ttl(Duration::from_millis(10));

    device.info().await.expect("第一次 info() 应成功");
    tokio::time::sleep(Duration::from_millis(30)).await;
    device.info().await.expect("缓存过期后的 info() 应成功");
}

#[tokio::test]
async fn test_clear_cache_forces_refetch() {
    let mut server = Server::new_async().await;

    let _device_info = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(r#""method":"deviceInfo""#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_success(device_info_payload(
            1080,
            2400,
            "com.example.settings",
        )))
        .expect(2)
        .create_async()
        .await;

    let device = create_mock_device(&server);
    device.set_cache_ttl(Duration::from_secs(5));

    device.info().await.expect("第一次 info() 应成功");
    device.clear_cache();
    device.info().await.expect("clear_cache() 后 info() 应成功");
}

#[tokio::test]
async fn test_disable_cache_turns_off_cache() {
    let mut server = Server::new_async().await;

    let _device_info = server
        .mock("POST", "/jsonrpc/0")
        .match_body(Matcher::Regex(r#""method":"deviceInfo""#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(jsonrpc_success(device_info_payload(
            1080,
            2400,
            "com.example.settings",
        )))
        .expect(2)
        .create_async()
        .await;

    let device = create_mock_device(&server);
    device.set_cache_ttl(Duration::from_secs(5));

    device.info().await.expect("第一次 info() 应成功");
    device.disable_cache();
    device
        .info()
        .await
        .expect("disable_cache() 后 info() 应成功");
}
