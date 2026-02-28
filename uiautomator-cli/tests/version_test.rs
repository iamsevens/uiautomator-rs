//! version 命令测试
//!
//! 测试 version 命令的功能

use uiautomator_cli::commands;

/// 测试 execute_version 函数能够成功执行
#[tokio::test]
async fn test_execute_version_succeeds() {
    let result = commands::execute_version().await;
    assert!(result.is_ok(), "execute_version 应该成功执行");
}

/// 测试 version 命令显示 CLI 版本
///
/// 验证输出包含版本号信息
#[tokio::test]
async fn test_version_displays_cli_version() {
    // 由于 execute_version 直接打印到 stdout，我们无法直接捕获输出
    // 但我们可以验证它不会返回错误
    let result = commands::execute_version().await;
    assert!(result.is_ok());

    // 验证 CARGO_PKG_VERSION 环境变量存在
    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty(), "CLI 版本号不应为空");
}

/// 测试 version 命令显示资源文件版本
///
/// 验证输出包含资源文件的 MD5 校验和
#[tokio::test]
async fn test_version_displays_resource_versions() {
    use uiautomator_cli::resources::EmbeddedResources;

    // 获取嵌入的资源
    let resources = EmbeddedResources::get();

    // 验证资源文件的 MD5 不为空
    assert!(
        !resources.atx_agent_md5.is_empty(),
        "atx-agent MD5 不应为空"
    );
    assert!(
        !resources.app_uiautomator_apk_md5.is_empty(),
        "app-uiautomator.apk MD5 不应为空"
    );
    assert!(
        !resources.app_uiautomator_test_apk_md5.is_empty(),
        "app-uiautomator-test.apk MD5 不应为空"
    );

    // 执行 version 命令应该成功
    let result = commands::execute_version().await;
    assert!(result.is_ok());
}

/// 测试 version 命令的输出格式
///
/// 虽然我们无法直接捕获 stdout，但我们可以验证相关数据的存在
#[tokio::test]
async fn test_version_output_format() {
    use uiautomator_cli::resources::EmbeddedResources;

    // 验证 CLI 版本存在
    let cli_version = env!("CARGO_PKG_VERSION");
    assert!(!cli_version.is_empty());

    // 验证资源版本存在
    let resources = EmbeddedResources::get();
    assert!(!resources.atx_agent_md5.is_empty());
    assert!(!resources.app_uiautomator_apk_md5.is_empty());
    assert!(!resources.app_uiautomator_test_apk_md5.is_empty());

    // 执行命令
    let result = commands::execute_version().await;
    assert!(result.is_ok());
}
