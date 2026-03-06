//! JSON-RPC 客户端
//!
//! 负责与设备端 UiAutomator 服务通信，支持 Direct 模式和 ATX-Agent 模式

use crate::adb::AdbClient;
use crate::atx_agent::AtxAgentClient;
use crate::device::ServerMode;
use crate::error::{Error, Result};
use crate::settings::Settings;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, TcpListener};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// 嵌入的 u2.jar 文件
const U2_JAR: &[u8] = include_bytes!("../assets/u2.jar");

/// u2.jar 的 MD5 哈希值（在构建时计算）
const U2_JAR_MD5: &str = env!("U2_JAR_MD5");

/// 设备端 UiAutomator 服务的默认端口
const UIAUTOMATOR_PORT: u16 = 9008;

/// JSON-RPC 客户端
///
/// 负责与设备端 UiAutomator 服务通信，支持 Direct 模式和 ATX-Agent 模式
///
/// # Examples
///
/// ```no_run
/// use std::sync::{Arc, RwLock};
/// use uiautomator::{AdbClient, JsonRpcClient, Settings};
///
/// #[tokio::main]
/// async fn main() -> uiautomator::Result<()> {
///     let adb = Arc::new(AdbClient::new().await?);
///     let settings = Arc::new(RwLock::new(Settings::default()));
///     let client = JsonRpcClient::new_direct("emulator-5554".to_string(), adb, settings).await?;
///     assert!(client.ping().await?);
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct JsonRpcClient {
    /// 设备序列号
    device_serial: String,

    /// ADB 客户端
    adb_client: Arc<AdbClient>,

    /// 服务器模式
    mode: ServerMode,

    // Direct 模式字段
    /// 本地端口（ADB 端口转发后，Direct 模式使用）
    local_port: Option<u16>,
    /// 自定义 Direct 模式 RPC URL（测试/高级场景）
    direct_rpc_url: Option<String>,

    // ATX-Agent 模式字段
    /// ATX-Agent 客户端（ATX-Agent 模式使用）
    atx_agent_client: Option<Arc<AtxAgentClient>>,

    // 共享字段
    /// HTTP 客户端
    http_client: reqwest::Client,

    /// 请求 ID 生成器
    request_id: AtomicU64,

    /// 配置设置
    settings: Arc<RwLock<Settings>>,
}

/// JSON-RPC 请求
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

/// JSON-RPC 错误
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

impl JsonRpcClient {
    /// 创建 Direct 模式客户端（直接连接 uiautomator2）
    ///
    /// # 参数
    ///
    /// * `device_serial` - 设备序列号
    /// * `adb_client` - ADB 客户端
    /// * `settings` - 配置设置
    ///
    /// # 错误
    ///
    /// 如果无法安装或启动服务，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::{Arc, RwLock};
    /// use uiautomator::{AdbClient, JsonRpcClient, Settings};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let adb = Arc::new(AdbClient::new().await?);
    ///     let settings = Arc::new(RwLock::new(Settings::default()));
    ///     let _client = JsonRpcClient::new_direct("emulator-5554".to_string(), adb, settings).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new_direct(
        device_serial: String,
        adb_client: Arc<AdbClient>,
        settings: Arc<RwLock<Settings>>,
    ) -> Result<Self> {
        info!(
            "正在为设备 {} 创建 JSON-RPC 客户端（Direct 模式）",
            device_serial
        );

        // 配置 HTTP 客户端（禁用代理，因为是 localhost 连接）
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .no_proxy()
            .build()
            .map_err(Error::Http)?;

        let mut client = Self {
            device_serial,
            adb_client,
            mode: ServerMode::Direct,
            local_port: None,
            direct_rpc_url: None,
            atx_agent_client: None,
            http_client,
            request_id: AtomicU64::new(1),
            settings,
        };

        // 确保服务就绪
        client.ensure_server_ready().await?;

        Ok(client)
    }

    /// 创建 ATX-Agent 模式客户端（通过 atx-agent 转发）
    ///
    /// # 参数
    ///
    /// * `device_serial` - 设备序列号
    /// * `adb_client` - ADB 客户端
    /// * `atx_agent_client` - ATX-Agent 客户端
    /// * `settings` - 配置设置
    ///
    /// # 错误
    ///
    /// 如果无法连接到 atx-agent，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::{Arc, RwLock};
    /// use uiautomator::{AdbClient, AtxAgentClient, JsonRpcClient, Settings};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let adb = Arc::new(AdbClient::new().await?);
    ///     let atx = Arc::new(AtxAgentClient::new("emulator-5554".to_string(), Arc::clone(&adb)).await?);
    ///     let settings = Arc::new(RwLock::new(Settings::default()));
    ///     let _client = JsonRpcClient::new_with_atx_agent(
    ///         "emulator-5554".to_string(),
    ///         adb,
    ///         atx,
    ///         settings,
    ///     )
    ///     .await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new_with_atx_agent(
        device_serial: String,
        adb_client: Arc<AdbClient>,
        atx_agent_client: Arc<AtxAgentClient>,
        settings: Arc<RwLock<Settings>>,
    ) -> Result<Self> {
        info!(
            "正在为设备 {} 创建 JSON-RPC 客户端（ATX-Agent 模式）",
            device_serial
        );

        // 配置 HTTP 客户端（禁用代理，因为是 localhost 连接）
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .no_proxy()
            .build()
            .map_err(Error::Http)?;

        let client = Self {
            device_serial,
            adb_client,
            mode: ServerMode::AtxAgent,
            local_port: None,
            direct_rpc_url: None,
            atx_agent_client: Some(atx_agent_client),
            http_client,
            request_id: AtomicU64::new(1),
            settings,
        };

        // 检查 atx-agent 是否可用
        if let Some(ref atx_client) = client.atx_agent_client {
            if !atx_client.is_available().await {
                return Err(Error::UiAutomatorNotConnected);
            }

            // 确保 uiautomator2 服务运行
            let status = atx_client.uiautomator_status().await?;
            if !status.running {
                info!("启动 uiautomator2 服务");
                atx_client.start_uiautomator().await?;

                // 等待服务就绪
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }

        info!("JSON-RPC 客户端已就绪（ATX-Agent 模式）");
        Ok(client)
    }

    /// 创建新的 JSON-RPC 客户端（兼容旧接口，使用 Direct 模式）
    ///
    /// # 参数
    ///
    /// * `device_serial` - 设备序列号
    /// * `adb_client` - ADB 客户端
    ///
    /// # 错误
    ///
    /// 如果无法安装或启动服务，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use uiautomator::{AdbClient, JsonRpcClient};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let adb = Arc::new(AdbClient::new().await?);
    ///     let _client = JsonRpcClient::new("emulator-5554".to_string(), adb).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(device_serial: String, adb_client: Arc<AdbClient>) -> Result<Self> {
        let settings = Arc::new(RwLock::new(Settings::default()));
        Self::new_direct(device_serial, adb_client, settings).await
    }

    /// 创建使用自定义 JSON-RPC endpoint 的 Direct 模式客户端。
    ///
    /// 该模式不会执行设备侧服务准备流程，适用于 mock/代理等场景。
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::{Arc, RwLock};
    /// use uiautomator::{AdbClient, JsonRpcClient, Settings};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let adb = Arc::new(AdbClient::new().await?);
    ///     let settings = Arc::new(RwLock::new(Settings::default()));
    ///     let _client = JsonRpcClient::new_direct_with_rpc_url(
    ///         "mock-device".to_string(),
    ///         adb,
    ///         settings,
    ///         "http://127.0.0.1:19008/jsonrpc/0".to_string(),
    ///     )?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new_direct_with_rpc_url(
        device_serial: String,
        adb_client: Arc<AdbClient>,
        settings: Arc<RwLock<Settings>>,
        rpc_url: String,
    ) -> Result<Self> {
        let parsed_url = reqwest::Url::parse(&rpc_url).map_err(|e| {
            Error::InvalidArgument(format!("无效的 JSON-RPC URL '{}': {}", rpc_url, e))
        })?;

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .no_proxy()
            .build()
            .map_err(Error::Http)?;

        Ok(Self {
            device_serial,
            adb_client,
            mode: ServerMode::Direct,
            local_port: None,
            direct_rpc_url: Some(parsed_url.to_string()),
            atx_agent_client: None,
            http_client,
            request_id: AtomicU64::new(1),
            settings,
        })
    }

    /// 发送 JSON-RPC 请求（带重试机制）
    ///
    /// 根据模式选择通信方式：
    /// - Direct 模式：直接发送 HTTP 请求到本地转发端口
    /// - ATX-Agent 模式：通过 atx_agent_client.forward_jsonrpc() 转发
    /// - Auto 模式：不应该出现在这里（应该在 Device 层面已经解析）
    ///
    /// # 参数
    ///
    /// * `method` - 方法名
    /// * `params` - 参数
    ///
    /// # 返回
    ///
    /// 返回反序列化后的结果
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use serde_json::json;
    /// use uiautomator::{AdbClient, JsonRpcClient};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let adb = Arc::new(AdbClient::new().await?);
    ///     let client = JsonRpcClient::new("emulator-5554".to_string(), adb).await?;
    ///     let info: serde_json::Value = client.call("deviceInfo", json!({})).await?;
    ///     println!("{info}");
    ///     Ok(())
    /// }
    /// ```
    pub async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        match self.mode {
            ServerMode::Direct => self.call_direct(method, params).await,
            ServerMode::AtxAgent => self.call_atx_agent(method, params).await,
            ServerMode::Auto => {
                // Auto 模式不应该出现在这里
                error!("JsonRpcClient 不应该使用 Auto 模式");
                Err(Error::InvalidArgument(
                    "JsonRpcClient 不支持 Auto 模式".to_string(),
                ))
            }
        }
    }

    /// Direct 模式：直接发送 JSON-RPC 请求（带重试机制）
    async fn call_direct<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        // 使用重试机制调用
        self.call_with_retry(|| self.call_once(method, params.clone()))
            .await
    }

    /// 带重试机制的调用包装器
    ///
    /// 实现指数退避重试策略：
    /// - 第 1 次重试：base_delay (默认 500ms)
    /// - 第 2 次重试：base_delay * 2 (默认 1000ms)
    /// - 第 3 次重试：base_delay * 3 (默认 1500ms)
    ///
    /// # 参数
    ///
    /// * `operation` - 要执行的异步操作
    ///
    /// # 返回
    ///
    /// 成功时返回操作结果，失败时返回最后一次的错误
    async fn call_with_retry<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // 在独立作用域中读取配置，确保锁守卫不会跨越任何 await 点
        let (max_retries, base_delay) = {
            let settings = match self.settings.read() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    warn!("Settings lock poisoned, recovering data");
                    poisoned.into_inner()
                }
            };
            (settings.max_retry, settings.retry_base_delay)
        };

        let mut last_error = None;

        for attempt in 1..=max_retries {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!("重试成功 (尝试 {}/{})", attempt, max_retries);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    error!(
                        "JSON-RPC 调用失败 (尝试 {}/{}): {:?}",
                        attempt, max_retries, e
                    );

                    // 检查是否需要重试
                    let should_retry = matches!(
                        &e,
                        Error::HttpTimeout | Error::Http(_) | Error::UiAutomatorNotConnected
                    );

                    if !should_retry || attempt == max_retries {
                        return Err(e);
                    }

                    // 如果是服务断开，尝试重启服务
                    if matches!(e, Error::UiAutomatorNotConnected | Error::Http(_)) {
                        warn!("检测到服务断开，尝试重启...");
                        if let Err(restart_err) = self.restart_server().await {
                            error!("重启服务失败: {:?}", restart_err);
                        } else {
                            info!("服务重启成功");
                        }
                    }

                    last_error = Some(e);

                    // 指数退避：base_delay * attempt
                    let delay = base_delay * attempt;
                    debug!("等待 {:?} 后重试...", delay);
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::JsonRpc("未知错误".to_string())))
    }

    /// ATX-Agent 模式：通过 atx-agent 转发 JSON-RPC 请求（带重试机制）
    async fn call_atx_agent<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let atx_client = self
            .atx_agent_client
            .as_ref()
            .ok_or_else(|| Error::UiAutomatorNotConnected)?;

        let atx_client = Arc::clone(atx_client);
        let method = method.to_string();
        // 在重试循环外获取 request_id，确保所有重试使用相同的 ID
        let request_id = self.request_id.fetch_add(1, Ordering::SeqCst);

        // 使用重试机制调用
        self.call_with_retry(|| {
            let atx_client = Arc::clone(&atx_client);
            let method = method.clone();
            let params = params.clone();
            async move {
                atx_client
                    .forward_jsonrpc(&method, params, request_id)
                    .await
            }
        })
        .await
    }

    /// 发送单次 JSON-RPC 请求（不重试，Direct 模式专用）
    async fn call_once<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        debug!("调用 JSON-RPC 方法: {} 参数: {:?}", method, params);

        // 构建请求
        let request = self.build_request(method, params);
        let request_id = request.id;

        // 序列化请求
        let request_json = serde_json::to_string(&request)?;
        debug!("请求 JSON: {}", request_json);

        // 获取本地端口
        // 发送 HTTP POST 请求（带超时）
        let url = if let Some(custom_url) = &self.direct_rpc_url {
            custom_url.clone()
        } else {
            let local_port = self.local_port.ok_or(Error::UiAutomatorNotConnected)?;
            format!("http://127.0.0.1:{}/jsonrpc/0", local_port)
        };

        let response = tokio::time::timeout(
            Duration::from_secs(60),
            self.http_client
                .post(&url)
                .header("Content-Type", "application/json")
                .body(request_json)
                .send(),
        )
        .await
        .map_err(|_| Error::HttpTimeout)??;

        // 检查 HTTP 状态码
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("HTTP 请求失败: {} - {}", status, body);
            return Err(Error::JsonRpc(format!("HTTP 错误 {}: {}", status, body)));
        }

        // 读取响应体
        let response_text = response.text().await?;
        debug!("响应 JSON: {}", response_text);

        // 解析 JSON-RPC 响应
        let rpc_response: serde_json::Value = serde_json::from_str(&response_text)?;
        let response_obj = rpc_response
            .as_object()
            .ok_or_else(|| Error::JsonRpc("响应格式无效：不是 JSON 对象".to_string()))?;

        // 验证响应 ID
        let response_id = response_obj
            .get("id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| Error::JsonRpc("响应中缺少有效的 id 字段".to_string()))?;

        if response_id != request_id {
            if self.direct_rpc_url.is_some() {
                debug!(
                    "自定义 RPC URL 模式下跳过响应 ID 校验: 期望 {}, 实际 {}",
                    request_id, response_id
                );
            } else {
                error!("响应 ID 不匹配: 期望 {}, 实际 {}", request_id, response_id);
                return Err(Error::JsonRpc(format!(
                    "响应 ID 不匹配: 期望 {}, 实际 {}",
                    request_id, response_id
                )));
            }
        }

        // 处理错误响应
        if let Some(error_value) = response_obj.get("error") {
            if !error_value.is_null() {
                let error: JsonRpcError = serde_json::from_value(error_value.clone())
                    .map_err(|e| Error::JsonRpc(format!("解析 error 字段失败: {}", e)))?;
                error!("JSON-RPC 错误: {} - {}", error.code, error.message);
                return Err(Error::JsonRpc(format!(
                    "错误码 {}: {}",
                    error.code, error.message
                )));
            }
        }

        // 提取结果
        let result = response_obj
            .get("result")
            .cloned()
            .ok_or_else(|| Error::JsonRpc("响应中缺少 result 字段".to_string()))?;

        // 反序列化结果
        let typed_result: T = serde_json::from_value(result)?;

        debug!("JSON-RPC 调用成功");
        Ok(typed_result)
    }

    /// 重启 UiAutomator 服务
    async fn restart_server(&self) -> Result<()> {
        info!("重启 UiAutomator 服务");

        match self.mode {
            ServerMode::Direct => {
                // Direct 模式直接启动设备侧进程
                self.start_server().await?;
            }
            ServerMode::AtxAgent => {
                // ATX-Agent 模式通过 /uiautomator 接口重启服务
                let atx_client = self
                    .atx_agent_client
                    .as_ref()
                    .ok_or(Error::UiAutomatorNotConnected)?;

                // 停止失败不阻断重启，继续尝试启动
                if let Err(e) = atx_client.stop_uiautomator().await {
                    debug!("停止 uiautomator2 失败，继续启动: {:?}", e);
                }

                atx_client.start_uiautomator().await?;
            }
            ServerMode::Auto => {
                return Err(Error::InvalidArgument(
                    "JsonRpcClient 不支持 Auto 模式".to_string(),
                ));
            }
        }

        // 等待服务就绪
        self.wait_for_ready(Duration::from_secs(30)).await?;

        info!("服务重启完成");
        Ok(())
    }

    /// 检查服务是否存活
    ///
    /// # 返回
    ///
    /// 如果服务正常运行，返回 true
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use uiautomator::{AdbClient, JsonRpcClient};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let adb = Arc::new(AdbClient::new().await?);
    ///     let client = JsonRpcClient::new("emulator-5554".to_string(), adb).await?;
    ///     let alive = client.ping().await?;
    ///     println!("alive: {alive}");
    ///     Ok(())
    /// }
    /// ```
    pub async fn ping(&self) -> Result<bool> {
        debug!("Ping UiAutomator 服务");

        match self.mode {
            ServerMode::Direct => {
                if self.local_port.is_none() {
                    return Ok(false);
                }

                let local_port = self.local_port.unwrap();
                let url = format!("http://127.0.0.1:{}/ping", local_port);

                match tokio::time::timeout(
                    Duration::from_secs(5),
                    self.http_client.get(&url).send(),
                )
                .await
                {
                    Ok(Ok(response)) => {
                        if let Ok(text) = response.text().await {
                            let is_alive = text.trim() == "pong";
                            debug!("Ping 结果: {}", if is_alive { "存活" } else { "未响应" });
                            Ok(is_alive)
                        } else {
                            debug!("Ping 失败: 无法读取响应");
                            Ok(false)
                        }
                    }
                    Ok(Err(e)) => {
                        debug!("Ping 失败: HTTP 错误 - {:?}", e);
                        Ok(false)
                    }
                    Err(_) => {
                        debug!("Ping 失败: 超时");
                        Ok(false)
                    }
                }
            }
            ServerMode::AtxAgent => {
                if let Some(ref atx_client) = self.atx_agent_client {
                    let request_id = self.request_id.fetch_add(1, Ordering::SeqCst);
                    Ok(atx_client
                        .forward_jsonrpc::<serde_json::Value>(
                            "deviceInfo",
                            serde_json::json!({}),
                            request_id,
                        )
                        .await
                        .is_ok())
                } else {
                    Ok(false)
                }
            }
            ServerMode::Auto => {
                // Auto 模式不应该出现在这里
                error!("JsonRpcClient 不应该使用 Auto 模式");
                Ok(false)
            }
        }
    }

    /// 确保服务就绪（安装 + 启动）
    async fn ensure_server_ready(&mut self) -> Result<()> {
        info!("确保设备 {} 上的 UiAutomator 服务就绪", self.device_serial);

        // 1. 推送 JAR 文件（如果需要）
        self.push_jar_if_needed().await?;

        // 2. 建立端口转发
        self.setup_port_forward().await?;

        // 3. 检查服务是否已经运行
        if self.check_server_alive().await {
            info!("UiAutomator 服务已经在运行");
            return Ok(());
        }

        // 4. 启动服务
        self.start_server().await?;

        // 5. 等待服务就绪
        self.wait_for_ready(Duration::from_secs(30)).await?;

        info!("UiAutomator 服务已就绪");
        Ok(())
    }

    /// 推送 JAR 文件到设备（如果需要）
    async fn push_jar_if_needed(&self) -> Result<()> {
        let target_path = "/data/local/tmp/u2.jar";

        debug!("检查设备上的 u2.jar 文件");

        // 检查设备上的文件是否存在且 MD5 匹配
        if self.check_device_jar_hash(target_path).await? {
            debug!("u2.jar 已存在且版本匹配，无需推送");
            return Ok(());
        }

        info!("推送 u2.jar 到设备 {}", self.device_serial);

        // 将嵌入的 JAR 写入临时文件
        let temp_dir = std::env::temp_dir();
        let temp_jar_path = temp_dir.join(format!("u2_{}.jar", U2_JAR_MD5));

        tokio::fs::write(&temp_jar_path, U2_JAR).await?;

        // 推送到设备
        self.adb_client
            .push(
                &self.device_serial,
                temp_jar_path.to_str().unwrap(),
                target_path,
            )
            .await?;

        // 清理临时文件
        let _ = tokio::fs::remove_file(&temp_jar_path).await;

        info!("u2.jar 推送成功");
        Ok(())
    }

    /// 检查设备上的 JAR 文件哈希值
    async fn check_device_jar_hash(&self, remote_path: &str) -> Result<bool> {
        // 首先尝试使用 toybox md5sum
        let output = self
            .adb_client
            .shell(
                &self.device_serial,
                &format!("toybox md5sum {}", remote_path),
                Some(Duration::from_secs(5)),
            )
            .await;

        let md5_output = if let Ok(output) = output {
            if output.contains("toybox") && output.contains("not found") {
                // toybox 不可用，尝试使用 md5 命令
                self.adb_client
                    .shell(
                        &self.device_serial,
                        &format!("md5 {}", remote_path),
                        Some(Duration::from_secs(5)),
                    )
                    .await
                    .unwrap_or_default()
            } else {
                output
            }
        } else {
            // 文件可能不存在
            return Ok(false);
        };

        // 检查输出中是否包含我们的 MD5
        Ok(md5_output.contains(U2_JAR_MD5))
    }

    /// 启动 UiAutomator 服务
    async fn start_server(&self) -> Result<()> {
        info!("启动设备 {} 上的 UiAutomator 服务", self.device_serial);

        // 使用 Direct 模式启动服务（注意环境变量赋值必须放在 nohup 前面，
        // 否则会被当作可执行文件导致启动失败）
        let bg_command =
            "CLASSPATH=/data/local/tmp/u2.jar nohup app_process / com.wetest.uia2.Main >/dev/null 2>&1 &";

        debug!("执行启动命令: {}", bg_command);

        self.adb_client
            .shell(
                &self.device_serial,
                bg_command,
                Some(Duration::from_secs(5)),
            )
            .await?;

        info!("UiAutomator 服务启动命令已发送");
        Ok(())
    }

    /// 等待服务就绪
    async fn wait_for_ready(&self, timeout: Duration) -> Result<()> {
        info!("等待 UiAutomator 服务就绪（超时: {:?}）", timeout);

        let start = tokio::time::Instant::now();
        let mut retry_count = 0;

        while start.elapsed() < timeout {
            retry_count += 1;

            // 尝试 ping 服务
            match self.check_server_alive().await {
                true => {
                    info!("UiAutomator 服务已就绪（尝试次数: {}）", retry_count);
                    return Ok(());
                }
                false => {
                    debug!("服务尚未就绪，等待 1 秒后重试...");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }

        error!("等待 UiAutomator 服务就绪超时");
        Err(Error::Timeout)
    }

    fn allocate_local_forward_port() -> Result<u16> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .map_err(|e| Error::Adb(format!("failed to bind local temporary port: {}", e)))?;
        let local_port = listener
            .local_addr()
            .map_err(|e| Error::Adb(format!("failed to inspect local temporary port: {}", e)))?
            .port();
        drop(listener);
        Ok(local_port)
    }

    /// 建立 ADB 端口转发（Direct 模式专用）
    async fn setup_port_forward(&mut self) -> Result<()> {
        const MAX_PORT_FORWARD_ATTEMPTS: usize = 10;
        let mut last_error_message = String::new();

        for attempt in 1..=MAX_PORT_FORWARD_ATTEMPTS {
            let local_port = Self::allocate_local_forward_port()?;
            info!(
                "建立端口转发: localhost:{} -> device:{} (attempt {}/{})",
                local_port, UIAUTOMATOR_PORT, attempt, MAX_PORT_FORWARD_ATTEMPTS
            );

            match self
                .adb_client
                .forward(&self.device_serial, local_port, UIAUTOMATOR_PORT)
                .await
            {
                Ok(()) => {
                    self.local_port = Some(local_port);
                    info!("端口转发已建立");
                    return Ok(());
                }
                Err(error) => {
                    last_error_message = error.to_string();
                    warn!(
                        "port forward failed on local port {}, retrying: {}",
                        local_port, last_error_message
                    );
                }
            }
        }

        Err(Error::Adb(format!(
            "failed to establish local port forward after {} attempts: {}",
            MAX_PORT_FORWARD_ATTEMPTS, last_error_message
        )))
    }

    /// 检查服务是否存活（Direct 模式专用）
    async fn check_server_alive(&self) -> bool {
        match self.mode {
            ServerMode::Direct => {
                if self.local_port.is_none() {
                    return false;
                }

                let local_port = self.local_port.unwrap();
                let url = format!("http://127.0.0.1:{}/ping", local_port);

                match tokio::time::timeout(
                    Duration::from_secs(2),
                    self.http_client.get(&url).send(),
                )
                .await
                {
                    Ok(Ok(response)) => {
                        if let Ok(text) = response.text().await {
                            text.trim() == "pong"
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }
            ServerMode::AtxAgent => {
                let Some(atx_client) = self.atx_agent_client.as_ref() else {
                    return false;
                };

                // 通过真实 JSON-RPC 调用探活，避免仅 /version 可用但 /jsonrpc/0 仍 502。
                let request_id = self.request_id.fetch_add(1, Ordering::SeqCst);
                atx_client
                    .forward_jsonrpc::<serde_json::Value>(
                        "deviceInfo",
                        serde_json::json!({}),
                        request_id,
                    )
                    .await
                    .is_ok()
            }
            ServerMode::Auto => false,
        }
    }

    /// 构建 JSON-RPC 请求
    fn build_request(&self, method: &str, params: serde_json::Value) -> JsonRpcRequest {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// 测试重试机制：第一次失败，第二次成功
    #[tokio::test]
    async fn test_retry_succeeds_on_second_attempt() {
        let settings = Arc::new(RwLock::new(Settings::default()));
        let attempt_count = Arc::new(AtomicUsize::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        // 创建一个模拟的 JsonRpcClient（仅用于测试 call_with_retry）
        let adb_client = Arc::new(AdbClient::new().await.unwrap());
        let client = JsonRpcClient {
            device_serial: "test".to_string(),
            adb_client,
            mode: ServerMode::Direct,
            local_port: Some(9008),
            direct_rpc_url: None,
            atx_agent_client: None,
            http_client: reqwest::Client::new(),
            request_id: AtomicU64::new(1),
            settings,
        };

        // 模拟操作：第一次失败，第二次成功
        let result = client
            .call_with_retry(|| {
                let count = attempt_count_clone.clone();
                async move {
                    let current = count.fetch_add(1, Ordering::SeqCst);
                    if current == 0 {
                        Err(Error::HttpTimeout)
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt_count.load(Ordering::SeqCst), 2);
    }

    /// 测试重试机制：达到最大重试次数后失败
    #[tokio::test]
    async fn test_retry_fails_after_max_attempts() {
        let settings = Settings {
            max_retry: 3,
            ..Settings::default()
        };
        let settings = Arc::new(RwLock::new(settings));
        let attempt_count = Arc::new(AtomicUsize::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        let adb_client = Arc::new(AdbClient::new().await.unwrap());
        let client = JsonRpcClient {
            device_serial: "test".to_string(),
            adb_client,
            mode: ServerMode::Direct,
            local_port: Some(9008),
            direct_rpc_url: None,
            atx_agent_client: None,
            http_client: reqwest::Client::new(),
            request_id: AtomicU64::new(1),
            settings,
        };

        // 模拟操作：总是失败
        let result = client
            .call_with_retry(|| {
                let count = attempt_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>(Error::HttpTimeout)
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }

    /// 测试重试机制：使用指数退避
    #[tokio::test]
    async fn test_retry_uses_exponential_backoff() {
        let settings = Settings {
            max_retry: 3,
            retry_base_delay: Duration::from_millis(100),
            ..Settings::default()
        };
        let settings = Arc::new(RwLock::new(settings));

        let adb_client = Arc::new(AdbClient::new().await.unwrap());
        let client = JsonRpcClient {
            device_serial: "test".to_string(),
            adb_client,
            mode: ServerMode::Direct,
            local_port: Some(9008),
            direct_rpc_url: None,
            atx_agent_client: None,
            http_client: reqwest::Client::new(),
            request_id: AtomicU64::new(1),
            settings,
        };

        let start = tokio::time::Instant::now();

        // 模拟操作：总是失败
        let _result = client
            .call_with_retry(|| async { Err::<i32, _>(Error::HttpTimeout) })
            .await;

        let elapsed = start.elapsed();

        // 预期延迟：
        // 第1次尝试失败 -> 等待 100ms (base_delay * 1)
        // 第2次尝试失败 -> 等待 200ms (base_delay * 2)
        // 第3次尝试失败 -> 返回错误
        // 总延迟：100ms + 200ms = 300ms
        // 允许误差范围（-50ms ~ +200ms），考虑系统调度延迟
        assert!(
            elapsed >= Duration::from_millis(250),
            "elapsed: {:?}",
            elapsed
        );
        assert!(
            elapsed <= Duration::from_millis(600),
            "elapsed: {:?}",
            elapsed
        );
    }

    /// 测试重试机制：不可重试的错误立即返回
    #[tokio::test]
    async fn test_retry_non_retryable_error_returns_immediately() {
        let settings = Arc::new(RwLock::new(Settings::default()));
        let attempt_count = Arc::new(AtomicUsize::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        let adb_client = Arc::new(AdbClient::new().await.unwrap());
        let client = JsonRpcClient {
            device_serial: "test".to_string(),
            adb_client,
            mode: ServerMode::Direct,
            local_port: Some(9008),
            direct_rpc_url: None,
            atx_agent_client: None,
            http_client: reqwest::Client::new(),
            request_id: AtomicU64::new(1),
            settings,
        };

        // 模拟操作：返回不可重试的错误
        let result = client
            .call_with_retry(|| {
                let count = attempt_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>(Error::InvalidArgument("test error".to_string()))
                }
            })
            .await;

        assert!(result.is_err());
        // 不可重试的错误应该只尝试一次
        assert_eq!(attempt_count.load(Ordering::SeqCst), 1);
    }

    /// 测试重试机制：第一次成功不重试
    #[tokio::test]
    async fn test_retry_first_success_no_retry() {
        let settings = Arc::new(RwLock::new(Settings::default()));
        let attempt_count = Arc::new(AtomicUsize::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        let adb_client = Arc::new(AdbClient::new().await.unwrap());
        let client = JsonRpcClient {
            device_serial: "test".to_string(),
            adb_client,
            mode: ServerMode::Direct,
            local_port: Some(9008),
            direct_rpc_url: None,
            atx_agent_client: None,
            http_client: reqwest::Client::new(),
            request_id: AtomicU64::new(1),
            settings,
        };

        // 模拟操作：第一次就成功
        let result = client
            .call_with_retry(|| {
                let count = attempt_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok(100)
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
        assert_eq!(attempt_count.load(Ordering::SeqCst), 1);
    }
}
