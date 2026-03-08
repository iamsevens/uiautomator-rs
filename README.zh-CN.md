[English](./README.md) | [简体中文](./README.zh-CN.md)

# uiautomator-rs

[![crates.io - uiautomator](https://img.shields.io/crates/v/uiautomator.svg)](https://crates.io/crates/uiautomator)
[![docs.rs - uiautomator](https://docs.rs/uiautomator/badge.svg)](https://docs.rs/uiautomator)
[![crates.io - uiautomator-cli](https://img.shields.io/crates/v/uiautomator-cli.svg)](https://crates.io/crates/uiautomator-cli)
[![docs.rs - uiautomator-cli](https://docs.rs/uiautomator-cli/badge.svg)](https://docs.rs/uiautomator-cli)

使用 Rust 实现的 Android UI 自动化库，提供接近 Python `uiautomator2` 的 API 体验。

## 概览

`uiautomator-rs` 包含两个可发布 crate 和一个仓库内测试 APK：

- `uiautomator/`：核心异步库（`Device`、`Selector`、`UiObject`、JSON-RPC/ATX 传输）
- `uiautomator-cli/`：ATX-Agent 初始化与生命周期管理 CLI
- `test-app/`：集成回归测试用 Android APK（仅仓库内使用，不发布）

## 当前状态

- 核心 API 已完成，并在模拟器和真机上验证。
- ATX-Agent 兼容性修复已落地（`shell_v2` 解码、`/version` 纯文本兼容）。
- Selector 兼容性修复已落地（mask 位、扩展字段、`child`/`sibling` 序列化）。
- 全量设备回归已脚本化，并输出机器可读结果。

## 关键能力

- 设备连接与模式路由（`Auto`、`AtxAgent`、`Direct`）
- 丰富 selector 的元素定位
- `UiObject` 操作（`click`、`long_click`、`set_text`、`wait`、`wait_gone` 等）
- 手势、按键、截图、应用生命周期控制
- 内置重试和结构化错误映射
- 多设备异步并发使用

## 仓库结构

```text
.
├── uiautomator/           # 核心 crate
├── uiautomator-cli/       # CLI crate
├── test-app/              # Android 测试 APK 项目
├── scripts/               # 回归/发布脚本
├── docs/public/           # 公开需求/设计/任务/发布文档
└── internal/              # 本地产物与内部记录
```

注意：本仓库不是 Cargo workspace，执行 Cargo 命令时请进入对应 crate 目录。

## 快速开始

多数使用者只需要两步：

1. 在宿主机安装一次 `uiautomator-cli`。
2. 在业务代码里使用 `uiautomator`。

### 1）前置条件

- Android 设备或模拟器，且 ADB 可访问
- Rust 工具链
- `adb` 已加入 `PATH`

### 2）安装已发布 crate

```bash
cargo install uiautomator-cli
```

在你的项目里添加库依赖：

```toml
[dependencies]
uiautomator = "1.0.2"
tokio = { version = "1", features = ["full"] }
```

### 3）初始化设备端环境（CLI）

```bash
uiautomator init --serial <serial> --force
uiautomator status --serial <serial>
```

如果当前只有一台在线设备，可以省略 `--serial`。

### 4）基础库调用

```rust
use uiautomator::{Device, Selector};

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    let d = Device::connect(None).await?;
    d.find(Selector::new().text("Settings")).click(None, None).await?;
    Ok(())
}
```

`Device::connect(None)` 要求当前只有一台 ADB 在线设备；多设备场景请传 `Some("<serial>")`。

### 5）从源码构建（仓库开发者）

```bash
cd uiautomator
cargo build

cd ../uiautomator-cli
cargo build
```

## 测试

### 分 crate 测试

```bash
cd uiautomator-cli
cargo test
cargo test -- --ignored --nocapture --test-threads=1

cd ../uiautomator
cargo test
cargo test -- --ignored --nocapture --test-threads=1
```

### 全量设备回归（推荐）

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/device-full-test.ps1 -Serial <serial>
```

输出包含日志以及机器可读文件：`summary.json`、`summary.junit.xml`。

### GitHub Actions 串行回归

```cmd
scripts\run-gh-device-regression.cmd -Serial <serial> -TargetName <name> -ExpectedAbi <abi> -ExpectedAndroidMajor <major>
```

如果只有一台 ADB 在线设备，可省略 `-Serial`。

## 公开文档

- [公开文档索引](docs/public/README.md)
- [需求基线](docs/public/REQUIREMENTS.md)
- [设计基线](docs/public/DESIGN.md)
- [任务台账](docs/public/TASKS.md)
- [测试与发布基线](docs/public/TESTING_RELEASE.md)

对应中文文档位于同目录下的 `*.zh-CN.md` 文件。

## Crate 关系

- `uiautomator` 是基础库。
- `uiautomator-cli` 依赖 `uiautomator`。
- crates.io 发布顺序为：先 `uiautomator`，后 `uiautomator-cli`。

## 发布

发布前检查与发布步骤只以 `PUBLISHING.zh-CN.md` 为唯一入口，其他文档不再重复流程。

## 支持与规范

- [贡献指南](./CONTRIBUTING.md)
- [安全策略](./SECURITY.md)
- [支持说明](./SUPPORT.zh-CN.md)
- [行为准则](./CODE_OF_CONDUCT.md)

## 许可证

MIT，详见 [LICENSE](./LICENSE)。
