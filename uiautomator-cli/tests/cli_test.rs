//! CLI 参数解析测试
//!
//! 测试命令行参数的正确解析

use clap::Parser;

// 导入 CLI 结构（将在 main.rs 中定义）
// 由于 main.rs 不是库，我们需要在 lib.rs 中公开这些结构
use uiautomator_cli::cli::{Cli, Commands};

#[test]
fn test_parse_init_command() {
    // 测试基本的 init 命令解析
    let args = vec!["uiautomator", "init"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Commands::Init { force: false }));
    assert_eq!(cli.serial, None);
}

#[test]
fn test_parse_init_with_force() {
    // 测试带 --force 选项的 init 命令
    let args = vec!["uiautomator", "init", "--force"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Commands::Init { force: true }));
}

#[test]
fn test_parse_init_with_force_short() {
    // 测试带 -f 短选项的 init 命令
    let args = vec!["uiautomator", "init", "-f"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Commands::Init { force: true }));
}

#[test]
fn test_parse_with_serial() {
    // 测试带 --serial 选项的命令
    let args = vec!["uiautomator", "--serial", "127.0.0.1:5555", "status"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.serial, Some("127.0.0.1:5555".to_string()));
    assert!(matches!(cli.command, Commands::Status));
}

#[test]
fn test_parse_with_serial_short() {
    // 测试带 -s 短选项的命令
    let args = vec!["uiautomator", "-s", "emulator-5554", "status"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.serial, Some("emulator-5554".to_string()));
    assert!(matches!(cli.command, Commands::Status));
}

#[test]
fn test_parse_status_command() {
    // 测试 status 命令解析
    let args = vec!["uiautomator", "status"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Commands::Status));
}

#[test]
fn test_parse_restart_command() {
    // 测试 restart 命令解析
    let args = vec!["uiautomator", "restart"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Commands::Restart));
}

#[test]
fn test_parse_uninstall_command() {
    // 测试 uninstall 命令解析
    let args = vec!["uiautomator", "uninstall"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Commands::Uninstall));
}

#[test]
fn test_parse_version_command() {
    // 测试 version 命令解析
    let args = vec!["uiautomator", "version"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert!(matches!(cli.command, Commands::Version));
}

#[test]
fn test_parse_invalid_command() {
    // 测试无效命令的错误处理
    let args = vec!["uiautomator", "invalid-command"];
    let result = Cli::try_parse_from(args);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::InvalidSubcommand);
}

#[test]
fn test_parse_missing_command() {
    // 测试缺少子命令的错误处理
    let args = vec!["uiautomator"];
    let result = Cli::try_parse_from(args);

    assert!(result.is_err());
    let err = result.unwrap_err();
    // clap 在缺少子命令时会显示帮助信息
    assert_eq!(
        err.kind(),
        clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
    );
}

#[test]
fn test_parse_help_flag() {
    // 测试 --help 标志
    let args = vec!["uiautomator", "--help"];
    let result = Cli::try_parse_from(args);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
}

#[test]
fn test_parse_version_flag() {
    // 测试 --version 标志
    let args = vec!["uiautomator", "--version"];
    let result = Cli::try_parse_from(args);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
}

#[test]
fn test_serial_before_command() {
    // 测试 --serial 在命令之前
    let args = vec!["uiautomator", "--serial", "device1", "init"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.serial, Some("device1".to_string()));
    assert!(matches!(cli.command, Commands::Init { force: false }));
}

#[test]
fn test_multiple_options() {
    // 测试多个选项组合
    let args = vec!["uiautomator", "-s", "device1", "init", "--force"];
    let cli = Cli::try_parse_from(args).unwrap();

    assert_eq!(cli.serial, Some("device1".to_string()));
    assert!(matches!(cli.command, Commands::Init { force: true }));
}
