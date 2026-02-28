//! ATX-Agent 安装模式示例
//!
//! 此示例演示如何使用 ATX-Agent 安装功能。
//!
//! 注意：此功能需要启用 `atx-agent-install` feature 并下载资源文件。
//!
//! 运行方式：
//! ```bash
//! # 1. 下载资源文件
//! cd assets
//! ./download_atx_agent.sh  # Linux/macOS
//! # 或
//! .\download_atx_agent.ps1  # Windows
//!
//! # 2. 启用 feature 运行示例
//! cargo run --example atx_agent_install_demo --features atx-agent-install
//! ```

#[cfg(feature = "atx-agent-install")]
use uiautomator::Device;

#[cfg(feature = "atx-agent-install")]
#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志
    env_logger::init();

    println!("=== ATX-Agent 安装模式示例 ===\n");

    // 连接到设备（使用 Direct 模式，因为还没有安装 ATX-Agent）
    println!("1. 连接到设备（Direct 模式）...");
    let device = Device::connect_quick(None).await?;
    println!("   ✓ 设备连接成功\n");

    // 检查是否已安装
    println!("2. 检查 ATX-Agent 是否已安装...");
    let installed = device.check_atx_agent_installed().await?;
    if installed {
        println!("   ✓ ATX-Agent 已安装\n");
    } else {
        println!("   - ATX-Agent 未安装\n");
    }

    // 安装 ATX-Agent
    println!("3. 安装 ATX-Agent...");
    println!("   这可能需要几分钟时间...");
    device.install_atx_agent(false).await?;
    println!("   ✓ ATX-Agent 安装完成\n");

    // 验证安装
    println!("4. 验证安装...");
    let installed = device.check_atx_agent_installed().await?;
    if installed {
        println!("   ✓ ATX-Agent 已成功安装\n");
    } else {
        println!("   ✗ ATX-Agent 安装失败\n");
        return Ok(());
    }

    // 现在可以使用 ATX-Agent 模式连接
    println!("5. 使用 ATX-Agent 模式重新连接...");
    let device = Device::connect(None).await?;
    println!("   ✓ 使用 ATX-Agent 模式连接成功\n");

    // 获取设备信息
    println!("6. 获取设备信息...");
    let info = device.info().await?;
    println!("   设备信息:");
    println!(
        "     - 屏幕尺寸: {}x{}",
        info.display_width, info.display_height
    );
    println!("     - SDK 版本: {}", info.sdk_int);
    println!("     - 当前应用: {}", info.current_package_name);
    println!();

    println!("=== 示例完成 ===");
    println!();
    println!("提示：");
    println!("  - ATX-Agent 模式提供更稳定的服务");
    println!("  - 服务会在后台持续运行");
    println!("  - 如需停止服务，使用: device.stop_atx_agent().await?");

    Ok(())
}

#[cfg(not(feature = "atx-agent-install"))]
fn main() {
    eprintln!("错误：此示例需要启用 'atx-agent-install' feature");
    eprintln!();
    eprintln!("请按以下步骤操作：");
    eprintln!("1. 下载资源文件:");
    eprintln!("   cd assets && ./download_atx_agent.sh");
    eprintln!();
    eprintln!("2. 启用 feature 运行:");
    eprintln!("   cargo run --example atx_agent_install_demo --features atx-agent-install");
    std::process::exit(1);
}
