// 使用实际存在的文本测试
//
// cargo run --example test_real_text

use serde_json::json;
use uiautomator::Device;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    println!("=== 使用实际存在的文本测试 ===\n");

    let device = Device::connect(None).await?;
    let client = device.jsonrpc_client();

    // 从 UI 树中看到的实际文本
    let test_cases = vec![
        ("12:44", "时间文本"),
        ("设置", "设置标题"),
        ("雷电游戏中心", "应用名称"),
        ("显示通知", "设置项"),
    ];

    for (text, description) in test_cases {
        println!("--- 测试: {} ('{}') ---", description, text);

        let selector = json!({"text": text});

        // 1. waitForExists
        print!("  waitForExists: ");
        match client
            .call::<bool>("waitForExists", json!([selector.clone(), 3000]))
            .await
        {
            Ok(exists) => println!("{}", if exists { "✓ true" } else { "✗ false" }),
            Err(e) => println!("✗ 错误: {}", e),
        }

        // 2. objInfo
        print!("  objInfo: ");
        match client
            .call::<serde_json::Value>("objInfo", json!([selector.clone()]))
            .await
        {
            Ok(info) => {
                let class_name = info["className"].as_str().unwrap_or("");
                let returned_text = info["text"].as_str().unwrap_or("");

                if class_name == "android.widget.TextView" && returned_text == text {
                    println!(
                        "✓ 正确返回 (className={}, text={})",
                        class_name, returned_text
                    );
                } else {
                    println!(
                        "✗ 返回不匹配 (className={}, text={})",
                        class_name, returned_text
                    );
                }
            }
            Err(e) => println!("✗ 错误: {}", e),
        }

        // 3. getText
        print!("  getText: ");
        match client.call::<String>("getText", json!([selector])).await {
            Ok(text_result) => println!("✓ '{}'", text_result),
            Err(e) => println!("✗ 错误: {}", e),
        }

        println!();
    }

    // 测试使用 resource-id
    println!("--- 测试: 使用 resource-id ---");
    let selector = json!({"resourceId": "com.android.systemui:id/clock"});

    print!("  waitForExists: ");
    match client
        .call::<bool>("waitForExists", json!([selector.clone(), 3000]))
        .await
    {
        Ok(exists) => println!("{}", if exists { "✓ true" } else { "✗ false" }),
        Err(e) => println!("✗ 错误: {}", e),
    }

    print!("  objInfo: ");
    match client
        .call::<serde_json::Value>("objInfo", json!([selector]))
        .await
    {
        Ok(info) => {
            println!(
                "✓ className={}, text={}",
                info["className"].as_str().unwrap_or(""),
                info["text"].as_str().unwrap_or("")
            );
        }
        Err(e) => println!("✗ 错误: {}", e),
    }

    println!("\n=== 测试完成 ===");

    Ok(())
}
