//! uiautomator-cli 主入口
//!
//! 命令行工具，用于管理 Android 设备上的 ATX-Agent 服务

use clap::Parser;
use colored::Colorize;

// 导入模块
mod cli;
mod commands;
mod error;
mod installer;
mod resources;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();

    // 解析命令行参数
    let cli = Cli::parse();

    // 执行命令
    let result = match cli.command {
        Commands::Init { force } => commands::execute_init(cli.serial, force).await,
        Commands::Status => commands::execute_status(cli.serial).await,
        Commands::Restart => commands::execute_restart(cli.serial).await,
        Commands::Uninstall => commands::execute_uninstall(cli.serial).await,
        Commands::Version => commands::execute_version().await,
    };

    // 处理结果
    if let Err(e) = result {
        eprintln!("{} {}", "✗ 错误:".red().bold(), e);
        std::process::exit(1);
    }
}
