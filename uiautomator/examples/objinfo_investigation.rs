// objInfo 深度调查工具
//
// 运行方式：
// cargo run --example objinfo_investigation

use serde_json::json;
use uiautomator::Device;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("=== objInfo 方法深度调查 ===\n");

    let device = Device::connect(None).await?;
    println!("✓ 已连接到设备: {}\n", device.serial());

    let client = device.jsonrpc_client();

    // 实验 1: 查看所有可用的 JSON-RPC 方法
    println!("--- 实验 1: 测试不同的 JSON-RPC 方法 ---\n");

    // 1.1 ping
    println!("1.1 测试 ping:");
    match client.call::<serde_json::Value>("ping", json!({})).await {
        Ok(result) => println!("  ✓ ping 成功: {}", result),
        Err(e) => println!("  ✗ ping 失败: {}", e),
    }
    println!();

    // 1.2 dumpHierarchy
    println!("1.2 测试 dumpHierarchy:");
    match client
        .call::<serde_json::Value>("dumpHierarchy", json!({}))
        .await
    {
        Ok(result) => {
            let result_str = result.to_string();
            if result_str.len() > 200 {
                println!("  ✓ dumpHierarchy 成功 (返回 {} 字符)", result_str.len());
                println!("  前 200 字符: {}", &result_str[..200]);
            } else {
                println!("  ✓ dumpHierarchy 成功: {}", result);
            }
        }
        Err(e) => println!("  ✗ dumpHierarchy 失败: {}", e),
    }
    println!();

    // 1.3 dumpWindowHierarchy
    println!("1.3 测试 dumpWindowHierarchy:");
    match client
        .call::<serde_json::Value>("dumpWindowHierarchy", json!({}))
        .await
    {
        Ok(result) => {
            let result_str = result.to_string();
            if result_str.len() > 200 {
                println!(
                    "  ✓ dumpWindowHierarchy 成功 (返回 {} 字符)",
                    result_str.len()
                );
                println!("  前 200 字符: {}", &result_str[..200]);
            } else {
                println!("  ✓ dumpWindowHierarchy 成功: {}", result);
            }
        }
        Err(e) => println!("  ✗ dumpWindowHierarchy 失败: {}", e),
    }
    println!();

    // 实验 2: 测试 objInfo 的不同参数格式
    println!("--- 实验 2: 测试 objInfo 的不同参数格式 ---\n");

    // 2.1 空参数
    println!("2.1 objInfo 空参数:");
    match client.call::<serde_json::Value>("objInfo", json!({})).await {
        Ok(result) => println!(
            "  ✓ 成功: {}",
            serde_json::to_string_pretty(&result).unwrap()
        ),
        Err(e) => println!("  ✗ 失败: {}", e),
    }
    println!();

    // 2.2 数组包装的参数
    println!("2.2 objInfo 数组包装参数:");
    match client
        .call::<serde_json::Value>("objInfo", json!([{"className": "android.widget.TextView"}]))
        .await
    {
        Ok(result) => println!(
            "  ✓ 成功: {}",
            serde_json::to_string_pretty(&result).unwrap()
        ),
        Err(e) => println!("  ✗ 失败: {}", e),
    }
    println!();

    // 2.3 直接对象参数
    println!("2.3 objInfo 直接对象参数:");
    match client
        .call::<serde_json::Value>("objInfo", json!({"className": "android.widget.TextView"}))
        .await
    {
        Ok(result) => println!(
            "  ✓ 成功: {}",
            serde_json::to_string_pretty(&result).unwrap()
        ),
        Err(e) => println!("  ✗ 失败: {}", e),
    }
    println!();

    // 实验 3: 测试其他可能的查找方法
    println!("--- 实验 3: 测试其他可能的查找方法 ---\n");

    // 3.1 waitForExists
    println!("3.1 测试 waitForExists:");
    match client
        .call::<serde_json::Value>(
            "waitForExists",
            json!([{"className": "android.widget.TextView"}, 5000]),
        )
        .await
    {
        Ok(result) => println!("  ✓ 成功: {}", result),
        Err(e) => println!("  ✗ 失败: {}", e),
    }
    println!();

    // 3.2 exist
    println!("3.2 测试 exist:");
    match client
        .call::<serde_json::Value>("exist", json!([{"className": "android.widget.TextView"}]))
        .await
    {
        Ok(result) => println!("  ✓ 成功: {}", result),
        Err(e) => println!("  ✗ 失败: {}", e),
    }
    println!();

    // 3.3 findObject
    println!("3.3 测试 findObject:");
    match client
        .call::<serde_json::Value>(
            "findObject",
            json!([{"className": "android.widget.TextView"}]),
        )
        .await
    {
        Ok(result) => println!("  ✓ 成功: {}", result),
        Err(e) => println!("  ✗ 失败: {}", e),
    }
    println!();

    // 实验 4: 测试 clickable 元素
    println!("--- 实验 4: 测试查找 clickable 元素 ---\n");

    println!("4.1 查找任意 clickable 元素:");
    match client
        .call::<serde_json::Value>("objInfo", json!([{"clickable": true}]))
        .await
    {
        Ok(result) => println!(
            "  ✓ 成功: {}",
            serde_json::to_string_pretty(&result).unwrap()
        ),
        Err(e) => println!("  ✗ 失败: {}", e),
    }
    println!();

    // 实验 5: 测试不同的 instance 值
    println!("--- 实验 5: 测试 instance 参数 ---\n");

    for i in 0..3 {
        println!("5.{} 查找 instance={}:", i + 1, i);
        match client
            .call::<serde_json::Value>("objInfo", json!([{"clickable": true, "instance": i}]))
            .await
        {
            Ok(result) => {
                if let Some(class_name) = result.get("className") {
                    println!("  ✓ 找到: className={}", class_name);
                } else {
                    println!("  ✓ 成功但无 className");
                }
            }
            Err(e) => println!("  ✗ 失败: {}", e),
        }
    }
    println!();

    // 实验 6: 获取当前应用的包名并查找
    println!("--- 实验 6: 使用当前应用包名查找 ---\n");

    let info = device.info().await?;
    println!("当前应用: {}", info.current_package_name);

    println!("6.1 查找当前应用的元素:");
    match client
        .call::<serde_json::Value>(
            "objInfo",
            json!([{"packageName": info.current_package_name}]),
        )
        .await
    {
        Ok(result) => println!(
            "  ✓ 成功: {}",
            serde_json::to_string_pretty(&result).unwrap()
        ),
        Err(e) => println!("  ✗ 失败: {}", e),
    }
    println!();

    println!("=== 调查完成 ===");

    Ok(())
}
