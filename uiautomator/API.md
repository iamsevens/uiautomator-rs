# uiautomator API 文档

完整的 API 参考文档。

## 目录

- [Device](#device) - 设备连接和操作
- [Selector](#selector) - UI 元素选择器
- [UiObject](#uiobject) - UI 对象操作
- [Key](#key) - 按键枚举
- [Settings](#settings) - 配置和设置
- [数据模型](#数据模型) - 数据结构定义
- [错误类型](#错误类型) - 错误处理

---

## Device

设备是库的核心入口，代表一个 Android 设备连接。

### 连接方法

#### `Device::connect(serial: Option<&str>) -> Result<Device>`

连接到设备（自动检测模式）。

**参数**:
- `serial`: 设备序列号，`None` 表示自动选择唯一设备

**返回**: `Result<Device>`

**示例**:
```rust
// 自动选择唯一设备
let device = Device::connect(None).await?;

// 连接到指定设备
let device = Device::connect(Some("emulator-5554")).await?;
```

**错误**:
- `DeviceConnection`: 设备连接失败
- `InvalidArgument`: 多个设备但未指定序列号

---

#### `Device::connect_quick(serial: Option<&str>) -> Result<Device>`

快速连接（强制使用 Direct 模式）。

**参数**:
- `serial`: 设备序列号

**返回**: `Result<Device>`

**示例**:
```rust
let device = Device::connect_quick(None).await?;
```

---

#### `Device::connect_with_mode(serial: Option<&str>, mode: ServerMode) -> Result<Device>`

使用指定模式连接到设备。

**参数**:
- `serial`: 设备序列号
- `mode`: 服务模式（`Direct`, `AtxAgent`, `Auto`）

**返回**: `Result<Device>`

**示例**:
```rust
use uiautomator::ServerMode;

// 使用 Direct 模式
let device = Device::connect_with_mode(None, ServerMode::Direct).await?;

// 使用 ATX-Agent 模式
let device = Device::connect_with_mode(None, ServerMode::AtxAgent).await?;

// 自动检测模式
let device = Device::connect_with_mode(None, ServerMode::Auto).await?;
```

---

### 设备信息

#### `device.info() -> Result<DeviceInfo>`

获取设备信息。

**返回**: `Result<DeviceInfo>`

**示例**:
```rust
let info = device.info().await?;
println!("屏幕: {}x{}", info.display_width, info.display_height);
println!("SDK: {}", info.sdk_int);
println!("旋转: {}", info.display_rotation);
```

---

#### `device.window_size() -> Result<(u32, u32)>`

获取屏幕尺寸。

**返回**: `Result<(width, height)>`

**示例**:
```rust
let (width, height) = device.window_size().await?;
println!("屏幕尺寸: {}x{}", width, height);
```

---

### 坐标操作

#### `device.click(x: u32, y: u32) -> Result<()>`

点击指定坐标。

**参数**:
- `x`: X 坐标（像素）
- `y`: Y 坐标（像素）

**返回**: `Result<()>`

**示例**:
```rust
device.click(500, 1000).await?;
```

---

#### `device.long_click(x: u32, y: u32, duration: Option<Duration>) -> Result<()>`

长按指定坐标。

**参数**:
- `x`: X 坐标
- `y`: Y 坐标
- `duration`: 长按时长，`None` 使用默认值（0.5 秒）

**返回**: `Result<()>`

**示例**:
```rust
use std::time::Duration;

// 使用默认时长（0.5 秒）
device.long_click(500, 1000, None).await?;

// 指定时长（2 秒）
device.long_click(500, 1000, Some(Duration::from_secs(2))).await?;
```

---

#### `device.double_click(x: u32, y: u32, duration: Option<Duration>) -> Result<()>`

双击指定坐标。

**参数**:
- `x`: X 坐标
- `y`: Y 坐标
- `duration`: 两次点击间隔，`None` 使用默认值（0.1 秒）

**返回**: `Result<()>`

**示例**:
```rust
// 使用默认间隔
device.double_click(500, 1000, None).await?;

// 自定义间隔
device.double_click(500, 1000, Some(Duration::from_millis(200))).await?;
```

---

#### `device.swipe(fx: u32, fy: u32, tx: u32, ty: u32, duration: Option<Duration>) -> Result<()>`

滑动操作。

**参数**:
- `fx`: 起始 X 坐标
- `fy`: 起始 Y 坐标
- `tx`: 结束 X 坐标
- `ty`: 结束 Y 坐标
- `duration`: 滑动时长，`None` 使用默认值（0.5 秒）

**返回**: `Result<()>`

**示例**:
```rust
// 向上滑动
device.swipe(500, 1500, 500, 500, None).await?;

// 快速滑动（0.2 秒）
device.swipe(500, 1500, 500, 500, Some(Duration::from_millis(200))).await?;
```

---

#### `device.drag(sx: u32, sy: u32, ex: u32, ey: u32, duration: Option<Duration>) -> Result<()>`

拖拽操作。

**参数**:
- `sx`: 起始 X 坐标
- `sy`: 起始 Y 坐标
- `ex`: 结束 X 坐标
- `ey`: 结束 Y 坐标
- `duration`: 拖拽时长，`None` 使用默认值（0.5 秒）

**返回**: `Result<()>`

**示例**:
```rust
device.drag(200, 300, 600, 800, Some(Duration::from_secs(1))).await?;
```

---

### 按键操作

#### `device.press(key: Key) -> Result<()>`

按下指定按键。

**参数**:
- `key`: 按键枚举

**返回**: `Result<()>`

**示例**:
```rust
use uiautomator::Key;

device.press(Key::Home).await?;
device.press(Key::Back).await?;
device.press(Key::Power).await?;
device.press(Key::VolumeUp).await?;
```

---

#### `device.press_keycode(keycode: u32) -> Result<()>`

按下指定键码。

**参数**:
- `keycode`: Android 键码

**返回**: `Result<()>`

**示例**:
```rust
device.press_keycode(3).await?; // Home 键
device.press_keycode(4).await?; // Back 键
```

---

### 元素定位

#### `device.find(selector: Selector) -> UiObject`

定位 UI 元素。

**参数**:
- `selector`: 元素选择器

**返回**: `UiObject`

**示例**:
```rust
use uiautomator::Selector;

// 通过文本定位
let element = device.find(Selector::new().text("设置"));

// 通过资源 ID 定位
let element = device.find(Selector::new().resource_id("com.android.settings:id/search"));

// 组合条件
let element = device.find(Selector::new()
    .text("确定")
    .class_name("android.widget.Button")
    .clickable(true));
```

---

### 应用管理

#### `device.app_start(package: &str, activity: Option<&str>) -> Result<()>`

启动应用。

**参数**:
- `package`: 应用包名
- `activity`: Activity 名称，`None` 表示启动主 Activity

**返回**: `Result<()>`

**示例**:
```rust
// 启动应用（主 Activity）
device.app_start("com.android.settings", None).await?;

// 启动指定 Activity
device.app_start("com.android.settings", Some(".Settings")).await?;
```

---

#### `device.app_stop(package: &str) -> Result<()>`

停止应用。

**参数**:
- `package`: 应用包名

**返回**: `Result<()>`

**示例**:
```rust
device.app_stop("com.android.settings").await?;
```

---

#### `device.app_clear(package: &str) -> Result<()>`

清除应用数据。

**参数**:
- `package`: 应用包名

**返回**: `Result<()>`

**示例**:
```rust
device.app_clear("com.android.settings").await?;
```

---

#### `device.app_current() -> Result<AppInfo>`

获取当前前台应用信息。

**返回**: `Result<AppInfo>`

**示例**:
```rust
let app = device.app_current().await?;
println!("当前应用: {}", app.package);
println!("Activity: {}", app.activity);
if let Some(pid) = app.pid {
    println!("PID: {}", pid);
}
```

---

#### `device.app_wait(package: &str, timeout: Option<Duration>) -> Result<u32>`

等待应用启动。

**参数**:
- `package`: 应用包名
- `timeout`: 超时时间，`None` 表示使用全局等待超时

**返回**: `Result<u32>` - 应用 PID

**示例**:
```rust
use std::time::Duration;

let pid = device
    .app_wait("com.android.settings", Some(Duration::from_secs(10)))
    .await?;
println!("应用已启动，PID: {}", pid);
```

---

### 截图

#### `device.screenshot() -> Result<DynamicImage>`

截取屏幕。

**返回**: `Result<DynamicImage>` - 图像数据

**示例**:
```rust
let image = device.screenshot().await?;
println!("截图尺寸: {}x{}", image.width(), image.height());

// 保存为文件
image.save("screenshot.png")?;
```

---

#### `device.screenshot_to_file(path: &str) -> Result<()>`

截图并保存到文件。

**参数**:
- `path`: 文件路径（支持 .png 和 .jpg）

**返回**: `Result<()>`

**示例**:
```rust
device.screenshot_to_file("screenshot.png").await?;
device.screenshot_to_file("screenshot.jpg").await?;
```

---

### 配置

#### `device.set_wait_timeout(timeout: Duration)`

设置全局等待超时。

**参数**:
- `timeout`: 超时时间

**示例**:
```rust
use std::time::Duration;

device.set_wait_timeout(Duration::from_secs(30)).await;
```

---

#### `device.get_wait_timeout() -> Duration`

获取当前等待超时设置。

**返回**: `Duration`

**示例**:
```rust
let timeout = device.get_wait_timeout().await;
println!("当前超时: {:?}", timeout);
```

---

## Selector

UI 元素选择器，用于构建元素查找条件。

### 构造方法

#### `Selector::new() -> Selector`

创建新的选择器。

**返回**: `Selector`

**示例**:
```rust
let selector = Selector::new();
```

---

### 构建器方法

所有构建器方法都返回 `Self`，支持链式调用。

#### `selector.text(text: impl Into<String>) -> Self`

匹配文本（精确匹配）。

**示例**:
```rust
let selector = Selector::new().text("设置");
```

---

#### `selector.text_contains(text: impl Into<String>) -> Self`

匹配包含指定文本。

**示例**:
```rust
let selector = Selector::new().text_contains("设");
```

---

#### `selector.text_starts_with(text: impl Into<String>) -> Self`

匹配以指定文本开头。

**示例**:
```rust
let selector = Selector::new().text_starts_with("设置");
```

---

#### `selector.resource_id(id: impl Into<String>) -> Self`

匹配资源 ID。

**示例**:
```rust
let selector = Selector::new().resource_id("com.android.settings:id/search");
```

---

#### `selector.class_name(name: impl Into<String>) -> Self`

匹配类名。

**示例**:
```rust
let selector = Selector::new().class_name("android.widget.Button");
```

---

#### `selector.description(desc: impl Into<String>) -> Self`

匹配内容描述。

**示例**:
```rust
let selector = Selector::new().description("搜索按钮");
```

---

#### `selector.clickable(clickable: bool) -> Self`

匹配可点击属性。

**示例**:
```rust
let selector = Selector::new().clickable(true);
```

---

#### `selector.enabled(enabled: bool) -> Self`

匹配启用状态。

**示例**:
```rust
let selector = Selector::new().enabled(true);
```

---

#### `selector.scrollable(scrollable: bool) -> Self`

匹配可滚动属性。

**示例**:
```rust
let selector = Selector::new().scrollable(true);
```

---

#### `selector.instance(instance: u32) -> Self`

选择第 N 个匹配的元素（从 0 开始）。

**示例**:
```rust
// 选择第二个匹配的元素
let selector = Selector::new().text("项目").instance(1);
```

---

### 组合使用

```rust
let selector = Selector::new()
    .text("确定")
    .class_name("android.widget.Button")
    .clickable(true)
    .enabled(true);
```

---

## UiObject

UI 对象，代表一个定位到的 UI 元素。

### 存在性检查

#### `uiobject.exists(timeout: Option<Duration>) -> Result<bool>`

检查元素是否存在。

**参数**:
- `timeout`: 超时时间，`None` 使用全局超时

**返回**: `Result<bool>`

**示例**:
```rust
if element.exists(None).await? {
    println!("元素存在");
}

// 指定超时
if element.exists(Some(Duration::from_secs(5))).await? {
    println!("元素在 5 秒内出现");
}
```

---

#### `uiobject.wait(timeout: Option<Duration>) -> Result<()>`

等待元素出现。

**参数**:
- `timeout`: 超时时间

**返回**: `Result<()>`

**错误**: 超时时返回 `Timeout`

**示例**:
```rust
// 等待元素出现（使用全局超时）
element.wait(None).await?;

// 等待最多 10 秒
element.wait(Some(Duration::from_secs(10))).await?;
```

---

#### `uiobject.wait_gone(timeout: Option<Duration>) -> Result<()>`

等待元素消失。

**参数**:
- `timeout`: 超时时间

**返回**: `Result<()>`

**示例**:
```rust
element.wait_gone(Some(Duration::from_secs(5))).await?;
```

---

### 元素信息

#### `uiobject.info() -> Result<ElementInfo>`

获取元素详细信息。

**返回**: `Result<ElementInfo>`

**示例**:
```rust
let info = element.info().await?;
println!("文本: {}", info.text);
println!("类名: {}", info.class_name);
println!("边界: {:?}", info.bounds);
println!("可点击: {}", info.clickable);
```

---

#### `uiobject.get_text() -> Result<String>`

获取元素文本。

**返回**: `Result<String>`

**示例**:
```rust
let text = element.get_text().await?;
println!("文本: {}", text);
```

---

#### `uiobject.center() -> Result<(u32, u32)>`

获取元素中心坐标。

**返回**: `Result<(x, y)>`

**示例**:
```rust
let (x, y) = element.center().await?;
println!("中心坐标: ({}, {})", x, y);
```

---

#### `uiobject.bounds() -> Result<Rect>`

获取元素边界。

**返回**: `Result<Rect>`

**示例**:
```rust
let bounds = element.bounds().await?;
println!("边界: left={}, top={}, right={}, bottom={}",
    bounds.left, bounds.top, bounds.right, bounds.bottom);
println!("宽度: {}, 高度: {}", bounds.width(), bounds.height());
```

---

### 元素操作

#### `uiobject.click(timeout: Option<Duration>, offset: Option<(f32, f32)>) -> Result<()>`

点击元素。

**参数**:
- `timeout`: 超时时间
- `offset`: 点击偏移（相对于元素中心，范围 0.0-1.0）

**返回**: `Result<()>`

**示例**:
```rust
// 点击元素中心
element.click(None, None).await?;

// 点击元素左上角
element.click(None, Some((0.0, 0.0))).await?;

// 点击元素右下角
element.click(None, Some((1.0, 1.0))).await?;

// 指定超时
element.click(Some(Duration::from_secs(5)), None).await?;
```

---

#### `uiobject.click_exists(timeout: Option<Duration>) -> Result<bool>`

如果元素存在则点击。

**参数**:
- `timeout`: 超时时间

**返回**: `Result<bool>` - 是否点击成功

**示例**:
```rust
if element.click_exists(None).await? {
    println!("元素存在并已点击");
} else {
    println!("元素不存在");
}
```

---

#### `uiobject.long_click(duration: Option<Duration>, timeout: Option<Duration>) -> Result<()>`

长按元素。

**参数**:
- `duration`: 长按时长，`None` 使用默认值（0.5 秒）
- `timeout`: 超时时间

**返回**: `Result<()>`

**示例**:
```rust
// 使用默认时长
element.long_click(None, None).await?;

// 长按 2 秒
element.long_click(Some(Duration::from_secs(2)), None).await?;
```

---

#### `uiobject.set_text(text: &str) -> Result<()>`

设置元素文本（会先清除原有文本）。

**参数**:
- `text`: 要设置的文本

**返回**: `Result<()>`

**示例**:
```rust
element.set_text("Hello, World!").await?;
```

---

#### `uiobject.clear_text() -> Result<()>`

清除元素文本。

**返回**: `Result<()>`

**示例**:
```rust
element.clear_text().await?;
```

---

#### `uiobject.screenshot() -> Result<DynamicImage>`

截取元素图像。

**返回**: `Result<DynamicImage>`

**示例**:
```rust
let image = element.screenshot().await?;
image.save("element.png")?;
```

---

## Key

按键枚举，表示 Android 物理按键和软键。

### 可用按键

```rust
pub enum Key {
    Home,           // Home 键
    Back,           // 返回键
    Power,          // 电源键
    VolumeUp,       // 音量加
    VolumeDown,     // 音量减
    VolumeMute,     // 静音
    Menu,           // 菜单键
    Search,         // 搜索键
    Enter,          // 回车键
    Delete,         // 删除键
    Recent,         // 最近任务键
    Camera,         // 相机键
    Up,             // 方向键上
    Down,           // 方向键下
    Left,           // 方向键左
    Right,          // 方向键右
    Center,         // 方向键中心
}
```

### 方法

#### `key.to_keycode() -> u32`

转换为 Android 键码。

**返回**: `u32`

**示例**:
```rust
let keycode = Key::Home.to_keycode(); // 3
```

---

#### `key.to_name() -> &'static str`

转换为按键名称。

**返回**: `&'static str`

**示例**:
```rust
let name = Key::Home.to_name(); // "home"
```

---

## Settings

配置和设置。

### 字段

```rust
pub struct Settings {
    pub wait_timeout: Duration,              // 等待超时（默认 20 秒）
    pub operation_delay_before: Duration,    // 操作前延迟（默认 0）
    pub operation_delay_after: Duration,     // 操作后延迟（默认 0）
    pub http_timeout: Duration,              // HTTP 超时（默认 60 秒）
    pub max_retry: u32,                      // 最大重试次数（默认 3）
}
```

### 默认值

```rust
Settings::default()
// wait_timeout: 20 秒
// operation_delay_before: 0
// operation_delay_after: 0
// http_timeout: 60 秒
// max_retry: 3
```

---

## 数据模型

### DeviceInfo

设备信息。

```rust
pub struct DeviceInfo {
    pub display_width: u32,          // 屏幕宽度
    pub display_height: u32,         // 屏幕高度
    pub display_rotation: u32,       // 屏幕旋转角度（0, 90, 180, 270）
    pub current_package_name: String, // 当前前台应用包名
    pub sdk_int: u32,                // Android SDK 版本
    pub screen_on: bool,             // 屏幕是否点亮
    pub natural_orientation: bool,   // 是否为自然方向
}
```

---

### ElementInfo

元素信息。

```rust
pub struct ElementInfo {
    pub text: String,                // 文本
    pub content_description: String, // 内容描述
    pub class_name: String,          // 类名
    pub package_name: String,        // 包名
    pub resource_id: String,         // 资源 ID
    pub bounds: Rect,                // 边界
    pub visible_bounds: Rect,        // 可见边界
    pub clickable: bool,             // 是否可点击
    pub enabled: bool,               // 是否启用
    pub focusable: bool,             // 是否可获得焦点
    pub focused: bool,               // 是否已获得焦点
    pub scrollable: bool,            // 是否可滚动
    pub long_clickable: bool,        // 是否可长按
    pub checkable: bool,             // 是否可勾选
    pub checked: bool,               // 是否已勾选
    pub selected: bool,              // 是否已选中
    pub child_count: u32,            // 子元素数量
}
```

---

### Rect

矩形区域。

```rust
pub struct Rect {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl Rect {
    pub fn width(&self) -> u32;      // 宽度
    pub fn height(&self) -> u32;     // 高度
    pub fn center(&self) -> (u32, u32); // 中心坐标
}
```

---

### AppInfo

应用信息。

```rust
pub struct AppInfo {
    pub package: String,    // 包名
    pub activity: String,   // Activity 名称
    pub pid: Option<u32>,   // 进程 ID
}
```

---

## 错误类型

### Error

错误枚举。

```rust
pub enum Error {
    DeviceConnection(String),        // 设备连接错误
    Adb(String),                     // ADB 错误
    UiObjectNotFound(String),        // UI 对象未找到
    Http(reqwest::Error),            // HTTP 请求错误
    HttpTimeout,                     // HTTP 超时
    JsonRpc(i32, String),            // JSON-RPC 错误
    UiAutomatorNotConnected,         // UiAutomator 服务未连接
    Timeout,                         // 操作超时
    Serialization(serde_json::Error), // 序列化错误
    Io(std::io::Error),              // IO 错误
    Image(image::ImageError),        // 图像处理错误
    InvalidArgument(String),         // 无效参数
    AppNotFound(String),             // 应用未找到
}
```

### Result

结果类型别名。

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

---

## ServerMode

服务模式枚举。

```rust
pub enum ServerMode {
    Direct,      // 直接模式（直接连接 uiautomator2）
    AtxAgent,    // ATX-Agent 模式（通过 atx-agent 转发）
    Auto,        // 自动检测模式（优先 ATX-Agent，失败则回退到 Direct）
}
```

---

## 工具函数

### `init_logger()`

初始化日志系统（使用默认配置）。

**示例**:
```rust
uiautomator::init_logger();
```

---

### `init_logger_with_level(level: LevelFilter)`

使用自定义日志级别初始化日志系统。

**参数**:
- `level`: 日志级别过滤器

**示例**:
```rust
use log::LevelFilter;

uiautomator::init_logger_with_level(LevelFilter::Debug);
```

---

## 完整示例

```rust
use uiautomator::{Device, Selector, Key};
use std::time::Duration;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 初始化日志
    uiautomator::init_logger();
    
    // 连接设备
    let device = Device::connect(None).await?;
    
    // 获取设备信息
    let info = device.info().await?;
    println!("设备: {}x{}, SDK {}", 
        info.display_width, info.display_height, info.sdk_int);
    
    // 启动应用
    device.app_start("com.android.settings", None).await?;
    device
        .app_wait("com.android.settings", Some(Duration::from_secs(10)))
        .await?;
    
    // 查找并点击元素
    let element = device.find(Selector::new().text("Wi-Fi"));
    element.wait(Some(Duration::from_secs(5))).await?;
    element.click(None, None).await?;
    
    // 等待加载
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // 获取元素文本
    let network = device.find(Selector::new()
        .resource_id("com.android.settings:id/title")
        .instance(0));
    
    if network.exists(None).await? {
        let text = network.get_text().await?;
        println!("网络: {}", text);
    }
    
    // 截图
    device.screenshot_to_file("screenshot.png").await?;
    
    // 返回
    device.press(Key::Back).await?;
    
    Ok(())
}
```

---

更多示例请参考 `examples/` 目录。
