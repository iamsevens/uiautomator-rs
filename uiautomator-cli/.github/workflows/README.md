# GitHub Actions 工作流

本目录包含 uiautomator-cli 项目的 CI/CD 工作流配置。

## 工作流说明

### 1. CI (ci.yml)

**触发条件：**
- 推送到 `main`、`master` 或 `develop` 分支
- 针对这些分支的 Pull Request

**功能：**
- 在 Linux、macOS 和 Windows 上运行测试
- 运行 clippy 进行代码检查
- 检查代码格式

**步骤：**
1. 检出代码
2. 安装 Rust 工具链
3. 缓存 cargo 依赖
4. 下载资源文件（atx-agent、APK）
5. 运行所有测试
6. 运行 clippy 检查
7. 检查代码格式

### 2. Release (release.yml)

**触发条件：**
- 推送以 `v` 开头的标签（例如 `v0.1.0`）

**功能：**
- 为多个平台构建二进制文件
- 创建 GitHub Release
- 上传构建产物和校验和

**支持的平台：**
- Linux x86_64
- Linux aarch64
- macOS x86_64 (Intel)
- macOS aarch64 (Apple Silicon)
- Windows x86_64

**步骤：**
1. 为每个目标平台构建二进制文件
2. 生成 SHA256 校验和
3. 创建 GitHub Release
4. 上传所有二进制文件和校验和

### 3. Build Test (build-test.yml)

**触发条件：**
- 手动触发（workflow_dispatch）
- 修改构建相关文件的 Pull Request

**功能：**
- 测试构建流程而不创建 Release
- 验证二进制文件可以正常构建和运行

**步骤：**
1. 为主要平台构建二进制文件
2. 验证二进制文件存在
3. 测试运行 `version` 命令

## 使用说明

### 发布新版本

1. 更新 `Cargo.toml` 中的版本号
2. 更新 CHANGELOG（如果有）
3. 提交更改
4. 创建并推送标签：
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
5. GitHub Actions 将自动构建并创建 Release

### 手动测试构建

在 GitHub 仓库页面：
1. 进入 "Actions" 标签
2. 选择 "Build Test" 工作流
3. 点击 "Run workflow"
4. 选择分支并运行

### 本地测试

在推送之前，可以本地测试：

```bash
# 运行测试
cd uiautomator-cli
cargo test

# 检查格式
cargo fmt -- --check

# 运行 clippy
cargo clippy -- -D warnings

# 构建 release 版本
cargo build --release
```

## 依赖说明

### GitHub Actions

- `actions/checkout@v4`: 检出代码
- `dtolnay/rust-toolchain@stable`: 安装 Rust 工具链
- `actions/cache@v4`: 缓存依赖
- `actions/upload-artifact@v4`: 上传构建产物
- `actions/download-artifact@v4`: 下载构建产物
- `softprops/action-gh-release@v1`: 创建 GitHub Release

### 交叉编译

对于 Linux aarch64 目标，使用 `cross` 工具进行交叉编译：
- https://github.com/cross-rs/cross

## 故障排查

### 资源文件下载失败

如果资源文件下载失败，检查：
1. `assets/download_atx_agent.sh` 脚本是否可执行
2. `assets/download_atx_agent.ps1` 脚本是否存在
3. 下载 URL 是否可访问

### 构建失败

如果构建失败，检查：
1. `Cargo.toml` 依赖是否正确
2. `build.rs` 是否正确处理资源文件
3. 资源文件是否存在于 `assets/` 目录

### 测试失败

如果测试失败，检查：
1. 单元测试是否需要真实设备（应该使用 mock）
2. 集成测试是否标记为 `#[ignore]`
3. 属性测试是否配置正确

## 安全注意事项

- `GITHUB_TOKEN` 由 GitHub Actions 自动提供
- 不需要额外配置 secrets
- Release 工作流需要 `contents: write` 权限

## 性能优化

- 使用 cargo 缓存加速构建
- 并行构建多个目标平台
- 只在必要时下载资源文件
