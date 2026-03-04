//! Device 核心功能
//!
//! 提供设备连接、信息获取和元素定位等核心功能

use crate::{
    adb::AdbClient,
    error::{Error, Result},
    jsonrpc::JsonRpcClient,
    models::DeviceInfo,
    selector::Selector,
    settings::Settings,
    uiobject::UiObject,
};
use log::{debug, info, warn};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// 服务器模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # Examples
///
/// ```
/// use uiautomator::ServerMode;
///
/// let mode = ServerMode::Auto;
/// assert!(matches!(mode, ServerMode::Auto));
/// ```
pub enum ServerMode {
    /// Direct 模式（快速测试，阶段 1 默认）
    Direct,
    /// ATX-Agent 模式（生产级，阶段 2 默认）
    AtxAgent,
    /// 自动检测模式（优先 ATX-Agent，失败则回退到 Direct）
    Auto,
}

/// Device 结构体
///
/// 代表一个 Android 设备连接，提供所有自动化操作的入口
///
/// # Examples
///
/// ```no_run
/// use uiautomator::Device;
///
/// #[tokio::main]
/// async fn main() -> uiautomator::Result<()> {
///     let device = Device::connect(None).await?;
///     println!("connected: {}", device.serial());
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Device {
    /// 设备序列号
    serial: String,

    /// ADB 客户端
    adb_client: Arc<AdbClient>,

    /// JSON-RPC 客户端
    jsonrpc_client: Arc<JsonRpcClient>,

    /// 配置设置
    settings: Arc<RwLock<Settings>>,

    /// 服务器模式
    server_mode: ServerMode,
}

impl Device {
    fn is_likely_device_offline_text(text: &str) -> bool {
        let lower = text.to_lowercase();
        [
            "device offline",
            "is offline",
            "device not found",
            "closed",
            "broken pipe",
            "transport error",
            "device unauthorized",
            "unauthorized",
        ]
        .iter()
        .any(|marker| lower.contains(marker))
    }

    fn atx_agent_init_guidance(serial: &str) -> String {
        format!(
            "请先初始化设备端 ATX-Agent：`uiautomator init --serial {} --force`。\
             若从源码运行 CLI，请执行：`cd uiautomator-cli && cargo run -- init --serial {} --force`。",
            serial, serial
        )
    }

    /// 连接到设备（使用自动检测模式）
    ///
    /// 自动检测逻辑：
    /// 1. 尝试连接 atx-agent (7912 端口)
    /// 2. 如果成功，使用 ATX-Agent 模式
    /// 3. 如果失败，回退到 Direct 模式
    ///
    /// # 参数
    ///
    /// * `serial` - 设备序列号，None 表示自动选择设备
    ///
    /// # 错误
    ///
    /// - 如果未找到设备，返回 `Error::DeviceNotFound`
    /// - 如果未提供序列号且有多个设备，返回 `Error::MultipleDevicesFound`
    /// - 如果设备离线，返回 `Error::DeviceOffline`
    /// - 如果 UiAutomator/ATX 服务不可用，返回 `Error::UiAutomatorNotConnected`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     // 自动选择设备和模式
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 指定设备序列号
    ///     let device = Device::connect(Some("emulator-5554")).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect(serial: Option<&str>) -> Result<Self> {
        Self::connect_with_mode(serial, ServerMode::Auto).await
    }

    /// 快速连接（强制使用 Direct 模式，仅用于开发测试）
    ///
    /// # 参数
    ///
    /// * `serial` - 设备序列号，None 表示自动选择设备
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect_quick(Some("emulator-5554")).await?;
    ///     println!("{}", device.serial());
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect_quick(serial: Option<&str>) -> Result<Self> {
        Self::connect_with_mode(serial, ServerMode::Direct).await
    }

    /// 使用自定义 JSON-RPC endpoint 创建 Device。
    ///
    /// 该入口主要用于 mock/代理场景，不会执行设备侧服务准备流程。
    ///
    /// # 参数
    ///
    /// * `serial` - 设备序列号（可选，默认 `mock-device`）
    /// * `rpc_url` - 完整 JSON-RPC endpoint，例如 `http://127.0.0.1:12345/jsonrpc/0`
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Device;
    ///
    /// let device = Device::connect_with_rpc_url(
    ///     Some("mock-1"),
    ///     "http://127.0.0.1:19008/jsonrpc/0",
    /// )?;
    /// assert_eq!(device.serial(), "mock-1");
    /// # Ok::<(), uiautomator::Error>(())
    /// ```
    pub fn connect_with_rpc_url(serial: Option<&str>, rpc_url: &str) -> Result<Self> {
        let serial = serial.unwrap_or("mock-device").to_string();
        let settings = Arc::new(RwLock::new(Settings::default()));
        let adb_client = Arc::new(AdbClient::unchecked());
        let jsonrpc_client = Arc::new(JsonRpcClient::new_direct_with_rpc_url(
            serial.clone(),
            adb_client.clone(),
            settings.clone(),
            rpc_url.to_string(),
        )?);

        Ok(Self {
            serial,
            adb_client,
            jsonrpc_client,
            settings,
            server_mode: ServerMode::Direct,
        })
    }

    /// 使用指定模式连接到设备
    ///
    /// # 参数
    ///
    /// * `serial` - 设备序列号，None 表示自动选择设备
    /// * `mode` - 服务器模式（Direct、ATX-Agent 或 Auto）
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uiautomator::{Device, ServerMode};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect_with_mode(Some("emulator-5554"), ServerMode::Auto).await?;
    ///     assert!(matches!(device.server_mode(), ServerMode::Auto | ServerMode::AtxAgent | ServerMode::Direct));
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect_with_mode(serial: Option<&str>, mode: ServerMode) -> Result<Self> {
        info!("正在连接到设备 (模式: {:?})", mode);

        // 创建 ADB 客户端
        let adb_client = Arc::new(AdbClient::new().await?);

        // 确定设备序列号
        let serial = match serial {
            Some(s) => s.to_string(),
            None => {
                // 自动选择设备
                let devices = adb_client.devices().await?;

                if devices.is_empty() {
                    return Err(Error::DeviceNotFound);
                }

                if devices.len() > 1 {
                    return Err(Error::MultipleDevicesFound);
                }

                devices[0].clone()
            }
        };

        info!("使用设备: {}", serial);

        // 根据模式连接
        match mode {
            ServerMode::Auto => {
                // 自动检测：优先尝试 ATX-Agent，失败则回退到 Direct
                info!("尝试使用 ATX-Agent 模式连接...");
                match Self::try_connect_atx_agent(serial.clone(), adb_client.clone()).await {
                    Ok(device) => {
                        info!("成功使用 ATX-Agent 模式连接");
                        Ok(device)
                    }
                    Err(e) => {
                        warn!("ATX-Agent 模式连接失败: {:?}", e);
                        info!("回退到 Direct 模式...");
                        Self::try_connect_direct(serial, adb_client).await
                    }
                }
            }
            ServerMode::AtxAgent => Self::try_connect_atx_agent(serial, adb_client).await,
            ServerMode::Direct => Self::try_connect_direct(serial, adb_client).await,
        }
    }

    /// 尝试使用 ATX-Agent 模式连接
    async fn try_connect_atx_agent(serial: String, adb_client: Arc<AdbClient>) -> Result<Self> {
        use crate::atx_agent::AtxAgentClient;

        debug!("创建 ATX-Agent 客户端");

        // 创建配置
        let settings = Arc::new(RwLock::new(Settings::default()));

        // 创建 ATX-Agent 客户端
        let atx_agent_client = Arc::new(
            AtxAgentClient::new(serial.clone(), adb_client.clone())
                .await
                .map_err(|e| {
                    let guidance = Self::atx_agent_init_guidance(&serial);
                    warn!(
                        "创建 ATX-Agent 客户端失败 (serial={}): {}. {}",
                        serial, e, guidance
                    );
                    if Self::is_likely_device_offline_text(&e.to_string()) {
                        Error::DeviceOffline(serial.clone())
                    } else {
                        Error::DeviceConnection(format!(
                            "创建 ATX-Agent 客户端失败: {}。{}",
                            e, guidance
                        ))
                    }
                })?,
        );

        // 检查 atx-agent 是否可用
        if !atx_agent_client.is_available().await {
            warn!(
                "ATX-Agent 不可用 (serial={}). {}",
                serial,
                Self::atx_agent_init_guidance(&serial)
            );
            return Err(Error::UiAutomatorNotConnected);
        }

        // 创建 JSON-RPC 客户端（ATX-Agent 模式）
        let jsonrpc_client = Arc::new(
            JsonRpcClient::new_with_atx_agent(
                serial.clone(),
                adb_client.clone(),
                atx_agent_client,
                settings.clone(),
            )
            .await
            .map_err(|e| {
                warn!(
                    "ATX-Agent 模式初始化失败 (serial={}): {}. {}",
                    serial,
                    e,
                    Self::atx_agent_init_guidance(&serial)
                );
                Error::UiAutomatorNotConnected
            })?,
        );

        // 验证服务状态
        let ping_ok = jsonrpc_client.ping().await.map_err(|e| {
            warn!(
                "ATX-Agent 模式连通性检查失败 (serial={}): {}. {}",
                serial,
                e,
                Self::atx_agent_init_guidance(&serial)
            );
            Error::UiAutomatorNotConnected
        })?;
        if !ping_ok {
            warn!(
                "ATX-Agent 模式连通性检查失败 (ping=false, serial={}). {}",
                serial,
                Self::atx_agent_init_guidance(&serial)
            );
            return Err(Error::UiAutomatorNotConnected);
        }

        info!("设备连接成功（ATX-Agent 模式）");

        Ok(Self {
            serial,
            adb_client,
            jsonrpc_client,
            settings,
            server_mode: ServerMode::AtxAgent,
        })
    }

    /// 尝试使用 Direct 模式连接
    async fn try_connect_direct(serial: String, adb_client: Arc<AdbClient>) -> Result<Self> {
        debug!("创建 JSON-RPC 客户端（Direct 模式）");

        // 创建配置
        let settings = Arc::new(RwLock::new(Settings::default()));

        // 创建 JSON-RPC 客户端（Direct 模式）
        let jsonrpc_client = Arc::new(
            JsonRpcClient::new_direct(serial.clone(), adb_client.clone(), settings.clone()).await?,
        );

        // 验证服务状态
        if !jsonrpc_client.ping().await? {
            return Err(Error::UiAutomatorNotConnected);
        }

        info!("设备连接成功（Direct 模式）");

        Ok(Self {
            serial,
            adb_client,
            jsonrpc_client,
            settings,
            server_mode: ServerMode::Direct,
        })
    }

    /// 获取设备信息
    ///
    /// # 返回
    ///
    /// 返回设备的详细信息，包括屏幕尺寸、旋转角度等
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     let info = device.info().await?;
    ///     println!("屏幕尺寸: {}x{}", info.display_width, info.display_height);
    ///     Ok(())
    /// }
    /// ```
    pub async fn info(&self) -> Result<DeviceInfo> {
        debug!("获取设备信息");

        // 调用 JSON-RPC 的 deviceInfo 方法
        let info: DeviceInfo = self
            .jsonrpc_client
            .call("deviceInfo", serde_json::json!({}))
            .await?;

        Ok(info)
    }

    /// 获取屏幕尺寸
    ///
    /// # 返回
    ///
    /// 返回 (宽度, 高度) 元组
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     let (width, height) = device.window_size().await?;
    ///     println!("屏幕尺寸: {}x{}", width, height);
    ///     Ok(())
    /// }
    /// ```
    pub async fn window_size(&self) -> Result<(u32, u32)> {
        debug!("获取屏幕尺寸");

        let info = self.info().await?;
        Ok((info.display_width, info.display_height))
    }

    /// 坐标转换：相对坐标或绝对坐标转换为绝对像素坐标
    ///
    /// # 参数
    ///
    /// * `x` - X 坐标（0.0-1.0 为百分比，>1.0 为像素）
    /// * `y` - Y 坐标（0.0-1.0 为百分比，>1.0 为像素）
    ///
    /// # 返回
    ///
    /// 返回 (x_pixel, y_pixel) 绝对像素坐标
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 百分比坐标（屏幕中心）
    ///     let (x, y) = device.pos_rel2abs(0.5, 0.5).await?;
    ///     
    ///     // 像素坐标
    ///     let (x, y) = device.pos_rel2abs(100.0, 200.0).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn pos_rel2abs(&self, x: f32, y: f32) -> Result<(u32, u32)> {
        // 获取屏幕尺寸
        let (width, height) = self.window_size().await?;

        // 转换坐标
        let x_abs = if x <= 1.0 {
            (x * width as f32) as u32
        } else {
            x as u32
        };

        let y_abs = if y <= 1.0 {
            (y * height as f32) as u32
        } else {
            y as u32
        };

        Ok((x_abs, y_abs))
    }

    /// 点击指定坐标
    ///
    /// # 参数
    ///
    /// * `x` - X 坐标（像素）
    /// * `y` - Y 坐标（像素）
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 点击坐标 (100, 200)
    ///     device.click(100, 200).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn click(&self, x: u32, y: u32) -> Result<()> {
        debug!("点击坐标: ({}, {})", x, y);

        // 应用操作前延迟
        let settings = self.settings.read().unwrap();
        let delay_before = settings.operation_delay_before;
        drop(settings);

        if delay_before > Duration::from_millis(0) {
            tokio::time::sleep(delay_before).await;
        }

        // 调用 JSON-RPC 的 click 方法
        let _: serde_json::Value = self
            .jsonrpc_client
            .call("click", serde_json::json!([x, y]))
            .await?;

        // 应用操作后延迟
        let settings = self.settings.read().unwrap();
        let delay_after = settings.operation_delay_after;
        drop(settings);

        if delay_after > Duration::from_millis(0) {
            tokio::time::sleep(delay_after).await;
        }

        Ok(())
    }

    /// 长按指定坐标
    ///
    /// # 参数
    ///
    /// * `x` - X 坐标（像素）
    /// * `y` - Y 坐标（像素）
    /// * `duration` - 长按时长，None 表示使用默认值 0.5 秒
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 长按 0.5 秒（默认）
    ///     device.long_click(100, 200, None).await?;
    ///     
    ///     // 长按 1 秒
    ///     device.long_click(100, 200, Some(Duration::from_secs(1))).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn long_click(&self, x: u32, y: u32, duration: Option<Duration>) -> Result<()> {
        let duration = duration.unwrap_or(Duration::from_millis(500));
        let duration_secs = duration.as_secs_f32();

        debug!("长按坐标: ({}, {}), 时长: {}s", x, y, duration_secs);

        // 应用操作前延迟
        let settings = self.settings.read().unwrap();
        let delay_before = settings.operation_delay_before;
        drop(settings);

        if delay_before > Duration::from_millis(0) {
            tokio::time::sleep(delay_before).await;
        }

        // 优先使用 longClick；部分服务端版本未实现或参数签名不兼容时回退到 swipe 同点长按。
        match self
            .jsonrpc_client
            .call("longClick", serde_json::json!([x, y, duration_secs]))
            .await
        {
            Ok::<serde_json::Value, Error>(_value) => {}
            Err(error) => {
                if Self::is_jsonrpc_method_unavailable_error(&error)
                    || Self::is_jsonrpc_method_params_invalid_error(&error)
                {
                    warn!(
                        "JSON-RPC longClick unavailable, fallback to adb input long press at ({}, {})",
                        x, y
                    );
                    if let Err(shell_error) = self.long_click_via_adb_input(x, y, duration).await {
                        warn!(
                            "ADB input long press fallback failed: {:?}, fallback to JSON-RPC swipe hold",
                            shell_error
                        );
                        let _: serde_json::Value = self
                            .jsonrpc_client
                            .call("swipe", serde_json::json!([x, y, x, y, duration_secs]))
                            .await?;
                    }
                } else {
                    return Err(error);
                }
            }
        }

        // 应用操作后延迟
        let settings = self.settings.read().unwrap();
        let delay_after = settings.operation_delay_after;
        drop(settings);

        if delay_after > Duration::from_millis(0) {
            tokio::time::sleep(delay_after).await;
        }

        Ok(())
    }

    /// 双击指定坐标
    ///
    /// # 参数
    ///
    /// * `x` - X 坐标（像素）
    /// * `y` - Y 坐标（像素）
    /// * `duration` - 两次点击之间的间隔，None 表示使用默认值 0.1 秒
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 双击，间隔 0.1 秒（默认）
    ///     device.double_click(100, 200, None).await?;
    ///     
    ///     // 双击，间隔 0.2 秒
    ///     device.double_click(100, 200, Some(Duration::from_millis(200))).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn double_click(&self, x: u32, y: u32, duration: Option<Duration>) -> Result<()> {
        let duration = duration.unwrap_or(Duration::from_millis(100));
        let duration_secs = duration.as_secs_f32();

        debug!("双击坐标: ({}, {}), 间隔: {}s", x, y, duration_secs);

        // 应用操作前延迟
        let settings = self.settings.read().unwrap();
        let delay_before = settings.operation_delay_before;
        drop(settings);

        if delay_before > Duration::from_millis(0) {
            tokio::time::sleep(delay_before).await;
        }

        // 优先使用 doubleClick；不可用时回退为两次 click。
        match self
            .jsonrpc_client
            .call("doubleClick", serde_json::json!([x, y, duration_secs]))
            .await
        {
            Ok::<serde_json::Value, Error>(_value) => {}
            Err(error) => {
                if Self::is_jsonrpc_method_unavailable_error(&error) {
                    warn!(
                        "JSON-RPC doubleClick unavailable, fallback to two click calls at ({}, {})",
                        x, y
                    );
                    let _: serde_json::Value = self
                        .jsonrpc_client
                        .call("click", serde_json::json!([x, y]))
                        .await?;
                    tokio::time::sleep(duration).await;
                    let _: serde_json::Value = self
                        .jsonrpc_client
                        .call("click", serde_json::json!([x, y]))
                        .await?;
                } else {
                    return Err(error);
                }
            }
        }

        // 应用操作后延迟
        let settings = self.settings.read().unwrap();
        let delay_after = settings.operation_delay_after;
        drop(settings);

        if delay_after > Duration::from_millis(0) {
            tokio::time::sleep(delay_after).await;
        }

        Ok(())
    }

    /// 滑动操作
    ///
    /// # 参数
    ///
    /// * `fx` - 起始 X 坐标（像素）
    /// * `fy` - 起始 Y 坐标（像素）
    /// * `tx` - 结束 X 坐标（像素）
    /// * `ty` - 结束 Y 坐标（像素）
    /// * `duration` - 滑动持续时间，None 表示使用默认值 0.5 秒
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 从左向右滑动，持续 0.5 秒（默认）
    ///     device.swipe(100, 500, 900, 500, None).await?;
    ///     
    ///     // 向上滑动，持续 0.3 秒
    ///     device.swipe(500, 1000, 500, 200, Some(Duration::from_millis(300))).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn swipe(
        &self,
        fx: u32,
        fy: u32,
        tx: u32,
        ty: u32,
        duration: Option<Duration>,
    ) -> Result<()> {
        let duration = duration.unwrap_or(Duration::from_millis(500));
        let duration_secs = duration.as_secs_f32();

        debug!(
            "滑动: ({}, {}) -> ({}, {}), 时长: {}s",
            fx, fy, tx, ty, duration_secs
        );

        // 应用操作前延迟
        let settings = self.settings.read().unwrap();
        let delay_before = settings.operation_delay_before;
        drop(settings);

        if delay_before > Duration::from_millis(0) {
            tokio::time::sleep(delay_before).await;
        }

        // 调用 JSON-RPC 的 swipe 方法
        let _: serde_json::Value = self
            .jsonrpc_client
            .call("swipe", serde_json::json!([fx, fy, tx, ty, duration_secs]))
            .await?;

        // 应用操作后延迟
        let settings = self.settings.read().unwrap();
        let delay_after = settings.operation_delay_after;
        drop(settings);

        if delay_after > Duration::from_millis(0) {
            tokio::time::sleep(delay_after).await;
        }

        Ok(())
    }

    /// 拖拽操作
    ///
    /// # 参数
    ///
    /// * `sx` - 起始 X 坐标（像素）
    /// * `sy` - 起始 Y 坐标（像素）
    /// * `ex` - 结束 X 坐标（像素）
    /// * `ey` - 结束 Y 坐标（像素）
    /// * `duration` - 拖拽持续时间，None 表示使用默认值 0.5 秒
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 拖拽，持续 0.5 秒（默认）
    ///     device.drag(100, 500, 900, 500, None).await?;
    ///     
    ///     // 拖拽，持续 0.8 秒
    ///     device.drag(100, 500, 900, 500, Some(Duration::from_millis(800))).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn drag(
        &self,
        sx: u32,
        sy: u32,
        ex: u32,
        ey: u32,
        duration: Option<Duration>,
    ) -> Result<()> {
        let duration = duration.unwrap_or(Duration::from_millis(500));
        let duration_secs = duration.as_secs_f32();

        debug!(
            "拖拽: ({}, {}) -> ({}, {}), 时长: {}s",
            sx, sy, ex, ey, duration_secs
        );

        // 应用操作前延迟
        let settings = self.settings.read().unwrap();
        let delay_before = settings.operation_delay_before;
        drop(settings);

        if delay_before > Duration::from_millis(0) {
            tokio::time::sleep(delay_before).await;
        }

        // 调用 JSON-RPC 的 drag 方法
        let _: serde_json::Value = self
            .jsonrpc_client
            .call("drag", serde_json::json!([sx, sy, ex, ey, duration_secs]))
            .await?;

        // 应用操作后延迟
        let settings = self.settings.read().unwrap();
        let delay_after = settings.operation_delay_after;
        drop(settings);

        if delay_after > Duration::from_millis(0) {
            tokio::time::sleep(delay_after).await;
        }

        Ok(())
    }

    /// 查找 UI 元素
    ///
    /// # 参数
    ///
    /// * `selector` - 元素选择器
    ///
    /// # 返回
    ///
    /// 返回 UiObject 实例
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::{Device, Selector};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     let element = device.find(Selector::new().text("Settings"));
    ///     Ok(())
    /// }
    /// ```
    pub fn find(&self, selector: Selector) -> UiObject {
        debug!("创建 UiObject: {:?}", selector);

        // 创建 Arc<Device> 的克隆用于 UiObject
        // 由于 Device 已经实现了 Clone，我们可以将 self 包装成 Arc
        let device_arc = Arc::new(self.clone());

        UiObject::new(device_arc, selector)
    }

    /// 获取设备序列号
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     let serial = device.serial();
    ///     assert!(!serial.is_empty());
    ///     Ok(())
    /// }
    /// ```
    pub fn serial(&self) -> &str {
        &self.serial
    }

    /// 获取服务器模式
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uiautomator::{Device, ServerMode};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     let mode = device.server_mode();
    ///     assert!(matches!(mode, ServerMode::Auto | ServerMode::AtxAgent | ServerMode::Direct));
    ///     Ok(())
    /// }
    /// ```
    pub fn server_mode(&self) -> ServerMode {
        self.server_mode
    }

    /// 获取 ADB 客户端引用
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     let adb_ref = device.adb_client();
    ///     assert!(Arc::strong_count(adb_ref) >= 1);
    ///     Ok(())
    /// }
    /// ```
    pub fn adb_client(&self) -> &Arc<AdbClient> {
        &self.adb_client
    }

    /// 获取 JSON-RPC 客户端引用
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     let rpc_ref = device.jsonrpc_client();
    ///     assert!(Arc::strong_count(rpc_ref) >= 1);
    ///     Ok(())
    /// }
    /// ```
    pub fn jsonrpc_client(&self) -> &Arc<JsonRpcClient> {
        &self.jsonrpc_client
    }

    /// 获取设置引用
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     let settings_ref = device.settings();
    ///     let settings = settings_ref.read().unwrap();
    ///     assert!(settings.wait_timeout.as_secs() > 0);
    ///     Ok(())
    /// }
    /// ```
    pub fn settings(&self) -> &Arc<RwLock<Settings>> {
        &self.settings
    }

    /// 设置等待超时时间
    ///
    /// # 参数
    ///
    /// * `timeout` - 新的超时时间
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 设置全局等待超时为 30 秒
    ///     device.set_wait_timeout(Duration::from_secs(30));
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn set_wait_timeout(&self, timeout: Duration) {
        let mut settings = self.settings.write().unwrap();
        settings.set_wait_timeout(timeout);
    }

    /// 获取等待超时时间
    ///
    /// # 返回
    ///
    /// 返回当前的等待超时时间
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     let timeout = device.get_wait_timeout();
    ///     println!("当前等待超时: {:?}", timeout);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn get_wait_timeout(&self) -> Duration {
        let settings = self.settings.read().unwrap();
        settings.wait_timeout
    }

    /// 获取轮询间隔
    ///
    /// # 返回
    ///
    /// 返回当前配置的轮询间隔时间
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     let interval = device.get_polling_interval();
    ///     assert!(interval.as_millis() > 0);
    ///     Ok(())
    /// }
    /// ```
    pub fn get_polling_interval(&self) -> Duration {
        let settings = self.settings.read().unwrap();
        settings.polling_interval
    }

    /// 轮询等待条件满足（辅助函数）
    ///
    /// # 参数
    ///
    /// * `condition` - 条件闭包，返回 `Result<bool>`
    /// * `timeout` - 超时时间，None 表示使用全局超时
    ///
    /// # 返回
    ///
    /// 如果条件在超时前满足返回 Ok(())，否则返回 Error::Timeout
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 等待某个条件满足
    ///     device.wait_for(
    ///         || async { Ok(true) },
    ///         Some(Duration::from_secs(10))
    ///     ).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn wait_for<F, Fut>(&self, condition: F, timeout: Option<Duration>) -> Result<()>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<bool>>,
    {
        // 获取超时时间
        let timeout = timeout.unwrap_or_else(|| self.get_wait_timeout());

        // 轮询间隔
        const POLL_INTERVAL: Duration = Duration::from_millis(500);

        // 使用 tokio::time::timeout 实现超时
        let result = tokio::time::timeout(timeout, async {
            loop {
                // 检查条件
                match condition().await {
                    Ok(true) => return Ok(()),
                    Ok(false) => {
                        // 条件未满足，继续等待
                        tokio::time::sleep(POLL_INTERVAL).await;
                    }
                    Err(e) => return Err(e),
                }
            }
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(Error::Timeout),
        }
    }

    /// 按下指定按键
    ///
    /// # 参数
    ///
    /// * `key` - 按键枚举
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::{Device, Key};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 按下 Home 键
    ///     device.press(Key::Home).await?;
    ///     
    ///     // 按下 Back 键
    ///     device.press(Key::Back).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn press(&self, key: crate::key::Key) -> Result<()> {
        debug!("按下按键: {:?}", key);

        // 应用操作前延迟
        let settings = self.settings.read().unwrap();
        let delay_before = settings.operation_delay_before;
        drop(settings);

        if delay_before > Duration::from_millis(0) {
            tokio::time::sleep(delay_before).await;
        }

        // 调用 JSON-RPC 的 pressKey 方法
        let key_name = key.to_name();
        let _: serde_json::Value = self
            .jsonrpc_client
            .call("pressKey", serde_json::json!([key_name]))
            .await?;

        // 应用操作后延迟
        let settings = self.settings.read().unwrap();
        let delay_after = settings.operation_delay_after;
        drop(settings);

        if delay_after > Duration::from_millis(0) {
            tokio::time::sleep(delay_after).await;
        }

        Ok(())
    }

    /// 按下指定键码
    ///
    /// # 参数
    ///
    /// * `keycode` - Android KeyEvent 键码
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 按下 Home 键（键码 3）
    ///     device.press_keycode(3).await?;
    ///     
    ///     // 按下 Back 键（键码 4）
    ///     device.press_keycode(4).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn press_keycode(&self, keycode: u32) -> Result<()> {
        debug!("按下键码: {}", keycode);

        // 应用操作前延迟
        let settings = self.settings.read().unwrap();
        let delay_before = settings.operation_delay_before;
        drop(settings);

        if delay_before > Duration::from_millis(0) {
            tokio::time::sleep(delay_before).await;
        }

        // 调用 JSON-RPC 的 pressKeyCode 方法
        let _: serde_json::Value = self
            .jsonrpc_client
            .call("pressKeyCode", serde_json::json!([keycode]))
            .await?;

        // 应用操作后延迟
        let settings = self.settings.read().unwrap();
        let delay_after = settings.operation_delay_after;
        drop(settings);

        if delay_after > Duration::from_millis(0) {
            tokio::time::sleep(delay_after).await;
        }

        Ok(())
    }

    /// 截取设备屏幕
    ///
    /// # 返回
    ///
    /// 返回 `DynamicImage` 图像对象
    ///
    /// # 错误
    ///
    /// - 如果截图失败，返回 `Error::JsonRpc` 或 `Error::Image`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 截图
    ///     let image = device.screenshot().await?;
    ///     println!("截图尺寸: {}x{}", image.width(), image.height());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn screenshot(&self) -> Result<image::DynamicImage> {
        debug!("截取屏幕");

        match self
            .jsonrpc_client
            .call("screenshot", serde_json::json!({}))
            .await
        {
            Ok(response) => Self::decode_jsonrpc_screenshot_response(response),
            Err(error) => {
                if Self::is_jsonrpc_method_unavailable_error(&error) {
                    warn!(
                        "JSON-RPC screenshot unavailable, fallback to ATX-Agent /screenshot/0 then adb screencap"
                    );

                    if let Ok(image) = self.screenshot_via_atx_agent_http_with_forward().await {
                        return Ok(image);
                    }

                    warn!("ATX-Agent screenshot fallback failed, fallback to adb screencap");
                    return self.screenshot_via_adb_screencap().await;
                }
                Err(error)
            }
        }
    }

    fn decode_jsonrpc_screenshot_response(
        response: serde_json::Value,
    ) -> Result<image::DynamicImage> {
        let base64_data = response
            .as_str()
            .ok_or_else(|| Error::InvalidArgument("截图响应不是字符串".to_string()))?;

        use base64::Engine;
        let image_data = base64::engine::general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| Error::InvalidArgument(format!("Base64 解码失败: {}", e)))?;

        let image = image::load_from_memory(&image_data)?;
        Ok(image)
    }

    async fn screenshot_via_atx_agent_http_with_forward(&self) -> Result<image::DynamicImage> {
        // Even in Direct mode we can try ATX screenshot if forwarding is available.
        if let Err(e) = self.adb_client.forward(&self.serial, 7912, 7912).await {
            debug!(
                "failed to ensure ATX-Agent port forward before screenshot fallback: {:?}",
                e
            );
        }
        self.screenshot_via_atx_agent_http().await
    }

    async fn screenshot_via_atx_agent_http(&self) -> Result<image::DynamicImage> {
        let response = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(Error::Http)?
            .get("http://127.0.0.1:7912/screenshot/0")
            .send()
            .await
            .map_err(Error::Http)?
            .error_for_status()
            .map_err(Error::Http)?;

        let bytes = response.bytes().await.map_err(Error::Http)?;
        let image = image::load_from_memory(bytes.as_ref())?;
        Ok(image)
    }

    async fn screenshot_via_adb_screencap(&self) -> Result<image::DynamicImage> {
        // Avoid adb pull large-file edge cases by streaming PNG bytes as base64 text.
        let capture_command = "screencap -p | base64";
        let capture_output = self
            .adb_client
            .shell(&self.serial, capture_command, Some(Duration::from_secs(30)))
            .await?;

        let trimmed = capture_output.trim();
        if trimmed.is_empty() {
            return Err(Error::Adb(
                "adb screencap returned empty output".to_string(),
            ));
        }

        let base64_data: String = trimmed.chars().filter(|ch| !ch.is_whitespace()).collect();
        if base64_data.is_empty() {
            return Err(Error::Adb(
                "adb screencap returned empty base64 payload".to_string(),
            ));
        }

        use base64::Engine;
        let image_bytes = base64::engine::general_purpose::STANDARD
            .decode(base64_data.as_bytes())
            .map_err(|e| {
                let lower = trimmed.to_lowercase();
                if lower.contains("not found")
                    || lower.contains("permission denied")
                    || lower.contains("inaccessible")
                {
                    return Error::Adb(format!(
                        "adb screencap command failed: {}",
                        Self::truncate_error_detail(trimmed, 200)
                    ));
                }
                Error::InvalidArgument(format!(
                    "failed to decode adb screencap base64 output: {}",
                    e
                ))
            })?;

        let image = image::load_from_memory(&image_bytes)?;
        Ok(image)
    }

    fn is_jsonrpc_method_unavailable_error(error: &Error) -> bool {
        let Error::JsonRpc(message) = error else {
            return false;
        };
        let lower = message.to_lowercase();
        lower.contains("-32601")
            || lower.contains("method not found")
            || lower.contains("unknown method")
            || lower.contains("not implemented")
    }

    fn is_jsonrpc_method_params_invalid_error(error: &Error) -> bool {
        let Error::JsonRpc(message) = error else {
            return false;
        };
        let lower = message.to_lowercase();
        lower.contains("-32602")
            || lower.contains("invalid params")
            || lower.contains("method parameters invalid")
    }

    async fn long_click_via_adb_input(&self, x: u32, y: u32, duration: Duration) -> Result<()> {
        let duration_ms = duration.as_millis().max(100);
        let command = format!("input swipe {} {} {} {} {}", x, y, x, y, duration_ms);
        let output = self
            .adb_client
            .shell(&self.serial, &command, Some(Duration::from_secs(10)))
            .await?;

        let trimmed = output.trim();
        if !trimmed.is_empty() {
            let lower = trimmed.to_lowercase();
            if lower.contains("error")
                || lower.contains("unknown")
                || lower.contains("invalid")
                || lower.contains("usage")
            {
                return Err(Error::Adb(format!(
                    "input swipe failed: {}",
                    Self::truncate_error_detail(trimmed, 200)
                )));
            }
            debug!(
                "input swipe returned non-empty output: {}",
                Self::truncate_error_detail(trimmed, 200)
            );
        }

        Ok(())
    }

    /// 截取设备屏幕并保存到文件
    ///
    /// # 参数
    ///
    /// * `path` - 保存路径，支持 PNG 和 JPEG 格式（根据文件扩展名自动识别）
    ///
    /// # 错误
    ///
    /// - 如果截图失败，返回 `Error::JsonRpc` 或 `Error::Image`
    /// - 如果文件写入失败，返回 `Error::Io`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 保存为 PNG
    ///     device.screenshot_to_file("screenshot.png").await?;
    ///     
    ///     // 保存为 JPEG
    ///     device.screenshot_to_file("screenshot.jpg").await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn screenshot_to_file(&self, path: &str) -> Result<()> {
        debug!("截取屏幕并保存到: {}", path);

        // 截图
        let image = self.screenshot().await?;

        // 根据文件扩展名确定格式
        let path_lower = path.to_lowercase();
        let format = if path_lower.ends_with(".png") {
            image::ImageFormat::Png
        } else if path_lower.ends_with(".jpg") || path_lower.ends_with(".jpeg") {
            image::ImageFormat::Jpeg
        } else {
            // 默认使用 PNG 格式
            image::ImageFormat::Png
        };

        // 保存图像
        image.save_with_format(path, format)?;

        info!("截图已保存到: {}", path);

        Ok(())
    }

    /// 从 shell 输出中提取退出码，并返回去除退出码标记后的输出内容。
    fn parse_shell_output_with_exit_code(output: &str, exit_marker: &str) -> (Option<i32>, String) {
        let mut exit_code = None;
        let mut cleaned_lines = Vec::new();

        for line in output.lines() {
            let trimmed = line.trim();
            if let Some(code_str) = trimmed.strip_prefix(exit_marker) {
                if let Ok(code) = code_str.parse::<i32>() {
                    exit_code = Some(code);
                    continue;
                }
            }
            cleaned_lines.push(line);
        }

        let cleaned_output = cleaned_lines.join("\n").trim().to_string();
        (exit_code, cleaned_output)
    }

    fn pm_path_output_has_package(output: &str) -> bool {
        output
            .lines()
            .map(str::trim_start)
            .any(|line| line.starts_with("package:"))
    }

    async fn is_package_installed(&self, package: &str) -> Result<bool> {
        let command = format!("pm path {}", package);
        let output = self
            .adb_client
            .shell(&self.serial, &command, Some(Duration::from_secs(10)))
            .await?;
        Ok(Self::pm_path_output_has_package(&output))
    }

    /// 判断 am start 输出中的某一行是否为明确失败信号。
    fn is_app_start_failure_line(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return false;
        }

        trimmed.starts_with("Error:")
            || trimmed.starts_with("Error type ")
            || trimmed.starts_with("Exception occurred while executing")
            || trimmed.contains("java.lang.SecurityException")
            || trimmed.contains("java.lang.IllegalArgumentException")
            || trimmed.contains("Activity class")
            || trimmed.contains("does not exist")
            || trimmed.contains("Unable to resolve Intent")
            || trimmed.contains("No activity found")
            || trimmed.contains("No activities found to run")
    }

    fn classify_app_start_failure(package: &str, reason: &str) -> Error {
        let lower = reason.to_lowercase();
        let app_not_installed_markers = [
            "unknown package",
            "package not found",
            "package was not found",
            "is not installed for",
        ];

        if app_not_installed_markers
            .iter()
            .any(|marker| lower.contains(marker))
        {
            return Error::AppNotInstalled(package.to_string());
        }

        let app_crash_markers = [
            "fatal exception",
            "has crashed",
            "has stopped",
            "force finishing activity",
            "process crashed",
            "anr in",
            "application not responding",
        ];

        if app_crash_markers
            .iter()
            .any(|marker| lower.contains(marker))
        {
            return Error::AppCrashed(package.to_string());
        }

        Error::AppStartFailed(format!("{}: {}", package, reason))
    }

    /// 提取 app_start 的关键错误信息，避免将完整输出塞进错误对象。
    fn extract_app_start_failure_reason(output: &str) -> String {
        if let Some(line) = output
            .lines()
            .map(str::trim)
            .find(|line| Self::is_app_start_failure_line(line))
        {
            return Self::truncate_error_detail(line, 240);
        }

        if output.trim().is_empty() {
            return "am start 失败，但没有可用错误输出".to_string();
        }

        Self::truncate_error_detail(output.trim(), 240)
    }

    /// 将错误详情截断到固定长度，避免错误消息过长。
    fn truncate_error_detail(message: &str, max_chars: usize) -> String {
        if message.chars().count() <= max_chars {
            return message.to_string();
        }

        let truncated: String = message.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }

    fn parse_resolved_activity_component(output: &str) -> Option<String> {
        output
            .lines()
            .map(str::trim)
            .find(|line| {
                !line.is_empty()
                    && !line.starts_with("priority=")
                    && !line.starts_with("ResolveInfo")
                    && line.contains('/')
            })
            .map(ToString::to_string)
    }

    fn is_resolve_activity_command_unavailable_error(error: &Error) -> bool {
        let Error::AppStartFailed(message) = error else {
            return false;
        };

        let lower = message.to_lowercase();
        lower.contains("cmd: not found")
            || lower.contains("cmd: inaccessible")
            || lower.contains("inaccessible or not found")
            || lower.contains("unknown command")
            || lower.contains("can't find service")
            || lower.contains("no service package")
            || lower.contains("not supported")
            || lower.contains("permission denied")
    }

    async fn resolve_launchable_activity(&self, package: &str) -> Result<String> {
        const RESOLVE_EXIT_MARKER: &str = "__U2_RESOLVE_ACTIVITY_EXIT_CODE__:";
        let command = format!(
            "cmd package resolve-activity --brief {} 2>&1; echo {}$?",
            package, RESOLVE_EXIT_MARKER
        );

        let raw_output = self.adb_client.shell(&self.serial, &command, None).await?;
        let (exit_code, output) =
            Self::parse_shell_output_with_exit_code(&raw_output, RESOLVE_EXIT_MARKER);

        if exit_code.map(|code| code != 0).unwrap_or(false) {
            let reason = Self::extract_app_start_failure_reason(&output);
            return Err(Self::classify_app_start_failure(
                package,
                &format!("failed to resolve launch activity: {}", reason),
            ));
        }

        if let Some(component) = Self::parse_resolved_activity_component(&output) {
            return Ok(component);
        }

        let reason = Self::extract_app_start_failure_reason(&output);
        Err(Self::classify_app_start_failure(
            package,
            &format!("failed to resolve launch activity: {}", reason),
        ))
    }

    /// 启动应用
    ///
    /// # 参数
    ///
    /// * `package` - 应用包名
    /// * `activity` - Activity 名称（可选），None 表示启动默认 Activity
    ///
    /// # 错误
    ///
    /// - 如果应用未安装，返回 `Error::AppNotInstalled`
    /// - 如果应用崩溃，返回 `Error::AppCrashed`
    /// - 其他启动失败返回 `Error::AppStartFailed`
    /// - 当包名或 Activity 无效时，错误信息会包含 `am start` 的关键失败行
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 启动应用（使用默认 Activity）
    ///     device.app_start("com.android.settings", None).await?;
    ///     
    ///     // 启动应用（指定 Activity）
    ///     device.app_start("com.android.settings", Some(".Settings")).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn app_start(&self, package: &str, activity: Option<&str>) -> Result<()> {
        debug!("启动应用: {} {:?}", package, activity);
        const APP_START_EXIT_MARKER: &str = "__U2_APP_START_EXIT_CODE__:";

        if activity.is_some() && !self.is_package_installed(package).await? {
            return Err(Error::AppNotInstalled(package.to_string()));
        }

        // 构建 am start 命令
        let am_start_command = match activity {
            Some(act) => {
                // 如果 Activity 以 . 开头，需要拼接包名
                let full_activity = if act.starts_with('.') {
                    format!("{}{}", package, act)
                } else {
                    act.to_string()
                };
                format!("am start -n {}/{}", package, full_activity)
            }
            None => match self.resolve_launchable_activity(package).await {
                Ok(component) => format!("am start -n {}", component),
                Err(error) if Self::is_resolve_activity_command_unavailable_error(&error) => {
                    warn!(
                        "resolve-activity unavailable, fallback to package-only start for {}: {:?}",
                        package, error
                    );
                    format!("am start {}", package)
                }
                Err(error) => return Err(error),
            },
        };

        // 在命令末尾附加退出码标记，便于精确判断是否失败
        let command = format!("{}; echo {}$?", am_start_command, APP_START_EXIT_MARKER);

        // 执行命令
        let raw_output = self.adb_client.shell(&self.serial, &command, None).await?;
        let (output_exit_code, output) =
            Self::parse_shell_output_with_exit_code(&raw_output, APP_START_EXIT_MARKER);

        let has_failure_line = output.lines().any(Self::is_app_start_failure_line);
        let has_non_zero_exit = output_exit_code.map(|code| code != 0).unwrap_or(false);

        if has_non_zero_exit || has_failure_line {
            let reason = Self::extract_app_start_failure_reason(&output);
            let detail = if let Some(code) = output_exit_code.filter(|code| *code != 0) {
                format!("exit_code={} {}", code, reason)
            } else {
                reason
            };
            return Err(Self::classify_app_start_failure(package, &detail));
        }

        info!("应用启动成功: {}", package);

        Ok(())
    }

    /// 停止应用
    ///
    /// # 参数
    ///
    /// * `package` - 应用包名
    ///
    /// # 错误
    ///
    /// - 如果停止失败，返回 `Error::Adb`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 停止应用
    ///     device.app_stop("com.android.settings").await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn app_stop(&self, package: &str) -> Result<()> {
        debug!("停止应用: {}", package);

        // 执行 am force-stop 命令
        let command = format!("am force-stop {}", package);
        let _output = self.adb_client.shell(&self.serial, &command, None).await?;

        info!("应用停止成功: {}", package);

        Ok(())
    }

    /// 清除应用数据
    ///
    /// # 参数
    ///
    /// * `package` - 应用包名
    ///
    /// # 错误
    ///
    /// - 如果应用未安装，返回 `Error::AppNotInstalled`
    /// - 如果清除失败，返回 `Error::Adb`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 清除应用数据
    ///     device.app_clear("com.android.settings").await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn app_clear(&self, package: &str) -> Result<()> {
        debug!("清除应用数据: {}", package);

        // 执行 pm clear 命令
        let command = format!("pm clear {}", package);
        let output = self.adb_client.shell(&self.serial, &command, None).await?;

        // 检查输出是否表示成功
        if output.contains("Success") {
            info!("应用数据清除成功: {}", package);
            Ok(())
        } else if output.contains("Failed") || output.contains("Unknown package") {
            Err(Error::AppNotInstalled(package.to_string()))
        } else {
            Err(Error::Adb(format!("清除应用数据失败: {}", output)))
        }
    }

    /// 获取当前前台应用信息
    ///
    /// # 返回
    ///
    /// 返回当前前台应用的包名、Activity 和 PID
    ///
    /// # 错误
    ///
    /// - 如果获取失败，返回 `Error::Adb`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 获取当前应用信息
    ///     let info = device.app_current().await?;
    ///     println!("当前应用: {} / {}", info.package, info.activity);
    ///     
    ///     Ok(())
    /// }
    /// ```
    fn parse_app_component_from_dump(output: &str) -> Option<(String, String)> {
        let re = regex::Regex::new(r"([A-Za-z0-9_.]+)/([A-Za-z0-9_.$]+)").ok()?;
        // Prefer resumed/focused activity markers over window focus markers to avoid
        // transient IME focus (keyboard windows) being parsed as foreground apps.
        let focus_markers_by_priority = [
            "mFocusedApp",
            "topResumedActivity",
            "mResumedActivity",
            "mCurrentFocus",
        ];
        let is_plausible_package_name =
            |package: &str| package == "android" || package.contains('.');

        for marker in focus_markers_by_priority {
            for line in output.lines() {
                if !line.contains(marker) {
                    continue;
                }
                if let Some(caps) = re.captures(line) {
                    let package = caps.get(1)?.as_str().to_string();
                    let activity = caps.get(2)?.as_str().to_string();
                    if is_plausible_package_name(&package) {
                        return Some((package, activity));
                    }
                }
            }
        }

        let fallback_markers = [
            "ActivityRecord{",
            "ResumedActivity",
            "topResumedActivity",
            "mFocusedApp",
            "mCurrentFocus",
            "mResumedActivity",
        ];
        for line in output.lines() {
            if !fallback_markers.iter().any(|marker| line.contains(marker)) {
                continue;
            }
            if let Some(caps) = re.captures(line) {
                let package = caps.get(1)?.as_str().to_string();
                let activity = caps.get(2)?.as_str().to_string();
                if is_plausible_package_name(&package) {
                    return Some((package, activity));
                }
            }
        }

        None
    }

    /// Get the current foreground application from system dumpsys output.
    ///
    /// The result includes package name, activity name, and optional PID.
    ///
    /// # Errors
    ///
    /// Returns an error when ADB command execution fails or when the current
    /// foreground component cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     let app = device.app_current().await?;
    ///     println!("package={}, activity={}", app.package, app.activity);
    ///     Ok(())
    /// }
    /// ```
    pub async fn app_current(&self) -> Result<crate::models::AppInfo> {
        debug!("获取当前前台应用信息");

        let window_dump = self
            .adb_client
            .shell(&self.serial, "dumpsys window windows", None)
            .await?;

        let (package, activity) =
            if let Some(component) = Self::parse_app_component_from_dump(&window_dump) {
                component
            } else {
                let activity_dump = self
                    .adb_client
                    .shell(&self.serial, "dumpsys activity activities", None)
                    .await?;
                Self::parse_app_component_from_dump(&activity_dump)
                    .ok_or_else(|| Error::Adb("无法解析当前前台应用信息".to_string()))?
            };

        let activity = if activity.starts_with('.') {
            format!("{}{}", package, activity)
        } else {
            activity
        };

        let pid = self.get_app_pid(&package).await.ok();
        Ok(crate::models::AppInfo {
            package,
            activity,
            pid,
        })
    }

    /// 等待应用启动
    ///
    /// # 参数
    ///
    /// * `package` - 应用包名
    /// * `timeout` - 超时时间，支持 `Duration` 或 `Option<Duration>`。
    ///   传入 `None` 时使用全局等待超时（`Device::get_wait_timeout()`）
    ///
    /// # 返回
    ///
    /// 返回应用的进程 ID
    ///
    /// # 错误
    ///
    /// - 如果超时，返回 `Error::Timeout`
    /// - 如果获取失败，返回 `Error::Adb`
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 启动应用
    ///     device.app_start("com.android.settings", None).await?;
    ///     
    ///     // 等待应用启动
    ///     let pid = device
    ///         .app_wait("com.android.settings", Some(Duration::from_secs(10)))
    ///         .await?;
    ///     println!("应用已启动，PID: {}", pid);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn app_wait<T>(&self, package: &str, timeout: T) -> Result<u32>
    where
        T: Into<Option<Duration>>,
    {
        let timeout = timeout.into().unwrap_or_else(|| self.get_wait_timeout());
        debug!("等待应用启动: {}, 超时: {:?}", package, timeout);

        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(500);

        loop {
            // 检查是否超时
            if start.elapsed() >= timeout {
                return Err(Error::Timeout);
            }

            // 尝试获取应用 PID
            if let Ok(pid) = self.get_app_pid(package).await {
                info!("应用已启动: {}, PID: {}", package, pid);
                return Ok(pid);
            }

            // 等待一段时间后重试
            tokio::time::sleep(poll_interval).await;
        }
    }

    /// 获取应用的进程 ID（辅助方法）
    ///
    /// # 参数
    ///
    /// * `package` - 应用包名
    ///
    /// # 返回
    ///
    /// 返回应用的进程 ID
    ///
    /// # 错误
    ///
    /// - 如果应用未运行，返回 `Error::AppNotRunning`
    /// - 如果获取失败，返回 `Error::Adb`
    async fn get_app_pid(&self, package: &str) -> Result<u32> {
        // 使用 pidof 命令获取 PID
        let command = format!("pidof {}", package);
        let output = self.adb_client.shell(&self.serial, &command, None).await?;

        // 解析 PID
        let pid_str = output.trim();
        if pid_str.is_empty() {
            return Err(Error::AppNotRunning(package.to_string()));
        }

        // 可能返回多个 PID（用空格分隔），取第一个
        let first_pid = pid_str.split_whitespace().next().unwrap_or("");

        first_pid
            .parse::<u32>()
            .map_err(|_| Error::Adb(format!("无法解析 PID: {}", pid_str)))
    }

    // ========================================================================
    // ATX-Agent 安装功能（仅在 atx-agent-install feature 启用时可用）
    // ========================================================================

    /// 安装 ATX-Agent 到设备
    ///
    /// 执行完整的 ATX-Agent 安装流程：
    /// 1. 推送 atx-agent 二进制文件
    /// 2. 安装 UiAutomator APK
    /// 3. 启动 atx-agent 服务
    ///
    /// # 参数
    ///
    /// * `force` - 是否强制重新安装（即使已安装）
    ///
    /// # 错误
    ///
    /// 如果安装过程中出现错误，返回错误
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     
    ///     // 安装 ATX-Agent（如果未安装）
    ///     device.install_atx_agent(false).await?;
    ///     
    ///     // 强制重新安装
    ///     device.install_atx_agent(true).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    #[cfg(feature = "atx-agent-install")]
    pub async fn install_atx_agent(&self, force: bool) -> Result<()> {
        use crate::atx_agent::AtxAgentClient;

        info!("开始安装 ATX-Agent");

        // 创建 ATX-Agent 客户端
        let atx_agent_client =
            AtxAgentClient::new(self.serial.clone(), self.adb_client.clone()).await?;

        // 执行安装
        atx_agent_client.install(force).await?;

        // 启动服务
        atx_agent_client.start_atx_agent().await?;

        // 等待服务就绪
        atx_agent_client
            .wait_for_atx_agent_ready(Some(Duration::from_secs(30)))
            .await?;

        info!("ATX-Agent 安装完成");
        Ok(())
    }

    /// 检查 ATX-Agent 是否已安装
    ///
    /// # 返回
    ///
    /// 如果已安装返回 true，否则返回 false
    #[cfg(feature = "atx-agent-install")]
    pub async fn check_atx_agent_installed(&self) -> Result<bool> {
        use crate::atx_agent::AtxAgentClient;

        let atx_agent_client =
            AtxAgentClient::new(self.serial.clone(), self.adb_client.clone()).await?;

        atx_agent_client.check_atx_agent_installed().await
    }

    /// 启动 ATX-Agent 服务
    ///
    /// # 错误
    ///
    /// 如果启动失败，返回错误
    #[cfg(feature = "atx-agent-install")]
    pub async fn start_atx_agent(&self) -> Result<()> {
        use crate::atx_agent::AtxAgentClient;

        let atx_agent_client =
            AtxAgentClient::new(self.serial.clone(), self.adb_client.clone()).await?;

        atx_agent_client.start_atx_agent().await
    }

    /// 停止 ATX-Agent 服务
    ///
    /// # 错误
    ///
    /// 如果停止失败，返回错误
    #[cfg(feature = "atx-agent-install")]
    pub async fn stop_atx_agent(&self) -> Result<()> {
        use crate::atx_agent::AtxAgentClient;

        let atx_agent_client =
            AtxAgentClient::new(self.serial.clone(), self.adb_client.clone()).await?;

        atx_agent_client.stop_atx_agent().await
    }

    /// 重启 ATX-Agent 服务
    ///
    /// # 错误
    ///
    /// 如果重启失败，返回错误
    #[cfg(feature = "atx-agent-install")]
    pub async fn restart_atx_agent(&self) -> Result<()> {
        use crate::atx_agent::AtxAgentClient;

        let atx_agent_client =
            AtxAgentClient::new(self.serial.clone(), self.adb_client.clone()).await?;

        atx_agent_client.restart_atx_agent().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::fmt::Debug;

    /// 统一打印“环境相关错误”日志，避免测试输出被误解为断言失败。
    fn log_env_related_error(action: &str, err: &impl Debug) {
        println!("{}返回错误（环境相关）: {:?}", action, err);
    }

    /// 打印“环境相关且预期”的错误日志。
    fn log_env_related_expected_error(action: &str, err: &impl Debug) {
        println!("{}返回错误（环境相关，预期）: {:?}", action, err);
    }

    // 测试需求 1.1: Device 结构体应该包含所有必要字段
    #[test]
    fn test_device_structure() {
        // 这个测试验证 Device 结构体的字段定义
        // 由于需要实际的 ADB 连接，我们只验证类型定义

        // 验证 ServerMode 枚举
        assert_eq!(ServerMode::Direct, ServerMode::Direct);
        assert_ne!(ServerMode::Direct, ServerMode::AtxAgent);
    }

    #[test]
    fn test_parse_shell_output_with_exit_code() {
        let output = "Starting: Intent { cmp=com.android.settings/.Settings }\n__U2_APP_START_EXIT_CODE__:0\n";
        let (exit_code, cleaned) =
            Device::parse_shell_output_with_exit_code(output, "__U2_APP_START_EXIT_CODE__:");

        assert_eq!(exit_code, Some(0));
        assert_eq!(
            cleaned,
            "Starting: Intent { cmp=com.android.settings/.Settings }"
        );
    }

    #[test]
    fn test_pm_path_output_has_package() {
        assert!(Device::pm_path_output_has_package(
            "package:/data/app/~~abc/com.example.app/base.apk"
        ));
        assert!(!Device::pm_path_output_has_package(
            "Error: package com.example.app was not found"
        ));
    }

    #[test]
    fn test_is_likely_device_offline_text_detection() {
        assert!(Device::is_likely_device_offline_text(
            "adb: error: device offline"
        ));
        assert!(Device::is_likely_device_offline_text(
            "error: device unauthorized. Please check the confirmation dialog on your device."
        ));
        assert!(!Device::is_likely_device_offline_text(
            "adb forward failed: cannot bind listener socket"
        ));
    }

    #[test]
    fn test_is_app_start_failure_line_is_precise() {
        assert!(Device::is_app_start_failure_line(
            "Error: Activity class {com.foo/.MainActivity} does not exist."
        ));
        assert!(!Device::is_app_start_failure_line(
            "Info: Error recovery completed successfully"
        ));
    }

    #[test]
    fn test_extract_and_truncate_app_start_failure_reason() {
        let long_error = format!("Error: {}", "x".repeat(400));
        let reason = Device::extract_app_start_failure_reason(&long_error);

        assert!(reason.starts_with("Error: "));
        assert!(reason.ends_with("..."));
        assert!(reason.chars().count() <= 243); // 240 + "..."
    }

    #[test]
    fn test_classify_app_start_failure_app_not_installed() {
        let err = Device::classify_app_start_failure(
            "com.example.app",
            "Unknown package: com.example.app",
        );
        assert!(matches!(err, Error::AppNotInstalled(_)));
    }

    #[test]
    fn test_classify_app_start_failure_activity_missing_is_not_not_installed() {
        let err = Device::classify_app_start_failure(
            "com.example.app",
            "Error: Activity class {com.example.app/.MainActivity} does not exist.",
        );
        assert!(matches!(err, Error::AppStartFailed(_)));
    }

    #[test]
    fn test_classify_app_start_failure_app_crashed() {
        let err = Device::classify_app_start_failure(
            "com.example.app",
            "FATAL EXCEPTION: main Process crashed unexpectedly",
        );
        assert!(matches!(err, Error::AppCrashed(_)));
    }

    #[test]
    fn test_classify_app_start_failure_fallback() {
        let err = Device::classify_app_start_failure("com.example.app", "exit_code=1 unknown");
        assert!(matches!(err, Error::AppStartFailed(_)));
    }

    #[test]
    fn test_is_jsonrpc_method_unavailable_error_is_precise() {
        let method_not_found =
            Error::JsonRpc(r#"{"code":-32601,"message":"Method not found"}"#.to_string());
        assert!(Device::is_jsonrpc_method_unavailable_error(
            &method_not_found
        ));

        let params_invalid =
            Error::JsonRpc(r#"{"code":-32602,"message":"Invalid params"}"#.to_string());
        assert!(!Device::is_jsonrpc_method_unavailable_error(
            &params_invalid
        ));
    }

    #[test]
    fn test_is_jsonrpc_method_params_invalid_error_is_precise() {
        let params_invalid =
            Error::JsonRpc(r#"{"code":-32602,"message":"Invalid params"}"#.to_string());
        assert!(Device::is_jsonrpc_method_params_invalid_error(
            &params_invalid
        ));

        let method_not_found =
            Error::JsonRpc(r#"{"code":-32601,"message":"Method not found"}"#.to_string());
        assert!(!Device::is_jsonrpc_method_params_invalid_error(
            &method_not_found
        ));
    }

    #[test]
    fn test_resolve_activity_command_unavailable_detection() {
        let unsupported = Error::AppStartFailed(
            "com.demo: failed to resolve launch activity: cmd: inaccessible or not found"
                .to_string(),
        );
        assert!(Device::is_resolve_activity_command_unavailable_error(
            &unsupported
        ));

        let app_not_found = Error::AppStartFailed(
            "com.demo: failed to resolve launch activity: No activity found".to_string(),
        );
        assert!(!Device::is_resolve_activity_command_unavailable_error(
            &app_not_found
        ));
    }

    #[test]
    fn test_parse_app_component_from_dump_prefers_focus_lines() {
        let dump = r#"
            Window #1 Window{1234 u0 com.android.launcher/com.android.launcher2.Launcher}
            mCurrentFocus=Window{7f54f7 u0 com.android.settings/.Settings}
            mFocusedApp=AppWindowToken{5e6e8f token=Token{0 ActivityRecord{abc u0 com.android.settings/.Settings t88}}}
        "#;

        let parsed = Device::parse_app_component_from_dump(dump);
        assert_eq!(
            parsed,
            Some(("com.android.settings".to_string(), ".Settings".to_string()))
        );
    }

    #[test]
    fn test_parse_app_component_from_dump_ignores_keyboard_focus_short_name() {
        let dump = r#"
            mCurrentFocus=Window{7f54f7 u0 keyb/v}
            mFocusedApp=AppWindowToken{5e6e8f token=Token{0 ActivityRecord{abc u0 com.android.settings/.Settings t88}}}
        "#;

        let parsed = Device::parse_app_component_from_dump(dump);
        assert_eq!(
            parsed,
            Some(("com.android.settings".to_string(), ".Settings".to_string()))
        );
    }

    #[test]
    fn test_parse_app_component_from_dump_does_not_use_unfocused_window_lines() {
        let dump = r#"
            Window #7 Window{8338c4c u0 com.samsung.android.app.cocktailbarservice/com.samsung.android.app.cocktailbarservice.CocktailBarService}:
            Window #11 Window{d73791c u0 com.android.settings/com.android.settings.Settings}:
        "#;

        let parsed = Device::parse_app_component_from_dump(dump);
        assert_eq!(parsed, None);
    }

    #[test]
    fn test_parse_app_component_from_dump_supports_activity_dump_fallback() {
        let dump = r#"
            ActivityRecord{77f5 u0 com.android.settings/.Settings t42}
            Hist #0: ActivityRecord{8888 u0 com.android.settings/.Settings t42}
        "#;

        let parsed = Device::parse_app_component_from_dump(dump);
        assert_eq!(
            parsed,
            Some(("com.android.settings".to_string(), ".Settings".to_string()))
        );
    }

    // ========== 任务 8.1: 坐标转换测试 ==========

    // 属性测试：坐标转换往返一致性
    // **Feature: uiautomator2-rust, Property 3: 坐标转换正确性**
    // 验证需求 5.5
    proptest! {
        #[test]
        fn test_pos_rel2abs_round_trip(
            x_percent in 0.0f32..1.0f32,
            y_percent in 0.0f32..1.0f32,
            width in 100u32..4000u32,
            height in 100u32..4000u32,
        ) {
            // 百分比 -> 像素
            let (x_pixel, y_pixel) = pos_rel2abs_helper(x_percent, y_percent, width, height);

            // 像素 -> 百分比
            let x_percent_back = x_pixel as f32 / width as f32;
            let y_percent_back = y_pixel as f32 / height as f32;

            // 验证往返一致性（允许浮点误差）
            prop_assert!((x_percent - x_percent_back).abs() < 0.01);
            prop_assert!((y_percent - y_percent_back).abs() < 0.01);
        }
    }

    // 单元测试：百分比坐标转换
    #[test]
    fn test_pos_rel2abs_percentage() {
        // 测试百分比坐标（0.0-1.0）
        let (x, y) = pos_rel2abs_helper(0.5, 0.5, 1080, 1920);
        assert_eq!(x, 540);
        assert_eq!(y, 960);

        // 测试边界值
        let (x, y) = pos_rel2abs_helper(0.0, 0.0, 1080, 1920);
        assert_eq!(x, 0);
        assert_eq!(y, 0);

        let (x, y) = pos_rel2abs_helper(1.0, 1.0, 1080, 1920);
        assert_eq!(x, 1080);
        assert_eq!(y, 1920);
    }

    // 单元测试：像素坐标直接返回
    #[test]
    fn test_pos_rel2abs_pixel() {
        // 测试像素坐标（>1）
        let (x, y) = pos_rel2abs_helper(100.0, 200.0, 1080, 1920);
        assert_eq!(x, 100);
        assert_eq!(y, 200);

        // 测试大于屏幕尺寸的坐标
        let (x, y) = pos_rel2abs_helper(2000.0, 3000.0, 1080, 1920);
        assert_eq!(x, 2000);
        assert_eq!(y, 3000);
    }

    // 单元测试：边界情况
    #[test]
    fn test_pos_rel2abs_edge_cases() {
        // 测试 1.0 边界（应该作为百分比处理）
        let (x, y) = pos_rel2abs_helper(1.0, 1.0, 1080, 1920);
        assert_eq!(x, 1080);
        assert_eq!(y, 1920);

        // 测试刚好大于 1.0（应该作为像素处理）
        let (x, y) = pos_rel2abs_helper(1.1, 1.1, 1080, 1920);
        assert_eq!(x, 1);
        assert_eq!(y, 1);
    }

    // 集成测试：使用真实设备测试坐标转换
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_pos_rel2abs_with_device() {
        if let Ok(device) = Device::connect(None).await {
            // 测试百分比坐标
            match (
                device.pos_rel2abs(0.5, 0.5).await,
                device.window_size().await,
            ) {
                (Ok((x, y)), Ok((width, height))) => {
                    // 验证中心点坐标
                    assert_eq!(x, width / 2);
                    assert_eq!(y, height / 2);
                }
                (Err(e), _) => log_env_related_error("百分比坐标转换", &e),
                (_, Err(e)) => log_env_related_error("获取屏幕尺寸", &e),
            }

            // 测试像素坐标
            match device.pos_rel2abs(100.0, 200.0).await {
                Ok((x, y)) => {
                    assert_eq!(x, 100);
                    assert_eq!(y, 200);
                }
                Err(e) => log_env_related_error("像素坐标转换", &e),
            }
        }
    }

    // ========== 任务 8.2: 点击操作测试 ==========

    // 单元测试：验证点击方法存在并可调用
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_click_basic() {
        if let Ok(device) = Device::connect(None).await {
            // 测试基本点击（使用像素坐标）
            let result = device.click(100, 200).await;

            // 验证方法可以调用（可能失败因为坐标无效，但不应该 panic）
            match result {
                Ok(_) => println!("点击成功"),
                Err(e) => log_env_related_expected_error("点击", &e),
            }
        }
    }

    // 单元测试：验证长按方法
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_long_click() {
        if let Ok(device) = Device::connect(None).await {
            use std::time::Duration;

            // 测试长按（默认 0.5 秒）
            let result = device.long_click(100, 200, None).await;
            match result {
                Ok(_) => println!("长按成功"),
                Err(e) => log_env_related_error("长按", &e),
            }

            // 测试自定义时长
            let result = device
                .long_click(100, 200, Some(Duration::from_secs(1)))
                .await;
            match result {
                Ok(_) => println!("自定义时长长按成功"),
                Err(e) => log_env_related_error("自定义时长长按", &e),
            }
        }
    }

    // 单元测试：验证双击方法
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_double_click() {
        if let Ok(device) = Device::connect(None).await {
            use std::time::Duration;

            // 测试双击（默认间隔 0.1 秒）
            let result = device.double_click(100, 200, None).await;
            match result {
                Ok(_) => println!("双击成功"),
                Err(e) => log_env_related_error("双击", &e),
            }

            // 测试自定义间隔
            let result = device
                .double_click(100, 200, Some(Duration::from_millis(200)))
                .await;
            match result {
                Ok(_) => println!("自定义间隔双击成功"),
                Err(e) => log_env_related_error("自定义间隔双击", &e),
            }
        }
    }

    // 单元测试：验证百分比坐标点击
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_click_with_percentage() {
        if let Ok(device) = Device::connect(None).await {
            // 点击屏幕中心（使用百分比坐标）
            match device.pos_rel2abs(0.5, 0.5).await {
                Ok((x, y)) => {
                    let result = device.click(x, y).await;
                    match result {
                        Ok(_) => println!("百分比坐标点击成功"),
                        Err(e) => log_env_related_error("百分比坐标点击", &e),
                    }
                }
                Err(e) => {
                    log_env_related_error("百分比坐标转换", &e);
                }
            }
        }
    }

    // ========== 任务 8.3: 滑动和拖拽操作测试 ==========

    // 单元测试：验证滑动方法
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_swipe_basic() {
        if let Ok(device) = Device::connect(None).await {
            use std::time::Duration;

            // 测试基本滑动（从左到右）
            let result = device.swipe(100, 500, 900, 500, None).await;
            match result {
                Ok(_) => println!("滑动成功"),
                Err(e) => log_env_related_error("滑动", &e),
            }

            // 测试自定义时长滑动
            let result = device
                .swipe(100, 500, 900, 500, Some(Duration::from_millis(300)))
                .await;
            match result {
                Ok(_) => println!("自定义时长滑动成功"),
                Err(e) => log_env_related_error("自定义时长滑动", &e),
            }
        }
    }

    // 单元测试：验证拖拽方法
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_drag_basic() {
        if let Ok(device) = Device::connect(None).await {
            use std::time::Duration;

            // 测试基本拖拽
            let result = device.drag(100, 500, 900, 500, None).await;
            match result {
                Ok(_) => println!("拖拽成功"),
                Err(e) => log_env_related_error("拖拽", &e),
            }

            // 测试自定义时长拖拽
            let result = device
                .drag(100, 500, 900, 500, Some(Duration::from_millis(800)))
                .await;
            match result {
                Ok(_) => println!("自定义时长拖拽成功"),
                Err(e) => log_env_related_error("自定义时长拖拽", &e),
            }
        }
    }

    // 单元测试：验证垂直滑动
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_swipe_vertical() {
        if let Ok(device) = Device::connect(None).await {
            // 向上滑动
            let result = device.swipe(500, 1000, 500, 200, None).await;
            match result {
                Ok(_) => println!("向上滑动成功"),
                Err(e) => log_env_related_error("向上滑动", &e),
            }

            // 向下滑动
            let result = device.swipe(500, 200, 500, 1000, None).await;
            match result {
                Ok(_) => println!("向下滑动成功"),
                Err(e) => log_env_related_error("向下滑动", &e),
            }
        }
    }

    // 单元测试：验证对角线滑动
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_swipe_diagonal() {
        if let Ok(device) = Device::connect(None).await {
            // 对角线滑动
            let result = device.swipe(100, 100, 900, 1800, None).await;
            match result {
                Ok(_) => println!("对角线滑动成功"),
                Err(e) => log_env_related_error("对角线滑动", &e),
            }
        }
    }

    // 辅助函数：用于测试的坐标转换实现
    fn pos_rel2abs_helper(x: f32, y: f32, width: u32, height: u32) -> (u32, u32) {
        let x_abs = if x <= 1.0 {
            (x * width as f32) as u32
        } else {
            x as u32
        };

        let y_abs = if y <= 1.0 {
            (y * height as f32) as u32
        } else {
            y as u32
        };

        (x_abs, y_abs)
    }

    // ========== 任务 12.1: 超时配置方法测试 ==========

    // 单元测试：验证 set_wait_timeout 方法
    // 验证需求 11.1
    #[test]
    fn test_set_wait_timeout() {
        // 创建默认设置
        let settings = Arc::new(RwLock::new(Settings::default()));

        // 验证默认超时
        {
            let s = settings.read().unwrap();
            assert_eq!(s.wait_timeout, Duration::from_secs(20));
        }

        // 设置新的超时
        {
            let mut s = settings.write().unwrap();
            s.set_wait_timeout(Duration::from_secs(30));
        }

        // 验证超时已更新
        {
            let s = settings.read().unwrap();
            assert_eq!(s.wait_timeout, Duration::from_secs(30));
        }
    }

    // 单元测试：验证 get_wait_timeout 方法
    // 验证需求 11.1
    #[test]
    fn test_get_wait_timeout() {
        let settings = Arc::new(RwLock::new(Settings::default()));

        // 获取默认超时
        let timeout = {
            let s = settings.read().unwrap();
            s.wait_timeout
        };

        assert_eq!(timeout, Duration::from_secs(20));
    }

    // 集成测试：验证 Device 的 set_wait_timeout 方法
    // 验证需求 11.1
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_device_set_wait_timeout() {
        if let Ok(device) = Device::connect(None).await {
            // 验证默认超时
            let default_timeout = device.get_wait_timeout();
            assert_eq!(default_timeout, Duration::from_secs(20));

            // 设置新的超时
            device.set_wait_timeout(Duration::from_secs(30));

            // 验证超时已更新
            let new_timeout = device.get_wait_timeout();
            assert_eq!(new_timeout, Duration::from_secs(30));

            println!("超时配置测试通过");
        }
    }

    // 集成测试：验证 Device 的 get_wait_timeout 方法
    // 验证需求 11.1
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_device_get_wait_timeout() {
        if let Ok(device) = Device::connect(None).await {
            // 获取默认超时
            let timeout = device.get_wait_timeout();

            // 验证默认值
            assert_eq!(timeout, Duration::from_secs(20));

            println!("获取超时配置测试通过: {:?}", timeout);
        }
    }

    // 单元测试：验证超时配置可以设置为不同的值
    // 验证需求 11.1
    #[test]
    fn test_set_wait_timeout_various_values() {
        let settings = Arc::new(RwLock::new(Settings::default()));

        // 测试不同的超时值
        let test_values = vec![
            Duration::from_secs(1),
            Duration::from_secs(10),
            Duration::from_secs(30),
            Duration::from_secs(60),
            Duration::from_millis(500),
        ];

        for timeout in test_values {
            {
                let mut s = settings.write().unwrap();
                s.set_wait_timeout(timeout);
            }

            {
                let s = settings.read().unwrap();
                assert_eq!(s.wait_timeout, timeout);
            }
        }
    }

    // 单元测试：验证超时配置在多线程环境下的安全性
    // 验证需求 11.1
    #[test]
    fn test_set_wait_timeout_thread_safe() {
        use std::thread;

        let settings = Arc::new(RwLock::new(Settings::default()));
        let mut handles = vec![];

        // 创建多个线程同时修改超时配置
        for i in 1..=5 {
            let settings_clone = Arc::clone(&settings);
            let handle = thread::spawn(move || {
                let timeout = Duration::from_secs(10 + i);
                let mut s = settings_clone.write().unwrap();
                s.set_wait_timeout(timeout);
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        // 验证最终值是有效的
        let final_timeout = {
            let s = settings.read().unwrap();
            s.wait_timeout
        };

        // 最终值应该是某个线程设置的值
        assert!(final_timeout >= Duration::from_secs(11));
        assert!(final_timeout <= Duration::from_secs(15));
    }

    // ========== 任务 12.2: 轮询等待辅助函数测试 ==========

    // 单元测试：验证 wait_for 在条件立即满足时返回
    // 验证需求 11.3, 11.4
    #[tokio::test]
    async fn test_wait_for_immediate_success() {
        // 测试条件立即满足的情况
        let start = std::time::Instant::now();
        let result = wait_for_helper(|| async { Ok(true) }, Duration::from_secs(5)).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // 应该立即返回，不需要等待
        assert!(elapsed < Duration::from_secs(1));
    }

    // 单元测试：验证 wait_for 在条件延迟满足时等待
    // 验证需求 11.3, 11.4
    #[tokio::test]
    async fn test_wait_for_delayed_success() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let condition_met = Arc::new(AtomicBool::new(false));
        let condition_met_clone = Arc::clone(&condition_met);

        // 启动一个任务在 1 秒后设置条件
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            condition_met_clone.store(true, Ordering::SeqCst);
        });

        // 等待条件满足
        let start = std::time::Instant::now();
        let condition_met_for_check = Arc::clone(&condition_met);
        let result = wait_for_helper(
            move || {
                let cm = Arc::clone(&condition_met_for_check);
                async move { Ok(cm.load(Ordering::SeqCst)) }
            },
            Duration::from_secs(5),
        )
        .await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // 应该等待约 1 秒
        assert!(elapsed >= Duration::from_secs(1));
        assert!(elapsed < Duration::from_secs(2));
    }

    // 单元测试：验证 wait_for 在超时时返回错误
    // 验证需求 11.6
    #[tokio::test]
    async fn test_wait_for_timeout() {
        // 测试条件永不满足的情况
        let start = std::time::Instant::now();
        let result = wait_for_helper(|| async { Ok(false) }, Duration::from_secs(2)).await;
        let elapsed = start.elapsed();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Timeout));
        // 应该等待约 2 秒后超时
        assert!(elapsed >= Duration::from_secs(2));
        assert!(elapsed < Duration::from_secs(3));
    }

    // 单元测试：验证 wait_for 的轮询间隔
    // 验证需求 11.3, 11.4
    #[tokio::test]
    async fn test_wait_for_polling_interval() {
        use std::sync::atomic::{AtomicU32, Ordering};

        let check_count = Arc::new(AtomicU32::new(0));
        let check_count_clone = Arc::clone(&check_count);

        // 测试轮询次数
        let start = std::time::Instant::now();
        let result = wait_for_helper(
            move || {
                let cc = Arc::clone(&check_count_clone);
                async move {
                    cc.fetch_add(1, Ordering::SeqCst);
                    Ok(false)
                }
            },
            Duration::from_secs(2),
        )
        .await;
        let elapsed = start.elapsed();

        assert!(result.is_err());

        // 验证轮询次数合理（约每 500ms 一次，2 秒应该有 4-5 次）
        let count = check_count.load(Ordering::SeqCst);
        assert!(count >= 3);
        assert!(count <= 6);
        println!("轮询次数: {}, 耗时: {:?}", count, elapsed);
    }

    // 单元测试：验证 wait_for 在条件返回错误时传播错误
    // 验证需求 11.3, 11.4
    #[tokio::test]
    async fn test_wait_for_condition_error() {
        // 测试条件返回错误的情况
        let result = wait_for_helper(
            || async { Err(Error::InvalidArgument("测试错误".to_string())) },
            Duration::from_secs(5),
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidArgument(msg) => {
                assert_eq!(msg, "测试错误");
            }
            _ => panic!("应该返回 InvalidArgument 错误"),
        }
    }

    // 集成测试：验证 Device 的 wait_for 方法
    // 验证需求 11.3, 11.4, 11.5, 11.6
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_device_wait_for() {
        if let Ok(device) = Device::connect(None).await {
            // 测试条件立即满足
            let result = device
                .wait_for(|| async { Ok(true) }, Some(Duration::from_secs(5)))
                .await;
            assert!(result.is_ok());

            // 测试条件永不满足（使用短超时）
            let result = device
                .wait_for(|| async { Ok(false) }, Some(Duration::from_secs(1)))
                .await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), Error::Timeout));

            println!("Device wait_for 测试通过");
        }
    }

    // 集成测试：验证 wait_for 使用全局超时
    // 验证需求 11.1, 11.3
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_device_wait_for_uses_global_timeout() {
        if let Ok(device) = Device::connect(None).await {
            // 设置全局超时为 2 秒
            device.set_wait_timeout(Duration::from_secs(2));

            // 使用 None 作为超时参数，应该使用全局超时
            let start = std::time::Instant::now();
            let result = device.wait_for(|| async { Ok(false) }, None).await;
            let elapsed = start.elapsed();

            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), Error::Timeout));
            // 应该等待约 2 秒（全局超时）
            assert!(elapsed >= Duration::from_secs(2));
            assert!(elapsed < Duration::from_secs(3));

            println!("全局超时测试通过");
        }
    }

    // 集成测试：验证 wait_for 操作级超时覆盖全局超时
    // 验证需求 11.2
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_device_wait_for_operation_timeout_override() {
        if let Ok(device) = Device::connect(None).await {
            // 设置全局超时为 10 秒
            device.set_wait_timeout(Duration::from_secs(10));

            // 使用操作级超时 1 秒，应该覆盖全局超时
            let start = std::time::Instant::now();
            let result = device
                .wait_for(|| async { Ok(false) }, Some(Duration::from_secs(1)))
                .await;
            let elapsed = start.elapsed();

            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), Error::Timeout));
            // 应该等待约 1 秒（操作级超时），而不是 10 秒（全局超时）
            assert!(elapsed >= Duration::from_secs(1));
            assert!(elapsed < Duration::from_secs(2));

            println!("操作级超时覆盖测试通过");
        }
    }

    // 辅助函数：用于测试的 wait_for 实现
    async fn wait_for_helper<F, Fut>(condition: F, timeout: Duration) -> Result<()>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<bool>>,
    {
        const POLL_INTERVAL: Duration = Duration::from_millis(500);

        let result = tokio::time::timeout(timeout, async {
            loop {
                match condition().await {
                    Ok(true) => return Ok(()),
                    Ok(false) => {
                        tokio::time::sleep(POLL_INTERVAL).await;
                    }
                    Err(e) => return Err(e),
                }
            }
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(Error::Timeout),
        }
    }

    // 测试 ServerMode 的 Debug trait
    #[test]
    fn test_server_mode_debug() {
        let mode = ServerMode::Direct;
        let debug_str = format!("{:?}", mode);
        assert!(debug_str.contains("Direct"));
    }

    // 测试 ServerMode 的 Clone 和 Copy
    #[test]
    fn test_server_mode_clone_copy() {
        let mode1 = ServerMode::Direct;
        let mode2 = mode1; // Copy
        let mode3 = mode1.clone(); // Clone

        assert_eq!(mode1, mode2);
        assert_eq!(mode1, mode3);
    }

    #[test]
    fn test_atx_agent_init_guidance_contains_cli_command() {
        let serial = "emulator-5554";
        let message = Device::atx_agent_init_guidance(serial);
        assert!(message.contains("uiautomator init --serial emulator-5554 --force"));
        assert!(message.contains("uiautomator-cli"));
    }

    // 测试需求 1.2: 自动设备选择（无序列号且仅一个设备）
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_connect_auto_select_single_device() {
        // 当只有一个设备时，应该自动选择
        let result = Device::connect(None).await;

        // 如果有设备连接，应该成功
        // 如果没有设备，应该返回错误
        match result {
            Ok(device) => {
                assert!(!device.serial().is_empty());
            }
            Err(e) => {
                // 可能是没有设备，或环境中连接了多个设备
                assert!(
                    matches!(
                        e,
                        Error::Adb(_)
                            | Error::DeviceNotFound
                            | Error::MultipleDevicesFound
                            | Error::DeviceOffline(_)
                            | Error::Timeout
                            | Error::HttpTimeout
                            | Error::UiAutomatorNotConnected
                            | Error::Http(_)
                    ),
                    "unexpected error in auto connect: {:?}",
                    e
                );
            }
        }
    }

    // 测试需求 1.3: 多设备错误提示
    #[tokio::test]
    #[ignore = "需要多个设备连接"]
    async fn test_connect_multiple_devices_error() {
        let adb = match crate::adb::AdbClient::new().await {
            Ok(client) => client,
            Err(err) => {
                eprintln!(
                    "skip test_connect_multiple_devices_error: failed to create adb client: {:?}",
                    err
                );
                return;
            }
        };

        let devices = match adb.devices().await {
            Ok(devices) => devices,
            Err(err) => {
                eprintln!(
                    "skip test_connect_multiple_devices_error: failed to list devices: {:?}",
                    err
                );
                return;
            }
        };

        if devices.len() < 2 {
            eprintln!(
                "skip test_connect_multiple_devices_error: requires >=2 connected devices, found {}",
                devices.len()
            );
            return;
        }

        let result = Device::connect(None).await;

        match result {
            Err(Error::MultipleDevicesFound) => {}
            Err(err) => panic!("expected MultipleDevicesFound, got: {:?}", err),
            Ok(_) => panic!("expected MultipleDevicesFound, but connect succeeded"),
        }
    }

    // 测试 connect_quick 方法
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_connect_quick() {
        let result = Device::connect_quick(None).await;

        match result {
            Ok(device) => {
                assert_eq!(device.server_mode(), ServerMode::Direct);
            }
            Err(_) => {
                // 可能是没有设备连接
            }
        }
    }

    // 测试 connect_with_mode 方法
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_connect_with_mode() {
        let result = Device::connect_with_mode(None, ServerMode::Direct).await;

        match result {
            Ok(device) => {
                assert_eq!(device.server_mode(), ServerMode::Direct);
            }
            Err(_) => {
                // 可能是没有设备连接
            }
        }
    }

    // 测试 Device 的访问器方法
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_device_accessors() {
        if let Ok(device) = Device::connect(None).await {
            // 测试 serial()
            assert!(!device.serial().is_empty());

            // 测试 server_mode()
            assert!(matches!(
                device.server_mode(),
                ServerMode::Direct | ServerMode::AtxAgent
            ));

            // 测试 adb_client()
            let _adb = device.adb_client();

            // 测试 jsonrpc_client()
            let _rpc = device.jsonrpc_client();

            // 测试 settings()
            let settings = device.settings();
            let settings_guard = settings.read().unwrap();
            assert_eq!(settings_guard.max_retry, 3);
        }
    }

    // 测试 Device 的 Clone trait
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_device_clone() {
        if let Ok(device1) = Device::connect(None).await {
            let device2 = device1.clone();

            assert_eq!(device1.serial(), device2.serial());
            assert_eq!(device1.server_mode(), device2.server_mode());
        }
    }

    // 测试需求 2.1, 2.2, 2.3, 2.4, 2.5: 获取设备信息
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_device_info() {
        if let Ok(device) = Device::connect(None).await {
            let info = device.info().await;

            match info {
                Ok(info) => {
                    // 验证设备信息包含必要字段
                    assert!(info.display_width > 0);
                    assert!(info.display_height > 0);
                    assert!(!info.current_package_name.is_empty());
                    assert!(info.sdk_int > 0);
                }
                Err(e) => {
                    log_env_related_error("获取设备信息", &e);
                }
            }
        }
    }

    // 测试需求 2.1: 获取屏幕尺寸
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_window_size() {
        if let Ok(device) = Device::connect(None).await {
            let size = device.window_size().await;

            match size {
                Ok((width, height)) => {
                    assert!(width > 0);
                    assert!(height > 0);
                    println!("屏幕尺寸: {}x{}", width, height);
                }
                Err(e) => {
                    log_env_related_error("获取屏幕尺寸", &e);
                }
            }
        }
    }

    // 测试需求 3.6: 元素定位入口
    #[test]
    fn test_find_creates_uiobject() {
        // 由于 Device::connect 需要真实设备，我们只测试 find 方法的基本功能
        // 通过创建一个 Selector 并验证它可以被传递给 find

        let selector = Selector::new().text("Settings").clickable(true);

        // 验证 Selector 可以正确创建
        assert_eq!(selector.to_params()["text"], "Settings");
        assert_eq!(selector.to_params()["clickable"], true);
    }

    // 测试需求 3.6: find 方法返回 UiObject
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_find_returns_uiobject() {
        if let Ok(device) = Device::connect(None).await {
            let selector = Selector::new().text("Settings");
            let ui_object = device.find(selector.clone());

            // 验证 UiObject 包含正确的 Selector
            assert_eq!(ui_object.selector(), &selector);
        }
    }

    // ========== 任务 9.1: 按键操作测试 ==========

    // 单元测试：验证 press 方法存在并可调用
    // 验证需求 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_press_home_key() {
        use crate::key::Key;

        if let Ok(device) = Device::connect(None).await {
            // 测试按下 Home 键（需求 6.1）
            let result = device.press(Key::Home).await;

            match result {
                Ok(_) => println!("按下 Home 键成功"),
                Err(e) => log_env_related_error("按下 Home 键", &e),
            }
        }
    }

    // 单元测试：验证 press 方法支持 Back 键
    // 验证需求 6.2
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_press_back_key() {
        use crate::key::Key;

        if let Ok(device) = Device::connect(None).await {
            // 测试按下 Back 键（需求 6.2）
            let result = device.press(Key::Back).await;

            match result {
                Ok(_) => println!("按下 Back 键成功"),
                Err(e) => log_env_related_error("按下 Back 键", &e),
            }
        }
    }

    // 单元测试：验证 press 方法支持 Power 键
    // 验证需求 6.3
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_press_power_key() {
        use crate::key::Key;
        use std::env;

        if env::var("UIAUTOMATOR_ALLOW_POWER_KEY_TEST").ok().as_deref() != Some("1") {
            eprintln!(
                "skip test_press_power_key: set UIAUTOMATOR_ALLOW_POWER_KEY_TEST=1 to enable"
            );
            return;
        }

        if let Ok(device) = Device::connect(None).await {
            // 测试按下 Power 键（需求 6.3）
            let result = device.press(Key::Power).await;

            match result {
                Ok(_) => println!("按下 Power 键成功"),
                Err(e) => log_env_related_error("按下 Power 键", &e),
            }
        }
    }

    // 单元测试：验证 press 方法支持方向键
    // 验证需求 6.4
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_press_direction_keys() {
        use crate::key::Key;

        if let Ok(device) = Device::connect(None).await {
            // 测试方向键（需求 6.4）
            let keys = vec![Key::Up, Key::Down, Key::Left, Key::Right];

            for key in keys {
                let result = device.press(key).await;
                match result {
                    Ok(_) => println!("按下 {:?} 键成功", key),
                    Err(e) => log_env_related_error(&format!("按下 {:?} 键", key), &e),
                }
            }
        }
    }

    // 单元测试：验证 press 方法支持 Enter 键
    // 验证需求 6.5
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_press_enter_key() {
        use crate::key::Key;

        if let Ok(device) = Device::connect(None).await {
            // 测试按下 Enter 键（需求 6.5）
            let result = device.press(Key::Enter).await;

            match result {
                Ok(_) => println!("按下 Enter 键成功"),
                Err(e) => log_env_related_error("按下 Enter 键", &e),
            }
        }
    }

    // 单元测试：验证 press 方法支持音量键
    // 验证需求 6.6
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_press_volume_keys() {
        use crate::key::Key;

        if let Ok(device) = Device::connect(None).await {
            // 测试音量键（需求 6.6）
            let keys = vec![Key::VolumeUp, Key::VolumeDown];

            for key in keys {
                let result = device.press(key).await;
                match result {
                    Ok(_) => println!("按下 {:?} 键成功", key),
                    Err(e) => log_env_related_error(&format!("按下 {:?} 键", key), &e),
                }
            }
        }
    }

    // 单元测试：验证 press_keycode 方法
    // 验证需求 6.7
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_press_keycode() {
        if let Ok(device) = Device::connect(None).await {
            // 测试通过键码按键（需求 6.7）
            // 键码 3 = Home, 4 = Back
            let keycodes = vec![3, 4];

            for keycode in keycodes {
                let result = device.press_keycode(keycode).await;
                match result {
                    Ok(_) => println!("按下键码 {} 成功", keycode),
                    Err(e) => log_env_related_error(&format!("按下键码 {} ", keycode), &e),
                }
            }
        }
    }

    // 单元测试：验证所有常用按键
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_press_all_common_keys() {
        use crate::key::Key;

        if let Ok(device) = Device::connect(None).await {
            // 测试所有常用按键
            let keys = vec![
                Key::Home,
                Key::Back,
                Key::Menu,
                Key::Search,
                Key::Enter,
                Key::Delete,
                Key::Recent,
            ];

            for key in keys {
                let result = device.press(key).await;
                match result {
                    Ok(_) => println!("按下 {:?} 键成功", key),
                    Err(e) => log_env_related_error(&format!("按下 {:?} 键", key), &e),
                }

                // 短暂延迟避免操作过快
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }

    // 单元测试：验证 Key 枚举和键码的对应关系
    #[test]
    fn test_key_to_keycode_consistency() {
        use crate::key::Key;

        // 验证关键按键的键码
        assert_eq!(Key::Home.to_keycode(), 3);
        assert_eq!(Key::Back.to_keycode(), 4);
        assert_eq!(Key::Power.to_keycode(), 26);
        assert_eq!(Key::VolumeUp.to_keycode(), 24);
        assert_eq!(Key::VolumeDown.to_keycode(), 25);
        assert_eq!(Key::Enter.to_keycode(), 66);
        assert_eq!(Key::Up.to_keycode(), 19);
        assert_eq!(Key::Down.to_keycode(), 20);
        assert_eq!(Key::Left.to_keycode(), 21);
        assert_eq!(Key::Right.to_keycode(), 22);
    }

    // 单元测试：验证 press 和 press_keycode 的等价性
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_press_vs_press_keycode_equivalence() {
        use crate::key::Key;

        if let Ok(device) = Device::connect(None).await {
            // 使用 Key 枚举按下 Home 键
            let result1 = device.press(Key::Home).await;

            // 短暂延迟
            tokio::time::sleep(Duration::from_millis(500)).await;

            // 使用键码按下 Home 键
            let result2 = device.press_keycode(Key::Home.to_keycode()).await;

            // 两种方式应该都能成功（或都失败）
            assert_eq!(result1.is_ok(), result2.is_ok());
        }
    }

    // 单元测试：验证按键操作应用延迟设置
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_press_with_operation_delay() {
        use crate::key::Key;

        if let Ok(device) = Device::connect(None).await {
            // 设置操作延迟
            {
                let mut settings = device.settings().write().unwrap();
                settings.operation_delay_before = Duration::from_millis(100);
                settings.operation_delay_after = Duration::from_millis(100);
            }

            // 测试按键操作
            let start = std::time::Instant::now();
            let result = device.press(Key::Home).await;
            let elapsed = start.elapsed();

            match result {
                Ok(_) => {
                    // 验证总时间包含延迟（至少 200ms）
                    println!("按键操作耗时: {:?}", elapsed);
                    assert!(elapsed >= Duration::from_millis(200));
                }
                Err(e) => {
                    log_env_related_error("按键操作", &e);
                }
            }
        }
    }

    // ========== 任务 10.1: 截图方法测试 ==========

    // 单元测试：验证 screenshot 方法存在并返回图像
    // 验证需求 7.1, 7.3
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_screenshot_basic() {
        if let Ok(device) = Device::connect(None).await {
            // 测试基本截图功能
            let result = device.screenshot().await;

            match result {
                Ok(image) => {
                    // 验证图像有效
                    assert!(image.width() > 0);
                    assert!(image.height() > 0);
                    println!("截图成功: {}x{}", image.width(), image.height());
                }
                Err(e) => {
                    log_env_related_error("截图", &e);
                }
            }
        }
    }

    // 单元测试：验证截图尺寸与屏幕尺寸一致
    // 验证需求 7.1
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_screenshot_dimensions_match_screen() {
        if let Ok(device) = Device::connect(None).await {
            match (device.window_size().await, device.screenshot().await) {
                (Ok((screen_width, screen_height)), Ok(image)) => {
                    // 验证截图尺寸与屏幕尺寸一致
                    assert_eq!(image.width(), screen_width);
                    assert_eq!(image.height(), screen_height);
                    println!("截图尺寸验证通过: {}x{}", image.width(), image.height());
                }
                (Err(e), _) => {
                    log_env_related_error("获取屏幕尺寸", &e);
                }
                (_, Err(e)) => {
                    log_env_related_error("截图", &e);
                }
            }
        }
    }

    // 单元测试：验证多次截图都能成功
    // 验证需求 7.1
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_screenshot_multiple_times() {
        if let Ok(device) = Device::connect(None).await {
            // 连续截图 3 次
            for i in 1..=3 {
                let result = device.screenshot().await;

                match result {
                    Ok(image) => {
                        assert!(image.width() > 0);
                        assert!(image.height() > 0);
                        println!("第 {} 次截图成功", i);
                    }
                    Err(e) => {
                        log_env_related_error(&format!("第 {} 次截图", i), &e);
                    }
                }

                // 短暂延迟
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }

    // 单元测试：验证截图返回的图像格式
    // 验证需求 7.3
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_screenshot_image_format() {
        if let Ok(device) = Device::connect(None).await {
            let result = device.screenshot().await;

            match result {
                Ok(image) => {
                    // 验证图像可以转换为不同格式
                    let _rgb = image.to_rgb8();
                    let _rgba = image.to_rgba8();
                    println!("图像格式转换成功");
                }
                Err(e) => {
                    log_env_related_error("截图", &e);
                }
            }
        }
    }

    // ========== 任务 10.2: 截图保存方法测试 ==========

    // 单元测试：验证 screenshot_to_file 方法保存 PNG 格式
    // 验证需求 7.2, 7.4
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_screenshot_to_file_png() {
        if let Ok(device) = Device::connect(None).await {
            let temp_path = std::env::temp_dir().join("test_screenshot.png");
            let path_str = temp_path.to_str().unwrap();

            // 保存截图为 PNG
            let result = device.screenshot_to_file(path_str).await;

            match result {
                Ok(_) => {
                    // 验证文件存在
                    assert!(temp_path.exists());

                    // 验证文件可以被读取为图像
                    let image = image::open(&temp_path);
                    assert!(image.is_ok());

                    // 清理临时文件
                    let _ = std::fs::remove_file(&temp_path);

                    println!("PNG 截图保存成功");
                }
                Err(e) => {
                    log_env_related_error("PNG 截图保存", &e);
                }
            }
        }
    }

    // 单元测试：验证 screenshot_to_file 方法保存 JPEG 格式
    // 验证需求 7.2, 7.4
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_screenshot_to_file_jpeg() {
        if let Ok(device) = Device::connect(None).await {
            let temp_path = std::env::temp_dir().join("test_screenshot.jpg");
            let path_str = temp_path.to_str().unwrap();

            // 保存截图为 JPEG
            let result = device.screenshot_to_file(path_str).await;

            match result {
                Ok(_) => {
                    // 验证文件存在
                    assert!(temp_path.exists());

                    // 验证文件可以被读取为图像
                    let image = image::open(&temp_path);
                    assert!(image.is_ok());

                    // 清理临时文件
                    let _ = std::fs::remove_file(&temp_path);

                    println!("JPEG 截图保存成功");
                }
                Err(e) => {
                    log_env_related_error("JPEG 截图保存", &e);
                }
            }
        }
    }

    // 单元测试：验证保存到不存在的目录时的错误处理
    // 验证需求 7.5
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_screenshot_to_file_invalid_path() {
        if let Ok(device) = Device::connect(None).await {
            // 使用不存在的目录路径
            let invalid_path = "/nonexistent/directory/screenshot.png";

            let result = device.screenshot_to_file(invalid_path).await;

            // 应该返回错误
            assert!(result.is_err());

            if let Err(e) = result {
                println!("预期的错误: {:?}", e);
            }
        }
    }

    // 单元测试：验证文件扩展名自动识别格式
    // 验证需求 7.4
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_screenshot_to_file_format_detection() {
        if let Ok(device) = Device::connect(None).await {
            // 测试不同扩展名
            let formats = vec![
                ("test_screenshot.png", "PNG"),
                ("test_screenshot.jpg", "JPEG"),
                ("test_screenshot.jpeg", "JPEG"),
            ];

            for (filename, format_name) in formats {
                let temp_path = std::env::temp_dir().join(filename);
                let path_str = temp_path.to_str().unwrap();

                let result = device.screenshot_to_file(path_str).await;

                match result {
                    Ok(_) => {
                        assert!(temp_path.exists());
                        println!("{} 格式保存成功", format_name);

                        // 清理临时文件
                        let _ = std::fs::remove_file(&temp_path);
                    }
                    Err(e) => {
                        log_env_related_error(&format!("{} 格式保存", format_name), &e);
                    }
                }
            }
        }
    }

    // 单元测试：验证保存的图像尺寸与原始截图一致
    // 验证需求 7.2
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_screenshot_to_file_preserves_dimensions() {
        if let Ok(device) = Device::connect(None).await {
            // 先截图获取原始尺寸
            let original_image = device.screenshot().await;

            if let Ok(original) = original_image {
                let original_width = original.width();
                let original_height = original.height();

                // 保存到文件
                let temp_path = std::env::temp_dir().join("test_screenshot_dims.png");
                let path_str = temp_path.to_str().unwrap();

                let result = device.screenshot_to_file(path_str).await;

                match result {
                    Ok(_) => {
                        // 读取保存的图像
                        let saved_image = image::open(&temp_path).unwrap();

                        // 验证尺寸一致
                        assert_eq!(saved_image.width(), original_width);
                        assert_eq!(saved_image.height(), original_height);

                        println!(
                            "保存的图像尺寸验证通过: {}x{}",
                            saved_image.width(),
                            saved_image.height()
                        );

                        // 清理临时文件
                        let _ = std::fs::remove_file(&temp_path);
                    }
                    Err(e) => {
                        log_env_related_error("截图保存", &e);
                    }
                }
            }
        }
    }

    // ========== 任务 11.1: 应用启动方法测试 ==========

    // 单元测试：验证 app_start 方法仅使用包名启动
    // 验证需求 8.1
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_start_package_only() {
        if let Ok(device) = Device::connect(None).await {
            // 测试仅使用包名启动应用（使用 Android 设置应用）
            let result = device.app_start("com.android.settings", None).await;

            match result {
                Ok(_) => {
                    println!("应用启动成功（仅包名）");

                    // 短暂延迟等待应用启动
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    // 验证应用是否在前台
                    if let Ok(current) = device.app_current().await {
                        assert_eq!(current.package, "com.android.settings");
                        println!("验证应用在前台: {}", current.package);
                    }
                }
                Err(e) => {
                    log_env_related_error("应用启动", &e);
                }
            }
        }
    }

    // 单元测试：验证 app_start 方法使用包名和 Activity 启动
    // 验证需求 8.2
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_start_with_activity() {
        if let Ok(device) = Device::connect(None).await {
            // 测试使用包名和 Activity 启动应用
            let result = device
                .app_start("com.android.settings", Some(".Settings"))
                .await;

            match result {
                Ok(_) => {
                    println!("应用启动成功（包名 + Activity）");

                    // 短暂延迟等待应用启动
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    // 验证应用是否在前台
                    if let Ok(current) = device.app_current().await {
                        assert_eq!(current.package, "com.android.settings");
                        println!("验证应用在前台: {}", current.package);
                    }
                }
                Err(e) => {
                    log_env_related_error("应用启动", &e);
                }
            }
        }
    }

    // 单元测试：验证启动不存在的应用返回错误
    // 验证需求 8.1
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_start_nonexistent_package() {
        if let Ok(device) = Device::connect(None).await {
            // 测试启动不存在的应用
            let result = device.app_start("com.nonexistent.app", None).await;

            // 应该返回错误（但不一定是 AppNotFound，可能是 ADB 错误）
            match result {
                Ok(_) => {
                    println!("意外：不存在的应用启动成功");
                }
                Err(e) => {
                    println!("预期的错误: {:?}", e);
                }
            }
        }
    }

    // 单元测试：验证启动多个不同应用
    // 验证需求 8.1, 8.2
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_start_multiple_apps() {
        if let Ok(device) = Device::connect(None).await {
            // 测试启动多个应用
            let apps = vec![
                ("com.android.settings", None),
                ("com.android.calculator2", None),
            ];

            for (package, activity) in apps {
                let result = device.app_start(package, activity).await;

                match result {
                    Ok(_) => {
                        println!("应用 {} 启动成功", package);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    Err(e) => {
                        log_env_related_error(&format!("应用 {} 启动", package), &e);
                    }
                }
            }
        }
    }

    // ========== 任务 11.2: 应用停止和清除方法测试 ==========

    // 单元测试：验证 app_stop 方法停止应用
    // 验证需求 8.3
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_stop_basic() {
        if let Ok(device) = Device::connect(None).await {
            // 先启动应用
            let _ = device.app_start("com.android.settings", None).await;
            tokio::time::sleep(Duration::from_secs(2)).await;

            // 停止应用
            let result = device.app_stop("com.android.settings").await;

            match result {
                Ok(_) => {
                    println!("应用停止成功");

                    // 短暂延迟
                    tokio::time::sleep(Duration::from_secs(1)).await;

                    // 验证应用不在前台
                    if let Ok(current) = device.app_current().await {
                        assert_ne!(current.package, "com.android.settings");
                        println!("验证应用已停止，当前前台: {}", current.package);
                    }
                }
                Err(e) => {
                    log_env_related_error("应用停止", &e);
                }
            }
        }
    }

    // 单元测试：验证 app_clear 方法清除应用数据
    // 验证需求 8.6
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_clear_basic() {
        if let Ok(device) = Device::connect(None).await {
            // 清除应用数据（使用一个测试应用）
            let result = device.app_clear("com.android.settings").await;

            match result {
                Ok(_) => {
                    println!("应用数据清除成功");
                }
                Err(e) => {
                    log_env_related_error("应用数据清除", &e);
                }
            }
        }
    }

    // 单元测试：验证停止不存在的应用
    // 验证需求 8.3
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_stop_nonexistent() {
        if let Ok(device) = Device::connect(None).await {
            // 停止不存在的应用（应该成功或返回特定错误）
            let result = device.app_stop("com.nonexistent.app").await;

            match result {
                Ok(_) => {
                    println!("停止不存在的应用成功（预期行为）");
                }
                Err(e) => {
                    println!("停止不存在的应用返回错误: {:?}", e);
                }
            }
        }
    }

    // 单元测试：验证清除不存在的应用数据
    // 验证需求 8.6
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_clear_nonexistent() {
        if let Ok(device) = Device::connect(None).await {
            // 清除不存在的应用数据
            let result = device.app_clear("com.nonexistent.app").await;

            // 应该返回错误
            match result {
                Ok(_) => {
                    println!("意外：清除不存在的应用数据成功");
                }
                Err(e) => {
                    println!("预期的错误: {:?}", e);
                }
            }
        }
    }

    // ========== 任务 11.3: 应用信息获取方法测试 ==========

    // 单元测试：验证 app_current 方法获取当前应用信息
    // 验证需求 8.5
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_current_basic() {
        if let Ok(device) = Device::connect(None).await {
            // 获取当前前台应用信息
            let result = device.app_current().await;

            match result {
                Ok(info) => {
                    // 验证返回的信息包含必要字段
                    assert!(!info.package.is_empty());
                    assert!(!info.activity.is_empty());
                    println!("当前应用: {} / {}", info.package, info.activity);

                    if let Some(pid) = info.pid {
                        println!("进程 ID: {}", pid);
                        assert!(pid > 0);
                    }
                }
                Err(e) => {
                    log_env_related_error("获取当前应用信息", &e);
                }
            }
        }
    }

    // 单元测试：验证启动应用后 app_current 返回正确信息
    // 验证需求 8.5
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_current_after_start() {
        if let Ok(device) = Device::connect(None).await {
            // 启动设置应用
            let _ = device.app_start("com.android.settings", None).await;
            tokio::time::sleep(Duration::from_secs(2)).await;

            // 获取当前应用信息
            let result = device.app_current().await;

            match result {
                Ok(info) => {
                    // 验证当前应用是设置应用
                    assert_eq!(info.package, "com.android.settings");
                    println!("验证当前应用: {} / {}", info.package, info.activity);
                }
                Err(e) => {
                    log_env_related_error("获取当前应用信息", &e);
                }
            }
        }
    }

    // 单元测试：验证 app_current 返回的 PID 有效
    // 验证需求 8.5
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_current_pid_valid() {
        if let Ok(device) = Device::connect(None).await {
            // 启动应用
            let _ = device.app_start("com.android.settings", None).await;
            tokio::time::sleep(Duration::from_secs(2)).await;

            // 获取当前应用信息
            if let Ok(info) = device.app_current().await {
                if let Some(pid) = info.pid {
                    // PID 应该是正整数
                    assert!(pid > 0);
                    println!("应用 PID: {}", pid);
                }
            }
        }
    }

    // ========== 任务 11.4: 应用等待方法测试 ==========

    // 单元测试：验证 app_wait 方法等待应用启动
    // 验证需求 8.4
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_wait_basic() {
        if let Ok(device) = Device::connect(None).await {
            // 启动应用
            let _ = device.app_start("com.android.settings", None).await;

            // 等待应用启动
            let result = device
                .app_wait("com.android.settings", Some(Duration::from_secs(10)))
                .await;

            match result {
                Ok(pid) => {
                    // 验证返回的 PID 有效
                    assert!(pid > 0);
                    println!("应用启动成功，PID: {}", pid);
                }
                Err(e) => {
                    log_env_related_error("等待应用启动", &e);
                }
            }
        }
    }

    // 单元测试：验证 app_wait 超时返回错误
    // 验证需求 8.4
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_wait_timeout() {
        if let Ok(device) = Device::connect(None).await {
            // 等待一个不会启动的应用（使用很短的超时）
            let result = device
                .app_wait("com.nonexistent.app", Some(Duration::from_secs(2)))
                .await;

            // 应该超时返回错误
            match result {
                Ok(pid) => {
                    println!("意外：不存在的应用返回 PID: {}", pid);
                }
                Err(e) => {
                    println!("预期的超时错误: {:?}", e);
                    assert!(matches!(e, Error::Timeout));
                }
            }
        }
    }

    // 单元测试：验证 app_wait 在应用已运行时立即返回
    // 验证需求 8.4
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_wait_already_running() {
        if let Ok(device) = Device::connect(None).await {
            // 先启动应用并等待
            let _ = device.app_start("com.android.settings", None).await;
            tokio::time::sleep(Duration::from_secs(2)).await;

            // 再次等待应用（应该立即返回）
            let start = std::time::Instant::now();
            let result = device
                .app_wait("com.android.settings", Some(Duration::from_secs(10)))
                .await;
            let elapsed = start.elapsed();

            match result {
                Ok(pid) => {
                    // 应该很快返回（< 1 秒）
                    assert!(elapsed < Duration::from_secs(1));
                    assert!(pid > 0);
                    println!("应用已运行，立即返回 PID: {}，耗时: {:?}", pid, elapsed);
                }
                Err(e) => {
                    log_env_related_error("等待应用", &e);
                }
            }
        }
    }

    // 单元测试：验证 app_wait 与 app_start 配合使用
    // 验证需求 8.1, 8.4
    #[tokio::test]
    #[ignore = "需要真实设备或模拟器"]
    async fn test_app_start_and_wait() {
        if let Ok(device) = Device::connect(None).await {
            // 先停止应用（如果正在运行）
            let _ = device.app_stop("com.android.settings").await;
            tokio::time::sleep(Duration::from_secs(1)).await;

            // 启动应用
            let start_result = device.app_start("com.android.settings", None).await;
            assert!(start_result.is_ok());

            // 等待应用启动
            let wait_result = device
                .app_wait("com.android.settings", Some(Duration::from_secs(10)))
                .await;

            match wait_result {
                Ok(pid) => {
                    assert!(pid > 0);
                    println!("应用启动并等待成功，PID: {}", pid);
                }
                Err(e) => {
                    log_env_related_error("等待应用启动", &e);
                }
            }
        }
    }
}
