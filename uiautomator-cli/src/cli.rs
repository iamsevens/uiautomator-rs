//! CLI 命令行接口定义
//!
//! 使用 clap 库定义命令行参数和子命令

use clap::{Parser, Subcommand};

/// uiautomator CLI 工具
/// # Examples
///
/// ```
/// use clap::Parser;
/// use uiautomator_cli::cli::{Cli, Commands};
///
/// let cli = Cli::parse_from(["uiautomator", "--serial", "emulator-5554", "status"]);
/// assert_eq!(cli.serial.as_deref(), Some("emulator-5554"));
/// assert!(matches!(cli.command, Commands::Status));
/// ```
#[derive(Parser, Debug)]
#[command(name = "uiautomator")]
#[command(version)]
#[command(about = "Android UI 自动化工具 - ATX-Agent 管理器", long_about = None)]
pub struct Cli {
    /// 设备序列号（可选，默认使用第一个连接的设备）
    #[arg(short, long, global = true)]
    pub serial: Option<String>,

    /// 子命令
    #[command(subcommand)]
    pub command: Commands,
}

/// 可用的子命令
/// # Examples
///
/// ```
/// use clap::Parser;
/// use uiautomator_cli::cli::{Cli, Commands};
///
/// let cli = Cli::parse_from(["uiautomator", "init", "--force"]);
/// assert!(matches!(cli.command, Commands::Init { force: true }));
/// ```
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 初始化设备（安装并启动 ATX-Agent）
    Init {
        /// 强制重新安装（即使已安装）
        #[arg(short, long)]
        force: bool,
    },

    /// 查看 ATX-Agent 状态
    Status,

    /// 重启 ATX-Agent 服务
    Restart,

    /// 卸载 ATX-Agent
    Uninstall,

    /// 显示版本信息
    Version,
}
