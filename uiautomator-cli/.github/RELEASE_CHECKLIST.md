# 发布检查清单

在发布新版本之前，请确保完成以下所有步骤。

## 发布前检查

### 1. 代码质量

- [ ] 所有测试通过
  ```bash
  cd uiautomator-cli
  cargo test
  ```

- [ ] 代码格式正确
  ```bash
  cargo fmt -- --check
  ```

- [ ] Clippy 检查通过
  ```bash
  cargo clippy -- -D warnings
  ```

- [ ] 属性测试通过
  ```bash
  cargo test --test property_resources_test
  cargo test --test property_idempotent_test
  ```

### 2. 文档更新

- [ ] 更新 `Cargo.toml` 中的版本号
- [ ] 更新 `README.md` 中的版本引用（如果有）
- [ ] 更新 CHANGELOG（添加新版本的变更说明）
- [ ] 检查所有文档链接是否有效
- [ ] 确认安装脚本中的 URL 正确

### 3. 功能验证

- [ ] 本地构建成功
  ```bash
  cargo build --release
  ```

- [ ] 测试 `init` 命令（需要真实设备）
  ```bash
  target/release/uiautomator init
  ```

- [ ] 测试 `status` 命令
  ```bash
  target/release/uiautomator status
  ```

- [ ] 测试 `restart` 命令
  ```bash
  target/release/uiautomator restart
  ```

- [ ] 测试 `version` 命令
  ```bash
  target/release/uiautomator version
  ```

- [ ] 测试 `--help` 输出
  ```bash
  target/release/uiautomator --help
  ```

### 4. 跨平台验证

- [ ] 在 Linux 上测试构建
- [ ] 在 macOS 上测试构建
- [ ] 在 Windows 上测试构建

### 5. 资源文件

- [ ] 确认 `assets/` 目录包含所有必需文件
  - `atx-agent`
  - `app-uiautomator.apk`
  - `app-uiautomator-test.apk`

- [ ] 验证资源文件 MD5 校验和
  ```bash
  cargo run --example verify_assets  # 如果有这个示例
  ```

## 发布流程

### 1. 准备发布

```bash
# 1. 确保在主分支上
git checkout main
git pull origin main

# 2. 更新版本号（例如 0.1.0 -> 0.2.0）
# 编辑 Cargo.toml 中的 version 字段

# 3. 更新 Cargo.lock
cargo update

# 4. 提交更改
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.0"

# 5. 推送到远程
git push origin main
```

### 2. 创建标签

```bash
# 创建标签（版本号前加 v）
git tag v0.2.0

# 推送标签（这将触发 Release 工作流）
git push origin v0.2.0
```

### 3. 监控构建

1. 访问 GitHub 仓库的 Actions 页面
2. 查看 "Release" 工作流的运行状态
3. 等待所有平台的构建完成（约 10-20 分钟）

### 4. 验证 Release

- [ ] 检查 GitHub Releases 页面
- [ ] 确认所有二进制文件已上传
  - `uiautomator-linux-x86_64`
  - `uiautomator-linux-aarch64`
  - `uiautomator-macos-x86_64`
  - `uiautomator-macos-aarch64`
  - `uiautomator-windows-x86_64.exe`

- [ ] 确认所有 SHA256 校验和文件已上传

- [ ] 下载并测试至少一个二进制文件
  ```bash
  # Linux/macOS
  curl -L https://github.com/YOUR_REPO/releases/download/v0.2.0/uiautomator-linux-x86_64 -o uiautomator
  chmod +x uiautomator
  ./uiautomator version
  
  # Windows (PowerShell)
  Invoke-WebRequest -Uri "https://github.com/YOUR_REPO/releases/download/v0.2.0/uiautomator-windows-x86_64.exe" -OutFile "uiautomator.exe"
  .\uiautomator.exe version
  ```

### 5. 更新安装脚本（如果需要）

如果这是第一个正式发布，更新安装脚本中的 URL：

- [ ] 更新 `install.sh` 中的下载 URL
- [ ] 更新 `install.ps1` 中的下载 URL
- [ ] 测试安装脚本

### 6. 发布公告

- [ ] 在 GitHub Release 页面编辑发布说明
- [ ] 添加主要变更和新功能
- [ ] 添加升级说明（如果有破坏性变更）
- [ ] 在相关社区发布公告（如果适用）

## 发布后检查

### 1. 验证安装

- [ ] 使用安装脚本测试安装
  ```bash
  # Linux/macOS
  curl -sSL https://raw.githubusercontent.com/YOUR_REPO/main/uiautomator-cli/install.sh | bash
  
  # Windows (PowerShell)
  iwr -useb https://raw.githubusercontent.com/YOUR_REPO/main/uiautomator-cli/install.ps1 | iex
  ```

### 2. 监控问题

- [ ] 监控 GitHub Issues 中的新问题
- [ ] 检查是否有安装或使用问题的报告
- [ ] 准备快速修复严重问题

### 3. 文档更新

- [ ] 更新主 README 中的版本徽章（如果有）
- [ ] 更新文档中的示例版本号
- [ ] 确认所有链接指向正确的版本

## 回滚流程

如果发现严重问题需要回滚：

1. **删除有问题的 Release**
   - 在 GitHub Releases 页面删除该 Release
   - 删除对应的 Git 标签：
     ```bash
     git tag -d v0.2.0
     git push origin :refs/tags/v0.2.0
     ```

2. **修复问题**
   - 修复代码中的问题
   - 运行所有测试确保修复有效

3. **发布修复版本**
   - 增加补丁版本号（例如 0.2.0 -> 0.2.1）
   - 重新执行发布流程

## 版本号规范

遵循语义化版本（Semantic Versioning）：

- **主版本号（Major）**：不兼容的 API 变更
- **次版本号（Minor）**：向后兼容的功能新增
- **修订号（Patch）**：向后兼容的问题修复

示例：
- `0.1.0` -> `0.2.0`：新增功能
- `0.2.0` -> `0.2.1`：修复 bug
- `0.2.1` -> `1.0.0`：稳定版本或重大变更

## 联系方式

如有问题，请联系：
- 提交 GitHub Issue
- 查看项目文档
- 联系维护者

---

**注意：** 首次发布前，请确保所有 URL 和仓库信息已正确配置。
