// 选择器可靠性测试 - Rust 版本
//
// 运行方式：
// cargo run --example selector_reliability_test

use serde::{Deserialize, Serialize};
use std::time::Duration;
use uiautomator::{Device, Selector};

#[derive(Debug, Serialize, Deserialize)]
struct TestResult {
    test_name: String,
    success: bool,
    details: serde_json::Value,
    timestamp: f64,
}

struct SelectorReliabilityTest {
    device: Device,
    results: Vec<TestResult>,
}

impl SelectorReliabilityTest {
    async fn new(serial: Option<&str>) -> uiautomator::Result<Self> {
        let device = Device::connect(serial).await?;
        Ok(Self {
            device,
            results: Vec::new(),
        })
    }

    fn log_result(&mut self, test_name: &str, success: bool, details: serde_json::Value) {
        let result = TestResult {
            test_name: test_name.to_string(),
            success,
            details: details.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        };
        self.results.push(result);

        let symbol = if success { "✓" } else { "✗" };
        println!("[{}] {}", symbol, test_name);
        if !details.is_null() {
            println!(
                "    详情: {}",
                serde_json::to_string_pretty(&details).unwrap()
            );
        }
    }

    async fn test_text_exact_match(&mut self) {
        let test_name = "text_exact_match";
        match self.test_text_exact_match_impl().await {
            Ok(details) => self.log_result(test_name, true, details),
            Err(e) => self.log_result(
                test_name,
                false,
                serde_json::json!({"error": e.to_string()}),
            ),
        }
    }

    async fn test_text_exact_match_impl(&self) -> uiautomator::Result<serde_json::Value> {
        let element = self.device.find(Selector::new().text("设置"));
        let exists = element.exists(Some(Duration::from_secs(5))).await?;

        if exists {
            let info = element.info().await?;
            Ok(serde_json::json!({
                "found": true,
                "text": info.text,
                "className": info.class_name,
                "resourceId": info.resource_id,
                "bounds": {
                    "left": info.bounds.left,
                    "top": info.bounds.top,
                    "right": info.bounds.right,
                    "bottom": info.bounds.bottom,
                }
            }))
        } else {
            Ok(serde_json::json!({"found": false}))
        }
    }

    async fn test_text_contains(&mut self) {
        let test_name = "text_contains";
        match self.test_text_contains_impl().await {
            Ok(details) => self.log_result(test_name, true, details),
            Err(e) => self.log_result(
                test_name,
                false,
                serde_json::json!({"error": e.to_string()}),
            ),
        }
    }

    async fn test_text_contains_impl(&self) -> uiautomator::Result<serde_json::Value> {
        let element = self.device.find(Selector::new().text_contains("设"));
        let exists = element.exists(Some(Duration::from_secs(5))).await?;

        if exists {
            let info = element.info().await?;
            Ok(serde_json::json!({
                "found": true,
                "text": info.text,
                "className": info.class_name,
            }))
        } else {
            Ok(serde_json::json!({"found": false}))
        }
    }

    async fn test_resource_id(&mut self) {
        let test_name = "resource_id_match";
        match self.test_resource_id_impl().await {
            Ok(details) => self.log_result(test_name, true, details),
            Err(e) => self.log_result(
                test_name,
                false,
                serde_json::json!({"error": e.to_string()}),
            ),
        }
    }

    async fn test_resource_id_impl(&self) -> uiautomator::Result<serde_json::Value> {
        let element = self
            .device
            .find(Selector::new().resource_id("android:id/content"));
        let exists = element.exists(Some(Duration::from_secs(5))).await?;

        if exists {
            let info = element.info().await?;
            Ok(serde_json::json!({
                "found": true,
                "resourceId": info.resource_id,
                "className": info.class_name,
            }))
        } else {
            Ok(serde_json::json!({"found": false}))
        }
    }

    async fn test_class_name(&mut self) {
        let test_name = "class_name_match";
        match self.test_class_name_impl().await {
            Ok(details) => self.log_result(test_name, true, details),
            Err(e) => self.log_result(
                test_name,
                false,
                serde_json::json!({"error": e.to_string()}),
            ),
        }
    }

    async fn test_class_name_impl(&self) -> uiautomator::Result<serde_json::Value> {
        let element = self
            .device
            .find(Selector::new().class_name("android.widget.TextView"));
        let exists = element.exists(Some(Duration::from_secs(5))).await?;

        if exists {
            let info = element.info().await?;
            Ok(serde_json::json!({
                "found": true,
                "className": info.class_name,
                "text": info.text,
            }))
        } else {
            Ok(serde_json::json!({"found": false}))
        }
    }

    async fn test_combined_selector(&mut self) {
        let test_name = "combined_selector";
        match self.test_combined_selector_impl().await {
            Ok(details) => self.log_result(test_name, true, details),
            Err(e) => self.log_result(
                test_name,
                false,
                serde_json::json!({"error": e.to_string()}),
            ),
        }
    }

    async fn test_combined_selector_impl(&self) -> uiautomator::Result<serde_json::Value> {
        let element = self.device.find(
            Selector::new()
                .class_name("android.widget.TextView")
                .clickable(true),
        );
        let exists = element.exists(Some(Duration::from_secs(5))).await?;

        if exists {
            let info = element.info().await?;
            Ok(serde_json::json!({
                "found": true,
                "className": info.class_name,
                "clickable": info.clickable,
                "text": info.text,
            }))
        } else {
            Ok(serde_json::json!({"found": false}))
        }
    }

    async fn test_element_not_found(&mut self) {
        let test_name = "element_not_found";
        match self.test_element_not_found_impl().await {
            Ok(details) => self.log_result(test_name, true, details),
            Err(e) => self.log_result(
                test_name,
                false,
                serde_json::json!({"error": e.to_string()}),
            ),
        }
    }

    async fn test_element_not_found_impl(&self) -> uiautomator::Result<serde_json::Value> {
        let element = self
            .device
            .find(Selector::new().text("这个元素绝对不存在_12345"));
        let exists = element.exists(Some(Duration::from_secs(2))).await?;

        Ok(serde_json::json!({
            "found": exists,
            "expected": false,
            "test_passed": !exists,
        }))
    }

    async fn test_multiple_instances(&mut self) {
        let test_name = "multiple_instances";
        match self.test_multiple_instances_impl().await {
            Ok(details) => self.log_result(test_name, true, details),
            Err(e) => self.log_result(
                test_name,
                false,
                serde_json::json!({"error": e.to_string()}),
            ),
        }
    }

    async fn test_multiple_instances_impl(&self) -> uiautomator::Result<serde_json::Value> {
        // 获取第一个和第二个 TextView
        let first = self.device.find(
            Selector::new()
                .class_name("android.widget.TextView")
                .instance(0),
        );
        let second = self.device.find(
            Selector::new()
                .class_name("android.widget.TextView")
                .instance(1),
        );

        let first_exists = first.exists(Some(Duration::from_secs(2))).await?;
        let second_exists = second.exists(Some(Duration::from_secs(2))).await?;

        let first_text = if first_exists {
            first.get_text().await.ok()
        } else {
            None
        };

        let second_text = if second_exists {
            second.get_text().await.ok()
        } else {
            None
        };

        Ok(serde_json::json!({
            "first_element": first_text,
            "second_element": second_text,
            "both_found": first_exists && second_exists,
        }))
    }

    async fn test_root_element(&mut self) {
        let test_name = "root_element";
        match self.test_root_element_impl().await {
            Ok(details) => self.log_result(test_name, true, details),
            Err(e) => self.log_result(
                test_name,
                false,
                serde_json::json!({"error": e.to_string()}),
            ),
        }
    }

    async fn test_root_element_impl(&self) -> uiautomator::Result<serde_json::Value> {
        let element = self
            .device
            .find(Selector::new().class_name("android.widget.FrameLayout"));
        let exists = element.exists(Some(Duration::from_secs(5))).await?;

        if exists {
            let info = element.info().await?;
            let is_root = info.bounds.left == 0 && info.bounds.top == 0;

            Ok(serde_json::json!({
                "found": true,
                "className": info.class_name,
                "is_likely_root": is_root,
                "bounds": {
                    "left": info.bounds.left,
                    "top": info.bounds.top,
                    "right": info.bounds.right,
                    "bottom": info.bounds.bottom,
                }
            }))
        } else {
            Ok(serde_json::json!({"found": false}))
        }
    }

    async fn test_repeated_find(&mut self) {
        let test_name = "repeated_find";
        match self.test_repeated_find_impl().await {
            Ok(details) => self.log_result(test_name, true, details),
            Err(e) => self.log_result(
                test_name,
                false,
                serde_json::json!({"error": e.to_string()}),
            ),
        }
    }

    async fn test_repeated_find_impl(&self) -> uiautomator::Result<serde_json::Value> {
        let mut results = Vec::new();

        for _ in 0..5 {
            let element = self
                .device
                .find(Selector::new().class_name("android.widget.TextView"));
            let exists = element.exists(Some(Duration::from_secs(2))).await?;
            results.push(exists);
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        let all_same = results.iter().all(|&r| r == results[0]);

        Ok(serde_json::json!({
            "results": results,
            "consistent": all_same,
        }))
    }

    async fn test_info_method_behavior(&mut self) {
        let test_name = "info_method_behavior";
        match self.test_info_method_behavior_impl().await {
            Ok(details) => self.log_result(test_name, true, details),
            Err(e) => self.log_result(
                test_name,
                false,
                serde_json::json!({"error": e.to_string()}),
            ),
        }
    }

    async fn test_info_method_behavior_impl(&self) -> uiautomator::Result<serde_json::Value> {
        let element = self
            .device
            .find(Selector::new().class_name("android.widget.TextView"));

        if element.exists(Some(Duration::from_secs(5))).await? {
            let info = element.info().await?;

            Ok(serde_json::json!({
                "has_required_fields": true,
                "sample_info": {
                    "text": info.text,
                    "className": info.class_name,
                    "clickable": info.clickable,
                    "enabled": info.enabled,
                    "resourceId": info.resource_id,
                }
            }))
        } else {
            Err(uiautomator::Error::ElementNotFound {
                selector: "android.widget.TextView".to_string(),
            })
        }
    }

    async fn run_all_tests(&mut self) {
        println!("{}", "=".repeat(60));
        println!("Rust uiautomator 选择器可靠性测试");
        println!("{}", "=".repeat(60));
        println!();

        // 获取设备信息
        match self.device.info().await {
            Ok(info) => println!("设备信息: {:?}", info),
            Err(e) => println!("获取设备信息失败: {}", e),
        }
        println!();

        // 运行所有测试
        self.test_text_exact_match().await;
        self.test_text_contains().await;
        self.test_resource_id().await;
        self.test_class_name().await;
        self.test_combined_selector().await;
        self.test_element_not_found().await;
        self.test_multiple_instances().await;
        self.test_root_element().await;
        self.test_repeated_find().await;
        self.test_info_method_behavior().await;

        // 统计结果
        println!();
        println!("{}", "=".repeat(60));
        println!("测试总结");
        println!("{}", "=".repeat(60));
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.success).count();
        println!("总测试数: {}", total);
        println!("通过: {}", passed);
        println!("失败: {}", total - passed);
        println!("通过率: {:.1}%", (passed as f64 / total as f64) * 100.0);

        // 保存结果到文件
        let json = serde_json::to_string_pretty(&self.results).unwrap();
        std::fs::write("rust_test_results.json", json).unwrap();
        println!();
        println!("详细结果已保存到: rust_test_results.json");
    }
}

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut tester = SelectorReliabilityTest::new(None).await?;
    tester.run_all_tests().await;

    Ok(())
}
