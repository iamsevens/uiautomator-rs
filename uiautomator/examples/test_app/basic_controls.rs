//! 基础控件测试
//!
//! 测试 BasicControlsActivity 中的所有控件

use crate::test_app::{click_by_id, click_menu_item, get_text_by_id, TestContext};
use std::time::Duration;
use uiautomator::Result;

pub async fn run_all_tests(ctx: &mut TestContext) -> Result<()> {
    println!("\n🎯 开始基础控件测试");

    // 进入基础控件页面
    click_menu_item(&ctx.device, "Basic Controls").await?;

    // 运行各个测试
    ctx.run_test("按钮点击测试", test_button_click).await;
    ctx.run_test("复选框测试", test_checkbox).await;
    ctx.run_test("单选按钮测试", test_radio_button).await;
    ctx.run_test("开关测试", test_switch).await;
    ctx.run_test("重置功能测试", test_reset).await;

    // 返回主页面
    ctx.go_home().await?;

    Ok(())
}

/// 测试按钮点击
fn test_button_click(
    device: &uiautomator::Device,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + '_>> {
    Box::pin(async move {
        // 点击普通按钮
        click_by_id(device, "btn_normal").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // 验证结果文本
        let result = get_text_by_id(device, "tv_result").await?;
        assert!(result.contains("Button clicked"), "按钮点击未生效");

        // 再次点击，验证计数增加
        click_by_id(device, "btn_normal").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let result = get_text_by_id(device, "tv_result").await?;
        assert!(result.contains("Count: 2"), "点击计数不正确");

        Ok(())
    })
}

/// 测试复选框
fn test_checkbox(
    device: &uiautomator::Device,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + '_>> {
    Box::pin(async move {
        // 点击复选框
        click_by_id(device, "cb_option").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // 验证结果
        let result = get_text_by_id(device, "tv_result").await?;
        assert!(result.contains("Checked"), "复选框选中状态不正确");

        // 再次点击，取消选中
        click_by_id(device, "cb_option").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let result = get_text_by_id(device, "tv_result").await?;
        assert!(result.contains("Unchecked"), "复选框取消选中状态不正确");

        Ok(())
    })
}

/// 测试单选按钮
fn test_radio_button(
    device: &uiautomator::Device,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + '_>> {
    Box::pin(async move {
        // 选择选项 1
        click_by_id(device, "rb_option1").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let result = get_text_by_id(device, "tv_result").await?;
        assert!(result.contains("Option 1"), "单选按钮选项 1 不正确");

        // 选择选项 2
        click_by_id(device, "rb_option2").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let result = get_text_by_id(device, "tv_result").await?;
        assert!(result.contains("Option 2"), "单选按钮选项 2 不正确");

        // 选择选项 3
        click_by_id(device, "rb_option3").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let result = get_text_by_id(device, "tv_result").await?;
        assert!(result.contains("Option 3"), "单选按钮选项 3 不正确");

        Ok(())
    })
}

/// 测试开关
fn test_switch(
    device: &uiautomator::Device,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + '_>> {
    Box::pin(async move {
        // 打开开关
        click_by_id(device, "sw_toggle").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let result = get_text_by_id(device, "tv_result").await?;
        assert!(result.contains("ON"), "开关打开状态不正确");

        // 关闭开关
        click_by_id(device, "sw_toggle").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let result = get_text_by_id(device, "tv_result").await?;
        assert!(result.contains("OFF"), "开关关闭状态不正确");

        Ok(())
    })
}

/// 测试重置功能
fn test_reset(
    device: &uiautomator::Device,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + '_>> {
    Box::pin(async move {
        // 先进行一些操作
        click_by_id(device, "btn_normal").await?;
        click_by_id(device, "cb_option").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // 点击重置
        click_by_id(device, "btn_reset").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // 验证重置后的状态
        let result = get_text_by_id(device, "tv_result").await?;
        assert!(result.contains("Reset"), "重置功能不正确");

        Ok(())
    })
}
