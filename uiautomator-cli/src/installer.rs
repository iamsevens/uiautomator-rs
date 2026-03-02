//! 安装器模块
//!
//! 提供 ATX-Agent 的安装、管理和状态查询功能

use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::sync::Arc;
use uiautomator::adb::AdbClient;

/// 未找到设备的错误消息
const NO_DEVICE_ERROR: &str = "未找到连接的设备\n\n\
可能的原因:\n\
  1. 设备未通过 USB 连接\n\
  2. ADB 服务未启动\n\
  3. 设备未启用 USB 调试\n\n\
解决方案:\n\
  1. 检查 USB 连接\n\
  2. 运行 'adb devices' 确认设备可见\n\
  3. 在设备上启用 USB 调试模式";

/// 服务状态
///
/// 表示 ATX-Agent 服务的运行状态和版本信息
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceStatus {
    /// 服务是否正在运行
    pub running: bool,
    /// ATX-Agent 版本（如果正在运行）
    pub version: Option<String>,
}

/// 安装器
///
/// 封装 ATX-Agent 的安装和管理逻辑
#[derive(Debug)]
pub struct Installer {
    /// ADB 客户端
    adb_client: Arc<AdbClient>,
    /// 设备序列号
    device_serial: String,
}

impl Installer {
    /// 创建新的安装器
    ///
    /// # 参数
    ///
    /// * `serial` - 设备序列号（可选）。如果为 None，将自动选择第一个连接的设备
    ///
    /// # 返回
    ///
    /// 返回配置好的安装器实例
    ///
    /// # 错误
    ///
    /// * 如果无法连接到 ADB 服务器，返回错误
    /// * 如果未找到连接的设备，返回错误
    /// * 如果指定的设备序列号不存在，返回错误
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator_cli::installer::Installer;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     // 自动选择第一个设备
    ///     let installer = Installer::new(None).await?;
    ///     
    ///     // 或指定设备序列号
    ///     let installer = Installer::new(Some("127.0.0.1:5555".to_string())).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(serial: Option<String>) -> Result<Self> {
        info!("正在创建安装器...");

        // 创建 ADB 客户端
        let adb_client = Arc::new(AdbClient::new().await?);

        // 使用 new_with_adb 创建安装器
        Self::new_with_adb(serial, adb_client).await
    }

    /// 使用已有的 ADB 客户端创建安装器
    ///
    /// # 参数
    ///
    /// * `serial` - 设备序列号（可选）。如果为 None，将自动选择第一个连接的设备
    /// * `adb_client` - 已创建的 ADB 客户端
    ///
    /// # 返回
    ///
    /// 返回配置好的安装器实例
    ///
    /// # 错误
    ///
    /// * 如果未找到连接的设备，返回错误
    /// * 如果指定的设备序列号不存在，返回错误
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator_cli::installer::Installer;
    /// use uiautomator::adb::AdbClient;
    /// use std::sync::Arc;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let adb_client = Arc::new(AdbClient::new().await?);
    ///     let installer = Installer::new_with_adb(None, adb_client).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new_with_adb(serial: Option<String>, adb_client: Arc<AdbClient>) -> Result<Self> {
        debug!("正在获取设备列表...");

        // 获取所有连接的设备
        let devices = adb_client.devices().await?;

        // 如果没有设备连接，返回错误
        if devices.is_empty() {
            return Err(anyhow!(NO_DEVICE_ERROR));
        }

        // 确定要使用的设备序列号
        let device_serial = Self::select_device(serial, &devices)?;

        Ok(Self {
            adb_client,
            device_serial,
        })
    }

    /// 选择要使用的设备
    ///
    /// # 参数
    ///
    /// * `serial` - 指定的设备序列号（可选）
    /// * `devices` - 可用的设备列表
    ///
    /// # 返回
    ///
    /// 返回选中的设备序列号
    ///
    /// # 错误
    ///
    /// 如果指定的设备不存在，返回错误
    fn select_device(serial: Option<String>, devices: &[String]) -> Result<String> {
        match serial {
            Some(s) => {
                // 验证指定的序列号是否存在
                if !devices.contains(&s) {
                    return Err(anyhow!(
                        "未找到设备: {}\n\n\
                        可用的设备:\n{}\n\n\
                        提示: 使用 'adb devices' 查看所有连接的设备",
                        s,
                        devices
                            .iter()
                            .map(|d| format!("  - {}", d))
                            .collect::<Vec<_>>()
                            .join("\n")
                    ));
                }
                info!("使用指定的设备: {}", s);
                Ok(s)
            }
            None => {
                // 自动选择第一个设备
                let first_device = devices[0].clone();
                info!("自动选择第一个设备: {}", first_device);
                Ok(first_device)
            }
        }
    }

    /// 获取设备序列号
    ///
    /// # 返回
    ///
    /// 返回当前安装器关联的设备序列号
    pub fn device_serial(&self) -> &str {
        &self.device_serial
    }

    /// 获取 ADB 客户端引用
    ///
    /// # 返回
    ///
    /// 返回 ADB 客户端的引用
    #[allow(dead_code)]
    pub fn adb_client(&self) -> &Arc<AdbClient> {
        &self.adb_client
    }

    /// 创建 AtxAgentClient
    ///
    /// 内部辅助方法，用于创建 AtxAgentClient 实例
    async fn create_atx_client(&self) -> Result<uiautomator::atx_agent::AtxAgentClient> {
        uiautomator::atx_agent::AtxAgentClient::new(
            self.device_serial.clone(),
            self.adb_client.clone(),
        )
        .await
        .map_err(|e| anyhow!("创建 ATX-Agent 客户端失败: {}", e))
    }

    async fn wait_service_ready_with_fallback(
        &self,
        client: &uiautomator::atx_agent::AtxAgentClient,
        timeout: std::time::Duration,
    ) -> Result<()> {
        match client.wait_for_atx_agent_ready(Some(timeout)).await {
            Ok(()) => Ok(()),
            Err(wait_err) => match client.check_atx_agent_status().await {
                Ok(true) => Err(anyhow!("服务进程存在，但健康检查未就绪: {}", wait_err)),
                Ok(false) => Err(anyhow!("服务未就绪: {}", wait_err)),
                Err(status_err) => Err(anyhow!(
                    "服务未就绪且状态检查失败: {} (status: {})",
                    wait_err,
                    status_err
                )),
            },
        }
    }

    /// 通过设备端 atx-agent 二进制获取版本信息（HTTP /version 失败时兜底）
    async fn get_version_from_binary(&self) -> Result<String> {
        let output = self
            .adb_client
            .shell(
                &self.device_serial,
                "/data/local/tmp/atx-agent version",
                Some(std::time::Duration::from_secs(10)),
            )
            .await
            .map_err(|e| anyhow!("通过二进制获取版本失败: {}", e))?;

        output
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(str::to_string)
            .ok_or_else(|| anyhow!("二进制版本输出为空"))
    }

    /// 检查 ATX-Agent 是否已安装
    ///
    /// 通过 uiautomator 库检查设备上的 ATX-Agent 安装状态
    ///
    /// # 返回
    ///
    /// 返回是否已安装
    ///
    /// # 错误
    ///
    /// 如果检查过程中出现错误，返回错误
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator_cli::installer::Installer;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let installer = Installer::new(None).await?;
    ///     
    ///     if installer.check_installed().await? {
    ///         println!("ATX-Agent 已安装");
    ///     } else {
    ///         println!("ATX-Agent 未安装");
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn check_installed(&self) -> Result<bool> {
        debug!("检查 ATX-Agent 安装状态");

        // 创建 AtxAgentClient 来使用其检查功能
        let client = self.create_atx_client().await?;

        // 使用库的检查方法
        let installed = client
            .check_atx_agent_installed()
            .await
            .map_err(|e| anyhow!("检查安装状态失败: {}", e))?;

        if installed {
            info!("✓ ATX-Agent 已安装");
        } else {
            info!("✗ ATX-Agent 未安装");
        }

        Ok(installed)
    }

    /// 安装 ATX-Agent
    ///
    /// 执行完整的安装流程：
    /// 1. 检查是否已安装（如果 force=false）
    /// 2. 推送 atx-agent 二进制文件
    /// 3. 安装 UiAutomator APK
    /// 4. 启动 atx-agent 服务
    /// 5. 等待服务就绪
    ///
    /// # 参数
    ///
    /// * `force` - 是否强制重新安装。如果为 true，即使已安装也会重新安装
    ///
    /// # 返回
    ///
    /// 安装成功返回 Ok(())
    ///
    /// # 错误
    ///
    /// 如果安装过程中出现错误，返回错误
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator_cli::installer::Installer;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let installer = Installer::new(None).await?;
    ///     
    ///     // 正常安装（如果已安装则跳过）
    ///     installer.install(false).await?;
    ///     
    ///     // 强制重新安装
    ///     installer.install(true).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn install(&self, force: bool) -> Result<()> {
        info!(
            "🚀 开始安装 ATX-Agent (设备: {}, 强制: {})",
            self.device_serial, force
        );

        // 创建 AtxAgentClient 来使用其安装功能
        let client = self.create_atx_client().await?;

        // 如果不是强制安装，先检查是否已安装
        if !force {
            debug!("检查是否已安装...");
            let installed = client
                .check_atx_agent_installed()
                .await
                .map_err(|e| anyhow!("检查安装状态失败: {}", e))?;

            if installed {
                info!("✓ ATX-Agent 已安装，跳过安装");
                return Ok(());
            }
        }

        // 执行安装
        info!("📦 推送 atx-agent 二进制文件和安装 APK...");
        client
            .install(force)
            .await
            .map_err(|e| anyhow!("安装失败: {}", e))?;

        // 启动服务
        info!("🔧 启动 atx-agent 服务...");
        client
            .start_atx_agent()
            .await
            .map_err(|e| anyhow!("启动服务失败: {}", e))?;

        // 等待服务就绪
        info!("⏳ 等待服务就绪...");
        self.wait_service_ready_with_fallback(&client, std::time::Duration::from_secs(30))
            .await?;

        info!("✅ ATX-Agent 安装完成");
        Ok(())
    }

    /// 查询 ATX-Agent 服务状态
    ///
    /// 检查 ATX-Agent 服务是否正在运行，并获取版本信息。
    ///
    /// # 返回
    ///
    /// 返回服务状态，包括运行状态和版本信息
    ///
    /// # 错误
    ///
    /// 如果查询过程中出现错误，返回错误
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator_cli::installer::Installer;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let installer = Installer::new(None).await?;
    ///     
    ///     let status = installer.status().await?;
    ///     
    ///     if status.running {
    ///         println!("ATX-Agent 正在运行");
    ///         if let Some(version) = status.version {
    ///             println!("版本: {}", version);
    ///         }
    ///     } else {
    ///         println!("ATX-Agent 未运行");
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn status(&self) -> Result<ServiceStatus> {
        debug!("查询 ATX-Agent 服务状态");

        // 创建 AtxAgentClient 来使用其状态查询功能
        let client = self.create_atx_client().await?;

        // 检查服务是否正在运行
        let running = client
            .check_atx_agent_status()
            .await
            .map_err(|e| anyhow!("检查服务状态失败: {}", e))?;

        // 如果正在运行，获取版本信息
        let version = if running {
            match client.version().await {
                Ok(v) => {
                    info!("✓ ATX-Agent 正在运行，版本: {}", v);
                    Some(v)
                }
                Err(e) => {
                    warn!("通过 HTTP 获取版本失败，尝试二进制兜底: {}", e);
                    match self.get_version_from_binary().await {
                        Ok(v) => {
                            info!("✓ 通过二进制获取版本成功: {}", v);
                            Some(v)
                        }
                        Err(binary_err) => {
                            warn!("无法获取版本信息: {}", binary_err);
                            // 即使无法获取版本，也认为服务在运行
                            None
                        }
                    }
                }
            }
        } else {
            info!("✗ ATX-Agent 未运行");
            None
        };

        Ok(ServiceStatus { running, version })
    }

    /// 重启 ATX-Agent 服务
    ///
    /// 停止当前运行的服务，然后重新启动。确保服务最终处于运行状态。
    ///
    /// # 返回
    ///
    /// 重启成功返回 Ok(())
    ///
    /// # 错误
    ///
    /// 如果重启过程中出现错误，返回错误
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator_cli::installer::Installer;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let installer = Installer::new(None).await?;
    ///     
    ///     // 重启服务
    ///     installer.restart().await?;
    ///     
    ///     println!("服务已重启");
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn restart(&self) -> Result<()> {
        info!("🔄 重启 ATX-Agent 服务");

        // 创建 AtxAgentClient 来使用其重启功能
        let client = self.create_atx_client().await?;

        // 执行重启（某些设备上 HTTP 就绪探测会偶发超时，交给后续统一就绪检查兜底）
        if let Err(e) = client.restart_atx_agent().await {
            warn!("重启命令返回错误，继续进行就绪确认: {}", e);
        }

        // 等待服务就绪，确保重启操作的原子性
        info!("⏳ 等待服务就绪...");
        self.wait_service_ready_with_fallback(&client, std::time::Duration::from_secs(30))
            .await?;

        info!("✅ ATX-Agent 服务重启完成");
        Ok(())
    }

    /// 卸载 ATX-Agent
    ///
    /// 停止服务并删除所有相关文件：
    /// 1. 停止 atx-agent 服务
    /// 2. 删除 atx-agent 二进制文件
    /// 3. 卸载 UiAutomator APK
    /// 4. 卸载 UiAutomator Test APK
    ///
    /// # 返回
    ///
    /// 卸载成功返回 Ok(())
    ///
    /// # 错误
    ///
    /// 如果卸载过程中出现错误，返回错误
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator_cli::installer::Installer;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let installer = Installer::new(None).await?;
    ///     
    ///     // 卸载 ATX-Agent
    ///     installer.uninstall().await?;
    ///     
    ///     println!("ATX-Agent 已卸载");
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn uninstall(&self) -> Result<()> {
        info!("🗑️  卸载 ATX-Agent");

        // 1. 停止服务
        info!("停止 atx-agent 服务...");
        let stop_result = match self.create_atx_client().await {
            Ok(client) => client.stop_atx_agent().await.map_err(|e| anyhow!(e)),
            Err(create_err) => {
                warn!(
                    "创建 ATX-Agent 客户端失败，改用 ADB shell 停止进程: {}",
                    create_err
                );
                self.adb_client
                    .shell(
                        &self.device_serial,
                        "for p in $(ps | grep atx-agent | grep -v grep | awk '{print $2}'); do kill $p; done",
                        Some(std::time::Duration::from_secs(10)),
                    )
                    .await
                    .map(|_| ())
                    .map_err(|e| anyhow!("通过 ADB shell 停止进程失败: {}", e))
            }
        };
        if let Err(e) = stop_result {
            warn!("停止服务时出现错误（可能未运行）: {}", e);
            // 继续执行卸载，即使停止失败
        }

        // 2. 删除二进制文件
        info!("删除 atx-agent 二进制文件...");
        if let Err(e) = self
            .adb_client
            .shell(
                &self.device_serial,
                "rm -f /data/local/tmp/atx-agent",
                Some(std::time::Duration::from_secs(10)),
            )
            .await
        {
            warn!("删除二进制文件时出现错误: {}", e);
            // 继续执行
        }

        // 3. 卸载 UiAutomator APK
        info!("卸载 UiAutomator APK...");
        if let Err(e) = self
            .adb_client
            .shell(
                &self.device_serial,
                "pm uninstall com.github.uiautomator",
                Some(std::time::Duration::from_secs(30)),
            )
            .await
        {
            warn!("卸载 UiAutomator APK 时出现错误: {}", e);
            // 继续执行
        }

        // 4. 卸载 UiAutomator Test APK
        info!("卸载 UiAutomator Test APK...");
        if let Err(e) = self
            .adb_client
            .shell(
                &self.device_serial,
                "pm uninstall com.github.uiautomator.test",
                Some(std::time::Duration::from_secs(30)),
            )
            .await
        {
            warn!("卸载 UiAutomator Test APK 时出现错误: {}", e);
            // 继续执行
        }

        info!("✅ ATX-Agent 卸载完成");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试：device_serial() 方法应该返回正确的序列号
    #[test]
    fn test_device_serial_getter() {
        let adb_client = Arc::new(unsafe { std::mem::zeroed() });
        let installer = Installer {
            adb_client,
            device_serial: "test-device-123".to_string(),
        };

        assert_eq!(installer.device_serial(), "test-device-123");
    }
}
