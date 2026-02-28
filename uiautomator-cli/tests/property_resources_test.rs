//! 资源文件完整性属性测试
//!
//! **Feature: uiautomator-cli, Property 2: 资源文件完整性**
//!
//! 该测试验证：*For any* 嵌入的资源文件，其 MD5 校验和应该与构建时计算的 MD5 值匹配，
//! 确保资源文件未被损坏。
//!
//! **Validates: Requirements 5.2**

use proptest::prelude::*;
use uiautomator_cli::resources::EmbeddedResources;

// Property 2: 资源文件 MD5 始终匹配
//
// 该属性测试验证所有嵌入的资源文件的 MD5 校验和始终与构建时计算的值匹配。
// 这确保了资源文件在编译后没有被损坏或篡改。
//
// 测试策略：
// 1. 使用 proptest 生成随机种子（模拟不同的测试场景）
// 2. 获取嵌入的资源文件
// 3. 计算每个资源文件的 MD5 校验和
// 4. 验证计算的 MD5 与构建时的 MD5 匹配
//
// 这是一个通用属性，应该在任何情况下都成立。
proptest! {
    #[test]
    fn test_atx_agent_md5_always_matches(seed in any::<u64>()) {
        // 使用种子来确保测试的可重复性（虽然资源文件是静态的）
        let _ = seed;

        let resources = EmbeddedResources::get();

        // 计算 atx-agent 的 MD5
        let computed_md5 = format!("{:x}", md5::compute(resources.atx_agent));

        // 验证 MD5 匹配
        prop_assert_eq!(
            computed_md5,
            resources.atx_agent_md5,
            "atx-agent MD5 校验和不匹配"
        );
    }

    #[test]
    fn test_app_uiautomator_apk_md5_always_matches(seed in any::<u64>()) {
        let _ = seed;

        let resources = EmbeddedResources::get();

        // 计算 app-uiautomator.apk 的 MD5
        let computed_md5 = format!("{:x}", md5::compute(resources.app_uiautomator_apk));

        // 验证 MD5 匹配
        prop_assert_eq!(
            computed_md5,
            resources.app_uiautomator_apk_md5,
            "app-uiautomator.apk MD5 校验和不匹配"
        );
    }

    #[test]
    fn test_app_uiautomator_test_apk_md5_always_matches(seed in any::<u64>()) {
        let _ = seed;

        let resources = EmbeddedResources::get();

        // 计算 app-uiautomator-test.apk 的 MD5
        let computed_md5 = format!("{:x}", md5::compute(resources.app_uiautomator_test_apk));

        // 验证 MD5 匹配
        prop_assert_eq!(
            computed_md5,
            resources.app_uiautomator_test_apk_md5,
            "app-uiautomator-test.apk MD5 校验和不匹配"
        );
    }

    #[test]
    fn test_verify_integrity_always_succeeds(seed in any::<u64>()) {
        let _ = seed;

        let resources = EmbeddedResources::get();

        // 使用内置的完整性验证方法
        let result = resources.verify_integrity();

        // 验证完整性检查总是成功
        prop_assert!(
            result.is_ok(),
            "资源文件完整性验证失败: {:?}",
            result.err()
        );
    }
}

/// 额外的单元测试：验证资源文件不为空
///
/// 这不是属性测试，但是是一个重要的基础检查
#[test]
fn test_resources_are_not_empty() {
    let resources = EmbeddedResources::get();

    assert!(!resources.atx_agent.is_empty(), "atx-agent 不应为空");
    assert!(
        !resources.app_uiautomator_apk.is_empty(),
        "app-uiautomator.apk 不应为空"
    );
    assert!(
        !resources.app_uiautomator_test_apk.is_empty(),
        "app-uiautomator-test.apk 不应为空"
    );
}

/// 额外的单元测试：验证 MD5 字符串格式正确
#[test]
fn test_md5_strings_are_valid() {
    let resources = EmbeddedResources::get();

    // MD5 应该是 32 个十六进制字符
    assert_eq!(
        resources.atx_agent_md5.len(),
        32,
        "atx-agent MD5 长度应为 32"
    );
    assert_eq!(
        resources.app_uiautomator_apk_md5.len(),
        32,
        "app-uiautomator.apk MD5 长度应为 32"
    );
    assert_eq!(
        resources.app_uiautomator_test_apk_md5.len(),
        32,
        "app-uiautomator-test.apk MD5 长度应为 32"
    );

    // 验证都是十六进制字符
    assert!(
        resources
            .atx_agent_md5
            .chars()
            .all(|c| c.is_ascii_hexdigit()),
        "atx-agent MD5 应该是十六进制"
    );
    assert!(
        resources
            .app_uiautomator_apk_md5
            .chars()
            .all(|c| c.is_ascii_hexdigit()),
        "app-uiautomator.apk MD5 应该是十六进制"
    );
    assert!(
        resources
            .app_uiautomator_test_apk_md5
            .chars()
            .all(|c| c.is_ascii_hexdigit()),
        "app-uiautomator-test.apk MD5 应该是十六进制"
    );
}
