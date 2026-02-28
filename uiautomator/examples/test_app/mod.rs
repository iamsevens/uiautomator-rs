//! 测试应用框架
//!
//! 提供测试基础设施和工具函数

pub mod basic_controls;

use std::time::{Duration, Instant};
use uiautomator::{Device, Result, Selector};

/// 测试应用包名
pub const TEST_APP_PACKAGE: &str = "com.uiautomator.testapp";

/// 测试应用主 Activity
pub const TEST_APP_ACTIVITY: &str = ".MainActivity";

/// 测试结果
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration: Duration,
    pub error: Option<String>,
}

/// 测试上下文
pub struct TestContext {
    pub device: Device,
    pub results: Vec<TestResult>,
}

impl TestContext {
    /// 创建测试上下文
    pub async fn new() -> Result<Self> {
        println!("🔌 正在连接设备...");
        let device = Device::connect(None).await?;
        println!("✅ 设备连接成功: {}", device.serial());

        Ok(Self {
            device,
            results: Vec::new(),
        })
    }

    /// 启动测试应用
    pub async fn launch_test_app(&self) -> Result<()> {
        println!("🚀 启动测试应用...");
        self.device
            .app_start(TEST_APP_PACKAGE, Some(TEST_APP_ACTIVITY))
            .await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("✅ 应用启动成功");
        Ok(())
    }

    /// 返回主页面
    pub async fn go_home(&self) -> Result<()> {
        // 多次按返回键确保回到主页面
        for _ in 0..5 {
            self.device.press(uiautomator::Key::Back).await?;
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
        Ok(())
    }

    /// 运行单个测试
    pub async fn run_test<F>(&mut self, name: &str, test_fn: F)
    where
        F: for<'a> FnOnce(
            &'a Device,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>>,
    {
        println!("\n📝 运行测试: {}", name);
        let start = Instant::now();

        match test_fn(&self.device).await {
            Ok(_) => {
                let duration = start.elapsed();
                println!("✅ 通过: {} ({:.2}s)", name, duration.as_secs_f64());
                self.results.push(TestResult {
                    name: name.to_string(),
                    passed: true,
                    duration,
                    error: None,
                });
            }
            Err(e) => {
                let duration = start.elapsed();
                println!("❌ 失败: {} - {:?}", name, e);
                self.results.push(TestResult {
                    name: name.to_string(),
                    passed: false,
                    duration,
                    error: Some(format!("{:?}", e)),
                });
            }
        }
    }

    /// 打印测试摘要
    pub fn print_summary(&self) {
        println!("\n{}", "=".repeat(60));
        println!("📊 测试摘要");
        println!("{}", "=".repeat(60));

        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let total_duration: Duration = self.results.iter().map(|r| r.duration).sum();

        println!("总计: {}", total);
        println!("通过: {} ✅", passed);
        println!("失败: {} ❌", failed);
        println!("成功率: {:.1}%", (passed as f64 / total as f64) * 100.0);
        println!("总耗时: {:.2}s", total_duration.as_secs_f64());

        if failed > 0 {
            println!("\n失败的测试:");
            for result in &self.results {
                if !result.passed {
                    println!("  ❌ {} - {}", result.name, result.error.as_ref().unwrap());
                }
            }
        }

        println!("{}", "=".repeat(60));
    }

    /// 截图保存
    #[allow(dead_code)]
    pub async fn take_screenshot(&self, name: &str) -> Result<()> {
        let filename = format!("screenshot_{}.png", name);
        self.device.screenshot_to_file(&filename).await?;
        println!("📸 截图已保存: {}", filename);
        Ok(())
    }
}

/// 辅助函数：点击菜单项
pub async fn click_menu_item(device: &Device, text: &str) -> Result<()> {
    device
        .find(Selector::new().text(text))
        .click(None, None)
        .await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(())
}

/// 辅助函数：通过 Resource ID 点击
pub async fn click_by_id(device: &Device, id: &str) -> Result<()> {
    let full_id = format!("{}:id/{}", TEST_APP_PACKAGE, id);
    device
        .find(Selector::new().resource_id(&full_id))
        .click(None, None)
        .await?;
    Ok(())
}

/// 辅助函数：通过 Resource ID 获取文本
pub async fn get_text_by_id(device: &Device, id: &str) -> Result<String> {
    let full_id = format!("{}:id/{}", TEST_APP_PACKAGE, id);
    device
        .find(Selector::new().resource_id(&full_id))
        .get_text()
        .await
}

/// 辅助函数：通过 Resource ID 输入文本
#[allow(dead_code)]
pub async fn set_text_by_id(device: &Device, id: &str, text: &str) -> Result<()> {
    let full_id = format!("{}:id/{}", TEST_APP_PACKAGE, id);
    device
        .find(Selector::new().resource_id(&full_id))
        .set_text(text)
        .await
}

/// 辅助函数：验证文本包含
#[allow(dead_code)]
pub async fn assert_text_contains(device: &Device, id: &str, expected: &str) -> Result<()> {
    let text = get_text_by_id(device, id).await?;
    if !text.contains(expected) {
        return Err(uiautomator::Error::ElementNotFound {
            selector: format!("{}:id/{}", TEST_APP_PACKAGE, id),
        });
    }
    Ok(())
}
