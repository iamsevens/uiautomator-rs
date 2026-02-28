# 集成测试指南

本目录包含 uiautomator 的集成测试，需要真实的 Android 设备或模拟器。

## 前置条件

### 1. Android 设备或模拟器

**选项 A: 使用 Android 模拟器**
```bash
# 启动 Android Studio 的 AVD Manager
# 或使用命令行启动模拟器
emulator -avd <avd_name>
```

**选项 B: 使用真实设备**
- 启用开发者选项
- 启用 USB 调试
- 通过 USB 连接设备

### 2. ADB 工具

确保 ADB 已安装并在 PATH 中：
```bash
adb version
```

验证设备连接：
```bash
adb devices
```

应该看到类似输出：
```
List of devices attached
emulator-5554   device
```

### 3. 设备端服务

集成测试会自动安装和启动 UiAutomator2 服务，无需手动操作。

## 运行测试

### 运行所有集成测试

```bash
# 在项目根目录
cd uiautomator

# 运行所有集成测试
cargo test --test '*' -- --test-threads=1
```

**注意**: 使用 `--test-threads=1` 避免并发测试冲突。

### 运行特定测试文件

```bash
# 设备连接和信息测试
cargo test --test integration_device_test

# 元素定位和操作测试
cargo test --test integration_element_test

# 手势和按键测试
cargo test --test integration_gesture_test

# 截图和应用管理测试
cargo test --test integration_app_test

# 并发操作测试
cargo test --test integration_concurrent_test
```

### 运行特定测试用例

```bash
# 运行单个测试
cargo test --test integration_device_test test_device_connect_with_serial

# 运行匹配模式的测试
cargo test --test integration_element_test test_element_find
```

### 指定设备序列号

如果有多个设备，可以通过环境变量指定：

```bash
# Linux/macOS
export TEST_DEVICE_SERIAL="emulator-5554"
cargo test --test '*'

# Windows (PowerShell)
$env:TEST_DEVICE_SERIAL="emulator-5554"
cargo test --test '*'

# Windows (CMD)
set TEST_DEVICE_SERIAL=emulator-5554
cargo test --test '*'
```

### 查看详细日志

```bash
# 显示测试输出
cargo test --test '*' -- --nocapture

# 启用调试日志
RUST_LOG=debug cargo test --test '*' -- --nocapture
```

## 测试结构

### common/mod.rs
通用测试辅助函数和配置：
- `init_test_env()`: 初始化测试环境
- `connect_test_device()`: 连接到测试设备
- `skip_if_no_device!()`: 跳过测试宏
- `cleanup_test_env()`: 清理测试环境

### integration_device_test.rs
设备连接和信息获取测试：
- 设备连接（自动选择、指定序列号）
- 设备信息获取（屏幕尺寸、旋转、SDK 版本等）
- 多设备场景

### integration_element_test.rs
元素定位和操作测试：
- 通过各种条件定位元素（text, resourceId, className 等）
- 元素操作（点击、长按、文本操作）
- 元素等待和超时
- 元素信息获取

### integration_gesture_test.rs
手势和按键操作测试：
- 坐标点击、长按、双击
- 滑动和拖拽
- 百分比坐标
- 各种按键（Home, Back, Volume 等）

### integration_app_test.rs
截图和应用管理测试：
- 截图功能（PNG, JPEG）
- 应用启动、停止、清除
- 应用信息获取
- 应用等待

### integration_concurrent_test.rs
并发操作测试：
- 并发设备信息获取
- 并发截图
- 并发元素操作
- 资源清理验证

## 测试最佳实践

### 1. 测试隔离

每个测试应该：
- 独立运行，不依赖其他测试
- 清理测试环境（返回主屏幕）
- 不修改系统设置

### 2. 超时设置

使用合理的超时时间：
- 元素查找: 5-10 秒
- 应用启动: 10-20 秒
- 短操作: 2-5 秒

### 3. 等待 UI 稳定

在操作后等待 UI 稳定：
```rust
common::wait_ui_stable().await;
```

### 4. 错误处理

测试应该优雅地处理错误：
```rust
if !common::check_device_available().await {
    eprintln!("⚠️  跳过测试：没有可用的 Android 设备");
    return;
}
```

## 故障排查

### 问题: 找不到设备

```
Error: 没有可用的 Android 设备
```

**解决方案**:
1. 检查设备连接: `adb devices`
2. 重启 ADB 服务: `adb kill-server && adb start-server`
3. 检查 USB 调试是否启用

### 问题: 服务启动失败

```
Error: UiAutomator 服务未连接
```

**解决方案**:
1. 检查设备存储空间
2. 手动推送 JAR: `adb push assets/u2.jar /data/local/tmp/`
3. 检查设备权限

### 问题: 测试超时

```
Error: 操作超时
```

**解决方案**:
1. 增加超时时间
2. 检查设备性能
3. 确保 UI 已加载完成

### 问题: 元素未找到

```
Error: UI 对象未找到
```

**解决方案**:
1. 使用 `adb shell uiautomator dump` 查看 UI 层级
2. 调整选择器条件
3. 增加等待时间

## 持续集成

在 CI 环境中运行测试：

```yaml
# GitHub Actions 示例
- name: 启动 Android 模拟器
  uses: reactivecircus/android-emulator-runner@v2
  with:
    api-level: 29
    script: cargo test --test '*' -- --test-threads=1
```

## 性能基准

典型测试执行时间（在模拟器上）：
- 设备连接测试: ~5 秒
- 元素操作测试: ~30 秒
- 手势测试: ~20 秒
- 应用管理测试: ~40 秒
- 并发测试: ~30 秒

**总计**: ~2-3 分钟

## 贡献指南

添加新的集成测试时：
1. 遵循现有测试结构
2. 使用 `common` 模块的辅助函数
3. 添加清晰的注释和文档
4. 确保测试可以独立运行
5. 清理测试环境

## 参考资源

- [Android Debug Bridge (ADB)](https://developer.android.com/studio/command-line/adb)
- [UiAutomator](https://developer.android.com/training/testing/ui-automator)
- [Tokio 异步测试](https://tokio.rs/tokio/topics/testing)
