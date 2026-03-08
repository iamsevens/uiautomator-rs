# 更新日志

所有重要的项目变更都会记录在此文件中。

格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)。

## [未发布]

### 说明
- 暂无未发布变更。

## [1.0.0] - 2026-03-08

### 说明
- 首个稳定版本发布，功能与 0.1.3 保持一致。

## [0.1.3] - 2026-03-06

### 修复
- `Installer::uninstall` 不再无条件吞掉关键卸载失败；停止服务、删除二进制、卸载 APK 若出现真实失败将汇总返回，避免“看起来卸载成功，实际残留未清”。
- 多设备场景下的 `new_with_adb(None, ...)` 测试断言更新为匹配当前语义：当存在多台设备时，显式要求用户传入 `--serial`。

### 改进
- Nightly / 设备回归相关脚本链路已固定到本地 GUI runner，发布前验证更贴近真实设备环境。

## [0.1.2] - 2026-03-05

### 修复
- 修复 `cargo publish --dry-run` 场景下的依赖兼容问题：
  - `Installer::wait_service_ready_with_fallback` 改为向 `wait_for_atx_agent_ready` 传入 `Duration`，兼容已发布的 `uiautomator` API 签名。
  - 避免 `Option<Duration>` 调用在 crates.io 依赖解析路径下触发编译失败。

### 改进
- 调整 crate 打包清单：发布包不再包含 `tests/**`，减小包体积并降低发布噪音。
- 完善发布与仓库文档：
  - 新增 `THIRD_PARTY_NOTICES.md`
  - 修复 README 中的仓库与发布链接占位符
  - 新增手动触发的发布检查/干跑工作流

## [0.1.0] - 2026-02-25

### 首发
- 提供 ATX-Agent 的安装、状态查询、重启、卸载命令。
- 内置资源完整性校验（MD5）。
- 支持 Windows / Linux / macOS。
