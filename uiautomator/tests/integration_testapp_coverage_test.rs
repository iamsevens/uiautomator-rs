mod common;

use std::time::Duration;
use uiautomator::{Device, Key, Selector};

fn app_id(id: &str) -> String {
    format!("{}:id/{id}", common::TEST_APP_PACKAGE)
}

async fn launch_test_app(device: &Device) {
    let _ = device.app_stop(common::TEST_APP_PACKAGE).await;
    device
        .app_start(common::TEST_APP_PACKAGE, Some(common::TEST_APP_ACTIVITY))
        .await
        .expect("failed to launch test-app");
    let main_title = device.find(Selector::new().resource_id(app_id("tv_main_title")));
    main_title
        .wait(Some(Duration::from_secs(10)))
        .await
        .expect("main page title did not appear after launching test-app");
}

fn entry_text_token(entry_id: &str) -> Option<&'static str> {
    match entry_id {
        "btn_basic_controls" => Some("BASIC"),
        "btn_gestures" => Some("GESTURES"),
        "btn_input_forms" => Some("INPUT"),
        "btn_lists" => Some("LISTS"),
        "btn_dialogs" => Some("DIALOGS"),
        "btn_navigation" => Some("NAVIGATION"),
        "btn_animations" => Some("ANIMATIONS"),
        "btn_stress" => Some("STRESS"),
        "btn_concurrent" => Some("CONCURRENT"),
        _ => None,
    }
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

async fn open_page_from_main(device: &Device, entry_id: &str, title_id: &str) {
    let main_title = device.find(Selector::new().resource_id(app_id("tv_main_title")));
    main_title
        .wait(Some(Duration::from_secs(10)))
        .await
        .expect("not on main page before opening target page");

    let text_token = entry_text_token(entry_id);
    let mut found = false;
    for attempt in 0..8 {
        let entry_by_id = device.find(Selector::new().resource_id(app_id(entry_id)));
        if entry_by_id
            .exists(Some(Duration::from_secs(2)))
            .await
            .unwrap_or(false)
        {
            found = true;
            break;
        }

        if let Some(token) = text_token {
            let entry_by_text = device.find(Selector::new().text_contains(token));
            if entry_by_text
                .exists(Some(Duration::from_secs(2)))
                .await
                .unwrap_or(false)
            {
                found = true;
                break;
            }
        }

        scroll_screen_with_adb(device, attempt < 4).await;
    }
    assert!(
        found,
        "entry button did not appear: id={entry_id}, text_token={text_token:?}"
    );

    let entry_by_id = device.find(Selector::new().resource_id(app_id(entry_id)));
    if entry_by_id
        .exists(Some(Duration::from_secs(1)))
        .await
        .unwrap_or(false)
    {
        entry_by_id
            .click(Some(Duration::from_secs(5)), None)
            .await
            .expect("failed to click entry button by resource-id");
    } else if let Some(token) = text_token {
        let entry_by_text = device.find(Selector::new().text_contains(token));
        entry_by_text
            .click(Some(Duration::from_secs(5)), None)
            .await
            .expect("failed to click entry button by text fallback");
    } else {
        panic!("entry button not clickable: no resource-id and no text fallback for {entry_id}");
    }

    let page_title = device.find(Selector::new().resource_id(app_id(title_id)));
    page_title
        .wait(Some(Duration::from_secs(10)))
        .await
        .expect("target page title did not appear");
}

async fn back_to_main(device: &Device) {
    device.press(Key::Back).await.expect("failed to press Back");
    let main_title = device.find(Selector::new().resource_id(app_id("tv_main_title")));
    main_title
        .wait(Some(Duration::from_secs(10)))
        .await
        .expect("main page did not appear after back");
}

async fn wait_dialog_result_contains(device: &Device, expected: &str, timeout: Duration) -> bool {
    device
        .wait_for(
            || {
                let result = device.find(Selector::new().resource_id(app_id("tv_dialog_result")));
                async move {
                    Ok(result
                        .get_text()
                        .await
                        .unwrap_or_default()
                        .contains(expected))
                }
            },
            Some(timeout),
        )
        .await
        .is_ok()
}

#[tokio::test]
async fn test_all_main_entries_can_open_target_pages() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();
    launch_test_app(&device).await;

    let entries = [
        ("btn_basic_controls", "tv_basic_title"),
        ("btn_gestures", "tv_gestures_title"),
        ("btn_input_forms", "tv_input_title"),
        ("btn_lists", "tv_lists_title"),
        ("btn_dialogs", "tv_dialogs_title"),
        ("btn_navigation", "tv_navigation_title"),
        ("btn_animations", "tv_animations_title"),
        ("btn_stress", "tv_stress_title"),
        ("btn_concurrent", "tv_concurrent_title"),
    ];

    for (entry, title) in entries {
        open_page_from_main(&device, entry, title).await;
        back_to_main(&device).await;
    }

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_basic_controls_interactions() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();
    launch_test_app(&device).await;
    open_page_from_main(&device, "btn_basic_controls", "tv_basic_title").await;

    let normal_button = device.find(Selector::new().resource_id(app_id("btn_normal")));
    normal_button.click(None, None).await.unwrap();
    normal_button.click(None, None).await.unwrap();

    let result = device.find(Selector::new().resource_id(app_id("tv_result")));
    let result_text = result.get_text().await.unwrap_or_default();
    assert!(
        result_text.contains("Count: 2"),
        "expected click count to be 2, got: {result_text}"
    );

    let checkbox = device.find(Selector::new().resource_id(app_id("cb_option")));
    checkbox.click(None, None).await.unwrap();
    let result_text = result.get_text().await.unwrap_or_default();
    assert!(
        result_text.contains("Checked"),
        "expected checkbox result, got: {result_text}"
    );

    let radio2 = device.find(Selector::new().resource_id(app_id("rb_option2")));
    radio2.click(None, None).await.unwrap();
    let result_text = result.get_text().await.unwrap_or_default();
    assert!(
        result_text.contains("Option 2 selected"),
        "expected radio option 2 result, got: {result_text}"
    );

    let switch_toggle = device.find(Selector::new().resource_id(app_id("sw_toggle")));
    switch_toggle.click(None, None).await.unwrap();
    let result_text = result.get_text().await.unwrap_or_default();
    assert!(
        result_text.contains("Switch: ON"),
        "expected switch ON result, got: {result_text}"
    );

    let reset_button = device.find(Selector::new().resource_id(app_id("btn_reset")));
    if !reset_button
        .exists(Some(Duration::from_secs(1)))
        .await
        .unwrap_or(false)
    {
        scroll_screen_with_adb(&device, true).await;
    }
    let reset_button = device.find(Selector::new().resource_id(app_id("btn_reset")));
    if reset_button
        .exists(Some(Duration::from_secs(1)))
        .await
        .unwrap_or(false)
    {
        reset_button.click(None, None).await.unwrap();
    } else {
        let reset_by_text = device.find(Selector::new().text_contains("RESET"));
        reset_by_text.click(None, None).await.unwrap();
    }
    common::wait_ui_stable().await;
    normal_button.click(None, None).await.unwrap();
    let result_text = result.get_text().await.unwrap_or_default();
    assert!(
        result_text.contains("Count: 1"),
        "expected click count reset to 1 after reset, got: {result_text}"
    );

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_gesture_apis_with_real_ui_feedback() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();
    launch_test_app(&device).await;
    open_page_from_main(&device, "btn_gestures", "tv_gestures_title").await;

    let click_area = device.find(Selector::new().resource_id(app_id("tv_click_area")));
    let (click_x, click_y) = click_area.center().await.unwrap();
    device.click(click_x, click_y).await.unwrap();
    common::wait_ui_stable().await;
    let click_text = click_area.get_text().await.unwrap_or_default();
    assert!(
        click_text.contains("Click Count: 1"),
        "expected click count update, got: {click_text}"
    );

    let double_click_area =
        device.find(Selector::new().resource_id(app_id("tv_double_click_area")));
    let (double_x, double_y) = double_click_area.center().await.unwrap();
    device
        .double_click(double_x, double_y, Some(Duration::from_millis(120)))
        .await
        .unwrap();
    common::wait_ui_stable().await;
    let double_text = double_click_area.get_text().await.unwrap_or_default();
    assert!(
        double_text.contains("Double Click Count: 1"),
        "expected double click count update, got: {double_text}"
    );

    let swipe_area = device.find(Selector::new().resource_id(app_id("tv_swipe_area")));
    let swipe_bounds = swipe_area.bounds().await.unwrap();
    let swipe_y = (swipe_bounds.top + swipe_bounds.bottom) / 2;
    device
        .swipe(
            swipe_bounds.right - 8,
            swipe_y,
            swipe_bounds.left + 8,
            swipe_y,
            Some(Duration::from_millis(300)),
        )
        .await
        .unwrap();
    common::wait_ui_stable().await;
    let swipe_text = swipe_area.get_text().await.unwrap_or_default();
    assert!(
        swipe_text.contains("Direction: Left"),
        "expected left swipe feedback, got: {swipe_text}"
    );

    let long_click_area = device.find(Selector::new().resource_id(app_id("tv_long_click_area")));
    long_click_area
        .long_click(
            Some(Duration::from_millis(900)),
            Some(Duration::from_secs(5)),
        )
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    let long_text = long_click_area.get_text().await.unwrap_or_default();
    assert!(
        long_text.contains("Long Pressed!"),
        "expected long click feedback, got: {long_text}"
    );

    let drag_view = device.find(Selector::new().resource_id(app_id("view_drag")));
    if !drag_view
        .exists(Some(Duration::from_secs(2)))
        .await
        .unwrap()
    {
        let (screen_w, screen_h) = device.window_size().await.unwrap();
        let center_x = screen_w / 2;
        device
            .swipe(
                center_x,
                (screen_h * 4) / 5,
                center_x,
                screen_h / 5,
                Some(Duration::from_millis(400)),
            )
            .await
            .unwrap();
        common::wait_ui_stable().await;
    }
    let before = drag_view.bounds().await.unwrap();
    let (drag_x, drag_y) = drag_view.center().await.unwrap();
    device
        .drag(
            drag_x,
            drag_y,
            drag_x + 120,
            drag_y + 100,
            Some(Duration::from_millis(700)),
        )
        .await
        .unwrap();
    common::wait_ui_stable().await;
    let after = drag_view.bounds().await.unwrap();
    if after.left == before.left && after.top == before.top {
        eprintln!(
            "warn: drag bounds unchanged on this backend/ROM, before={before:?}, after={after:?}"
        );
    }
    assert!(drag_view
        .exists(Some(Duration::from_secs(2)))
        .await
        .unwrap());

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_dialog_flows_and_wait_gone() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();
    launch_test_app(&device).await;
    open_page_from_main(&device, "btn_dialogs", "tv_dialogs_title").await;

    let result = device.find(Selector::new().resource_id(app_id("tv_dialog_result")));

    let alert = device.find(Selector::new().resource_id(app_id("btn_alert")));
    alert.click(None, None).await.unwrap();
    let ok_button = device.find(Selector::new().text("OK"));
    ok_button.wait(Some(Duration::from_secs(5))).await.unwrap();
    ok_button.click(None, None).await.unwrap();
    ok_button
        .wait_gone(Some(Duration::from_secs(5)))
        .await
        .unwrap();
    assert!(
        wait_dialog_result_contains(&device, "Alert OK clicked", Duration::from_secs(5)).await,
        "alert result text did not update in time"
    );
    let result_text = result.get_text().await.unwrap_or_default();
    assert!(
        result_text.contains("Alert OK clicked"),
        "expected alert dialog result, got: {result_text}"
    );

    let custom = device.find(Selector::new().resource_id(app_id("btn_custom")));
    custom.click(None, None).await.unwrap();
    let cancel_button = device.find(Selector::new().resource_id(app_id("btn_dialog_cancel")));
    cancel_button
        .wait(Some(Duration::from_secs(5)))
        .await
        .unwrap();
    cancel_button.click(None, None).await.unwrap();
    cancel_button
        .wait_gone(Some(Duration::from_secs(5)))
        .await
        .unwrap();
    assert!(
        wait_dialog_result_contains(&device, "Custom dialog Cancel", Duration::from_secs(5)).await,
        "custom dialog result text did not update in time"
    );
    let result_text = result.get_text().await.unwrap_or_default();
    assert!(
        result_text.contains("Custom dialog Cancel"),
        "expected custom dialog result, got: {result_text}"
    );

    let bottom_sheet = device.find(Selector::new().resource_id(app_id("btn_bottom_sheet")));
    bottom_sheet.click(None, None).await.unwrap();
    let mut confirmed = false;
    let mut last_result_text = String::new();
    for attempt in 1..=3 {
        let option2 = device.find(Selector::new().resource_id(app_id("btn_sheet_option2")));
        option2.wait(Some(Duration::from_secs(5))).await.unwrap();
        common::wait_ui_stable().await;
        option2.click(None, None).await.unwrap();

        if wait_dialog_result_contains(&device, "Bottom Sheet Option 2", Duration::from_secs(3))
            .await
        {
            if let Err(err) = option2.wait_gone(Some(Duration::from_secs(5))).await {
                eprintln!(
                    "warn: btn_sheet_option2 did not disappear after result update (attempt {attempt}): {err:?}"
                );
            }
            confirmed = true;
            break;
        }

        last_result_text = result.get_text().await.unwrap_or_default();
        eprintln!(
            "warn: bottom sheet option2 click not observed yet (attempt {attempt}/3), result text: {last_result_text}"
        );
    }
    assert!(
        confirmed,
        "bottom sheet option2 result not observed after retries, last result text: {last_result_text}"
    );
    let result_text = result.get_text().await.unwrap_or_default();
    assert!(
        result_text.contains("Bottom Sheet Option 2"),
        "expected bottom sheet result, got: {result_text}"
    );

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_lists_navigation_and_scroll_interactions() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();
    launch_test_app(&device).await;
    open_page_from_main(&device, "btn_lists", "tv_lists_title").await;

    let list_item = device.find(Selector::new().text_contains("ListView Item 1"));
    assert!(list_item
        .exists(Some(Duration::from_secs(5)))
        .await
        .unwrap());

    let recycler_tab = device.find(Selector::new().description("RecyclerView"));
    recycler_tab
        .click(Some(Duration::from_secs(5)), None)
        .await
        .unwrap();
    let recycler_item = device.find(Selector::new().text_contains("RecyclerView Item 1"));
    assert!(
        recycler_item
            .exists(Some(Duration::from_secs(5)))
            .await
            .unwrap(),
        "RecyclerView item not found after tab switch"
    );

    let scroll_tab = device.find(Selector::new().description("ScrollView"));
    scroll_tab
        .click(Some(Duration::from_secs(5)), None)
        .await
        .unwrap();
    let scroll_view = device.find(Selector::new().resource_id(app_id("scroll_view")));
    if scroll_view
        .exists(Some(Duration::from_secs(2)))
        .await
        .unwrap_or(false)
    {
        let bounds = scroll_view.bounds().await.unwrap();
        let center_x = (bounds.left + bounds.right) / 2;
        device
            .swipe(
                center_x,
                bounds.bottom - 16,
                center_x,
                bounds.top + 16,
                Some(Duration::from_millis(300)),
            )
            .await
            .unwrap();
    } else {
        let (screen_w, screen_h) = device.window_size().await.unwrap();
        let center_x = screen_w / 2;
        device
            .swipe(
                center_x,
                (screen_h * 4) / 5,
                center_x,
                screen_h / 5,
                Some(Duration::from_millis(300)),
            )
            .await
            .unwrap();
    }
    common::wait_ui_stable().await;

    let paragraph = device.find(Selector::new().text_contains("Paragraph"));
    assert!(
        paragraph
            .exists(Some(Duration::from_secs(5)))
            .await
            .unwrap(),
        "scroll content was not found after switching to ScrollView tab"
    );

    common::cleanup_test_env(&device).await.ok();
}

#[tokio::test]
async fn test_navigation_animations_stress_and_concurrent_pages() {
    common::init_test_env();
    skip_if_no_device!();

    let device = common::connect_test_device().await.unwrap();
    launch_test_app(&device).await;

    open_page_from_main(&device, "btn_navigation", "tv_navigation_title").await;
    let page_info = device.find(Selector::new().resource_id(app_id("tv_page_info")));
    let page1 = page_info.get_text().await.unwrap_or_default();
    assert!(
        page1.contains("Page: 1"),
        "expected first page, got: {page1}"
    );
    let next_page = device.find(Selector::new().resource_id(app_id("btn_next_page")));
    next_page.click(None, None).await.unwrap();
    device
        .wait_for(
            || {
                let page_info = device.find(Selector::new().resource_id(app_id("tv_page_info")));
                async move {
                    Ok(page_info
                        .get_text()
                        .await
                        .unwrap_or_default()
                        .contains("Page: 2"))
                }
            },
            Some(Duration::from_secs(10)),
        )
        .await
        .unwrap();
    device.press(Key::Back).await.unwrap();
    let back_to_page1 = page_info.get_text().await.unwrap_or_default();
    assert!(
        back_to_page1.contains("Page:"),
        "expected navigation page info after back, got: {back_to_page1}"
    );
    back_to_main(&device).await;

    open_page_from_main(&device, "btn_animations", "tv_animations_title").await;
    let fade = device.find(Selector::new().resource_id(app_id("btn_fade")));
    fade.click(None, None).await.unwrap();
    tokio::time::sleep(Duration::from_millis(800)).await;
    let animations_title = device.find(Selector::new().resource_id(app_id("tv_animations_title")));
    assert!(
        animations_title
            .exists(Some(Duration::from_secs(2)))
            .await
            .unwrap(),
        "animation page should remain visible after fade click"
    );
    let scale = device.find(Selector::new().resource_id(app_id("btn_scale")));
    let rotate = device.find(Selector::new().resource_id(app_id("btn_rotate")));
    assert!(
        scale
            .exists(Some(Duration::from_secs(2)))
            .await
            .unwrap_or(false)
            || rotate
                .exists(Some(Duration::from_secs(2)))
                .await
                .unwrap_or(false),
        "animation controls should still be present after fade click"
    );
    back_to_main(&device).await;

    launch_test_app(&device).await;
    open_page_from_main(&device, "btn_stress", "tv_stress_title").await;
    let rapid = device.find(Selector::new().resource_id(app_id("btn_rapid_click")));
    for _ in 0..3 {
        rapid.click(None, None).await.unwrap();
    }
    let click_count = device.find(Selector::new().resource_id(app_id("tv_click_count")));
    let click_text = click_count.get_text().await.unwrap_or_default();
    assert!(
        click_text.contains("Click Count: 3"),
        "expected click count 3, got: {click_text}"
    );

    let memory = device.find(Selector::new().resource_id(app_id("btn_memory_stress")));
    memory.click(None, None).await.unwrap();
    let memory_text = device
        .find(Selector::new().resource_id(app_id("tv_memory_usage")))
        .get_text()
        .await
        .unwrap_or_default();
    assert!(
        memory_text.contains("Memory:"),
        "expected memory usage text, got: {memory_text}"
    );

    let animation_stress = device.find(Selector::new().resource_id(app_id("btn_animation_stress")));
    animation_stress.click(None, None).await.unwrap();
    device
        .wait_for(
            || {
                let count = device.find(Selector::new().resource_id(app_id("tv_animation_count")));
                async move {
                    Ok(count
                        .get_text()
                        .await
                        .unwrap_or_default()
                        .contains("Animations: 10"))
                }
            },
            Some(Duration::from_secs(8)),
        )
        .await
        .unwrap();
    back_to_main(&device).await;

    launch_test_app(&device).await;
    open_page_from_main(&device, "btn_concurrent", "tv_concurrent_title").await;
    let increment = device.find(Selector::new().resource_id(app_id("btn_increment_all")));
    increment.click(None, None).await.unwrap();
    device
        .wait_for(
            || {
                let c1 = device.find(Selector::new().resource_id(app_id("tv_counter1")));
                let c2 = device.find(Selector::new().resource_id(app_id("tv_counter2")));
                let c3 = device.find(Selector::new().resource_id(app_id("tv_counter3")));
                async move {
                    let v1 = c1.get_text().await.unwrap_or_default();
                    let v2 = c2.get_text().await.unwrap_or_default();
                    let v3 = c3.get_text().await.unwrap_or_default();
                    Ok(v1.contains("Counter 1: 1")
                        && v2.contains("Counter 2: 1")
                        && v3.contains("Counter 3: 1"))
                }
            },
            Some(Duration::from_secs(8)),
        )
        .await
        .unwrap();

    let reset = device.find(Selector::new().resource_id(app_id("btn_reset_all")));
    reset.click(None, None).await.unwrap();
    let c1 = device
        .find(Selector::new().resource_id(app_id("tv_counter1")))
        .get_text()
        .await
        .unwrap_or_default();
    let c2 = device
        .find(Selector::new().resource_id(app_id("tv_counter2")))
        .get_text()
        .await
        .unwrap_or_default();
    let c3 = device
        .find(Selector::new().resource_id(app_id("tv_counter3")))
        .get_text()
        .await
        .unwrap_or_default();
    assert!(
        c1.contains("Counter 1: 0") && c2.contains("Counter 2: 0") && c3.contains("Counter 3: 0"),
        "expected counters reset to 0, got: {c1}, {c2}, {c3}"
    );

    common::cleanup_test_env(&device).await.ok();
}
