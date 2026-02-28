// 闆嗘垚娴嬭瘯锛氬厓绱犲畾浣嶅拰鎿嶄綔
// 闇€姹? 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8

mod common;

use std::time::{Duration, Instant};
use uiautomator::{Device, Key, Selector};

async fn launch_test_app(device: &Device) {
    // Stop first to avoid inheriting an unpredictable previous page stack/state.
    let _ = device.app_stop(common::TEST_APP_PACKAGE).await;
    device
        .app_start(common::TEST_APP_PACKAGE, Some(common::TEST_APP_ACTIVITY))
        .await
        .expect("failed to launch test-app");
    common::wait_ui_stable().await;
}
async fn scroll_screen_with_adb(device: &Device, forward: bool) {
    let (screen_w, screen_h) = device
        .window_size()
        .await
        .expect("failed to get window size");
    let center_x = screen_w / 2;
    let start_y = (screen_h * 4) / 5;
    let end_y = (screen_h * 2) / 5;
    let (from_y, to_y) = if forward {
        (start_y, end_y)
    } else {
        (end_y, start_y)
    };
    let cmd = format!("input swipe {center_x} {from_y} {center_x} {to_y} 300");
    device
        .adb_client()
        .shell(device.serial(), &cmd, None)
        .await
        .expect("failed to scroll main page with adb input swipe");
    common::wait_ui_stable().await;
}
async fn try_open_input_forms_once(device: &Device) -> bool {
    let input_entry =
        device.find(Selector::new().resource_id("com.uiautomator.testapp:id/btn_input_forms"));
    if input_entry
        .exists(Some(Duration::from_secs(2)))
        .await
        .unwrap_or(false)
    {
        if input_entry
            .click(Some(Duration::from_secs(5)), None)
            .await
            .is_ok()
        {
            let username =
                device.find(Selector::new().resource_id("com.uiautomator.testapp:id/et_username"));
            if username
                .exists(Some(Duration::from_secs(4)))
                .await
                .unwrap_or(false)
            {
                return true;
            }
        }
    }
    let input_text_entry = device.find(Selector::new().text_contains("INPUT"));
    if input_text_entry
        .exists(Some(Duration::from_secs(2)))
        .await
        .unwrap_or(false)
    {
        if input_text_entry
            .click(Some(Duration::from_secs(5)), None)
            .await
            .is_ok()
        {
            let username =
                device.find(Selector::new().resource_id("com.uiautomator.testapp:id/et_username"));
            if username
                .exists(Some(Duration::from_secs(4)))
                .await
                .unwrap_or(false)
            {
                return true;
            }
        }
    }
    false
}
async fn open_input_forms_page(device: &Device) {
    launch_test_app(device).await;
    for attempt in 0..8 {
        if try_open_input_forms_once(device).await {
            return;
        }
        if attempt == 3 {
            // Relaunch once in the middle to recover from transient overlays/dialogs.
            launch_test_app(device).await;
            continue;
        }
        scroll_screen_with_adb(device, attempt < 4).await;
    }
    panic!("failed to open Input Forms page after retries");
}

fn is_soft_keyboard_package(package: &str) -> bool {
    let package = package.to_ascii_lowercase();
    package.contains("honeyboard")
        || package.contains("inputmethod")
        || package.contains("keyboard")
        || package.contains("latin")
}

async fn dismiss_soft_keyboard_if_needed(device: &Device) {
    if let Ok(info) = device.info().await {
        if is_soft_keyboard_package(&info.current_package_name) {
            let _ = device.press(Key::Back).await;
            common::wait_ui_stable().await;
        }
    }
}

async fn ensure_input_forms_context(device: &Device) {
    let username = device.find(Selector::new().resource_id("com.uiautomator.testapp:id/et_username"));
    if username
        .exists(Some(Duration::from_secs(1)))
        .await
        .unwrap_or(false)
    {
        return;
    }
    open_input_forms_page(device).await;
}

async fn set_text_with_retry(device: &Device, resource_id: &str, text: &str) -> bool {
    for attempt in 0..8 {
        ensure_input_forms_context(device).await;
        dismiss_soft_keyboard_if_needed(device).await;

        let field = device.find(Selector::new().resource_id(resource_id));
        if !field
            .exists(Some(Duration::from_secs(1)))
            .await
            .unwrap_or(false)
        {
            scroll_screen_with_adb(device, attempt % 2 == 0).await;
            continue;
        }

        common::wait_ui_stable().await;
        if field.set_text(text).await.is_ok() {
            common::wait_ui_stable().await;
            dismiss_soft_keyboard_if_needed(device).await;
            return true;
        }

        common::wait_ui_stable().await;
    }

    false
}

async fn clear_text_with_retry(device: &Device, resource_id: &str) -> bool {
    for attempt in 0..8 {
        ensure_input_forms_context(device).await;
        dismiss_soft_keyboard_if_needed(device).await;

        let field = device.find(Selector::new().resource_id(resource_id));
        if !field
            .exists(Some(Duration::from_secs(1)))
            .await
            .unwrap_or(false)
        {
            scroll_screen_with_adb(device, attempt % 2 == 0).await;
            continue;
        }

        common::wait_ui_stable().await;
        if field.clear_text().await.is_ok() {
            common::wait_ui_stable().await;
            dismiss_soft_keyboard_if_needed(device).await;
            return true;
        }

        common::wait_ui_stable().await;
    }

    false
}

async fn field_has_non_hint_text(device: &Device, resource_id: &str, hint_text: &str) -> bool {
    let field = device.find(Selector::new().resource_id(resource_id));
    if !field
        .exists(Some(Duration::from_secs(1)))
        .await
        .unwrap_or(false)
    {
        return false;
    }

    match field.get_text().await {
        Ok(text) => {
            let value = text.trim();
            !value.is_empty() && !value.eq_ignore_ascii_case(hint_text)
        }
        Err(_) => false,
    }
}

async fn fill_required_form_fields(device: &Device) -> bool {
    let username_id = "com.uiautomator.testapp:id/et_username";
    let password_id = "com.uiautomator.testapp:id/et_password";
    let email_id = "com.uiautomator.testapp:id/et_email";

    for _ in 0..4 {
        open_input_forms_page(device).await;
        if !(set_text_with_retry(device, username_id, "bob").await
            && set_text_with_retry(device, password_id, "123456").await
            && set_text_with_retry(device, email_id, "bob@example.com").await)
        {
            continue;
        }

        dismiss_soft_keyboard_if_needed(device).await;

        let username_ok = field_has_non_hint_text(device, username_id, "Username").await;
        let password_ok = field_has_non_hint_text(device, password_id, "Password").await;
        let email_ok = field_has_non_hint_text(device, email_id, "Email").await;
        if username_ok && password_ok && email_ok {
            return true;
        }
    }

    false
}

async fn click_submit_with_retry(device: &Device) -> bool {
    let submit_id = "com.uiautomator.testapp:id/btn_submit";
    dismiss_soft_keyboard_if_needed(device).await;

    for _ in 0..6 {
        let submit = device.find(Selector::new().resource_id(submit_id));
        if submit
            .exists(Some(Duration::from_secs(1)))
            .await
            .unwrap_or(false)
            && submit.click(Some(Duration::from_secs(3)), None).await.is_ok()
        {
            return true;
        }

        scroll_screen_with_adb(device, true).await;
    }

    false
}

#[tokio::test]
async fn test_element_find_by_text() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 鍚姩 test-app
    launch_test_app(&device).await;

    // 鏌ユ壘棣栭〉鏍囬
    let selector = Selector::new().text_contains("UIAutomator Test Suite");
    let element = device.find(selector);

    let exists = element.exists(Some(Duration::from_secs(5))).await.unwrap();
    assert!(exists, "搴旇鑳芥壘鍒?test-app 棣栭〉鏍囬");
    println!("鍏冪礌瀛樺湪: {}", exists);

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_element_find_by_resource_id() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 鍚姩 test-app
    launch_test_app(&device).await;

    // 鏌ユ壘 test-app 棣栭〉鎸夐挳
    let selector = Selector::new().resource_id("com.uiautomator.testapp:id/btn_basic_controls");
    let element = device.find(selector);

    let exists = element.exists(Some(Duration::from_secs(5))).await.unwrap();
    assert!(exists, "搴旇鑳介€氳繃璧勬簮 ID 鎵惧埌 Basic Controls 鎸夐挳");
    println!("閫氳繃璧勬簮 ID 鎵惧埌鍏冪礌: {}", exists);

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_element_find_by_class_name() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 鍚姩 test-app
    launch_test_app(&device).await;

    // Find TextView element.
    let selector = Selector::new().class_name("android.widget.TextView");
    let element = device.find(selector);

    let exists = element.exists(Some(Duration::from_secs(5))).await.unwrap();
    assert!(exists, "搴旇鑳芥壘鍒?TextView 鍏冪礌");

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_element_combined_selector() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 鍚姩 test-app
    launch_test_app(&device).await;

    // 缁勫悎澶氫釜鏉′欢
    let selector = Selector::new()
        .class_name("android.widget.Button")
        .clickable(true);
    let element = device.find(selector);

    let exists = element.exists(Some(Duration::from_secs(5))).await.unwrap();
    assert!(exists, "棣栭〉搴旇瀛樺湪鍙偣鍑荤殑鎸夐挳");
    println!("缁勫悎閫夋嫨鍣ㄦ壘鍒板厓绱? {}", exists);

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_element_not_found() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // Look up a non-existing element.
    let selector = Selector::new().text("ThisTextDefinitelyDoesNotExist12345");
    let element = device.find(selector);

    let exists = element.exists(Some(Duration::from_secs(2))).await.unwrap();
    assert!(!exists, "涓嶅瓨鍦ㄧ殑鍏冪礌搴旇杩斿洖 false");

    println!("鉁?姝ｇ‘澶勭悊鍏冪礌涓嶅瓨鍦ㄧ殑鎯呭喌");
}

#[tokio::test]
async fn test_element_set_and_clear_text() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    let username_id = "com.uiautomator.testapp:id/et_username";
    let mut success = false;
    for _ in 0..3 {
        open_input_forms_page(&device).await;
        let username = device.find(Selector::new().resource_id(username_id));
        if !username
            .exists(Some(Duration::from_secs(5)))
            .await
            .unwrap_or(false)
        {
            continue;
        }

        if set_text_with_retry(&device, username_id, "alice").await
            && clear_text_with_retry(&device, username_id).await
            && set_text_with_retry(&device, username_id, "bob").await
            && clear_text_with_retry(&device, username_id).await
        {
            success = true;
            break;
        }
    }

    assert!(success, "failed to set and clear username after retries");

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_element_submit_form_updates_result() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    assert!(
        fill_required_form_fields(&device).await,
        "failed to fill form fields after retries"
    );
    assert!(click_submit_with_retry(&device).await, "failed to click submit button");

    let title_id = "com.uiautomator.testapp:id/tv_input_title";
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut latest = String::new();
    let mut refill_attempts = 0u8;
    loop {
        let title = device.find(Selector::new().resource_id(title_id));
        if title
            .exists(Some(Duration::from_secs(1)))
            .await
            .unwrap_or(false)
        {
            if let Ok(text) = title.get_text().await {
                latest = text.clone();
                if text.contains("Submitted") {
                    break;
                }
                if text.contains("Invalid") && refill_attempts < 2 {
                    refill_attempts += 1;
                    assert!(
                        fill_required_form_fields(&device).await,
                        "failed to refill required fields after invalid submit"
                    );
                    assert!(
                        click_submit_with_retry(&device).await,
                        "failed to click submit button after refilling"
                    );
                }
            }
        }

        if Instant::now() >= deadline {
            panic!("submit result did not match expectation, latest text: {latest}");
        }

        common::wait_ui_stable().await;
    }

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_element_wait_appear() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 鍚姩 test-app
    launch_test_app(&device).await;

    // 绛夊緟鍏冪礌鍑虹幇
    let selector = Selector::new().resource_id("com.uiautomator.testapp:id/btn_gestures");
    let element = device.find(selector);

    let result = element.wait(Some(Duration::from_secs(10))).await;
    assert!(result.is_ok(), "鍏冪礌搴旇鍦ㄨ秴鏃跺墠鍑虹幇");

    println!("鉁?鍏冪礌绛夊緟鍔熻兘姝ｅ父");

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_element_wait_timeout() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // Wait for a non-existing element.
    let selector = Selector::new().text("ThisWillNeverAppear12345");
    let element = device.find(selector);

    let result = element.wait(Some(Duration::from_secs(2))).await;
    assert!(result.is_err(), "绛夊緟涓嶅瓨鍦ㄧ殑鍏冪礌搴旇瓒呮椂");

    println!("鉁?绛夊緟瓒呮椂鍔熻兘姝ｅ父");
}

#[tokio::test]
async fn test_element_get_info() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 鍚姩 test-app
    launch_test_app(&device).await;

    // 鏌ユ壘涓€涓厓绱犲苟鑾峰彇淇℃伅
    let selector = Selector::new().resource_id("com.uiautomator.testapp:id/btn_basic_controls");
    let element = device.find(selector);

    if element.exists(Some(Duration::from_secs(5))).await.unwrap() {
        let info = element.info().await;

        if let Ok(info) = info {
            println!("鍏冪礌淇℃伅:");
            println!("  绫诲悕: {}", info.class_name);
            println!("  鏂囨湰: {}", info.text);
            println!("  杈圭晫: {:?}", info.bounds);
            println!("  鍙偣鍑? {}", info.clickable);

            assert!(!info.class_name.is_empty(), "class name should not be empty");
            assert!(
                info.class_name.contains("Button"),
                "搴旇鍛戒腑鎸夐挳鎺т欢锛屽疄闄? {}",
                info.class_name
            );
        }
    }

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_element_get_text() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 鍚姩 test-app
    launch_test_app(&device).await;

    // 鏌ユ壘鏈夋枃鏈殑鍏冪礌
    let selector = Selector::new().resource_id("com.uiautomator.testapp:id/tv_main_title");
    let element = device.find(selector);

    if element.exists(Some(Duration::from_secs(5))).await.unwrap() {
        let text = element.get_text().await;

        if let Ok(text) = text {
            println!("鍏冪礌鏂囨湰: {}", text);
            assert!(text.contains("UIAutomator"), "鏍囬鏂囨湰搴斿寘鍚?UIAutomator");
        }
    }

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_element_click() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 鍚姩 test-app
    launch_test_app(&device).await;

    // Click Basic Controls entry on main page.
    let selector = Selector::new().resource_id("com.uiautomator.testapp:id/btn_basic_controls");
    let element = device.find(selector);

    if element.exists(Some(Duration::from_secs(5))).await.unwrap() {
        let result = element.click(None, None).await;

        if result.is_ok() {
            println!("鉁?鍏冪礌鐐瑰嚮鎴愬姛");
            common::wait_ui_stable().await;

            let basic_title = device
                .find(Selector::new().resource_id("com.uiautomator.testapp:id/tv_basic_title"));
            assert!(
                basic_title
                    .exists(Some(Duration::from_secs(5)))
                    .await
                    .unwrap(),
                "鐐瑰嚮鍚庡簲杩涘叆 Basic Controls 椤甸潰"
            );

            // 杩斿洖
            device.press(Key::Back).await.ok();
        }
    }

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_element_click_exists() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 鍚姩 test-app
    launch_test_app(&device).await;

    // 灏濊瘯鐐瑰嚮瀛樺湪鐨勫厓绱犲苟楠岃瘉瀵艰埅
    let selector = Selector::new().resource_id("com.uiautomator.testapp:id/btn_gestures");
    let element = device.find(selector);

    let clicked = element
        .click_exists(Some(Duration::from_secs(5)))
        .await
        .unwrap();
    println!("鍏冪礌鏄惁琚偣鍑? {}", clicked);

    if clicked {
        common::wait_ui_stable().await;

        let gestures_title = device
            .find(Selector::new().resource_id("com.uiautomator.testapp:id/tv_gestures_title"));
        assert!(
            gestures_title
                .exists(Some(Duration::from_secs(5)))
                .await
                .unwrap(),
            "鐐瑰嚮鍚庡簲杩涘叆 Gestures 椤甸潰"
        );

        device.press(Key::Back).await.ok();
    }

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_element_bounds() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 鍚姩 test-app
    launch_test_app(&device).await;

    // 鑾峰彇鍏冪礌杈圭晫
    let selector = Selector::new().resource_id("com.uiautomator.testapp:id/btn_input_forms");
    let element = device.find(selector);

    if element.exists(Some(Duration::from_secs(5))).await.unwrap() {
        let bounds = element.bounds().await;

        if let Ok(bounds) = bounds {
            println!(
                "鍏冪礌杈圭晫: left={}, top={}, right={}, bottom={}",
                bounds.left, bounds.top, bounds.right, bounds.bottom
            );

            assert!(bounds.width() > 0, "瀹藉害搴旇澶т簬 0");
            assert!(bounds.height() > 0, "楂樺害搴旇澶т簬 0");
        }
    }

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_element_center() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();

    // 鍚姩 test-app
    launch_test_app(&device).await;

    // 鑾峰彇鍏冪礌涓績鍧愭爣
    let selector = Selector::new().resource_id("com.uiautomator.testapp:id/btn_input_forms");
    let element = device.find(selector);

    if element.exists(Some(Duration::from_secs(5))).await.unwrap() {
        let center = element.center().await;

        if let Ok((x, y)) = center {
            println!("鍏冪礌涓績鍧愭爣: ({}, {})", x, y);

            assert!(x > 0, "X 鍧愭爣搴旇澶т簬 0");
            assert!(y > 0, "Y 鍧愭爣搴旇澶т簬 0");
        }
    }

    common::cleanup_test_env(&device).await.ok();
}
