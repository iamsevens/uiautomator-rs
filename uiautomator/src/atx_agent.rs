//! ATX-Agent REST API 客户端
//!
//! 提供与 atx-agent 守护进程通信的 REST API 接口和 JSON-RPC 转发功能。
//!
//! ATX-Agent 是一个运行在 Android 设备上的守护进程，提供：
//! - 稳定的服务管理（自动重启崩溃的服务）
//! - JSON-RPC 请求转发到 uiautomator2
//! - 设备信息查询
//! - UI 层级结构获取

use crate::adb::AdbClient;
use crate::error::{Error, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// ATX-Agent 默认端口
const ATX_AGENT_PORT: u16 = 7912;

/// ATX-Agent 设备端路径
const ATX_AGENT_DEVICE_PATH: &str = "/data/local/tmp/atx-agent";

/// UiAutomator APK 包名
const UIAUTOMATOR_PACKAGE: &str = "com.github.uiautomator";
const UIAUTOMATOR_TEST_PACKAGE: &str = "com.github.uiautomator.test";

// 嵌入资源文件（仅在文件存在时）
#[cfg(feature = "atx-agent-install")]
mod atx_agent_assets {
    include!(concat!(env!("OUT_DIR"), "/atx_agent_assets.rs"));
}

const ATX_AGENT_HEALTHCHECK_TIMEOUT: Duration = Duration::from_secs(3);

#[cfg(feature = "atx-agent-install")]
#[derive(Clone, Copy)]
struct EmbeddedAtxAgentBinary {
    label: &'static str,
    bytes: &'static [u8],
    md5: &'static str,
}

const UIAUTOMATOR_APK: &[u8] = include_bytes!("../assets/app-uiautomator.apk");
const UIAUTOMATOR_APK_MD5: &str = env!("UIAUTOMATOR_APK_MD5");

#[cfg(feature = "atx-agent-install")]
const UIAUTOMATOR_TEST_APK: &[u8] = include_bytes!("../assets/app-uiautomator-test.apk");
#[cfg(feature = "atx-agent-install")]
const UIAUTOMATOR_TEST_APK_MD5: &str = env!("UIAUTOMATOR_TEST_APK_MD5");

#[cfg(not(feature = "atx-agent-install"))]
const UIAUTOMATOR_TEST_APK: &[u8] = &[];
#[cfg(not(feature = "atx-agent-install"))]
const UIAUTOMATOR_TEST_APK_MD5: &str = "placeholder";

/// ATX-Agent REST API 客户端
///
/// 负责与设备上的 atx-agent 守护进程通信，提供 REST API 接口和 JSON-RPC 转发。
///
/// # 示例
///
/// ```no_run
/// # use uiautomator::{AdbClient, atx_agent::AtxAgentClient};
/// # use std::sync::Arc;
/// # async fn example() -> uiautomator::Result<()> {
/// let adb = AdbClient::new().await?;
/// let client = AtxAgentClient::new("device_serial".to_string(), Arc::new(adb)).await?;
///
/// // 检查 atx-agent 是否可用
/// if client.is_available().await {
///     println!("ATX-Agent 可用");
/// }
///
/// // 获取版本
/// let version = client.version().await?;
/// println!("ATX-Agent 版本: {}", version);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct AtxAgentClient {
    /// 设备序列号
    device_serial: String,
    /// ADB 客户端
    adb_client: Arc<AdbClient>,
    /// 本地端口（ADB 端口转发后）
    local_port: u16,
    /// ATX-Agent 基础 URL
    base_url: String,
    /// HTTP 客户端
    http_client: reqwest::Client,
}

impl AtxAgentClient {
    /// 创建 ATX-Agent 客户端
    ///
    /// 假设设备上已安装并启动 atx-agent。
    ///
    /// # 参数
    ///
    /// * `device_serial` - 设备序列号
    /// * `adb_client` - ADB 客户端
    ///
    /// # 返回
    ///
    /// 返回配置好的 ATX-Agent 客户端
    ///
    /// # 错误
    ///
    /// 如果无法建立端口转发，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use uiautomator::{AdbClient, AtxAgentClient};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let adb = AdbClient::new().await?;
    ///     let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    ///     let _ = client.version().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(device_serial: String, adb_client: Arc<AdbClient>) -> Result<Self> {
        info!("创建 ATX-Agent 客户端: {}", device_serial);

        let local_port = ATX_AGENT_PORT;
        let base_url = format!("http://127.0.0.1:{}", local_port);

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| Error::Http(e))?;

        let mut client = Self {
            device_serial,
            adb_client,
            local_port,
            base_url,
            http_client,
        };

        // 建立 ADB 端口转发
        client.setup_port_forward().await?;

        Ok(client)
    }

    /// 检查 atx-agent 是否可用
    ///
    /// 通过尝试访问 /version 端点来检查服务是否运行。
    ///
    /// # 返回
    ///
    /// 如果 atx-agent 可用返回 true，否则返回 false
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// let available = client.is_available().await;
    /// println!("available: {available}");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_available(&self) -> bool {
        debug!("检查 ATX-Agent 是否可用");

        match self
            .version_with_timeout(ATX_AGENT_HEALTHCHECK_TIMEOUT)
            .await
        {
            Ok(_) => {
                info!("ATX-Agent 可用");
                true
            }
            Err(e) => {
                warn!("ATX-Agent 不可用: {}", e);
                false
            }
        }
    }

    // ========================================================================
    // REST API 接口
    // ========================================================================

    /// 获取 atx-agent 版本
    ///
    /// # 返回
    ///
    /// 返回 atx-agent 的版本字符串
    ///
    /// # 错误
    ///
    /// 如果请求失败或解析失败，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// let version = client.version().await?;
    /// assert!(!version.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn version(&self) -> Result<String> {
        debug!("获取 ATX-Agent 版本");

        let response = self.get("/version").await?;
        let body = response.text().await.map_err(|e| Error::Http(e))?;

        Self::parse_version_response(&body)
    }

    async fn version_with_timeout(&self, timeout: Duration) -> Result<String> {
        let response = self.get_with_timeout("/version", timeout).await?;
        let body = response.text().await.map_err(|e| Error::Http(e))?;
        Self::parse_version_response(&body)
    }

    /// 获取设备信息
    ///
    /// # 返回
    ///
    /// 返回设备的详细信息
    ///
    /// # 错误
    ///
    /// 如果请求失败或解析失败，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// let info = client.device_info().await?;
    /// println!("{} {}", info.brand, info.model);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn device_info(&self) -> Result<AtxDeviceInfo> {
        debug!("获取设备信息");

        let response = self.get("/info").await?;
        let info: AtxDeviceInfo = response.json().await.map_err(|e| Error::Http(e))?;

        Ok(info)
    }

    /// 启动 uiautomator2 服务
    ///
    /// # 错误
    ///
    /// 如果请求失败，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// client.start_uiautomator().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_uiautomator(&self) -> Result<()> {
        info!("启动 uiautomator2 服务");

        let body = serde_json::json!({});
        let _response = self.post("/uiautomator", body).await?;

        Ok(())
    }

    /// 停止 uiautomator2 服务
    ///
    /// # 错误
    ///
    /// 如果请求失败，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// client.stop_uiautomator().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stop_uiautomator(&self) -> Result<()> {
        info!("停止 uiautomator2 服务");

        // ATX-Agent 使用 DELETE 方法停止服务
        let url = format!("{}/uiautomator", self.base_url);
        let _response = self
            .http_client
            .delete(&url)
            .send()
            .await
            .map_err(|e| Error::Http(e))?;

        Ok(())
    }

    /// 检查 uiautomator2 服务状态
    ///
    /// # 返回
    ///
    /// 返回服务的运行状态
    ///
    /// # 错误
    ///
    /// 如果请求失败或解析失败，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// let status = client.uiautomator_status().await?;
    /// println!("running: {}", status.running);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn uiautomator_status(&self) -> Result<UiAutomatorStatus> {
        debug!("检查 uiautomator2 服务状态");

        let response = self.get("/uiautomator").await?;
        let status: UiAutomatorStatus = response.json().await.map_err(|e| Error::Http(e))?;

        Ok(status)
    }

    /// 获取 UI 层级结构（特殊接口）
    ///
    /// 这是 atx-agent 提供的特殊接口，可以更高效地获取 UI 层级。
    ///
    /// # 返回
    ///
    /// 返回 XML 格式的 UI 层级结构
    ///
    /// # 错误
    ///
    /// 如果请求失败，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// let xml = client.dump_hierarchy().await?;
    /// assert!(!xml.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dump_hierarchy(&self) -> Result<String> {
        debug!("获取 UI 层级结构");

        let body = serde_json::json!({});
        let response = self.post("/dump/hierarchy", body).await?;
        let text = response.text().await.map_err(|e| Error::Http(e))?;

        Ok(text)
    }

    // ========================================================================
    // JSON-RPC 转发接口
    // ========================================================================

    /// 转发 JSON-RPC 请求到 uiautomator2
    ///
    /// 通过 POST /jsonrpc/0 接口转发所有 JSON-RPC 请求。
    ///
    /// # 参数
    ///
    /// * `method` - JSON-RPC 方法名
    /// * `params` - 方法参数
    /// * `request_id` - 请求 ID
    ///
    /// # 返回
    ///
    /// 返回反序列化后的结果
    ///
    /// # 错误
    ///
    /// 如果请求失败或返回错误响应，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use serde_json::json;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// let sdk: i32 = client
    ///     .forward_jsonrpc("deviceInfo", json!([]), 1)
    ///     .await
    ///     .unwrap_or_default();
    /// println!("sdk: {sdk}");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn forward_jsonrpc<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
        request_id: u64,
    ) -> Result<T> {
        debug!("转发 JSON-RPC 请求: method={}, id={}", method, request_id);

        // 构建 JSON-RPC 请求
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params,
        });

        // 通过 /jsonrpc/0 转发到 uiautomator2
        let response = self.post("/jsonrpc/0", request).await?;
        let json_response: serde_json::Value = response.json().await.map_err(|e| Error::Http(e))?;

        // 解析响应
        if let Some(error) = json_response.get("error") {
            let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) as i32;
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            return Err(Error::JsonRpc(format!("错误码 {}: {}", code, message)));
        }

        if let Some(result) = json_response.get("result") {
            return serde_json::from_value(result.clone()).map_err(|e| Error::Serialization(e));
        }

        Err(Error::JsonRpc("Invalid JSON-RPC response".to_string()))
    }

    // ========================================================================
    // 内部方法
    // ========================================================================

    /// 建立 ADB 端口转发（7912 -> 7912）
    async fn setup_port_forward(&mut self) -> Result<()> {
        info!(
            "建立 ADB 端口转发: {} -> {}",
            self.local_port, ATX_AGENT_PORT
        );

        self.adb_client
            .forward(&self.device_serial, self.local_port, ATX_AGENT_PORT)
            .await
            .map_err(|e| Error::Adb(format!("端口转发失败: {}", e)))?;

        Ok(())
    }

    /// 发送 GET 请求
    async fn get(&self, path: &str) -> Result<reqwest::Response> {
        self.get_with_timeout(path, Duration::from_secs(30)).await
    }

    async fn get_with_timeout(&self, path: &str, timeout: Duration) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        debug!("GET {}", url);

        let response = self
            .http_client
            .get(&url)
            .timeout(timeout)
            .send()
            .await
            .map_err(|e| Error::Http(e))?;

        if !response.status().is_success() {
            return Err(Error::Http(reqwest::Error::from(
                response.error_for_status().unwrap_err(),
            )));
        }

        Ok(response)
    }

    /// 发送 POST 请求
    async fn post(&self, path: &str, body: serde_json::Value) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        debug!("POST {} with body: {}", url, body);

        let response = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Http(e))?;

        if !response.status().is_success() {
            return Err(Error::Http(reqwest::Error::from(
                response.error_for_status().unwrap_err(),
            )));
        }

        Ok(response)
    }

    /// 解析 /version 响应，兼容 JSON 与纯文本两种格式。
    ///
    /// 常见返回：
    /// - `{"version":"0.10.0"}`
    /// - `0.10.0`
    fn is_plausible_version(version: &str) -> bool {
        let candidate = version.trim().trim_start_matches('v');
        if candidate.is_empty() {
            return false;
        }

        let mut has_digit = false;
        let mut dot_count = 0;
        for ch in candidate.chars() {
            if ch.is_ascii_digit() {
                has_digit = true;
            } else if ch == '.' {
                dot_count += 1;
            } else if ch == '-' || ch == '+' || ch == '_' || ch.is_ascii_alphabetic() {
                // valid semver-ish suffix chars
            } else {
                return false;
            }
        }

        has_digit && dot_count >= 1
    }

    fn parse_version_response(body: &str) -> Result<String> {
        let trimmed = body.trim();
        if trimmed.is_empty() {
            return Err(Error::JsonRpc("版本响应为空".to_string()));
        }

        // 优先尝试 JSON 格式
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                if Self::is_plausible_version(version) {
                    return Ok(version.to_string());
                }
                return Err(Error::JsonRpc(format!("版本响应格式无效: {}", trimmed)));
            }
            if let Some(version) = json.as_str() {
                if Self::is_plausible_version(version) {
                    return Ok(version.to_string());
                }
                return Err(Error::JsonRpc(format!("版本响应格式无效: {}", trimmed)));
            }
        }

        // 兼容旧格式：直接返回纯文本版本号
        if Self::is_plausible_version(trimmed) {
            Ok(trimmed.to_string())
        } else {
            Err(Error::JsonRpc(format!("版本响应格式无效: {}", trimmed)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AtxAgentClient;

    #[test]
    fn test_parse_version_response_json_object() {
        let version = AtxAgentClient::parse_version_response(r#"{"version":"0.10.0"}"#).unwrap();
        assert_eq!(version, "0.10.0");
    }

    #[test]
    fn test_parse_version_response_plain_text() {
        let version = AtxAgentClient::parse_version_response("0.10.0\n").unwrap();
        assert_eq!(version, "0.10.0");
    }

    #[test]
    fn test_parse_version_response_empty_body() {
        let err = AtxAgentClient::parse_version_response("  ").unwrap_err();
        assert!(matches!(err, crate::Error::JsonRpc(_)));
    }

    #[test]
    fn test_parse_version_response_invalid_plain_text() {
        let err = AtxAgentClient::parse_version_response("service ready").unwrap_err();
        assert!(matches!(err, crate::Error::JsonRpc(_)));
    }

    #[test]
    fn test_parse_version_response_invalid_json_version() {
        let err =
            AtxAgentClient::parse_version_response(r#"{"version":"service ready"}"#).unwrap_err();
        assert!(matches!(err, crate::Error::JsonRpc(_)));
    }

    #[cfg(feature = "atx-agent-install")]
    #[test]
    fn test_candidate_labels_for_x86_64_prefers_amd64_then_386() {
        assert_eq!(
            AtxAgentClient::candidate_labels_for_abi("x86_64"),
            &["amd64", "386", "legacy"]
        );
    }
}

// ============================================================================
// 数据结构
// ============================================================================

/// ATX-Agent 设备信息
///
/// # Examples
///
/// ```
/// use uiautomator::atx_agent::AtxDeviceInfo;
///
/// let info = AtxDeviceInfo {
///     udid: "demo-udid".to_string(),
///     serial: "emulator-5554".to_string(),
///     brand: "google".to_string(),
///     model: "sdk_gphone64_x86_64".to_string(),
///     hwaddr: "00:00:00:00:00:00".to_string(),
///     agent_version: "0.10.0".to_string(),
/// };
/// assert_eq!(info.serial, "emulator-5554");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtxDeviceInfo {
    /// 设备唯一标识符
    pub udid: String,
    /// 设备序列号
    pub serial: String,
    /// 设备品牌
    pub brand: String,
    /// 设备型号
    pub model: String,
    /// 硬件地址
    pub hwaddr: String,
    /// ATX-Agent 版本
    #[serde(rename = "agentVersion")]
    pub agent_version: String,
}

/// UiAutomator 服务状态
///
/// # Examples
///
/// ```
/// use uiautomator::atx_agent::UiAutomatorStatus;
///
/// let status = UiAutomatorStatus { running: true };
/// assert!(status.running);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiAutomatorStatus {
    /// 服务是否正在运行
    pub running: bool,
}

// ============================================================================
// ATX-Agent 安装功能
// ============================================================================

impl AtxAgentClient {
    /// 检查 atx-agent 是否已安装
    ///
    /// 通过检查设备上的二进制文件是否存在来判断。
    ///
    /// # 返回
    ///
    /// 如果已安装返回 true，否则返回 false
    #[cfg(feature = "atx-agent-install")]
    async fn get_device_abis(&self) -> Result<Vec<String>> {
        let mut abis = Vec::new();

        if let Ok(output) = self
            .adb_client
            .shell(
                &self.device_serial,
                "getprop ro.product.cpu.abilist",
                Some(Duration::from_secs(5)),
            )
            .await
        {
            for abi in output.trim().split(',') {
                let value = abi.trim();
                if !value.is_empty() {
                    abis.push(value.to_string());
                }
            }
        }

        if abis.is_empty() {
            let output = self
                .adb_client
                .shell(
                    &self.device_serial,
                    "getprop ro.product.cpu.abi",
                    Some(Duration::from_secs(5)),
                )
                .await
                .map_err(|e| Error::Adb(format!("failed to detect device ABI: {}", e)))?;
            let value = output.trim();
            if !value.is_empty() {
                abis.push(value.to_string());
            }
        }

        if abis.is_empty() {
            return Err(Error::Adb(
                "failed to detect device ABI from getprop".to_string(),
            ));
        }

        Ok(abis)
    }

    #[cfg(feature = "atx-agent-install")]
    fn all_embedded_atx_agent_binaries(&self) -> [EmbeddedAtxAgentBinary; 5] {
        [
            EmbeddedAtxAgentBinary {
                label: "armv7",
                bytes: atx_agent_assets::ATX_AGENT_ARMV7,
                md5: atx_agent_assets::ATX_AGENT_ARMV7_MD5,
            },
            EmbeddedAtxAgentBinary {
                label: "arm64",
                bytes: atx_agent_assets::ATX_AGENT_ARM64,
                md5: atx_agent_assets::ATX_AGENT_ARM64_MD5,
            },
            EmbeddedAtxAgentBinary {
                label: "amd64",
                bytes: atx_agent_assets::ATX_AGENT_AMD64,
                md5: atx_agent_assets::ATX_AGENT_AMD64_MD5,
            },
            EmbeddedAtxAgentBinary {
                label: "386",
                bytes: atx_agent_assets::ATX_AGENT_386,
                md5: atx_agent_assets::ATX_AGENT_386_MD5,
            },
            EmbeddedAtxAgentBinary {
                label: "legacy",
                bytes: atx_agent_assets::ATX_AGENT_LEGACY,
                md5: atx_agent_assets::ATX_AGENT_LEGACY_MD5,
            },
        ]
    }

    #[cfg(feature = "atx-agent-install")]
    fn candidate_labels_for_abi(abi: &str) -> &'static [&'static str] {
        match abi {
            "arm64-v8a" | "arm64" => &["arm64", "legacy"],
            "armeabi-v7a" | "armeabi" | "arm" => &["armv7", "legacy"],
            // Prefer native x86_64 build, then fallback to 32-bit x86 for emulator compatibility.
            "x86_64" | "amd64" => &["amd64", "386", "legacy"],
            "x86" | "386" | "i686" => &["386", "legacy"],
            _ => &[],
        }
    }

    #[cfg(feature = "atx-agent-install")]
    fn select_embedded_binaries_for_abis(&self, abis: &[String]) -> Vec<EmbeddedAtxAgentBinary> {
        let binaries = self.all_embedded_atx_agent_binaries();
        let mut selected = Vec::new();

        for abi in abis {
            for label in Self::candidate_labels_for_abi(abi) {
                if let Some(candidate) = binaries.iter().find(|item| {
                    item.label == *label && !item.bytes.is_empty() && item.md5 != "placeholder"
                }) {
                    if !selected
                        .iter()
                        .any(|existing: &EmbeddedAtxAgentBinary| existing.label == candidate.label)
                    {
                        selected.push(*candidate);
                    }
                }
            }
        }

        if let Some(legacy) = binaries.iter().find(|item| {
            item.label == "legacy" && !item.bytes.is_empty() && item.md5 != "placeholder"
        }) {
            if !selected
                .iter()
                .any(|existing: &EmbeddedAtxAgentBinary| existing.label == legacy.label)
            {
                selected.push(*legacy);
            }
        }

        selected
    }

    #[cfg(feature = "atx-agent-install")]
    async fn resolve_atx_agent_binary_candidates(&self) -> Result<Vec<EmbeddedAtxAgentBinary>> {
        let abis = self.get_device_abis().await?;
        let candidates = self.select_embedded_binaries_for_abis(&abis);
        if candidates.is_empty() {
            return Err(Error::InvalidArgument(
                "no usable atx-agent binary found for current device ABI".to_string(),
            ));
        }

        let labels: Vec<&str> = candidates.iter().map(|item| item.label).collect();
        info!(
            "candidate atx-agent binaries for device ABIs [{}]: {}",
            abis.join(", "),
            labels.join(" -> ")
        );
        Ok(candidates)
    }

    #[cfg(feature = "atx-agent-install")]
    async fn push_and_verify_atx_agent_binary(&self, binary: EmbeddedAtxAgentBinary) -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("atx-agent-{}", binary.label));

        std::fs::write(&temp_path, binary.bytes).map_err(Error::Io)?;
        let temp_path_str = temp_path.to_str().ok_or_else(|| {
            Error::InvalidArgument(
                "failed to convert temporary atx-agent path to string".to_string(),
            )
        })?;

        let push_result = self
            .adb_client
            .push(&self.device_serial, temp_path_str, ATX_AGENT_DEVICE_PATH)
            .await
            .map_err(|e| {
                Error::Adb(format!(
                    "failed to push atx-agent ({}): {}",
                    binary.label, e
                ))
            });
        let _ = std::fs::remove_file(&temp_path);
        push_result?;

        self.adb_client
            .shell(
                &self.device_serial,
                &format!("chmod 755 {}", ATX_AGENT_DEVICE_PATH),
                Some(Duration::from_secs(10)),
            )
            .await
            .map_err(|e| {
                Error::Adb(format!(
                    "failed to chmod atx-agent ({}): {}",
                    binary.label, e
                ))
            })?;

        let version_output = self
            .adb_client
            .shell(
                &self.device_serial,
                &format!("{} version", ATX_AGENT_DEVICE_PATH),
                Some(Duration::from_secs(10)),
            )
            .await
            .map_err(|e| {
                Error::Adb(format!(
                    "failed to verify atx-agent binary '{}': {}",
                    binary.label, e
                ))
            })?;

        let version_text = version_output.trim();
        let version_lower = version_text.to_lowercase();
        let has_obvious_exec_error = version_lower.contains("no such file")
            || version_lower.contains("not found")
            || version_lower.contains("permission denied")
            || version_lower.contains("exec format error")
            || version_lower.contains("syntax error")
            || version_lower.contains("error");
        let looks_like_version = version_text.chars().any(|ch| ch.is_ascii_digit());
        if version_text.is_empty() || has_obvious_exec_error || !looks_like_version {
            let short_output: String = version_text.chars().take(200).collect();
            return Err(Error::Adb(format!(
                "binary '{}' verification output is invalid: {}",
                binary.label, short_output
            )));
        }

        Ok(())
    }

    /// Check whether `atx-agent` is installed and valid on device.
    ///
    /// When the `atx-agent-install` feature is enabled, this method also
    /// verifies that the device binary checksum matches one of bundled assets.
    ///
    /// # Returns
    ///
    /// - `Ok(true)`: binary exists and passes validation.
    /// - `Ok(false)`: binary missing or validation mismatch.
    ///
    /// # Errors
    ///
    /// Returns an error when ADB commands fail unexpectedly.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use uiautomator::{AdbClient, AtxAgentClient};
    ///
    /// #[tokio::main]
    /// async fn main() -> uiautomator::Result<()> {
    ///     let adb = AdbClient::new().await?;
    ///     let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    ///     let installed = client.check_atx_agent_installed().await?;
    ///     println!("atx-agent installed: {installed}");
    ///     Ok(())
    /// }
    /// ```
    pub async fn check_atx_agent_installed(&self) -> Result<bool> {
        debug!("检查 atx-agent 是否已安装");

        // 检查文件是否存在
        let output = self
            .adb_client
            .shell(
                &self.device_serial,
                &format!(
                    "test -f {} && echo 'exists' || echo 'not_exists'",
                    ATX_AGENT_DEVICE_PATH
                ),
                Some(Duration::from_secs(10)),
            )
            .await
            .map_err(|e| Error::Adb(format!("检查 atx-agent 失败: {}", e)))?;

        let exists = output.trim() == "exists";

        #[cfg(feature = "atx-agent-install")]
        if exists {
            let candidates = match self.resolve_atx_agent_binary_candidates().await {
                Ok(value) => value,
                Err(error) => {
                    warn!(
                        "failed to resolve atx-agent binary candidates for checksum validation: {}",
                        error
                    );
                    return Ok(false);
                }
            };

            match self.get_device_atx_agent_md5().await {
                Ok(device_md5) => {
                    if let Some(matched) = candidates
                        .iter()
                        .find(|candidate| candidate.md5 == device_md5)
                    {
                        info!(
                            "atx-agent installed and checksum matched ({})",
                            matched.label
                        );
                        return Ok(true);
                    }

                    info!("atx-agent checksum mismatch, reinstall required");
                    return Ok(false);
                }
                Err(error) => {
                    warn!("failed to get md5 of device atx-agent: {}", error);
                    return Ok(false);
                }
            }
        }

        Ok(exists)
    }

    /// 获取设备上 atx-agent 的 MD5
    #[cfg(feature = "atx-agent-install")]
    async fn get_device_atx_agent_md5(&self) -> Result<String> {
        let output = self
            .adb_client
            .shell(
                &self.device_serial,
                &format!("md5sum {}", ATX_AGENT_DEVICE_PATH),
                Some(Duration::from_secs(10)),
            )
            .await
            .map_err(|e| Error::Adb(format!("获取 MD5 失败: {}", e)))?;

        // md5sum 输出格式: "hash  filename"
        let md5 = output
            .split_whitespace()
            .next()
            .ok_or_else(|| Error::Adb("无法解析 MD5 输出".to_string()))?
            .to_string();

        Ok(md5)
    }

    /// 推送 atx-agent 二进制文件到设备
    ///
    /// # 错误
    ///
    /// 如果推送失败或设置权限失败，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// client.push_atx_agent().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn push_atx_agent(&self) -> Result<()> {
        #[cfg(feature = "atx-agent-install")]
        {
            let candidates = self.resolve_atx_agent_binary_candidates().await?;
            let mut failures = Vec::new();

            for candidate in candidates {
                match self.push_and_verify_atx_agent_binary(candidate).await {
                    Ok(()) => {
                        info!("atx-agent push completed with binary '{}'", candidate.label);
                        return Ok(());
                    }
                    Err(error) => {
                        warn!(
                            "atx-agent binary '{}' failed, trying next candidate: {}",
                            candidate.label, error
                        );
                        failures.push(format!("{}: {}", candidate.label, error));
                    }
                }
            }

            return Err(Error::Adb(format!(
                "failed to push a usable atx-agent binary; candidates tried: {}",
                failures.join(" | ")
            )));
        }

        #[cfg(not(feature = "atx-agent-install"))]
        return Err(Error::InvalidArgument(
            "atx-agent install feature is disabled in current build".to_string(),
        ));
    }

    /// 安装 UiAutomator APK
    ///
    /// 安装 app-uiautomator.apk 和 app-uiautomator-test.apk
    ///
    /// # 错误
    ///
    /// 如果安装失败，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// client.install_uiautomator_apks().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn install_uiautomator_apks(&self) -> Result<()> {
        info!("安装 UiAutomator APK");

        // 检查资源文件是否可用
        if UIAUTOMATOR_APK_MD5 == "placeholder" || UIAUTOMATOR_TEST_APK_MD5 == "placeholder" {
            return Err(Error::InvalidArgument(
                "UiAutomator APK 资源文件不可用，请先下载资源文件".to_string(),
            ));
        }

        // 安装 app-uiautomator.apk
        self.install_apk("app-uiautomator.apk", UIAUTOMATOR_APK, UIAUTOMATOR_PACKAGE)
            .await?;

        // 安装 app-uiautomator-test.apk
        self.install_apk(
            "app-uiautomator-test.apk",
            UIAUTOMATOR_TEST_APK,
            UIAUTOMATOR_TEST_PACKAGE,
        )
        .await?;

        info!("UiAutomator APK 安装完成");
        Ok(())
    }

    /// 安装单个 APK
    async fn install_apk(&self, name: &str, apk_data: &[u8], _package: &str) -> Result<()> {
        info!("安装 {}", name);

        // 写入临时文件
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(name);

        std::fs::write(&temp_path, apk_data).map_err(|e| Error::Io(e))?;

        // 推送到设备
        let device_path = format!("/data/local/tmp/{}", name);
        self.adb_client
            .push(
                &self.device_serial,
                temp_path.to_str().unwrap(),
                &device_path,
            )
            .await
            .map_err(|e| Error::Adb(format!("推送 {} 失败: {}", name, e)))?;

        // 安装 APK
        let output = self
            .adb_client
            .shell(
                &self.device_serial,
                &format!("pm install -r -t {}", device_path),
                Some(Duration::from_secs(180)),
            )
            .await
            .map_err(|e| Error::Adb(format!("安装 {} 失败: {}", name, e)))?;

        if !output.contains("Success") {
            return Err(Error::Adb(format!("安装 {} 失败: {}", name, output)));
        }

        // 清理设备上的 APK 文件
        let _ = self
            .adb_client
            .shell(
                &self.device_serial,
                &format!("rm {}", device_path),
                Some(Duration::from_secs(10)),
            )
            .await;

        // 清理临时文件
        let _ = std::fs::remove_file(&temp_path);

        info!("{} 安装成功", name);
        Ok(())
    }

    /// 完整安装 ATX-Agent
    ///
    /// 执行完整的安装流程：
    /// 1. 检查是否已安装
    /// 2. 推送 atx-agent 二进制文件
    /// 3. 安装 UiAutomator APK
    ///
    /// # 参数
    ///
    /// * `force` - 是否强制重新安装
    ///
    /// # 错误
    ///
    /// 如果安装过程中出现错误，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// client.install(false).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn install(&self, force: bool) -> Result<()> {
        info!("开始安装 ATX-Agent");

        // 检查是否已安装
        if !force {
            let installed = self.check_atx_agent_installed().await?;
            if installed {
                info!("ATX-Agent 已安装，跳过安装");
                return Ok(());
            }
        }

        // 推送 atx-agent 二进制文件
        self.push_atx_agent().await?;

        // 安装 UiAutomator APK
        self.install_uiautomator_apks().await?;

        info!("ATX-Agent 安装完成");
        Ok(())
    }
}

// ============================================================================
// ATX-Agent 服务管理
// ============================================================================

impl AtxAgentClient {
    /// 启动 atx-agent 守护进程
    ///
    /// 通过 ADB shell 启动 atx-agent 服务。
    ///
    /// # 错误
    ///
    /// 如果启动失败，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// client.start_atx_agent().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_atx_agent(&self) -> Result<()> {
        info!("启动 atx-agent 守护进程");

        // 检查是否已安装
        let installed = self.check_atx_agent_installed().await?;
        if !installed {
            return Err(Error::InvalidArgument(
                "atx-agent 未安装，请先调用 install() 方法".to_string(),
            ));
        }

        // 先尝试清理旧进程，避免重复启动导致状态不一致
        let _ = self.stop_atx_agent().await;

        // 启动守护进程
        // 使用 -d 参数以守护进程模式运行
        let command = format!("{} server -d", ATX_AGENT_DEVICE_PATH);

        self.adb_client
            .shell(&self.device_serial, &command, Some(Duration::from_secs(10)))
            .await
            .map_err(|e| Error::Adb(format!("启动 atx-agent 失败: {}", e)))?;

        info!("atx-agent 守护进程已启动");
        Ok(())
    }

    /// 等待 atx-agent 服务就绪
    ///
    /// 轮询检查服务是否可用，直到超时。
    ///
    /// # 参数
    ///
    /// * `timeout` - 超时时间，支持 `Duration` 或 `Option<Duration>`。
    ///   传入 `None` 时使用默认值（30 秒）
    ///
    /// # 错误
    ///
    /// 如果超时仍未就绪，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use std::time::Duration;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// client
    ///     .wait_for_atx_agent_ready(Some(Duration::from_secs(15)))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_atx_agent_ready<T>(&self, timeout: T) -> Result<()>
    where
        T: Into<Option<Duration>>,
    {
        let timeout = timeout.into().unwrap_or(Duration::from_secs(30));
        info!("等待 atx-agent 服务就绪");

        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(500);

        loop {
            if start.elapsed() > timeout {
                return Err(Error::Timeout);
            }

            // 尝试访问版本接口（短超时健康探测）
            if self
                .version_with_timeout(ATX_AGENT_HEALTHCHECK_TIMEOUT)
                .await
                .is_ok()
            {
                info!("atx-agent 服务已就绪");
                return Ok(());
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    /// 检查 atx-agent 服务状态
    ///
    /// # 返回
    ///
    /// 返回服务是否正在运行
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// let running = client.check_atx_agent_status().await?;
    /// println!("running: {running}");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check_atx_agent_status(&self) -> Result<bool> {
        debug!("检查 atx-agent 服务状态");

        if self
            .version_with_timeout(ATX_AGENT_HEALTHCHECK_TIMEOUT)
            .await
            .is_ok()
        {
            return Ok(true);
        }

        let output = self
            .adb_client
            .shell(
                &self.device_serial,
                "ps | grep atx-agent | grep -v grep",
                Some(Duration::from_secs(10)),
            )
            .await
            .map_err(|e| Error::Adb(format!("检查服务状态失败: {}", e)))?;

        if !output.trim().is_empty() {
            warn!("atx-agent 进程存在，但健康检查失败");
        }

        Ok(false)
    }

    /// 停止 atx-agent 服务
    ///
    /// 用于清理或重启服务。
    ///
    /// # 错误
    ///
    /// 如果停止失败，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// client.stop_atx_agent().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stop_atx_agent(&self) -> Result<()> {
        info!("停止 atx-agent 服务");

        // 查找进程 ID
        let output = self
            .adb_client
            .shell(
                &self.device_serial,
                "ps | grep atx-agent | grep -v grep | awk '{print $2}'",
                Some(Duration::from_secs(10)),
            )
            .await
            .map_err(|e| Error::Adb(format!("查找进程失败: {}", e)))?;

        let pids: Vec<&str> = output
            .split_whitespace()
            .filter(|pid| !pid.is_empty())
            .collect();

        if pids.is_empty() {
            info!("atx-agent 服务未运行");
            return Ok(());
        }

        // 杀死进程
        for pid in pids {
            self.adb_client
                .shell(
                    &self.device_serial,
                    &format!("kill {}", pid),
                    Some(Duration::from_secs(10)),
                )
                .await
                .map_err(|e| Error::Adb(format!("停止服务失败: {}", e)))?;
        }

        info!("atx-agent 服务已停止");
        Ok(())
    }

    /// 重启 atx-agent 服务
    ///
    /// 先停止再启动服务。
    ///
    /// # 错误
    ///
    /// 如果重启失败，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// client.restart_atx_agent().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn restart_atx_agent(&self) -> Result<()> {
        info!("重启 atx-agent 服务");

        // 停止服务
        self.stop_atx_agent().await?;

        // 等待一下确保进程完全停止
        tokio::time::sleep(Duration::from_secs(1)).await;

        // 启动服务
        self.start_atx_agent().await?;

        // 等待服务就绪
        self.wait_for_atx_agent_ready(Some(Duration::from_secs(30)))
            .await?;

        info!("atx-agent 服务重启完成");
        Ok(())
    }

    /// 确保 atx-agent 服务运行
    ///
    /// 检查服务状态，如果未运行则启动。
    ///
    /// # 错误
    ///
    /// 如果启动失败，返回错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use uiautomator::{AdbClient, AtxAgentClient};
    /// # async fn example() -> uiautomator::Result<()> {
    /// let adb = AdbClient::new().await?;
    /// let client = AtxAgentClient::new("emulator-5554".to_string(), Arc::new(adb)).await?;
    /// client.ensure_atx_agent_running().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn ensure_atx_agent_running(&self) -> Result<()> {
        debug!("确保 atx-agent 服务运行");

        // 检查服务状态
        let running = self.check_atx_agent_status().await?;

        if !running {
            info!("atx-agent 服务未运行，正在启动...");
            self.start_atx_agent().await?;
            self.wait_for_atx_agent_ready(Some(Duration::from_secs(30)))
                .await?;
        } else {
            debug!("atx-agent 服务已在运行");
        }

        Ok(())
    }
}
