[English](./README.md) | [简体中文](./README.zh-CN.md)

# uiautomator-cli

Android UI 自动化命令行工具 - ATX-Agent 管理器

## 简介

`uiautomator-cli` 是一个独立的命令行工具，用于简化 Android 设备上 ATX-Agent 的安装和管理。它将所有必需的资源文件嵌入到可执行文件中，提供开箱即用的体验，无需额外下载或配置。

## 特性

- ✅ **简单易用** - 一条命令完成设备初始化
- ✅ **离线可用** - 内置所有资源文件，无需网络连接
- ✅ **跨平台** - 支持 Linux、macOS、Windows
- ✅ **独立部署** - 提供预编译二进制文件，无需 Rust 环境

## 快速开始

### 安装

#### 方法 1: 下载预编译二进制文件（推荐）

```bash
# Linux x86_64
curl -L https://github.com/iamsevens/uiautomator-rs/releases/latest/download/uiautomator-linux-x86_64 -o /usr/local/bin/uiautomator
chmod +x /usr/local/bin/uiautomator

# macOS
curl -L https://github.com/iamsevens/uiautomator-rs/releases/latest/download/uiautomator-macos-x86_64 -o /usr/local/bin/uiautomator
chmod +x /usr/local/bin/uiautomator

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/iamsevens/uiautomator-rs/releases/latest/download/uiautomator-windows-x86_64.exe -OutFile uiautomator.exe
```

#### 方法 2: 从源码编译

```bash
# 1. 克隆仓库
git clone https://github.com/iamsevens/uiautomator-rs.git
cd uiautomator-rs/uiautomator-cli

# 2. 下载资源文件
cd assets
./download_atx_agent.sh  # Linux/macOS
# 或
.\download_atx_agent.ps1  # Windows

# 3. 编译
cd ..
cargo build --release

# 4. 安装
cp target/release/uiautomator /usr/local/bin/  # Linux/macOS
```

### 使用

```bash
# 初始化设备（一次性操作）
uiautomator init

# 查看状态
uiautomator status

# 重启服务
uiautomator restart

# 卸载
uiautomator uninstall

# 查看帮助
uiautomator --help
```

## 命令参考

### `uiautomator init`

初始化设备，安装并启动 ATX-Agent 服务。

```bash
uiautomator init [OPTIONS]

选项:
  -f, --force          强制重新安装
  -s, --serial <SERIAL>  指定设备序列号
  -h, --help           显示帮助信息
```

示例:
```bash
# 初始化第一个连接的设备
uiautomator init

# 强制重新安装
uiautomator init --force

# 初始化指定设备
uiautomator init --serial 127.0.0.1:5555
```

### `uiautomator status`

查看 ATX-Agent 运行状态。

```bash
uiautomator status [OPTIONS]

选项:
  -s, --serial <SERIAL>  指定设备序列号
  -h, --help           显示帮助信息
```

### `uiautomator restart`

重启 ATX-Agent 服务。

```bash
uiautomator restart [OPTIONS]

选项:
  -s, --serial <SERIAL>  指定设备序列号
  -h, --help           显示帮助信息
```

### `uiautomator uninstall`

卸载 ATX-Agent。

```bash
uiautomator uninstall [OPTIONS]

选项:
  -s, --serial <SERIAL>  指定设备序列号
  -h, --help           显示帮助信息
```

### `uiautomator version`

显示版本信息。

```bash
uiautomator version
```

## 常见问题

### 设备未找到

**问题**: 运行 `uiautomator init` 时提示"未找到连接的设备"

**解决方案**:
1. 检查 USB 连接
2. 运行 `adb devices` 确认设备可见
3. 在设备上启用 USB 调试模式
4. 确保 ADB 服务正在运行：`adb start-server`

### 权限不足

**问题**: 安装失败，提示权限不足

**解决方案**:
1. 确保设备已 root 或使用开发者模式
2. 检查 ADB 授权（设备上会弹出授权提示）
3. 在设备上允许 USB 调试授权

### 内网环境使用

**问题**: 在内网环境中无法下载资源文件

**解决方案**:
使用预编译的二进制文件，所有资源已内置，无需网络连接。

### 多设备管理

**问题**: 如何在多个设备之间切换？

**解决方案**:
使用 `--serial` 参数指定设备：
```bash
# 查看所有设备
adb devices

# 初始化指定设备
uiautomator init --serial 127.0.0.1:5555
```

### 服务无法启动

**问题**: 初始化成功但服务无法启动

**解决方案**:
1. 检查设备存储空间是否充足
2. 尝试重启设备
3. 使用 `--force` 强制重新安装：`uiautomator init --force`
4. 查看 ADB 日志：`adb logcat | grep atx-agent`

### 端口冲突

**问题**: ATX-Agent 默认端口 7912 被占用

**解决方案**:
1. 检查端口占用：`adb shell netstat -tuln | grep 7912`
2. 重启 ATX-Agent 服务：`uiautomator restart`
3. 如果问题持续，卸载后重新安装：
   ```bash
   uiautomator uninstall
   uiautomator init
   ```

### Windows 防火墙问题

**问题**: Windows 上无法连接到设备

**解决方案**:
1. 允许 ADB 通过防火墙
2. 以管理员身份运行命令提示符
3. 检查 Windows Defender 是否阻止了连接

### 离线环境使用

**问题**: 完全离线环境如何使用？

**解决方案**:
1. 在有网络的环境下载预编译二进制文件
2. 将二进制文件复制到离线环境
3. 所有资源文件已内置，可直接使用

## 开发指南

### 前置要求

- Rust 1.70+
- Android SDK（用于 ADB）
- Android 设备（用于测试）

### 从源码构建

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

### TDD 开发流程

本项目采用 **测试驱动开发（TDD）** 方法：

#### 1. Red（红）- 先写测试

```bash
# 创建测试文件
touch tests/my_feature_test.rs

# 编写失败的测试
cargo test my_feature  # 测试失败 ❌
```

#### 2. Green（绿）- 实现功能

```bash
# 编写最小化代码使测试通过
cargo test my_feature  # 测试通过 ✅
```

#### 3. Refactor（重构）- 优化代码

```bash
# 重构代码，保持测试通过
cargo test  # 所有测试通过 ✅
```

### 测试策略

#### 单元测试

测试单个模块的功能：

```bash
# 运行所有单元测试
cargo test --lib

# 运行特定模块的测试
cargo test --lib resources
cargo test --lib installer
```

#### 集成测试

测试完整的命令流程（需要真实设备）：

```bash
# 运行集成测试
cargo test --test integration_init_test -- --ignored
cargo test --test integration_service_test -- --ignored

# 运行所有集成测试
cargo test --test '*' -- --ignored
```

#### 属性测试（Property-Based Testing）

使用 `proptest` 验证通用属性：

```bash
# 运行属性测试
cargo test property

# 运行特定属性测试
cargo test property_resources
cargo test property_idempotent
```

### 项目结构

```
uiautomator-cli/
├── src/
│   ├── main.rs           # CLI 入口
│   ├── cli.rs            # 命令行参数定义
│   ├── commands.rs       # 命令实现
│   ├── installer.rs      # 安装逻辑
│   ├── resources.rs      # 资源文件管理
│   ├── error.rs          # 错误处理
│   └── lib.rs            # 库入口
├── tests/
│   ├── cli_test.rs                 # CLI 参数测试
│   ├── resources_test.rs           # 资源文件测试
│   ├── installer_test.rs           # 安装器测试
│   ├── error_test.rs               # 错误处理测试
│   ├── integration_init_test.rs    # init 命令集成测试
│   ├── integration_service_test.rs # 服务管理集成测试
│   ├── status_test.rs              # status 命令测试
│   ├── uninstall_test.rs           # uninstall 命令测试
│   ├── version_test.rs             # version 命令测试
│   ├── property_resources_test.rs  # 资源完整性属性测试
│   └── property_idempotent_test.rs # 幂等性属性测试
├── assets/               # 资源文件（构建时嵌入）
│   ├── atx-agent
│   ├── app-uiautomator.apk
│   └── app-uiautomator-test.apk
├── build.rs              # 构建脚本
└── Cargo.toml            # 项目配置
```

### 添加新功能

遵循 TDD 流程：

1. **编写测试**：在 `tests/` 目录创建测试文件
2. **运行测试**：确认测试失败（Red）
3. **实现功能**：在 `src/` 目录编写代码
4. **运行测试**：确认测试通过（Green）
5. **重构代码**：优化实现，保持测试通过（Refactor）

### 代码风格

```bash
# 格式化代码
cargo fmt

# 检查代码规范
cargo clippy

# 检查代码覆盖率
cargo tarpaulin --out Html
```

### 调试技巧

```bash
# 启用详细日志
RUST_LOG=debug cargo run -- init

# 使用 rust-gdb 调试
rust-gdb target/debug/uiautomator

# 查看 ADB 日志
adb logcat | grep atx-agent
```

### 发布流程

1. **更新版本号**：修改 `Cargo.toml` 中的 `version`
2. **运行所有测试**：`cargo test`
3. **构建发布版本**：`cargo build --release`
4. **创建 Git 标签**：`git tag v0.1.0`
5. **推送标签**：`git push origin v0.1.0`
6. **GitHub Actions 自动构建并发布**

### 贡献指南

1. Fork 本仓库
2. 创建特性分支：`git checkout -b feature/my-feature`
3. 遵循 TDD 流程开发
4. 确保所有测试通过：`cargo test`
5. 提交代码：`git commit -am 'Add my feature'`
6. 推送分支：`git push origin feature/my-feature`
7. 创建 Pull Request

### 资源文件更新

当需要更新嵌入的资源文件时：

```bash
# 1. 下载最新资源文件
cd assets
./download_atx_agent.sh

# 2. 验证资源文件
cd ..
cargo run --example verify_assets

# 3. 重新构建（build.rs 会自动计算新的 MD5）
cargo build --release
```

## 文档

- [README.md](README.md) - 快速开始和基本使用
- [CHANGELOG.md](CHANGELOG.md) - 版本变更记录
- [FAQ.md](FAQ.md) - 常见问题解答
- [CONTRIBUTING.md](CONTRIBUTING.md) - 贡献指南和开发流程
- [SETUP.md](SETUP.md) - 详细的设置说明
- [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) - 第三方资源与许可证说明
- [GitHub Tests](https://github.com/iamsevens/uiautomator-rs/tree/main/uiautomator-cli/tests) - 完整测试用例（发布包不包含）

发布说明：
- crates 发布包仅包含运行所需文件，不包含 `tests/**`。

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

详细的贡献指南请参考 [CONTRIBUTING.md](CONTRIBUTING.md)。

### 快速开始贡献

1. Fork 本仓库
2. 创建特性分支：`git checkout -b feature/my-feature`
3. 遵循 TDD 流程开发
4. 提交代码：`git commit -am 'feat: 添加新功能'`
5. 推送分支：`git push origin feature/my-feature`
6. 创建 Pull Request

## 社区

- **Issues**: [GitHub Issues](https://github.com/iamsevens/uiautomator-rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/iamsevens/uiautomator-rs/discussions)
- **文档**: [在线文档](https://github.com/iamsevens/uiautomator-rs/tree/main/docs)

## 致谢

感谢所有贡献者！

特别感谢：
- [uiautomator2](https://github.com/openatx/uiautomator2) - Python 版本的灵感来源
- [atx-agent](https://github.com/openatx/atx-agent) - ATX-Agent 服务

