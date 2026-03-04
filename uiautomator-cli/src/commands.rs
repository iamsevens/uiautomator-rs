//! 命令实现模块
//!
//! 包含所有子命令的具体实现

use anyhow::Result;
use colored::Colorize;

use crate::resources::EmbeddedResources;

/// 执行 init 命令
///
/// 初始化 Android 设备，安装并启动 ATX-Agent 服务
///
/// # 参数
///
/// * `serial` - 设备序列号（可选）。如果为 None，将自动选择第一个连接的设备
/// * `force` - 是否强制重新安装。如果为 true，即使已安装也会重新安装
///
/// # 返回
///
/// 初始化成功返回 Ok(())
///
/// # 错误
///
/// * 如果未找到连接的设备，返回错误
/// * 如果安装过程失败，返回错误
/// * 如果服务启动失败，返回错误
/// # Examples
///
/// ```no_run
/// use uiautomator_cli::commands::execute_init;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     execute_init(Some("emulator-5554".to_string()), false).await?;
///     Ok(())
/// }
/// ```
pub async fn execute_init(serial: Option<String>, force: bool) -> Result<()> {
    use crate::installer::Installer;
    use anyhow::Context;

    println!("{}", "🚀 开始初始化设备...".cyan().bold());
    println!();

    // 1. 创建安装器（会自动检测设备）
    print!("⏳ 正在连接设备...");
    let installer = Installer::new(serial.clone())
        .await
        .map_err(|e| anyhow::anyhow!("无法连接到设备: {e}"))?;
    println!(" {}", "完成".green());

    // 2. 显示设备信息
    println!("✓ 检测到设备: {}", installer.device_serial().green());
    println!();

    // 3. 检查是否已安装（如果不是强制安装）
    if !force {
        print!("⏳ 检查安装状态...");
        let installed = installer
            .check_installed()
            .await
            .context("检查安装状态失败")?;
        println!(" {}", "完成".green());

        if installed {
            println!("{}", "ℹ ATX-Agent 已安装".yellow());

            // 检查服务状态
            print!("⏳ 检查服务状态...");
            let status = installer.status().await.context("检查服务状态失败")?;
            println!(" {}", "完成".green());

            if status.running {
                println!();
                println!("{}", "✓ ATX-Agent 服务正在运行".green().bold());
                if let Some(ref version) = status.version {
                    println!("  版本: {}", version.cyan());
                }
                println!("  端口: {}", "7912".cyan());
                println!();
                println!("{}", "💡 提示: 如需重新安装，请使用 --force 选项".yellow());
                println!("   例如: uiautomator init --force");
                return Ok(());
            } else {
                println!();
                println!("{}", "⚠ ATX-Agent 已安装但未运行，将重新启动...".yellow());
            }
        }
    } else {
        println!("{}", "⚠ 强制重新安装模式".yellow());
    }

    // 4. 执行安装
    println!();
    println!("{}", "📦 开始安装 ATX-Agent...".cyan().bold());
    installer
        .install(force)
        .await
        .context("安装 ATX-Agent 失败")?;

    // 5. 验证安装结果
    print!("⏳ 验证安装结果...");
    let status = installer.status().await.context("验证安装结果失败")?;
    println!(" {}", "完成".green());

    if status.running {
        println!();
        println!("{}", "✅ 初始化完成！".green().bold());
        println!();
        if let Some(ref version) = status.version {
            println!("  {} {}", "ATX-Agent 版本:".bold(), version.cyan());
        }
        println!("  {} {}", "服务端口:".bold(), "7912".cyan());
        println!();
        println!("{}", "💡 下一步:".cyan().bold());
        println!("   现在可以使用 uiautomator 库连接到设备了");
        println!("   运行 'uiautomator status' 查看服务状态");
    } else {
        return Err(anyhow::anyhow!(
            "安装完成但服务未运行\n\n\
            可能的原因:\n\
              1. 设备权限不足\n\
              2. 服务启动超时\n\
              3. 设备资源不足\n\n\
            解决方案:\n\
              1. 检查设备是否有足够的存储空间\n\
              2. 尝试重启设备后再次运行 init\n\
              3. 运行 'uiautomator restart' 尝试重启服务"
        ));
    }

    Ok(())
}

/// 执行 status 命令
///
/// 查询并显示 ATX-Agent 服务的运行状态
///
/// # 参数
///
/// * `serial` - 设备序列号（可选）。如果为 None，将自动选择第一个连接的设备
///
/// # 返回
///
/// 查询成功返回 Ok(())
///
/// # 错误
///
/// * 如果未找到连接的设备，返回错误
/// * 如果查询状态失败，返回错误
/// # Examples
///
/// ```no_run
/// use uiautomator_cli::commands::execute_status;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     execute_status(Some("emulator-5554".to_string())).await?;
///     Ok(())
/// }
/// ```
pub async fn execute_status(serial: Option<String>) -> Result<()> {
    use crate::installer::Installer;
    use anyhow::Context;

    println!("{}", "📊 查询设备状态...".cyan().bold());
    println!();

    // 1. 创建安装器（会自动检测设备）
    print!("⏳ 正在连接设备...");
    let installer = Installer::new(serial.clone())
        .await
        .map_err(|e| anyhow::anyhow!("无法连接到设备: {e}"))?;
    println!(" {}", "完成".green());

    // 2. 显示设备信息
    println!("✓ 设备: {}", installer.device_serial().green());
    println!();

    // 3. 查询服务状态
    print!("⏳ 查询 ATX-Agent 状态...");
    let status = installer.status().await.context("查询服务状态失败")?;
    println!(" {}", "完成".green());
    println!();

    // 4. 显示状态信息
    display_status_info(&status);

    Ok(())
}

/// 显示状态信息
///
/// 格式化并显示 ATX-Agent 服务的状态信息
///
/// # 参数
///
/// * `status` - 服务状态
fn display_status_info(status: &crate::installer::ServiceStatus) {
    if status.running {
        // 服务正在运行
        println!("{}", "✅ ATX-Agent 状态: 运行中".green().bold());
        println!();

        // 显示详细信息
        if let Some(ref version) = status.version {
            println!("  {} {}", "版本:".bold(), version.cyan());
        }
        println!("  {} {}", "服务端口:".bold(), "7912".cyan());
        println!(
            "  {} {}",
            "HTTP 端点:".bold(),
            "http://127.0.0.1:7912".cyan()
        );
        println!();

        // 显示提示信息
        println!("{}", "💡 提示:".cyan().bold());
        println!("   服务正常运行，可以使用 uiautomator 库连接到设备");
        println!("   运行 'uiautomator restart' 重启服务");
        println!("   运行 'uiautomator uninstall' 卸载服务");
    } else {
        // 服务未运行
        println!("{}", "✗ ATX-Agent 状态: 未运行".red().bold());
        println!();

        // 显示提示信息
        println!("{}", "💡 提示:".yellow().bold());
        println!("   运行 'uiautomator init' 初始化设备");
        println!("   或运行 'uiautomator restart' 重启服务（如果已安装）");
    }
}

/// 执行 restart 命令
///
/// 重启 ATX-Agent 服务
///
/// # 参数
///
/// * `serial` - 设备序列号（可选）。如果为 None，将自动选择第一个连接的设备
///
/// # 返回
///
/// 重启成功返回 Ok(())
///
/// # 错误
///
/// * 如果未找到连接的设备，返回错误
/// * 如果服务未安装，返回错误
/// * 如果重启失败，返回错误
/// # Examples
///
/// ```no_run
/// use uiautomator_cli::commands::execute_restart;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     execute_restart(Some("emulator-5554".to_string())).await?;
///     Ok(())
/// }
/// ```
pub async fn execute_restart(serial: Option<String>) -> Result<()> {
    use crate::installer::Installer;
    use anyhow::Context;

    println!("{}", "🔄 重启 ATX-Agent 服务...".cyan().bold());
    println!();

    // 1. 创建安装器（会自动检测设备）
    print!("⏳ 正在连接设备...");
    let installer = Installer::new(serial.clone())
        .await
        .map_err(|e| anyhow::anyhow!("无法连接到设备: {e}"))?;
    println!(" {}", "完成".green());

    // 2. 显示设备信息
    println!("✓ 设备: {}", installer.device_serial().green());
    println!();

    // 3. 检查是否已安装
    print!("⏳ 检查安装状态...");
    let installed = installer
        .check_installed()
        .await
        .context("检查安装状态失败")?;
    println!(" {}", "完成".green());

    if !installed {
        println!();
        println!("{}", "✗ ATX-Agent 未安装".red().bold());
        println!();
        println!("{}", "💡 提示:".yellow().bold());
        println!("   请先运行 'uiautomator init' 初始化设备");
        return Err(anyhow::anyhow!(
            "ATX-Agent 未安装\n\n\
            解决方案:\n\
              运行 'uiautomator init' 初始化设备"
        ));
    }

    // 4. 执行重启
    println!();
    print!("⏳ 正在重启服务...");
    installer.restart().await.context("重启服务失败")?;
    println!(" {}", "完成".green());

    // 5. 验证服务状态
    print!("⏳ 验证服务状态...");
    let status = installer.status().await.context("验证服务状态失败")?;
    println!(" {}", "完成".green());

    if status.running {
        println!();
        println!("{}", "✅ 服务重启完成！".green().bold());
        println!();
        if let Some(ref version) = status.version {
            println!("  {} {}", "ATX-Agent 版本:".bold(), version.cyan());
        }
        println!("  {} {}", "服务端口:".bold(), "7912".cyan());
        println!();
        println!("{}", "💡 提示:".cyan().bold());
        println!("   运行 'uiautomator status' 查看服务状态");
    } else {
        return Err(anyhow::anyhow!(
            "重启完成但服务未运行\n\n\
            可能的原因:\n\
              1. 设备权限不足\n\
              2. 服务启动超时\n\
              3. 设备资源不足\n\n\
            解决方案:\n\
              1. 尝试再次运行 'uiautomator restart'\n\
              2. 运行 'uiautomator init --force' 重新安装"
        ));
    }

    Ok(())
}

/// 执行 uninstall 命令
///
/// 卸载 ATX-Agent 服务和相关文件
///
/// # 参数
///
/// * `serial` - 设备序列号（可选）。如果为 None，将自动选择第一个连接的设备
///
/// # 返回
///
/// 卸载成功返回 Ok(())
///
/// # 错误
///
/// * 如果未找到连接的设备，返回错误
/// * 如果卸载过程失败，返回错误
/// # Examples
///
/// ```no_run
/// use uiautomator_cli::commands::execute_uninstall;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     execute_uninstall(Some("emulator-5554".to_string())).await?;
///     Ok(())
/// }
/// ```
pub async fn execute_uninstall(serial: Option<String>) -> Result<()> {
    use crate::installer::Installer;
    use anyhow::Context;

    println!("{}", "🗑️  卸载 ATX-Agent...".cyan().bold());
    println!();

    // 1. 创建安装器（会自动检测设备）
    print!("⏳ 正在连接设备...");
    let installer = Installer::new(serial.clone())
        .await
        .map_err(|e| anyhow::anyhow!("无法连接到设备: {e}"))?;
    println!(" {}", "完成".green());

    // 2. 显示设备信息
    println!("✓ 设备: {}", installer.device_serial().green());
    println!();

    // 3. 检查是否已安装
    print!("⏳ 检查安装状态...");
    let installed = installer
        .check_installed()
        .await
        .context("检查安装状态失败")?;
    println!(" {}", "完成".green());

    if !installed {
        println!();
        println!("{}", "ℹ ATX-Agent 未安装".yellow());
        println!();
        println!("{}", "💡 提示:".cyan().bold());
        println!("   设备上未安装 ATX-Agent，无需卸载");
        return Ok(());
    }

    // 4. 执行卸载
    println!();
    println!("{}", "📦 开始卸载...".cyan().bold());

    print!("⏳ 停止服务...");
    // 卸载方法会处理所有步骤
    installer.uninstall().await.context("卸载失败")?;
    println!(" {}", "完成".green());

    // 5. 验证卸载结果
    print!("⏳ 验证卸载结果...");
    let status = installer.status().await.context("验证卸载结果失败")?;
    println!(" {}", "完成".green());

    if !status.running {
        println!();
        println!("{}", "✅ 卸载完成！".green().bold());
        println!();
        println!("{}", "💡 提示:".cyan().bold());
        println!("   ATX-Agent 已从设备上完全移除");
        println!("   如需重新使用，请运行 'uiautomator init'");
    } else {
        return Err(anyhow::anyhow!(
            "卸载完成但服务仍在运行\n\n\
            可能的原因:\n\
              1. 服务停止失败\n\
              2. 设备权限不足\n\n\
            解决方案:\n\
              1. 尝试手动停止服务\n\
              2. 重启设备后再次尝试卸载"
        ));
    }

    Ok(())
}

/// 执行 version 命令
///
/// 显示 CLI 工具版本和内置资源文件版本
///
/// # 返回
///
/// 显示成功返回 Ok(())
///
/// # 需求
///
/// 满足需求 5.4 和 7.4：
/// - 5.4: WHEN 用户查询版本信息 THEN CLI Tool SHALL 显示内置资源文件的版本号
/// - 7.4: WHEN 用户执行 `uiautomator version` 命令 THEN CLI Tool SHALL 显示 CLI 工具版本和内置资源版本
/// # Examples
///
/// ```no_run
/// use uiautomator_cli::commands::execute_version;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     execute_version().await?;
///     Ok(())
/// }
/// ```
pub async fn execute_version() -> Result<()> {
    // 显示 CLI 工具信息
    println!();
    println!("{}", "📦 uiautomator-cli".cyan().bold());
    println!();
    println!("  {} {}", "版本:".bold(), env!("CARGO_PKG_VERSION").green());
    println!("  {} {}", "描述:".bold(), env!("CARGO_PKG_DESCRIPTION"));
    println!();

    // 显示内置资源版本
    println!("{}", "🔧 内置资源版本:".cyan().bold());
    println!();

    let resources = EmbeddedResources::get();
    println!(
        "  {} {}",
        "atx-agent:".bold(),
        resources.atx_agent_md5.yellow()
    );
    println!(
        "  {} {}",
        "app-uiautomator.apk:".bold(),
        resources.app_uiautomator_apk_md5.yellow()
    );
    println!(
        "  {} {}",
        "app-uiautomator-test.apk:".bold(),
        resources.app_uiautomator_test_apk_md5.yellow()
    );
    println!();

    // 显示提示信息
    println!("{}", "💡 提示:".cyan().bold());
    println!("   运行 'uiautomator --help' 查看所有可用命令");
    println!("   运行 'uiautomator init' 初始化 Android 设备");
    println!();

    Ok(())
}
