# 更新日志

所有重要的项目变更都会记录在此文件中。

格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)。

## [未发布]

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
