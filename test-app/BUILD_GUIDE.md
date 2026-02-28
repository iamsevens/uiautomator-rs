# 快速构建指南

## 方式 1: 使用 Android Studio（推荐）

1. **打开项目**
   - 启动 Android Studio
   - File -> Open -> 选择 `test-app` 目录
   - 等待 Gradle 同步完成

2. **构建 APK**
   - Build -> Build Bundle(s) / APK(s) -> Build APK(s)
   - 或点击工具栏的 "Build" 按钮

3. **APK 位置**
   ```
   test-app/app/build/outputs/apk/debug/app-debug.apk
   ```

4. **安装到设备**
   - 连接设备或启动模拟器
   - Run -> Run 'app'
   - 或使用命令: `adb install app/build/outputs/apk/debug/app-debug.apk`

## 方式 2: 使用命令行

### Windows

```cmd
cd test-app

# 首次构建（会自动下载 Gradle）
gradlew.bat assembleDebug

# 安装到设备
adb install app\build\outputs\apk\debug\app-debug.apk
```

### Linux/macOS

```bash
cd test-app

# 赋予执行权限
chmod +x gradlew

# 构建
./gradlew assembleDebug

# 安装到设备
adb install app/build/outputs/apk/debug/app-debug.apk
```

## 常见问题

### 1. Gradle 下载慢

编辑 `build.gradle`，在 `repositories` 中添加阿里云镜像：

```gradle
repositories {
    maven { url 'https://maven.aliyun.com/repository/google/' }
    maven { url 'https://maven.aliyun.com/repository/public/' }
    google()
    mavenCentral()
}
```

### 2. Java 版本问题

确保安装了 JDK 8 或更高版本：

```bash
java -version
```

### 3. Android SDK 未找到

设置环境变量：

```bash
# Windows
set ANDROID_HOME=%LOCALAPPDATA%\Android\Sdk

# Linux/macOS
export ANDROID_HOME=$HOME/Android/Sdk
```

### 4. 首次构建时间长

首次构建会下载依赖，需要 5-10 分钟，请耐心等待。

## 验证安装

```bash
# 查看已安装的应用
adb shell pm list packages | grep uiautomator

# 启动应用
adb shell am start -n com.uiautomator.testapp/.MainActivity

# 查看应用日志
adb logcat | grep UIAutomator
```

## 下一步

构建成功后，可以开始编写 Rust 测试脚本了！

参考 `../uiautomator/examples/test_app/` 目录中的测试示例。
