use super::*;

// 测试需求 3.1: 通过 text 属性定位元素
#[test]
fn test_selector_text() {
    let selector = Selector::new().text("Settings");

    assert_eq!(selector.text, Some("Settings".to_string()));

    let params = selector.to_params();
    assert_eq!(params["text"], "Settings");
    // 验证 mask 字段存在
    assert!(params.get("mask").is_some());
    assert_eq!(params["mask"], 0x01);
}

// 测试需求 3.2: 通过 resourceId 属性定位元素
#[test]
fn test_selector_resource_id() {
    let selector = Selector::new().resource_id("com.example:id/button");

    assert_eq!(
        selector.resource_id,
        Some("com.example:id/button".to_string())
    );

    let params = selector.to_params();
    assert_eq!(params["resourceId"], "com.example:id/button");
    // 验证 mask 字段存在
    assert!(params.get("mask").is_some());
    assert_eq!(params["mask"], 0x200000);
}

// 测试需求 3.3: 通过 className 属性定位元素
#[test]
fn test_selector_class_name() {
    let selector = Selector::new().class_name("android.widget.TextView");

    assert_eq!(
        selector.class_name,
        Some("android.widget.TextView".to_string())
    );

    let params = selector.to_params();
    assert_eq!(params["className"], "android.widget.TextView");
    // 验证 mask 字段存在
    assert!(params.get("mask").is_some());
    assert_eq!(params["mask"], 0x10);
}

// 测试需求 3.4: 通过 description 属性定位元素
#[test]
fn test_selector_description() {
    let selector = Selector::new().description("Submit button");

    assert_eq!(selector.description, Some("Submit button".to_string()));

    let params = selector.to_params();
    assert_eq!(params["description"], "Submit button");
    // 验证 mask 字段存在
    assert!(params.get("mask").is_some());
    assert_eq!(params["mask"], 0x40);
}

// 测试需求 3.5: 组合多个属性定位元素
#[test]
fn test_selector_multiple_conditions() {
    let selector = Selector::new()
        .text("Settings")
        .class_name("android.widget.TextView")
        .clickable(true)
        .enabled(true);

    assert_eq!(selector.text, Some("Settings".to_string()));
    assert_eq!(
        selector.class_name,
        Some("android.widget.TextView".to_string())
    );
    assert_eq!(selector.clickable, Some(true));
    assert_eq!(selector.enabled, Some(true));

    let params = selector.to_params();
    assert_eq!(params["text"], "Settings");
    assert_eq!(params["className"], "android.widget.TextView");
    assert_eq!(params["clickable"], true);
    assert_eq!(params["enabled"], true);
    // 验证 mask 字段存在并正确计算
    assert!(params.get("mask").is_some());
    // mask = 0x01 (text) | 0x10 (className) | 0x1000 (clickable) | 0x8000 (enabled)
    assert_eq!(params["mask"], 0x01 | 0x10 | 0x1000 | 0x8000);
}

// 测试需求 3.7: 通过 instance 参数选择特定实例
#[test]
fn test_selector_instance() {
    let selector = Selector::new().text("Item").instance(2);

    assert_eq!(selector.text, Some("Item".to_string()));
    assert_eq!(selector.instance, Some(2));

    let params = selector.to_params();
    assert_eq!(params["text"], "Item");
    assert_eq!(params["instance"], 2);
    // 验证 mask 字段存在
    assert!(params.get("mask").is_some());
    assert_eq!(params["mask"], 0x01 | 0x01000000);
}

// 测试文本包含条件
#[test]
fn test_selector_text_contains() {
    let selector = Selector::new().text_contains("Set");

    let params = selector.to_params();
    assert_eq!(params["textContains"], "Set");
}

// 测试文本开头匹配
#[test]
fn test_selector_text_starts_with() {
    let selector = Selector::new().text_starts_with("Set");

    let params = selector.to_params();
    assert_eq!(params["textStartsWith"], "Set");
}

// 测试文本正则匹配
#[test]
fn test_selector_text_matches() {
    let selector = Selector::new().text_matches("Set.*");

    let params = selector.to_params();
    assert_eq!(params["textMatches"], "Set.*");
}

// 测试布尔属性
#[test]
fn test_selector_boolean_properties() {
    let selector = Selector::new()
        .clickable(true)
        .enabled(false)
        .focusable(true)
        .scrollable(false);

    let params = selector.to_params();
    assert_eq!(params["clickable"], true);
    assert_eq!(params["enabled"], false);
    assert_eq!(params["focusable"], true);
    assert_eq!(params["scrollable"], false);
}

// 测试空选择器
#[test]
fn test_selector_empty() {
    let selector = Selector::new();

    let params = selector.to_params();
    // 空选择器应该只有 mask 字段，值为 0
    assert_eq!(params.as_object().unwrap().len(), 1);
    assert_eq!(params["mask"], 0);
}

// === Mask 计算测试 ===

// 测试单字段 mask 值
#[test]
fn test_mask_single_field_text() {
    let selector = Selector::new().text("test");
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x01);
}

#[test]
fn test_mask_single_field_text_contains() {
    let selector = Selector::new().text_contains("test");
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x02);
}

#[test]
fn test_mask_single_field_text_matches() {
    let selector = Selector::new().text_matches("test.*");
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x04);
}

#[test]
fn test_mask_single_field_text_starts_with() {
    let selector = Selector::new().text_starts_with("test");
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x08);
}

#[test]
fn test_mask_single_field_class_name() {
    let selector = Selector::new().class_name("android.widget.TextView");
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x10);
}

#[test]
fn test_mask_single_field_description() {
    let selector = Selector::new().description("desc");
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x40);
}

#[test]
fn test_mask_single_field_description_contains() {
    let selector = Selector::new().description_contains("desc");
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x80);
}

#[test]
fn test_mask_single_field_clickable() {
    let selector = Selector::new().clickable(true);
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x1000);
}

#[test]
fn test_mask_single_field_scrollable() {
    let selector = Selector::new().scrollable(true);
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x4000);
}

#[test]
fn test_mask_single_field_enabled() {
    let selector = Selector::new().enabled(true);
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x8000);
}

#[test]
fn test_mask_single_field_focusable() {
    let selector = Selector::new().focusable(true);
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x010000);
}

#[test]
fn test_mask_single_field_package_name() {
    let selector = Selector::new().package_name("com.example");
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x080000);
}

#[test]
fn test_mask_single_field_resource_id() {
    let selector = Selector::new().resource_id("com.example:id/button");
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x200000);
}

#[test]
fn test_mask_single_field_instance() {
    let selector = Selector::new().instance(0);
    let params = selector.to_params();
    assert_eq!(params["mask"], 0x01000000);
}

// 测试多字段组合 mask 值
#[test]
fn test_mask_combination_text_clickable() {
    let selector = Selector::new().text("test").clickable(true);
    let params = selector.to_params();
    // mask = 0x01 (text) | 0x1000 (clickable) = 0x1001
    assert_eq!(params["mask"], 0x1001);
}

#[test]
fn test_mask_combination_three_fields() {
    let selector = Selector::new()
        .text("test")
        .class_name("android.widget.TextView")
        .enabled(true);
    let params = selector.to_params();
    // mask = 0x01 (text) | 0x10 (className) | 0x8000 (enabled) = 0x8011
    assert_eq!(params["mask"], 0x8011);
}

#[test]
fn test_mask_combination_all_text_fields() {
    let selector = Selector::new()
        .text("test")
        .text_contains("contains")
        .text_starts_with("starts")
        .text_matches("matches");
    let params = selector.to_params();
    // mask = 0x01 | 0x02 | 0x08 | 0x04 = 0x0F
    assert_eq!(params["mask"], 0x0F);
}

#[test]
fn test_mask_combination_all_boolean_fields() {
    let selector = Selector::new()
        .clickable(true)
        .scrollable(true)
        .enabled(true)
        .focusable(true);
    let params = selector.to_params();
    // mask = 0x1000 | 0x4000 | 0x8000 | 0x010000 = 0x01D000
    assert_eq!(params["mask"], 0x01D000);
}

#[test]
fn test_mask_combination_complex() {
    let selector = Selector::new()
        .text("Settings")
        .resource_id("com.example:id/settings")
        .class_name("android.widget.TextView")
        .clickable(true)
        .enabled(true)
        .instance(0);
    let params = selector.to_params();
    // mask = 0x01 | 0x200000 | 0x10 | 0x1000 | 0x8000 | 0x01000000
    assert_eq!(params["mask"], 0x01209011);
}

// 测试链式调用
#[test]
fn test_selector_chaining() {
    let selector = Selector::new()
        .text("Settings")
        .resource_id("com.example:id/settings")
        .class_name("android.widget.TextView")
        .description("Settings button")
        .clickable(true)
        .enabled(true)
        .instance(0);

    let params = selector.to_params();
    assert_eq!(params["text"], "Settings");
    assert_eq!(params["resourceId"], "com.example:id/settings");
    assert_eq!(params["className"], "android.widget.TextView");
    assert_eq!(params["description"], "Settings button");
    assert_eq!(params["clickable"], true);
    assert_eq!(params["enabled"], true);
    assert_eq!(params["instance"], 0);
}

// 测试 Default trait
#[test]
fn test_selector_default() {
    let selector1 = Selector::new();
    let selector2 = Selector::default();

    assert_eq!(selector1, selector2);
}

// 测试 Clone trait
#[test]
fn test_selector_clone() {
    let selector1 = Selector::new().text("Settings").clickable(true);

    let selector2 = selector1.clone();

    assert_eq!(selector1, selector2);
}

// 测试所有字符串字段
#[test]
fn test_selector_all_string_fields() {
    let selector = Selector::new()
        .text("text")
        .text_contains("contains")
        .text_starts_with("starts")
        .text_matches("matches")
        .resource_id("id")
        .class_name("class")
        .description("desc")
        .description_contains("desc_contains")
        .package_name("package");

    let params = selector.to_params();
    assert_eq!(params["text"], "text");
    assert_eq!(params["textContains"], "contains");
    assert_eq!(params["textStartsWith"], "starts");
    assert_eq!(params["textMatches"], "matches");
    assert_eq!(params["resourceId"], "id");
    assert_eq!(params["className"], "class");
    assert_eq!(params["description"], "desc");
    assert_eq!(params["descriptionContains"], "desc_contains");
    assert_eq!(params["packageName"], "package");
}

// === 扩展布尔字段测试 ===

#[test]
fn test_mask_single_field_checkable() {
    let selector = Selector::new().checkable(true);
    let params = selector.to_params();
    assert_eq!(params["checkable"], true);
    assert_eq!(params["mask"], 0x0400);
}

#[test]
fn test_mask_single_field_checked() {
    let selector = Selector::new().checked(true);
    let params = selector.to_params();
    assert_eq!(params["checked"], true);
    assert_eq!(params["mask"], 0x0800);
}

#[test]
fn test_mask_single_field_long_clickable() {
    let selector = Selector::new().long_clickable(true);
    let params = selector.to_params();
    assert_eq!(params["longClickable"], true);
    assert_eq!(params["mask"], 0x2000);
}

#[test]
fn test_mask_single_field_focused() {
    let selector = Selector::new().focused(true);
    let params = selector.to_params();
    assert_eq!(params["focused"], true);
    assert_eq!(params["mask"], 0x020000);
}

#[test]
fn test_mask_single_field_selected() {
    let selector = Selector::new().selected(true);
    let params = selector.to_params();
    assert_eq!(params["selected"], true);
    assert_eq!(params["mask"], 0x040000);
}

#[test]
fn test_mask_combination_all_extended_boolean_fields() {
    let selector = Selector::new()
        .checkable(true)
        .checked(false)
        .long_clickable(true)
        .focused(true)
        .selected(false);
    let params = selector.to_params();
    // mask = 0x0400 | 0x0800 | 0x2000 | 0x020000 | 0x040000 = 0x062C00
    assert_eq!(
        params["mask"],
        0x0400 | 0x0800 | 0x2000 | 0x020000 | 0x040000
    );
    assert_eq!(params["checkable"], true);
    assert_eq!(params["checked"], false);
    assert_eq!(params["longClickable"], true);
    assert_eq!(params["focused"], true);
    assert_eq!(params["selected"], false);
}

// === Matches 正则匹配字段测试 ===

#[test]
fn test_mask_single_field_class_name_matches() {
    let selector = Selector::new().class_name_matches(".*Button$");
    let params = selector.to_params();
    assert_eq!(params["classNameMatches"], ".*Button$");
    assert_eq!(params["mask"], 0x20);
}

#[test]
fn test_mask_single_field_description_matches() {
    let selector = Selector::new().description_matches("Submit.*");
    let params = selector.to_params();
    assert_eq!(params["descriptionMatches"], "Submit.*");
    assert_eq!(params["mask"], 0x0100);
}

#[test]
fn test_mask_single_field_description_starts_with() {
    let selector = Selector::new().description_starts_with("Submit");
    let params = selector.to_params();
    assert_eq!(params["descriptionStartsWith"], "Submit");
    assert_eq!(params["mask"], 0x0200);
}

#[test]
fn test_mask_single_field_package_name_matches() {
    let selector = Selector::new().package_name_matches(r"com\.example\..*");
    let params = selector.to_params();
    assert_eq!(params["packageNameMatches"], r"com\.example\..*");
    assert_eq!(params["mask"], 0x100000);
}

#[test]
fn test_mask_single_field_resource_id_matches() {
    let selector = Selector::new().resource_id_matches(".*:id/btn_.*");
    let params = selector.to_params();
    assert_eq!(params["resourceIdMatches"], ".*:id/btn_.*");
    assert_eq!(params["mask"], 0x400000);
}

#[test]
fn test_mask_combination_matches_fields() {
    let selector = Selector::new()
        .class_name_matches(".*Button$")
        .description_matches("Submit.*")
        .resource_id_matches(".*:id/btn_.*");
    let params = selector.to_params();
    // mask = 0x20 | 0x0100 | 0x400000
    assert_eq!(params["mask"], 0x20 | 0x0100 | 0x400000);
}

// === Index 字段测试 ===

#[test]
fn test_mask_single_field_index() {
    let selector = Selector::new().index(3);
    let params = selector.to_params();
    assert_eq!(params["index"], 3);
    assert_eq!(params["mask"], 0x800000);
}

#[test]
fn test_index_vs_instance() {
    let selector = Selector::new()
        .class_name("android.widget.TextView")
        .index(2)
        .instance(1);
    let params = selector.to_params();
    assert_eq!(params["index"], 2);
    assert_eq!(params["instance"], 1);
    // mask = 0x10 (className) | 0x800000 (index) | 0x01000000 (instance)
    assert_eq!(params["mask"], 0x10 | 0x800000 | 0x01000000);
}

// === 层级选择器测试 ===

#[test]
fn test_child_selector() {
    let selector = Selector::new()
        .text("Parent")
        .child(Selector::new().text("Child"));
    let params = selector.to_params();
    assert_eq!(params["text"], "Parent");
    assert_eq!(params["childOrSibling"][0], "child");
    assert_eq!(params["childOrSiblingSelector"][0]["text"], "Child");
    assert_eq!(params["childOrSiblingSelector"][0]["mask"], 0x01);
}

#[test]
fn test_sibling_selector() {
    let selector = Selector::new()
        .text("Label")
        .sibling(Selector::new().class_name("android.widget.EditText"));
    let params = selector.to_params();
    assert_eq!(params["childOrSibling"][0], "sibling");
    assert_eq!(
        params["childOrSiblingSelector"][0]["className"],
        "android.widget.EditText"
    );
    assert_eq!(params["childOrSiblingSelector"][0]["mask"], 0x10);
}

#[test]
fn test_nested_child_selector() {
    let selector = Selector::new().resource_id("list").child(
        Selector::new()
            .class_name("Item")
            .child(Selector::new().text("Title")),
    );
    let params = selector.to_params();
    assert_eq!(params["childOrSibling"][0], "child");
    let child = &params["childOrSiblingSelector"][0];
    assert_eq!(child["className"], "Item");
    // 子选择器也有自己的 childOrSibling
    assert_eq!(child["childOrSibling"][0], "child");
    assert_eq!(child["childOrSiblingSelector"][0]["text"], "Title");
}

#[test]
fn test_no_child_or_sibling_fields_when_empty() {
    let selector = Selector::new().text("test");
    let params = selector.to_params();
    assert!(params.get("childOrSibling").is_none());
    assert!(params.get("childOrSiblingSelector").is_none());
}
