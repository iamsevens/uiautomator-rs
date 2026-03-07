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
    pub(crate) child_or_sibling_selector: Vec<Selector>,
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
        self.child_or_sibling_selector.push(child_selector);
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
        self.child_or_sibling_selector.push(sibling_selector);
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
#[path = "selector_tests.rs"]
mod tests;
