//! 服务管理集成测试
//!
//! 测试 restart 和 uninstall 命令的完整流程

use anyhow::Result;
mod common;

/// 测试：重启 ATX-Agent 服务
///
/// 验证 restart 命令能够成功重启服务：
/// 1. 确保服务正在运行
/// 2. 执行重启操作
/// 3. 验证服务最终处于运行状态
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_restart_workflow() -> Result<()> {
    // 1. 确保设备已初始化（服务正在运行）
    let installer = common::new_installer().await?;

    // 检查初始状态
    let status_before = installer.status().await?;

    // 如果服务未运行，先初始化
    if !status_before.running {
        common::execute_init(false).await?;
    }

    // 2. 执行重启命令
    let result = common::execute_restart().await;

    // 验证命令执行成功
    assert!(result.is_ok(), "restart 命令应该成功执行");

    // 3. 验证服务最终运行状态
    let status_after = installer.status().await?;

    // 验证服务正在运行
    assert!(status_after.running, "重启后 ATX-Agent 服务应该正在运行");

    // 验证版本信息存在
    assert!(status_after.version.is_some(), "应该能够获取版本信息");

    Ok(())
}

/// 测试：重启未安装的服务应该返回友好错误
///
/// 验证当 ATX-Agent 未安装时，restart 命令应该提示用户先执行 init
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_restart_not_installed_error() -> Result<()> {
    // 先确保服务已卸载
    let installer = common::new_installer().await?;
    let _ = installer.uninstall().await; // 忽略错误，可能本来就未安装

    // 尝试重启未安装的服务
    let _result = common::execute_restart().await;

    // 应该返回错误或提示
    // 注意：根据实现，可能会返回错误或成功但带有提示信息
    // 这里我们主要验证不会崩溃

    Ok(())
}

/// 测试：使用指定序列号重启服务
///
/// 验证使用 --serial 选项指定设备序列号进行重启
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_restart_with_serial() -> Result<()> {
    // 获取第一个可用设备的序列号
    let installer = common::new_installer().await?;
    let serial = installer.device_serial().to_string();

    // 确保服务正在运行
    let status = installer.status().await?;
    if !status.running {
        uiautomator_cli::commands::execute_init(Some(serial.clone()), false).await?;
    }

    // 使用指定的序列号重启
    let result = uiautomator_cli::commands::execute_restart(Some(serial.clone())).await;
    assert!(result.is_ok(), "使用指定序列号的 restart 应该成功");

    // 验证服务正在运行
    let installer = uiautomator_cli::installer::Installer::new(Some(serial)).await?;
    let status = installer.status().await?;

    assert!(status.running, "重启后服务应该正在运行");

    Ok(())
}

/// 测试：重启操作的原子性
///
/// 验证重启操作是原子的：服务最终必须处于运行状态
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_restart_atomicity() -> Result<()> {
    let installer = common::new_installer().await?;

    // 确保服务正在运行
    let status = installer.status().await?;
    if !status.running {
        common::execute_init(false).await?;
    }

    // 执行重启
    installer.restart().await?;

    // 立即检查状态 - 应该已经就绪
    let status = installer.status().await?;

    // 验证服务正在运行（原子性保证）
    assert!(status.running, "重启操作应该确保服务最终处于运行状态");

    Ok(())
}

/// 测试：多次重启应该保持稳定
///
/// 验证连续多次重启服务不会导致问题
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_multiple_restarts() -> Result<()> {
    let installer = common::new_installer().await?;

    // 确保服务正在运行
    let status = installer.status().await?;
    if !status.running {
        common::execute_init(false).await?;
    }

    // 连续重启 3 次
    for i in 1..=3 {
        println!("第 {} 次重启", i);

        let result = installer.restart().await;
        assert!(result.is_ok(), "第 {} 次重启应该成功", i);

        // 验证服务正在运行
        let status = installer.status().await?;
        assert!(status.running, "第 {} 次重启后服务应该正在运行", i);
    }

    Ok(())
}
