//! init 命令集成测试
//!
//! 测试完整的初始化流程、--force 选项和幂等性

use anyhow::Result;
use uiautomator::adb::AdbClient;
mod common;

async fn has_connected_devices() -> bool {
    let adb_client = match AdbClient::new().await {
        Ok(client) => client,
        Err(err) => {
            println!("跳过测试：无法连接 ADB 服务 ({err})");
            return true;
        }
    };

    match adb_client.devices().await {
        Ok(devices) if !devices.is_empty() => {
            println!("跳过测试：检测到设备连接 ({})", devices.join(", "));
            true
        }
        Ok(_) => false,
        Err(err) => {
            println!("跳过测试：无法获取设备列表 ({err})");
            true
        }
    }
}

/// 测试：完整的初始化流程
///
/// 验证 init 命令能够成功完成设备初始化：
/// 1. 检测设备连接
/// 2. 安装 ATX-Agent
/// 3. 启动服务
/// 4. 验证服务运行状态
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_full_init_workflow() -> Result<()> {
    // 执行 init 命令（不带 --force）
    let result = common::execute_init(false).await;

    // 验证命令执行成功
    assert!(result.is_ok(), "init 命令应该成功执行");

    // 验证安装结果：创建 Installer 并检查状态
    let installer = common::new_installer().await?;
    let status = installer.status().await?;

    // 验证服务正在运行
    assert!(status.running, "ATX-Agent 服务应该正在运行");

    // 验证版本信息存在
    assert!(status.version.is_some(), "应该能够获取版本信息");

    Ok(())
}

/// 测试：--force 选项强制重新安装
///
/// 验证 --force 选项能够强制重新安装已安装的 ATX-Agent
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_init_with_force_option() -> Result<()> {
    // 先执行一次正常安装
    let result1 = common::execute_init(false).await;
    assert!(result1.is_ok(), "第一次 init 应该成功");

    // 使用 --force 强制重新安装
    let result2 = common::execute_init(true).await;
    assert!(result2.is_ok(), "带 --force 的 init 应该成功");

    // 验证服务仍然正常运行
    let installer = common::new_installer().await?;
    let status = installer.status().await?;

    assert!(status.running, "强制重新安装后服务应该正在运行");
    assert!(status.version.is_some(), "应该能够获取版本信息");

    Ok(())
}

/// 测试：幂等性 - 重复初始化不改变状态
///
/// 验证在不使用 --force 的情况下，重复执行 init 命令：
/// 1. 应该检测到已安装状态
/// 2. 不应该重新安装
/// 3. 服务状态保持一致
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_init_idempotent() -> Result<()> {
    // 第一次初始化
    let result1 = common::execute_init(false).await;
    assert!(result1.is_ok(), "第一次 init 应该成功");

    // 获取第一次初始化后的状态
    let installer = common::new_installer().await?;
    let status1 = installer.status().await?;

    // 第二次初始化（不带 --force）
    let result2 = common::execute_init(false).await;
    assert!(result2.is_ok(), "第二次 init 应该成功");

    // 获取第二次初始化后的状态
    let status2 = installer.status().await?;

    // 验证状态一致
    assert_eq!(status1.running, status2.running, "运行状态应该保持一致");
    assert_eq!(status1.version, status2.version, "版本信息应该保持一致");

    // 验证服务正在运行
    assert!(status2.running, "服务应该正在运行");

    Ok(())
}

/// 测试：指定设备序列号初始化
///
/// 验证使用 --serial 选项指定设备序列号进行初始化
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_init_with_serial() -> Result<()> {
    // 获取第一个可用设备的序列号
    let installer = common::new_installer().await?;
    let serial = installer.device_serial().to_string();

    // 使用指定的序列号初始化
    let result = uiautomator_cli::commands::execute_init(Some(serial.clone()), false).await;
    assert!(result.is_ok(), "使用指定序列号的 init 应该成功");

    // 验证服务正在运行
    let installer = uiautomator_cli::installer::Installer::new(Some(serial)).await?;
    let status = installer.status().await?;

    assert!(status.running, "服务应该正在运行");

    Ok(())
}

/// 测试：无设备连接时的错误处理
///
/// 验证当没有设备连接时，init 命令应该返回友好的错误消息
///
/// 注意：这个测试需要在没有设备连接的环境中运行
#[tokio::test]
#[ignore] // 需要特定环境（无设备连接）
async fn test_init_no_device_error() {
    if has_connected_devices().await {
        return;
    }

    // 尝试在没有设备的情况下初始化
    let result = uiautomator_cli::commands::execute_init(None, false).await;

    // 验证返回错误
    assert!(result.is_err(), "没有设备时应该返回错误");

    // 验证错误消息包含有用的提示
    let error_msg = format!("{:#}", result.unwrap_err());
    assert!(
        error_msg.contains("未找到连接的设备"),
        "错误消息应该提示未找到设备，实际: {error_msg}"
    );
}

/// 测试：指定不存在的设备序列号
///
/// 验证当指定的设备序列号不存在时，应该返回友好的错误消息
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_init_invalid_serial_error() {
    // 使用一个不存在的设备序列号
    let result = uiautomator_cli::commands::execute_init(
        Some("invalid-device-serial-12345".to_string()),
        false,
    )
    .await;

    // 验证返回错误
    assert!(result.is_err(), "使用无效序列号应该返回错误");

    // 验证错误消息包含有用的提示
    let error_msg = format!("{:#}", result.unwrap_err());
    assert!(
        error_msg.contains("invalid-device-serial-12345"),
        "错误消息应该包含无效序列号，实际: {error_msg}"
    );
    assert!(
        error_msg.contains("未找到设备"),
        "错误消息应该提示未找到设备，实际: {error_msg}"
    );
}
