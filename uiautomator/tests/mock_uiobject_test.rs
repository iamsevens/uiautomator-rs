//! UiObject Mock 测试
//!
//! 使用 mockito 创建 mock HTTP 服务器，测试 UiObject 的核心方法

use mockito::Server;
use std::time::Duration;
use uiautomator::{Device, Error, Selector};

/// 创建一个使用 mock 服务器的 Device
fn create_mock_device(server: &Server) -> Device {
    let rpc_url = format!("{}/jsonrpc/0", server.url());
    Device::connect_with_rpc_url(Some("mock-device"), &rpc_url).expect("创建 mock Device 失败")
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
