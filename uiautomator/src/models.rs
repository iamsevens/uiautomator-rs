//! 数据模型定义
//!
//! 本模块包含与 Android 设备和 UI 元素相关的数据结构。

use serde::{Deserialize, Deserializer, Serialize};

/// 设备信息
///
/// 包含设备的基本信息，如屏幕尺寸、旋转角度、当前应用等。
/// # Examples
///
/// ```
/// use uiautomator::models::DeviceInfo;
///
/// let info = DeviceInfo {
///     display_width: 1080,
///     display_height: 2400,
///     display_rotation: 0,
///     current_package_name: "com.android.settings".to_string(),
///     sdk_int: 34,
///     screen_on: true,
///     natural_orientation: true,
/// };
/// assert_eq!(info.display_width, 1080);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    /// 屏幕宽度（像素）
    pub display_width: u32,
    /// 屏幕高度（像素）
    pub display_height: u32,
    /// 屏幕旋转角度（0, 90, 180, 270）
    #[serde(deserialize_with = "deserialize_display_rotation")]
    pub display_rotation: u32,
    /// 当前前台应用的包名
    pub current_package_name: String,
    /// Android SDK 版本号
    pub sdk_int: u32,
    /// 屏幕是否点亮
    pub screen_on: bool,
    /// 是否为自然方向
    pub natural_orientation: bool,
}

/// UI 元素信息
///
/// 包含 UI 元素的所有属性,如文本、边界、状态等。
/// # Examples
///
/// ```
/// use uiautomator::models::{ElementInfo, Rect};
///
/// let info = ElementInfo {
///     text: "Login".to_string(),
///     content_description: "".to_string(),
///     class_name: "android.widget.Button".to_string(),
///     package_name: "com.example.app".to_string(),
///     resource_id: "com.example.app:id/login".to_string(),
///     bounds: Rect::new(0, 0, 300, 120),
///     visible_bounds: Rect::new(0, 0, 300, 120),
///     clickable: true,
///     enabled: true,
///     focusable: true,
///     focused: false,
///     scrollable: false,
///     long_clickable: false,
///     checkable: false,
///     checked: false,
///     selected: false,
///     child_count: 0,
/// };
/// assert!(info.clickable);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementInfo {
    /// 元素显示的文本
    #[serde(default, deserialize_with = "deserialize_null_string")]
    pub text: String,
    /// 元素的内容描述（用于无障碍）
    #[serde(default, deserialize_with = "deserialize_null_string")]
    pub content_description: String,
    /// 元素的类名
    #[serde(default, deserialize_with = "deserialize_null_string")]
    pub class_name: String,
    /// 元素所属应用的包名
    #[serde(default, deserialize_with = "deserialize_null_string")]
    pub package_name: String,
    /// 元素的资源 ID
    #[serde(
        default,
        alias = "resourceName",
        deserialize_with = "deserialize_null_string"
    )]
    pub resource_id: String,
    /// 元素的边界矩形
    pub bounds: Rect,
    /// 元素的可见边界矩形
    pub visible_bounds: Rect,
    /// 是否可点击
    pub clickable: bool,
    /// 是否启用
    pub enabled: bool,
    /// 是否可获得焦点
    pub focusable: bool,
    /// 是否已获得焦点
    pub focused: bool,
    /// 是否可滚动
    pub scrollable: bool,
    /// 是否可长按
    pub long_clickable: bool,
    /// 是否可勾选
    pub checkable: bool,
    /// 是否已勾选
    pub checked: bool,
    /// 是否已选中
    pub selected: bool,
    /// 子元素数量
    pub child_count: u32,
}

/// 矩形区域
///
/// 表示屏幕上的一个矩形区域，使用左上角和右下角坐标定义。
/// # Examples
///
/// ```
/// use uiautomator::models::Rect;
///
/// let rect = Rect { left: 10, top: 20, right: 110, bottom: 220 };
/// assert_eq!(rect.center(), (60, 120));
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rect {
    /// 左边界 x 坐标
    pub left: u32,
    /// 上边界 y 坐标
    pub top: u32,
    /// 右边界 x 坐标
    pub right: u32,
    /// 下边界 y 坐标
    pub bottom: u32,
}

impl Rect {
    /// 创建新的矩形
    ///
    /// # 参数
    ///
    /// * `left` - 左边界 x 坐标
    /// * `top` - 上边界 y 坐标
    /// * `right` - 右边界 x 坐标
    /// * `bottom` - 下边界 y 坐标
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::models::Rect;
    ///
    /// let rect = Rect::new(10, 20, 100, 200);
    /// assert_eq!(rect.width(), 90);
    /// assert_eq!(rect.height(), 180);
    /// ```
    pub fn new(left: u32, top: u32, right: u32, bottom: u32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// 获取矩形宽度
    ///
    /// # 返回
    ///
    /// 矩形的宽度（像素）
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::models::Rect;
    ///
    /// let rect = Rect::new(10, 20, 100, 200);
    /// assert_eq!(rect.width(), 90);
    /// ```
    pub fn width(&self) -> u32 {
        self.right.saturating_sub(self.left)
    }

    /// 获取矩形高度
    ///
    /// # 返回
    ///
    /// 矩形的高度（像素）
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::models::Rect;
    ///
    /// let rect = Rect::new(10, 20, 100, 200);
    /// assert_eq!(rect.height(), 180);
    /// ```
    pub fn height(&self) -> u32 {
        self.bottom.saturating_sub(self.top)
    }

    /// 获取矩形中心点坐标
    ///
    /// # 返回
    ///
    /// 中心点的 (x, y) 坐标
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::models::Rect;
    ///
    /// let rect = Rect::new(10, 20, 100, 200);
    /// assert_eq!(rect.center(), (55, 110));
    /// ```
    pub fn center(&self) -> (u32, u32) {
        let x = self.left + self.width() / 2;
        let y = self.top + self.height() / 2;
        (x, y)
    }
}

/// 应用信息
///
/// 包含应用的包名、Activity 和进程 ID。
/// # Examples
///
/// ```
/// use uiautomator::models::AppInfo;
///
/// let app = AppInfo {
///     package: "com.example.app".to_string(),
///     activity: ".MainActivity".to_string(),
///     pid: Some(12345),
/// };
/// assert_eq!(app.package, "com.example.app");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    /// 应用包名
    pub package: String,
    /// 当前 Activity 名称
    pub activity: String,
    /// 进程 ID（可能为空）
    pub pid: Option<u32>,
}

fn deserialize_null_string<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

fn deserialize_display_rotation<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = u32::deserialize(deserializer)?;
    Ok(match raw {
        0 => 0,
        1 => 90,
        2 => 180,
        3 => 270,
        _ => raw,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_width() {
        let rect = Rect::new(10, 20, 100, 200);
        assert_eq!(rect.width(), 90);
    }

    #[test]
    fn test_rect_height() {
        let rect = Rect::new(10, 20, 100, 200);
        assert_eq!(rect.height(), 180);
    }

    #[test]
    fn test_rect_center() {
        let rect = Rect::new(10, 20, 100, 200);
        assert_eq!(rect.center(), (55, 110));
    }

    #[test]
    fn test_rect_center_odd_dimensions() {
        let rect = Rect::new(0, 0, 101, 201);
        assert_eq!(rect.center(), (50, 100));
    }

    #[test]
    fn test_rect_zero_size() {
        let rect = Rect::new(50, 50, 50, 50);
        assert_eq!(rect.width(), 0);
        assert_eq!(rect.height(), 0);
        assert_eq!(rect.center(), (50, 50));
    }

    #[test]
    fn test_rect_serialization() {
        let rect = Rect::new(10, 20, 100, 200);
        let json = serde_json::to_string(&rect).unwrap();
        let deserialized: Rect = serde_json::from_str(&json).unwrap();
        assert_eq!(rect, deserialized);
    }

    #[test]
    fn test_device_info_serialization() {
        let info = DeviceInfo {
            display_width: 1080,
            display_height: 1920,
            display_rotation: 0,
            current_package_name: "com.example.app".to_string(),
            sdk_int: 30,
            screen_on: true,
            natural_orientation: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: DeviceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.display_width, deserialized.display_width);
        assert_eq!(info.display_height, deserialized.display_height);
    }

    #[test]
    fn test_device_info_rotation_from_surface_index() {
        let json = r#"{
            "displayWidth": 1080,
            "displayHeight": 1920,
            "displayRotation": 1,
            "currentPackageName": "com.example.app",
            "sdkInt": 30,
            "screenOn": true,
            "naturalOrientation": true
        }"#;

        let info: DeviceInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.display_rotation, 90);
    }

    #[test]
    fn test_device_info_rotation_from_degrees() {
        let json = r#"{
            "displayWidth": 1080,
            "displayHeight": 1920,
            "displayRotation": 270,
            "currentPackageName": "com.example.app",
            "sdkInt": 30,
            "screenOn": true,
            "naturalOrientation": false
        }"#;

        let info: DeviceInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.display_rotation, 270);
    }

    #[test]
    fn test_app_info_serialization() {
        let info = AppInfo {
            package: "com.example.app".to_string(),
            activity: "MainActivity".to_string(),
            pid: Some(12345),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: AppInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.package, deserialized.package);
        assert_eq!(info.activity, deserialized.activity);
        assert_eq!(info.pid, deserialized.pid);
    }

    #[test]
    fn test_element_info_deserialize_null_string_fields() {
        let json = r#"{
            "text": "BUTTON",
            "contentDescription": null,
            "className": "android.widget.Button",
            "packageName": "com.uiautomator.testapp",
            "resourceName": "com.example:id/button",
            "bounds": {"left": 0, "top": 0, "right": 100, "bottom": 50},
            "visibleBounds": {"left": 0, "top": 0, "right": 100, "bottom": 50},
            "clickable": true,
            "enabled": true,
            "focusable": true,
            "focused": false,
            "scrollable": false,
            "longClickable": false,
            "checkable": false,
            "checked": false,
            "selected": false,
            "childCount": 0
        }"#;

        let info: ElementInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.text, "BUTTON");
        assert_eq!(info.content_description, "");
        assert_eq!(info.resource_id, "com.example:id/button");
    }
}
