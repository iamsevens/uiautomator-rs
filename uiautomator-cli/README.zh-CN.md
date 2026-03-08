[English](./README.md) | [简体中文](./README.zh-CN.md)

# uiautomator-cli

用于初始化和管理 Android 设备上 ATX-Agent 的命令行工具。

## 概览

`uiautomator-cli` 提供稳定、可脚本化的设备侧环境初始化流程，适合在 Windows、Linux、macOS 上统一搭建自动化运行环境。

## 核心命令

- `uiautomator init`：安装、启动并校验 ATX-Agent 运行环境
- `uiautomator status`：检查服务健康状态、版本和端口
- `uiautomator restart`：重启服务
- `uiautomator uninstall`：移除已安装组件
- `uiautomator version`：显示 CLI 版本信息

常用参数：

- `-s, --serial <SERIAL>`：指定目标设备
- `-f, --force`：在 `init` 时强制重装

## 快速使用

先从 crates.io 安装：

```bash
cargo install uiautomator-cli
```

然后初始化设备：

```bash
# 初始化目标设备
uiautomator init --serial <serial> --force

# 查看状态
uiautomator status --serial <serial>

# 重启服务
uiautomator restart --serial <serial>

# 卸载
uiautomator uninstall --serial <serial>
```

如果当前只有一台在线 ADB 设备，可以省略 `--serial`。

## 从源码构建

```bash
cd uiautomator-cli
cargo build
cargo test --lib
```

运行 ignored / 集成测试（需要设备）：

```bash
cargo test -- --ignored --nocapture --test-threads=1
```

## 发布与验证

发布流程与门禁以仓库根目录 `PUBLISHING.zh-CN.md` 为准。
发布前验证基线与证据模板见 `../docs/public/TESTING_RELEASE.md`。

## 与 `uiautomator` 的关系

`uiautomator-cli` 依赖 `uiautomator`，发布顺序应为先 `uiautomator`，后 `uiautomator-cli`。

## 文档

- 公开测试与发布基线：`../docs/public/TESTING_RELEASE.md`
- 公开任务台账：`../docs/public/TASKS.md`

## 许可证

MIT。
