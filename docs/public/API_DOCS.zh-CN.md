[English](./API_DOCS.md) | [简体中文](./API_DOCS.zh-CN.md)

# API 文档指南

## 1. 目的

本文件是 `uiautomator-rs` 的公开 API 指南。
它补充 `docs.rs` 的逐项文档，用于稳定说明 crate 关系、API 约定和示例入口。

## 2. 生成式 API 文档入口

- 核心库：[docs.rs/uiautomator](https://docs.rs/uiautomator)
- CLI 库：[docs.rs/uiautomator-cli](https://docs.rs/uiautomator-cli)

逐项签名、示例、类型定义以 `docs.rs` 为准。
本文件负责给出更高层的使用约定。

## 3. 两个 crate 的关系

- `uiautomator`：异步 Android UI 自动化库，负责 ADB、JSON-RPC 和可选 ATX-Agent 传输
- `uiautomator-cli`：配套环境工具，负责设备侧 ATX-Agent 的初始化与生命周期管理

推荐使用方式：

1. 先用 `uiautomator-cli init` 完成设备环境准备。
2. 再在业务代码中使用 `uiautomator`。
3. 默认优先使用 `Auto` 模式，只有明确需要时再强制 `Direct` 或 `AtxAgent`。

## 4. 核心库 API 结构

### `Device`

统一入口，负责：

- 设备连接与模式选择
- 通过 `find` 查找元素
- 手势与按键输入
- 截图和应用管理

常用入口方法：

- `Device::connect`
- `Device::connect_with_mode`
- `Device::info`
- `Device::find`
- `Device::click`
- `Device::click_coord`
- `Device::long_click_coord`
- `Device::double_click_coord`
- `Device::swipe`
- `Device::swipe_coord`
- `Device::drag_coord`
- `Device::set_cache_ttl`
- `Device::clear_cache`
- `Device::disable_cache`
- `Device::app_start`
- `Device::app_stop`
- `Device::app_wait`

### `Selector`

用于构建 UI 查询条件的 builder。

支持的选择器族包括：

- 文本与正则文本
- 资源 ID、类名、描述
- 布尔状态字段
- `index` / `instance`
- `child` / `sibling` 等层级选择器

### `Coord`

用于统一公开 API 中的坐标表达，既支持：

- 绝对像素：`Coord::pixel(200)`
- 相对百分比：`Coord::percent(0.5)`

现有像素 API 保持不变，同时新增显式坐标辅助方法，例如：

- `Device::click_coord`
- `Device::long_click_coord`
- `Device::double_click_coord`
- `Device::swipe_coord`
- `Device::drag_coord`

### `UiObject`

由 `Selector` 派生出的惰性元素句柄。

常用操作：

- `exists`
- `wait`
- `wait_gone`
- `click`
- `long_click`
- `set_text`
- `clear_text`
- `get_text`
- `info`
- `bounds`
- `center`

## 5. CLI API 结构

`uiautomator-cli` 同时暴露命令行接口和可复用安装器能力。

命令级入口：

- `init`
- `status`
- `restart`
- `uninstall`
- `version`

库级入口：

- `Installer::new`
- `Installer::install`
- `Installer::status`
- `Installer::restart`
- `Installer::uninstall`

## 6. API 约定

### 异步

- 设备和元素操作 API 以异步为主。
- 推荐使用 `tokio` 或兼容 runtime。

### 返回值

- 可失败操作返回 `uiautomator::Result<T>` 或 CLI 的 `Result<T>`。
- `exists` 这类布尔结果方法，对“正常未命中”返回 `Ok(false)`，对传输、协议或语义错误才返回 `Err(...)`。

### 超时

- 带超时参数的库 API 统一使用 `Option<Duration>`。
- `None` 表示使用配置值或默认值。

### 模式

- `Auto`：先尝试 ATX-Agent，再回退 Direct
- `AtxAgent`：强制使用 ATX-Agent 通道
- `Direct`：绕过 ATX-Agent，直接连接 JSON-RPC 运行时

### 缓存

- `Device::info()` 默认不使用缓存。
- `Device::set_cache_ttl(...)` 用于显式开启设备信息缓存。
- `Device::clear_cache()` 会强制下一次 `info()` 重新拉取。
- `Device::disable_cache()` 会恢复为每次实时读取。

### 错误

公开 API 使用结构化错误，而不是宽泛字符串失败。
重点错误类型包括：

- 设备选择与离线错误
- RPC/传输错误
- 元素未找到与等待超时错误
- 应用生命周期错误

## 7. 示例与测试入口

发布到 crates.io 的包默认不包含仓库内完整的 `tests/` 和 `examples/` 目录。
完整参考请查看 GitHub 仓库中的：

- `uiautomator/examples/`
- `uiautomator/tests/`
- `uiautomator-cli/tests/`
- `test-app/`

同时，`docs.rs` 里每个公开项的示例也保持为完整覆盖，可作为最快的单项参考入口。

## 8. 文档质量状态

当前基线：

- 公开 API 文档覆盖率：`100%`
- 公开 API 示例覆盖率：`100%`

验证路径：

- 脚本：`scripts/docs-coverage-report.ps1`
- 最新摘要：
  - `internal/testlogs/docs/latest-summary.json`
  - `internal/testlogs/docs/latest-summary.md`
- 最新验证汇总：
  - 文档 `331/331`
  - 示例 `187/187`
- CI 守门：
  - `.github/workflows/docs-coverage.yml`

## 9. 相关文档

- `REQUIREMENTS.md`
- `DESIGN.md`
- `MIGRATION.md`
- `TASKS.md`
- `TESTING_RELEASE.md`
- `RELEASE_NOTES.md`
