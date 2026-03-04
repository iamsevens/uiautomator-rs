//! 错误处理模块
//!
//! 提供友好的错误消息和彩色输出

use colored::Colorize;
use std::fmt;

/// CLI 错误类型
#[allow(dead_code)]
#[derive(Debug, Clone)]
/// # Examples
///
/// ```
/// use uiautomator_cli::error::CliError;
///
/// let err = CliError::DeviceNotFound;
/// assert!(!err.to_string().is_empty());
/// ```
pub enum CliError {
    /// 未找到连接的设备
    DeviceNotFound,

    /// 检测到多个设备，需要指定序列号
    MultipleDevices(Vec<String>),

    /// 安装失败
    InstallFailed(String),

    /// ADB 错误
    AdbError(String),

    /// 服务错误
    ServiceError(String),

    /// 资源错误
    ResourceError(String),
}

/// 错误消息构建器，用于减少重复代码
#[allow(dead_code)]
struct ErrorMessageBuilder {
    title: String,
    reasons: Vec<String>,
    solutions: Vec<String>,
}

#[allow(dead_code)]
impl ErrorMessageBuilder {
    fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            reasons: Vec::new(),
            solutions: Vec::new(),
        }
    }

    fn add_reason(mut self, reason: impl Into<String>) -> Self {
        self.reasons.push(reason.into());
        self
    }

    fn add_solution(mut self, solution: impl Into<String>) -> Self {
        self.solutions.push(solution.into());
        self
    }

    fn build(&self) -> String {
        let mut message = self.title.clone();

        if !self.reasons.is_empty() {
            message.push_str("\n\n可能的原因:");
            for (i, reason) in self.reasons.iter().enumerate() {
                message.push_str(&format!("\n  {}. {}", i + 1, reason));
            }
        }

        if !self.solutions.is_empty() {
            message.push_str("\n\n解决方案:");
            for (i, solution) in self.solutions.iter().enumerate() {
                message.push_str(&format!("\n  {}. {}", i + 1, solution));
            }
        }

        message
    }

    fn build_colored(&self) -> String {
        let mut message = self.title.red().bold().to_string();

        if !self.reasons.is_empty() {
            message.push_str(&format!("\n\n{}", "可能的原因:".yellow().bold()));
            for (i, reason) in self.reasons.iter().enumerate() {
                message.push_str(&format!("\n  {}. {}", i + 1, reason.yellow()));
            }
        }

        if !self.solutions.is_empty() {
            message.push_str(&format!("\n\n{}", "解决方案:".green().bold()));
            for (i, solution) in self.solutions.iter().enumerate() {
                message.push_str(&format!("\n  {}. {}", i + 1, solution.green()));
            }
        }

        message
    }
}

impl CliError {
    /// 获取彩色格式的错误消息
    #[allow(dead_code)]
    /// # Examples
    ///
    /// ```
    /// use uiautomator_cli::error::CliError;
    ///
    /// let msg = CliError::DeviceNotFound.colored_message();
    /// assert!(!msg.is_empty());
    /// ```
    pub fn colored_message(&self) -> String {
        match self {
            CliError::DeviceNotFound => ErrorMessageBuilder::new("未找到连接的设备")
                .add_reason("设备未通过 USB 连接")
                .add_reason("ADB 服务未启动")
                .add_reason("设备未启用 USB 调试")
                .add_solution("检查 USB 连接")
                .add_solution("运行 'adb devices' 确认设备可见")
                .add_solution("在设备上启用 USB 调试模式")
                .build_colored(),

            CliError::MultipleDevices(devices) => {
                let device_list = devices
                    .iter()
                    .map(|d| format!("  - {}", d))
                    .collect::<Vec<_>>()
                    .join("\n");

                format!(
                    "{}\n\n{}\n{}\n\n{}\n{}",
                    "检测到多个设备".red().bold(),
                    "已连接的设备:".yellow().bold(),
                    device_list.yellow(),
                    "解决方案:".green().bold(),
                    format!(
                        "  使用 --serial 参数指定设备，例如: uiautomator --serial {} init",
                        devices[0]
                    )
                    .green(),
                )
            }

            CliError::InstallFailed(reason) => {
                ErrorMessageBuilder::new(format!("安装失败: {}", reason))
                    .add_reason("设备存储空间不足")
                    .add_reason("权限不足")
                    .add_solution("检查设备存储空间")
                    .add_solution("尝试使用 --force 选项重新安装")
                    .build_colored()
            }

            CliError::AdbError(error) => ErrorMessageBuilder::new(format!("ADB 错误: {}", error))
                .add_reason("ADB 服务未运行")
                .add_reason("设备连接不稳定")
                .add_solution("重启 ADB 服务: adb kill-server && adb start-server")
                .add_solution("重新连接设备")
                .build_colored(),

            CliError::ServiceError(error) => {
                ErrorMessageBuilder::new(format!("服务错误: {}", error))
                    .add_reason("ATX-Agent 未正确安装")
                    .add_reason("服务启动超时")
                    .add_solution("尝试重新安装: uiautomator init --force")
                    .add_solution("检查设备日志: adb logcat | grep atx-agent")
                    .build_colored()
            }

            CliError::ResourceError(error) => {
                ErrorMessageBuilder::new(format!("资源错误: {}", error))
                    .add_solution("请重新下载 CLI 工具或从源码重新编译")
                    .build_colored()
            }
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            CliError::DeviceNotFound => ErrorMessageBuilder::new("未找到连接的设备")
                .add_reason("设备未通过 USB 连接")
                .add_reason("ADB 服务未启动")
                .add_reason("设备未启用 USB 调试")
                .add_solution("检查 USB 连接")
                .add_solution("运行 'adb devices' 确认设备可见")
                .add_solution("在设备上启用 USB 调试模式")
                .build(),

            CliError::MultipleDevices(devices) => {
                let device_list = devices
                    .iter()
                    .map(|d| format!("  - {}", d))
                    .collect::<Vec<_>>()
                    .join("\n");

                format!(
                    "检测到多个设备\n\n已连接的设备:\n{}\n\n解决方案:\n  使用 --serial 参数指定设备，例如: uiautomator --serial {} init",
                    device_list,
                    devices[0]
                )
            }

            CliError::InstallFailed(reason) => {
                ErrorMessageBuilder::new(format!("安装失败: {}", reason))
                    .add_reason("设备存储空间不足")
                    .add_reason("权限不足")
                    .add_solution("检查设备存储空间")
                    .add_solution("尝试使用 --force 选项重新安装")
                    .build()
            }

            CliError::AdbError(error) => ErrorMessageBuilder::new(format!("ADB 错误: {}", error))
                .add_reason("ADB 服务未运行")
                .add_reason("设备连接不稳定")
                .add_solution("重启 ADB 服务: adb kill-server && adb start-server")
                .add_solution("重新连接设备")
                .build(),

            CliError::ServiceError(error) => {
                ErrorMessageBuilder::new(format!("服务错误: {}", error))
                    .add_reason("ATX-Agent 未正确安装")
                    .add_reason("服务启动超时")
                    .add_solution("尝试重新安装: uiautomator init --force")
                    .add_solution("检查设备日志: adb logcat | grep atx-agent")
                    .build()
            }

            CliError::ResourceError(error) => {
                ErrorMessageBuilder::new(format!("资源错误: {}", error))
                    .add_solution("请重新下载 CLI 工具或从源码重新编译")
                    .build()
            }
        };

        write!(f, "{}", message)
    }
}

impl std::error::Error for CliError {}

// 实现从其他错误类型的转换
impl From<anyhow::Error> for CliError {
    fn from(error: anyhow::Error) -> Self {
        CliError::AdbError(error.to_string())
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::AdbError(error.to_string())
    }
}
