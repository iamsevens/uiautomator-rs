# 常见问题解答（FAQ）

## 安装相关

### Q1: 如何在没有网络的环境中使用？

**A**: `uiautomator-cli` 的所有资源文件都已嵌入到可执行文件中，完全支持离线使用。

步骤：
1. 在有网络的环境下载预编译二进制文件
2. 将二进制文件复制到离线环境
3. 直接使用，无需额外下载

### Q2: 安装脚本失败怎么办？

**A**: 可以手动下载并安装：

**Linux/macOS**:
```bash
# 下载
curl -L https://github.com/iamsevens/uiautomator-rs/releases/latest/download/uiautomator-linux-x86_64 -o uiautomator

# 添加执行权限
chmod +x uiautomator

# 移动到 PATH 中的目录
sudo mv uiautomator /usr/local/bin/
```

**Windows**:
1. 从 GitHub Releases 下载 `uiautomator-windows-x86_64.exe`
2. 重命名为 `uiautomator.exe`
3. 将文件放到 PATH 中的目录（如 `C:\Windows\System32`）

### Q3: 如何更新到最新版本？

**A**: 重新运行安装脚本即可：

```bash
# Linux/macOS
curl -sSL https://raw.githubusercontent.com/.../install.sh | bash

# Windows (PowerShell)
iwr -useb https://raw.githubusercontent.com/.../install.ps1 | iex
```

或手动下载最新版本替换旧文件。

## 设备连接相关

### Q4: 提示"未找到连接的设备"怎么办？

**A**: 按以下步骤排查：

1. **检查 USB 连接**
   ```bash
   adb devices
   ```
   应该能看到设备列表

2. **启动 ADB 服务**
   ```bash
   adb start-server
   ```

3. **检查 USB 调试**
   - 在设备上进入"开发者选项"
   - 启用"USB 调试"
   - 连接时允许调试授权

4. **检查 USB 驱动**（Windows）
   - 确保安装了正确的 USB 驱动
   - 可以使用 Google USB Driver 或设备厂商提供的驱动

### Q5: 如何连接网络设备（通过 WiFi）？

**A**: 使用 ADB 的网络连接功能：

```bash
# 1. 首先通过 USB 连接设备
adb devices

# 2. 启用网络 ADB（设备需要 root 或开发者模式）
adb tcpip 5555

# 3. 断开 USB，通过 WiFi 连接
adb connect 192.168.1.100:5555

# 4. 使用 uiautomator-cli
uiautomator init --serial 192.168.1.100:5555
```

### Q6: 多个设备连接时如何选择特定设备？

**A**: 使用 `--serial` 参数：

```bash
# 1. 查看所有设备
adb devices

# 输出示例:
# List of devices attached
# 127.0.0.1:5555    device
# emulator-5554     device

# 2. 指定设备
uiautomator init --serial 127.0.0.1:5555
uiautomator status --serial emulator-5554
```

## 安装和初始化相关

### Q7: 初始化失败，提示权限不足？

**A**: 可能的原因和解决方案：

1. **ADB 授权未允许**
   - 设备上会弹出"允许 USB 调试"提示
   - 勾选"始终允许"并点击"允许"

2. **设备存储空间不足**
   ```bash
   adb shell df -h
   ```
   确保 `/data/local/tmp` 有足够空间（至少 50MB）

3. **SELinux 限制**（部分设备）
   ```bash
   adb shell setenforce 0
   ```

### Q8: 初始化成功但服务无法启动？

**A**: 按以下步骤排查：

1. **检查进程**
   ```bash
   adb shell ps | grep atx-agent
   ```

2. **查看日志**
   ```bash
   adb logcat | grep atx-agent
   ```

3. **手动启动**
   ```bash
   adb shell /data/local/tmp/atx-agent -d
   ```

4. **检查端口占用**
   ```bash
   adb shell netstat -tuln | grep 7912
   ```

5. **强制重新安装**
   ```bash
   uiautomator init --force
   ```

### Q9: 如何验证安装是否成功？

**A**: 使用以下命令：

```bash
# 1. 查看状态
uiautomator status

# 输出示例:
# ✓ ATX-Agent 状态: 运行中
#   版本: 0.10.0
#   端口: 7912

# 2. 测试连接
curl http://127.0.0.1:7912/version

# 3. 使用 uiautomator 库测试
# （如果你在使用 Rust 库）
```

## 服务管理相关

### Q10: 重启服务后仍然无法连接？

**A**: 尝试以下步骤：

1. **完全卸载后重新安装**
   ```bash
   uiautomator uninstall
   uiautomator init
   ```

2. **检查防火墙**
   - 确保端口 7912 未被防火墙阻止
   - Windows: 检查 Windows Defender 防火墙
   - Linux: 检查 iptables 或 firewalld

3. **重启设备**
   ```bash
   adb reboot
   # 等待设备重启后
   uiautomator init
   ```

### Q11: 卸载后仍有残留文件？

**A**: 手动清理：

```bash
# 删除 atx-agent 二进制文件
adb shell rm -f /data/local/tmp/atx-agent

# 卸载 APK
adb uninstall com.github.uiautomator
adb uninstall com.github.uiautomator.test

# 清理日志文件
adb shell rm -rf /sdcard/atx-agent
```

### Q12: 如何查看 ATX-Agent 的详细日志？

**A**: 使用 ADB logcat：

```bash
# 实时查看日志
adb logcat | grep atx-agent

# 保存日志到文件
adb logcat | grep atx-agent > atx-agent.log

# 查看特定级别的日志
adb logcat *:E | grep atx-agent  # 只看错误
```

## 跨平台相关

### Q13: Windows 上提示"无法识别的命令"？

**A**: 可能的原因：

1. **未添加到 PATH**
   - 检查 `uiautomator.exe` 是否在 PATH 中
   - 或使用完整路径运行

2. **PowerShell 执行策略**
   ```powershell
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

3. **使用 CMD 而非 PowerShell**
   - 某些功能在 CMD 中可能表现不同

### Q14: macOS 上提示"无法打开，因为无法验证开发者"？

**A**: 允许运行未签名的应用：

```bash
# 方法 1: 移除隔离属性
xattr -d com.apple.quarantine /usr/local/bin/uiautomator

# 方法 2: 在系统偏好设置中允许
# 系统偏好设置 -> 安全性与隐私 -> 通用 -> 仍要打开
```

### Q15: Linux 上提示"权限被拒绝"？

**A**: 添加执行权限：

```bash
chmod +x /usr/local/bin/uiautomator

# 或者使用 sudo
sudo chmod +x /usr/local/bin/uiautomator
```

## 性能和优化相关

### Q16: 初始化速度很慢？

**A**: 可能的原因和优化：

1. **USB 连接速度慢**
   - 使用 USB 3.0 接口
   - 更换质量更好的 USB 线

2. **设备性能较低**
   - 关闭不必要的后台应用
   - 清理设备存储空间

3. **ADB 服务问题**
   ```bash
   adb kill-server
   adb start-server
   ```

### Q17: 如何减少资源占用？

**A**: ATX-Agent 本身占用很少的资源（约 10-20MB 内存）。如果需要进一步优化：

1. **不使用时停止服务**
   ```bash
   adb shell pkill atx-agent
   ```

2. **使用时再启动**
   ```bash
   uiautomator restart
   ```

## 开发和调试相关

### Q18: 如何启用详细日志？

**A**: 使用环境变量：

```bash
# Linux/macOS
RUST_LOG=debug uiautomator init

# Windows (PowerShell)
$env:RUST_LOG="debug"; uiautomator init

# Windows (CMD)
set RUST_LOG=debug && uiautomator init
```

### Q19: 如何从源码构建？

**A**: 参考开发指南：

```bash
# 1. 克隆仓库
git clone https://github.com/iamsevens/uiautomator-rs.git
cd uiautomator-rs/uiautomator-cli

# 2. 下载资源文件
cd assets
./download_atx_agent.sh  # Linux/macOS
# 或
.\download_atx_agent.ps1  # Windows

# 3. 构建
cd ..
cargo build --release

# 4. 运行
./target/release/uiautomator --help
```

### Q20: 如何贡献代码？

**A**: 欢迎贡献！步骤：

1. Fork 本仓库
2. 创建特性分支：`git checkout -b feature/my-feature`
3. 遵循 TDD 流程开发
4. 确保所有测试通过：`cargo test`
5. 提交代码：`git commit -am 'Add my feature'`
6. 推送分支：`git push origin feature/my-feature`
7. 创建 Pull Request

详见 [开发指南](README.md#开发指南)

## 其他问题

### Q21: 支持哪些 Android 版本？

**A**: 支持 Android 5.0（API 21）及以上版本。推荐使用 Android 7.0（API 24）及以上。

### Q22: 是否支持模拟器？

**A**: 是的，支持以下模拟器：
- Android Emulator（官方）
- Genymotion
- BlueStacks
- 其他支持 ADB 的模拟器

### Q23: 如何报告 Bug？

**A**: 在 GitHub 上创建 Issue：

1. 访问 https://github.com/iamsevens/uiautomator-rs/issues
2. 点击"New Issue"
3. 提供以下信息：
   - 操作系统和版本
   - `uiautomator version` 输出
   - 完整的错误消息
   - 复现步骤

### Q24: 有没有图形界面版本？

**A**: 目前只提供命令行版本。如果需要图形界面，可以考虑：
- 使用 Python uiautomator2 的 weditor
- 使用 Android Studio 的 Layout Inspector

### Q25: 如何获取帮助？

**A**: 多种方式：

1. **查看文档**
   - [README.md](README.md)
   - [FAQ.md](FAQ.md)（本文档）
   - [开发指南](README.md#开发指南)

2. **命令行帮助**
   ```bash
   uiautomator --help
   uiautomator init --help
   ```

3. **GitHub Issues**
   - 搜索已有问题
   - 创建新 Issue

4. **社区讨论**
   - GitHub Discussions
   - 相关技术论坛

---

如果你的问题未在此列出，请在 GitHub 上创建 Issue。
