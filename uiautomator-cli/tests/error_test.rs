//! 错误处理测试
//!
//! 测试错误消息的格式、内容和彩色输出

use uiautomator_cli::error::CliError;

#[test]
fn test_device_not_found_error_message() {
    let error = CliError::DeviceNotFound;
    let message = format!("{}", error);

    // 验证错误消息包含关键信息
    assert!(
        message.contains("未找到连接的设备"),
        "错误消息应该包含主要描述"
    );
    assert!(message.contains("可能的原因"), "错误消息应该包含可能的原因");
    assert!(message.contains("解决方案"), "错误消息应该包含解决方案");

    // 验证包含具体的解决建议
    assert!(message.contains("USB"), "应该提到 USB 连接");
    assert!(message.contains("adb devices"), "应该提到 adb devices 命令");
    assert!(message.contains("USB 调试"), "应该提到 USB 调试模式");
}

#[test]
fn test_install_failed_error_message() {
    let error = CliError::InstallFailed("推送文件失败".to_string());
    let message = format!("{}", error);

    // 验证错误消息包含失败原因
    assert!(message.contains("安装失败"), "错误消息应该包含安装失败描述");
    assert!(message.contains("推送文件失败"), "错误消息应该包含具体原因");
    assert!(message.contains("解决方案"), "错误消息应该包含解决方案");
}

#[test]
fn test_adb_error_message() {
    let error = CliError::AdbError("ADB 连接超时".to_string());
    let message = format!("{}", error);

    // 验证错误消息包含 ADB 相关信息
    assert!(message.contains("ADB 错误"), "错误消息应该标识为 ADB 错误");
    assert!(message.contains("ADB 连接超时"), "错误消息应该包含具体错误");
}

#[test]
fn test_service_error_message() {
    let error = CliError::ServiceError("服务启动超时".to_string());
    let message = format!("{}", error);

    // 验证错误消息包含服务相关信息
    assert!(message.contains("服务错误"), "错误消息应该标识为服务错误");
    assert!(message.contains("服务启动超时"), "错误消息应该包含具体错误");
}

#[test]
fn test_error_message_has_color() {
    let error = CliError::InstallFailed("测试错误".to_string());
    let colored_message = error.colored_message();
    let plain_message = format!("{}", error);

    // 在无 TTY / 禁色环境中，colored crate 可能返回纯文本。
    // 这里优先保证语义正确；若启用了颜色，再校验 ANSI 码。
    assert!(
        colored_message.contains("安装失败"),
        "彩色消息应包含错误标题"
    );
    assert!(
        colored_message.contains("测试错误"),
        "彩色消息应包含原始错误原因"
    );
    if colored_message != plain_message {
        assert!(
            colored_message.contains("\x1b["),
            "启用彩色输出时应包含 ANSI 颜色代码"
        );
    }
}

#[test]
fn test_device_not_found_colored_message() {
    let error = CliError::DeviceNotFound;
    let colored_message = error.colored_message();
    let plain_message = format!("{}", error);

    // 验证彩色消息包含完整语义内容；仅在启用彩色时验证 ANSI 码。
    assert!(
        colored_message.contains("未找到连接的设备"),
        "应该包含错误描述"
    );
    assert!(colored_message.contains("可能的原因"), "应该包含原因分析");
    assert!(colored_message.contains("解决方案"), "应该包含解决方案");
    if colored_message != plain_message {
        assert!(
            colored_message.contains("\x1b["),
            "启用彩色时应该包含颜色代码"
        );
    }
}

#[test]
fn test_multiple_devices_error_message() {
    let devices = vec!["device1".to_string(), "device2".to_string()];
    let error = CliError::MultipleDevices(devices);
    let message = format!("{}", error);

    // 验证错误消息包含设备列表
    assert!(
        message.contains("检测到多个设备"),
        "错误消息应该说明有多个设备"
    );
    assert!(message.contains("device1"), "应该列出第一个设备");
    assert!(message.contains("device2"), "应该列出第二个设备");
    assert!(message.contains("--serial"), "应该提示使用 --serial 参数");
}

#[test]
fn test_resource_error_message() {
    let error = CliError::ResourceError("资源文件损坏".to_string());
    let message = format!("{}", error);

    // 验证错误消息包含资源相关信息
    assert!(message.contains("资源错误"), "错误消息应该标识为资源错误");
    assert!(message.contains("资源文件损坏"), "错误消息应该包含具体错误");
}

#[test]
fn test_error_provides_actionable_solutions() {
    let error = CliError::DeviceNotFound;
    let message = format!("{}", error);

    // 验证解决方案是可操作的
    assert!(message.contains("检查"), "解决方案应该包含具体操作");
    assert!(message.contains("运行"), "解决方案应该包含具体命令");
    assert!(message.contains("启用"), "解决方案应该包含具体设置");
}

#[test]
fn test_error_display_format() {
    let error = CliError::InstallFailed("测试".to_string());
    let message = format!("{}", error);

    // 验证错误消息格式清晰
    assert!(!message.is_empty(), "错误消息不应为空");
    assert!(message.len() > 10, "错误消息应该有足够的信息量");
}
