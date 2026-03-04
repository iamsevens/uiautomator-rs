// Selector 模块 - UI 元素选择器
//
// 用于构建 UI 元素查找条件，支持多种属性组合定位元素

use serde_json::{json, Value};

// Mask 常量定义 - 用于标识哪些字段被设置
// 这些值必须与 Python UIAutomator2 保持一致

/// Text 字段的 mask 位
const MASK_TEXT: u32 = 0x01;
/// TextContains 字段的 mask 位
const MASK_TEXT_CONTAINS: u32 = 0x02;
/// TextMatches 字段的 mask 位
const MASK_TEXT_MATCHES: u32 = 0x04;
/// TextStartsWith 字段的 mask 位
const MASK_TEXT_STARTS_WITH: u32 = 0x08;
/// ClassName 字段的 mask 位
const MASK_CLASS_NAME: u32 = 0x10;
/// Description 字段的 mask 位
const MASK_DESCRIPTION: u32 = 0x40;
/// DescriptionContains 字段的 mask 位
const MASK_DESCRIPTION_CONTAINS: u32 = 0x80;
/// Clickable 字段的 mask 位
const MASK_CLICKABLE: u32 = 0x1000;
/// Scrollable 字段的 mask 位
const MASK_SCROLLABLE: u32 = 0x4000;
/// Enabled 字段的 mask 位
const MASK_ENABLED: u32 = 0x8000;
/// Focusable 字段的 mask 位
const MASK_FOCUSABLE: u32 = 0x010000;
/// PackageName 字段的 mask 位
const MASK_PACKAGE_NAME: u32 = 0x080000;
/// ResourceId 字段的 mask 位
const MASK_RESOURCE_ID: u32 = 0x200000;
/// Instance 字段的 mask 位
const MASK_INSTANCE: u32 = 0x01000000;

// === 扩展布尔字段 ===
/// Checkable 字段的 mask 位
const MASK_CHECKABLE: u32 = 0x0400;
/// Checked 字段的 mask 位
const MASK_CHECKED: u32 = 0x0800;
/// LongClickable 字段的 mask 位
const MASK_LONG_CLICKABLE: u32 = 0x2000;
/// Focused 字段的 mask 位
const MASK_FOCUSED: u32 = 0x020000;
/// Selected 字段的 mask 位
const MASK_SELECTED: u32 = 0x040000;

// === Matches 正则匹配字段 ===
/// ClassNameMatches 字段的 mask 位
const MASK_CLASS_NAME_MATCHES: u32 = 0x20;
/// DescriptionMatches 字段的 mask 位
const MASK_DESCRIPTION_MATCHES: u32 = 0x0100;
/// DescriptionStartsWith 字段的 mask 位
const MASK_DESCRIPTION_STARTS_WITH: u32 = 0x0200;
/// PackageNameMatches 字段的 mask 位
const MASK_PACKAGE_NAME_MATCHES: u32 = 0x100000;
/// ResourceIdMatches 字段的 mask 位
const MASK_RESOURCE_ID_MATCHES: u32 = 0x400000;

// === Index 字段 ===
/// Index 字段的 mask 位
const MASK_INDEX: u32 = 0x800000;

/// UI 元素选择器
///
/// 用于构建元素查找条件，支持链式调用。
///
/// # 示例
///
/// ```
/// use uiautomator::Selector;
///
/// // 通过文本定位
/// let selector = Selector::new().text("Settings");
///
/// // 组合多个条件
/// let selector = Selector::new()
///     .text("Settings")
///     .class_name("android.widget.TextView")
///     .clickable(true);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Selector {
    /// 精确匹配文本
    pub(crate) text: Option<String>,
    /// 文本包含
    pub(crate) text_contains: Option<String>,
    /// 文本开头匹配
    pub(crate) text_starts_with: Option<String>,
    /// 文本正则匹配
    pub(crate) text_matches: Option<String>,
    /// 资源 ID
    pub(crate) resource_id: Option<String>,
    /// 类名
    pub(crate) class_name: Option<String>,
    /// 描述
    pub(crate) description: Option<String>,
    /// 描述包含
    pub(crate) description_contains: Option<String>,
    /// 包名
    pub(crate) package_name: Option<String>,
    /// 是否可点击
    pub(crate) clickable: Option<bool>,
    /// 是否启用
    pub(crate) enabled: Option<bool>,
    /// 是否可获得焦点
    pub(crate) focusable: Option<bool>,
    /// 是否可滚动
    pub(crate) scrollable: Option<bool>,
    /// 实例索引（当有多个匹配元素时）
    pub(crate) instance: Option<u32>,

    // === 扩展布尔字段 ===
    /// 是否可勾选
    pub(crate) checkable: Option<bool>,
    /// 是否已勾选
    pub(crate) checked: Option<bool>,
    /// 是否可长按
    pub(crate) long_clickable: Option<bool>,
    /// 是否已获得焦点
    pub(crate) focused: Option<bool>,
    /// 是否已选中
    pub(crate) selected: Option<bool>,

    // === Matches 正则匹配字段 ===
    /// 类名正则匹配
    pub(crate) class_name_matches: Option<String>,
    /// 描述正则匹配
    pub(crate) description_matches: Option<String>,
    /// 描述开头匹配
    pub(crate) description_starts_with: Option<String>,
    /// 包名正则匹配
    pub(crate) package_name_matches: Option<String>,
    /// 资源 ID 正则匹配
    pub(crate) resource_id_matches: Option<String>,

    // === Index 字段 ===
    /// 元素在父容器中的索引位置
    pub(crate) index: Option<u32>,

    // === 层级选择器 ===
    /// 子元素/兄弟元素关系类型（"child" 或 "sibling"）
    pub(crate) child_or_sibling: Vec<String>,
    /// 子元素/兄弟元素选择器
    pub(crate) child_or_sibling_selector: Vec<Box<Selector>>,
}

impl Selector {
    /// 创建新的选择器
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置精确匹配文本
    ///
    /// # 参数
    ///
    /// * `text` - 要匹配的文本
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().text("Settings");
    /// ```
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// 设置文本包含条件
    ///
    /// # 参数
    ///
    /// * `text` - 文本应包含的子串
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().text_contains("Set");
    /// ```
    pub fn text_contains(mut self, text: impl Into<String>) -> Self {
        self.text_contains = Some(text.into());
        self
    }

    /// 设置文本开头匹配条件
    ///
    /// # 参数
    ///
    /// * `text` - 文本应以此开头
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().text_starts_with("Set");
    /// ```
    pub fn text_starts_with(mut self, text: impl Into<String>) -> Self {
        self.text_starts_with = Some(text.into());
        self
    }

    /// 设置文本正则匹配条件
    ///
    /// # 参数
    ///
    /// * `pattern` - 正则表达式模式
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().text_matches("Set.*");
    /// ```
    pub fn text_matches(mut self, pattern: impl Into<String>) -> Self {
        self.text_matches = Some(pattern.into());
        self
    }

    /// 设置资源 ID
    ///
    /// # 参数
    ///
    /// * `id` - 资源 ID（如 "com.example:id/button"）
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().resource_id("com.example:id/button");
    /// ```
    pub fn resource_id(mut self, id: impl Into<String>) -> Self {
        self.resource_id = Some(id.into());
        self
    }

    /// 设置类名
    ///
    /// # 参数
    ///
    /// * `name` - 类名（如 "android.widget.TextView"）
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().class_name("android.widget.TextView");
    /// ```
    pub fn class_name(mut self, name: impl Into<String>) -> Self {
        self.class_name = Some(name.into());
        self
    }

    /// 设置描述
    ///
    /// # 参数
    ///
    /// * `desc` - 内容描述
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().description("Submit button");
    /// ```
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// 设置描述包含条件
    ///
    /// # 参数
    ///
    /// * `desc` - 描述应包含的子串
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().description_contains("Submit");
    /// ```
    pub fn description_contains(mut self, desc: impl Into<String>) -> Self {
        self.description_contains = Some(desc.into());
        self
    }

    /// 设置包名
    ///
    /// # 参数
    ///
    /// * `name` - 应用包名
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().package_name("com.example.app");
    /// ```
    pub fn package_name(mut self, name: impl Into<String>) -> Self {
        self.package_name = Some(name.into());
        self
    }

    /// 设置是否可点击
    ///
    /// # 参数
    ///
    /// * `clickable` - true 表示只匹配可点击元素
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().clickable(true);
    /// ```
    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = Some(clickable);
        self
    }

    /// 设置是否启用
    ///
    /// # 参数
    ///
    /// * `enabled` - true 表示只匹配启用的元素
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().enabled(true);
    /// ```
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// 设置是否可获得焦点
    ///
    /// # 参数
    ///
    /// * `focusable` - true 表示只匹配可获得焦点的元素
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().focusable(true);
    /// ```
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = Some(focusable);
        self
    }

    /// 设置是否可滚动
    ///
    /// # 参数
    ///
    /// * `scrollable` - true 表示只匹配可滚动元素
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().scrollable(true);
    /// ```
    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = Some(scrollable);
        self
    }

    /// 设置实例索引
    ///
    /// 当有多个元素匹配相同条件时，使用此参数选择特定实例。
    /// 索引从 0 开始。
    ///
    /// # 参数
    ///
    /// * `instance` - 实例索引（从 0 开始）
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// // 选择第二个匹配的元素
    /// let selector = Selector::new()
    ///     .text("Item")
    ///     .instance(1);
    /// ```
    pub fn instance(mut self, instance: u32) -> Self {
        self.instance = Some(instance);
        self
    }

    /// 设置是否可勾选
    ///
    /// # 参数
    ///
    /// * `checkable` - true 表示只匹配可勾选元素
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().checkable(true);
    /// ```
    pub fn checkable(mut self, checkable: bool) -> Self {
        self.checkable = Some(checkable);
        self
    }

    /// 设置是否已勾选
    ///
    /// # 参数
    ///
    /// * `checked` - true 表示只匹配已勾选元素
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().checked(true);
    /// ```
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// 设置是否可长按
    ///
    /// # 参数
    ///
    /// * `long_clickable` - true 表示只匹配可长按元素
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().long_clickable(true);
    /// ```
    pub fn long_clickable(mut self, long_clickable: bool) -> Self {
        self.long_clickable = Some(long_clickable);
        self
    }

    /// 设置是否已获得焦点
    ///
    /// # 参数
    ///
    /// * `focused` - true 表示只匹配已获得焦点的元素
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().focused(true);
    /// ```
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = Some(focused);
        self
    }

    /// 设置是否已选中
    ///
    /// # 参数
    ///
    /// * `selected` - true 表示只匹配已选中元素
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().selected(true);
    /// ```
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// 设置类名正则匹配条件
    ///
    /// # 参数
    ///
    /// * `pattern` - 正则表达式模式
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().class_name_matches(".*Button$");
    /// ```
    pub fn class_name_matches(mut self, pattern: impl Into<String>) -> Self {
        self.class_name_matches = Some(pattern.into());
        self
    }

    /// 设置描述正则匹配条件
    ///
    /// # 参数
    ///
    /// * `pattern` - 正则表达式模式
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().description_matches("Submit.*");
    /// ```
    pub fn description_matches(mut self, pattern: impl Into<String>) -> Self {
        self.description_matches = Some(pattern.into());
        self
    }

    /// 设置描述开头匹配条件
    ///
    /// # 参数
    ///
    /// * `text` - 描述应以此开头
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().description_starts_with("Submit");
    /// ```
    pub fn description_starts_with(mut self, text: impl Into<String>) -> Self {
        self.description_starts_with = Some(text.into());
        self
    }

    /// 设置包名正则匹配条件
    ///
    /// # 参数
    ///
    /// * `pattern` - 正则表达式模式
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().package_name_matches(r"com\\.example\\..*");
    /// ```
    pub fn package_name_matches(mut self, pattern: impl Into<String>) -> Self {
        self.package_name_matches = Some(pattern.into());
        self
    }

    /// 设置资源 ID 正则匹配条件
    ///
    /// # 参数
    ///
    /// * `pattern` - 正则表达式模式
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new().resource_id_matches(".*:id/btn_.*");
    /// ```
    pub fn resource_id_matches(mut self, pattern: impl Into<String>) -> Self {
        self.resource_id_matches = Some(pattern.into());
        self
    }

    /// 设置元素在父容器中的索引位置
    ///
    /// 注意：index 和 instance 的区别：
    /// - index: 元素在父容器中的位置（从 0 开始）
    /// - instance: 匹配结果中的索引（从 0 开始）
    ///
    /// # 参数
    ///
    /// * `index` - 元素在父容器中的索引（从 0 开始）
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new()
    ///     .class_name("android.widget.TextView")
    ///     .index(2);
    /// ```
    pub fn index(mut self, index: u32) -> Self {
        self.index = Some(index);
        self
    }

    /// 定位当前元素的子元素
    ///
    /// # 参数
    ///
    /// * `child_selector` - 子元素的选择条件
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new()
    ///     .resource_id("com.example:id/list")
    ///     .child(Selector::new().text("Item"));
    /// ```
    pub fn child(mut self, child_selector: Selector) -> Self {
        self.child_or_sibling.push("child".to_string());
        self.child_or_sibling_selector
            .push(Box::new(child_selector));
        self
    }

    /// 定位当前元素的兄弟元素
    ///
    /// # 参数
    ///
    /// * `sibling_selector` - 兄弟元素的选择条件
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let selector = Selector::new()
    ///     .text("Username")
    ///     .sibling(Selector::new().class_name("android.widget.EditText"));
    /// ```
    pub fn sibling(mut self, sibling_selector: Selector) -> Self {
        self.child_or_sibling.push("sibling".to_string());
        self.child_or_sibling_selector
            .push(Box::new(sibling_selector));
        self
    }

    /// 转换为 JSON-RPC 参数
    ///
    /// 将选择器转换为 JSON 对象，用于 JSON-RPC 调用。
    /// 只包含已设置的字段，并自动计算 mask 字段。
    ///
    /// # 返回
    ///
    /// JSON 对象，包含所有已设置的选择条件和 mask 字段
    ///
    /// # Examples
    ///
    /// ```
    /// use uiautomator::Selector;
    ///
    /// let params = Selector::new()
    ///     .text("Settings")
    ///     .clickable(true)
    ///     .to_params();
    ///
    /// assert_eq!(params["text"], "Settings");
    /// assert_eq!(params["clickable"], true);
    /// assert!(params.get("mask").is_some());
    /// ```
    pub fn to_params(&self) -> Value {
        let mut params = serde_json::Map::new();
        let mut mask: u32 = 0;

        if let Some(ref text) = self.text {
            params.insert("text".to_string(), json!(text));
            mask |= MASK_TEXT;
        }
        if let Some(ref text) = self.text_contains {
            params.insert("textContains".to_string(), json!(text));
            mask |= MASK_TEXT_CONTAINS;
        }
        if let Some(ref text) = self.text_starts_with {
            params.insert("textStartsWith".to_string(), json!(text));
            mask |= MASK_TEXT_STARTS_WITH;
        }
        if let Some(ref pattern) = self.text_matches {
            params.insert("textMatches".to_string(), json!(pattern));
            mask |= MASK_TEXT_MATCHES;
        }
        if let Some(ref id) = self.resource_id {
            params.insert("resourceId".to_string(), json!(id));
            mask |= MASK_RESOURCE_ID;
        }
        if let Some(ref name) = self.class_name {
            params.insert("className".to_string(), json!(name));
            mask |= MASK_CLASS_NAME;
        }
        if let Some(ref desc) = self.description {
            params.insert("description".to_string(), json!(desc));
            mask |= MASK_DESCRIPTION;
        }
        if let Some(ref desc) = self.description_contains {
            params.insert("descriptionContains".to_string(), json!(desc));
            mask |= MASK_DESCRIPTION_CONTAINS;
        }
        if let Some(ref name) = self.package_name {
            params.insert("packageName".to_string(), json!(name));
            mask |= MASK_PACKAGE_NAME;
        }
        if let Some(clickable) = self.clickable {
            params.insert("clickable".to_string(), json!(clickable));
            mask |= MASK_CLICKABLE;
        }
        if let Some(enabled) = self.enabled {
            params.insert("enabled".to_string(), json!(enabled));
            mask |= MASK_ENABLED;
        }
        if let Some(focusable) = self.focusable {
            params.insert("focusable".to_string(), json!(focusable));
            mask |= MASK_FOCUSABLE;
        }
        if let Some(scrollable) = self.scrollable {
            params.insert("scrollable".to_string(), json!(scrollable));
            mask |= MASK_SCROLLABLE;
        }
        if let Some(instance) = self.instance {
            params.insert("instance".to_string(), json!(instance));
            mask |= MASK_INSTANCE;
        }

        // 扩展布尔字段
        if let Some(checkable) = self.checkable {
            params.insert("checkable".to_string(), json!(checkable));
            mask |= MASK_CHECKABLE;
        }
        if let Some(checked) = self.checked {
            params.insert("checked".to_string(), json!(checked));
            mask |= MASK_CHECKED;
        }
        if let Some(long_clickable) = self.long_clickable {
            params.insert("longClickable".to_string(), json!(long_clickable));
            mask |= MASK_LONG_CLICKABLE;
        }
        if let Some(focused) = self.focused {
            params.insert("focused".to_string(), json!(focused));
            mask |= MASK_FOCUSED;
        }
        if let Some(selected) = self.selected {
            params.insert("selected".to_string(), json!(selected));
            mask |= MASK_SELECTED;
        }

        // Matches 正则匹配字段
        if let Some(ref pattern) = self.class_name_matches {
            params.insert("classNameMatches".to_string(), json!(pattern));
            mask |= MASK_CLASS_NAME_MATCHES;
        }
        if let Some(ref pattern) = self.description_matches {
            params.insert("descriptionMatches".to_string(), json!(pattern));
            mask |= MASK_DESCRIPTION_MATCHES;
        }
        if let Some(ref text) = self.description_starts_with {
            params.insert("descriptionStartsWith".to_string(), json!(text));
            mask |= MASK_DESCRIPTION_STARTS_WITH;
        }
        if let Some(ref pattern) = self.package_name_matches {
            params.insert("packageNameMatches".to_string(), json!(pattern));
            mask |= MASK_PACKAGE_NAME_MATCHES;
        }
        if let Some(ref pattern) = self.resource_id_matches {
            params.insert("resourceIdMatches".to_string(), json!(pattern));
            mask |= MASK_RESOURCE_ID_MATCHES;
        }

        // Index 字段
        if let Some(index) = self.index {
            params.insert("index".to_string(), json!(index));
            mask |= MASK_INDEX;
        }

        // 添加 mask 字段
        params.insert("mask".to_string(), json!(mask));

        // 层级选择器
        if !self.child_or_sibling.is_empty() {
            params.insert("childOrSibling".to_string(), json!(self.child_or_sibling));
            let child_selectors: Vec<Value> = self
                .child_or_sibling_selector
                .iter()
                .map(|s| s.to_params())
                .collect();
            params.insert("childOrSiblingSelector".to_string(), json!(child_selectors));
        }

        Value::Object(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试需求 3.1: 通过 text 属性定位元素
    #[test]
    fn test_selector_text() {
        let selector = Selector::new().text("Settings");

        assert_eq!(selector.text, Some("Settings".to_string()));

        let params = selector.to_params();
        assert_eq!(params["text"], "Settings");
        // 验证 mask 字段存在
        assert!(params.get("mask").is_some());
        assert_eq!(params["mask"], 0x01);
    }

    // 测试需求 3.2: 通过 resourceId 属性定位元素
    #[test]
    fn test_selector_resource_id() {
        let selector = Selector::new().resource_id("com.example:id/button");

        assert_eq!(
            selector.resource_id,
            Some("com.example:id/button".to_string())
        );

        let params = selector.to_params();
        assert_eq!(params["resourceId"], "com.example:id/button");
        // 验证 mask 字段存在
        assert!(params.get("mask").is_some());
        assert_eq!(params["mask"], 0x200000);
    }

    // 测试需求 3.3: 通过 className 属性定位元素
    #[test]
    fn test_selector_class_name() {
        let selector = Selector::new().class_name("android.widget.TextView");

        assert_eq!(
            selector.class_name,
            Some("android.widget.TextView".to_string())
        );

        let params = selector.to_params();
        assert_eq!(params["className"], "android.widget.TextView");
        // 验证 mask 字段存在
        assert!(params.get("mask").is_some());
        assert_eq!(params["mask"], 0x10);
    }

    // 测试需求 3.4: 通过 description 属性定位元素
    #[test]
    fn test_selector_description() {
        let selector = Selector::new().description("Submit button");

        assert_eq!(selector.description, Some("Submit button".to_string()));

        let params = selector.to_params();
        assert_eq!(params["description"], "Submit button");
        // 验证 mask 字段存在
        assert!(params.get("mask").is_some());
        assert_eq!(params["mask"], 0x40);
    }

    // 测试需求 3.5: 组合多个属性定位元素
    #[test]
    fn test_selector_multiple_conditions() {
        let selector = Selector::new()
            .text("Settings")
            .class_name("android.widget.TextView")
            .clickable(true)
            .enabled(true);

        assert_eq!(selector.text, Some("Settings".to_string()));
        assert_eq!(
            selector.class_name,
            Some("android.widget.TextView".to_string())
        );
        assert_eq!(selector.clickable, Some(true));
        assert_eq!(selector.enabled, Some(true));

        let params = selector.to_params();
        assert_eq!(params["text"], "Settings");
        assert_eq!(params["className"], "android.widget.TextView");
        assert_eq!(params["clickable"], true);
        assert_eq!(params["enabled"], true);
        // 验证 mask 字段存在并正确计算
        assert!(params.get("mask").is_some());
        // mask = 0x01 (text) | 0x10 (className) | 0x1000 (clickable) | 0x8000 (enabled)
        assert_eq!(params["mask"], 0x01 | 0x10 | 0x1000 | 0x8000);
    }

    // 测试需求 3.7: 通过 instance 参数选择特定实例
    #[test]
    fn test_selector_instance() {
        let selector = Selector::new().text("Item").instance(2);

        assert_eq!(selector.text, Some("Item".to_string()));
        assert_eq!(selector.instance, Some(2));

        let params = selector.to_params();
        assert_eq!(params["text"], "Item");
        assert_eq!(params["instance"], 2);
        // 验证 mask 字段存在
        assert!(params.get("mask").is_some());
        assert_eq!(params["mask"], 0x01 | 0x01000000);
    }

    // 测试文本包含条件
    #[test]
    fn test_selector_text_contains() {
        let selector = Selector::new().text_contains("Set");

        let params = selector.to_params();
        assert_eq!(params["textContains"], "Set");
    }

    // 测试文本开头匹配
    #[test]
    fn test_selector_text_starts_with() {
        let selector = Selector::new().text_starts_with("Set");

        let params = selector.to_params();
        assert_eq!(params["textStartsWith"], "Set");
    }

    // 测试文本正则匹配
    #[test]
    fn test_selector_text_matches() {
        let selector = Selector::new().text_matches("Set.*");

        let params = selector.to_params();
        assert_eq!(params["textMatches"], "Set.*");
    }

    // 测试布尔属性
    #[test]
    fn test_selector_boolean_properties() {
        let selector = Selector::new()
            .clickable(true)
            .enabled(false)
            .focusable(true)
            .scrollable(false);

        let params = selector.to_params();
        assert_eq!(params["clickable"], true);
        assert_eq!(params["enabled"], false);
        assert_eq!(params["focusable"], true);
        assert_eq!(params["scrollable"], false);
    }

    // 测试空选择器
    #[test]
    fn test_selector_empty() {
        let selector = Selector::new();

        let params = selector.to_params();
        // 空选择器应该只有 mask 字段，值为 0
        assert_eq!(params.as_object().unwrap().len(), 1);
        assert_eq!(params["mask"], 0);
    }

    // === Mask 计算测试 ===

    // 测试单字段 mask 值
    #[test]
    fn test_mask_single_field_text() {
        let selector = Selector::new().text("test");
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x01);
    }

    #[test]
    fn test_mask_single_field_text_contains() {
        let selector = Selector::new().text_contains("test");
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x02);
    }

    #[test]
    fn test_mask_single_field_text_matches() {
        let selector = Selector::new().text_matches("test.*");
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x04);
    }

    #[test]
    fn test_mask_single_field_text_starts_with() {
        let selector = Selector::new().text_starts_with("test");
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x08);
    }

    #[test]
    fn test_mask_single_field_class_name() {
        let selector = Selector::new().class_name("android.widget.TextView");
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x10);
    }

    #[test]
    fn test_mask_single_field_description() {
        let selector = Selector::new().description("desc");
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x40);
    }

    #[test]
    fn test_mask_single_field_description_contains() {
        let selector = Selector::new().description_contains("desc");
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x80);
    }

    #[test]
    fn test_mask_single_field_clickable() {
        let selector = Selector::new().clickable(true);
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x1000);
    }

    #[test]
    fn test_mask_single_field_scrollable() {
        let selector = Selector::new().scrollable(true);
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x4000);
    }

    #[test]
    fn test_mask_single_field_enabled() {
        let selector = Selector::new().enabled(true);
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x8000);
    }

    #[test]
    fn test_mask_single_field_focusable() {
        let selector = Selector::new().focusable(true);
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x010000);
    }

    #[test]
    fn test_mask_single_field_package_name() {
        let selector = Selector::new().package_name("com.example");
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x080000);
    }

    #[test]
    fn test_mask_single_field_resource_id() {
        let selector = Selector::new().resource_id("com.example:id/button");
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x200000);
    }

    #[test]
    fn test_mask_single_field_instance() {
        let selector = Selector::new().instance(0);
        let params = selector.to_params();
        assert_eq!(params["mask"], 0x01000000);
    }

    // 测试多字段组合 mask 值
    #[test]
    fn test_mask_combination_text_clickable() {
        let selector = Selector::new().text("test").clickable(true);
        let params = selector.to_params();
        // mask = 0x01 (text) | 0x1000 (clickable) = 0x1001
        assert_eq!(params["mask"], 0x1001);
    }

    #[test]
    fn test_mask_combination_three_fields() {
        let selector = Selector::new()
            .text("test")
            .class_name("android.widget.TextView")
            .enabled(true);
        let params = selector.to_params();
        // mask = 0x01 (text) | 0x10 (className) | 0x8000 (enabled) = 0x8011
        assert_eq!(params["mask"], 0x8011);
    }

    #[test]
    fn test_mask_combination_all_text_fields() {
        let selector = Selector::new()
            .text("test")
            .text_contains("contains")
            .text_starts_with("starts")
            .text_matches("matches");
        let params = selector.to_params();
        // mask = 0x01 | 0x02 | 0x08 | 0x04 = 0x0F
        assert_eq!(params["mask"], 0x0F);
    }

    #[test]
    fn test_mask_combination_all_boolean_fields() {
        let selector = Selector::new()
            .clickable(true)
            .scrollable(true)
            .enabled(true)
            .focusable(true);
        let params = selector.to_params();
        // mask = 0x1000 | 0x4000 | 0x8000 | 0x010000 = 0x01D000
        assert_eq!(params["mask"], 0x01D000);
    }

    #[test]
    fn test_mask_combination_complex() {
        let selector = Selector::new()
            .text("Settings")
            .resource_id("com.example:id/settings")
            .class_name("android.widget.TextView")
            .clickable(true)
            .enabled(true)
            .instance(0);
        let params = selector.to_params();
        // mask = 0x01 | 0x200000 | 0x10 | 0x1000 | 0x8000 | 0x01000000
        assert_eq!(params["mask"], 0x01209011);
    }

    // 测试链式调用
    #[test]
    fn test_selector_chaining() {
        let selector = Selector::new()
            .text("Settings")
            .resource_id("com.example:id/settings")
            .class_name("android.widget.TextView")
            .description("Settings button")
            .clickable(true)
            .enabled(true)
            .instance(0);

        let params = selector.to_params();
        assert_eq!(params["text"], "Settings");
        assert_eq!(params["resourceId"], "com.example:id/settings");
        assert_eq!(params["className"], "android.widget.TextView");
        assert_eq!(params["description"], "Settings button");
        assert_eq!(params["clickable"], true);
        assert_eq!(params["enabled"], true);
        assert_eq!(params["instance"], 0);
    }

    // 测试 Default trait
    #[test]
    fn test_selector_default() {
        let selector1 = Selector::new();
        let selector2 = Selector::default();

        assert_eq!(selector1, selector2);
    }

    // 测试 Clone trait
    #[test]
    fn test_selector_clone() {
        let selector1 = Selector::new().text("Settings").clickable(true);

        let selector2 = selector1.clone();

        assert_eq!(selector1, selector2);
    }

    // 测试所有字符串字段
    #[test]
    fn test_selector_all_string_fields() {
        let selector = Selector::new()
            .text("text")
            .text_contains("contains")
            .text_starts_with("starts")
            .text_matches("matches")
            .resource_id("id")
            .class_name("class")
            .description("desc")
            .description_contains("desc_contains")
            .package_name("package");

        let params = selector.to_params();
        assert_eq!(params["text"], "text");
        assert_eq!(params["textContains"], "contains");
        assert_eq!(params["textStartsWith"], "starts");
        assert_eq!(params["textMatches"], "matches");
        assert_eq!(params["resourceId"], "id");
        assert_eq!(params["className"], "class");
        assert_eq!(params["description"], "desc");
        assert_eq!(params["descriptionContains"], "desc_contains");
        assert_eq!(params["packageName"], "package");
    }

    // === 扩展布尔字段测试 ===

    #[test]
    fn test_mask_single_field_checkable() {
        let selector = Selector::new().checkable(true);
        let params = selector.to_params();
        assert_eq!(params["checkable"], true);
        assert_eq!(params["mask"], 0x0400);
    }

    #[test]
    fn test_mask_single_field_checked() {
        let selector = Selector::new().checked(true);
        let params = selector.to_params();
        assert_eq!(params["checked"], true);
        assert_eq!(params["mask"], 0x0800);
    }

    #[test]
    fn test_mask_single_field_long_clickable() {
        let selector = Selector::new().long_clickable(true);
        let params = selector.to_params();
        assert_eq!(params["longClickable"], true);
        assert_eq!(params["mask"], 0x2000);
    }

    #[test]
    fn test_mask_single_field_focused() {
        let selector = Selector::new().focused(true);
        let params = selector.to_params();
        assert_eq!(params["focused"], true);
        assert_eq!(params["mask"], 0x020000);
    }

    #[test]
    fn test_mask_single_field_selected() {
        let selector = Selector::new().selected(true);
        let params = selector.to_params();
        assert_eq!(params["selected"], true);
        assert_eq!(params["mask"], 0x040000);
    }

    #[test]
    fn test_mask_combination_all_extended_boolean_fields() {
        let selector = Selector::new()
            .checkable(true)
            .checked(false)
            .long_clickable(true)
            .focused(true)
            .selected(false);
        let params = selector.to_params();
        // mask = 0x0400 | 0x0800 | 0x2000 | 0x020000 | 0x040000 = 0x062C00
        assert_eq!(
            params["mask"],
            0x0400 | 0x0800 | 0x2000 | 0x020000 | 0x040000
        );
        assert_eq!(params["checkable"], true);
        assert_eq!(params["checked"], false);
        assert_eq!(params["longClickable"], true);
        assert_eq!(params["focused"], true);
        assert_eq!(params["selected"], false);
    }

    // === Matches 正则匹配字段测试 ===

    #[test]
    fn test_mask_single_field_class_name_matches() {
        let selector = Selector::new().class_name_matches(".*Button$");
        let params = selector.to_params();
        assert_eq!(params["classNameMatches"], ".*Button$");
        assert_eq!(params["mask"], 0x20);
    }

    #[test]
    fn test_mask_single_field_description_matches() {
        let selector = Selector::new().description_matches("Submit.*");
        let params = selector.to_params();
        assert_eq!(params["descriptionMatches"], "Submit.*");
        assert_eq!(params["mask"], 0x0100);
    }

    #[test]
    fn test_mask_single_field_description_starts_with() {
        let selector = Selector::new().description_starts_with("Submit");
        let params = selector.to_params();
        assert_eq!(params["descriptionStartsWith"], "Submit");
        assert_eq!(params["mask"], 0x0200);
    }

    #[test]
    fn test_mask_single_field_package_name_matches() {
        let selector = Selector::new().package_name_matches(r"com\.example\..*");
        let params = selector.to_params();
        assert_eq!(params["packageNameMatches"], r"com\.example\..*");
        assert_eq!(params["mask"], 0x100000);
    }

    #[test]
    fn test_mask_single_field_resource_id_matches() {
        let selector = Selector::new().resource_id_matches(".*:id/btn_.*");
        let params = selector.to_params();
        assert_eq!(params["resourceIdMatches"], ".*:id/btn_.*");
        assert_eq!(params["mask"], 0x400000);
    }

    #[test]
    fn test_mask_combination_matches_fields() {
        let selector = Selector::new()
            .class_name_matches(".*Button$")
            .description_matches("Submit.*")
            .resource_id_matches(".*:id/btn_.*");
        let params = selector.to_params();
        // mask = 0x20 | 0x0100 | 0x400000
        assert_eq!(params["mask"], 0x20 | 0x0100 | 0x400000);
    }

    // === Index 字段测试 ===

    #[test]
    fn test_mask_single_field_index() {
        let selector = Selector::new().index(3);
        let params = selector.to_params();
        assert_eq!(params["index"], 3);
        assert_eq!(params["mask"], 0x800000);
    }

    #[test]
    fn test_index_vs_instance() {
        let selector = Selector::new()
            .class_name("android.widget.TextView")
            .index(2)
            .instance(1);
        let params = selector.to_params();
        assert_eq!(params["index"], 2);
        assert_eq!(params["instance"], 1);
        // mask = 0x10 (className) | 0x800000 (index) | 0x01000000 (instance)
        assert_eq!(params["mask"], 0x10 | 0x800000 | 0x01000000);
    }

    // === 层级选择器测试 ===

    #[test]
    fn test_child_selector() {
        let selector = Selector::new()
            .text("Parent")
            .child(Selector::new().text("Child"));
        let params = selector.to_params();
        assert_eq!(params["text"], "Parent");
        assert_eq!(params["childOrSibling"][0], "child");
        assert_eq!(params["childOrSiblingSelector"][0]["text"], "Child");
        assert_eq!(params["childOrSiblingSelector"][0]["mask"], 0x01);
    }

    #[test]
    fn test_sibling_selector() {
        let selector = Selector::new()
            .text("Label")
            .sibling(Selector::new().class_name("android.widget.EditText"));
        let params = selector.to_params();
        assert_eq!(params["childOrSibling"][0], "sibling");
        assert_eq!(
            params["childOrSiblingSelector"][0]["className"],
            "android.widget.EditText"
        );
        assert_eq!(params["childOrSiblingSelector"][0]["mask"], 0x10);
    }

    #[test]
    fn test_nested_child_selector() {
        let selector = Selector::new().resource_id("list").child(
            Selector::new()
                .class_name("Item")
                .child(Selector::new().text("Title")),
        );
        let params = selector.to_params();
        assert_eq!(params["childOrSibling"][0], "child");
        let child = &params["childOrSiblingSelector"][0];
        assert_eq!(child["className"], "Item");
        // 子选择器也有自己的 childOrSibling
        assert_eq!(child["childOrSibling"][0], "child");
        assert_eq!(child["childOrSiblingSelector"][0]["text"], "Title");
    }

    #[test]
    fn test_no_child_or_sibling_fields_when_empty() {
        let selector = Selector::new().text("test");
        let params = selector.to_params();
        assert!(params.get("childOrSibling").is_none());
        assert!(params.get("childOrSiblingSelector").is_none());
    }
}
