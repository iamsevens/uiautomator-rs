# API vs test-app coverage

Updated: 2026-02-26

This note records the static mapping between the public UI-facing APIs and
the scenarios provided by `test-app`.

## Matrix

| API area | Representative methods | test-app scenario | Coverage source | Status |
| --- | --- | --- | --- | --- |
| Main navigation | `UiObject::wait`, `UiObject::click`, `Device::press(Key::Back)` | 9 entry buttons in `MainActivity` | `integration_testapp_coverage_test.rs::test_all_main_entries_can_open_target_pages` | Covered |
| Basic controls | `UiObject::click`, `UiObject::get_text` | `BasicControlsActivity` | `integration_testapp_coverage_test.rs::test_basic_controls_interactions` | Covered |
| Gesture APIs | `Device::click`, `Device::double_click`, `Device::swipe`, `Device::drag`, `UiObject::long_click`, `UiObject::bounds`, `UiObject::center` | `GesturesActivity` | `integration_testapp_coverage_test.rs::test_gesture_apis_with_real_ui_feedback` | Covered |
| Dialog lifecycle | `UiObject::wait`, `UiObject::click`, `UiObject::wait_gone`, `UiObject::get_text` | `DialogsActivity` alert/custom/bottom-sheet | `integration_testapp_coverage_test.rs::test_dialog_flows_and_wait_gone` | Covered |
| Lists and scroll | `UiObject::exists`, `UiObject::bounds`, `Device::swipe` | `ListsActivity` tabs | `integration_testapp_coverage_test.rs::test_lists_navigation_and_scroll_interactions` | Covered |
| Navigation/animation/stress/concurrency | `Device::wait_for`, `UiObject::click`, `UiObject::get_text` | `NavigationActivity`, `AnimationsActivity`, `StressTestActivity`, `ConcurrentTestActivity` | `integration_testapp_coverage_test.rs::test_navigation_animations_stress_and_concurrent_pages` | Covered |
| Input/forms | `UiObject::set_text`, `UiObject::clear_text`, `UiObject::get_text`, `UiObject::click` | `InputFormsActivity` | `integration_element_test.rs` existing cases | Covered |
| App lifecycle + screenshot | `Device::app_start`, `Device::app_stop`, `Device::app_current`, `Device::app_wait`, `Device::app_clear`, `Device::screenshot`, `Device::screenshot_to_file` | App process lifecycle | `integration_app_test.rs` existing cases | Covered |

## Gap result

- No missing `test-app` resource IDs were found for IDs referenced by integration tests.
- All 9 main entry pages now have integration coverage.
- No additional APK UI capability was required in this round.

## Out of scope for test-app UI scenes

These are environment/infrastructure APIs and should be validated by dedicated
device/installer tests, not by `test-app` UI pages:

- `Device::connect`, `Device::connect_quick`, `Device::connect_with_mode`
- `Device::install_atx_agent`, `Device::check_atx_agent_installed`
- `Device::start_atx_agent`, `Device::stop_atx_agent`, `Device::restart_atx_agent`
