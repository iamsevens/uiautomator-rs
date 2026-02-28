# uiautomator-cli 开发环境设置

本文档说明如何设置 uiautomator-cli 的开发环境。

## 前置要求

1. **Rust 工具链** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Android SDK 和 ADB**
   - 确保 `adb` 命令在 PATH 中
   - 运行 `adb version` 验证安装

3. **Android 设备**（用于集成测试）
   - 启用 USB 调试
   - 连接到电脑

## 设置步骤

### 1. 克隆仓库

```bash
git clone https://github.com/iamsevens/uiautomator-rs.git
cd uiautomator-rs/uiautomator-cli
```

### 2. 下载资源文件

uiautomator-cli 需要以下资源文件：
- `atx-agent` - ATX-Agent 二进制文件（约 8-10MB）
- `app-uiautomator.apk` - UiAutomator2 APK（约 1.5MB）
- `app-uiautomator-test.apk` - UiAutomator2 测试 APK（约 1.5MB）

#### 方法 1: 使用下载脚本（推荐）

**Linux/macOS:**
```bash
cd assets
chmod +x download_atx_agent.sh
./download_atx_agent.sh
cd ..
```

**Windows PowerShell:**
```powershell
cd assets
.\download_atx_agent.ps1
cd ..
```

#### 方法 2: 从 uiautomator 库复制

如果你已经设置了 uiautomator 库：

**Linux/macOS:**
```bash
cd assets
cp ../../uiautomator/assets/app-uiautomator.apk ./
# 然后下载其他文件
./download_atx_agent.sh
cd ..
```

**Windows PowerShell:**
```powershell
cd assets
Copy-Item ..\..\uiautomator\assets\app-uiautomator.apk -Destination .\
# 然后下载其他文件
.\download_atx_agent.ps1
cd ..
```

### 3. 验证资源文件

```bash
# 检查文件是否存在
ls -lh assets/

# 应该看到:
# atx-agent (约 8-10MB)
# app-uiautomator.apk (约 1.5MB)
# app-uiautomator-test.apk (约 1.5MB)
```

### 4. 构建项目

```bash
cargo build
```

如果资源文件缺失，构建脚本会报错并提示你下载。

### 5. 运行测试

```bash
# 运行单元测试（不需要设备）
cargo test --lib

# 运行集成测试（需要连接设备）
cargo test --test '*' -- --ignored
```

## 开发工作流

### TDD 开发流程

本项目采用测试驱动开发（TDD）方法：

1. **Red（红）** - 先写测试，测试失败
2. **Green（绿）** - 编写代码使测试通过
3. **Refactor（重构）** - 优化代码

### 运行特定测试

```bash
# 运行特定测试文件
cargo test --test cli_test

# 运行特定测试函数
cargo test test_parse_init_command

# 运行属性测试
cargo test property
```

### 调试

```bash
# 启用详细日志
RUST_LOG=debug cargo run -- init

# 使用 rust-gdb 调试
rust-gdb target/debug/uiautomator
```

## 常见问题

### 构建失败：资源文件不存在

**错误信息:**
```
error: atx-agent 文件不存在，请先下载资源文件
```

**解决方案:**
运行资源文件下载脚本（见步骤 2）

### 测试失败：设备未连接

**错误信息:**
```
Error: 未找到连接的设备
```

**解决方案:**
1. 连接 Android 设备
2. 运行 `adb devices` 确认设备可见
3. 确保设备已启用 USB 调试

### Windows 上 tar 命令不可用

**错误信息:**
```
tar: command not found
```

**解决方案:**
Windows 10+ 自带 tar 命令。如果不可用，请：
1. 更新到 Windows 10 1803 或更高版本
2. 或手动解压 .tar.gz 文件

## 项目结构

```
uiautomator-cli/
├── Cargo.toml              # 项目配置
├── build.rs                # 构建脚本（计算资源文件 MD5）
├── src/
│   ├── main.rs             # CLI 入口
│   ├── commands/           # 命令实现（后续任务）
│   ├── installer.rs        # 安装逻辑（后续任务）
│   └── resources.rs        # 资源文件管理（后续任务）
├── assets/                 # 资源文件目录
│   ├── atx-agent           # ATX-Agent 二进制
│   ├── app-uiautomator.apk # UiAutomator APK
│   └── app-uiautomator-test.apk # UiAutomator Test APK
├── tests/                  # 测试目录
│   ├── cli_test.rs         # CLI 测试（后续任务）
│   ├── resources_test.rs   # 资源测试（后续任务）
│   └── ...                 # 其他测试
└── README.md               # 项目说明
```

## 下一步

完成设置后，你可以：
1. 查看 [tasks.md](../.kiro/specs/uiautomator-cli/tasks.md) 了解开发任务
2. 查看 [design.md](../.kiro/specs/uiautomator-cli/design.md) 了解设计文档
3. 开始实现功能（从任务 2 开始）
