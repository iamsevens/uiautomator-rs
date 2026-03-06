//! ADB 客户端封装
//!
//! 封装 adb_client 库,提供设备连接、命令执行和文件传输功能

use crate::error::{Error, Result};
use adb_client::server::ADBServer;
use adb_client::server_device::ADBServerDevice;
use adb_client::ADBDeviceExt;
use log::{debug, info, warn};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

/// ADB 客户端
///
/// 封装 adb_client 库,提供与 Android 设备通信的功能
/// # Examples
///
/// ```no_run
/// use uiautomator::adb::AdbClient;
///
/// #[tokio::main]
/// async fn main() -> uiautomator::Result<()> {
///     let client = AdbClient::new().await?;
///     let _devices = client.devices().await?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct AdbClient {
    // 使用 () 作为占位符,因为新版本的 adb_client 使用不同的连接模型
    _marker: (),
}

impl AdbClient {
    const MAX_CONCURRENT_TIMEOUT_SHELLS: usize = 4;

    /// shell v2 数据流中的 stdout/stderr/exit packet 类型
    const SHELL_V2_STDOUT: u8 = 1;
    const SHELL_V2_STDERR: u8 = 2;
    const SHELL_V2_EXIT: u8 = 3;

    fn timeout_shell_semaphore() -> &'static Arc<tokio::sync::Semaphore> {
        static SEMAPHORE: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
        SEMAPHORE.get_or_init(|| {
            Arc::new(tokio::sync::Semaphore::new(
                Self::MAX_CONCURRENT_TIMEOUT_SHELLS,
            ))
        })
    }

    /// 创建一个未做连通性检查的客户端。
    ///
    /// 主要用于构造仅依赖 HTTP JSON-RPC 的场景（如 mock 测试）。
    pub(crate) fn unchecked() -> Self {
        Self { _marker: () }
    }

    /// 创建新的 ADB 客户端
    ///
    /// 连接到本地 ADB 服务器 (默认端口 5037)
    ///
    /// # 错误
    ///
    /// 如果无法连接到 ADB 服务器,返回 `Error::Adb`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::adb::AdbClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let client = AdbClient::new().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> Result<Self> {
        debug!("正在连接到 ADB 服务器...");

        // 测试连接
        tokio::task::spawn_blocking(|| {
            let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037);
            let mut server = ADBServer::new(addr);

            // 尝试获取设备列表来验证连接
            server.devices().map_err(|e| {
                Error::Adb(format!(
                    "无法连接到 ADB 服务器: {}. 请确保 ADB 服务已启动",
                    e
                ))
            })?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Adb(format!("无法启动 ADB 连接任务: {}", e)))??;

        info!("成功连接到 ADB 服务器");
        Ok(Self { _marker: () })
    }

    /// 列出所有连接的设备
    ///
    /// # 返回
    ///
    /// 返回设备序列号列表
    ///
    /// # 错误
    ///
    /// 如果无法获取设备列表,返回 `Error::Adb`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::adb::AdbClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let client = AdbClient::new().await?;
    ///     let devices = client.devices().await?;
    ///     println!("找到 {} 个设备", devices.len());
    ///     for device in devices {
    ///         println!("  - {}", device);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn devices(&self) -> Result<Vec<String>> {
        debug!("正在获取设备列表...");

        let devices = tokio::task::spawn_blocking(|| {
            let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037);
            let mut server = ADBServer::new(addr);

            // 获取设备列表
            let devices = server
                .devices()
                .map_err(|e| Error::Adb(format!("无法获取设备列表: {}", e)))?;

            // 提取序列号
            let serials: Vec<String> = devices.into_iter().map(|d| d.identifier).collect();

            Ok::<Vec<String>, Error>(serials)
        })
        .await
        .map_err(|e| Error::Adb(format!("设备列表任务失败: {}", e)))??;

        info!("找到 {} 个设备", devices.len());
        Ok(devices)
    }

    /// 在设备上执行 shell 命令
    ///
    /// # 参数
    ///
    /// * `serial` - 设备序列号
    /// * `command` - 要执行的 shell 命令
    /// * `timeout` - 命令执行超时时间 (可选)
    ///
    /// # 返回
    ///
    /// 返回命令的标准输出
    ///
    /// # 错误
    ///
    /// 如果命令执行失败或超时,返回 `Error::Adb`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::adb::AdbClient;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let client = AdbClient::new().await?;
    ///     let output = client.shell("emulator-5554", "getprop ro.build.version.sdk", Some(Duration::from_secs(5))).await?;
    ///     println!("SDK 版本: {}", output.trim());
    ///     Ok(())
    /// }
    /// ```
    pub async fn shell(
        &self,
        serial: &str,
        command: &str,
        timeout: Option<Duration>,
    ) -> Result<String> {
        debug!("在设备 {} 上执行命令: {}", serial, command);

        let serial = serial.to_string();
        let command = command.to_string();

        let result = if let Some(timeout) = timeout {
            let semaphore = Arc::clone(Self::timeout_shell_semaphore());
            let permit = tokio::time::timeout(timeout, semaphore.acquire_owned())
                .await
                .map_err(|_| {
                    warn!(
                        "Shell 命令超时（等待可用工作线程）: serial={}, command={}",
                        serial, command
                    );
                    Error::Timeout
                })?
                .map_err(|_| Error::Internal("timeout shell semaphore closed".to_string()))?;

            let (tx, rx) = tokio::sync::oneshot::channel();
            let serial_clone = serial.clone();
            let command_clone = command.clone();
            std::thread::Builder::new()
                .name("uiautomator-adb-shell".to_string())
                .spawn(move || {
                    let _permit = permit;
                    let result = Self::run_shell_blocking(serial_clone, command_clone);
                    let _ = tx.send(result);
                })
                .map_err(|e| Error::Adb(format!("启动 Shell 工作线程失败: {}", e)))?;

            tokio::time::timeout(timeout, rx)
                .await
                .map_err(|_| {
                    warn!("Shell 命令超时: serial={}, command={}", serial, command);
                    Error::Timeout
                })?
                .map_err(|_| Error::Adb("Shell 工作线程异常退出".to_string()))??
        } else {
            tokio::task::spawn_blocking(move || Self::run_shell_blocking(serial, command))
                .await
                .map_err(|e| Error::Adb(format!("Shell 命令任务失败: {}", e)))??
        };

        debug!("命令执行成功,输出长度: {} 字节", result.len());
        Ok(result)
    }

    fn run_shell_blocking(serial: String, command: String) -> Result<String> {
        let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037);

        // 获取设备
        let mut device = ADBServerDevice::new(serial, Some(addr));

        // 执行 shell 命令并捕获输出
        let mut output = Vec::new();
        device
            .shell_command(&command, Some(&mut output), None)
            .map_err(|e| Error::Adb(format!("执行命令失败: {}", e)))?;

        // adb_client 在 shell_v2 模式下会返回原始 packet 流，这里统一解码为可读文本
        Self::decode_shell_output(&output)
            .map_err(|e| Error::Adb(format!("命令输出不是有效的 UTF-8: {}", e)))
    }

    /// 将 adb shell 返回的原始字节流解码为文本。
    ///
    /// 兼容两种格式：
    /// 1. 纯文本（legacy shell）
    /// 2. shell_v2 packet 流（stdout/stderr/exit code）
    fn decode_shell_output(raw: &[u8]) -> std::result::Result<String, std::string::FromUtf8Error> {
        if let Some(decoded) = Self::try_decode_shell_v2(raw) {
            return String::from_utf8(decoded);
        }

        String::from_utf8(raw.to_vec())
    }

    /// 尝试按 shell_v2 协议解码。
    ///
    /// 协议格式：1-byte channel + 4-byte LE payload len + payload
    /// - channel=1: stdout
    /// - channel=2: stderr
    /// - channel=3: exit code
    ///
    /// 解析失败时返回 None，让上层按纯文本处理。
    fn try_decode_shell_v2(raw: &[u8]) -> Option<Vec<u8>> {
        if raw.len() < 5 {
            return None;
        }

        let mut i = 0usize;
        let mut saw_shell_v2_packet = false;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        while i + 5 <= raw.len() {
            let channel = raw[i];
            let len = u32::from_le_bytes([raw[i + 1], raw[i + 2], raw[i + 3], raw[i + 4]]) as usize;
            i += 5;

            if i + len > raw.len() {
                return None;
            }

            let payload = &raw[i..i + len];
            i += len;

            match channel {
                Self::SHELL_V2_STDOUT => {
                    saw_shell_v2_packet = true;
                    stdout.extend_from_slice(payload);
                }
                Self::SHELL_V2_STDERR => {
                    saw_shell_v2_packet = true;
                    stderr.extend_from_slice(payload);
                }
                Self::SHELL_V2_EXIT => {
                    saw_shell_v2_packet = true;
                }
                // 非 shell_v2 channel，说明不是 packet 流
                _ => return None,
            }
        }

        // 仍有尾巴字节未按 packet 消费，说明不是合法 shell_v2 流
        if i != raw.len() || !saw_shell_v2_packet {
            return None;
        }

        if !stderr.is_empty() {
            if !stdout.is_empty() && !stdout.ends_with(b"\n") {
                stdout.push(b'\n');
            }
            stdout.extend_from_slice(&stderr);
        }

        Some(stdout)
    }

    /// 推送文件到设备
    ///
    /// # 参数
    ///
    /// * `serial` - 设备序列号
    /// * `local` - 本地文件路径
    /// * `remote` - 设备上的目标路径
    ///
    /// # 错误
    ///
    /// 如果文件传输失败,返回 `Error::Adb`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::adb::AdbClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let client = AdbClient::new().await?;
    ///     client.push("emulator-5554", "/tmp/test.txt", "/data/local/tmp/test.txt").await?;
    ///     println!("文件推送成功");
    ///     Ok(())
    /// }
    /// ```
    pub async fn push(&self, serial: &str, local: &str, remote: &str) -> Result<()> {
        info!("推送文件: {} -> {} (设备: {})", local, remote, serial);

        let serial = serial.to_string();
        let local = local.to_string();
        let remote = remote.to_string();

        tokio::task::spawn_blocking(move || {
            use std::fs::File;
            use std::io::Read;

            let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037);

            // 获取设备
            let mut device = ADBServerDevice::new(serial, Some(addr));

            // 读取本地文件
            let mut file = File::open(&local).map_err(Error::Io)?;

            let mut content = Vec::new();
            file.read_to_end(&mut content).map_err(Error::Io)?;

            // 推送文件
            device
                .push(&mut content.as_slice(), &remote)
                .map_err(|e| Error::Adb(format!("推送文件失败: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Adb(format!("文件推送任务失败: {}", e)))??;

        info!("文件推送成功");
        Ok(())
    }

    /// 从设备拉取文件
    ///
    /// # 参数
    ///
    /// * `serial` - 设备序列号
    /// * `remote` - 设备上的文件路径
    /// * `local` - 本地目标路径
    ///
    /// # 错误
    ///
    /// 如果文件传输失败,返回 `Error::Adb`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::adb::AdbClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let client = AdbClient::new().await?;
    ///     client.pull("emulator-5554", "/data/local/tmp/test.txt", "/tmp/test.txt").await?;
    ///     println!("文件拉取成功");
    ///     Ok(())
    /// }
    /// ```
    pub async fn pull(&self, serial: &str, remote: &str, local: &str) -> Result<()> {
        info!("拉取文件: {} -> {} (设备: {})", remote, local, serial);

        let serial = serial.to_string();
        let remote = remote.to_string();
        let local = local.to_string();

        tokio::task::spawn_blocking(move || {
            use std::fs::File;
            use std::io::Write;

            let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037);

            // 获取设备
            let mut device = ADBServerDevice::new(serial, Some(addr));

            // 拉取文件
            let mut content = Vec::new();
            device
                .pull(&remote, &mut content)
                .map_err(|e| Error::Adb(format!("拉取文件失败: {}", e)))?;

            // 写入本地文件
            let mut file = File::create(&local).map_err(Error::Io)?;

            file.write_all(&content).map_err(Error::Io)?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Adb(format!("文件拉取任务失败: {}", e)))??;

        info!("文件拉取成功");
        Ok(())
    }

    /// 建立端口转发
    ///
    /// # 参数
    ///
    /// * `serial` - 设备序列号
    /// * `local` - 本地端口
    /// * `remote` - 设备端口
    ///
    /// # 错误
    ///
    /// 如果端口转发失败,返回 `Error::Adb`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::adb::AdbClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let client = AdbClient::new().await?;
    ///     client.forward("emulator-5554", 9008, 9008).await?;
    ///     println!("端口转发成功");
    ///     Ok(())
    /// }
    /// ```
    pub async fn forward(&self, serial: &str, local: u16, remote: u16) -> Result<()> {
        info!("建立端口转发: {} -> {} (设备: {})", local, remote, serial);

        let serial = serial.to_string();

        tokio::task::spawn_blocking(move || {
            let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037);

            // 获取设备
            let mut device = ADBServerDevice::new(serial, Some(addr));

            // 建立端口转发
            device
                .forward(format!("tcp:{}", remote), format!("tcp:{}", local))
                .map_err(|e| Error::Adb(format!("端口转发失败: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Adb(format!("端口转发任务失败: {}", e)))??;

        info!("端口转发成功");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AdbClient;

    #[test]
    fn test_decode_shell_output_plain_text() {
        let raw = b"hello\n";
        let decoded = AdbClient::decode_shell_output(raw).expect("decode should succeed");
        assert_eq!(decoded, "hello\n");
    }

    #[test]
    fn test_decode_shell_output_shell_v2_stdout_exit() {
        // stdout "28\n" + exit code 0
        let raw = vec![1, 3, 0, 0, 0, b'2', b'8', b'\n', 3, 1, 0, 0, 0, 0];

        let decoded = AdbClient::decode_shell_output(&raw).expect("decode should succeed");
        assert_eq!(decoded, "28\n");
    }

    #[test]
    fn test_decode_shell_output_shell_v2_with_stderr() {
        // stdout "ok\n" + stderr "warn\n" + exit code 0
        let raw = vec![
            1, 3, 0, 0, 0, b'o', b'k', b'\n', 2, 5, 0, 0, 0, b'w', b'a', b'r', b'n', b'\n', 3, 1,
            0, 0, 0, 0,
        ];

        let decoded = AdbClient::decode_shell_output(&raw).expect("decode should succeed");
        assert_eq!(decoded, "ok\nwarn\n");
    }

    #[test]
    fn test_try_decode_shell_v2_invalid_stream_returns_none() {
        // 非法 packet，长度超过实际数据
        let raw = vec![1, 10, 0, 0, 0, b'a'];
        assert!(AdbClient::try_decode_shell_v2(&raw).is_none());
    }
}
