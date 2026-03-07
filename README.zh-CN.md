[English](./README.md) | [简体中文](./README.zh-CN.md)

# uiautomator-rs

[![crates.io - uiautomator](https://img.shields.io/crates/v/uiautomator.svg)](https://crates.io/crates/uiautomator)
[![docs.rs - uiautomator](https://docs.rs/uiautomator/badge.svg)](https://docs.rs/uiautomator)
[![crates.io - uiautomator-cli](https://img.shields.io/crates/v/uiautomator-cli.svg)](https://crates.io/crates/uiautomator-cli)
[![docs.rs - uiautomator-cli](https://docs.rs/uiautomator-cli/badge.svg)](https://docs.rs/uiautomator-cli)

使用 Rust 实现的 Android UI 自动化测试库，复刻 Python uiautomator2 的核心功能。

## 特性

- 🚀 **高性能**: 基于 Rust 和 Tokio 异步运行时，提供卓越的执行效率
- 🔒 **类型安全**: 利用 Rust 的类型系统在编译时捕获错误
- 🔄 **异步支持**: 完整的异步 API，支持高并发场景下同时控制多个设备
- 📱 **功能完整**: 支持元素定位、手势操作、应用管理、截图等完整功能
- 🛡️ **错误恢复**: 自动重试和服务恢复机制，提高测试稳定性
- 📝 **完整日志**: 内置日志系统，方便调试和问题排查
- 🎯 **API 兼容**: 与 Python uiautomator2 保持相似的 API 设计，降低迁移成本

## 项目状态

✅ **核心功能已完成** - 可用于生产环境

### 已完成功能

#### 核心功能
- ✅ 设备连接和管理（支持多设备）
- ✅ 设备信息获取（屏幕尺寸、旋转、SDK 版本等）
- ✅ UI 元素定位（支持多种选择器条件）
- ✅ UI 元素操作（点击、长按、文本输入等）
- ✅ 元素等待机制（等待出现/消失）

#### 扩展选择器
- ✅ 布尔属性：checkable, checked, long_clickable, focused, selected
- ✅ 正则匹配：class_name_matches, description_matches, resource_id_matches 等
- ✅ index：元素在父容器中的位置（区别于 instance）
- ✅ 层级选择器：child / sibling

#### 手势操作
- ✅ 坐标点击、长按、双击
- ✅ 滑动和拖拽操作
- ✅ 百分比坐标支持

#### 按键操作
- ✅ 物理按键模拟（Home、Back、Power 等）
- ✅ 方向键和音量键
- ✅ 自定义键码支持

#### 应用管理
- ✅ 应用启动和停止
- ✅ 应用等待机制
- ✅ 获取当前应用信息
- ✅ 清除应用数据

#### 截图功能
- ✅ 屏幕截图
- ✅ 保存为 PNG/JPEG 格式
- ✅ 元素截图

#### 高级特性
- ✅ 异步 API 设计
- ✅ 并发安全（支持多设备并发操作）
- ✅ 自动重试和错误恢复
- ✅ 灵活的超时配置
- ✅ 完整的错误处理

### 测试覆盖

- ✅ 52+ 单元测试（100% 通过率）
- ✅ 集成测试（设备连接、元素操作、手势、应用管理等）
- ✅ 并发安全测试
- ✅ 资源清理测试

### 进行中

- ✅ ATX-Agent 连接模式（提供生产级稳定性）
- ✅ ATX-Agent 安装模式（可选，不依赖 Python）
- 🔄 更多示例和文档

### 平台兼容性

✅ 支持 Windows、Linux、macOS 平台。

**支持的开发环境**:
- ✅ Windows
- ✅ Linux (Ubuntu, Debian, Fedora 等)
- ✅ macOS

---

## 快速开始

### 前置要求

1. **Android 设备或模拟器**
   - 已启用 USB 调试
   - 通过 ADB 可访问

2. **ADB 工具**
   ```bash
   # 验证 ADB 是否可用
   adb devices
   ```

3. **设备端服务**
   - 设备上需要安装 UiAutomator2 服务
   - 首次连接时会自动安装（需要约 5-10 秒）

### 安装

将以下内容添加到 `Cargo.toml`:

```toml
[dependencies]
uiautomator = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

### 基础使用

```rust
use uiautomator::{Device, Selector, Key};
use std::time::Duration;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志系统
    uiautomator::init_logger();

    // 连接到设备（自动选择唯一设备）
    let device = Device::connect(None).await?;

    // 获取设备信息
    let info = device.info().await?;
    println!("设备: {}x{}", info.display_width, info.display_height);
    println!("Android SDK: {}", info.sdk_int);

    // 点击坐标
    device.click(100, 200).await?;

    // 按下 Home 键
    device.press(Key::Home).await?;

    Ok(())
}
```

---

## API 参考

### Device API

`Device` 是所有操作的入口点，用于与 Android 设备进行交互。

#### 连接设备

| 方法 | 说明 |
|------|------|
| `Device::connect(serial)` | 自动检测模式连接设备 |
| `Device::connect_quick(serial)` | 快速连接（Direct 模式） |
| `Device::connect_with_mode(serial, mode)` | 使用指定模式连接 |
| `Device::connect_with_rpc_url(serial, url)` | 使用自定义 RPC URL 连接 |

```rust
// 自动连接（唯一设备）
let device = Device::connect(None).await?;

// 指定设备序列号
let device = Device::connect(Some("emulator-5554")).await?;

// 快速连接（仅用于测试）
let device = Device::connect_quick(None).await?;

// 使用 ATX-Agent 模式
use uiautomator::ServerMode;
let device = Device::connect_with_mode(None, ServerMode::AtxAgent).await?;
```

#### 设备信息

| 方法 | 返回类型 | 说明 |
|------|----------|------|
| `info()` | `DeviceInfo` | 获取设备完整信息 |
| `window_size()` | `(u32, u32)` | 获取窗口尺寸 |
| `pos_rel2abs(x, y)` | `(u32, u32)` | 百分比坐标转换为绝对坐标 |
| `serial()` | `&str` | 获取设备序列号 |

```rust
let info = device.info().await?;
println!("分辨率: {}x{}", info.display_width, info.display_height);
println!("SDK 版本: {}", info.sdk_int);
println!("当前应用: {}", info.current_package_name);
println!("屏幕旋转: {}", info.display_rotation);

// 窗口尺寸
let (width, height) = device.window_size().await?;

// 百分比坐标转换
let (abs_x, abs_y) = device.pos_rel2abs(0.5, 0.5).await?;
```

#### 坐标手势操作

| 方法 | 说明 |
|------|------|
| `click(x, y)` | 点击指定坐标 |
| `long_click(x, y, duration)` | 长按指定坐标 |
| `double_click(x, y, duration)` | 双击指定坐标 |
| `swipe(x1, y1, x2, y2, duration)` | 滑动 |
| `drag(x1, y1, x2, y2, duration)` | 拖拽 |
| `click_percent(x, y)` | 使用百分比坐标点击 |

```rust
// 点击
device.click(500, 1000).await?;

// 长按（默认 0.5 秒）
device.long_click(500, 1000, None).await?;

// 长按 2 秒
use std::time::Duration;
device.long_click(500, 1000, Some(Duration::from_secs(2))).await?;

// 双击
device.double_click(500, 1000, None).await?;

// 滑动（从上到下）
device.swipe(540, 500, 540, 1000, Some(Duration::from_secs_f32(0.5))).await?;

// 拖拽
device.drag(200, 300, 600, 800, Some(Duration::from_secs(1))).await?;

// 百分比坐标点击屏幕中心
device.click_percent(0.5, 0.5).await?;
```

#### 按键操作

| 方法 | 说明 |
|------|------|
| `press(key)` | 按下预定义按键 |
| `press_keycode(keycode)` | 按下指定键码 |

```rust
use uiautomator::Key;

// 系统按键
device.press(Key::Home).await?;
device.press(Key::Back).await?;
device.press(Key::Power).await?;
device.press(Key::Recent).await?;

// 音量键
device.press(Key::VolumeUp).await?;
device.press(Key::VolumeDown).await?;
device.press(Key::VolumeMute).await?;

// 方向键
device.press(Key::Up).await?;
device.press(Key::Down).await?;
device.press(Key::Left).await?;
device.press(Key::Right).await?;
device.press(Key::Center).await?;

// 媒体键
device.press(Key::MediaPlayPause).await?;
device.press(Key::MediaNext).await?;

// 使用键码直接按键
device.press_keycode(3).await?; // Home 键的键码是 3
```

#### 应用管理

| 方法 | 说明 |
|------|------|
| `app_start(package, activity)` | 启动应用 |
| `app_stop(package)` | 停止应用 |
| `app_clear(package)` | 清除应用数据 |
| `app_current()` | 获取当前前台应用信息 |
| `app_wait(package, timeout)` | 等待应用启动 |

```rust
// 仅指定包名启动
device.app_start("com.android.settings", None).await?;

// 指定包名和 Activity 启动
device.app_start("com.android.settings", Some(".Settings")).await?;

// 等待应用启动（最多 10 秒）
let pid = device
    .app_wait("com.android.settings", Some(Duration::from_secs(10)))
    .await?;
println!("应用 PID: {}", pid);

// 获取当前前台应用
use uiautomator::models::AppInfo;
let app_info = device.app_current().await?;
println!("当前应用: {} / {}", app_info.package, app_info.activity);

// 停止应用
device.app_stop("com.android.settings").await?;

// 清除应用数据
device.app_clear("com.android.settings").await?;
```

#### 截图操作

| 方法 | 返回类型 | 说明 |
|------|----------|------|
| `screenshot()` | `DynamicImage` | 获取屏幕截图 |
| `screenshot_to_file(path)` | `()` | 保存截图到文件 |

```rust
use image::DynamicImage;

// 获取图像数据
let image = device.screenshot().await?;
println!("截图尺寸: {}x{}", image.width(), image.height());

// 保存为 PNG
device.screenshot_to_file("screenshot.png").await?;

// 保存为 JPEG
use std::fs::File;
use std::io::BufWriter;
let image = device.screenshot().await?;
let file = File::create("screenshot.jpg")?;
let writer = BufWriter::new(file);
image.write_to(writer, image::ImageOutputFormat::Jpeg(80))?;
```

#### 查找元素

| 方法 | 返回类型 | 说明 |
|------|----------|------|
| `find(selector)` | `UiObject` | 根据选择器查找元素 |

```rust
use uiautomator::Selector;

// 查找元素
let element = device.find(Selector::new().text("设置"));

// 立即使用
element.click(None, None).await?;
```

#### 等待条件

| 方法 | 说明 |
|------|------|
| `wait_for(condition, timeout)` | 等待自定义条件满足 |

```rust
use std::time::Duration;

// 等待元素出现
let element = device.find(Selector::new().text("加载完成"));
element.wait(Some(Duration::from_secs(10))).await?;

// 自定义等待条件
device.wait_for(
    || async { device.info().await.map(|_| true) },
    Some(Duration::from_secs(5)),
).await?;
```

#### 配置选项

| 方法 | 说明 |
|------|------|
| `set_wait_timeout(duration)` | 设置默认等待超时 |
| `get_wait_timeout()` | 获取当前等待超时 |
| `get_polling_interval()` | 获取轮询间隔 |
| `settings()` | 获取完整配置对象 |

```rust
use std::time::Duration;

// 设置全局等待超时
device.set_wait_timeout(Duration::from_secs(30));

// 获取当前超时设置
let timeout = device.get_wait_timeout();
println!("当前超时: {:?}", timeout);

// 获取轮询间隔
let interval = device.get_polling_interval();
```

#### ATX-Agent 操作

| 方法 | 说明 |
|------|------|
| `install_atx_agent(force)` | 安装 ATX-Agent |
| `check_atx_agent_installed()` | 检查 ATX-Agent 是否已安装 |
| `start_atx_agent()` | 启动 ATX-Agent |
| `stop_atx_agent()` | 停止 ATX-Agent |
| `restart_atx_agent()` | 重启 ATX-Agent |

```rust
// 检查 ATX-Agent 是否已安装
if !device.check_atx_agent_installed().await? {
    // 安装 ATX-Agent
    device.install_atx_agent(false).await?;
}

// 启动 ATX-Agent
device.start_atx_agent().await?;

// 停止 ATX-Agent
device.stop_atx_agent().await?;

// 重启 ATX-Agent
device.restart_atx_agent().await?;
```

---

### UiObject API

`UiObject` 代表一个 UI 元素，提供元素的定位、查询和操作功能。

#### 元素属性查询

| 方法 | 返回类型 | 说明 |
|------|----------|------|
| `exists(timeout)` | `bool` | 检查元素是否存在 |
| `info()` | `ElementInfo` | 获取元素完整信息 |
| `get_text()` | `String` | 获取元素文本 |
| `center()` | `(u32, u32)` | 获取元素中心坐标 |
| `bounds()` | `Rect` | 获取元素边界 |
| `selector()` | `&Selector` | 获取元素选择器 |

```rust
use std::time::Duration;

let element = device.find(Selector::new().text("设置"));

// 检查元素是否存在
if element.exists(Some(Duration::from_secs(5))).await? {
    println!("元素存在");
}

// 获取元素信息
let info = element.info().await?;
println!("文本: {}", info.text);
println!("类名: {}", info.class_name);
println!("资源 ID: {}", info.resource_id);
println!("包名: {}", info.package_name);
println!("可点击: {}", info.clickable);
println!("已启用: {}", info.enabled);
println!("子元素数: {}", info.child_count);

// 获取文本
let text = element.get_text().await?;
println!("元素文本: {}", text);

// 获取中心坐标
let (cx, cy) = element.center().await?;
println!("中心坐标: ({}, {})", cx, cy);

// 获取边界
let bounds = element.bounds().await?;
println!("边界: ({}, {})-({}, {})", bounds.left, bounds.top, bounds.right, bounds.bottom);
println!("尺寸: {}x{}", bounds.width(), bounds.height());

// 获取选择器
let selector = element.selector();
```

#### 元素等待

| 方法 | 说明 |
|------|------|
| `wait(timeout)` | 等待元素出现 |
| `wait_gone(timeout)` | 等待元素消失 |

```rust
use std::time::Duration;

let element = device.find(Selector::new().text("加载中"));

// 等待元素出现（最多 10 秒）
element.wait(Some(Duration::from_secs(10))).await?;
println!("元素已出现");

// 等待元素消失
element.wait_gone(Some(Duration::from_secs(10))).await?;
println!("元素已消失");
```

#### 元素点击操作

| 方法 | 说明 |
|------|------|
| `click(timeout, interval)` | 点击元素 |
| `click_exists(timeout)` | 尝试点击，返回是否成功 |

```rust
use std::time::Duration;

let element = device.find(Selector::new().text("设置"));

// 点击元素（使用默认超时和间隔）
element.click(None, None).await?;

// 自定义超时和轮询间隔
element.click(
    Some(Duration::from_secs(10)),
    Some(Duration::from_millis(200))
).await?;

// 尝试点击，返回是否成功
let clicked = element.click_exists(Some(Duration::from_secs(5))).await?;
if clicked {
    println!("点击成功");
} else {
    println!("元素不存在");
}
```

#### 元素手势操作

| 方法 | 说明 |
|------|------|
| `long_click(duration, timeout)` | 长按元素 |

```rust
use std::time::Duration;

let element = device.find(Selector::new().text("长按我"));

// 长按（使用默认持续时间和超时）
element.long_click(None, None).await?;

// 长按 2 秒
element.long_click(
    Some(Duration::from_secs(2)),
    None,
).await?;
```

#### 文本操作

| 方法 | 说明 |
|------|------|
| `set_text(text)` | 设置元素文本 |
| `clear_text()` | 清除元素文本 |

```rust
let element = device.find(Selector::new().class_name("android.widget.EditText"));

// 设置文本
element.set_text("Hello, World!").await?;

// 清除文本
element.clear_text().await?;

// 先清除再输入新文本
element.clear_text().await?;
element.set_text("新的内容").await?;
```

---

### Selector API

`Selector` 用于定位 UI 元素，支持多种选择条件。

#### 基本定位

| 方法 | 说明 |
|------|------|
| `text(text)` | 精确匹配文本 |
| `text_contains(text)` | 文本包含 |
| `text_starts_with(text)` | 文本开头匹配 |
| `text_matches(pattern)` | 文本正则匹配 |
| `resource_id(id)` | 匹配资源 ID |
| `class_name(name)` | 匹配类名 |
| `description(desc)` | 匹配内容描述 |
| `package_name(name)` | 匹配包名 |

```rust
use uiautomator::Selector;

// 精确文本
Selector::new().text("设置");

// 文本包含
Selector::new().text_contains("Wifi");

// 文本开头
Selector::new().text_starts_with("欢迎使用");

// 文本正则匹配
Selector::new().text_matches("^第.*章$");

// 资源 ID
Selector::new().resource_id("com.android.settings:id/search");

// 类名
Selector::new().class_name("android.widget.Button");

// 内容描述
Selector::new().description("Clear");

// 包名
Selector::new().package_name("com.android.settings");

// 描述包含
Selector::new().description_contains("menu");
```

#### 布尔属性

| 方法 | 说明 |
|------|------|
| `clickable(value)` | 是否可点击 |
| `enabled(value)` | 是否启用 |
| `focusable(value)` | 是否可获得焦点 |
| `scrollable(value)` | 是否可滚动 |
| `checkable(value)` | 是否可勾选 |
| `checked(value)` | 是否已勾选 |
| `long_clickable(value)` | 是否可长按 |
| `focused(value)` | 是否已获得焦点 |
| `selected(value)` | 是否已选中 |

```rust
// 可点击的按钮
Selector::new()
    .class_name("android.widget.Button")
    .clickable(true);

// 已启用的输入框
Selector::new()
    .class_name("android.widget.EditText")
    .enabled(true);

// 可滚动的列表
Selector::new()
    .class_name("android.widget.ListView")
    .scrollable(true);

// 已选中的选项
Selector::new()
    .class_name("android.widget.RadioButton")
    .checked(true);

// 已获得焦点的元素
Selector::new().focused(true);
```

#### 正则匹配

| 方法 | 说明 |
|------|------|
| `class_name_matches(pattern)` | 类名正则匹配 |
| `description_matches(pattern)` | 描述正则匹配 |
| `resource_id_matches(pattern)` | 资源 ID 正则匹配 |
| `package_name_matches(pattern)` | 包名正则匹配 |

```rust
// 类名以 Button 结尾
Selector::new().class_name_matches(".*Button$");

// 资源 ID 包含特定模式
Selector::new().resource_id_matches(".*:id/button_\\d+");

// 描述以 menu 开头
Selector::new().description_matches("^menu");

// 包名匹配特定公司
Selector::new().package_name_matches("com\\.company\\..*");
```

#### 位置和索引

| 方法 | 说明 |
|------|------|
| `instance(value)` | 第几个匹配的元素实例 |
| `index(value)` | 元素在父容器中的位置 |

```rust
// 第三个匹配的按钮
Selector::new()
    .class_name("android.widget.Button")
    .instance(2); // 索引从 0 开始，2 表示第三个

// 列表的第二个子项
Selector::new()
    .class_name("android.widget.TextView")
    .index(1);
```

#### 层级选择器

| 方法 | 说明 |
|------|------|
| `child(selector)` | 子元素选择器 |
| `sibling(selector)` | 兄弟元素选择器 |

```rust
// 列表中包含特定文本的子元素
Selector::new()
    .resource_id("com.example:id/list")
    .child(Selector::new().text("Title"));

// 文本为 "Label" 旁边的输入框
Selector::new()
    .text("Label")
    .sibling(Selector::new().class_name("android.widget.EditText"));

// 复杂组合：列表中索引为 2、包含 "Active" 文本的按钮
Selector::new()
    .resource_id("list")
    .child(
        Selector::new()
            .class_name("android.widget.Button")
            .index(2)
            .text("Active")
    );
```

#### 组合条件

选择器方法可以链式调用，实现组合条件：

```rust
// 组合多个条件
Selector::new()
    .text("确定")
    .class_name("android.widget.Button")
    .clickable(true)
    .enabled(true);

// 复杂组合
Selector::new()
    .class_name_matches(".*EditText$")
    .enabled(true)
    .focusable(true);
```

---

### Key API

`Key` 枚举定义了 Android 系统的常用按键。

#### 系统按键

| 按键 | 说明 |
|------|------|
| `Key::Home` | Home 键 - 返回主屏幕 |
| `Key::Back` | Back 键 - 返回上一个界面 |
| `Key::Power` | 电源键 - 切换屏幕点亮状态 |
| `Key::Recent` | 最近任务键 |
| `Key::Menu` | 菜单键 |
| `Key::Search` | 搜索键 |

#### 音量键

| 按键 | 说明 |
|------|------|
| `Key::VolumeUp` | 音量增加键 |
| `Key::VolumeDown` | 音量减少键 |
| `Key::VolumeMute` | 静音键 |

#### 方向键

| 按键 | 说明 |
|------|------|
| `Key::Up` | 方向键 - 上 |
| `Key::Down` | 方向键 - 下 |
| `Key::Left` | 方向键 - 左 |
| `Key::Right` | 方向键 - 右 |
| `Key::Center` | 方向键 - 中心/确认 |

#### 功能键

| 按键 | 说明 |
|------|------|
| `Key::Enter` | 回车键/确认键 |
| `Key::Delete` | 删除键 |
| `Key::Tab` | Tab 键 |
| `Key::Space` | 空格键 |
| `Key::Escape` | 退出键 |

#### 媒体键

| 按键 | 说明 |
|------|------|
| `Key::MediaPlayPause` | 播放/暂停键 |
| `Key::MediaStop` | 停止键 |
| `Key::MediaNext` | 下一曲 |
| `Key::MediaPrevious` | 上一曲 |
| `Key::MediaFastForward` | 快进 |
| `Key::MediaRewind` | 快退 |

#### 其他按键

| 按键 | 说明 |
|------|------|
| `Key::Camera` | 相机键 |
| `Key::Call` | 通话键 |
| `Key::EndCall` | 挂断键 |

#### 按键转换

```rust
use uiautomator::Key;

// 获取按键对应的键码
let keycode = Key::Home.to_keycode();
println!("Home 键码: {}", keycode); // 输出: 3

// 获取按键名称
let name = Key::Home.to_name();
println!("Home 名称: {}", name); // 输出: "HOME"

// 通过名称获取按键
if let Some(key) = Key::from_name("HOME") {
    println!("找到按键: {:?}", key);
}
```

---

### Settings API

`Settings` 用于配置设备连接的行为参数。

#### 配置项

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `wait_timeout` | 等待超时时间 | 20 秒 |
| `operation_delay_before` | 操作前延迟 | 0 秒 |
| `operation_delay_after` | 操作后延迟 | 0 秒 |
| `http_timeout` | HTTP 请求超时 | 30 秒 |
| `max_retry` | 最大重试次数 | 3 |
| `polling_interval` | 轮询间隔 | 0.5 秒 |
| `retry_base_delay` | 重试基础延迟 | 1 秒 |

#### 使用方法

```rust
use std::time::Duration;

// 创建默认配置
let settings = uiautomator::Settings::default();

// 设置等待超时
settings.write().await.set_wait_timeout(Duration::from_secs(30));

// 设置操作延迟
settings.write().await.set_operation_delay_before(Duration::from_millis(100));
settings.write().await.set_operation_delay_after(Duration::from_millis(200));

// 设置 HTTP 超时
settings.write().await.set_http_timeout(Duration::from_secs(60));

// 设置最大重试次数
settings.write().await.set_max_retry(5);

// 设置轮询间隔
settings.write().await.set_polling_interval(Duration::from_millis(250));

// 设置重试基础延迟
settings.write().await.set_retry_base_delay(Duration::from_secs(2));
```

#### 使用 Device 设置

```rust
use std::time::Duration;

let device = Device::connect(None).await?;

// 设置等待超时
device.set_wait_timeout(Duration::from_secs(30));

// 使用 Builder 风格设置
let settings = device.settings();
settings.write().await
    .with_wait_timeout(Duration::from_secs(30))
    .with_operation_delay_before(Duration::from_millis(100))
    .with_operation_delay_after(Duration::from_millis(200))
    .with_http_timeout(Duration::from_secs(60))
    .with_max_retry(5)
    .with_polling_interval(Duration::from_millis(250))
    .with_retry_base_delay(Duration::from_secs(2));
```

---

### Error API

库使用 Rust 的 `Result` 类型进行错误处理，所有可能失败的操作都返回 `Result<T>`。

#### 错误类型

| 错误类型 | 说明 | 错误码 |
|---------|------|--------|
| `DeviceNotFound` | 未找到设备 | 1001 |
| `MultipleDevicesFound` | 发现多个设备 | 1002 |
| `DeviceConnection` | 设备连接失败 | 1003 |
| `DeviceOffline` | 设备离线 | 1004 |
| `ElementNotFound` | 元素未找到 | 2001 |
| `ElementTimeout` | 元素查找超时 | 2002 |
| `AppNotInstalled` | 应用未安装 | 3001 |
| `AppNotRunning` | 应用未运行 | 3002 |
| `AppCrashed` | 应用崩溃 | 3003 |
| `AppStartFailed` | 应用启动失败 | 3004 |
| `UiAutomatorNotConnected` | UiAutomator 服务未连接 | 4000 |
| `JsonRpc` | JSON-RPC 调用失败 | 4001 |
| `JsonRpcParse` | JSON-RPC 解析失败 | 4002 |
| `JsonRpcCode` | JSON-RPC 错误码 | 4003 |
| `Http` | HTTP 请求失败 | 4100 |
| `HttpTimeout` | HTTP 请求超时 | 4101 |
| `Adb` | ADB 命令失败 | 4102 |
| `Image` | 图像处理失败 | 4200 |
| `Timeout` | 操作超时 | 5000 |
| `AtxAgent` | ATX-Agent 错误 | 6000 |

#### 错误处理示例

```rust
use uiautomator::{Device, Error};
use std::time::Duration;

#[tokio::main]
async fn main() {
    // 简单的错误处理
    match Device::connect(None).await {
        Ok(device) => println!("设备已连接"),
        Err(Error::DeviceNotFound) => eprintln!("未找到设备"),
        Err(Error::MultipleDevicesFound) => eprintln!("发现多个设备，请指定序列号"),
        Err(e) => eprintln!("连接失败: {}", e),
    }

    // 使用 ? 操作符传播错误
    let device = Device::connect(None).await.unwrap_or_else(|e| {
        eprintln!("连接失败: {}", e);
        std::process::exit(1);
    });

    // 处理元素查找错误
    match device.find(Selector::new().text("登录")).click(None, None).await {
        Ok(_) => println!("点击成功"),
        Err(Error::ElementNotFound { selector }) => eprintln!("元素未找到: {}", selector),
        Err(e) => eprintln!("点击失败: {}", e),
    }
}

// 返回 Result 的函数示例
async fn click_button(device: &Device, text: &str) -> Result<(), uiautomator::Error> {
    let element = device.find(Selector::new().text(text));
    element.wait(Some(Duration::from_secs(10))).await?;
    element.click(None, None).await?;
    Ok(())
}
```

#### 错误码和类别

```rust
use uiautomator::Error;

// 获取错误码
let err = Error::DeviceNotFound;
println!("错误码: {}", err.code()); // 输出: 1001

// 获取错误类别
let err = Error::ElementNotFound { selector: "test".to_string() };
println!("错误类别: {}", err.category()); // 输出: Element

// 错误类别包括：
// - Device: 设备相关错误
// - Element: 元素相关错误
// - Application: 应用相关错误
// - Network: 网络相关错误
// - Image: 图像处理错误
// - General: 通用错误
```

#### 日志记录错误

```rust
use log::error;

fn log_error(error: &uiautomator::Error) {
    error!("操作失败 [{}:{}]: {}", error.code(), error.category(), error);
}
```

#### 自动重试

库内置了自动重试机制：
- HTTP 超时：自动重试最多 3 次
- 服务崩溃：自动重启服务并重试
- 网络错误：自动重试

```rust
// 无需手动处理重试，库会自动处理
device.click(100, 200).await?; // 失败时自动重试
```

---

## 服务模式

支持两种设备端服务模式：

### Direct 模式（快速测试）

- ✅ 快速启动（5-10 秒）
- ✅ 简单部署（只需 u2.jar）
- ⚠️ 稳定性较低（可能被系统杀死）
- 适合：开发测试、短时间运行

```rust
use uiautomator::ServerMode;

// 显式使用 Direct 模式
let device = Device::connect_quick(None).await?;

// 或
let device = Device::connect_with_mode(None, ServerMode::Direct).await?;
```

### ATX-Agent 模式（推荐用于生产）⭐

- ✅ 生产级稳定性
- ✅ 自动恢复（服务崩溃自动重启）
- ✅ 屏幕锁定保护
- ✅ 守护进程保证服务持续运行
- 适合：生产环境、长时间运行

```rust
use uiautomator::ServerMode;

// 使用 ATX-Agent 模式（需要设备已安装 atx-agent）
let device = Device::connect_with_mode(None, ServerMode::AtxAgent).await?;

// 自动检测模式（优先 ATX-Agent，失败则回退到 Direct）
let device = Device::connect(None).await?;
```

### 安装 ATX-Agent

有两种方式安装 ATX-Agent：

#### 方式 1: 通过 Python uiautomator2（推荐）

```bash
pip install uiautomator2
python -m uiautomator2 init
```

#### 方式 2: 通过 Rust 库安装（需要 feature）

如果您不想依赖 Python 环境，可以使用 Rust 库直接安装：

**1. 启用 feature**

在 `Cargo.toml` 中添加：

```toml
[dependencies]
uiautomator = { version = "0.1", features = ["atx-agent-install"] }
```

**2. 下载资源文件**

```bash
# Linux/macOS
cd assets
./download_atx_agent.sh

# Windows PowerShell
cd assets
.\download_atx_agent.ps1
```

**3. 安装到设备**

```rust
use uiautomator::Device;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 连接到设备（Direct 模式）
    let device = Device::connect_quick(None).await?;

    // 安装 ATX-Agent
    device.install_atx_agent(false).await?;

    // 现在可以使用 ATX-Agent 模式
    let device = Device::connect(None).await?;

    Ok(())
}
```

**注意**：
- 资源文件约 12-14MB，不包含在库中
- 需要手动下载后才能使用安装功能
- 安装过程需要几分钟时间

---

## 多设备操作

支持同时控制多个设备进行并发操作：

```rust
use uiautomator::Device;
use tokio::try_join;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 连接到多个设备
    let device1 = Device::connect(Some("emulator-5554")).await?;
    let device2 = Device::connect(Some("emulator-5556")).await?;

    // 并发执行操作
    let (info1, info2) = try_join!(
        device1.info(),
        device2.info()
    )?;

    println!("设备 1: {}x{}", info1.display_width, info1.display_height);
    println!("设备 2: {}x{}", info2.display_width, info2.display_height);

    // 同时点击多个设备
    try_join!(
        device1.click(100, 200),
        device2.click(100, 200)
    )?;

    Ok(())
}
```

---

## 日志系统

库内置了基于 `env_logger` 的日志系统，可以通过环境变量控制日志级别：

```bash
# 显示所有日志（包括调试信息）
RUST_LOG=debug cargo run

# 只显示信息级别及以上（默认）
RUST_LOG=info cargo run

# 只显示警告和错误
RUST_LOG=warn cargo run

# 只显示特定模块的日志
RUST_LOG=uiautomator::device=debug cargo run
```

在代码中初始化日志：

```rust
// 使用默认配置（Info 级别）
uiautomator::init_logger();

// 或指定日志级别
use log::LevelFilter;
uiautomator::init_logger_with_level(LevelFilter::Debug);
```

日志输出示例：

```
[2024-01-17 10:30:15] INFO  uiautomator::device - 连接到设备: emulator-5554
[2024-01-17 10:30:16] DEBUG uiautomator::jsonrpc - 发送 JSON-RPC 请求: deviceInfo
[2024-01-17 10:30:16] INFO  uiautomator::device - 设备信息: 1080x1920, SDK 30
```

---

## 从 Python uiautomator2 迁移

本库的 API 设计尽可能与 Python uiautomator2 保持一致，降低迁移成本。

### 主要差异

1. **异步编程**: 所有 I/O 操作都需要 `.await?`
2. **类型安全**: 使用 Rust 的类型系统（如 `Duration`, `Option<T>`）
3. **错误处理**: 使用 `Result<T>` 和 `?` 操作符，而不是异常
4. **可选参数**: 使用 `Option<T>`，传 `None` 表示使用默认值

### API 对比表

| 功能 | Python uiautomator2 | uiautomator (Rust) |
|------|---------------------|-------------------|
| **设备连接** |
| 连接设备 | `d = u2.connect()` | `let d = Device::connect(None).await?` |
| 指定设备 | `d = u2.connect("serial")` | `let d = Device::connect(Some("serial")).await?` |
| **坐标操作** |
| 点击 | `d.click(100, 200)` | `d.click(100, 200).await?` |
| 长按 | `d.long_click(100, 200, 1.0)` | `d.long_click(100, 200, Some(Duration::from_secs(1))).await?` |
| 双击 | `d.double_click(100, 200)` | `d.double_click(100, 200, None).await?` |
| 滑动 | `d.swipe(100, 200, 300, 400, 0.5)` | `d.swipe(100, 200, 300, 400, Some(Duration::from_secs_f32(0.5))).await?` |
| **元素定位** |
| 文本定位 | `d(text="Settings")` | `d.find(Selector::new().text("Settings"))` |
| 资源 ID | `d(resourceId="id")` | `d.find(Selector::new().resource_id("id"))` |
| 类名 | `d(className="Button")` | `d.find(Selector::new().class_name("Button"))` |
| 组合条件 | `d(text="OK", clickable=True)` | `d.find(Selector::new().text("OK").clickable(true))` |
| **元素操作** |
| 点击元素 | `d(text="OK").click()` | `d.find(Selector::new().text("OK")).click(None, None).await?` |
| 长按元素 | `d(text="OK").long_click(1.0)` | `d.find(Selector::new().text("OK")).long_click(Some(Duration::from_secs(1)), None).await?` |
| 获取文本 | `text = d(resourceId="id").get_text()` | `let text = d.find(Selector::new().resource_id("id")).get_text().await?` |
| 设置文本 | `d(className="EditText").set_text("hi")` | `d.find(Selector::new().class_name("EditText")).set_text("hi").await?` |
| 等待出现 | `d(text="OK").wait(timeout=10)` | `d.find(Selector::new().text("OK")).wait(Some(Duration::from_secs(10))).await?` |
| 等待消失 | `d(text="OK").wait_gone(timeout=10)` | `d.find(Selector::new().text("OK")).wait_gone(Some(Duration::from_secs(10))).await?` |
| **按键操作** |
| Home 键 | `d.press("home")` | `d.press(Key::Home).await?` |
| Back 键 | `d.press("back")` | `d.press(Key::Back).await?` |
| 键码 | `d.press(3)` | `d.press_keycode(3).await?` |
| **应用管理** |
| 启动应用 | `d.app_start("pkg")` | `d.app_start("pkg", None).await?` |
| 启动 Activity | `d.app_start("pkg", "Activity")` | `d.app_start("pkg", Some("Activity")).await?` |
| 停止应用 | `d.app_stop("pkg")` | `d.app_stop("pkg").await?` |
| 等待应用 | `d.app_wait("pkg", timeout=10)` | `d.app_wait("pkg", Some(Duration::from_secs(10))).await?` |
| 当前应用 | `info = d.app_current()` | `let info = d.app_current().await?` |
| **截图** |
| 截图保存 | `d.screenshot("ss.png")` | `d.screenshot_to_file("ss.png").await?` |
| 获取图像 | `img = d.screenshot()` | `let img = d.screenshot().await?` |

### 代码示例对比

#### Python 版本

```python
import uiautomator2 as u2

# 连接设备
d = u2.connect()

# 启动应用
d.app_start("com.android.settings")

# 点击元素
d(text="Wi-Fi").click()

# 等待元素
d(text="Connected").wait(timeout=10)

# 获取文本
network_name = d(resourceId="com.android.settings:id/title").get_text()
print(f"网络: {network_name}")

# 返回
d.press("back")
```

#### Rust 版本

```rust
use uiautomator::{Device, Selector, Key};
use std::time::Duration;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 连接设备
    let d = Device::connect(None).await?;

    // 启动应用
    d.app_start("com.android.settings", None).await?;

    // 点击元素
    d.find(Selector::new().text("Wi-Fi"))
        .click(None, None).await?;

    // 等待元素
    d.find(Selector::new().text("Connected"))
        .wait(Some(Duration::from_secs(10))).await?;

    // 获取文本
    let network_name = d.find(Selector::new()
        .resource_id("com.android.settings:id/title"))
        .get_text().await?;
    println!("网络: {}", network_name);

    // 返回
    d.press(Key::Back).await?;

    Ok(())
}
```

---

## 示例程序

```bash
# 基础使用
cargo run --example basic

# 设备操作
cargo run --example device_demo

# UI 对象操作
cargo run --example uiobject_demo

# 手势操作
cargo run --example gesture_demo

# 按键操作
cargo run --example key_demo

# 应用管理
cargo run --example app_demo

# 截图
cargo run --example screenshot_demo

# 并发操作
cargo run --example concurrent

# 日志系统
RUST_LOG=debug cargo run --example logging_demo
```

---

## 构建与测试

```bash
# 构建所有子项目
cargo build

# 运行单元测试
cargo test -p uiautomator

# 运行所有测试
cargo test

# 运行 CLI 工具
cargo run -p uiautomator-cli -- --help

# 格式化代码
cargo fmt

# 检查代码质量
cargo clippy
```

---

## 数据模型

### DeviceInfo

设备信息结构：

```rust
pub struct DeviceInfo {
    pub display_width: u32,          // 屏幕宽度
    pub display_height: u32,         // 屏幕高度
    pub display_rotation: u32,       // 屏幕旋转角度
    pub current_package_name: String, // 当前包名
    pub sdk_int: u32,                // SDK 版本号
    pub screen_on: bool,             // 屏幕是否点亮
    pub natural_orientation: bool,   // 是否为自然方向
}
```

### ElementInfo

元素信息结构：

```rust
pub struct ElementInfo {
    pub text: String,                // 元素文本
    pub content_description: String, // 内容描述
    pub class_name: String,          // 类名
    pub package_name: String,        // 包名
    pub resource_id: String,         // 资源 ID
    pub bounds: Rect,                // 边界
    pub visible_bounds: Rect,        // 可见边界
    pub clickable: bool,             // 可点击
    pub enabled: bool,               // 已启用
    pub focusable: bool,             // 可获得焦点
    pub focused: bool,               // 已获得焦点
    pub scrollable: bool,            // 可滚动
    pub long_clickable: bool,        // 可长按
    pub checkable: bool,            // 可勾选
    pub checked: bool,              // 已勾选
    pub selected: bool,             // 已选中
    pub child_count: u32,           // 子元素数
}
```

### Rect

矩形区域：

```rust
pub struct Rect {
    pub left: u32,    // 左边界
    pub top: u32,     // 上边界
    pub right: u32,   // 右边界
    pub bottom: u32,  // 下边界
}

impl Rect {
    pub fn new(left: u32, top: u32, right: u32, bottom: u32) -> Self;
    pub fn width(&self) -> u32;
    pub fn height(&self) -> u32;
    pub fn center(&self) -> (u32, u32);
}
```

---

## 故障排查

### 设备连接失败

```
Error: DeviceConnection("无法连接到设备")
```

**解决方法**:
1. 检查 ADB 是否运行：`adb devices`
2. 确保设备已启用 USB 调试
3. 尝试重启 ADB：`adb kill-server && adb start-server`
4. 检查设备是否被其他程序占用

### 元素未找到

```
Error: UiObjectNotFound("元素未找到")
```

**解决方法**:
1. 使用 `uiautomatorviewer` 检查元素属性
2. 增加等待时间：`.wait(Some(Duration::from_secs(10)))`
3. 检查选择器条件是否正确
4. 确保元素在当前屏幕上可见

### 服务启动失败

```
Error: UiAutomatorNotConnected
```

**解决方法**:
1. 等待几秒后重试（首次启动需要时间）
2. 检查设备存储空间是否充足
3. 手动清理设备上的旧文件：`adb shell rm /data/local/tmp/u2.jar`
4. 重启设备

### 操作超时

```
Error: Timeout
```

**解决方法**:
1. 增加超时时间：`device.set_wait_timeout(Duration::from_secs(30))`
2. 检查设备性能和响应速度
3. 确保网络连接稳定
4. 检查设备是否卡顿

---

## 依赖项

主要依赖:
- `tokio` - 异步运行时
- `reqwest` - HTTP 客户端
- `serde` / `serde_json` - JSON 序列化
- `adb_client` - ADB 通信
- `thiserror` - 错误处理
- `log` / `env_logger` - 日志系统

---

## 仓库文档

- 发布前门禁（推荐）：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\trigger-gh-release-gate.ps1 -Repo iamsevens/uiautomator-rs -Ref main
```

该脚本会串行执行 `Release Check` 和 `Publish Dry Run`，任一失败即停止。

- [PUBLISHING.md](PUBLISHING.md) - 发布流程
- [SECURITY.md](SECURITY.md) - 安全漏洞上报流程
- [SUPPORT.md](SUPPORT.md) - 使用支持与反馈渠道
- [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) - 第三方资源说明

---

## 许可证

MIT

---

## 贡献

欢迎提交 Issue 和 Pull Request!

### 开发环境设置

```bash
# 克隆仓库
git clone https://github.com/iamsevens/uiautomator-rs.git
cd uiautomator-rs

# 构建项目
cargo build

# 运行测试
cargo test

# 运行示例
cargo run --example basic
```

### 代码规范

- 遵循 Rust 官方代码风格
- 运行 `cargo fmt` 格式化代码
- 运行 `cargo clippy` 检查代码质量
- 为新功能添加测试
- 更新文档

---

## 致谢

- [Python uiautomator2](https://github.com/openatx/uiautomator2) - 原始项目和灵感来源
- [adb_client](https://github.com/JanCVanB/adb_client) - ADB 客户端库
- [Tokio](https://tokio.rs/) - 异步运行时
- Android UiAutomator Framework - 底层自动化框架

---

## 相关项目

- [uiautomator2](https://github.com/openatx/uiautomator2) - Python 版本
- [atx-agent](https://github.com/openatx/atx-agent) - 设备端守护进程
- [android-uiautomator-server](https://github.com/openatx/android-uiautomator-server) - 设备端服务

---

**注意**: 本项目仅用于自动化测试和开发目的。请遵守相关法律法规，不要用于非法用途。



