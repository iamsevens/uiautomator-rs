//! UiObject 使用示例
//!
//! 展示如何使用 UiObject 进行 UI 元素操作
//!
//! 注意: 此示例需要在 Device 实现后运行

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志
    uiautomator::init_logger();

    println!("UiObject 使用示例");
    println!("注意: 此示例需要在 Device 实现后运行\n");

    // 连接到设备(待实现)
    // let device = Device::connect(None).await?;

    // 示例 1: 检查元素是否存在
    println!("示例 1: 检查元素是否存在");
    // let settings_button = device.find(Selector::new().text("Settings"));
    // if settings_button.exists(Some(Duration::from_secs(5))).await? {
    //     println!("找到设置按钮");
    // } else {
    //     println!("未找到设置按钮");
    // }

    // 示例 2: 等待元素出现并点击
    println!("\n示例 2: 等待元素出现并点击");
    // let submit_button = device.find(Selector::new()
    //     .text("Submit")
    //     .clickable(true));
    // submit_button.wait(Some(Duration::from_secs(10))).await?;
    // submit_button.click(None, None).await?;
    // println!("提交按钮已点击");

    // 示例 3: 获取元素信息
    println!("\n示例 3: 获取元素信息");
    // let title = device.find(Selector::new().resource_id("com.example:id/title"));
    // let info = title.info().await?;
    // println!("元素文本: {}", info.text);
    // println!("元素类名: {}", info.class_name);
    // println!("是否可点击: {}", info.clickable);
    // println!("元素边界: {:?}", info.bounds);

    // 示例 4: 输入文本
    println!("\n示例 4: 输入文本");
    // let input_field = device.find(Selector::new()
    //     .resource_id("com.example:id/username")
    //     .class_name("android.widget.EditText"));
    // input_field.clear_text().await?;
    // input_field.set_text("test_user").await?;
    // println!("文本已输入");

    // 示例 5: 长按元素
    println!("\n示例 5: 长按元素");
    // let item = device.find(Selector::new().text("Item 1"));
    // item.long_click(Some(Duration::from_secs(1)), None).await?;
    // println!("元素已长按");

    // 示例 6: 条件点击
    println!("\n示例 6: 条件点击");
    // let skip_button = device.find(Selector::new().text("Skip"));
    // if skip_button.click_exists(Some(Duration::from_secs(3))).await? {
    //     println!("跳过按钮已点击");
    // } else {
    //     println!("跳过按钮不存在, 跳过执行");
    // }

    // 示例 7: 等待元素消失
    println!("\n示例 7: 等待元素消失");
    // let loading = device.find(Selector::new().text("Loading..."));
    // loading.wait_gone(Some(Duration::from_secs(30))).await?;
    // println!("加载完成");

    // 示例 8: 带偏移的点击
    println!("\n示例 8: 带偏移的点击");
    // let image = device.find(Selector::new().resource_id("com.example:id/image"));
    // // 点击图片的右下角
    // image.click(None, Some((0.9, 0.9))).await?;
    // println!("图片右下角已点击");

    // 示例 9: 获取元素中心坐标
    println!("\n示例 9: 获取元素中心坐标");
    // let button = device.find(Selector::new().text("Button"));
    // let (x, y) = button.center().await?;
    // println!("按钮中心坐标: ({}, {})", x, y);

    // 示例 10: 搜索流程
    println!("\n示例 10: 搜索流程");
    // let search_box = device.find(Selector::new()
    //     .resource_id("com.example:id/search")
    //     .class_name("android.widget.EditText"));
    //
    // // 等待搜索框出现
    // search_box.wait(Some(Duration::from_secs(5))).await?;
    //
    // // 清空并输入文本
    // search_box.clear_text().await?;
    // search_box.set_text("Rust").await?;
    //
    // // 点击搜索按钮
    // let search_button = device.find(Selector::new()
    //     .resource_id("com.example:id/search_button"));
    // search_button.click(None, None).await?;
    //
    // println!("搜索完成");

    println!("\n所有示例代码已准备就绪, 在 Device 实现后即可运行!");

    Ok(())
}
