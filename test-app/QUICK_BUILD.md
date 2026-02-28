# 快速构建指南

## ⚠️ 自动构建遇到问题

由于网络限制，无法自动下载 Gradle wrapper 文件。但不用担心，你已经安装了 Android Studio，可以非常简单地构建！

## ✅ 推荐方式：使用 Android Studio（最简单）

### 步骤 1: 打开项目
```
1. 启动 Android Studio
2. 点击 "Open"
3. 选择目录: <repo-root>\test-app
4. 点击 "OK"
```

### 步骤 2: 等待同步
- Android Studio 会自动下载所需的 Gradle 和依赖
- 首次同步需要 5-10 分钟
- 底部会显示进度条

### 步骤 3: 构建 APK
```
方式 1: 使用菜单
Build -> Build Bundle(s) / APK(s) -> Build APK(s)

方式 2: 使用快捷键
Ctrl + F9 (Windows/Linux)
Cmd + F9 (macOS)

方式 3: 使用 Gradle 面板
右侧 Gradle 面板 -> app -> Tasks -> build -> assembleDebug
```

### 步骤 4: 找到 APK
构建成功后，Android Studio 会弹出通知，点击 "locate" 即可找到 APK。

或者手动查找：
```
<repo-root>\test-app\app\build\outputs\apk\debug\app-debug.apk
```

## 🔧 备选方式：修复 Gradle Wrapper

如果你想使用命令行，需要先修复 Gradle wrapper：

### 方式 1: 从 Android Studio 生成
```
1. 在 Android Studio 中打开项目
2. 打开 Terminal (底部工具栏)
3. 运行: gradle wrapper
4. 之后就可以使用 gradlew.bat 了
```

### 方式 2: 手动下载
```
1. 访问: https://services.gradle.org/distributions/gradle-8.0-bin.zip
2. 下载并解压到: C:\gradle\gradle-8.0
3. 设置环境变量: GRADLE_HOME=C:\gradle\gradle-8.0
4. 添加到 PATH: %GRADLE_HOME%\bin
5. 在项目目录运行: gradle wrapper
```

## 📱 安装到设备

构建成功后：

```bash
# 连接设备或启动模拟器
adb devices

# 安装 APK
adb install app\build\outputs\apk\debug\app-debug.apk

# 启动应用
adb shell am start -n com.uiautomator.testapp/.MainActivity
```

## 🧪 运行测试

APK 安装后，运行 Rust 测试：

```bash
cd <repo-root>\uiautomator
cargo run --example test_app_suite
```

## 💡 提示

1. **首次构建慢是正常的** - Gradle 需要下载依赖
2. **使用 Android Studio 最简单** - 它会自动处理所有配置
3. **确保网络畅通** - 下载依赖需要网络连接
4. **可以使用国内镜像** - 如果下载慢，可以配置阿里云镜像

## 🎯 下一步

1. ✅ 使用 Android Studio 打开 `test-app` 目录
2. ✅ 等待 Gradle 同步完成
3. ✅ 点击 Build -> Build APK(s)
4. ✅ 安装到设备
5. ✅ 运行测试脚本

就这么简单！🚀
