//! 嵌入资源文件管理模块
//!
//! 该模块负责管理嵌入到可执行文件中的资源文件，包括：
//! - atx-agent 二进制文件
//! - app-uiautomator.apk
//! - app-uiautomator-test.apk
//!
//! 所有资源文件在编译时通过 `include_bytes!` 宏嵌入到可执行文件中，
//! 并在 build.rs 中计算 MD5 校验和以确保完整性。

/// 嵌入的资源文件
///
/// 包含所有必需的资源文件及其 MD5 校验和
/// # Examples
///
/// ```no_run
/// use uiautomator_cli::resources::EmbeddedResources;
///
/// let resources = EmbeddedResources::get();
/// assert!(!resources.atx_agent.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct EmbeddedResources {
    /// atx-agent 二进制文件（ARM64 架构）
    pub atx_agent: &'static [u8],

    /// atx-agent 的 MD5 校验和（在 build.rs 中计算）
    pub atx_agent_md5: &'static str,

    /// app-uiautomator.apk 文件
    pub app_uiautomator_apk: &'static [u8],

    /// app-uiautomator.apk 的 MD5 校验和
    pub app_uiautomator_apk_md5: &'static str,

    /// app-uiautomator-test.apk 文件
    pub app_uiautomator_test_apk: &'static [u8],

    /// app-uiautomator-test.apk 的 MD5 校验和
    pub app_uiautomator_test_apk_md5: &'static str,
}

impl EmbeddedResources {
    /// 获取嵌入的资源文件
    ///
    /// 该方法返回一个包含所有嵌入资源文件的结构体。
    /// 资源文件在编译时通过 `include_bytes!` 宏嵌入，
    /// MD5 校验和在 build.rs 中计算并通过环境变量传递。
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uiautomator_cli::resources::EmbeddedResources;
    ///
    /// let resources = EmbeddedResources::get();
    /// println!("atx-agent 大小: {} bytes", resources.atx_agent.len());
    /// println!("atx-agent MD5: {}", resources.atx_agent_md5);
    /// ```
    pub fn get() -> Self {
        Self {
            // 嵌入 atx-agent 二进制文件
            atx_agent: include_bytes!("../assets/atx-agent"),
            atx_agent_md5: env!("ATX_AGENT_MD5"),

            // 嵌入 app-uiautomator.apk
            app_uiautomator_apk: include_bytes!("../assets/app-uiautomator.apk"),
            app_uiautomator_apk_md5: env!("APP_UIAUTOMATOR_APK_MD5"),

            // 嵌入 app-uiautomator-test.apk
            app_uiautomator_test_apk: include_bytes!("../assets/app-uiautomator-test.apk"),
            app_uiautomator_test_apk_md5: env!("APP_UIAUTOMATOR_TEST_APK_MD5"),
        }
    }

    /// 验证资源文件的 MD5 校验和
    ///
    /// 该方法计算嵌入资源文件的 MD5 校验和，并与构建时计算的值进行比较。
    /// 如果校验和不匹配，说明资源文件可能已损坏。
    ///
    /// # Returns
    ///
    /// 如果所有资源文件的 MD5 校验和都匹配，返回 `Ok(())`，
    /// 否则返回包含错误信息的 `Err`。
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uiautomator_cli::resources::EmbeddedResources;
    ///
    /// let resources = EmbeddedResources::get();
    /// match resources.verify_integrity() {
    ///     Ok(_) => println!("所有资源文件完整性验证通过"),
    ///     Err(e) => eprintln!("资源文件完整性验证失败: {}", e),
    /// }
    /// ```
    #[allow(dead_code)]
    pub fn verify_integrity(&self) -> Result<(), String> {
        // 辅助函数：验证单个资源文件的 MD5
        fn verify_md5(name: &str, data: &[u8], expected_md5: &str) -> Result<(), String> {
            let computed_md5 = format!("{:x}", md5::compute(data));
            if computed_md5 != expected_md5 {
                return Err(format!(
                    "{} MD5 校验和不匹配: 期望 {}, 实际 {}",
                    name, expected_md5, computed_md5
                ));
            }
            Ok(())
        }

        // 验证所有资源文件
        verify_md5("atx-agent", self.atx_agent, self.atx_agent_md5)?;
        verify_md5(
            "app-uiautomator.apk",
            self.app_uiautomator_apk,
            self.app_uiautomator_apk_md5,
        )?;
        verify_md5(
            "app-uiautomator-test.apk",
            self.app_uiautomator_test_apk,
            self.app_uiautomator_test_apk_md5,
        )?;

        Ok(())
    }

    /// 获取资源文件的总大小（字节）
    #[allow(dead_code)]
    /// # Examples
    ///
    /// ```no_run
    /// use uiautomator_cli::resources::EmbeddedResources;
    ///
    /// let resources = EmbeddedResources::get();
    /// let bytes = resources.total_size();
    /// assert!(bytes > 0);
    /// ```
    pub fn total_size(&self) -> usize {
        self.atx_agent.len() + self.app_uiautomator_apk.len() + self.app_uiautomator_test_apk.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resources_can_be_loaded() {
        let resources = EmbeddedResources::get();
        assert!(!resources.atx_agent.is_empty());
        assert!(!resources.app_uiautomator_apk.is_empty());
        assert!(!resources.app_uiautomator_test_apk.is_empty());
    }

    #[test]
    fn test_verify_integrity_passes() {
        let resources = EmbeddedResources::get();
        assert!(resources.verify_integrity().is_ok());
    }
}
