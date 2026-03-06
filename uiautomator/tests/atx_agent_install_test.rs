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

use std::time::Duration;
use uiautomator::{AtxAgentClient, Device};

mod common;

async fn atx_client(device: &Device) -> uiautomator::Result<AtxAgentClient> {
    AtxAgentClient::new(device.serial().to_string(), device.adb_client().clone()).await
}

async fn atx_running(device: &Device) -> uiautomator::Result<bool> {
    atx_client(device).await?.check_atx_agent_status().await
}

async fn wait_atx_ready(device: &Device) -> uiautomator::Result<()> {
    atx_client(device)
        .await?
        .wait_for_atx_agent_ready(Some(Duration::from_secs(15)))
        .await
}

async fn restore_atx_state(
    device: &Device,
    installed_before: bool,
    running_before: bool,
) -> uiautomator::Result<()> {
    if installed_before {
        if running_before {
            device.start_atx_agent().await?;
            wait_atx_ready(device).await?;
        } else {
            let _ = device.stop_atx_agent().await;
        }
        return Ok(());
    }

    let _ = device.stop_atx_agent().await;
    let adb = device.adb_client();
    let serial = device.serial();
    let _ = adb
        .shell(
            serial,
            "rm -f /data/local/tmp/atx-agent",
            Some(Duration::from_secs(10)),
        )
        .await;
    let _ = adb
        .shell(
            serial,
            "pm uninstall com.github.uiautomator",
            Some(Duration::from_secs(30)),
        )
        .await;
    let _ = adb
        .shell(
            serial,
            "pm uninstall com.github.uiautomator.test",
            Some(Duration::from_secs(30)),
        )
        .await;
    Ok(())
}

struct AtxStateGuard {
    device: Device,
    installed_before: bool,
    running_before: bool,
}

impl AtxStateGuard {
    async fn capture(device: &Device) -> Self {
        let installed_before = device.check_atx_agent_installed().await.unwrap_or(false);
        let running_before = if installed_before {
            atx_running(device).await.unwrap_or(false)
        } else {
            false
        };

        Self {
            device: device.clone(),
            installed_before,
            running_before,
        }
    }
}

impl Drop for AtxStateGuard {
    fn drop(&mut self) {
        let device = self.device.clone();
        let installed_before = self.installed_before;
        let running_before = self.running_before;

        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            match runtime {
                Ok(runtime) => {
                    let _ = runtime.block_on(async {
                        restore_atx_state(&device, installed_before, running_before).await
                    });
                }
                Err(error) => {
                    eprintln!("failed to create runtime for ATX state restore: {error}");
                }
            }
        })
        .join()
        .ok();
    }
}

/// 测试 ATX-Agent 安装流程
#[tokio::test]
#[ignore] // 需要真实设备，默认跳过
async fn test_atx_agent_installation() {
    common::init_test_env();

    // 连接到设备（Direct 模式）
    let device = Device::connect_quick(None).await.expect("无法连接到设备");
    let _state_guard = AtxStateGuard::capture(&device).await;

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
    let _state_guard = AtxStateGuard::capture(&device).await;

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
    wait_atx_ready(&device).await.expect("等待服务就绪失败");

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
    let _state_guard = AtxStateGuard::capture(&device).await;

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
    let _state_guard = AtxStateGuard::capture(&device).await;

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
    let _state_guard = AtxStateGuard::capture(&device).await;

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
