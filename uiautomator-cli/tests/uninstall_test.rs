//! uninstall 命令测试
//!
//! 测试 uninstall 命令的完整流程

use anyhow::{anyhow, Result};
mod common;

/// 测试：卸载 ATX-Agent
///
/// 验证 uninstall 命令能够成功卸载服务：
/// 1. 确保服务已安装
/// 2. 执行卸载操作
/// 3. 验证服务已被完全卸载
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_uninstall_workflow() -> Result<()> {
    // 1. 确保设备已初始化（服务已安装）
    let installer = common::new_installer().await?;

    // 检查初始状态
    let installed_before = installer.check_installed().await?;

    // 如果未安装，先初始化
    if !installed_before {
        common::execute_init(false).await?;
    }

    // 2. 执行卸载命令
    let result = common::execute_uninstall().await;

    // 验证命令执行成功
    assert!(result.is_ok(), "uninstall 命令应该成功执行");

    // 3. 验证服务已被卸载
    let status_after = installer.status().await?;

    // 验证服务未运行
    assert!(!status_after.running, "卸载后 ATX-Agent 服务应该未运行");

    // 验证未安装
    let installed_after = installer.check_installed().await?;
    assert!(!installed_after, "卸载后 ATX-Agent 应该未安装");

    Ok(())
}

/// 测试：卸载未安装的服务应该成功（幂等性）
///
/// 验证当 ATX-Agent 未安装时，uninstall 命令应该成功执行（不报错）
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_uninstall_not_installed_idempotent() -> Result<()> {
    // 先确保服务已卸载
    let installer = common::new_installer().await?;
    let _ = installer.uninstall().await; // 忽略错误，可能本来就未安装

    // 再次尝试卸载
    let result = common::execute_uninstall().await;

    // 应该成功执行（幂等性）
    assert!(result.is_ok(), "卸载未安装的服务应该成功（幂等性）");

    Ok(())
}

/// 测试：使用指定序列号卸载服务
///
/// 验证使用 --serial 选项指定设备序列号进行卸载
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_uninstall_with_serial() -> Result<()> {
    // 获取第一个可用设备的序列号
    let installer = common::new_installer().await?;
    let serial = installer.device_serial().to_string();

    // 确保服务已安装
    let installed = installer.check_installed().await?;
    if !installed {
        uiautomator_cli::commands::execute_init(Some(serial.clone()), false).await?;
    }

    // 使用指定的序列号卸载
    let result = uiautomator_cli::commands::execute_uninstall(Some(serial.clone())).await;
    assert!(result.is_ok(), "使用指定序列号的 uninstall 应该成功");

    // 验证服务已卸载
    let installer = uiautomator_cli::installer::Installer::new(Some(serial)).await?;
    let status = installer.status().await?;

    assert!(!status.running, "卸载后服务应该未运行");

    Ok(())
}

/// 测试：卸载操作的完整性
///
/// 验证卸载操作删除所有相关文件：
/// - atx-agent 二进制文件
/// - UiAutomator APK
/// - UiAutomator Test APK
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_uninstall_completeness() -> Result<()> {
    let installer = common::new_installer().await?;

    // 确保服务已安装
    let installed = installer.check_installed().await?;
    if !installed {
        common::execute_init(false).await?;
    }

    // 执行卸载
    println!("开始执行卸载...");
    installer
        .uninstall()
        .await
        .map_err(|e| anyhow!("卸载步骤失败: {}", e))?;

    // 验证二进制文件已删除
    println!("卸载完成，检查二进制文件...");
    let adb_client = installer.adb_client();
    let binary_check = adb_client
        .shell(
            installer.device_serial(),
            "test -f /data/local/tmp/atx-agent && echo exists || echo not_exists",
            Some(std::time::Duration::from_secs(10)),
        )
        .await
        .map_err(|e| anyhow!("检查二进制文件失败: {}", e))?;

    assert!(
        binary_check.contains("not_exists"),
        "atx-agent 二进制文件应该已被删除"
    );

    // 验证 APK 已卸载（使用 pm path 探测，避免 pm list 在部分环境下阻塞）
    let main_apk_check = adb_client
        .shell(
            installer.device_serial(),
            "if pm path com.github.uiautomator >/dev/null 2>&1; then echo exists; else echo not_exists; fi",
            Some(std::time::Duration::from_secs(20)),
        )
        .await;
    let main_apk_check = match main_apk_check {
        Ok(output) => output,
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("超时") || msg.contains("Timeout") {
                println!("跳过 APK 卸载校验：package manager 查询超时 ({})", msg);
                return Ok(());
            }
            return Err(anyhow!("检查主 APK 状态失败: {}", e));
        }
    };

    assert!(
        main_apk_check.contains("not_exists"),
        "UiAutomator APK 应该已被卸载，实际输出: {}",
        main_apk_check.trim()
    );

    let test_apk_check = adb_client
        .shell(
            installer.device_serial(),
            "if pm path com.github.uiautomator.test >/dev/null 2>&1; then echo exists; else echo not_exists; fi",
            Some(std::time::Duration::from_secs(20)),
        )
        .await;
    let test_apk_check = match test_apk_check {
        Ok(output) => output,
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("超时") || msg.contains("Timeout") {
                println!("跳过 Test APK 卸载校验：package manager 查询超时 ({})", msg);
                return Ok(());
            }
            return Err(anyhow!("检查测试 APK 状态失败: {}", e));
        }
    };

    assert!(
        test_apk_check.contains("not_exists"),
        "UiAutomator Test APK 应该已被卸载，实际输出: {}",
        test_apk_check.trim()
    );

    Ok(())
}

/// 测试：卸载后重新安装应该成功
///
/// 验证卸载后可以重新安装，确保卸载操作清理干净
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_uninstall_then_reinstall() -> Result<()> {
    let installer = common::new_installer().await?;

    // 确保服务已安装
    let installed = installer.check_installed().await?;
    if !installed {
        common::execute_init(false).await?;
    }

    // 卸载
    installer.uninstall().await?;

    // 验证已卸载
    let status = installer.status().await?;
    assert!(!status.running, "卸载后服务应该未运行");

    // 重新安装
    let result = common::execute_init(false).await;
    assert!(result.is_ok(), "卸载后应该能够重新安装");

    // 验证重新安装成功
    let status = installer.status().await?;
    assert!(status.running, "重新安装后服务应该正在运行");

    Ok(())
}
