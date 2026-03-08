[English](./REQUIREMENTS.md) | [简体中文](./REQUIREMENTS.zh-CN.md)

# Requirements Baseline

## 1. 文档目的

本文件不是摘要提纲，而是对内部 `.kiro/specs` 的公开化需求基线，面向：

- 发布前功能对账
- 设计与实现追踪
- 测试与验收判定

## 2. 来源与追踪矩阵

| 公开需求域 | 内部来源 | 当前状态 |
|---|---|---|
| 公开文档与示例覆盖 | `docs-examples-quality/tasks.md` | 已完成 |
| 核心库基础能力 | `uiautomator/requirements.md` 需求 1~12 | 已完成 |
| CLI 设备环境管理 | `uiautomator-cli/requirements.md` 需求 1~7 | 已完成 |
| Selector 关键修复 | `bugfix/requirements.md` 需求 1~5 | 已完成 |
| Phase2 能力增强 | `uiautomator-phase2/requirements.md` 需求 1~7 | 已完成 |

## 3. 术语

- `Device`: 通过 ADB 连接的 Android 设备（真机/模拟器）
- `Selector`: 元素匹配条件组合
- `UiObject`: 基于 Selector 绑定的元素操作对象
- `Direct`: 直接访问设备端 JSON-RPC
- `ATX-Agent`: 设备侧守护服务（REST + JSON-RPC）
- `Auto`: 优先 ATX-Agent，失败回退 Direct

## 4. 功能需求（发布阻塞）

### FR-UA-01 设备连接与初始化

用户故事：作为自动化测试工程师，我希望以最少参数连接目标设备并自动拉起依赖服务。

验收标准：

1. 指定序列号时，必须连接该设备，否则返回明确错误。
2. 未指定序列号且仅一台设备在线时，必须自动连接。
3. 未指定序列号且多台设备在线时，必须返回“需指定设备”错误。
4. 连接后必须验证可通信状态，并在必要时拉起服务。

### FR-UA-02 设备信息获取

用户故事：作为脚本开发者，我需要稳定读取设备状态用于坐标换算和断言。

验收标准：

1. 必须可获取分辨率、旋转角、SDK 版本、前台包名、点亮状态。
2. 信息读取失败时必须返回可诊断错误，不可静默吞掉。

### FR-UA-03 元素定位

用户故事：作为测试开发者，我需要通过多种条件可靠定位 UI 元素。

验收标准：

1. 支持 `text/resourceId/className/description` 等基础条件。
2. 支持多条件组合，组合逻辑与 Python uiautomator2 语义一致。
3. 支持 `instance/index` 区分多实例元素。
4. 支持超时等待与元素不存在返回。

### FR-UA-04 元素操作

用户故事：作为测试开发者，我需要对定位到的元素进行一致、可预期的操作。

验收标准：

1. 支持点击、长按、输入、清空、读取文本。
2. 支持 `exists/wait/wait_gone`。
3. 支持获取 `info/bounds/center`。
4. 元素不存在时必须返回明确的 `ElementNotFound/ElementTimeout` 类错误。

### FR-UA-05 手势与按键

验收标准：

1. 支持坐标点击、长按、滑动、拖拽、双击。
2. 支持百分比坐标换算为像素坐标。
3. 支持 Home/Back/Power/方向键/音量键等按键能力。

### FR-UA-06 截图与文件输出

验收标准：

1. 支持返回图像字节数据与保存到文件两种模式。
2. 保存失败或截图失败必须返回明确错误。

### FR-UA-07 应用管理

验收标准：

1. 支持 `app_start/app_stop/app_wait/app_current/app_clear`。
2. 应用启动等待超时时必须返回可定位错误。

### FR-UA-08 JSON-RPC 通信与恢复

验收标准：

1. 请求/响应必须正确解析 `result` 与 `error`。
2. 超时、网络抖动、服务异常场景必须有可预期错误。
3. 必须具备重试与恢复策略，成功后立即停止重试。

### FR-UA-09 异步与并发

验收标准：

1. 公开 I/O API 支持异步使用。
2. 多设备并发时不应出现共享状态破坏。
3. 取消/超时后资源应正确释放。

### FR-CLI-01 init 命令

用户故事：作为使用者，我希望一条命令完成设备环境初始化。

验收标准：

1. `init` 必须检查设备连接、安装资源、启动并验证 ATX-Agent。
2. 支持 `--serial` 指定设备。
3. 支持 `--force` 强制重装流程。

### FR-CLI-02 status/restart/uninstall 命令

验收标准：

1. `status` 返回运行状态、版本、端口等关键信息。
2. `restart` 执行“先停后起”并返回最终状态。
3. `uninstall` 执行停止、卸载、清理流程并给出结果。

### FR-CLI-03 资源内嵌与离线能力

验收标准：

1. 构建产物内嵌 atx-agent/APK/JAR 资源。
2. 初始化流程不依赖在线下载。
3. 缺失资源时构建阶段应失败并提示补齐方式。

### FR-CLI-04 跨平台一致性与可读错误

验收标准：

1. Linux/macOS/Windows 行为一致。
2. `--help` 完整，错误消息可操作。
3. 输出应避免“看似失败但实际成功”的误导表达。

### FR-BUG-01 Selector mask 与字段完整性

验收标准：

1. `to_params()` 输出必须包含正确 `mask`。
2. 支持补齐布尔字段、正则字段、`index`。
3. 支持 `child/sibling` 层级选择器。
4. 序列化语义与 Python uiautomator2 对齐。

### FR-BUG-02 ATX 环境兼容性修复

验收标准：

1. 正确处理 ADB `shell_v2` 输出解码。
2. 兼容 `/version` 纯文本响应。
3. 多架构安装遵循候选回退链（如 `amd64 -> 386`）。

## 5. 增强需求（Phase2，默认不阻塞当前发布）

### PH2-01 错误类型全链路落地

1. 统一错误类型和上下文信息。
2. API 文档与示例反映新错误语义。

### PH2-02 API 一致性

1. 超时参数统一为 `Option<Duration>`。
2. 坐标参数统一模型与行为。

### PH2-03 Mock 覆盖体系化

1. 建立稳定的 mock 测试分层。
2. 对关键错误与重试路径提供可重复验证。

### PH2-04 缓存与性能优化（可选）

1. 设备信息缓存可配置 TTL。
2. 默认行为不破坏现有 API 语义。

## 6. 非功能性需求

### NFR-01 兼容性

- OS：Windows/Linux/macOS
- Android：API 21+
- 架构：arm64、x86_64（含回退链路）

### NFR-02 稳定性

- 网络抖动可重试，服务异常可恢复。
- 长流程场景优先保障 ATX-Agent 模式稳定性。

### NFR-03 可观测性

- 全量回归输出 `summary.json` 与 `summary.junit.xml`。
- API 对账输出 `api-coverage.json` 与 `api-coverage.md`。
- 文档/示例覆盖输出 `docs-coverage-summary.json` 与 `docs-coverage-summary.md`。

### NFR-04 可维护性

- 公共 API 需文档化。
- 公共 API 需保持发布级示例覆盖（可运行或 `no_run`）。
- 核心路径需具备测试覆盖与发布前脚本化检查。

## 7. 发布验收门槛

当前版本可发布需同时满足：

1. 核心库与 CLI 的发布阻塞需求全部达成。
2. “清空环境 -> 重建 -> 全量”脚本化回归通过。
3. 至少覆盖真机 + 模拟器组合验证。
4. docs/examples 覆盖证据已生成并审阅。
5. 发布包检查通过，发布顺序以 `PUBLISHING.zh-CN.md` 为准（先 `uiautomator` 后 `uiautomator-cli`）。

当前已纳管的 Phase2 增强项已完成；后续若新增增强需求，应作为新的任务流继续进入 `TASKS.md`。

发布流程细节统一维护在仓库根目录 `PUBLISHING.zh-CN.md`。

