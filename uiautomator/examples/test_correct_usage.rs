// 验证 waitForExists + objInfo 的正确使用方式
//
// cargo run --example test_correct_usage

use serde_json::json;
use uiautomator::Device;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    println!("=== 验证 objInfo 的正确使用方式 ===\n");

    let device = Device::connect(None).await?;
    let client = device.jsonrpc_client();

    // 测试 1: 错误的方式（直接调用 objInfo）
    println!("--- 测试 1: 错误的方式（直接调用 objInfo）---");
    let selector = json!({"text": "设置"});

    println!("直接调用 objInfo:");
    match client
        .call::<serde_json::Value>("objInfo", json!([selector.clone()]))
        .await
    {
        Ok(info) => {
            println!("  className: {}", info["className"]);
            println!("  text: {}", info["text"]);
            println!("  结果: 返回了 {} (可能是根元素)", info["className"]);
        }
        Err(e) => println!("  错误: {}", e),
    }
    println!();

    // 测试 2: 正确的方式（先 waitForExists，再 objInfo）
    println!("--- 测试 2: 正确的方式（先 waitForExists，再 objInfo）---");

    println!("步骤 1: 调用 waitForExists");
    match client
        .call::<bool>("waitForExists", json!([selector.clone(), 5000]))
        .await
    {
        Ok(exists) => println!("  结果: {}", exists),
        Err(e) => println!("  错误: {}", e),
    }

    println!("\n步骤 2: 调用 objInfo");
    match client
        .call::<serde_json::Value>("objInfo", json!([selector.clone()]))
        .await
    {
        Ok(info) => {
            println!("  className: {}", info["className"]);
            println!("  text: {}", info["text"]);
            println!("  resourceName: {}", info["resourceName"]);

            if info["className"] == "android.widget.TextView" && info["text"] == "设置" {
                println!("  ✓ 成功！返回了正确的元素");
            } else {
                println!("  ✗ 失败：仍然返回了错误的元素");
            }
        }
        Err(e) => println!("  错误: {}", e),
    }
    println!();

    // 测试 3: 测试不存在的元素
    println!("--- 测试 3: 不存在的元素 ---");
    let nonexistent = json!({"text": "这个元素绝对不存在_12345"});

    println!("步骤 1: 调用 waitForExists");
    match client
        .call::<bool>("waitForExists", json!([nonexistent.clone(), 2000]))
        .await
    {
        Ok(exists) => {
            println!("  结果: {}", exists);
            if !exists {
                println!("  ✓ 正确：元素不存在");
                println!("\n步骤 2: 不调用 objInfo（因为元素不存在）");
            }
        }
        Err(e) => println!("  错误: {}", e),
    }
    println!();

    // 测试 4: 测试多个不同的元素
    println!("--- 测试 4: 测试多个不同的元素 ---");

    let test_cases = vec![
        ("12:44", "时间"),
        ("雷电游戏中心", "应用名"),
        ("显示通知", "设置项"),
    ];

    for (text, desc) in test_cases {
        println!("\n测试: {} ('{}')", desc, text);
        let sel = json!({"text": text});

        // 先 waitForExists
        match client
            .call::<bool>("waitForExists", json!([sel.clone(), 3000]))
            .await
        {
            Ok(true) => {
                // 然后 objInfo
                match client
                    .call::<serde_json::Value>("objInfo", json!([sel]))
                    .await
                {
                    Ok(info) => {
                        let returned_text = info["text"].as_str().unwrap_or("");
                        let class_name = info["className"].as_str().unwrap_or("");

                        if returned_text == text {
                            println!("  ✓ 成功: className={}, text={}", class_name, returned_text);
                        } else {
                            println!(
                                "  ✗ 失败: 期望 text='{}', 实际 text='{}', className={}",
                                text, returned_text, class_name
                            );
                        }
                    }
                    Err(e) => println!("  ✗ objInfo 错误: {}", e),
                }
            }
            Ok(false) => println!("  - 元素不存在"),
            Err(e) => println!("  ✗ waitForExists 错误: {}", e),
        }
    }

    println!("\n=== 测试完成 ===");

    Ok(())
}
