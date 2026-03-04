//! 错误类型定义
//!
//! 本模块定义了库中所有可能的错误类型，使用 `thiserror` 提供清晰的错误消息。
//!
//! # 错误分类
//!
//! ## 设备相关错误
//!
//! - [`DeviceNotFound`](Error::DeviceNotFound) - 未找到连接的设备
//! - [`MultipleDevicesFound`](Error::MultipleDevicesFound) - 发现多个设备，需要指定序列号
//! - [`DeviceOffline`](Error::DeviceOffline) - 设备离线或无响应
//! - [`DeviceConnection`](Error::DeviceConnection) - 设备连接错误（通用）
//!
//! ## 元素相关错误
//!
//! - [`ElementNotFound`](Error::ElementNotFound) - 元素未找到
//! - [`ElementTimeout`](Error::ElementTimeout) - 元素查找超时
//!
//! ## 应用相关错误
//!
//! - [`AppNotInstalled`](Error::AppNotInstalled) - 应用未安装
//! - [`AppNotRunning`](Error::AppNotRunning) - 应用未运行
//! - [`AppCrashed`](Error::AppCrashed) - 应用崩溃
//! - [`AppStartFailed`](Error::AppStartFailed) - 应用启动失败
//!
//! ## 网络和通信错误
//!
//! - [`JsonRpc`](Error::JsonRpc) - JSON-RPC 调用失败
//! - [`Http`](Error::Http) - HTTP 请求失败
//! - [`HttpTimeout`](Error::HttpTimeout) - HTTP 请求超时
//! - [`UiAutomatorNotConnected`](Error::UiAutomatorNotConnected) - UiAutomator 服务未连接
//!
//! ## 其他错误
//!
//! - [`Internal`](Error::Internal) - 内部错误（如锁中毒）
//!
//! # 基本用法
//!
//! ```no_run
//! use uiautomator::{Device, Error};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! match Device::connect(None).await {
//!     Ok(device) => println!("设备已连接"),
//!     Err(Error::DeviceNotFound) => eprintln!("未找到设备"),
//!     Err(e) => eprintln!("连接失败: {}", e),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # 错误码和分类
//!
//! 每个错误都有唯一的错误码和类别，用于日志聚合和监控：
//!
//! ```
//! use uiautomator::Error;
//!
//! let err = Error::DeviceNotFound;
//! println!("错误码: {}, 类别: {}", err.code(), err.category());
//! // 输出: 错误码: 1001, 类别: Device
//! ```
//!
//! 更多示例请参考 API 文档中各错误类型的说明。
//!
//! # 推荐的错误处理模式
//!
//! ```no_run
//! use std::time::Duration;
//! use uiautomator::{Device, Error};
//!
//! # async fn run() -> Result<(), Error> {
//! let device = Device::connect(None).await?;
//! match device.app_wait("com.example.app", Some(Duration::from_secs(5))).await {
//!     Ok(pid) => println!("app ready: {pid}"),
//!     Err(Error::AppNotInstalled(pkg)) => eprintln!("install app first: {pkg}"),
//!     Err(Error::AppCrashed(pkg)) => eprintln!("app crashed: {pkg}"),
//!     Err(Error::Timeout) => eprintln!("wait timeout"),
//!     Err(e) => return Err(e),
//! }
//! # Ok(())
//! # }
//! ```

use std::time::Duration;
use thiserror::Error;

/// 库的错误类型
///
/// # Examples
///
/// ```
/// use uiautomator::Error;
///
/// let err = Error::InvalidArgument("x must be >= 0".to_string());
/// assert_eq!(err.category(), "Other");
/// ```
#[derive(Debug, Error)]
pub enum Error {
    // ========== 设备相关错误 ==========
    /// 设备未找到
    #[error("设备未找到")]
    DeviceNotFound,

    /// 发现多个设备，需要指定设备序列号
    #[error("发现多个设备，请指定设备序列号")]
    MultipleDevicesFound,

    /// 设备离线
    #[error("设备 {0} 离线")]
    DeviceOffline(String),

    /// 设备连接错误（通用）
    #[error("设备连接错误: {0}")]
    DeviceConnection(String),

    // ========== 元素相关错误 ==========
    /// 元素未找到
    #[error("元素未找到: {selector}")]
    ElementNotFound {
        /// 选择器描述
        selector: String,
    },

    /// 元素查找超时
    #[error("元素查找超时 ({timeout:?}): {selector}")]
    ElementTimeout {
        /// 选择器描述
        selector: String,
        /// 超时时长
        timeout: Duration,
    },

    /// UI 对象未找到（保留向后兼容）
    #[deprecated(since = "0.1.0", note = "使用 ElementNotFound 代替")]
    #[error("UI 对象未找到: {0}")]
    UiObjectNotFound(String),

    // ========== 应用相关错误 ==========
    /// 应用未安装
    #[error("应用未安装: {0}")]
    AppNotInstalled(String),

    /// 应用未运行
    #[error("应用未运行: {0}")]
    AppNotRunning(String),

    /// 应用崩溃
    #[error("应用崩溃: {0}")]
    AppCrashed(String),

    /// 应用启动失败
    #[error("无法启动应用: {0}")]
    AppStartFailed(String),

    /// 应用未找到（保留向后兼容）
    #[deprecated(since = "0.1.0", note = "使用 AppNotInstalled 代替")]
    #[error("应用未找到: {0}")]
    AppNotFound(String),

    // ========== JSON-RPC 错误 ==========
    /// JSON-RPC 调用失败
    #[error("JSON-RPC 调用失败: {0}")]
    JsonRpc(String),

    /// JSON-RPC 响应解析失败
    #[error("JSON-RPC 响应解析失败: {0}")]
    JsonRpcParse(String),

    /// JSON-RPC 错误码（保留向后兼容）
    #[deprecated(since = "0.1.0", note = "使用 JsonRpc 或 JsonRpcParse 代替")]
    #[error("JSON-RPC 错误: code={0}, message={1}")]
    JsonRpcCode(i32, String),

    // ========== ADB 错误 ==========
    /// ADB 命令执行失败
    #[error("ADB 命令执行失败: {0}")]
    Adb(String),

    // ========== 网络错误 ==========
    /// HTTP 请求错误
    #[error("HTTP 请求失败: {0}")]
    Http(#[from] reqwest::Error),

    /// HTTP 超时
    #[error("HTTP 超时")]
    HttpTimeout,

    /// UiAutomator 服务未连接
    #[error("UiAutomator 服务未连接")]
    UiAutomatorNotConnected,

    // ========== 其他错误 ==========
    /// 操作超时
    #[error("操作超时")]
    Timeout,

    /// 序列化错误
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 图像处理错误
    #[error("图像处理错误: {0}")]
    Image(#[from] image::ImageError),

    /// 无效参数
    #[error("无效参数: {0}")]
    InvalidArgument(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),

    /// 未知错误
    #[error("未知错误: {0}")]
    Unknown(String),
}

#[allow(deprecated)]
impl Error {
    /// 获取错误码
    ///
    /// 返回一个唯一的错误码，用于日志聚合和监控。
    ///
    /// # 错误码范围和分配规则
    ///
    /// - 1000-1999: 设备相关错误
    /// - 2000-2999: 元素相关错误
    /// - 3000-3999: 应用相关错误
    /// - 4000-4999: 网络和通信错误
    ///   - 4000-4099: JSON-RPC 错误
    ///   - 4100-4199: ADB 错误
    ///   - 4200-4299: HTTP 错误
    /// - 5000-5999: 其他错误
    ///
    /// **分配规则**：
    /// - 每个范围内的错误从 x001 开始递增
    /// - 废弃的错误类型使用较大的编号（如 x003, x004, x005）
    /// - 预留编号空间用于未来扩展
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Error;
    ///
    /// let err = Error::DeviceNotFound;
    /// assert_eq!(err.code(), 1001);
    ///
    /// let err = Error::ElementNotFound { selector: "text=Button".to_string() };
    /// assert_eq!(err.code(), 2001);
    /// ```
    pub fn code(&self) -> u32 {
        match self {
            // 设备相关错误 (1000-1999)
            Error::DeviceNotFound => 1001,
            Error::MultipleDevicesFound => 1002,
            Error::DeviceOffline(_) => 1003,
            Error::DeviceConnection(_) => 1004,

            // 元素相关错误 (2000-2999)
            Error::ElementNotFound { .. } => 2001,
            Error::ElementTimeout { .. } => 2002,
            Error::UiObjectNotFound(_) => 2003,

            // 应用相关错误 (3000-3999)
            Error::AppNotInstalled(_) => 3001,
            Error::AppNotRunning(_) => 3002,
            Error::AppCrashed(_) => 3003,
            Error::AppStartFailed(_) => 3004,
            Error::AppNotFound(_) => 3005,

            // JSON-RPC 错误 (4000-4999)
            Error::JsonRpc(_) => 4001,
            Error::JsonRpcParse(_) => 4002,
            Error::JsonRpcCode(_, _) => 4003,

            // ADB 错误 (4100-4199)
            Error::Adb(_) => 4101,

            // 网络错误 (4200-4299)
            Error::Http(_) => 4201,
            Error::HttpTimeout => 4202,
            Error::UiAutomatorNotConnected => 4203,

            // 其他错误 (5000-5999)
            Error::Timeout => 5001,
            Error::Serialization(_) => 5002,
            Error::Io(_) => 5003,
            Error::Image(_) => 5004,
            Error::InvalidArgument(_) => 5005,
            Error::Internal(_) => 5006,
            Error::Unknown(_) => 5999,
        }
    }

    /// 获取错误类别
    ///
    /// 返回错误所属的类别，用于错误分类和统计。
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Error;
    ///
    /// let err = Error::DeviceNotFound;
    /// assert_eq!(err.category(), "Device");
    ///
    /// let err = Error::ElementNotFound { selector: "text=Button".to_string() };
    /// assert_eq!(err.category(), "Element");
    /// ```
    pub fn category(&self) -> &'static str {
        match self {
            Error::DeviceNotFound
            | Error::MultipleDevicesFound
            | Error::DeviceOffline(_)
            | Error::DeviceConnection(_) => "Device",

            Error::ElementNotFound { .. }
            | Error::ElementTimeout { .. }
            | Error::UiObjectNotFound(_) => "Element",

            Error::AppNotInstalled(_)
            | Error::AppNotRunning(_)
            | Error::AppCrashed(_)
            | Error::AppStartFailed(_)
            | Error::AppNotFound(_) => "Application",

            Error::JsonRpc(_)
            | Error::JsonRpcParse(_)
            | Error::JsonRpcCode(_, _)
            | Error::Adb(_)
            | Error::Http(_)
            | Error::HttpTimeout
            | Error::UiAutomatorNotConnected => "Network",

            Error::Timeout
            | Error::Serialization(_)
            | Error::Io(_)
            | Error::Image(_)
            | Error::InvalidArgument(_)
            | Error::Internal(_)
            | Error::Unknown(_) => "Other",
        }
    }
}

/// 库的 Result 类型别名
///
/// # Examples
///
/// ```
/// use uiautomator::{Error, Result};
///
/// fn validate_positive(v: i32) -> Result<i32> {
///     if v > 0 {
///         Ok(v)
///     } else {
///         Err(Error::InvalidArgument("v must be positive".to_string()))
///     }
/// }
///
/// assert!(validate_positive(1).is_ok());
/// assert!(validate_positive(0).is_err());
/// ```
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    // ========== 设备相关错误测试 ==========

    #[test]
    fn test_device_not_found() {
        let err = Error::DeviceNotFound;
        assert_eq!(err.to_string(), "设备未找到");
    }

    #[test]
    fn test_multiple_devices_found() {
        let err = Error::MultipleDevicesFound;
        assert_eq!(err.to_string(), "发现多个设备，请指定设备序列号");
    }

    #[test]
    fn test_device_offline() {
        let err = Error::DeviceOffline("emulator-5554".to_string());
        assert_eq!(err.to_string(), "设备 emulator-5554 离线");
        assert_eq!(err.code(), 1003);
        assert_eq!(err.category(), "Device");
    }

    #[test]
    fn test_device_connection() {
        let err = Error::DeviceConnection("无法连接".to_string());
        assert_eq!(err.to_string(), "设备连接错误: 无法连接");
    }

    // ========== 元素相关错误测试 ==========

    #[test]
    fn test_element_not_found() {
        let err = Error::ElementNotFound {
            selector: "text=Settings".to_string(),
        };
        assert_eq!(err.to_string(), "元素未找到: text=Settings");
    }

    #[test]
    fn test_element_timeout() {
        let err = Error::ElementTimeout {
            selector: "text=Button".to_string(),
            timeout: Duration::from_secs(10),
        };
        let err_str = err.to_string();
        assert!(err_str.contains("元素查找超时"));
        assert!(err_str.contains("text=Button"));
        assert!(err_str.contains("10s"));
    }

    // ========== 应用相关错误测试 ==========

    #[test]
    fn test_app_not_installed() {
        let err = Error::AppNotInstalled("com.example.app".to_string());
        assert_eq!(err.to_string(), "应用未安装: com.example.app");
    }

    #[test]
    fn test_app_not_running() {
        let err = Error::AppNotRunning("com.example.app".to_string());
        assert_eq!(err.to_string(), "应用未运行: com.example.app");
    }

    #[test]
    fn test_app_crashed() {
        let err = Error::AppCrashed("com.example.app".to_string());
        assert_eq!(err.to_string(), "应用崩溃: com.example.app");
    }

    #[test]
    fn test_app_start_failed() {
        let err = Error::AppStartFailed("com.example.app".to_string());
        assert_eq!(err.to_string(), "无法启动应用: com.example.app");
    }

    // ========== JSON-RPC 错误测试 ==========

    #[test]
    fn test_jsonrpc_error() {
        let err = Error::JsonRpc("method not found".to_string());
        assert_eq!(err.to_string(), "JSON-RPC 调用失败: method not found");
    }

    #[test]
    fn test_jsonrpc_parse_error() {
        let err = Error::JsonRpcParse("invalid json".to_string());
        assert_eq!(err.to_string(), "JSON-RPC 响应解析失败: invalid json");
        assert_eq!(err.code(), 4002);
        assert_eq!(err.category(), "Network");
    }

    // ========== ADB 错误测试 ==========

    #[test]
    fn test_adb_error() {
        let err = Error::Adb("命令失败".to_string());
        assert_eq!(err.to_string(), "ADB 命令执行失败: 命令失败");
    }

    // ========== 其他错误测试 ==========

    #[test]
    fn test_http_timeout() {
        let err = Error::HttpTimeout;
        assert_eq!(err.to_string(), "HTTP 超时");
    }

    #[test]
    fn test_uiautomator_not_connected() {
        let err = Error::UiAutomatorNotConnected;
        assert_eq!(err.to_string(), "UiAutomator 服务未连接");
    }

    #[test]
    fn test_timeout() {
        let err = Error::Timeout;
        assert_eq!(err.to_string(), "操作超时");
    }

    #[test]
    fn test_invalid_argument() {
        let err = Error::InvalidArgument("坐标超出范围".to_string());
        assert_eq!(err.to_string(), "无效参数: 坐标超出范围");
    }

    #[test]
    fn test_internal_error() {
        let err = Error::Internal("Settings lock poisoned".to_string());
        assert_eq!(err.to_string(), "内部错误: Settings lock poisoned");
    }

    #[test]
    fn test_unknown_error() {
        let err = Error::Unknown("未知问题".to_string());
        assert_eq!(err.to_string(), "未知错误: 未知问题");
    }

    // ========== 错误转换测试 ==========

    #[test]
    fn test_error_from_serde_json() {
        let json_err = serde_json::from_str::<i32>("invalid").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件未找到");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_result_type() {
        fn returns_error() -> Result<i32> {
            Err(Error::Timeout)
        }

        let result = returns_error();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Timeout));
    }

    #[test]
    fn test_error_debug() {
        let err = Error::DeviceConnection("测试".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("DeviceConnection"));
        assert!(debug_str.contains("测试"));
    }

    // ========== 错误上下文测试 ==========

    #[test]
    fn test_element_not_found_contains_selector() {
        let err = Error::ElementNotFound {
            selector: "resourceId=com.example:id/button".to_string(),
        };
        let err_str = err.to_string();
        assert!(err_str.contains("resourceId=com.example:id/button"));
    }

    #[test]
    fn test_element_timeout_contains_both_selector_and_timeout() {
        let err = Error::ElementTimeout {
            selector: "text=Login".to_string(),
            timeout: Duration::from_secs(5),
        };
        let err_str = err.to_string();
        assert!(err_str.contains("text=Login"));
        assert!(err_str.contains("5s"));
    }

    #[test]
    fn test_device_offline_contains_serial() {
        let err = Error::DeviceOffline("192.168.1.100:5555".to_string());
        let err_str = err.to_string();
        assert!(err_str.contains("192.168.1.100:5555"));
    }

    // ========== 错误码测试 ==========

    #[test]
    fn test_error_codes_device() {
        assert_eq!(Error::DeviceNotFound.code(), 1001);
        assert_eq!(Error::MultipleDevicesFound.code(), 1002);
        assert_eq!(Error::DeviceOffline("test".to_string()).code(), 1003);
        assert_eq!(Error::DeviceConnection("test".to_string()).code(), 1004);
    }

    #[test]
    fn test_error_codes_element() {
        assert_eq!(
            Error::ElementNotFound {
                selector: "test".to_string()
            }
            .code(),
            2001
        );
        assert_eq!(
            Error::ElementTimeout {
                selector: "test".to_string(),
                timeout: Duration::from_secs(5)
            }
            .code(),
            2002
        );
    }

    #[test]
    fn test_error_codes_application() {
        assert_eq!(Error::AppNotInstalled("test".to_string()).code(), 3001);
        assert_eq!(Error::AppNotRunning("test".to_string()).code(), 3002);
        assert_eq!(Error::AppCrashed("test".to_string()).code(), 3003);
        assert_eq!(Error::AppStartFailed("test".to_string()).code(), 3004);
    }

    #[test]
    fn test_error_codes_network() {
        assert_eq!(Error::JsonRpc("test".to_string()).code(), 4001);
        assert_eq!(Error::JsonRpcParse("test".to_string()).code(), 4002);
        assert_eq!(Error::Adb("test".to_string()).code(), 4101);
        assert_eq!(Error::HttpTimeout.code(), 4202);
        assert_eq!(Error::UiAutomatorNotConnected.code(), 4203);
    }

    #[test]
    fn test_error_codes_other() {
        assert_eq!(Error::Timeout.code(), 5001);
        assert_eq!(Error::InvalidArgument("test".to_string()).code(), 5005);
        assert_eq!(Error::Internal("test".to_string()).code(), 5006);
        assert_eq!(Error::Unknown("test".to_string()).code(), 5999);
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(Error::DeviceNotFound.category(), "Device");
        assert_eq!(
            Error::ElementNotFound {
                selector: "test".to_string()
            }
            .category(),
            "Element"
        );
        assert_eq!(
            Error::AppNotInstalled("test".to_string()).category(),
            "Application"
        );
        assert_eq!(
            Error::AppNotRunning("test".to_string()).category(),
            "Application"
        );
        assert_eq!(Error::JsonRpc("test".to_string()).category(), "Network");
        assert_eq!(Error::Timeout.category(), "Other");
        assert_eq!(Error::Internal("test".to_string()).category(), "Other");
    }

    #[test]
    fn test_error_code_uniqueness() {
        // 确保错误码在各自范围内是唯一的
        let device_codes = vec![
            Error::DeviceNotFound.code(),
            Error::MultipleDevicesFound.code(),
            Error::DeviceOffline("".to_string()).code(),
            Error::DeviceConnection("".to_string()).code(),
        ];
        let unique_device: std::collections::HashSet<_> = device_codes.iter().collect();
        assert_eq!(
            device_codes.len(),
            unique_device.len(),
            "设备错误码应该唯一"
        );

        let element_codes = vec![
            Error::ElementNotFound {
                selector: "".to_string(),
            }
            .code(),
            Error::ElementTimeout {
                selector: "".to_string(),
                timeout: Duration::from_secs(1),
            }
            .code(),
        ];
        let unique_element: std::collections::HashSet<_> = element_codes.iter().collect();
        assert_eq!(
            element_codes.len(),
            unique_element.len(),
            "元素错误码应该唯一"
        );
    }
}
