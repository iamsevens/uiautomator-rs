//! # uiautomator
//!
//! 使用 Rust 实现的 Android UI 自动化测试库
//!
//! 这是一个复刻 Python uiautomator2 的 Rust 库，提供类型安全、高性能的 Android 自动化测试能力。
//! 通过 ADB 与 Android 设备通信，使用 JSON-RPC 协议调用设备端的 UiAutomator 服务。
//!
//! ## 特性
//!
//! - 🚀 **高性能**: 基于 Rust 和 Tokio 异步运行时
//! - 🔒 **类型安全**: 利用 Rust 的类型系统避免运行时错误
//! - 🔄 **异步支持**: 支持高并发场景下同时控制多个设备
//! - 📱 **完整功能**: 支持元素定位、手势操作、应用管理、截图等
//! - 🛡️ **错误恢复**: 自动重试和服务恢复机制
//!
//! ## 快速开始
//!
//! ### 基础使用
//!
//! ```no_run
//! use uiautomator::{Device, Selector, Key};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> uiautomator::Result<()> {
//!     // 初始化日志系统
//!     uiautomator::init_logger();
//!     
//!     // 连接到设备（自动选择唯一设备）
//!     let device = Device::connect(None).await?;
//!     
//!     // 获取设备信息
//!     let info = device.info().await?;
//!     println!("设备: {}x{}", info.display_width, info.display_height);
//!     
//!     // 点击坐标
//!     device.click(100, 200).await?;
//!     
//!     // 按键操作
//!     device.press(Key::Home).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### 元素定位和操作
//!
//! ```no_run
//! # use uiautomator::{Device, Selector};
//! # async fn example(device: &Device) -> uiautomator::Result<()> {
//! // 通过文本定位元素并点击
//! device.find(Selector::new().text("设置"))
//!     .click(None, None).await?;
//!
//! // 组合条件定位
//! device.find(Selector::new()
//!     .text("确定")
//!     .class_name("android.widget.Button")
//!     .clickable(true))
//!     .click(None, None).await?;
//!
//! // 等待元素出现
//! device.find(Selector::new().text("加载完成"))
//!     .wait(Some(std::time::Duration::from_secs(10))).await?;
//!
//! // 获取元素文本
//! let text = device.find(Selector::new().resource_id("com.example:id/title"))
//!     .get_text().await?;
//! println!("标题: {}", text);
//! # Ok(())
//! # }
//! ```
//!
//! ### 应用管理
//!
//! ```no_run
//! # use uiautomator::Device;
//! # async fn example(device: &Device) -> uiautomator::Result<()> {
//! // 启动应用
//! device.app_start("com.android.settings", None).await?;
//!
//! // 等待应用启动
//! device.app_wait("com.android.settings",
//!     Some(std::time::Duration::from_secs(10))).await?;
//!
//! // 获取当前应用
//! let app = device.app_current().await?;
//! println!("当前应用: {}", app.package);
//!
//! // 停止应用
//! device.app_stop("com.android.settings").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### 截图
//!
//! ```no_run
//! # use uiautomator::Device;
//! # async fn example(device: &Device) -> uiautomator::Result<()> {
//! // 截图并保存
//! device.screenshot_to_file("screenshot.png").await?;
//!
//! // 获取图像数据
//! let image = device.screenshot().await?;
//! println!("截图尺寸: {}x{}", image.width(), image.height());
//! # Ok(())
//! # }
//! ```
//!
//! ## 从 Python uiautomator2 迁移
//!
//! 本库的 API 设计尽可能与 Python 版本保持一致，主要差异：
//!
//! - 所有 I/O 操作都是异步的，需要 `.await?`
//! - 使用 Rust 的类型系统（如 `Duration` 而不是浮点数秒）
//! - 使用 `Result<T>` 和 `?` 操作符处理错误
//! - 可选参数使用 `Option<T>`
//!
//! ## 服务模式
//!
//! 支持两种设备端服务模式：
//!
//! - **Direct 模式**（当前默认）: 快速启动，适合开发测试
//! - **ATX-Agent 模式**（未来默认）: 生产级稳定性，推荐长期运行
//!
//! ```no_run
//! # use uiautomator::{Device, ServerMode};
//! # async fn example() -> uiautomator::Result<()> {
//! // 使用默认模式
//! let device = Device::connect(None).await?;
//!
//! // 快速模式（Direct）
//! let device = Device::connect_quick(None).await?;
//!
//! // 显式指定模式
//! let device = Device::connect_with_mode(None, ServerMode::Direct).await?;
//! # Ok(())
//! # }
//! ```

// ============================================================================
// 模块声明
// ============================================================================

/// 错误类型和结果类型定义
pub mod error;

/// 设备连接和操作的核心实现
pub mod device;

/// ADB 客户端封装
pub mod adb;

/// JSON-RPC 客户端实现
pub mod jsonrpc;

/// ATX-Agent REST API 客户端
pub mod atx_agent;

/// UI 元素选择器
pub mod selector;

/// UI 对象操作
pub mod uiobject;

/// 数据模型定义
pub mod models;

/// 按键枚举
pub mod key;

/// 配置和设置
pub mod settings;

// ============================================================================
// 公共 API 导出
// ============================================================================

// 核心类型
pub use device::{Device, ServerMode};
pub use error::{Error, Result};
pub use key::Key;
pub use selector::Selector;
pub use settings::Settings;
pub use uiobject::UiObject;

// 客户端（通常不需要直接使用，但导出以便高级用户使用）
pub use adb::AdbClient;
pub use atx_agent::AtxAgentClient;
pub use jsonrpc::JsonRpcClient;

// 数据模型
pub use models::{AppInfo, Coord, DeviceInfo, ElementInfo, Rect};

// ============================================================================
// 工具函数
// ============================================================================

/// 初始化日志系统
///
/// 这个函数会初始化 env_logger，允许通过 RUST_LOG 环境变量控制日志级别。
///
/// # 示例
///
/// ```no_run
/// uiautomator::init_logger();
/// ```
///
/// 然后可以通过设置环境变量来控制日志级别：
/// - `RUST_LOG=debug` - 显示调试信息
/// - `RUST_LOG=info` - 显示一般信息（默认）
/// - `RUST_LOG=warn` - 只显示警告
/// - `RUST_LOG=error` - 只显示错误
///
/// # 注意
///
/// 这个函数只能调用一次。如果需要自定义日志级别，请使用 [`init_logger_with_level`]。
pub fn init_logger() {
    let _ = env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .try_init();
}

/// 使用自定义日志级别初始化日志系统
///
/// # 参数
///
/// * `level` - 日志级别过滤器
///
/// # 示例
///
/// ```no_run
/// use log::LevelFilter;
/// uiautomator::init_logger_with_level(LevelFilter::Debug);
/// ```
///
/// # 注意
///
/// 这个函数只能调用一次。重复调用会被忽略。
pub fn init_logger_with_level(level: log::LevelFilter) {
    let _ = env_logger::Builder::from_default_env()
        .filter_level(level)
        .try_init();
}
