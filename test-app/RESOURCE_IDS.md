# UIAutomator Test App - Resource IDs Reference

## 完整的 Resource ID 列表

所有控件的 Resource ID，用于 Rust 测试脚本。

### MainActivity (主菜单)

```
com.uiautomator.testapp:id/tv_main_title
com.uiautomator.testapp:id/tv_main_subtitle
com.uiautomator.testapp:id/btn_basic_controls
com.uiautomator.testapp:id/btn_gestures
com.uiautomator.testapp:id/btn_input_forms
com.uiautomator.testapp:id/btn_lists
com.uiautomator.testapp:id/btn_dialogs
com.uiautomator.testapp:id/btn_navigation
com.uiautomator.testapp:id/btn_animations
com.uiautomator.testapp:id/btn_stress
com.uiautomator.testapp:id/btn_concurrent
```

### BasicControlsActivity (基础控件)

```
com.uiautomator.testapp:id/tv_basic_title
com.uiautomator.testapp:id/btn_normal
com.uiautomator.testapp:id/btn_disabled
com.uiautomator.testapp:id/cb_option
com.uiautomator.testapp:id/rg_options
com.uiautomator.testapp:id/rb_option1
com.uiautomator.testapp:id/rb_option2
com.uiautomator.testapp:id/rb_option3
com.uiautomator.testapp:id/sw_toggle
com.uiautomator.testapp:id/tv_result
com.uiautomator.testapp:id/btn_reset
```

### GesturesActivity (手势)

```
com.uiautomator.testapp:id/tv_gestures_title
com.uiautomator.testapp:id/tv_click_area
com.uiautomator.testapp:id/tv_long_click_area
com.uiautomator.testapp:id/tv_double_click_area
com.uiautomator.testapp:id/tv_swipe_area
com.uiautomator.testapp:id/view_drag
```

### InputFormsActivity (输入表单)

```
com.uiautomator.testapp:id/tv_input_title
com.uiautomator.testapp:id/et_username
com.uiautomator.testapp:id/et_password
com.uiautomator.testapp:id/et_email
com.uiautomator.testapp:id/et_comment
com.uiautomator.testapp:id/tv_country_label
com.uiautomator.testapp:id/sp_country
com.uiautomator.testapp:id/btn_submit
com.uiautomator.testapp:id/btn_clear
com.uiautomator.testapp:id/tv_form_result
```

### ListsActivity (列表)

```
com.uiautomator.testapp:id/tv_lists_title
com.uiautomator.testapp:id/tab_layout
com.uiautomator.testapp:id/list_view
com.uiautomator.testapp:id/recycler_view
com.uiautomator.testapp:id/scroll_view
com.uiautomator.testapp:id/tv_scroll_content
```

### DialogsActivity (对话框)

```
com.uiautomator.testapp:id/tv_dialogs_title
com.uiautomator.testapp:id/btn_alert
com.uiautomator.testapp:id/btn_confirm
com.uiautomator.testapp:id/btn_custom
com.uiautomator.testapp:id/btn_bottom_sheet
com.uiautomator.testapp:id/tv_dialog_result
com.uiautomator.testapp:id/btn_dialog_ok
com.uiautomator.testapp:id/btn_dialog_cancel
com.uiautomator.testapp:id/btn_sheet_option1
com.uiautomator.testapp:id/btn_sheet_option2
com.uiautomator.testapp:id/btn_sheet_option3
```

### NavigationActivity (导航)

```
com.uiautomator.testapp:id/tv_navigation_title
com.uiautomator.testapp:id/tv_page_info
com.uiautomator.testapp:id/btn_next_page
com.uiautomator.testapp:id/btn_back
```

### AnimationsActivity (动画)

```
com.uiautomator.testapp:id/tv_animations_title
com.uiautomator.testapp:id/view_animation_target
com.uiautomator.testapp:id/btn_fade
com.uiautomator.testapp:id/btn_slide
com.uiautomator.testapp:id/btn_rotate
com.uiautomator.testapp:id/btn_scale
com.uiautomator.testapp:id/btn_stop_all
com.uiautomator.testapp:id/tv_animation_status
```

### StressTestActivity (压力测试)

```
com.uiautomator.testapp:id/tv_stress_title
com.uiautomator.testapp:id/btn_rapid_click
com.uiautomator.testapp:id/tv_click_count
com.uiautomator.testapp:id/btn_memory_stress
com.uiautomator.testapp:id/tv_memory_usage
com.uiautomator.testapp:id/btn_animation_stress
com.uiautomator.testapp:id/tv_animation_count
```

### ConcurrentTestActivity (并发测试)

```
com.uiautomator.testapp:id/tv_concurrent_title
com.uiautomator.testapp:id/tv_counter1
com.uiautomator.testapp:id/tv_counter2
com.uiautomator.testapp:id/tv_counter3
com.uiautomator.testapp:id/btn_increment_all
com.uiautomator.testapp:id/btn_reset_all
```

## Rust 测试示例

```rust
use uiautomator::{Device, Selector};

const APP_PACKAGE: &str = "com.uiautomator.testapp";

// 点击按钮
device.find(Selector::new()
    .resource_id("com.uiautomator.testapp:id/btn_normal"))
    .click(None, None).await?;

// 获取文本
let text = device.find(Selector::new()
    .resource_id("com.uiautomator.testapp:id/tv_result"))
    .get_text().await?;

// 输入文本
device.find(Selector::new()
    .resource_id("com.uiautomator.testapp:id/et_username"))
    .set_text("testuser").await?;
```
