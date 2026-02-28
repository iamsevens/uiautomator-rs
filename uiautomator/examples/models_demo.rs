//! 数据模型使用示例
//!
//! 演示如何使用 uiautomator 库的各种模型

use std::time::Duration;
use uiautomator::{AppInfo, DeviceInfo, ElementInfo, Key, Rect, Settings};

fn main() {
    println!("=== 数据模型示例 ===\n");

    // 1. Rect 示例
    println!("1. Rect (矩形) 示例:");
    let rect = Rect::new(100, 200, 500, 800);
    println!("   矩形: {:?}", rect);
    println!("   宽度: {}", rect.width());
    println!("   高度: {}", rect.height());
    println!("   中心: {:?}", rect.center());
    println!();

    // 2. DeviceInfo 示例
    println!("2. DeviceInfo (设备信息) 示例:");
    let device_info = DeviceInfo {
        display_width: 1080,
        display_height: 1920,
        display_rotation: 0,
        current_package_name: "com.android.settings".to_string(),
        sdk_int: 30,
        screen_on: true,
        natural_orientation: true,
    };
    println!("   设备信息: {:?}", device_info);
    println!(
        "   屏幕尺寸: {}x{}",
        device_info.display_width, device_info.display_height
    );
    println!("   当前应用: {}", device_info.current_package_name);
    println!();

    // 3. ElementInfo 示例
    println!("3. ElementInfo (元素信息) 示例:");
    let element_info = ElementInfo {
        text: "设置".to_string(),
        content_description: "设置按钮".to_string(),
        class_name: "android.widget.TextView".to_string(),
        package_name: "com.android.settings".to_string(),
        resource_id: "com.android.settings:id/title".to_string(),
        bounds: Rect::new(100, 200, 500, 300),
        visible_bounds: Rect::new(100, 200, 500, 300),
        clickable: true,
        enabled: true,
        focusable: true,
        focused: false,
        scrollable: false,
        long_clickable: true,
        checkable: false,
        checked: false,
        selected: false,
        child_count: 0,
    };
    println!("   元素信息: {:?}", element_info);
    println!("   文本: {}", element_info.text);
    println!("   类名: {}", element_info.class_name);
    println!("   可点击: {}", element_info.clickable);
    println!();

    // 4. AppInfo 示例
    println!("4. AppInfo (应用信息) 示例:");
    let app_info = AppInfo {
        package: "com.android.settings".to_string(),
        activity: ".Settings".to_string(),
        pid: Some(12345),
    };
    println!("   应用信息: {:?}", app_info);
    println!("   包名: {}", app_info.package);
    println!("   Activity: {}", app_info.activity);
    println!("   进程 ID: {:?}", app_info.pid);
    println!();

    // 5. Key 示例
    println!("5. Key (按键) 示例:");
    let keys = vec![
        Key::Home,
        Key::Back,
        Key::Power,
        Key::VolumeUp,
        Key::VolumeDown,
        Key::Enter,
    ];
    for key in keys {
        println!(
            "   {:?}: 键码={}, 名称={}",
            key,
            key.to_keycode(),
            key.to_name()
        );
    }
    println!();

    // 6. Key 名称解析示例
    println!("6. Key 名称解析示例:");
    let key_names = vec!["home", "back", "volume_up", "invalid"];
    for name in key_names {
        match Key::from_name(name) {
            Some(key) => println!("   '{}' -> {:?} (键码: {})", name, key, key.to_keycode()),
            None => println!("   '{}' -> 无效的按键名称", name),
        }
    }
    println!();

    // 7. Settings 示例
    println!("7. Settings (配置) 示例:");
    let settings = Settings::default();
    println!("   默认配置: {:?}", settings);
    println!("   等待超时: {:?}", settings.wait_timeout);
    println!("   HTTP 超时: {:?}", settings.http_timeout);
    println!("   最大重试次数: {}", settings.max_retry);
    println!();

    // 8. Settings 构建器模式示例
    println!("8. Settings 构建器模式示例:");
    let custom_settings = Settings::new()
        .with_wait_timeout(Duration::from_secs(30))
        .with_http_timeout(Duration::from_secs(120))
        .with_max_retry(5)
        .with_operation_delay_before(Duration::from_millis(100))
        .with_operation_delay_after(Duration::from_millis(200));
    println!("   自定义配置: {:?}", custom_settings);
    println!();

    // 9. JSON 序列化示例
    println!("9. JSON 序列化示例:");
    let rect_json = serde_json::to_string_pretty(&rect).unwrap();
    println!("   Rect JSON:\n{}", rect_json);
    println!();

    let device_info_json = serde_json::to_string_pretty(&device_info).unwrap();
    println!("   DeviceInfo JSON:\n{}", device_info_json);
    println!();

    println!("=== 示例完成 ===");
}
