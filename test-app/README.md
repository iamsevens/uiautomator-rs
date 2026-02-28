# UIAutomator Test App

这是一个专门用于测试 UIAutomator 功能的 Android 应用。

## 快速开始

### 前置要求

1. **Java JDK 17+**
   ```bash
   java -version
   ```

2. **Android SDK** (可选，Gradle 会自动下载)
   ```bash
   # 如果没有，设置环境变量
   export ANDROID_HOME=/path/to/android-sdk
   ```

3. **Gradle 8.11.1** (已包含在项目中)
   - 本地 Gradle 位置: `test-app/gradle/gradle-8.11.1-bin.zip`
   - 配置文件: `gradle/wrapper/gradle-wrapper.properties`
   - **无需额外下载**,直接使用本地版本

### 编译 APK

```bash
# 1. 进入项目目录
cd test-app

# 2. 赋予执行权限（Linux/macOS）
chmod +x gradlew

# 3. 编译 Debug APK
./gradlew assembleDebug

# Windows 使用
gradlew.bat assembleDebug

# 4. APK 位置
# app/build/outputs/apk/debug/app-debug.apk
```

**注意**: 项目已配置使用本地 Gradle 8.11.1,不会从网络下载。如需更换 Gradle 版本:
1. 将新版本的 `gradle-x.x.x-bin.zip` 放到 `test-app/gradle/` 目录
2. 修改 `gradle/wrapper/gradle-wrapper.properties` 中的 `distributionUrl=../gradle-x.x.x-bin.zip`

### 安装到设备

```bash
# 安装 APK
adb install app/build/outputs/apk/debug/app-debug.apk

# 启动应用
adb shell am start -n com.uiautomator.testapp/.MainActivity
```

### 清理构建

```bash
./gradlew clean
```

## 应用结构

### 主页面 (MainActivity)
- 测试场景选择菜单
- 跳转到各个测试页面

### 测试页面

1. **Basic Controls** - 基础控件测试
   - Button, TextView, CheckBox, RadioButton, Switch
   - 测试点击、状态切换

2. **Gestures** - 手势测试
   - 点击、长按、双击
   - 滑动、拖拽
   - 显示操作结果

3. **Input & Forms** - 输入和表单测试
   - EditText 输入
   - Spinner 下拉选择
   - 表单提交验证

4. **Lists & Scrolling** - 列表和滚动测试
   - ListView (100 项)
   - RecyclerView (1000 项)
   - ScrollView 长内容

5. **Dialogs & Popups** - 对话框测试
   - AlertDialog
   - ConfirmDialog
   - CustomDialog
   - BottomSheet

6. **Navigation** - 页面导航测试
   - 多级页面跳转
   - 返回栈测试
   - Intent 传递

7. **Animations** - 动画测试
   - 淡入淡出
   - 移动动画
   - 旋转缩放

8. **Stress Test** - 压力测试
   - 快速点击
   - 内存压力
   - 动画压力

9. **Concurrent Test** - 并发测试
   - 多个计数器
   - 同时操作测试

## Resource IDs 规范

所有控件都有明确的 resource ID，格式：
```
com.uiautomator.testapp:id/{控件名称}
```

例如：
- `com.uiautomator.testapp:id/btn_basic_controls`
- `com.uiautomator.testapp:id/tv_result`
- `com.uiautomator.testapp:id/et_username`

## 包名

```
com.uiautomator.testapp
```

## 主 Activity

```
com.uiautomator.testapp.MainActivity
```

## 测试脚本

配合 Rust UIAutomator 库使用：

```rust
const TEST_APP_PACKAGE: &str = "com.uiautomator.testapp";
const TEST_APP_ACTIVITY: &str = ".MainActivity";

// 启动应用
device.app_start(TEST_APP_PACKAGE, Some(TEST_APP_ACTIVITY)).await?;

// 点击按钮
device.find(Selector::new()
    .resource_id("com.uiautomator.testapp:id/btn_basic_controls"))
    .click(None, None).await?;
```

## 故障排查

### Gradle 下载慢
```bash
# 使用国内镜像（修改 build.gradle）
maven { url 'https://maven.aliyun.com/repository/public/' }
maven { url 'https://maven.aliyun.com/repository/google/' }
```

### 编译失败
```bash
# 清理后重新编译
./gradlew clean
./gradlew assembleDebug --stacktrace
```

### 签名问题
Debug APK 使用默认签名，无需配置。

## 许可证

MIT
