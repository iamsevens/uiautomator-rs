# AGENTS.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## 项目概述

Rust 实现的 Android UI 自动化测试库，复刻 Python uiautomator2 的核心功能。通过 ADB 与 Android 设备通信，使用 JSON-RPC 协议调用设备端 UiAutomator 服务。

## 仓库结构

本仓库包含三个独立项目（无 workspace，各自独立编译）：

- **`uiautomator/`** — 核心 Rust 库，提供 Device、Selector、UiObject 等 API
- **`uiautomator-cli/`** — CLI 工具，依赖 `uiautomator` 库（path 依赖），用于管理设备上的 ATX-Agent
- **`test-app/`** — Android 测试应用（Gradle 项目），用于集成测试验证

## 平台支持

支持 Windows、Linux、macOS 平台。

## 构建和测试命令

所有 cargo 命令需在对应子目录下执行（无顶层 workspace）。

### uiautomator 库

```bash
# 构建
cd uiautomator && cargo build

# 格式化 + lint
cargo fmt
cargo clippy

# 运行所有单元测试
cargo test --lib

# 运行单个测试
cargo test test_name

# 显示测试输出
cargo test -- --nocapture

# 集成测试（需要连接真实 Android 设备）
cargo test --test '*'

# 运行示例
cargo run --example basic
RUST_LOG=debug cargo run --example logging_demo
```

### uiautomator-cli

```bash
cd uiautomator-cli && cargo build

# 单元测试
cargo test --lib

# 属性测试
cargo test property

# 集成测试（需要真实设备，标记为 ignored）
cargo test --test '*' -- --ignored
```

### test-app（Android）

```bash
cd test-app
# Windows
gradlew.bat assembleDebug
# Linux/macOS
./gradlew assembleDebug

# APK 输出: app/build/outputs/apk/debug/app-debug.apk
# 包名: com.uiautomator.testapp
```

## 架构

### 核心库数据流

```
用户代码
  → Device (设备入口，管理连接和模式选择)
    → JsonRpcClient (JSON-RPC，与设备端 UiAutomator 通信)
      → AdbClient (封装 adb_client，端口转发 + shell)
        → Android 设备 (端口 9008)
    → AtxAgentClient (REST API，端口 7912，生产级模式)
```

### 关键模块职责

- **`device.rs`** — `Device` 结构体，操作入口。支持 Direct/AtxAgent/Auto 三种连接模式
- **`jsonrpc.rs`** — JSON-RPC 客户端，通过 `include_bytes!` 嵌入 `assets/u2.jar`
- **`atx_agent.rs`** — ATX-Agent REST API，ADB 端口转发访问 7912 端口
- **`selector.rs`** — UI 元素选择器，mask 位必须与 Python uiautomator2 保持一致
- **`uiobject.rs`** — UI 元素操作，通过 `poll_until` 轮询实现等待逻辑
- **`settings.rs`** — 全局配置，通过 `Arc<RwLock<Settings>>` 共享
- **`error.rs`** — 统一错误类型，每个错误有唯一错误码和类别

### build.rs 机制

两个 crate 都有 `build.rs`，编译时计算 `assets/` 下资源文件的 MD5，通过 `cargo:rustc-env` 注入为编译期常量。资源文件不在 git 仓库中，需手动下载（见 `assets/download_atx_agent.sh`）。

### CLI 结构 (uiautomator-cli/)

- `cli.rs` — clap 命令行参数定义
- `commands.rs` — 命令实现（init/status/restart/uninstall）
- `installer.rs` — ATX-Agent 安装逻辑
- `resources.rs` — 嵌入式资源文件管理

## 编码规范

- 使用 `Result<T>` 和 `?` 操作符，避免 `unwrap()`
- 并发安全：`Arc` 共享所有权，`RwLock`/`Mutex` 保护可变状态
- 异步测试使用 `#[tokio::test]`
- 提交信息使用约定式格式：`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`, `perf:`, `ci:`
- 为所有公共类型和方法添加文档注释
