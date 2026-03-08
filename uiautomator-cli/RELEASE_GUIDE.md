# 发布指南

本文档说明如何发布 uiautomator-cli 的新版本。

## 前提条件

1. 拥有仓库的写入权限
2. 本地已安装 Rust 工具链
3. 本地已安装 Git
4. 已配置 GitHub 远程仓库

## 快速发布流程

### 1. 本地测试

在发布前，先在本地运行完整的测试：

**Linux/macOS:**
```bash
cd uiautomator-cli
bash test-build.sh
```

**Windows:**
```powershell
cd uiautomator-cli
.\test-build.ps1
```

### 2. 更新版本号

编辑 `Cargo.toml`，更新版本号：

```toml
[package]
name = "uiautomator-cli"
version = "1.0.1"  # 从 1.0.0 更新到 1.0.1
```

### 3. 更新依赖锁文件

```bash
cargo update
```

### 4. 提交更改

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 1.0.1"
git push origin main
```

### 5. 创建并推送标签

```bash
# 创建标签（注意版本号前加 v）
git tag v1.0.1

# 推送标签到远程（这将触发 Release 工作流）
git push origin v1.0.1
```

### 6. 监控构建

1. 访问 GitHub 仓库的 Actions 页面
2. 查看 "Release" 工作流的运行状态
3. 等待所有平台的构建完成（约 10-20 分钟）

### 7. 验证发布

1. 访问 GitHub Releases 页面
2. 确认新版本已创建
3. 检查所有二进制文件和校验和文件已上传
4. 下载并测试至少一个二进制文件

## 详细说明

### 版本号规范

遵循语义化版本（Semantic Versioning 2.0.0）：

- **主版本号（Major）**: 不兼容的 API 变更
  - 例如：`1.0.0` -> `2.0.0`
  
- **次版本号（Minor）**: 向后兼容的功能新增
  - 例如：`1.0.0` -> `1.1.0`
  
- **修订号（Patch）**: 向后兼容的问题修复
  - 例如：`1.0.0` -> `1.0.1`

### GitHub Actions 工作流

#### CI 工作流 (ci.yml)

**触发条件：**
- 推送到主分支
- Pull Request

**功能：**
- 在 Linux、macOS、Windows 上运行测试
- 运行 clippy 和格式检查

#### Release 工作流 (release.yml)

**触发条件：**
- 推送以 `v` 开头的标签

**功能：**
- 为多个平台构建二进制文件
- 创建 GitHub Release
- 上传构建产物

**支持的平台：**
- Linux x86_64
- Linux aarch64
- macOS x86_64 (Intel)
- macOS aarch64 (Apple Silicon)
- Windows x86_64

#### Build Test 工作流 (build-test.yml)

**触发条件：**
- 手动触发

**功能：**
- 测试构建流程而不创建 Release

### 构建产物

每个 Release 包含以下文件：

```
uiautomator-linux-x86_64
uiautomator-linux-x86_64.sha256
uiautomator-linux-aarch64
uiautomator-linux-aarch64.sha256
uiautomator-macos-x86_64
uiautomator-macos-x86_64.sha256
uiautomator-macos-aarch64
uiautomator-macos-aarch64.sha256
uiautomator-windows-x86_64.exe
uiautomator-windows-x86_64.exe.sha256
```

### 验证校验和

下载后验证文件完整性：

**Linux/macOS:**
```bash
sha256sum -c uiautomator-linux-x86_64.sha256
```

**Windows (PowerShell):**
```powershell
$hash = Get-FileHash uiautomator-windows-x86_64.exe -Algorithm SHA256
$expected = Get-Content uiautomator-windows-x86_64.exe.sha256
if ($hash.Hash -eq $expected.Split()[0]) {
    Write-Host "✓ 校验和匹配"
} else {
    Write-Host "✗ 校验和不匹配"
}
```

## 故障排查

### 构建失败

如果 Release 工作流失败：

1. **检查日志**
   - 在 Actions 页面查看详细日志
   - 找到失败的步骤

2. **常见问题**
   - 资源文件下载失败：检查下载脚本
   - 编译错误：本地测试是否通过
   - 测试失败：检查测试代码

3. **修复流程**
   ```bash
   # 删除有问题的标签
   git tag -d v0.2.0
   git push origin :refs/tags/v0.2.0
   
   # 修复问题后重新发布
   git tag v0.2.0
   git push origin v0.2.0
   ```

### 手动测试构建

如果想在不创建 Release 的情况下测试构建：

1. 访问 GitHub Actions 页面
2. 选择 "Build Test" 工作流
3. 点击 "Run workflow"
4. 选择分支并运行

### 本地跨平台构建

如果需要在本地测试跨平台构建：

**使用 cross 工具：**
```bash
# 安装 cross
cargo install cross --git https://github.com/cross-rs/cross

# 构建 Linux aarch64
cross build --release --target aarch64-unknown-linux-gnu

# 构建其他目标
cross build --release --target x86_64-unknown-linux-gnu
```

## 发布后任务

### 1. 更新文档

- [ ] 更新 README 中的版本号
- [ ] 更新安装脚本中的 URL（如果是首次发布）
- [ ] 更新示例代码中的版本引用

### 2. 公告

- [ ] 在 GitHub Release 页面编辑发布说明
- [ ] 添加变更日志
- [ ] 在相关社区发布公告

### 3. 监控

- [ ] 监控 GitHub Issues
- [ ] 检查用户反馈
- [ ] 准备快速修复严重问题

## 回滚流程

如果发现严重问题需要回滚：

```bash
# 1. 删除 GitHub Release（在网页上操作）

# 2. 删除标签
git tag -d v0.2.0
git push origin :refs/tags/v0.2.0

# 3. 修复问题

# 4. 发布修复版本（增加补丁号）
git tag v0.2.1
git push origin v0.2.1
```

## 最佳实践

1. **始终在本地测试**
   - 运行 `test-build.sh` 或 `test-build.ps1`
   - 确保所有测试通过

2. **遵循语义化版本**
   - 破坏性变更增加主版本号
   - 新功能增加次版本号
   - Bug 修复增加修订号

3. **编写清晰的发布说明**
   - 列出主要变更
   - 说明升级步骤
   - 标注破坏性变更

4. **保持 CHANGELOG**
   - 记录每个版本的变更
   - 方便用户了解更新内容

5. **测试安装脚本**
   - 确保用户可以轻松安装
   - 验证下载 URL 正确

## 参考资料

- [语义化版本规范](https://semver.org/lang/zh-CN/)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Rust 发布最佳实践](https://doc.rust-lang.org/cargo/reference/publishing.html)

## 联系方式

如有问题，请：
- 提交 GitHub Issue
- 查看 `.github/workflows/README.md`
- 查看 `.github/RELEASE_CHECKLIST.md`
> **Canonical release gate entrypoint (repository root)**  
> `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\trigger-gh-release-gate.ps1 -Repo iamsevens/uiautomator-rs -Ref main`  
>  
> This command is the single release gate entrypoint (`Release Check` + `Publish Dry Run`).  
> If this document conflicts with root-level `PUBLISHING.md`, follow `PUBLISHING.md`.
