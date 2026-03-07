//! Device 核心功能
//!
//! 提供设备连接、信息获取和元素定位等核心功能

use crate::{
    adb::AdbClient,
    error::{Error, Result},
    jsonrpc::JsonRpcClient,
    models::{Coord, DeviceInfo},
    selector::Selector,
    settings::Settings,
    uiobject::UiObject,
};
use log::{debug, info, warn};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

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

    /// 设备信息缓存（默认关闭，仅在显式设置 TTL 后启用）
    info_cache: Arc<RwLock<Option<CacheEntry<DeviceInfo>>>>,

    /// 设备信息缓存 TTL
    cache_ttl: Arc<RwLock<Option<Duration>>>,

    /// 服务器模式
    server_mode: ServerMode,
}

#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    cached_at: Instant,
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
    fn with_settings_read<T>(&self, reader: impl FnOnce(&Settings) -> T) -> T {
        match self.settings.read() {
            Ok(settings) => reader(&settings),
            Err(poisoned) => {
                warn!("Settings read lock poisoned, recovering");
                let settings = poisoned.into_inner();
                reader(&settings)
            }
        }
    }

    fn with_settings_write<T>(&self, writer: impl FnOnce(&mut Settings) -> T) -> T {
        match self.settings.write() {
            Ok(mut settings) => writer(&mut settings),
            Err(poisoned) => {
                warn!("Settings write lock poisoned, recovering");
                let mut settings = poisoned.into_inner();
                writer(&mut settings)
            }
        }
    }

    fn operation_delays(&self) -> (Duration, Duration) {
        self.with_settings_read(|settings| {
            (
                settings.operation_delay_before,
                settings.operation_delay_after,
            )
        })
    }

    fn is_valid_package_name(package: &str) -> bool {
        if package.is_empty() || package.len() > 255 {
            return false;
        }

        let mut segment_count = 0usize;
        for segment in package.split('.') {
            if segment.is_empty() {
                return false;
            }

            let mut chars = segment.chars();
            let Some(first_char) = chars.next() else {
                return false;
            };

            if !(first_char.is_ascii_alphabetic() || first_char == '_') {
                return false;
            }

            if !chars.all(|character| character.is_ascii_alphanumeric() || character == '_') {
                return false;
            }

            segment_count += 1;
        }

        segment_count >= 2
    }

    fn is_valid_activity_name(activity: &str) -> bool {
        !activity.is_empty()
            && activity.len() <= 255
            && activity.chars().all(|character| {
                character.is_ascii_alphanumeric()
                    || character == '_'
                    || character == '.'
                    || character == '$'
            })
    }

    fn ensure_valid_package_name(package: &str) -> Result<()> {
        if Self::is_valid_package_name(package) {
            return Ok(());
        }

        Err(Error::InvalidArgument(format!(
            "鏃犳晥鐨勫寘鍚? {}",
            package
        )))
    }

    fn ensure_valid_activity_name(activity: &str) -> Result<()> {
        if Self::is_valid_activity_name(activity) {
            return Ok(());
        }

        Err(Error::InvalidArgument(format!(
            "鏃犳晥鐨?Activity 鍚? {}",
            activity
        )))
    }

    /// 连接到 Android 设备。
    ///
    /// 默认使用 `ServerMode::Auto`，会优先尝试 `ATX-Agent`，
    /// 不可用时自动回退到 `Direct` 模式。
    ///
    /// # 参数
    ///
    /// * `serial` - 设备序列号，传入 `None` 时自动选择设备
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(Some("emulator-5554")).await?;
    ///     println!("connected: {}", device.serial());
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
        let info_cache = Arc::new(RwLock::new(None));
        let cache_ttl = Arc::new(RwLock::new(None));
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
            info_cache,
            cache_ttl,
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
        let info_cache = Arc::new(RwLock::new(None));
        let cache_ttl = Arc::new(RwLock::new(None));

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
            info_cache,
            cache_ttl,
            server_mode: ServerMode::AtxAgent,
        })
    }

    /// 尝试使用 Direct 模式连接
    async fn try_connect_direct(serial: String, adb_client: Arc<AdbClient>) -> Result<Self> {
        debug!("创建 JSON-RPC 客户端（Direct 模式）");

        // 创建配置
        let settings = Arc::new(RwLock::new(Settings::default()));
        let info_cache = Arc::new(RwLock::new(None));
        let cache_ttl = Arc::new(RwLock::new(None));

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
            info_cache,
            cache_ttl,
            server_mode: ServerMode::Direct,
        })
    }

    /// 获取设备信息
    ///
    /// # 返回
    ///
    /// 返回设备的详细信息，包括屏幕尺寸、旋转角度等
    ///
    /// 如果已通过 [`Device::set_cache_ttl`] 显式启用缓存，且缓存尚未过期，
    /// 则直接返回缓存值；默认情况下每次都会实时调用设备端 `deviceInfo`。
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

        if let Some(ttl) = self.cache_ttl() {
            if let Some(info) = self.get_cached_info(ttl) {
                return Ok(info);
            }
        }

        // 调用 JSON-RPC 的 deviceInfo 方法
        let info: DeviceInfo = self
            .jsonrpc_client
            .call("deviceInfo", serde_json::json!({}))
            .await?;

        if self.cache_ttl().is_some() {
            self.store_cached_info(info.clone());
        }

        Ok(info)
    }

    /// 启用设备信息缓存，并设置缓存 TTL。
    ///
    /// 默认情况下，`Device::info()` 每次都会实时向设备端发起 RPC。
    /// 调用本方法后，在 TTL 未过期时，`Device::info()` 会优先返回缓存值。
    ///
    /// # 参数
    ///
    /// * `ttl` - 缓存有效期
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     device.set_cache_ttl(Duration::from_secs(2));
    ///     let info = device.info().await?;
    ///     println!("cached width: {}", info.display_width);
    ///     Ok(())
    /// }
    /// ```
    pub fn set_cache_ttl(&self, ttl: Duration) {
        self.set_cache_ttl_internal(Some(ttl));
        self.clear_cache();
    }

    /// 清除当前设备信息缓存。
    ///
    /// 这不会关闭缓存功能；后续 `Device::info()` 会重新发起一次 RPC，
    /// 并在缓存仍启用时写回新的缓存值。
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     device.set_cache_ttl(Duration::from_secs(2));
    ///     device.clear_cache();
    ///     Ok(())
    /// }
    /// ```
    pub fn clear_cache(&self) {
        self.with_info_cache_write(|cache| *cache = None);
    }

    /// 关闭设备信息缓存，并清除已有缓存值。
    ///
    /// 调用后，`Device::info()` 会恢复为每次都实时请求设备端。
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use uiautomator::Device;
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     device.set_cache_ttl(Duration::from_secs(2));
    ///     device.disable_cache();
    ///     Ok(())
    /// }
    /// ```
    pub fn disable_cache(&self) {
        self.set_cache_ttl_internal(None);
        self.clear_cache();
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

    async fn coords_to_abs(&self, x: Coord, y: Coord) -> Result<(u32, u32)> {
        let (width, height) = self.window_size().await?;
        Ok((x.to_pixel(width)?, y.to_pixel(height)?))
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

        let (delay_before, delay_after) = self.operation_delays();

        if delay_before > Duration::from_millis(0) {
            tokio::time::sleep(delay_before).await;
        }

        // 调用 JSON-RPC 的 click 方法
        let result: serde_json::Value = self
            .jsonrpc_client
            .call("click", serde_json::json!([x, y]))
            .await?;
        Self::ensure_action_rpc_result("click", result)?;

        if delay_after > Duration::from_millis(0) {
            tokio::time::sleep(delay_after).await;
        }

        Ok(())
    }

    /// 点击指定坐标，支持像素和百分比坐标。
    ///
    /// # 参数
    ///
    /// * `x` - X 坐标，支持像素和百分比
    /// * `y` - Y 坐标，支持像素和百分比
    ///
    /// # Errors
    ///
    /// 当百分比坐标超出 `0.0..=1.0` 或无法获取屏幕尺寸时返回错误。
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use uiautomator::{Coord, Device};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///
    ///     device.click_coord(Coord::percent(0.5), Coord::percent(0.5)).await?;
    ///     device.click_coord(Coord::pixel(200), Coord::pixel(400)).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn click_coord(&self, x: Coord, y: Coord) -> Result<()> {
        let (x, y) = self.coords_to_abs(x, y).await?;
        self.click(x, y).await
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

        let (delay_before, delay_after) = self.operation_delays();

        if delay_before > Duration::from_millis(0) {
            tokio::time::sleep(delay_before).await;
        }

        // 优先使用 longClick；部分服务端版本未实现或参数签名不兼容时回退到 swipe 同点长按。
        match self
            .jsonrpc_client
            .call("longClick", serde_json::json!([x, y, duration_secs]))
            .await
        {
            Ok::<serde_json::Value, Error>(value) => {
                Self::ensure_action_rpc_result("longClick", value)?;
            }
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
                        let swipe_result: serde_json::Value = self
                            .jsonrpc_client
                            .call("swipe", serde_json::json!([x, y, x, y, duration_secs]))
                            .await?;
                        Self::ensure_action_rpc_result("swipe", swipe_result)?;
                    }
                } else {
                    return Err(error);
                }
            }
        }

        if delay_after > Duration::from_millis(0) {
            tokio::time::sleep(delay_after).await;
        }

        Ok(())
    }

    /// 长按指定坐标，支持像素和百分比坐标。
    ///
    /// # 参数
    ///
    /// * `x` - X 坐标，支持像素和百分比
    /// * `y` - Y 坐标，支持像素和百分比
    /// * `duration` - 长按时长，None 表示使用默认值
    ///
    /// # Errors
    ///
    /// 当百分比坐标超出 `0.0..=1.0` 或无法获取屏幕尺寸时返回错误。
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use uiautomator::{Coord, Device};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     device
    ///         .long_click_coord(
    ///             Coord::percent(0.5),
    ///             Coord::percent(0.5),
    ///             Some(Duration::from_secs(1)),
    ///         )
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn long_click_coord(
        &self,
        x: Coord,
        y: Coord,
        duration: Option<Duration>,
    ) -> Result<()> {
        let (x, y) = self.coords_to_abs(x, y).await?;
        self.long_click(x, y, duration).await
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

        let (delay_before, delay_after) = self.operation_delays();

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

        if delay_after > Duration::from_millis(0) {
            tokio::time::sleep(delay_after).await;
        }

        Ok(())
    }

    /// 双击指定坐标，支持像素和百分比坐标。
    ///
    /// # 参数
    ///
    /// * `x` - X 坐标，支持像素和百分比
    /// * `y` - Y 坐标，支持像素和百分比
    /// * `duration` - 两次点击间隔，None 表示使用默认值
    ///
    /// # Errors
    ///
    /// 当百分比坐标超出 `0.0..=1.0` 或无法获取屏幕尺寸时返回错误。
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use uiautomator::{Coord, Device};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     device
    ///         .double_click_coord(
    ///             Coord::percent(0.4),
    ///             Coord::percent(0.6),
    ///             Some(Duration::from_millis(150)),
    ///         )
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn double_click_coord(
        &self,
        x: Coord,
        y: Coord,
        duration: Option<Duration>,
    ) -> Result<()> {
        let (x, y) = self.coords_to_abs(x, y).await?;
        self.double_click(x, y, duration).await
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

        let (delay_before, delay_after) = self.operation_delays();

        if delay_before > Duration::from_millis(0) {
            tokio::time::sleep(delay_before).await;
        }

        // 调用 JSON-RPC 的 swipe 方法
        let _: serde_json::Value = self
            .jsonrpc_client
            .call("swipe", serde_json::json!([fx, fy, tx, ty, duration_secs]))
            .await?;

        if delay_after > Duration::from_millis(0) {
            tokio::time::sleep(delay_after).await;
        }

        Ok(())
    }

    /// 滑动操作，支持像素和百分比坐标。
    ///
    /// # 参数
    ///
    /// * `fx` - 起始 X 坐标，支持像素和百分比
    /// * `fy` - 起始 Y 坐标，支持像素和百分比
    /// * `tx` - 结束 X 坐标，支持像素和百分比
    /// * `ty` - 结束 Y 坐标，支持像素和百分比
    /// * `duration` - 滑动持续时间，None 表示使用默认值
    ///
    /// # Errors
    ///
    /// 当百分比坐标超出 `0.0..=1.0` 或无法获取屏幕尺寸时返回错误。
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use uiautomator::{Coord, Device};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     device
    ///         .swipe_coord(
    ///             Coord::percent(0.8),
    ///             Coord::percent(0.8),
    ///             Coord::percent(0.2),
    ///             Coord::percent(0.2),
    ///             Some(Duration::from_millis(300)),
    ///         )
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn swipe_coord(
        &self,
        fx: Coord,
        fy: Coord,
        tx: Coord,
        ty: Coord,
        duration: Option<Duration>,
    ) -> Result<()> {
        let (width, height) = self.window_size().await?;
        self.swipe(
            fx.to_pixel(width)?,
            fy.to_pixel(height)?,
            tx.to_pixel(width)?,
            ty.to_pixel(height)?,
            duration,
        )
        .await
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

        let (delay_before, delay_after) = self.operation_delays();

        if delay_before > Duration::from_millis(0) {
            tokio::time::sleep(delay_before).await;
        }

        // 调用 JSON-RPC 的 drag 方法
        let _: serde_json::Value = self
            .jsonrpc_client
            .call("drag", serde_json::json!([sx, sy, ex, ey, duration_secs]))
            .await?;

        if delay_after > Duration::from_millis(0) {
            tokio::time::sleep(delay_after).await;
        }

        Ok(())
    }

    /// 拖拽操作，支持像素和百分比坐标。
    ///
    /// # 参数
    ///
    /// * `sx` - 起始 X 坐标，支持像素和百分比
    /// * `sy` - 起始 Y 坐标，支持像素和百分比
    /// * `ex` - 结束 X 坐标，支持像素和百分比
    /// * `ey` - 结束 Y 坐标，支持像素和百分比
    /// * `duration` - 拖拽持续时间，None 表示使用默认值
    ///
    /// # Errors
    ///
    /// 当百分比坐标超出 `0.0..=1.0` 或无法获取屏幕尺寸时返回错误。
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use uiautomator::{Coord, Device};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let device = Device::connect(None).await?;
    ///     device
    ///         .drag_coord(
    ///             Coord::pixel(100),
    ///             Coord::percent(0.6),
    ///             Coord::percent(0.9),
    ///             Coord::percent(0.6),
    ///             Some(Duration::from_millis(600)),
    ///         )
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn drag_coord(
        &self,
        sx: Coord,
        sy: Coord,
        ex: Coord,
        ey: Coord,
        duration: Option<Duration>,
    ) -> Result<()> {
        let (width, height) = self.window_size().await?;
        self.drag(
            sx.to_pixel(width)?,
            sy.to_pixel(height)?,
            ex.to_pixel(width)?,
            ey.to_pixel(height)?,
            duration,
        )
        .await
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
        self.with_settings_write(|settings| {
            settings.set_wait_timeout(timeout);
        });
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
        self.with_settings_read(|settings| settings.wait_timeout)
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
        self.with_settings_read(|settings| {
            if settings.polling_interval.is_zero() {
                Duration::from_millis(500)
            } else {
                settings.polling_interval
            }
        })
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

        let (delay_before, delay_after) = self.operation_delays();

        if delay_before > Duration::from_millis(0) {
            tokio::time::sleep(delay_before).await;
        }

        // 调用 JSON-RPC 的 pressKey 方法
        let key_name = key.to_name();
        let _: serde_json::Value = self
            .jsonrpc_client
            .call("pressKey", serde_json::json!([key_name]))
            .await?;

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

        let (delay_before, delay_after) = self.operation_delays();

        if delay_before > Duration::from_millis(0) {
            tokio::time::sleep(delay_before).await;
        }

        // 调用 JSON-RPC 的 pressKeyCode 方法
        let _: serde_json::Value = self
            .jsonrpc_client
            .call("pressKeyCode", serde_json::json!([keycode]))
            .await?;

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

    fn ensure_action_rpc_result(method: &str, result: serde_json::Value) -> Result<()> {
        match result {
            serde_json::Value::Bool(true) => Ok(()),
            serde_json::Value::Bool(false) => Err(Error::JsonRpc(format!(
                "{} returned false (action failed)",
                method
            ))),
            other => {
                debug!(
                    "JSON-RPC {} returned non-boolean result, treating as success: {:?}",
                    method, other
                );
                Ok(())
            }
        }
    }

    fn with_info_cache_read<T>(
        &self,
        reader: impl FnOnce(&Option<CacheEntry<DeviceInfo>>) -> T,
    ) -> T {
        match self.info_cache.read() {
            Ok(cache) => reader(&cache),
            Err(poisoned) => {
                warn!("Device info cache read lock poisoned, recovering");
                let cache = poisoned.into_inner();
                reader(&cache)
            }
        }
    }

    fn with_info_cache_write<T>(
        &self,
        writer: impl FnOnce(&mut Option<CacheEntry<DeviceInfo>>) -> T,
    ) -> T {
        match self.info_cache.write() {
            Ok(mut cache) => writer(&mut cache),
            Err(poisoned) => {
                warn!("Device info cache write lock poisoned, recovering");
                let mut cache = poisoned.into_inner();
                writer(&mut cache)
            }
        }
    }

    fn cache_ttl(&self) -> Option<Duration> {
        match self.cache_ttl.read() {
            Ok(ttl) => *ttl,
            Err(poisoned) => {
                warn!("Device info cache TTL lock poisoned, recovering");
                *poisoned.into_inner()
            }
        }
    }

    fn set_cache_ttl_internal(&self, ttl: Option<Duration>) {
        match self.cache_ttl.write() {
            Ok(mut cache_ttl) => *cache_ttl = ttl,
            Err(poisoned) => {
                warn!("Device info cache TTL lock poisoned, recovering");
                *poisoned.into_inner() = ttl;
            }
        }
    }

    fn get_cached_info(&self, ttl: Duration) -> Option<DeviceInfo> {
        self.with_info_cache_read(|cache| {
            cache.as_ref().and_then(|entry| {
                if entry.cached_at.elapsed() <= ttl {
                    Some(entry.value.clone())
                } else {
                    None
                }
            })
        })
    }

    fn store_cached_info(&self, info: DeviceInfo) {
        self.with_info_cache_write(|cache| {
            *cache = Some(CacheEntry {
                value: info,
                cached_at: Instant::now(),
            });
        });
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
        Self::ensure_valid_package_name(package)?;
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
        Self::ensure_valid_package_name(package)?;
        if let Some(act) = activity {
            Self::ensure_valid_activity_name(act)?;
        }

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
        Self::ensure_valid_package_name(package)?;

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
        Self::ensure_valid_package_name(package)?;

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
        Self::ensure_valid_package_name(package)?;
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
        Self::ensure_valid_package_name(package)?;
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
#[path = "device_tests.rs"]
mod tests;
