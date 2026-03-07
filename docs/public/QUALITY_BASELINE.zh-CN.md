[English](./QUALITY_BASELINE.md) | [简体中文](./QUALITY_BASELINE.zh-CN.md)

# 质量基线

## 1. 文档定位

本文件沉淀一次全项目验证式 code review 之后留下的稳定结论。

它只记录长期有效的质量边界、修复结果和后续维护约定，不记录批次进度、临时统计或本地扫描过程。

## 2. 当前基线结论

- 全项目分批审计已经完成。
- 审计中确认的 correctness 问题已经完成修复，或被明确重新归类为设计边界。
- 当前没有悬而未决的已确认阻塞性缺陷。
- review 过程中识别出的若干维护性问题也已经完成收口，不再保留为开放 action item。

## 3. 已验证的加固结果

本轮基线包含以下稳定改进：

- ADB timeout shell 路径增加并发边界，避免失控扩张后台 worker。
- `UiObject::click` / `long_click` 尊重等待超时参数，不再忽略调用方的超时语义。
- 指针类操作对 JSON-RPC 布尔结果做语义校验，避免“RPC 调用成功但动作失败”被静默吞掉。
- `Settings` 中的 `max_retry=0` 与 `polling_interval=0` 被防御性钳制，避免退化为无重试或热轮询。
- JSON-RPC 与 ATX-Agent 的临时文件处理改为唯一文件名，并去掉不安全的路径 `unwrap()`。
- Direct / ATX-Agent 两条端口转发路径都在 client 生命周期结束时执行清理。
- CLI 卸载流程不再对关键清理失败报成功，而是聚合错误返回。
- 与真实设备相关的安装/服务测试增加状态恢复与 readiness 等待，降低脆弱性。
- Android test app 中影响自动化稳定性的若干行为问题已修复，包括无界内存累积、回调重叠、重置文案被监听器覆盖，以及主菜单滚动区域布局不稳定。

## 4. 明确的设计边界

### 4.1 Selector 语义边界

- 设备侧 `objInfo(selector)` 仍然是 selector 完整语义的权威来源。
- 客户端只对 `ElementInfo` 可以直接表达的维度做防御性复核，例如精确匹配、contains、starts_with、布尔属性，以及若干 regex 字段。
- `index`、`instance`、`child/sibling` 层级语义不在客户端重放。

原因：

- `ElementInfo` 不包含足够的结构信息，无法无损重建层级选择器语义。
- 在客户端重放这些规则会形成第二套 selector 解释器，并引入与 Android / Python `uiautomator2` 漂移的风险。

### 4.2 非缺陷策略项

以下结论被视为当前策略选择，而不是隐藏的 correctness bug：

- `Rect::width()` / `height()` 的饱和减法属于防御性归一化策略。
- `build.rs` 对安装资源缺失的处理属于打包/发布策略问题，而不是运行时逻辑错误。
- 某些集成测试中的串行化/互斥约束属于测试设计取舍，用于兼容真实设备能力差异。

## 5. 维护性收口结果

在不改变外部行为的前提下，以下维护性问题已经一并收口：

- `device.rs` 的大块内联单元测试已拆到独立测试文件。
- `selector.rs` 的大块内联单元测试已拆到独立测试文件。
- `Key` 的 keycode / 名称映射已收敛到共享元数据源，避免多处手工维护。
- test app 主菜单的按钮跳转逻辑已通过 helper 收敛，避免线性复制粘贴扩张。
- CLI 资源 helper 已接入 `version` 命令输出，不再只是测试或文档里的孤立能力。

## 6. 验证基线

本轮稳定基线至少要求以下验证能够通过：

- `uiautomator/`：`cargo test --lib`
- `uiautomator-cli/`：`cargo test`
- `test-app/`：`gradlew.bat assembleDebug` 或 `./gradlew assembleDebug`

说明：

- 真机相关 `ignored` 测试仍然依赖外部设备环境，不能用本地无设备结果替代。
- 发布前的完整门槛仍以 [`TESTING_RELEASE.zh-CN.md`](./TESTING_RELEASE.zh-CN.md) 为准。

## 7. 使用方式

建议在以下场景优先查阅本文件：

- 评估某个问题是否已经在审计中确认并处理过
- 判断某个行为差异是 bug 还是既定设计边界
- 准备继续做质量治理，但不想重新阅读本地 review 过程文件

若要查看更详细的长期设计解释，继续阅读 [`DESIGN.zh-CN.md`](./DESIGN.zh-CN.md)。
