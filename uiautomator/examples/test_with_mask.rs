// 测试带 mask 字段的选择器
//
// cargo run --example test_with_mask

use serde_json::json;
use uiautomator::Device;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    println!("=== 测试带 mask 字段的选择器 ===\n");

    let device = Device::connect(None).await?;
    let client = device.jsonrpc_client();

    // 测试 1: 不带 mask（我们当前的方式）
    println!("--- 测试 1: 不带 mask ---");
    let selector_no_mask = json!({"text": "设置"});
    println!("选择器: {}", selector_no_mask);

    match client
        .call::<bool>("waitForExists", json!([selector_no_mask.clone(), 3000]))
        .await
    {
        Ok(exists) => println!("  waitForExists: {}", exists),
        Err(e) => println!("  waitForExists 错误: {}", e),
    }

    match client
        .call::<serde_json::Value>("objInfo", json!([selector_no_mask]))
        .await
    {
        Ok(info) => println!(
            "  objInfo: className={}, text={}",
            info["className"], info["text"]
        ),
        Err(e) => println!("  objInfo 错误: {}", e),
    }
    println!();

    // 测试 2: 带 mask（Python 的方式）
    println!("--- 测试 2: 带 mask (MASK_TEXT = 0x01) ---");
    let selector_with_mask = json!({
        "mask": 1,
        "childOrSibling": [],
        "childOrSiblingSelector": [],
        "text": "设置"
    });
    println!("选择器: {}", selector_with_mask);

    match client
        .call::<bool>("waitForExists", json!([selector_with_mask.clone(), 3000]))
        .await
    {
        Ok(exists) => println!("  waitForExists: {}", exists),
        Err(e) => println!("  waitForExists 错误: {}", e),
    }

    match client
        .call::<serde_json::Value>("objInfo", json!([selector_with_mask]))
        .await
    {
        Ok(info) => {
            println!(
                "  objInfo: className={}, text={}",
                info["className"], info["text"]
            );

            if info["text"] == "设置" {
                println!("  ✓ 成功！返回了正确的元素");
            } else {
                println!("  ✗ 失败：仍然返回错误的元素");
            }
        }
        Err(e) => println!("  objInfo 错误: {}", e),
    }
    println!();

    // 测试 3: className 带 mask
    println!("--- 测试 3: className 带 mask (MASK_CLASSNAME = 0x10) ---");
    let selector_classname = json!({
        "mask": 16,
        "childOrSibling": [],
        "childOrSiblingSelector": [],
        "className": "android.widget.TextView"
    });

    match client
        .call::<bool>("waitForExists", json!([selector_classname.clone(), 3000]))
        .await
    {
        Ok(exists) => println!("  waitForExists: {}", exists),
        Err(e) => println!("  waitForExists 错误: {}", e),
    }

    match client
        .call::<serde_json::Value>("objInfo", json!([selector_classname]))
        .await
    {
        Ok(info) => {
            println!(
                "  objInfo: className={}, text={}",
                info["className"], info["text"]
            );

            if info["className"] == "android.widget.TextView" {
                println!("  ✓ 成功！返回了 TextView");
            } else {
                println!("  ✗ 失败：返回了 {}", info["className"]);
            }
        }
        Err(e) => println!("  objInfo 错误: {}", e),
    }
    println!();

    // 测试 4: resourceId 带 mask
    println!("--- 测试 4: resourceId 带 mask (MASK_RESOURCEID = 0x200000) ---");
    let selector_resourceid = json!({
        "mask": 2097152,  // 0x200000
        "childOrSibling": [],
        "childOrSiblingSelector": [],
        "resourceId": "com.android.systemui:id/clock"
    });

    match client
        .call::<bool>("waitForExists", json!([selector_resourceid.clone(), 3000]))
        .await
    {
        Ok(exists) => println!("  waitForExists: {}", exists),
        Err(e) => println!("  waitForExists 错误: {}", e),
    }

    match client
        .call::<serde_json::Value>("objInfo", json!([selector_resourceid]))
        .await
    {
        Ok(info) => {
            println!(
                "  objInfo: className={}, text={}, resourceName={}",
                info["className"], info["text"], info["resourceName"]
            );

            if info["resourceName"] == "com.android.systemui:id/clock" {
                println!("  ✓ 成功！返回了正确的元素");
            } else {
                println!("  ✗ 失败");
            }
        }
        Err(e) => println!("  objInfo 错误: {}", e),
    }
    println!();

    // 测试 5: 不存在的元素带 mask
    println!("--- 测试 5: 不存在的元素带 mask ---");
    let selector_nonexistent = json!({
        "mask": 1,
        "childOrSibling": [],
        "childOrSiblingSelector": [],
        "text": "这个元素绝对不存在_12345"
    });

    match client
        .call::<bool>("waitForExists", json!([selector_nonexistent, 2000]))
        .await
    {
        Ok(exists) => {
            if exists {
                println!("  waitForExists: true ✗ (应该是 false)");
            } else {
                println!("  waitForExists: false ✓ (正确)");
            }
        }
        Err(e) => println!("  waitForExists 错误: {}", e),
    }

    println!("\n=== 测试完成 ===");

    Ok(())
}
