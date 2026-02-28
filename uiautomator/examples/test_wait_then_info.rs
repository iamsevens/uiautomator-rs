// 测试 waitForExists + objInfo 组合
//
// 运行方式：
// cargo run --example test_wait_then_info

use serde_json::json;
use uiautomator::Device;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("=== 测试 waitForExists + objInfo 组合 ===\n");

    let device = Device::connect(None).await?;
    let client = device.jsonrpc_client();

    // 测试 1: TextView
    println!("--- 测试 1: TextView ---");
    let selector = json!({"className": "android.widget.TextView"});

    println!("1.1 调用 waitForExists:");
    match client
        .call::<bool>("waitForExists", json!([selector.clone(), 5000]))
        .await
    {
        Ok(exists) => println!("  结果: {}", exists),
        Err(e) => println!("  错误: {}", e),
    }

    println!("\n1.2 立即调用 objInfo:");
    match client
        .call::<serde_json::Value>("objInfo", json!([selector.clone()]))
        .await
    {
        Ok(info) => {
            println!("  className: {}", info["className"]);
            println!("  text: {}", info["text"]);
            println!("  clickable: {}", info["clickable"]);
        }
        Err(e) => println!("  错误: {}", e),
    }
    println!();

    // 测试 2: 尝试不同的选择器
    println!("--- 测试 2: 尝试更具体的选择器 ---");

    // 2.1 只用 clickable
    println!("2.1 clickable=true:");
    let selector2 = json!({"clickable": true});
    match client
        .call::<bool>("waitForExists", json!([selector2.clone(), 5000]))
        .await
    {
        Ok(exists) => {
            println!("  waitForExists: {}", exists);
            if exists {
                match client
                    .call::<serde_json::Value>("objInfo", json!([selector2]))
                    .await
                {
                    Ok(info) => println!("  objInfo className: {}", info["className"]),
                    Err(e) => println!("  objInfo 错误: {}", e),
                }
            }
        }
        Err(e) => println!("  错误: {}", e),
    }
    println!();

    // 2.2 enabled=true
    println!("2.2 enabled=true:");
    let selector3 = json!({"enabled": true});
    match client
        .call::<bool>("waitForExists", json!([selector3.clone(), 5000]))
        .await
    {
        Ok(exists) => {
            println!("  waitForExists: {}", exists);
            if exists {
                match client
                    .call::<serde_json::Value>("objInfo", json!([selector3]))
                    .await
                {
                    Ok(info) => println!("  objInfo className: {}", info["className"]),
                    Err(e) => println!("  objInfo 错误: {}", e),
                }
            }
        }
        Err(e) => println!("  错误: {}", e),
    }
    println!();

    // 测试 3: 测试 dumpWindowHierarchy 的不同参数
    println!("--- 测试 3: dumpWindowHierarchy 不同参数 ---");

    println!("3.1 参数: [true]");
    match client
        .call::<serde_json::Value>("dumpWindowHierarchy", json!([true]))
        .await
    {
        Ok(result) => {
            let s = result.to_string();
            println!("  成功，长度: {} 字符", s.len());
            if s.len() > 500 {
                println!("  前 500 字符:\n{}", &s[..500]);
            } else {
                println!("  内容:\n{}", s);
            }
        }
        Err(e) => println!("  错误: {}", e),
    }
    println!();

    println!("3.2 参数: [false]");
    match client
        .call::<serde_json::Value>("dumpWindowHierarchy", json!([false]))
        .await
    {
        Ok(result) => {
            let s = result.to_string();
            println!("  成功，长度: {} 字符", s.len());
            if s.len() > 500 {
                println!("  前 500 字符:\n{}", &s[..500]);
            } else {
                println!("  内容:\n{}", s);
            }
        }
        Err(e) => println!("  错误: {}", e),
    }
    println!();

    println!("3.3 参数: [true, true]");
    match client
        .call::<serde_json::Value>("dumpWindowHierarchy", json!([true, true]))
        .await
    {
        Ok(result) => {
            let s = result.to_string();
            println!("  成功，长度: {} 字符", s.len());
            if s.len() > 500 {
                println!("  前 500 字符:\n{}", &s[..500]);
            } else {
                println!("  内容:\n{}", s);
            }
        }
        Err(e) => println!("  错误: {}", e),
    }
    println!();

    // 测试 4: 查看是否有其他可用方法
    println!("--- 测试 4: 尝试其他可能的方法 ---");

    let methods = vec![
        "click",
        "getText",
        "info",
        "exists",
        "count",
        "waitUntilGone",
    ];

    for method in methods {
        print!("  {}: ", method);
        match client
            .call::<serde_json::Value>(method, json!([selector.clone()]))
            .await
        {
            Ok(_) => println!("✓ 存在"),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("-32601") {
                    println!("✗ 方法不存在");
                } else {
                    println!("✗ {}", err_str);
                }
            }
        }
    }

    println!("\n=== 测试完成 ===");

    Ok(())
}
