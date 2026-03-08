[English](./README.md) | [简体中文](./README.zh-CN.md)

# uiautomator

使用 Rust 实现的异步 Android UI 自动化库，提供接近 Python `uiautomator2` 的 API 体验。

## 本 crate 提供什么

- 设备连接与模式路由（`Auto`、`AtxAgent`、`Direct`）
- 基于 selector 的元素定位
- `UiObject` 操作（点击、输入、清空、等待、信息获取、文本和边界获取）
- 手势、按键事件、截图、应用生命周期控制
- 面向不稳定设备 / RPC 场景的结构化错误和重试支持

## 安装

推荐的首次使用路径：

1. 先对目标设备执行一次 `uiautomator-cli init`。
2. 再在业务代码中使用本 crate。

```toml
[dependencies]
uiautomator = "1.0.1"
tokio = { version = "1", features = ["full"] }
```

可选的 ATX-Agent 安装能力：

```toml
[dependencies]
uiautomator = { version = "1.0.1", features = ["atx-agent-install"] }
```

说明：

- `uiautomator` 发布包为控制 crates.io 体积，不内置多架构 `atx-agent` 二进制。
- 推荐在使用 `AtxAgent` 模式前，先通过 `uiautomator-cli init`（或其他外部方式）完成设备环境初始化。
- `Device::connect(None)` 要求当前只有一台在线 ADB 设备；多设备场景请传 `Some("<serial>")`。

## 最小示例

```rust
use uiautomator::{Device, Selector};

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    let d = Device::connect(None).await?;

    let settings = d.find(Selector::new().text("Settings"));
    if settings.exists(None).await? {
        settings.click(None, None).await?;
    }

    Ok(())
}
```

## 错误处理示例

```rust
use std::time::Duration;
use uiautomator::{Device, Error};

async fn run(device: &Device) -> Result<(), Error> {
    match device.app_wait("com.example.app", Some(Duration::from_secs(5))).await {
        Ok(pid) => {
            println!("app ready: {pid}");
            Ok(())
        }
        Err(Error::AppNotInstalled(pkg)) => Err(Error::AppNotInstalled(pkg)),
        Err(Error::AppCrashed(pkg)) => Err(Error::AppCrashed(pkg)),
        Err(Error::Timeout) => Err(Error::Timeout),
        Err(other) => Err(other),
    }
}
```

## 模式说明

- `Auto`（默认）：优先尝试 ATX-Agent，失败时回退到 Direct。
- `AtxAgent`：使用 ATX-Agent 传输，适合长时间、稳定性优先的自动化。
- `Direct`：直接使用 JSON-RPC，初始化更快，但鲁棒性较低。

## 常用 API

- `Device`：`connect`、`info`、`find`、`click`、`swipe`、`press`、`screenshot`、`app_start`、`app_stop`
- `UiObject`：`exists`、`wait`、`wait_gone`、`click`、`long_click`、`set_text`、`clear_text`、`get_text`、`info`
- `Selector`：文本、资源 ID、类名、描述，以及正则与层级关系（`child` / `sibling`）

## 测试

```bash
cargo test
cargo test -- --ignored --nocapture --test-threads=1
```

如果要执行完整设备回归，请在仓库根目录运行：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/device-full-test.ps1 -Serial <serial>
```

## 文档

- 公开需求 / 设计 / 任务 / 发布文档：
  - `../docs/public/REQUIREMENTS.md`
  - `../docs/public/DESIGN.md`
  - `../docs/public/TASKS.md`
  - `../docs/public/TESTING_RELEASE.md`
- 错误处理指南：
  - `ERROR_HANDLING.md`

说明：`tests/**` 和 `examples/**` 保留在 GitHub 仓库中，默认不会包含在 crates 发布包内。

## 许可证

MIT。
