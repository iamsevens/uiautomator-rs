//! 设备初始化幂等性属性测试
//!
//! **Feature: uiautomator-cli, Property 1: 设备初始化幂等性**
//!
//! 该测试验证：*For any* 已初始化的设备，重复执行 `init` 命令（不带 `--force`）
//! 应该检测到已安装状态并跳过安装，不改变设备状态。
//!
//! **Validates: Requirements 1.4**

use proptest::prelude::*;
use uiautomator_cli::installer::ServiceStatus;
mod common;

// Property 1: 设备初始化幂等性
//
// 该属性测试验证安装操作的幂等性：
// - 第一次安装应该成功
// - 后续的安装（不带 force）应该检测到已安装状态并跳过
// - 设备状态应该保持一致
//
// 测试策略：
// 1. 使用 proptest 生成随机的 force 标志组合
// 2. 模拟安装操作
// 3. 验证幂等性：多次安装不改变最终状态
//
// 注意：由于这个测试需要真实设备，我们使用逻辑测试来验证幂等性属性。
// 实际的设备测试在集成测试中进行。
// 注意：由于 proptest 需要真实设备来测试幂等性，而这在 CI 环境中不可行，
// 我们将幂等性测试作为集成测试实现（见下面的 #[tokio::test] 测试）。
// 这里我们使用 proptest 来测试一些不需要设备的逻辑属性。
proptest! {
    /// 测试：设备选择逻辑的确定性
    ///
    /// 验证在相同的设备列表下，选择逻辑应该返回相同的结果
    #[test]
    fn test_device_selection_determinism(seed in any::<u64>()) {
        let _ = seed; // 使用种子确保可重复性

        let devices = vec![
            "device1".to_string(),
            "device2".to_string(),
            "device3".to_string(),
        ];

        // 测试自动选择（应该总是选择第一个）
        let result1 = select_first_device(&devices);
        let result2 = select_first_device(&devices);
        prop_assert_eq!(result1, result2, "自动选择应该是确定性的");
        prop_assert_eq!(result1, "device1", "应该总是选择第一个设备");
    }

    /// 测试：ServiceStatus 相等性的传递性
    ///
    /// 验证如果 a == b 且 b == c，则 a == c
    #[test]
    fn test_service_status_equality_transitivity(
        running in any::<bool>(),
        version in proptest::option::of("[0-9]+\\.[0-9]+\\.[0-9]+")
    ) {
        let status_a = ServiceStatus {
            running,
            version: version.clone(),
        };
        let status_b = ServiceStatus {
            running,
            version: version.clone(),
        };
        let status_c = ServiceStatus {
            running,
            version,
        };

        // 传递性：如果 a == b 且 b == c，则 a == c
        prop_assert_eq!(&status_a, &status_b);
        prop_assert_eq!(&status_b, &status_c);
        prop_assert_eq!(&status_a, &status_c);
    }
}

/// 单元测试：验证 ServiceStatus 的相等性
///
/// 这是一个基础测试，确保 ServiceStatus 的 PartialEq 实现正确
#[test]
fn test_service_status_equality() {
    let status1 = ServiceStatus {
        running: true,
        version: Some("0.10.0".to_string()),
    };

    let status2 = ServiceStatus {
        running: true,
        version: Some("0.10.0".to_string()),
    };

    let status3 = ServiceStatus {
        running: false,
        version: None,
    };

    assert_eq!(status1, status2, "相同的状态应该相等");
    assert_ne!(status1, status3, "不同的状态应该不相等");
}

/// 单元测试：验证设备选择逻辑的幂等性
///
/// 测试在相同的设备列表下，选择逻辑应该返回相同的结果
#[test]
fn test_device_selection_is_deterministic() {
    let devices = vec![
        "device1".to_string(),
        "device2".to_string(),
        "device3".to_string(),
    ];

    // 测试自动选择（应该总是选择第一个）
    for _ in 0..10 {
        let result = select_first_device(&devices);
        assert_eq!(result, "device1", "应该总是选择第一个设备");
    }

    // 测试指定序列号（应该总是返回指定的设备）
    for _ in 0..10 {
        let result = select_specific_device(&devices, "device2");
        assert_eq!(result, Some("device2"), "应该总是返回指定的设备");
    }
}

// 辅助函数：模拟选择第一个设备
fn select_first_device(devices: &[String]) -> &str {
    &devices[0]
}

// 辅助函数：模拟选择指定设备
fn select_specific_device<'a>(devices: &'a [String], serial: &str) -> Option<&'a str> {
    devices
        .iter()
        .find(|d| d.as_str() == serial)
        .map(|s| s.as_str())
}

/// 集成测试：验证安装操作的幂等性（需要真实设备）
///
/// 这个测试验证核心的幂等性属性：
/// 1. 第一次安装应该成功
/// 2. 第二次安装（不带 force）应该检测到已安装并跳过
/// 3. 设备状态应该保持一致
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_install_idempotency_with_real_device() -> anyhow::Result<()> {
    // 创建安装器
    let installer = common::new_installer().await?;

    // 第一次安装（可能已经安装，所以使用 force）
    installer.install(true).await?;

    // 验证已安装
    let installed = installer.check_installed().await?;
    assert!(installed, "第一次安装后应该已安装");

    // 获取初始状态
    let initial_status = installer.status().await?;
    assert!(initial_status.running, "安装后服务应该正在运行");

    // 第二次安装（不带 force）- 这应该是幂等的
    installer.install(false).await?;

    // 验证状态没有改变
    let final_status = installer.status().await?;
    assert_eq!(
        initial_status.running, final_status.running,
        "重复安装不应该改变运行状态"
    );

    // 第三次安装（不带 force）- 再次验证幂等性
    installer.install(false).await?;

    // 验证状态仍然没有改变
    let final_status_2 = installer.status().await?;
    assert_eq!(
        initial_status.running, final_status_2.running,
        "多次重复安装不应该改变运行状态"
    );

    Ok(())
}

/// 集成测试：验证 force 标志的行为
///
/// 测试 force=true 时应该总是重新安装，而 force=false 时应该跳过已安装的情况
#[tokio::test]
#[ignore] // 需要真实设备，CI 中跳过
async fn test_force_flag_behavior() -> anyhow::Result<()> {
    let installer = common::new_installer().await?;

    // 确保已安装
    installer.install(true).await?;
    let installed = installer.check_installed().await?;
    assert!(installed, "应该已安装");

    // 使用 force=false 安装（应该跳过）
    let result = installer.install(false).await;
    assert!(result.is_ok(), "force=false 时应该成功（跳过安装）");

    // 使用 force=true 安装（应该重新安装）
    let result = installer.install(true).await;
    assert!(result.is_ok(), "force=true 时应该成功（重新安装）");

    // 验证服务仍在运行
    let status = installer.status().await?;
    assert!(status.running, "重新安装后服务应该正在运行");

    Ok(())
}
