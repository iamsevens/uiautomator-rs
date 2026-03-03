// 闆嗘垚娴嬭瘯锛氬厓绱犲畾浣嶅拰鎿嶄綔
// 闇€姹? 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8

mod common;

use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uiautomator::{Device, Key, Selector};

const INPUT_FORMS_ENTRY_ID: &str = "com.uiautomator.testapp:id/btn_input_forms";
const INPUT_FORMS_SUBMIT_ID: &str = "com.uiautomator.testapp:id/btn_submit";
const INPUT_FORMS_USERNAME_ID: &str = "com.uiautomator.testapp:id/et_username";
const INPUT_FORMS_PASSWORD_ID: &str = "com.uiautomator.testapp:id/et_password";
const INPUT_FORMS_EMAIL_ID: &str = "com.uiautomator.testapp:id/et_email";
const INPUT_FORMS_TITLE_ID: &str = "com.uiautomator.testapp:id/tv_input_title";
const ELEMENT_FAILURE_LOG_DIR: &str = "internal/testlogs/integration-element-failures";

fn artifact_prefix(device: &Device, label: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let serial = device
        .serial()
        .chars()
        .map(|c| {
            if matches!(c, ':' | '/' | '\\') {
                '_'
            } else {
                c
            }
        })
        .collect::<String>();
    format!("{label}-{serial}-{timestamp}")
}

async fn capture_debug_artifacts(device: &Device, label: &str) {
    let log_dir = PathBuf::from(ELEMENT_FAILURE_LOG_DIR);
    let _ = std::fs::create_dir_all(&log_dir);

    let prefix = artifact_prefix(device, label);

    let screenshot_path = log_dir.join(format!("{prefix}.png"));
    let _ = device
        .screenshot_to_file(&screenshot_path.to_string_lossy())
        .await;

    let xml_remote = format!("/data/local/tmp/{prefix}-window_dump.xml");
    let xml_local = log_dir.join(format!("{prefix}.xml"));
    let dump_cmd = format!("uiautomator dump {xml_remote}");
    let _ = device
        .adb_client()
        .shell(device.serial(), &dump_cmd, Some(Duration::from_secs(10)))
        .await;
    let _ = device
        .adb_client()
        .pull(
            device.serial(),
            &xml_remote,
            &xml_local.to_string_lossy().to_string(),
        )
        .await;
    let _ = device
        .adb_client()
        .shell(
            device.serial(),
            &format!("rm -f {xml_remote}"),
            Some(Duration::from_secs(5)),
        )
        .await;

    if let Ok(info) = device.info().await {
        let info_path = log_dir.join(format!("{prefix}.info.txt"));
        let content = format!(
            "serial={}\ncurrent_package={}\nscreen_on={}\nrotation={}\n",
            device.serial(),
            info.current_package_name,
            info.screen_on,
            info.display_rotation
        );
        let _ = std::fs::write(info_path, content);
    }
}

async fn exists_with_relaunch_retry(device: &Device, selector: Selector, attempts: usize) -> bool {
    for _ in 0..attempts {
        launch_test_app(device).await;
        let element = device.find(selector.clone());
        if element
            .exists(Some(Duration::from_secs(5)))
            .await
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

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

async fn scroll_form_towards_submit_with_adb(device: &Device) {
    let (screen_w, screen_h) = device
        .window_size()
        .await
        .expect("failed to get window size");
    let center_x = screen_w / 2;
    // Avoid starting swipe inside OEM soft keyboard area.
    let start_y = (screen_h * 11) / 20; // 55%
    let end_y = screen_h / 4; // 25%
    let cmd = format!("input swipe {center_x} {start_y} {center_x} {end_y} 300");
    device
        .adb_client()
        .shell(device.serial(), &cmd, None)
        .await
        .expect("failed to scroll form page towards submit with adb input swipe");
    common::wait_ui_stable().await;
}

async fn is_soft_keyboard_visible(device: &Device) -> bool {
    let cmd = "dumpsys input_method | grep -i mInputShown | head -n 1";
    match device
        .adb_client()
        .shell(device.serial(), cmd, Some(Duration::from_secs(3)))
        .await
    {
        Ok(output) => output.contains("mInputShown=true"),
        Err(_) => false,
    }
}

async fn hide_soft_keyboard_with_back_if_needed(device: &Device) {
    if !is_soft_keyboard_visible(device).await {
        return;
    }

    let _ = device
        .adb_client()
        .shell(
            device.serial(),
            "input keyevent 4",
            Some(Duration::from_secs(3)),
        )
        .await;
    common::wait_ui_stable().await;
}

async fn try_open_input_forms_once(device: &Device) -> bool {
    let input_entry = device.find(Selector::new().resource_id(INPUT_FORMS_ENTRY_ID));
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
            let username = device.find(Selector::new().resource_id(INPUT_FORMS_USERNAME_ID));
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
            let username = device.find(Selector::new().resource_id(INPUT_FORMS_USERNAME_ID));
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

async fn dismiss_soft_keyboard_if_needed(device: &Device) {
    // Defocus by tapping a non-editable area. This avoids BACK navigation and
    // avoids vendor IME interpreting keyevents as actual text input.
    let title = device.find(Selector::new().resource_id(INPUT_FORMS_TITLE_ID));
    if title
        .exists(Some(Duration::from_millis(500)))
        .await
        .unwrap_or(false)
    {
        if let Ok(info) = title.info().await {
            let center_x = ((info.bounds.left + info.bounds.right) / 2) as u32;
            let center_y = ((info.bounds.top + info.bounds.bottom) / 2) as u32;
            let _ = device.click(center_x, center_y).await;
            common::wait_ui_stable().await;
            return;
        }
    }

    if let Ok((w, h)) = device.window_size().await {
        let _ = device.click(w / 2, h / 8).await;
    }
    common::wait_ui_stable().await;
    hide_soft_keyboard_with_back_if_needed(device).await;
}

async fn ensure_input_forms_context(device: &Device) {
    let markers = [
        Selector::new().resource_id(INPUT_FORMS_USERNAME_ID),
        Selector::new().resource_id(INPUT_FORMS_EMAIL_ID),
        Selector::new().resource_id(INPUT_FORMS_SUBMIT_ID),
        Selector::new().resource_id(INPUT_FORMS_TITLE_ID),
    ];

    for _ in 0..3 {
        for marker in &markers {
            if device
                .find(marker.clone())
                .exists(Some(Duration::from_secs(1)))
                .await
                .unwrap_or(false)
            {
                return;
            }
        }
        common::wait_ui_stable().await;
    }

    // If entry button exists, we are on the app main page and can navigate safely.
    let input_entry = device.find(Selector::new().resource_id(INPUT_FORMS_ENTRY_ID));
    if input_entry
        .exists(Some(Duration::from_secs(1)))
        .await
        .unwrap_or(false)
    {
        open_input_forms_page(device).await;
        return;
    }

    // Last resort: reopen input forms page.
    open_input_forms_page(device).await;
}

async fn ensure_submit_context(device: &Device) {
    let submit = device.find(Selector::new().resource_id(INPUT_FORMS_SUBMIT_ID));

    if submit
        .exists(Some(Duration::from_secs(1)))
        .await
        .unwrap_or(false)
    {
        return;
    }

    for _ in 0..3 {
        scroll_form_towards_submit_with_adb(device).await;
        if submit
            .exists(Some(Duration::from_secs(1)))
            .await
            .unwrap_or(false)
        {
            return;
        }
    }
}

async fn read_title_text(device: &Device) -> Option<String> {
    read_field_text(device, INPUT_FORMS_TITLE_ID).await
}

async fn wait_submit_transition(
    device: &Device,
    baseline: &str,
    timeout: Duration,
) -> Option<String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(title) = read_title_text(device).await {
            if title.contains("Submitted") {
                return Some(title);
            }
            if title.contains("Invalid") && title != baseline {
                return Some(title);
            }
        }
        common::wait_ui_stable().await;
    }
    None
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

async fn read_field_text(device: &Device, resource_id: &str) -> Option<String> {
    let field = device.find(Selector::new().resource_id(resource_id));
    if !field
        .exists(Some(Duration::from_secs(1)))
        .await
        .unwrap_or(false)
    {
        return None;
    }

    field
        .get_text()
        .await
        .ok()
        .map(|text| text.trim().to_string())
}

fn is_valid_email_text(value: &str) -> bool {
    let v = value.trim();
    !v.is_empty() && v.contains('@') && v.contains('.')
}

async fn log_required_field_values(device: &Device, label: &str) {
    let username = read_field_text(device, INPUT_FORMS_USERNAME_ID).await;
    let password = read_field_text(device, INPUT_FORMS_PASSWORD_ID).await;
    let email = read_field_text(device, INPUT_FORMS_EMAIL_ID).await;
    let password_desc = password
        .as_ref()
        .map(|v| format!("len={}", v.chars().count()))
        .unwrap_or_else(|| "none".to_string());
    println!(
        "[form-debug][{label}] username={:?} password={} email={:?}",
        username, password_desc, email
    );
}

async fn required_form_fields_ready(device: &Device) -> bool {
    let username = read_field_text(device, INPUT_FORMS_USERNAME_ID).await;
    let password = read_field_text(device, INPUT_FORMS_PASSWORD_ID).await;
    let email = read_field_text(device, INPUT_FORMS_EMAIL_ID).await;

    let username_ok = username
        .as_deref()
        .map(|v| v.eq_ignore_ascii_case("bob"))
        .unwrap_or(false);
    let password_ok = password
        .as_deref()
        .map(|v| !v.is_empty() && !v.eq_ignore_ascii_case("Password"))
        .unwrap_or(false);
    let email_ok = email.as_deref().map(is_valid_email_text).unwrap_or(false);

    username_ok && password_ok && email_ok
}

async fn ensure_required_form_fields(device: &Device) -> bool {
    for _ in 0..4 {
        ensure_input_forms_context(device).await;
        ensure_submit_context(device).await;
        if required_form_fields_ready(device).await {
            return true;
        }

        if !set_text_with_retry(device, INPUT_FORMS_USERNAME_ID, "bob").await {
            continue;
        }
        if !set_text_with_retry(device, INPUT_FORMS_PASSWORD_ID, "123456").await {
            continue;
        }
        if !set_text_with_retry(device, INPUT_FORMS_EMAIL_ID, "bob@example.com").await {
            continue;
        }

        dismiss_soft_keyboard_if_needed(device).await;
        if required_form_fields_ready(device).await {
            return true;
        }
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
    let username_id = INPUT_FORMS_USERNAME_ID;
    let password_id = INPUT_FORMS_PASSWORD_ID;
    let email_id = INPUT_FORMS_EMAIL_ID;

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
        if username_ok && password_ok && email_ok && required_form_fields_ready(device).await {
            return true;
        }
    }

    false
}

async fn click_submit_with_retry(device: &Device) -> bool {
    for _ in 0..8 {
        ensure_input_forms_context(device).await;
        ensure_submit_context(device).await;
        dismiss_soft_keyboard_if_needed(device).await;
        hide_soft_keyboard_with_back_if_needed(device).await;
        let baseline_title = read_title_text(device).await.unwrap_or_default();

        let submit_selectors = [
            Selector::new().resource_id(INPUT_FORMS_SUBMIT_ID),
            Selector::new().text("SUBMIT"),
            Selector::new().text("Submit"),
            Selector::new().text_contains("SUBMIT"),
            Selector::new().text_contains("Submit"),
        ];

        for selector in submit_selectors {
            let submit = device.find(selector);
            if submit
                .exists(Some(Duration::from_secs(1)))
                .await
                .unwrap_or(false)
                && submit
                    .click(Some(Duration::from_secs(3)), None)
                    .await
                    .is_ok()
            {
                if wait_submit_transition(device, &baseline_title, Duration::from_secs(2))
                    .await
                    .is_some()
                {
                    return true;
                }
            }
        }

        scroll_form_towards_submit_with_adb(device).await;
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
    let exists = exists_with_relaunch_retry(&device, selector, 3).await;
    if !exists {
        capture_debug_artifacts(&device, "find_by_text").await;
    }
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
    let exists = exists_with_relaunch_retry(&device, selector, 3).await;
    if !exists {
        capture_debug_artifacts(&device, "find_by_resource_id").await;
    }
    assert!(
        exists,
        "搴旇鑳介€氳繃璧勬簮 ID 鎵惧埌 Basic Controls 鎸夐挳"
    );
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

    let filled = fill_required_form_fields(&device).await;
    if !filled {
        capture_debug_artifacts(&device, "submit_fill_fields").await;
    }
    assert!(filled, "failed to fill form fields after retries");
    log_required_field_values(&device, "after_initial_fill").await;
    assert!(
        ensure_required_form_fields(&device).await,
        "failed to ensure required field values before submit"
    );
    log_required_field_values(&device, "before_initial_submit").await;

    ensure_submit_context(&device).await;
    let clicked = click_submit_with_retry(&device).await;
    if !clicked {
        capture_debug_artifacts(&device, "submit_click_button").await;
    }
    assert!(clicked, "failed to click submit button");

    let title_id = INPUT_FORMS_TITLE_ID;
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
                    log_required_field_values(&device, "invalid_before_refill").await;
                    assert!(
                        ensure_required_form_fields(&device).await
                            || fill_required_form_fields(&device).await,
                        "failed to refill required fields after invalid submit"
                    );
                    log_required_field_values(&device, "invalid_after_refill").await;
                    assert!(
                        click_submit_with_retry(&device).await,
                        "failed to click submit button after refilling"
                    );
                }
            }
        }

        if Instant::now() >= deadline {
            capture_debug_artifacts(&device, "submit_result_timeout").await;
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

            assert!(
                !info.class_name.is_empty(),
                "class name should not be empty"
            );
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
            assert!(
                text.contains("UIAutomator"),
                "鏍囬鏂囨湰搴斿寘鍚?UIAutomator"
            );
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
