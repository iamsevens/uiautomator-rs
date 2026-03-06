//! ATX-Agent 安装模式集成测试
//!
//! 这些测试需要真实的 Android 设备或模拟器。
//!
//! 运行测试：
//! ```bash
//! # 1. 确保已下载资源文件
//! cd assets && ./download_atx_agent.sh
//!
//! # 2. 连接 Android 设备
//! adb devices
//!
//! # 3. 运行测试
//! cargo test --test atx_agent_install_test --features atx-agent-install -- --test-threads=1
//! ```

#![cfg(feature = "atx-agent-install")]

use uiautomator::Device;

mod common;

/// 测试 ATX-Agent 安装流程
#[tokio::test]
#[ignore] // 需要真实设备，默认跳过
async fn test_atx_agent_installation() {
    common::init_test_env();

    // 连接到设备（Direct 模式）
    let device = Device::connect_quick(None).await.expect("无法连接到设备");

    // 检查是否已安装
    let installed_before = device
        .check_atx_agent_installed()
        .await
        .expect("检查安装状态失败");

    println!(
        "安装前状态: {}",
        if installed_before {
            "已安装"
        } else {
            "未安装"
        }
    );

    // 安装 ATX-Agent（强制重新安装以测试完整流程）
    device
        .install_atx_agent(true)
        .await
        .expect("安装 ATX-Agent 失败");

    // 验证安装
    let installed_after = device
        .check_atx_agent_installed()
        .await
        .expect("检查安装状态失败");

    assert!(installed_after, "ATX-Agent 应该已安装");

    println!("✓ ATX-Agent 安装测试通过");
}

/// 测试 ATX-Agent 服务管理
#[tokio::test]
#[ignore] // 需要真实设备，默认跳过
async fn test_atx_agent_service_management() {
    common::init_test_env();

    // 连接到设备
    let device = Device::connect_quick(None).await.expect("无法连接到设备");

    // 确保已安装
    let installed = device
        .check_atx_agent_installed()
        .await
        .expect("检查安装状态失败");

    if !installed {
        println!("ATX-Agent 未安装，先安装...");
        device
            .install_atx_agent(false)
            .await
            .expect("安装 ATX-Agent 失败");
    }

    // 测试启动服务
    println!("测试启动服务...");
    device.start_atx_agent().await.expect("启动服务失败");

    // 等待一下确保服务启动
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // 测试停止服务
    println!("测试停止服务...");
    device.stop_atx_agent().await.expect("停止服务失败");

    // 测试重启服务
    println!("测试重启服务...");
    device.restart_atx_agent().await.expect("重启服务失败");

    println!("✓ ATX-Agent 服务管理测试通过");
}

/// 测试使用 ATX-Agent 模式连接
#[tokio::test]
#[ignore] // 需要真实设备，默认跳过
async fn test_connect_with_atx_agent_mode() {
    common::init_test_env();

    // 先确保 ATX-Agent 已安装
    let device = Device::connect_quick(None).await.expect("无法连接到设备");

    let installed = device
        .check_atx_agent_installed()
        .await
        .expect("检查安装状态失败");

    if !installed {
        println!("ATX-Agent 未安装，先安装...");
        device
            .install_atx_agent(false)
            .await
            .expect("安装 ATX-Agent 失败");
    }

    // 使用 ATX-Agent 模式连接
    println!("使用 ATX-Agent 模式连接...");
    let device = Device::connect(None)
        .await
        .expect("无法使用 ATX-Agent 模式连接");

    // 验证连接
    let info = device.info().await.expect("获取设备信息失败");

    assert!(info.display_width > 0, "屏幕宽度应该大于 0");
    assert!(info.display_height > 0, "屏幕高度应该大于 0");

    println!("✓ ATX-Agent 模式连接测试通过");
    println!("  设备信息: {}x{}", info.display_width, info.display_height);
}

/// 测试版本检查
#[tokio::test]
#[ignore] // 需要真实设备，默认跳过
async fn test_version_check() {
    common::init_test_env();

    // 连接到设备
    let device = Device::connect_quick(None).await.expect("无法连接到设备");

    // 安装 ATX-Agent
    device
        .install_atx_agent(false)
        .await
        .expect("安装 ATX-Agent 失败");

    // 再次安装（应该跳过，因为版本匹配）
    println!("测试版本检查（应该跳过安装）...");
    device.install_atx_agent(false).await.expect("版本检查失败");

    println!("✓ 版本检查测试通过");
}

/// 测试强制重新安装
#[tokio::test]
#[ignore] // 需要真实设备，默认跳过
async fn test_force_reinstall() {
    common::init_test_env();

    // 连接到设备
    let device = Device::connect_quick(None).await.expect("无法连接到设备");

    // 第一次安装
    println!("第一次安装...");
    device
        .install_atx_agent(false)
        .await
        .expect("安装 ATX-Agent 失败");

    // 强制重新安装
    println!("强制重新安装...");
    device
        .install_atx_agent(true)
        .await
        .expect("强制重新安装失败");

    // 验证安装
    let installed = device
        .check_atx_agent_installed()
        .await
        .expect("检查安装状态失败");

    assert!(installed, "ATX-Agent 应该已安装");

    println!("✓ 强制重新安装测试通过");
}
