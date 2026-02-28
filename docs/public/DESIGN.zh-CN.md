[English](./DESIGN.md) | [简体中文](./DESIGN.zh-CN.md)

# Design Baseline

## 1. 文档定位

本文件是公开设计基线，来源于内部 `.kiro/specs` 的四套设计文档整编：

- `uiautomator/design.md`
- `uiautomator-cli/design.md`
- `uiautomator-phase2/design.md`
- `bugfix/design.md`

目标是把“需求、实现、测试、发布”打通为可审计的设计说明，而不是高层摘要。

## 2. 设计范围与边界

### 2.1 In Scope

- 核心库 `uiautomator` 的连接、定位、操作、通信、错误处理
- CLI `uiautomator-cli` 的设备环境搭建与服务管理
- Selector mask 与扩展字段兼容性修复
- 测试与可观测性工程（结构化日志、覆盖对账）

### 2.2 Out of Scope

- 业务流程自动化编排（翻页、业务兜底策略）
- 设备 Farm 调度平台
- 云端执行系统

## 3. 需求追踪矩阵（Design Traceability)

| 需求ID | 设计落点 | 核心模块 | 验证路径 |
|---|---|---|---|
| FR-UA-01 设备连接 | 连接模式/设备枚举/幂等连接 | `device.rs`, `adb.rs` | 集成测试 + 全量脚本 |
| FR-UA-03 元素定位 | Selector DSL + mask 序列化 | `selector.rs`, `uiobject.rs` | 单测 + 真机/模拟器 |
| FR-UA-04 元素操作 | UiObject 操作与等待轮询 | `uiobject.rs` | mock + 集成 |
| FR-UA-08 RPC恢复 | 重试/回退/错误映射 | `jsonrpc.rs`, `error.rs` | 失败场景测试 |
| FR-CLI-01 init | 安装器流程机 | `installer.rs`, `commands.rs` | ignored 集成测试 |
| FR-CLI-03 离线资源 | 资源内嵌与 hash 校验 | `resources.rs`, `build.rs` | 构建检查 |
| FR-BUG-01 Selector兼容 | mask/字段/层级选择器 | `selector.rs` | bugfix 测试集 |
| FR-BUG-02 ATX兼容 | shell_v2 + /version文本兼容 + 架构回退 | `atx_agent.rs`, `device.rs` | 多设备回归 |

## 4. 总体架构

### 4.1 逻辑视图

`User Code / CLI -> Device -> (JsonRpcClient | AtxAgentClient) -> AdbClient -> Android Device`

### 4.2 核心设计原则

- 与 Python uiautomator2 保持行为心智一致
- 默认异步 API，面向多设备并发
- 把设备不稳定性作为常态处理（重试、回退、恢复）
- 错误必须可定位，禁止模糊吞错

### 4.3 连接模式策略

1. `ATX-Agent`：生产稳定优先，走 7912 REST/JSON-RPC。
2. `Direct`：直接 JSON-RPC（9008），作为快速/回退路径。
3. `Auto`：先 ATX，再 Direct。

## 5. 关键模块设计

### 5.1 `device.rs`（统一入口）

职责：

- 管理设备序列号与连接模式
- 聚合元素、手势、按键、截图、应用管理 API
- 承载模式切换和上层错误语义

关键决策：

- API 入口尽量保持统一签名和一致超时语义
- 不在库内引入业务层“自动兜底导航”逻辑

### 5.2 `jsonrpc.rs`（协议与恢复）

职责：

- 构建请求、解析 `result/error`
- 处理超时和网络错误
- 实现可配置重试（最大次数 + 退避）

关键决策：

- 成功即停止重试，失败返回最后一次可诊断错误
- 错误映射优先基于结构化上下文，避免宽泛字符串误判

### 5.3 `atx_agent.rs`（设备侧守护接口）

职责：

- `/version`, `/info`, `/jsonrpc/0` 等接口封装
- 服务状态检查、安装启动、重启、卸载协作

关键决策：

- 兼容 `/version` 纯文本响应
- 兼容 ADB shell_v2 输出格式
- 架构候选策略支持回退（例如 `amd64 -> 386`）

### 5.4 `adb.rs`（执行层）

职责：

- 设备枚举、shell、push/pull、forward
- 命令执行与退出状态处理

关键决策：

- 把“退出码 + stderr + 关键上下文”一并上抛
- 统一命令超时，防止无界等待

### 5.5 `selector.rs`（定位 DSL）

职责：

- 构建选择器条件
- 输出 JSON-RPC 参数
- 维护 mask 位语义一致性

关键决策：

- `to_params()` 必须始终生成正确 mask
- 支持布尔字段、正则字段、`index`、`child/sibling`
- 序列化语义对齐 Python 实现

### 5.6 `uiobject.rs`（元素行为）

职责：

- exists/wait/wait_gone 轮询
- info/get_text/get_bounds/center
- click/input/clear_text/set_text

关键决策：

- 等待逻辑统一由轮询机制驱动（超时可控）
- 元素不存在与协议异常区分错误类型
- 关键操作返回值做语义校验，避免“调用成功但无效”

### 5.7 `error.rs`（统一错误域）

职责：

- 定义错误类别、错误码、上下文
- 承担跨模块错误归一和用户可读信息

关键决策：

- 面向“可定位”而非“可打印”设计
- 保留必要上下文（serial、selector、package、timeout）

## 6. 关键流程设计

### 6.1 设备初始化流程（CLI init）

1. 检测设备可用性
2. 检查 ATX 现状
3. 根据 `--force` 决定是否重装
4. 推送二进制/APK/JAR 资源
5. 启动服务并探活验证
6. 输出结构化状态

### 6.2 元素等待流程（UiObject wait）

1. 计算最终超时（调用参数优先，其次设置默认值）
2. 周期轮询 `objInfo/exists`
3. 命中条件立即返回
4. 到达超时边界返回 `ElementTimeout`

### 6.3 Auto 模式回退流程

1. 尝试 ATX-Agent 健康检查
2. 成功则使用 ATX 路径
3. 失败时记录原因并回退 Direct
4. Direct 失败则返回聚合错误

## 7. 数据模型设计

核心模型：

- `DeviceInfo`: 屏幕、旋转、SDK、点亮状态等
- `ElementInfo`: 文本、类名、bounds、可点击状态等
- `Rect`: 区域与中心点计算
- `AppInfo`: 前台应用包名/Activity
- `Settings`: 超时、重试、轮询相关配置

约束：

- 使用 `serde` 保证协议映射
- 字段语义与设备返回保持一致
- 便捷方法不改变源数据语义

## 8. 正确性属性

关键属性：

1. 连接幂等：重复连接不破坏状态
2. 定位一致：同 selector 同环境结果一致
3. 坐标正确：百分比换算可预测
4. 超时边界：不会无限等待
5. 请求对应：response 与 request 匹配
6. 错误幂等：失败恢复操作可重复
7. 并发安全：共享状态不会竞态破坏
8. 资源清理：端口转发/临时文件可回收

## 9. 错误处理与可观测性

### 9.1 错误分层

- 设备层：未连接、离线、权限、超时
- 协议层：RPC 解析失败、服务异常
- 语义层：元素未找到、应用状态不符合

### 9.2 日志策略

- 用户可读日志：步骤与结果
- 机器可读摘要：`summary.json`, `summary.junit.xml`
- 失败日志需包含命令、退出码、关键输出路径

### 9.3 反误读设计

- 避免 `contains("error")` 这类宽匹配直接判失败
- 统一输出编码 UTF-8
- 用明确状态字段表示 pass/fail

## 10. 测试架构设计

### 10.1 分层

- 单元测试：算法、序列化、错误映射
- 集成测试：真实设备端到端
- ignored：重依赖环境和慢测

### 10.2 稳定性实践

- 设备相关流程强制串号 pin
- 长任务加硬超时
- 失败后产出可重放日志

### 10.3 API 覆盖对账

- 使用 `scripts/api-coverage-report.ps1`
- 输出公开 API 与测试映射清单
- 标记未覆盖项，作为发布前证据

## 11. 构建与发布设计

### 11.1 资源与构建

- `build.rs` 计算资源校验信息并注入构建时常量
- 缺失资源在构建阶段即失败

### 11.2 包发布关系

- 先发布库 `uiautomator`
- 再发布 `uiautomator-cli`

原因：CLI 在 crates 关系上依赖库版本。

## 12. Phase2 设计计划（不阻塞当前发布）

- 错误类型全链路收敛
- API 参数一致性收敛
- Mock 测试体系化
- 缓存与性能优化（可选）

对应任务详见 `docs/public/TASKS.md`。

## 13. 风险与缓解

1. 设备/模拟器差异导致行为波动
缓解：最小发布矩阵覆盖 arm64 真机 + x86_64 模拟器。

2. 环境依赖导致测试“看似失败”
缓解：结构化摘要 + 编码统一 + 失败分类。

3. 架构识别与二进制兼容问题
缓解：候选链路回退与探活验证。

4. 协议返回变化导致兼容回归
缓解：关键接口做宽容解析并加回归样例。

