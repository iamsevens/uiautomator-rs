// 资源文件管理测试
// 测试嵌入的资源文件是否存在且完整

#[cfg(test)]
mod tests {
    // 导入 resources 模块
    use uiautomator_cli::resources::EmbeddedResources;

    #[test]
    fn test_embedded_resources_exist() {
        let resources = EmbeddedResources::get();

        // 测试 atx-agent 二进制文件存在且不为空
        assert!(
            !resources.atx_agent.is_empty(),
            "atx-agent 二进制文件不应为空"
        );
        assert!(
            resources.atx_agent.len() > 1024,
            "atx-agent 文件大小应该大于 1KB"
        );

        // 测试 app-uiautomator.apk 存在且不为空
        assert!(
            !resources.app_uiautomator_apk.is_empty(),
            "app-uiautomator.apk 不应为空"
        );
        assert!(
            resources.app_uiautomator_apk.len() > 1024,
            "APK 文件大小应该大于 1KB"
        );

        // 测试 app-uiautomator-test.apk 存在且不为空
        assert!(
            !resources.app_uiautomator_test_apk.is_empty(),
            "app-uiautomator-test.apk 不应为空"
        );
        assert!(
            resources.app_uiautomator_test_apk.len() > 1024,
            "Test APK 文件大小应该大于 1KB"
        );
    }

    #[test]
    fn test_resource_md5_matches() {
        let resources = EmbeddedResources::get();

        // 测试 atx-agent 的 MD5 校验和
        let computed_md5 = format!("{:x}", md5::compute(resources.atx_agent));
        assert_eq!(
            computed_md5, resources.atx_agent_md5,
            "atx-agent MD5 校验和不匹配"
        );

        // 测试 app-uiautomator.apk 的 MD5 校验和
        let computed_apk_md5 = format!("{:x}", md5::compute(resources.app_uiautomator_apk));
        assert_eq!(
            computed_apk_md5, resources.app_uiautomator_apk_md5,
            "app-uiautomator.apk MD5 校验和不匹配"
        );

        // 测试 app-uiautomator-test.apk 的 MD5 校验和
        let computed_test_apk_md5 =
            format!("{:x}", md5::compute(resources.app_uiautomator_test_apk));
        assert_eq!(
            computed_test_apk_md5, resources.app_uiautomator_test_apk_md5,
            "app-uiautomator-test.apk MD5 校验和不匹配"
        );
    }

    #[test]
    fn test_resource_metadata() {
        let resources = EmbeddedResources::get();

        // 验证 MD5 字符串格式正确（32个十六进制字符）
        assert_eq!(resources.atx_agent_md5.len(), 32, "MD5 应该是 32 个字符");
        assert!(
            resources
                .atx_agent_md5
                .chars()
                .all(|c: char| c.is_ascii_hexdigit()),
            "MD5 应该只包含十六进制字符"
        );

        assert_eq!(resources.app_uiautomator_apk_md5.len(), 32);
        assert!(resources
            .app_uiautomator_apk_md5
            .chars()
            .all(|c: char| c.is_ascii_hexdigit()));

        assert_eq!(resources.app_uiautomator_test_apk_md5.len(), 32);
        assert!(resources
            .app_uiautomator_test_apk_md5
            .chars()
            .all(|c: char| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_verify_integrity() {
        let resources = EmbeddedResources::get();

        // 使用内置的完整性验证方法
        let result = resources.verify_integrity();
        assert!(result.is_ok(), "资源文件完整性验证应该通过: {:?}", result);
    }

    #[test]
    fn test_total_size() {
        let resources = EmbeddedResources::get();

        let total = resources.total_size();
        let expected = resources.atx_agent.len()
            + resources.app_uiautomator_apk.len()
            + resources.app_uiautomator_test_apk.len();

        assert_eq!(total, expected, "总大小计算应该正确");
        assert!(total > 0, "总大小应该大于 0");
    }
}
