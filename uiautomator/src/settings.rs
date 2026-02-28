//! 配置设置
//!
//! 本模块定义了库的全局配置选项，如超时时间、重试次数等。

use std::time::Duration;

/// 库的配置设置
///
/// 包含各种超时时间、延迟和重试策略的配置。
#[derive(Debug, Clone)]
pub struct Settings {
    /// 等待元素出现的默认超时时间
    ///
    /// 用于 `wait()`, `exists()` 等方法的默认超时。
    /// 默认值: 20 秒
    pub wait_timeout: Duration,

    /// 操作前延迟
    ///
    /// 在执行操作（如点击、滑动）之前等待的时间。
    /// 默认值: 0 毫秒
    pub operation_delay_before: Duration,

    /// 操作后延迟
    ///
    /// 在执行操作（如点击、滑动）之后等待的时间。
    /// 默认值: 0 毫秒
    pub operation_delay_after: Duration,

    /// HTTP 请求超时时间
    ///
    /// 发送 JSON-RPC 请求到设备端的超时时间。
    /// 默认值: 60 秒
    pub http_timeout: Duration,

    /// 最大重试次数
    ///
    /// 当操作失败时的最大重试次数。
    /// 默认值: 3 次
    pub max_retry: u32,

    /// 元素轮询间隔
    ///
    /// 在等待元素出现或消失时，每次检查之间的间隔时间。
    /// 默认值: 500 毫秒
    pub polling_interval: Duration,

    /// 重试基础延迟
    ///
    /// JSON-RPC 重试时的基础延迟时间，实际延迟使用指数退避策略。
    /// 默认值: 500 毫秒
    pub retry_base_delay: Duration,
}

impl Default for Settings {
    /// 创建默认配置
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Settings;
    ///
    /// let settings = Settings::default();
    /// assert_eq!(settings.wait_timeout.as_secs(), 20);
    /// assert_eq!(settings.max_retry, 3);
    /// ```
    fn default() -> Self {
        Self {
            wait_timeout: Duration::from_secs(20),
            operation_delay_before: Duration::from_millis(0),
            operation_delay_after: Duration::from_millis(0),
            http_timeout: Duration::from_secs(60),
            max_retry: 3,
            polling_interval: Duration::from_millis(500),
            retry_base_delay: Duration::from_millis(500),
        }
    }
}

impl Settings {
    /// 创建新的配置（使用默认值）
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Settings;
    ///
    /// let settings = Settings::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置等待超时时间
    ///
    /// # 参数
    ///
    /// * `timeout` - 新的超时时间
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Settings;
    /// use std::time::Duration;
    ///
    /// let mut settings = Settings::new();
    /// settings.set_wait_timeout(Duration::from_secs(30));
    /// assert_eq!(settings.wait_timeout.as_secs(), 30);
    /// ```
    pub fn set_wait_timeout(&mut self, timeout: Duration) {
        self.wait_timeout = timeout;
    }

    /// 设置操作前延迟
    ///
    /// # 参数
    ///
    /// * `delay` - 新的延迟时间
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Settings;
    /// use std::time::Duration;
    ///
    /// let mut settings = Settings::new();
    /// settings.set_operation_delay_before(Duration::from_millis(100));
    /// assert_eq!(settings.operation_delay_before.as_millis(), 100);
    /// ```
    pub fn set_operation_delay_before(&mut self, delay: Duration) {
        self.operation_delay_before = delay;
    }

    /// 设置操作后延迟
    ///
    /// # 参数
    ///
    /// * `delay` - 新的延迟时间
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Settings;
    /// use std::time::Duration;
    ///
    /// let mut settings = Settings::new();
    /// settings.set_operation_delay_after(Duration::from_millis(200));
    /// assert_eq!(settings.operation_delay_after.as_millis(), 200);
    /// ```
    pub fn set_operation_delay_after(&mut self, delay: Duration) {
        self.operation_delay_after = delay;
    }

    /// 设置 HTTP 超时时间
    ///
    /// # 参数
    ///
    /// * `timeout` - 新的超时时间
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Settings;
    /// use std::time::Duration;
    ///
    /// let mut settings = Settings::new();
    /// settings.set_http_timeout(Duration::from_secs(120));
    /// assert_eq!(settings.http_timeout.as_secs(), 120);
    /// ```
    pub fn set_http_timeout(&mut self, timeout: Duration) {
        self.http_timeout = timeout;
    }

    /// 设置最大重试次数
    ///
    /// # 参数
    ///
    /// * `max_retry` - 新的最大重试次数
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Settings;
    ///
    /// let mut settings = Settings::new();
    /// settings.set_max_retry(5);
    /// assert_eq!(settings.max_retry, 5);
    /// ```
    pub fn set_max_retry(&mut self, max_retry: u32) {
        self.max_retry = max_retry;
    }

    /// 设置轮询间隔
    ///
    /// # 参数
    ///
    /// * `interval` - 新的轮询间隔时间
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Settings;
    /// use std::time::Duration;
    ///
    /// let mut settings = Settings::new();
    /// settings.set_polling_interval(Duration::from_millis(300));
    /// assert_eq!(settings.polling_interval.as_millis(), 300);
    /// ```
    pub fn set_polling_interval(&mut self, interval: Duration) {
        self.polling_interval = interval;
    }

    /// 设置重试基础延迟
    ///
    /// # 参数
    ///
    /// * `delay` - 新的重试基础延迟时间
    ///
    /// # 注意
    ///
    /// - 如果延迟为零，将使用默认值 500ms
    /// - 如果延迟超过 60 秒，将记录警告
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Settings;
    /// use std::time::Duration;
    ///
    /// let mut settings = Settings::new();
    /// settings.set_retry_base_delay(Duration::from_millis(1000));
    /// assert_eq!(settings.retry_base_delay.as_millis(), 1000);
    /// ```
    pub fn set_retry_base_delay(&mut self, delay: Duration) {
        if delay.as_millis() == 0 {
            log::warn!("retry_base_delay 不应为零，使用默认值 500ms");
            self.retry_base_delay = Duration::from_millis(500);
        } else if delay.as_secs() > 60 {
            log::warn!("retry_base_delay 非常大 (>60s)，这可能导致长时间延迟");
            self.retry_base_delay = delay;
        } else {
            self.retry_base_delay = delay;
        }
    }

    /// 使用构建器模式设置等待超时时间
    ///
    /// # 参数
    ///
    /// * `timeout` - 新的超时时间
    ///
    /// # 返回
    ///
    /// 返回 self 以支持链式调用
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Settings;
    /// use std::time::Duration;
    ///
    /// let settings = Settings::new()
    ///     .with_wait_timeout(Duration::from_secs(30))
    ///     .with_max_retry(5);
    /// ```
    pub fn with_wait_timeout(mut self, timeout: Duration) -> Self {
        self.wait_timeout = timeout;
        self
    }

    /// 使用构建器模式设置操作前延迟
    ///
    /// # 参数
    ///
    /// * `delay` - 新的延迟时间
    ///
    /// # 返回
    ///
    /// 返回 self 以支持链式调用
    pub fn with_operation_delay_before(mut self, delay: Duration) -> Self {
        self.operation_delay_before = delay;
        self
    }

    /// 使用构建器模式设置操作后延迟
    ///
    /// # 参数
    ///
    /// * `delay` - 新的延迟时间
    ///
    /// # 返回
    ///
    /// 返回 self 以支持链式调用
    pub fn with_operation_delay_after(mut self, delay: Duration) -> Self {
        self.operation_delay_after = delay;
        self
    }

    /// 使用构建器模式设置 HTTP 超时时间
    ///
    /// # 参数
    ///
    /// * `timeout` - 新的超时时间
    ///
    /// # 返回
    ///
    /// 返回 self 以支持链式调用
    pub fn with_http_timeout(mut self, timeout: Duration) -> Self {
        self.http_timeout = timeout;
        self
    }

    /// 使用构建器模式设置最大重试次数
    ///
    /// # 参数
    ///
    /// * `max_retry` - 新的最大重试次数
    ///
    /// # 返回
    ///
    /// 返回 self 以支持链式调用
    pub fn with_max_retry(mut self, max_retry: u32) -> Self {
        self.max_retry = max_retry;
        self
    }

    /// 使用构建器模式设置轮询间隔
    ///
    /// # 参数
    ///
    /// * `interval` - 新的轮询间隔时间
    ///
    /// # 返回
    ///
    /// 返回 self 以支持链式调用
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Settings;
    /// use std::time::Duration;
    ///
    /// let settings = Settings::new()
    ///     .with_polling_interval(Duration::from_millis(300));
    /// assert_eq!(settings.polling_interval.as_millis(), 300);
    /// ```
    pub fn with_polling_interval(mut self, interval: Duration) -> Self {
        self.polling_interval = interval;
        self
    }

    /// 使用构建器模式设置重试基础延迟
    ///
    /// # 参数
    ///
    /// * `delay` - 新的重试基础延迟时间
    ///
    /// # 返回
    ///
    /// 返回 self 以支持链式调用
    ///
    /// # 注意
    ///
    /// - 如果延迟为零，将使用默认值 500ms
    /// - 如果延迟超过 60 秒，将记录警告
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Settings;
    /// use std::time::Duration;
    ///
    /// let settings = Settings::new()
    ///     .with_retry_base_delay(Duration::from_millis(1000));
    /// assert_eq!(settings.retry_base_delay.as_millis(), 1000);
    /// ```
    pub fn with_retry_base_delay(mut self, delay: Duration) -> Self {
        self.set_retry_base_delay(delay);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.wait_timeout, Duration::from_secs(20));
        assert_eq!(settings.operation_delay_before, Duration::from_millis(0));
        assert_eq!(settings.operation_delay_after, Duration::from_millis(0));
        assert_eq!(settings.http_timeout, Duration::from_secs(60));
        assert_eq!(settings.max_retry, 3);
        assert_eq!(settings.polling_interval, Duration::from_millis(500));
    }

    #[test]
    fn test_new_settings() {
        let settings = Settings::new();
        assert_eq!(settings.wait_timeout, Duration::from_secs(20));
    }

    #[test]
    fn test_set_wait_timeout() {
        let mut settings = Settings::new();
        settings.set_wait_timeout(Duration::from_secs(30));
        assert_eq!(settings.wait_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_set_operation_delays() {
        let mut settings = Settings::new();
        settings.set_operation_delay_before(Duration::from_millis(100));
        settings.set_operation_delay_after(Duration::from_millis(200));
        assert_eq!(settings.operation_delay_before, Duration::from_millis(100));
        assert_eq!(settings.operation_delay_after, Duration::from_millis(200));
    }

    #[test]
    fn test_set_http_timeout() {
        let mut settings = Settings::new();
        settings.set_http_timeout(Duration::from_secs(120));
        assert_eq!(settings.http_timeout, Duration::from_secs(120));
    }

    #[test]
    fn test_set_max_retry() {
        let mut settings = Settings::new();
        settings.set_max_retry(5);
        assert_eq!(settings.max_retry, 5);
    }

    #[test]
    fn test_set_polling_interval() {
        let mut settings = Settings::new();
        settings.set_polling_interval(Duration::from_millis(300));
        assert_eq!(settings.polling_interval, Duration::from_millis(300));
    }

    #[test]
    fn test_builder_pattern() {
        let settings = Settings::new()
            .with_wait_timeout(Duration::from_secs(30))
            .with_operation_delay_before(Duration::from_millis(100))
            .with_operation_delay_after(Duration::from_millis(200))
            .with_http_timeout(Duration::from_secs(120))
            .with_max_retry(5)
            .with_polling_interval(Duration::from_millis(300));

        assert_eq!(settings.wait_timeout, Duration::from_secs(30));
        assert_eq!(settings.operation_delay_before, Duration::from_millis(100));
        assert_eq!(settings.operation_delay_after, Duration::from_millis(200));
        assert_eq!(settings.http_timeout, Duration::from_secs(120));
        assert_eq!(settings.max_retry, 5);
        assert_eq!(settings.polling_interval, Duration::from_millis(300));
    }

    #[test]
    fn test_clone() {
        let settings1 = Settings::new().with_wait_timeout(Duration::from_secs(30));
        let settings2 = settings1.clone();
        assert_eq!(settings1.wait_timeout, settings2.wait_timeout);
    }
}
