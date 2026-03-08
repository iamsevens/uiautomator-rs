# ATX-Agent 安装指南

本文档详细说明如何安装和使用 ATX-Agent 模式。

## 什么是 ATX-Agent？

ATX-Agent 是一个运行在 Android 设备上的守护进程，提供：

- **生产级稳定性**：守护进程保证服务持续运行
- **自动恢复**：服务崩溃时自动重启
- **屏幕锁定保护**：锁屏后服务继续运行
- **功能丰富**：提供截图、文件传输、应用管理等额外功能

## 为什么需要 ATX-Agent？

**Direct 模式的问题**：
- Android 系统会主动清理后台进程
- 屏幕锁定后直接启动的服务会停止
- 长时间运行的测试任务容易中断

**ATX-Agent 的优势**：
- 守护进程级别运行，不易被系统杀死
- 自动监控和重启 uiautomator2 服务
- 经过 Python uiautomator2 大量实践验证

## 安装方式

### 方式 1: 通过 Python uiautomator2（推荐）

这是最简单的方式，适合大多数用户。

**步骤**：

1. 安装 Python uiautomator2：
   ```bash
   pip install uiautomator2
   ```

2. 连接 Android 设备并初始化：
   ```bash
   # 确保设备已连接
   adb devices
   
   # 初始化（会自动安装 atx-agent）
   python -m uiautomator2 init
   ```

3. 在 Rust 代码中使用：
   ```rust
   use uiautomator::Device;
   
   #[tokio::main]
   async fn main() -> uiautomator::Result<()> {
       // 自动检测模式（会优先使用 ATX-Agent）
       let device = Device::connect(None).await?;
       
       // 或显式使用 ATX-Agent 模式
       let device = Device::connect_with_mode(
           None,
           uiautomator::ServerMode::AtxAgent
       ).await?;
       
       Ok(())
   }
   ```

### 方式 2: 通过 Rust 库安装（不依赖 Python）

如果您不想安装 Python 环境，可以使用 Rust 库直接安装。

**前置要求**：
- 需要下载约 12-14MB 的资源文件
- 需要启用 `atx-agent-install` feature

**步骤**：

#### 1. 启用 feature

在 `Cargo.toml` 中添加：

```toml
[dependencies]
uiautomator = { version = "1.0.1", features = ["atx-agent-install"] }
```

#### 2. 下载资源文件

**Linux/macOS**：
```bash
cd assets
chmod +x download_atx_agent.sh
./download_atx_agent.sh
```

**Windows PowerShell**：
```powershell
cd assets
.\download_atx_agent.ps1
```

**手动下载**：

如果脚本无法运行，可以手动下载：

1. atx-agent 二进制文件：
   - 下载地址：https://github.com/openatx/atx-agent/releases/latest
   - 选择 `atx-agent_*_linux_armv7.tar.gz`
   - 解压后放到 `assets/atx-agent`

2. app-uiautomator-test.apk：
   - 下载地址：https://github.com/openatx/android-uiautomator-server/releases/latest
   - 下载 `app-uiautomator-test.apk`
   - 放到 `assets/app-uiautomator-test.apk`

#### 3. 验证资源文件

```bash
cargo run --example verify_assets
```

应该看到：
```
【ATX-Agent 模式资源】
  ✓ ATX-Agent 二进制文件                (8.xx MB)
  ✓ UiAutomator2 测试 APK            (1.xx MB)
```

#### 4. 安装到设备

```rust
use uiautomator::Device;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 连接到设备（Direct 模式）
    let device = Device::connect_quick(None).await?;
    
    // 检查是否已安装
    let installed = device.check_atx_agent_installed().await?;
    if !installed {
        println!("正在安装 ATX-Agent...");
        device.install_atx_agent(false).await?;
        println!("安装完成！");
    }
    
    // 现在可以使用 ATX-Agent 模式
    let device = Device::connect(None).await?;
    
    // 使用设备...
    let info = device.info().await?;
    println!("设备信息: {}x{}", info.display_width, info.display_height);
    
    Ok(())
}
```

## 使用示例

### 基础使用

```rust
use uiautomator::Device;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 自动检测模式（推荐）
    let device = Device::connect(None).await?;
    
    // 执行操作
    device.click(100, 200).await?;
    
    Ok(())
}
```

### 显式指定模式

```rust
use uiautomator::{Device, ServerMode};

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    // 强制使用 ATX-Agent 模式
    let device = Device::connect_with_mode(
        None,
        ServerMode::AtxAgent
    ).await?;
    
    // 如果 ATX-Agent 不可用，会返回错误
    
    Ok(())
}
```

### 服务管理

```rust
use uiautomator::Device;

#[tokio::main]
async fn main() -> uiautomator::Result<()> {
    let device = Device::connect_quick(None).await?;
    
    // 启动服务
    device.start_atx_agent().await?;
    
    // 停止服务
    device.stop_atx_agent().await?;
    
    // 重启服务
    device.restart_atx_agent().await?;
    
    Ok(())
}
```

## 故障排查

### 问题 1: 无法连接到 ATX-Agent

**症状**：
```
Error: DeviceConnection("ATX-Agent 不可用")
```

**解决方案**：

1. 检查 atx-agent 是否已安装：
   ```bash
   adb shell "test -f /data/local/tmp/atx-agent && echo 'installed' || echo 'not installed'"
   ```

2. 检查 atx-agent 是否正在运行：
   ```bash
   adb shell "ps | grep atx-agent"
   ```

3. 手动启动 atx-agent：
   ```bash
   adb shell "/data/local/tmp/atx-agent server -d"
   ```

4. 检查端口转发：
   ```bash
   adb forward tcp:7912 tcp:7912
   ```

### 问题 2: 资源文件下载失败

**症状**：
```
cargo:warning=atx-agent 文件不存在，ATX-Agent 安装功能将不可用
```

**解决方案**：

1. 检查网络连接
2. 使用代理或镜像下载
3. 手动下载资源文件（见上文"手动下载"部分）

### 问题 3: 安装失败

**症状**：
```
Error: Adb("安装 app-uiautomator.apk 失败")
```

**解决方案**：

1. 确保设备有足够的存储空间
2. 检查设备是否允许安装未知来源应用
3. 尝试手动安装：
   ```bash
   adb install -r -t assets/app-uiautomator.apk
   adb install -r -t assets/app-uiautomator-test.apk
   ```

### 问题 4: 服务启动后立即停止

**症状**：
服务启动后几秒钟就停止了

**解决方案**：

1. 检查设备日志：
   ```bash
   adb logcat | grep atx-agent
   ```

2. 确保设备没有启用省电模式
3. 将应用添加到白名单（不被系统杀死）

## 性能对比

| 特性 | Direct 模式 | ATX-Agent 模式 |
|------|------------|----------------|
| 启动时间 | 5-10 秒 | 10-15 秒（首次） |
| 稳定性 | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| 自动恢复 | ❌ | ✅ |
| 屏幕锁定保护 | ❌ | ✅ |
| 资源占用 | 低 | 中 |
| 适用场景 | 开发测试 | 生产环境 |

## 最佳实践

1. **开发阶段**：使用 Direct 模式快速迭代
   ```rust
   let device = Device::connect_quick(None).await?;
   ```

2. **生产环境**：使用 ATX-Agent 模式保证稳定性
   ```rust
   let device = Device::connect(None).await?;
   ```

3. **CI/CD**：在测试环境中预先安装 ATX-Agent
   ```bash
   python -m uiautomator2 init
   ```

4. **长时间运行**：定期检查服务状态
   ```rust
   // 每小时检查一次
   loop {
       // 执行测试...
       
       tokio::time::sleep(Duration::from_secs(3600)).await;
   }
   ```

## 参考资料

- [atx-agent GitHub](https://github.com/openatx/atx-agent)
- [Python uiautomator2](https://github.com/openatx/uiautomator2)
- [android-uiautomator-server](https://github.com/openatx/android-uiautomator-server)

## 常见问题

**Q: ATX-Agent 和 Direct 模式可以同时使用吗？**

A: 不可以。一个设备同一时间只能使用一种模式。但您可以在不同的测试中切换模式。

**Q: 安装 ATX-Agent 会影响设备性能吗？**

A: 影响很小。ATX-Agent 是一个轻量级守护进程，正常情况下 CPU 和内存占用都很低。

**Q: 可以在多个设备上同时使用 ATX-Agent 吗？**

A: 可以。每个设备独立运行自己的 ATX-Agent 实例。

**Q: ATX-Agent 需要 root 权限吗？**

A: 不需要。ATX-Agent 运行在 `/data/local/tmp` 目录，不需要 root 权限。

**Q: 如何卸载 ATX-Agent？**

A: 
```bash
# 停止服务
adb shell "killall atx-agent"

# 删除文件
adb shell "rm /data/local/tmp/atx-agent"

# 卸载 APK
adb uninstall com.github.uiautomator
adb uninstall com.github.uiautomator.test
```
