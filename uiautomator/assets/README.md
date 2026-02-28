# ATX-Agent 资源文件

本目录保存 `uiautomator` 在 ATX-Agent 安装模式下需要的资源。

## 文件说明

### 已内置（仓库提供）
- `u2.jar`
- `app-uiautomator.apk`

### ATX-Agent 安装相关（建议通过脚本下载）
- `atx-agent-armv7`
- `atx-agent-arm64`
- `atx-agent-amd64`
- `atx-agent-386`
- `app-uiautomator-test.apk`

脚本会额外生成一个兼容文件：
- `atx-agent`（默认复制自 `atx-agent-armv7`，用于兼容旧逻辑）

## 为什么要多架构

Android 设备/模拟器 ABI 不同（如 `arm64-v8a`、`armeabi-v7a`、`x86_64`、`x86`）。
如果只提供单一架构二进制，ATX-Agent 可能“进程存在但服务不可用”，表现为 `/version` 超时。

## 快速下载

### Linux/macOS
```bash
cd assets
chmod +x download_atx_agent.sh
./download_atx_agent.sh
```

### Windows PowerShell
```powershell
cd assets
.\download_atx_agent.ps1
```

## 版本来源

默认下载版本：
- ATX-Agent: `0.10.0`
- android-uiautomator-server: `2.3.6`

可通过环境变量覆盖：
- `ATX_AGENT_VERSION`
- `UIAUTOMATOR_VERSION`

## 验证建议

下载后可执行：
```bash
cargo test --lib
```

并在真机/模拟器上执行 CLI 初始化与状态检查：
```bash
cd ../uiautomator-cli
cargo run -- init -s <serial>
cargo run -- status -s <serial>
```
