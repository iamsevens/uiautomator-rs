[English](./TESTING_RELEASE.md) | [简体中文](./TESTING_RELEASE.zh-CN.md)

# Testing And Release Baseline

## 1. 文档目的

本文件定义对外发布前的测试与发布基线，解决三个问题：

1. 如何稳定复现“从空环境到全量通过”的验证流程。
2. 如何区分真实失败和环境噪音，避免误读日志。
3. 如何形成可审计的发布证据。

## 2. 来源与适用范围

来源：

- `.kiro/specs/uiautomator*/tasks.md`
- `.kiro/specs/bugfix/tasks.md`
- 仓库脚本：`scripts/run-validation-gate.ps1`、`scripts/device-full-test.ps1`、`scripts/api-coverage-report.ps1`
- 发布检查脚本：`scripts/release-check.ps1`、`scripts/release-check.sh`

适用对象：

- `uiautomator` 核心库发布验证
- `uiautomator-cli` 工具链发布验证

## 3. 测试分层与术语

### 3.1 测试分层

- `unit`：纯逻辑与序列化/错误映射测试。
- `integration`：真实设备端到端测试。
- `ignored`：环境依赖或慢测试，默认不自动执行。

### 3.2 `non-ignored` 与 `ignored` 区别

- `non-ignored`：日常回归主通道，默认执行。
- `ignored`：额外验证通道，通常需要设备状态更稳定、耗时更长。
- 发布前建议两者都执行，避免“主通道通过但慢场景回归”漏检。

## 4. 固化全量流程（必须脚本化）

### 4.1 标准命令

```powershell
C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/run-validation-gate.ps1 -Mode full -Serial <serial>
```

### 4.2 固定步骤

1. 执行 gate 预检（`PowerShell`、`adb`、Rust 工具链、设备就绪）。
2. 清理 ATX/uiautomator 运行状态与残留进程。
3. `init -f` 从空环境重建依赖。
4. 安装并校验 `test-app`（包名 `com.uiautomator.testapp`）。
5. 执行四组矩阵：`uiautomator-cli non-ignored`、`uiautomator-cli ignored`、`uiautomator non-ignored`、`uiautomator ignored`。

### 4.3 硬约束

- 必须显式指定 `-Serial`。
- 必须设置硬超时，禁止无界等待。
- 必须输出结构化摘要，不接受只有控制台日志的结果。

## 5. 回归矩阵模板

### 5.1 最小发布矩阵（推荐）

- `arm64` 真机 1 台（Android 主流版本）
- `x86_64` 模拟器 1 台（ATX 重建验证）

### 5.2 增强矩阵（建议）

- 多 Android 主版本（至少 2 个）
- 不同设备类型（真机 + 至少两类模拟器）
- 覆盖“已安装 ATX”与“空环境重建”两种初始状态

### 5.3 记录模板

| RunID | Device Serial | Device Type | Arch | Android | Initial State | CLI non-ignored | CLI ignored | Lib non-ignored | Lib ignored | Result |
|---|---|---|---|---|---|---|---|---|---|---|
| `<run-id>` | `<serial>` | `real/emulator` | `arm64/x86_64` | `<ver>` | `clean/preinstalled` | pass/fail | pass/fail | pass/fail | pass/fail | pass/fail |

## 6. 日志标准化与机器可读结果

### 6.1 输出目录

- Gate 编排层：`internal/testlogs/validation-gate/<run-id>/`
- 全量回归：`internal/testlogs/full-device/<run-id>/`
- API 对账：`internal/testlogs/api-coverage/<run-id>/`

### 6.2 必须产物

- 人类可读：`*.log`、`*.err.log`
- 机器可读：`summary.json`、`summary.junit.xml`、`api-coverage.json`、`api-coverage.md`

### 6.3 统一编码

- 控制台与子进程统一 UTF-8。
- 文件读取统一 UTF-8 解码。
- 任何编码异常必须记为环境问题并重跑。

## 7. 失败分类规范（避免误读）

### 7.1 分类编码

- `E-ENV`：环境问题（设备离线、ADB 异常、权限弹窗、端口冲突）。
- `E-ATX`：ATX 环境问题（安装失败、探活失败、版本不兼容）。
- `E-APP`：测试应用问题（APK 版本不匹配、页面结构变化）。
- `E-TEST`：测试用例本身问题（断言错误、超时设置不合理）。
- `E-LIB`：库实现回归（API 行为错误、协议处理错误）。
- `E-CLI`：CLI 行为回归（命令流程、输出语义错误）。

### 7.1.1 Gate 机器可读失败码

- `env_missing_powershell`
- `env_missing_adb`
- `env_missing_rust_toolchain`
- `adb_device_unavailable`
- `test_app_build_failed`
- `step_timeout`
- `manifest_or_summary_invalid`
- `test_or_runtime_failure`

### 7.2 处理规则

1. `E-ENV/E-ATX`：先修环境再重跑，不直接判产品失败。
2. `E-APP/E-TEST`：先修测试资产，再对比修复前后结果。
3. `E-LIB/E-CLI`：直接进入缺陷修复流程，修复后必须全量回归。

### 7.3 失败记录模板

| Timestamp | RunID | Device | Step | Code | Symptom | Root Cause | Action | Re-run Result |
|---|---|---|---|---|---|---|---|---|
| `<time>` | `<run-id>` | `<serial>` | `<step>` | `E-*` | `<symptom>` | `<root>` | `<fix>` | pass/fail |

## 8. API 覆盖对账

### 8.1 执行命令

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/api-coverage-report.ps1
```

### 8.2 目标

- 建立“公开 API -> 测试用例”映射。
- 自动标记未覆盖 API。
- 输出发布证据，避免“全量通过但覆盖不足”。

### 8.3 发布门槛建议

- 不允许核心公开 API 出现“未覆盖且无豁免说明”。
- 对暂缓覆盖项必须在发布说明写明原因与计划版本。

## 9. 发布前硬门槛（Release Gates）

### 9.1 Hard Gates（必须满足）

1. `release-check` 通过（包内容、元数据、资源完整性）。
2. 至少一台真机 + 一台模拟器完成“清空环境 -> 重建 -> 全量”通过。
3. `summary.json` 与 `summary.junit.xml` 完整可追溯。
4. API 覆盖对账产物已生成并审阅。
5. `cargo publish --dry-run` 在两个 crate 内均通过。

### 9.2 Soft Gates（强烈建议）

1. 持续维护多版本 Android 回归矩阵（Task 23，基线已完成）。
2. 持续维护 `cargo install` 烟测链路（Task 24，基线已完成）。
3. 持续维护 Nightly 回归守门（Task 25，基线已完成）。

## 10. 发布执行顺序

### 10.1 命令清单（Windows）

```powershell
# 1) 包内容检查
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/release-check.ps1

# 2) 设备全量回归（每台设备各跑一次）
C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/run-validation-gate.ps1 -Mode full -Serial <serial>

# 3) API 覆盖对账
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/api-coverage-report.ps1

# 4) dry-run 发布检查（分别在两个 crate 目录）
cargo publish --dry-run
```

### 10.2 命令清单（Linux/macOS）

```bash
# 1) 包内容检查
bash scripts/release-check.sh

# 2) 其余流程与 Windows 同步（设备脚本可用 pwsh 执行）
```

### 10.3 crates 发布顺序

1. 发布 `uiautomator`
2. 等 crates 索引可见
3. 发布 `uiautomator-cli`

原因：CLI 依赖库版本。

## 11. 发布验收记录模板

```markdown
## Release Evidence - <version>

- Date: <YYYY-MM-DD>
- Reviewer: <name>
- Commit: <sha>

### Device Matrix
- <run-id-1>: <serial> / <arch> / pass
- <run-id-2>: <serial> / <arch> / pass

### Artifacts
- summary.json: <path>
- summary.junit.xml: <path>
- api-coverage.json: <path>
- api-coverage.md: <path>

### Gate Check
- release-check: pass/fail
- full regression: pass/fail
- coverage review: pass/fail
- cargo publish --dry-run (uiautomator): pass/fail
- cargo publish --dry-run (uiautomator-cli): pass/fail

### Exceptions
- <none / details>

### Final Decision
- Release: yes/no
- Notes: <text>
```

## 12. 发布后守门建议

- Nightly 定时执行固化全量脚本。
- CI 消费 `summary.json` / `summary.junit.xml` 并保留工件。
- 失败自动分类到 `E-*` 并触发告警。

对应任务：见 `docs/public/TASKS.md`（Task 23/24/25 已完成，当前开放项主要是 Phase2 后续增强）。

