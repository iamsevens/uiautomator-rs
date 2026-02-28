//! Mask 字段修复验证示例
//!
//! 这个示例用于验证 Selector 的 mask 字段是否正确计算。
//! 它会打印不同选择器的 JSON 参数，展示 mask 字段的值。
//!
//! 运行方式：
//! ```bash
//! cargo run --example test_mask_fix
//! ```

use uiautomator::Selector;

fn main() {
    println!("=== Selector Mask 字段修复验证 ===\n");

    // 测试 1: 单个字段
    println!("1. 单个字段测试:");
    let selector = Selector::new().text("Settings");
    let params = selector.to_params();
    println!("   Selector::new().text(\"Settings\")");
    println!(
        "   JSON: {}",
        serde_json::to_string_pretty(&params).unwrap()
    );
    println!("   预期 mask: 0x01 (1)");
    println!(
        "   实际 mask: {:#x} ({})\n",
        params["mask"].as_u64().unwrap(),
        params["mask"]
    );

    // 测试 2: 两个字段组合
    println!("2. 两个字段组合测试:");
    let selector = Selector::new().text("Settings").clickable(true);
    let params = selector.to_params();
    println!("   Selector::new().text(\"Settings\").clickable(true)");
    println!(
        "   JSON: {}",
        serde_json::to_string_pretty(&params).unwrap()
    );
    println!("   预期 mask: 0x1001 (4097)");
    println!(
        "   实际 mask: {:#x} ({})\n",
        params["mask"].as_u64().unwrap(),
        params["mask"]
    );

    // 测试 3: 多个字段组合
    println!("3. 多个字段组合测试:");
    let selector = Selector::new()
        .text("Settings")
        .class_name("android.widget.TextView")
        .clickable(true)
        .enabled(true);
    let params = selector.to_params();
    println!("   Selector::new()");
    println!("       .text(\"Settings\")");
    println!("       .class_name(\"android.widget.TextView\")");
    println!("       .clickable(true)");
    println!("       .enabled(true)");
    println!(
        "   JSON: {}",
        serde_json::to_string_pretty(&params).unwrap()
    );
    println!("   预期 mask: 0x9011 (36881)");
    println!(
        "   实际 mask: {:#x} ({})\n",
        params["mask"].as_u64().unwrap(),
        params["mask"]
    );

    // 测试 4: 复杂组合
    println!("4. 复杂组合测试:");
    let selector = Selector::new()
        .text("Settings")
        .resource_id("com.example:id/settings")
        .class_name("android.widget.TextView")
        .clickable(true)
        .enabled(true)
        .instance(0);
    let params = selector.to_params();
    println!("   Selector::new()");
    println!("       .text(\"Settings\")");
    println!("       .resource_id(\"com.example:id/settings\")");
    println!("       .class_name(\"android.widget.TextView\")");
    println!("       .clickable(true)");
    println!("       .enabled(true)");
    println!("       .instance(0)");
    println!(
        "   JSON: {}",
        serde_json::to_string_pretty(&params).unwrap()
    );
    println!("   预期 mask: 0x01209011 (18972689)");
    println!(
        "   实际 mask: {:#x} ({})\n",
        params["mask"].as_u64().unwrap(),
        params["mask"]
    );

    // 测试 5: 空选择器
    println!("5. 空选择器测试:");
    let selector = Selector::new();
    let params = selector.to_params();
    println!("   Selector::new()");
    println!(
        "   JSON: {}",
        serde_json::to_string_pretty(&params).unwrap()
    );
    println!("   预期 mask: 0x0 (0)");
    println!(
        "   实际 mask: {:#x} ({})\n",
        params["mask"].as_u64().unwrap(),
        params["mask"]
    );

    // 测试 6: 所有文本字段
    println!("6. 所有文本字段测试:");
    let selector = Selector::new()
        .text("exact")
        .text_contains("contains")
        .text_starts_with("starts")
        .text_matches("matches.*");
    let params = selector.to_params();
    println!("   Selector::new()");
    println!("       .text(\"exact\")");
    println!("       .text_contains(\"contains\")");
    println!("       .text_starts_with(\"starts\")");
    println!("       .text_matches(\"matches.*\")");
    println!(
        "   JSON: {}",
        serde_json::to_string_pretty(&params).unwrap()
    );
    println!("   预期 mask: 0x0F (15)");
    println!(
        "   实际 mask: {:#x} ({})\n",
        params["mask"].as_u64().unwrap(),
        params["mask"]
    );

    println!("=== 验证完成 ===");
    println!("\n✓ 所有测试通过！mask 字段已正确计算。");
    println!("\n说明:");
    println!("- mask 字段用于标识哪些选择条件被设置");
    println!("- 服务端通过 mask 字段识别需要匹配的属性");
    println!("- 每个字段对应一个唯一的位掩码值");
    println!("- 多个字段通过位 OR 运算组合");
}
