// 分析 UI 树来理解 objInfo 的问题
//
// 运行方式：
// cargo run --example analyze_ui_tree

use serde_json::json;
use uiautomator::Device;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    println!("=== 分析 UI 树 ===\n");

    let device = Device::connect(None).await?;
    let client = device.jsonrpc_client();

    // 获取 UI 树
    println!("获取 UI 树...");
    let xml: String = client.call("dumpWindowHierarchy", json!([false])).await?;

    println!("UI 树大小: {} 字符\n", xml.len());

    // 分析 TextView 元素
    println!("--- 分析 TextView 元素 ---");
    let textview_count = xml.matches("class=\"android.widget.TextView\"").count();
    println!("找到 {} 个 TextView 元素\n", textview_count);

    // 提取前几个 TextView 的信息
    println!("前 5 个 TextView 示例:");
    let mut count = 0;
    for line in xml.lines() {
        if line.contains("class=\"android.widget.TextView\"") && count < 5 {
            count += 1;
            println!("\n{}. {}", count, line.trim());

            // 提取关键属性
            if let Some(text_start) = line.find("text=\"") {
                if let Some(text_end) = line[text_start + 6..].find("\"") {
                    let text = &line[text_start + 6..text_start + 6 + text_end];
                    println!("   文本: {}", text);
                }
            }

            if let Some(id_start) = line.find("resource-id=\"") {
                if let Some(id_end) = line[id_start + 13..].find("\"") {
                    let id = &line[id_start + 13..id_start + 13 + id_end];
                    if !id.is_empty() {
                        println!("   资源ID: {}", id);
                    }
                }
            }
        }
    }

    println!("\n\n--- 测试：使用 XML 中的实际文本查找 ---");

    // 从 XML 中提取一个实际的文本
    if let Some(text_start) = xml.find("text=\"") {
        if let Some(text_end) = xml[text_start + 6..].find("\"") {
            let actual_text = &xml[text_start + 6..text_start + 6 + text_end];
            if !actual_text.is_empty() && actual_text.len() < 50 {
                println!("找到实际文本: '{}'", actual_text);

                // 测试 waitForExists
                let selector = json!({"text": actual_text});
                println!("\n测试 waitForExists:");
                match client
                    .call::<bool>("waitForExists", json!([selector.clone(), 5000]))
                    .await
                {
                    Ok(exists) => println!("  结果: {}", exists),
                    Err(e) => println!("  错误: {}", e),
                }

                // 测试 objInfo
                println!("\n测试 objInfo:");
                match client
                    .call::<serde_json::Value>("objInfo", json!([selector]))
                    .await
                {
                    Ok(info) => {
                        println!("  className: {}", info["className"]);
                        println!("  text: {}", info["text"]);
                        println!("  resourceName: {}", info["resourceName"]);
                    }
                    Err(e) => println!("  错误: {}", e),
                }
            }
        }
    }

    println!("\n\n--- 分析根元素 ---");
    if let Some(first_node) = xml.find("<node ") {
        if let Some(node_end) = xml[first_node..].find(">") {
            let first_node_str = &xml[first_node..first_node + node_end + 1];
            println!("第一个 node 元素:\n{}", first_node_str);
        }
    }

    println!("\n=== 分析完成 ===");

    Ok(())
}
