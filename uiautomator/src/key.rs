//! Android 按键定义
//!
//! 本模块定义了 Android 系统的常用按键及其键码。

/// Android 按键枚举
///
/// 包含 Android 系统中常用的物理按键和软键。
/// 每个按键都对应一个 Android KeyEvent 键码。
///
/// # Examples
///
/// ```
/// use uiautomator::Key;
///
/// let keys = [Key::Home, Key::Back, Key::Enter];
/// let keycodes: Vec<u32> = keys.iter().map(Key::to_keycode).collect();
/// assert_eq!(keycodes, vec![3, 4, 66]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// Home 键 - 返回主屏幕
    Home,
    /// Back 键 - 返回上一个界面
    Back,
    /// Power 键 - 电源键，切换屏幕点亮状态
    Power,
    /// 音量增加键
    VolumeUp,
    /// 音量减少键
    VolumeDown,
    /// 静音键
    VolumeMute,
    /// 菜单键
    Menu,
    /// 搜索键
    Search,
    /// 回车键/确认键
    Enter,
    /// 删除键
    Delete,
    /// 最近任务键
    Recent,
    /// 相机键
    Camera,
    /// 方向键 - 上
    Up,
    /// 方向键 - 下
    Down,
    /// 方向键 - 左
    Left,
    /// 方向键 - 右
    Right,
    /// 方向键 - 中心/确认
    Center,
    /// Tab 键
    Tab,
    /// 空格键
    Space,
    /// 退出键
    Escape,
    /// 播放/暂停键
    MediaPlayPause,
    /// 停止键
    MediaStop,
    /// 下一曲
    MediaNext,
    /// 上一曲
    MediaPrevious,
    /// 快进
    MediaFastForward,
    /// 快退
    MediaRewind,
    /// 通话键
    Call,
    /// 挂断键
    EndCall,
}

impl Key {
    const ALL_KEYS: &'static [Key] = &[
        Key::Home,
        Key::Back,
        Key::Power,
        Key::VolumeUp,
        Key::VolumeDown,
        Key::VolumeMute,
        Key::Menu,
        Key::Search,
        Key::Enter,
        Key::Delete,
        Key::Recent,
        Key::Camera,
        Key::Up,
        Key::Down,
        Key::Left,
        Key::Right,
        Key::Center,
        Key::Tab,
        Key::Space,
        Key::Escape,
        Key::MediaPlayPause,
        Key::MediaStop,
        Key::MediaNext,
        Key::MediaPrevious,
        Key::MediaFastForward,
        Key::MediaRewind,
        Key::Call,
        Key::EndCall,
    ];

    fn metadata(&self) -> (u32, &'static str) {
        match self {
            Key::Home => (3, "home"),
            Key::Back => (4, "back"),
            Key::Power => (26, "power"),
            Key::VolumeUp => (24, "volume_up"),
            Key::VolumeDown => (25, "volume_down"),
            Key::VolumeMute => (164, "volume_mute"),
            Key::Menu => (82, "menu"),
            Key::Search => (84, "search"),
            Key::Enter => (66, "enter"),
            Key::Delete => (67, "delete"),
            Key::Recent => (187, "recent"),
            Key::Camera => (27, "camera"),
            Key::Up => (19, "up"),
            Key::Down => (20, "down"),
            Key::Left => (21, "left"),
            Key::Right => (22, "right"),
            Key::Center => (23, "center"),
            Key::Tab => (61, "tab"),
            Key::Space => (62, "space"),
            Key::Escape => (111, "escape"),
            Key::MediaPlayPause => (85, "media_play_pause"),
            Key::MediaStop => (86, "media_stop"),
            Key::MediaNext => (87, "media_next"),
            Key::MediaPrevious => (88, "media_previous"),
            Key::MediaFastForward => (90, "media_fast_forward"),
            Key::MediaRewind => (89, "media_rewind"),
            Key::Call => (5, "call"),
            Key::EndCall => (6, "end_call"),
        }
    }

    /// 将按键转换为 Android KeyEvent 键码
    ///
    /// # 返回
    ///
    /// 对应的 Android KeyEvent 键码
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Key;
    ///
    /// assert_eq!(Key::Home.to_keycode(), 3);
    /// assert_eq!(Key::Back.to_keycode(), 4);
    /// assert_eq!(Key::Power.to_keycode(), 26);
    /// ```
    pub fn to_keycode(&self) -> u32 {
        self.metadata().0
    }

    /// 将按键转换为名称字符串
    ///
    /// # 返回
    ///
    /// 按键的名称字符串（小写，用下划线分隔）
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Key;
    ///
    /// assert_eq!(Key::Home.to_name(), "home");
    /// assert_eq!(Key::VolumeUp.to_name(), "volume_up");
    /// assert_eq!(Key::MediaPlayPause.to_name(), "media_play_pause");
    /// ```
    pub fn to_name(&self) -> &'static str {
        self.metadata().1
    }

    /// 从名称字符串创建按键
    ///
    /// # 参数
    ///
    /// * `name` - 按键名称（不区分大小写）
    ///
    /// # 返回
    ///
    /// 如果名称有效则返回对应的按键，否则返回 None
    ///
    /// # 示例
    ///
    /// ```
    /// use uiautomator::Key;
    ///
    /// assert_eq!(Key::from_name("home"), Some(Key::Home));
    /// assert_eq!(Key::from_name("HOME"), Some(Key::Home));
    /// assert_eq!(Key::from_name("volume_up"), Some(Key::VolumeUp));
    /// assert_eq!(Key::from_name("invalid"), None);
    /// ```
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL_KEYS
            .iter()
            .copied()
            .find(|key| key.to_name().eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_keycode() {
        assert_eq!(Key::Home.to_keycode(), 3);
        assert_eq!(Key::Back.to_keycode(), 4);
        assert_eq!(Key::Power.to_keycode(), 26);
        assert_eq!(Key::VolumeUp.to_keycode(), 24);
        assert_eq!(Key::VolumeDown.to_keycode(), 25);
        assert_eq!(Key::Enter.to_keycode(), 66);
        assert_eq!(Key::Up.to_keycode(), 19);
        assert_eq!(Key::Down.to_keycode(), 20);
        assert_eq!(Key::Left.to_keycode(), 21);
        assert_eq!(Key::Right.to_keycode(), 22);
    }

    #[test]
    fn test_key_to_name() {
        assert_eq!(Key::Home.to_name(), "home");
        assert_eq!(Key::Back.to_name(), "back");
        assert_eq!(Key::VolumeUp.to_name(), "volume_up");
        assert_eq!(Key::MediaPlayPause.to_name(), "media_play_pause");
    }

    #[test]
    fn test_key_from_name() {
        assert_eq!(Key::from_name("home"), Some(Key::Home));
        assert_eq!(Key::from_name("HOME"), Some(Key::Home));
        assert_eq!(Key::from_name("back"), Some(Key::Back));
        assert_eq!(Key::from_name("volume_up"), Some(Key::VolumeUp));
        assert_eq!(Key::from_name("VOLUME_UP"), Some(Key::VolumeUp));
        assert_eq!(Key::from_name("invalid"), None);
    }

    #[test]
    fn test_key_round_trip() {
        // 测试所有按键的名称转换往返
        for key in Key::ALL_KEYS {
            let name = key.to_name();
            let parsed = Key::from_name(name);
            assert_eq!(parsed, Some(*key), "Failed round trip for key: {:?}", key);
        }
    }

    #[test]
    fn test_all_keys_have_unique_keycodes() {
        use std::collections::HashSet;

        let mut keycodes = HashSet::new();
        for key in Key::ALL_KEYS {
            let keycode = key.to_keycode();
            assert!(
                keycodes.insert(keycode),
                "Duplicate keycode {} for key {:?}",
                keycode,
                key
            );
        }
    }
}
