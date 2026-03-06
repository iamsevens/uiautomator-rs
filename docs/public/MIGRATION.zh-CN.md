[English](./MIGRATION.md) | [简体中文](./MIGRATION.zh-CN.md)

# Migration Guide

## 1. 当前状态

当前还没有已经发布并且需要用户执行迁移动作的破坏性变更。

现阶段 `0.1.x` 版本线会尽量保持向后兼容，最近新增的能力，例如：

- `Coord`
- `Device::click_coord`
- `Device::long_click_coord`
- `Device::double_click_coord`
- `Device::swipe_coord`
- `Device::drag_coord`
- `Device::set_cache_ttl`
- `Device::clear_cache`
- `Device::disable_cache`

都属于非破坏性扩展。

## 2. 后续版本的迁移记录方式

如果未来版本引入了对使用者可见的破坏性变更，本文件会明确记录：

1. 从哪个版本迁移到哪个版本。
2. 受影响的 API 范围。
3. 对应的替换方式。
4. 变更属于源码破坏、行为破坏，还是环境/部署破坏。
5. 是否存在自动迁移方式或兼容层。

## 3. 当前升级建议

对于当前已发布版本线的升级，建议：

1. 依赖升级后重新执行你自己的设备回归。
2. 如果你依赖 ATX 环境搭建，在至少一台真机和一台模拟器上重新验证 `uiautomator-cli init`。
3. 如果你需要百分比坐标，优先使用 `*_coord` 方法，而不是改写已有像素调用语义。
4. 设备信息缓存默认保持按需开启，不建议在没有明确收益的路径上默认打开。

## 4. 相关文档

- `REQUIREMENTS.md`
- `DESIGN.md`
- `API_DOCS.md`
- `TESTING_RELEASE.md`
