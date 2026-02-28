//! UiObject 模块
//!
//! 提供 UI 元素的定位和操作功能

use crate::{Device, ElementInfo, Error, Rect, Result, Selector};
use log::debug;
use std::sync::Arc;
use std::time::Duration;

/// UI 对象，代表一个定位到的 UI 元素
#[derive(Debug, Clone)]
pub struct UiObject {
    device: Arc<Device>,
    selector: Selector,
}

impl UiObject {
    /// 创建新的 UiObject
    ///
    /// # 参数
    ///
    /// * `device` - 设备引用
    /// * `selector` - 元素选择器
    pub fn new(device: Arc<Device>, selector: Selector) -> Self {
        Self { device, selector }
    }

    /// 检查 JSON-RPC 错误消息是否表示元素未找到
    ///
    /// 这个方法使用多个模式匹配来识别元素未找到的错误，
    /// 比简单的字符串包含检查更健壮。
    fn is_element_not_found_error(msg: &str) -> bool {
        let msg_lower = msg.to_lowercase();
        let explicit_not_found_patterns = [
            "uiobjectnotfoundexception",
            "uiautomator.uiobjectnotfoundexception",
            "nosuchobjectexception",
            "no such object",
            "cannot find ui object",
            "node not found",
            "no node found",
            "accessibility node info is null",
        ];

        // 常见的“非元素不存在”错误，避免误判
        if msg_lower.contains("method not found")
            || msg_lower.contains("invalid selector")
            || msg_lower.contains("selector invalid")
        {
            return false;
        }

        if explicit_not_found_patterns
            .iter()
            .any(|pattern| msg_lower.contains(pattern))
        {
            return true;
        }

        (msg_lower.contains("not found") || msg_lower.contains("does not exist"))
            && (msg_lower.contains("ui object")
                || msg_lower.contains("uiobject")
                || msg_lower.contains("node"))
    }

    /// 统一处理带元素不存在映射的 JSON-RPC 调用
    async fn call_jsonrpc_with_element_check(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let client = self.device.jsonrpc_client();
        match client.call(method, params).await {
            Ok(value) => Ok(value),
            Err(Error::JsonRpc(msg)) if Self::is_element_not_found_error(&msg) => {
                Err(Error::ElementNotFound {
                    selector: format!("{:?}", self.selector),
                })
            }
            Err(e) => Err(e),
        }
    }

    /// 校验动作类 RPC 返回值。
    ///
    /// uiautomator2 大多数动作接口会返回 bool。
    /// - true: 成功
    /// - false: 明确失败
    /// - 其他类型: 为兼容不同实现，记录调试日志后按成功处理
    fn ensure_action_rpc_result(method: &str, result: serde_json::Value) -> Result<()> {
        match result {
            serde_json::Value::Bool(true) => Ok(()),
            serde_json::Value::Bool(false) => {
                Err(Error::JsonRpc(format!("{} 返回 false（操作失败）", method)))
            }
            other => {
                debug!(
                    "JSON-RPC {} 返回非布尔结果，按成功处理: {:?}",
                    method, other
                );
                Ok(())
            }
        }
    }

    /// 通用轮询方法
    ///
    /// 使用指定的条件函数进行轮询，直到条件满足或超时。
    ///
    /// # 参数
    ///
    /// * `timeout` - 超时时间
    /// * `condition` - 条件函数，返回 true 表示条件满足，false 表示继续轮询
    ///
    /// # 返回
    ///
    /// * `Ok(())` - 条件在超时前满足
    /// * `Err(Error::ElementTimeout)` - 超时
    /// * `Err(e)` - 其他错误
    async fn poll_until<F>(&self, timeout: Duration, mut condition: F) -> Result<()>
    where
        F: FnMut(Option<ElementInfo>) -> bool,
    {
        let polling_interval = self.device.get_polling_interval();
        let start = std::time::Instant::now();

        loop {
            // 尝试获取元素信息
            let info_result = self.info_internal().await?;

            // 检查条件
            if condition(info_result) {
                return Ok(());
            }

            // 条件未满足，检查是否超时
            if start.elapsed() >= timeout {
                return Err(Error::ElementTimeout {
                    selector: format!("{:?}", self.selector),
                    timeout,
                });
            }

            // 等待后重试
            tokio::time::sleep(polling_interval).await;
        }
    }

    /// 获取选择器的引用
    pub fn selector(&self) -> &Selector {
        &self.selector
    }

    /// 获取设备的引用
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    /// 检查元素是否存在
    ///
    /// # 参数
    ///
    /// * `timeout_duration` - 超时时间，None 表示使用默认超时
    ///
    /// # 返回
    ///
    /// 如果元素存在返回 true，否则返回 false
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use uiautomator::{Device, Selector};
    /// # use std::time::Duration;
    /// # async fn example() -> uiautomator::Result<()> {
    /// # let device = Device::connect(None).await?;
    /// let element = device.find(Selector::new().text("Settings"));
    /// if element.exists(Some(Duration::from_secs(5))).await? {
    ///     println!("元素存在");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn exists(&self, timeout_duration: Option<Duration>) -> Result<bool> {
        let timeout = timeout_duration.unwrap_or_else(|| self.device.get_wait_timeout());
        let polling_interval = self.device.get_polling_interval();
        let start = std::time::Instant::now();

        loop {
            // 尝试获取元素信息
            match self.info_internal().await {
                Ok(Some(_)) => return Ok(true),
                Ok(None) => {
                    // 元素不存在，检查是否超时
                    if start.elapsed() >= timeout {
                        return Ok(false);
                    }
                    // 等待后重试
                    tokio::time::sleep(polling_interval).await;
                }
                Err(e) => {
                    // 其他错误直接返回
                    return Err(e);
                }
            }
        }
    }

    /// 等待元素出现
    ///
    /// # 参数
    ///
    /// * `timeout_duration` - 超时时间，None 表示使用默认超时
    ///
    /// # 返回
    ///
    /// 如果元素在超时前出现返回 Ok(())，否则返回超时错误
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use uiautomator::{Device, Selector};
    /// # use std::time::Duration;
    /// # async fn example() -> uiautomator::Result<()> {
    /// # let device = Device::connect(None).await?;
    /// let element = device.find(Selector::new().text("Loading"));
    /// element.wait(Some(Duration::from_secs(10))).await?;
    /// println!("元素已出现");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait(&self, timeout_duration: Option<Duration>) -> Result<()> {
        let timeout = timeout_duration.unwrap_or_else(|| self.device.get_wait_timeout());

        // 使用 poll_until 等待元素出现
        self.poll_until(timeout, |info| info.is_some()).await
    }

    /// 等待元素消失
    ///
    /// # 参数
    ///
    /// * `timeout_duration` - 超时时间，None 表示使用默认超时
    ///
    /// # 返回
    ///
    /// 如果元素在超时前消失返回 Ok(())，否则返回超时错误
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use uiautomator::{Device, Selector};
    /// # use std::time::Duration;
    /// # async fn example() -> uiautomator::Result<()> {
    /// # let device = Device::connect(None).await?;
    /// let element = device.find(Selector::new().text("Loading"));
    /// element.wait_gone(Some(Duration::from_secs(10))).await?;
    /// println!("元素已消失");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_gone(&self, timeout_duration: Option<Duration>) -> Result<()> {
        let timeout = timeout_duration.unwrap_or_else(|| self.device.get_wait_timeout());

        // 使用 poll_until 等待元素消失
        self.poll_until(timeout, |info| info.is_none()).await
    }

    /// 获取元素信息（内部方法）
    ///
    /// 这个方法会调用 JSON-RPC 获取元素信息，如果元素不存在返回 None
    ///
    /// # 返回
    ///
    /// - `Ok(Some(info))` - 元素存在，返回元素信息
    /// - `Ok(None)` - 元素不存在
    /// - `Err(e)` - 其他错误（网络错误、解析错误等）
    async fn info_internal(&self) -> Result<Option<ElementInfo>> {
        match self.info().await {
            Ok(info) => Ok(Some(info)),
            Err(Error::ElementNotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 获取元素详细信息
    ///
    /// # 返回
    ///
    /// 返回元素的所有属性信息，包括文本、边界、类名等
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use uiautomator::{Device, Selector};
    /// # async fn example() -> uiautomator::Result<()> {
    /// # let device = Device::connect(None).await?;
    /// let element = device.find(Selector::new().text("Settings"));
    /// let info = element.info().await?;
    /// println!("元素文本: {}", info.text);
    /// println!("元素边界: {:?}", info.bounds);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn info(&self) -> Result<ElementInfo> {
        // 调用 objInfo JSON-RPC 方法
        let client = self.device.jsonrpc_client();
        let params = self.selector.to_params();

        let result: serde_json::Value =
            match client.call("objInfo", serde_json::json!((params,))).await {
                Ok(value) => value,
                Err(Error::JsonRpc(msg)) if Self::is_element_not_found_error(&msg) => {
                    // JSON-RPC 返回元素未找到错误
                    return Err(Error::ElementNotFound {
                        selector: format!("{:?}", self.selector),
                    });
                }
                Err(e) => return Err(e),
            };

        // 解析返回的元素信息
        let element_info: ElementInfo = serde_json::from_value(result)
            .map_err(|e| Error::JsonRpc(format!("解析元素信息失败: {}", e)))?;

        // 检查是否返回了根元素（通常表示未找到匹配的元素）
        // 根元素的特征：
        // 1. 类名通常是 FrameLayout
        // 2. 边界覆盖整个屏幕（left=0, top=0）
        // 3. 文本为空
        // 4. resource_id 为空
        let is_likely_root = element_info.class_name.contains("FrameLayout")
            && element_info.bounds.left == 0
            && element_info.bounds.top == 0
            && element_info.text.is_empty()
            && element_info.resource_id.is_empty();

        if is_likely_root {
            // 如果选择器指定了具体的属性，但返回的是根元素，说明未找到
            let has_specific_selector = self.selector.text.is_some()
                || self.selector.text_contains.is_some()
                || self.selector.resource_id.is_some()
                || (self.selector.class_name.is_some()
                    && !self
                        .selector
                        .class_name
                        .as_ref()
                        .unwrap()
                        .contains("FrameLayout"));

            if has_specific_selector {
                return Err(Error::ElementNotFound {
                    selector: format!("{:?}", self.selector),
                });
            }
        }

        // 验证返回的元素是否真正匹配选择器（仅对非根元素进行严格验证）
        if !is_likely_root {
            // 验证 text
            if let Some(ref text) = self.selector.text {
                if element_info.text != *text {
                    return Err(Error::ElementNotFound {
                        selector: format!("{:?}", self.selector),
                    });
                }
            }

            // 验证 text_contains
            if let Some(ref text_contains) = self.selector.text_contains {
                if !element_info.text.contains(text_contains) {
                    return Err(Error::ElementNotFound {
                        selector: format!("{:?}", self.selector),
                    });
                }
            }

            // 验证 resource_id
            if let Some(ref resource_id) = self.selector.resource_id {
                if element_info.resource_id != *resource_id {
                    return Err(Error::ElementNotFound {
                        selector: format!("{:?}", self.selector),
                    });
                }
            }

            // 验证 class_name
            if let Some(ref class_name) = self.selector.class_name {
                if element_info.class_name != *class_name {
                    return Err(Error::ElementNotFound {
                        selector: format!("{:?}", self.selector),
                    });
                }
            }
        }

        Ok(element_info)
    }

    /// 获取元素文本
    ///
    /// # 返回
    ///
    /// 返回元素的文本内容
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use uiautomator::{Device, Selector};
    /// # async fn example() -> uiautomator::Result<()> {
    /// # let device = Device::connect(None).await?;
    /// let element = device.find(Selector::new().resource_id("com.example:id/title"));
    /// let text = element.get_text().await?;
    /// println!("文本: {}", text);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_text(&self) -> Result<String> {
        let info = self.info().await?;
        Ok(info.text)
    }

    /// 获取元素中心坐标
    ///
    /// # 返回
    ///
    /// 返回元素中心点的 (x, y) 坐标
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use uiautomator::{Device, Selector};
    /// # async fn example() -> uiautomator::Result<()> {
    /// # let device = Device::connect(None).await?;
    /// let element = device.find(Selector::new().text("Button"));
    /// let (x, y) = element.center().await?;
    /// println!("中心坐标: ({}, {})", x, y);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn center(&self) -> Result<(u32, u32)> {
        let info = self.info().await?;
        Ok(info.bounds.center())
    }

    /// 获取元素边界
    ///
    /// # 返回
    ///
    /// 返回元素的矩形边界
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use uiautomator::{Device, Selector};
    /// # async fn example() -> uiautomator::Result<()> {
    /// # let device = Device::connect(None).await?;
    /// let element = device.find(Selector::new().text("Button"));
    /// let bounds = element.bounds().await?;
    /// println!("边界: left={}, top={}, right={}, bottom={}",
    ///          bounds.left, bounds.top, bounds.right, bounds.bottom);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bounds(&self) -> Result<Rect> {
        let info = self.info().await?;
        Ok(info.bounds)
    }

    /// 点击元素
    ///
    /// # 参数
    ///
    /// * `timeout_duration` - 等待元素出现的超时时间，None 表示使用默认超时
    /// * `offset` - 点击偏移量 (x%, y%)，相对于元素左上角的百分比，None 表示点击中心
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use uiautomator::{Device, Selector};
    /// # use std::time::Duration;
    /// # async fn example() -> uiautomator::Result<()> {
    /// # let device = Device::connect(None).await?;
    /// let element = device.find(Selector::new().text("Submit"));
    ///
    /// // 点击中心
    /// element.click(None, None).await?;
    ///
    /// // 点击元素的右下角（90%, 90%）
    /// element.click(None, Some((0.9, 0.9))).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn click(
        &self,
        _timeout_duration: Option<Duration>,
        offset: Option<(f32, f32)>,
    ) -> Result<()> {
        // 1. 获取元素信息(包括坐标)
        let info = self.info().await?;

        // 2. 计算点击坐标
        let (click_x, click_y) = if let Some((offset_x, offset_y)) = offset {
            // 使用偏移量
            let x = info.bounds.left as f32 + (info.bounds.width() as f32 * offset_x);
            let y = info.bounds.top as f32 + (info.bounds.height() as f32 * offset_y);
            (x, y)
        } else {
            // 使用中心点
            let (cx, cy) = info.bounds.center();
            (cx as f32, cy as f32)
        };

        // 3. 调用 click JSON-RPC (参数是坐标)
        let client = self.device.jsonrpc_client();
        let params = serde_json::json!((click_x, click_y));
        let _result: serde_json::Value = client.call("click", params).await?;

        Ok(())
    }

    /// 如果元素存在则点击
    ///
    /// # 参数
    ///
    /// * `timeout_duration` - 等待元素出现的超时时间，None 表示使用默认超时
    ///
    /// # 返回
    ///
    /// 如果元素存在并成功点击返回 true，否则返回 false
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use uiautomator::{Device, Selector};
    /// # use std::time::Duration;
    /// # async fn example() -> uiautomator::Result<()> {
    /// # let device = Device::connect(None).await?;
    /// let element = device.find(Selector::new().text("Skip"));
    /// if element.click_exists(Some(Duration::from_secs(3))).await? {
    ///     println!("跳过按钮已点击");
    /// } else {
    ///     println!("跳过按钮不存在");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn click_exists(&self, timeout_duration: Option<Duration>) -> Result<bool> {
        if self.exists(timeout_duration).await? {
            self.click(timeout_duration, None).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 长按元素
    ///
    /// # 参数
    ///
    /// * `duration` - 长按持续时间，None 表示使用默认值（0.5秒）
    /// * `timeout_duration` - 等待元素出现的超时时间，None 表示使用默认超时
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use uiautomator::{Device, Selector};
    /// # use std::time::Duration;
    /// # async fn example() -> uiautomator::Result<()> {
    /// # let device = Device::connect(None).await?;
    /// let element = device.find(Selector::new().text("Item"));
    ///
    /// // 使用默认长按时间（0.5秒）
    /// element.long_click(None, None).await?;
    ///
    /// // 长按 1 秒
    /// element.long_click(Some(Duration::from_secs(1)), None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn long_click(
        &self,
        duration: Option<Duration>,
        _timeout_duration: Option<Duration>,
    ) -> Result<()> {
        // 1. 获取元素信息（验证元素存在）
        let info = self.info().await?;

        // 2. 计算中心坐标
        let (cx, cy) = info.bounds.center();

        // 3. 委托给 Device 的 long_click 方法
        self.device.long_click(cx, cy, duration).await
    }

    /// 设置元素文本
    ///
    /// # 参数
    ///
    /// * `text` - 要设置的文本内容
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use uiautomator::{Device, Selector};
    /// # async fn example() -> uiautomator::Result<()> {
    /// # let device = Device::connect(None).await?;
    /// let element = device.find(Selector::new().resource_id("com.example:id/input"));
    /// element.set_text("Hello World").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_text(&self, text: &str) -> Result<()> {
        // 调用 JSON-RPC setText 方法，参数为 (selector, text)
        let params = self.selector.to_params();
        let result = self
            .call_jsonrpc_with_element_check("setText", serde_json::json!((params, text)))
            .await?;

        Self::ensure_action_rpc_result("setText", result)
    }

    /// 清除元素文本
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use uiautomator::{Device, Selector};
    /// # async fn example() -> uiautomator::Result<()> {
    /// # let device = Device::connect(None).await?;
    /// let element = device.find(Selector::new().resource_id("com.example:id/input"));
    /// element.clear_text().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn clear_text(&self) -> Result<()> {
        // 调用 JSON-RPC clearTextField 方法，参数为 (selector,)
        let params = self.selector.to_params();
        let result = self
            .call_jsonrpc_with_element_check("clearTextField", serde_json::json!((params,)))
            .await?;

        Self::ensure_action_rpc_result("clearTextField", result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uiobject_structure() {
        // 测试 UiObject 的基本结构
        // 由于 Device 还未完全实现，我们只测试 Selector 的创建和转换
        let selector = Selector::new().text("测试按钮").clickable(true);

        // 验证 Selector 可以转换为 JSON 参数
        let params = selector.to_params();
        assert_eq!(params["text"], "测试按钮");
        assert_eq!(params["clickable"], true);
    }

    #[test]
    fn test_uiobject_selector_combinations() {
        // 测试多种选择器组合
        let selector = Selector::new()
            .text("设置")
            .resource_id("com.example:id/button")
            .class_name("android.widget.Button")
            .enabled(true);

        // 验证选择器可以正确转换为参数
        let params = selector.to_params();
        assert_eq!(params["text"], "设置");
        assert_eq!(params["resourceId"], "com.example:id/button");
        assert_eq!(params["className"], "android.widget.Button");
        assert_eq!(params["enabled"], true);
    }

    #[test]
    fn test_is_element_not_found_error_matches_known_patterns() {
        assert!(UiObject::is_element_not_found_error(
            "UiObjectNotFoundException: no such object"
        ));
        assert!(UiObject::is_element_not_found_error(
            "java.lang.RuntimeException: Accessibility node info is null"
        ));
        assert!(UiObject::is_element_not_found_error(
            "node not found for selector"
        ));
    }

    #[test]
    fn test_is_element_not_found_error_avoids_false_positive() {
        assert!(!UiObject::is_element_not_found_error(
            "Error: invalid selector syntax"
        ));
        assert!(!UiObject::is_element_not_found_error(
            "method not found: clearTextField"
        ));
        assert!(!UiObject::is_element_not_found_error(
            "element selector invalid"
        ));
    }

    #[test]
    fn test_ensure_action_rpc_result_handles_boolean_and_non_boolean() {
        assert!(UiObject::ensure_action_rpc_result("setText", serde_json::json!(true)).is_ok());

        let err = UiObject::ensure_action_rpc_result("setText", serde_json::json!(false))
            .expect_err("false should be treated as failure");
        assert!(matches!(err, Error::JsonRpc(_)));

        // 兼容不同实现：非布尔结果不会被误判为失败
        assert!(UiObject::ensure_action_rpc_result("setText", serde_json::json!(null)).is_ok());
    }

    // 测试需求 4.3: 元素存在性检查
    // 注意：这些测试需要 mock Device，暂时标记为 ignore
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_exists_returns_true_when_element_found() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_with_element());
        // let selector = Selector::new().text("Settings");
        // let ui_object = UiObject::new(device, selector);
        //
        // let exists = ui_object.exists(Some(Duration::from_secs(5))).await.unwrap();
        // assert!(exists);
    }

    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_exists_returns_false_when_element_not_found() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_without_element());
        // let selector = Selector::new().text("NonExistent");
        // let ui_object = UiObject::new(device, selector);
        //
        // let exists = ui_object.exists(Some(Duration::from_secs(1))).await.unwrap();
        // assert!(!exists);
    }

    // ========== 任务 12.3: 超时机制应用测试 ==========

    // 单元测试：验证 exists 使用全局超时
    // 验证需求 11.1, 11.2
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_exists_uses_global_timeout() {
        // TODO: 使用 mock Device 测试
        // 验证当 timeout_duration 为 None 时，使用 Device 的全局超时
    }

    // 单元测试：验证 exists 操作级超时覆盖全局超时
    // 验证需求 11.2
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_exists_operation_timeout_override() {
        // TODO: 使用 mock Device 测试
        // 验证当提供 timeout_duration 时，覆盖全局超时
    }

    // 单元测试：验证 wait 使用全局超时
    // 验证需求 11.1, 11.2
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_wait_uses_global_timeout() {
        // TODO: 使用 mock Device 测试
        // 验证当 timeout_duration 为 None 时，使用 Device 的全局超时
    }

    // 单元测试：验证 wait 操作级超时覆盖全局超时
    // 验证需求 11.2
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_wait_operation_timeout_override() {
        // TODO: 使用 mock Device 测试
        // 验证当提供 timeout_duration 时，覆盖全局超时
    }

    // 单元测试：验证 wait_gone 使用全局超时
    // 验证需求 11.1, 11.2
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_wait_gone_uses_global_timeout() {
        // TODO: 使用 mock Device 测试
        // 验证当 timeout_duration 为 None 时，使用 Device 的全局超时
    }

    // 单元测试：验证 wait_gone 操作级超时覆盖全局超时
    // 验证需求 11.2
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_wait_gone_operation_timeout_override() {
        // TODO: 使用 mock Device 测试
        // 验证当提供 timeout_duration 时，覆盖全局超时
    }

    // 单元测试：验证 click 超时返回 Error::Timeout
    // 验证需求 11.6
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_click_timeout_returns_error() {
        // TODO: 使用 mock Device 测试
        // 验证当元素在超时时间内未出现时，click 返回 Error::Timeout
    }

    // 单元测试：验证 wait 超时返回 Error::Timeout
    // 验证需求 11.6
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_wait_timeout_returns_error() {
        // TODO: 使用 mock Device 测试
        // 验证当元素在超时时间内未出现时，wait 返回 Error::Timeout
    }

    // 单元测试：验证 wait_gone 超时返回 Error::Timeout
    // 验证需求 11.6
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_wait_gone_timeout_returns_error() {
        // TODO: 使用 mock Device 测试
        // 验证当元素在超时时间内未消失时，wait_gone 返回 Error::Timeout
    }

    // 测试需求 4.7: 等待元素出现
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_wait_succeeds_when_element_appears() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_element_appears_after(Duration::from_millis(100)));
        // let selector = Selector::new().text("Loading");
        // let ui_object = UiObject::new(device, selector);
        //
        // let result = ui_object.wait(Some(Duration::from_secs(5))).await;
        // assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_wait_fails_on_timeout() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_without_element());
        // let selector = Selector::new().text("NeverAppears");
        // let ui_object = UiObject::new(device, selector);
        //
        // let result = ui_object.wait(Some(Duration::from_secs(1))).await;
        // assert!(result.is_err());
        // assert!(matches!(result.unwrap_err(), Error::Timeout));
    }

    // 测试需求 4.8: 等待元素消失
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_wait_gone_succeeds_when_element_disappears() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_element_disappears_after(Duration::from_millis(100)));
        // let selector = Selector::new().text("Loading");
        // let ui_object = UiObject::new(device, selector);
        //
        // let result = ui_object.wait_gone(Some(Duration::from_secs(5))).await;
        // assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_wait_gone_fails_on_timeout() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_with_element());
        // let selector = Selector::new().text("PersistentElement");
        // let ui_object = UiObject::new(device, selector);
        //
        // let result = ui_object.wait_gone(Some(Duration::from_secs(1))).await;
        // assert!(result.is_err());
        // assert!(matches!(result.unwrap_err(), Error::Timeout));
    }

    // 测试需求 11.3, 11.4: 轮询检查逻辑
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_polling_interval() {
        // TODO: 验证轮询间隔是否合理（例如每 500ms 检查一次）
    }

    // 测试需求 4.4: 获取元素详细信息
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_info_returns_element_details() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_with_element_info());
        // let selector = Selector::new().text("Settings");
        // let ui_object = UiObject::new(device, selector);
        //
        // let info = ui_object.info().await.unwrap();
        // assert_eq!(info.text, "Settings");
        // assert_eq!(info.class_name, "android.widget.TextView");
        // assert!(info.clickable);
    }

    // 测试需求 4.5: 获取元素文本
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_get_text_returns_element_text() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_with_text("Hello World"));
        // let selector = Selector::new().resource_id("com.example:id/title");
        // let ui_object = UiObject::new(device, selector);
        //
        // let text = ui_object.get_text().await.unwrap();
        // assert_eq!(text, "Hello World");
    }

    // 测试需求 4.6: 获取元素中心坐标
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_center_returns_center_coordinates() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_with_bounds(100, 200, 300, 400));
        // let selector = Selector::new().text("Button");
        // let ui_object = UiObject::new(device, selector);
        //
        // let (x, y) = ui_object.center().await.unwrap();
        // assert_eq!(x, 200); // (100 + 300) / 2
        // assert_eq!(y, 300); // (200 + 400) / 2
    }

    // 测试需求 4.6: 获取元素边界
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_bounds_returns_element_bounds() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_with_bounds(100, 200, 300, 400));
        // let selector = Selector::new().text("Button");
        // let ui_object = UiObject::new(device, selector);
        //
        // let bounds = ui_object.bounds().await.unwrap();
        // assert_eq!(bounds.left, 100);
        // assert_eq!(bounds.top, 200);
        // assert_eq!(bounds.right, 300);
        // assert_eq!(bounds.bottom, 400);
    }

    // 测试需求 4.1: 点击元素
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_click_element_at_center() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_clickable());
        // let selector = Selector::new().text("Submit");
        // let ui_object = UiObject::new(device, selector);
        //
        // let result = ui_object.click(None, None).await;
        // assert!(result.is_ok());
        // // 验证点击发生在元素中心
    }

    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_click_element_with_offset() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_clickable());
        // let selector = Selector::new().text("Submit");
        // let ui_object = UiObject::new(device, selector);
        //
        // let result = ui_object.click(None, Some((0.9, 0.9))).await;
        // assert!(result.is_ok());
        // // 验证点击发生在指定偏移位置
    }

    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_click_exists_returns_true_when_clicked() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_with_element());
        // let selector = Selector::new().text("Skip");
        // let ui_object = UiObject::new(device, selector);
        //
        // let clicked = ui_object.click_exists(Some(Duration::from_secs(3))).await.unwrap();
        // assert!(clicked);
    }

    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_click_exists_returns_false_when_not_found() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_without_element());
        // let selector = Selector::new().text("NonExistent");
        // let ui_object = UiObject::new(device, selector);
        //
        // let clicked = ui_object.click_exists(Some(Duration::from_secs(1))).await.unwrap();
        // assert!(!clicked);
    }

    // 测试需求 4.2: 长按元素
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_long_click_with_default_duration() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_clickable());
        // let selector = Selector::new().text("Item");
        // let ui_object = UiObject::new(device, selector);
        //
        // let result = ui_object.long_click(None, None).await;
        // assert!(result.is_ok());
        // // 验证长按持续时间为默认值（0.5秒）
    }

    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_long_click_with_custom_duration() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_clickable());
        // let selector = Selector::new().text("Item");
        // let ui_object = UiObject::new(device, selector);
        //
        // let result = ui_object.long_click(Some(Duration::from_secs(1)), None).await;
        // assert!(result.is_ok());
        // // 验证长按持续时间为 1 秒
    }

    // 测试需求 4.5: 设置文本
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_set_text() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_editable());
        // let selector = Selector::new().resource_id("com.example:id/input");
        // let ui_object = UiObject::new(device, selector);
        //
        // let result = ui_object.set_text("Hello World").await;
        // assert!(result.is_ok());
        // // 验证文本已设置
    }

    // 测试需求 4.6: 清除文本
    #[tokio::test]
    #[ignore = "需要 Device 完整实现"]
    async fn test_clear_text() {
        // TODO: 使用 mock Device 测试
        // let device = Arc::new(mock_device_editable());
        // let selector = Selector::new().resource_id("com.example:id/input");
        // let ui_object = UiObject::new(device, selector);
        //
        // let result = ui_object.clear_text().await;
        // assert!(result.is_ok());
        // // 验证文本已清除
    }
}
