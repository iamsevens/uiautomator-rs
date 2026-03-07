# 手动测试脚本

本目录包含用于手动测试 uiautomator-cli 的辅助脚本。

## 脚本列表

### 1. quick-test.sh - 快速功能测试

快速验证所有基本功能是否正常工作。

**用途**: 
- 日常开发后的快速验证
- CI/CD 流程中的冒烟测试
- 发布前的基本功能检查

**运行方式**:
```bash
cd uiautomator-cli/scripts
chmod +x quick-test.sh
./quick-test.sh
```

**测试内容**:
- ✓ CLI 工具是否可用
- ✓ version 命令
- ✓ help 命令
- ✓ 设备连接检查
- ✓ init 命令
- ✓ status 命令
- ✓ restart 命令

**预期时间**: 1-2 分钟

---

### 2. multi-device-test.sh - 多设备测试

测试 CLI 工具在多设备环境下的行为。

**用途**:
- 验证多设备支持
- 测试设备独立性
- 验证 --serial 参数

**前置条件**:
- 至少连接 2 个 Android 设备或模拟器

**运行方式**:
```bash
cd uiautomator-cli/scripts
chmod +x multi-device-test.sh
./multi-device-test.sh
```

**测试内容**:
- ✓ 检测所有连接的设备
- ✓ 为每个设备执行完整测试流程
- ✓ 测试设备独立性
- ✓ 测试默认设备选择
- ✓ 测试无效设备序列号处理

**预期时间**: 3-5 分钟（取决于设备数量）

---

### 3. error-test.sh - 错误场景测试

测试各种错误情况的处理。

**用途**:
- 验证错误消息的清晰度
- 确保错误处理的健壮性
- 验证帮助信息

**运行方式**:
```bash
cd uiautomator-cli/scripts
chmod +x error-test.sh
./error-test.sh
```

**测试内容**:
- ✓ 无效命令处理
- ✓ 无效选项处理
- ✓ 无效设备序列号处理
- ✓ 服务未安装错误
- ✓ 帮助信息显示
- ✓ 版本信息显示

**预期时间**: 1-2 分钟

---

### 4. offline-test.sh - 离线环境测试

测试 CLI 工具在完全离线环境下的功能。

**用途**:
- 验证内置资源文件的使用
- 确保无网络依赖
- 测试离线场景的用户体验

**前置条件**:
- Linux 系统（推荐，可以自动隔离网络）
- 或手动断开网络连接

**运行方式**:
```bash
cd uiautomator-cli/scripts
chmod +x offline-test.sh

# Linux 系统（需要 root 权限）
sudo ./offline-test.sh

# 其他系统（需要手动断网）
./offline-test.sh
```

**测试内容**:
- ✓ 网络隔离
- ✓ 验证网络已断开
- ✓ 离线环境下的 init 命令
- ✓ 离线环境下的其他命令
- ✓ 网络恢复

**预期时间**: 2-3 分钟

**注意事项**:
- 在 Linux 上，脚本会使用 iptables 自动隔离网络
- 在 macOS/Windows 上，需要手动断开网络
- 脚本会在测试完成后恢复网络配置

---

## 使用指南

### 完整测试流程

建议按以下顺序执行测试：

```bash
# 1. 快速功能测试（确保基本功能正常）
./quick-test.sh

# 2. 错误场景测试（验证错误处理）
./error-test.sh

# 3. 多设备测试（如果有多个设备）
./multi-device-test.sh

# 4. 离线环境测试（验证离线可用性）
sudo ./offline-test.sh  # Linux
# 或手动断网后运行: ./offline-test.sh
```

### 在 Windows 上运行

这些脚本是为 Bash 编写的，在 Windows 上可以使用：

1. **Git Bash**:
   ```bash
   cd uiautomator-cli/scripts
   ./quick-test.sh
   ```

2. **WSL (Windows Subsystem for Linux)**:
   ```bash
   cd /mnt/c/path/to/uiautomator-cli/scripts
   ./quick-test.sh
   ```

3. **PowerShell** (需要安装 Git for Windows):
   ```powershell
   cd uiautomator-cli\scripts
   bash quick-test.sh
   ```

### 在 macOS 上运行

```bash
cd uiautomator-cli/scripts
chmod +x *.sh
./quick-test.sh
```

### 在 Linux 上运行

```bash
cd uiautomator-cli/scripts
chmod +x *.sh
./quick-test.sh
```

---

## 测试结果解读

### 成功输出示例

```
==========================================
  uiautomator-cli 快速功能测试
==========================================

✓ CLI 工具已安装
✓ version 命令 ... 通过
✓ help 命令 ... 通过
✓ 找到 1 个设备
✓ 初始化成功
✓ 状态查询成功
✓ 重启成功
✓ 状态查询成功

==========================================
测试结果: 7 通过, 0 失败
==========================================
```

### 失败输出示例

```
==========================================
  uiautomator-cli 快速功能测试
==========================================

✓ CLI 工具已安装
✓ version 命令 ... 通过
✗ help 命令 ... 失败
✓ 找到 1 个设备
✗ 初始化失败

==========================================
测试结果: 3 通过, 2 失败
==========================================
```

---

## 故障排除

### 问题: 脚本无法执行

**解决方案**:
```bash
chmod +x *.sh
```

### 问题: 找不到 uiautomator 命令

**解决方案**:
1. 确保已编译 CLI 工具
2. 将 CLI 工具添加到 PATH
3. 或使用完整路径运行

```bash
# 编译
cd uiautomator-cli
cargo build --release

# 添加到 PATH（临时）
export PATH="$PWD/target/release:$PATH"

# 或使用完整路径
alias uiautomator="$PWD/target/release/uiautomator"
```

### 问题: 未找到设备

**解决方案**:
1. 检查设备连接: `adb devices`
2. 启用 USB 调试
3. 接受 USB 调试授权
4. 重启 ADB: `adb kill-server && adb start-server`

### 问题: 离线测试无法隔离网络

**解决方案**:
- Linux: 使用 `sudo` 运行脚本
- macOS/Windows: 手动断开网络连接

---

## 自定义测试

你可以基于这些脚本创建自己的测试：

```bash
#!/bin/bash
# my-custom-test.sh

# 引入颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# 你的测试逻辑
echo "运行自定义测试..."

if uiautomator version; then
    echo -e "${GREEN}✓ 测试通过${NC}"
else
    echo -e "${RED}✗ 测试失败${NC}"
fi
```

---

## 贡献

如果你发现测试脚本有问题或想添加新的测试场景，欢迎提交 PR！

---

## 相关文档

- [手动测试指南](../MANUAL_TEST_GUIDE.md) - 详细的手动测试步骤
- [手动测试报告模板](../MANUAL_TEST_REPORT.md) - 测试报告模板
- [README](../README.md) - 项目主文档
- [FAQ](../FAQ.md) - 常见问题解答
> These helper scripts are for local/manual checks only.  
> For release gating, use the canonical repository-root entrypoint:  
> `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\trigger-gh-release-gate.ps1 -Repo iamsevens/uiautomator-rs -Ref main`
