# 贡献指南

感谢你对 `uiautomator-cli` 项目的关注！本文档将帮助你了解如何为项目做出贡献。

## 目录

- [开发环境设置](#开发环境设置)
- [TDD 开发流程](#tdd-开发流程)
- [代码规范](#代码规范)
- [测试指南](#测试指南)
- [提交代码](#提交代码)
- [发布流程](#发布流程)

## 开发环境设置

### 前置要求

- **Rust**: 1.70 或更高版本
- **Android SDK**: 包含 ADB 工具
- **Android 设备**: 用于集成测试（可选）
- **Git**: 版本控制

### 安装 Rust

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# 访问 https://rustup.rs/ 下载安装程序
```

### 克隆仓库

```bash
git clone https://github.com/iamsevens/uiautomator-rs.git
cd uiautomator-rs/uiautomator-cli
```

### 下载资源文件

```bash
cd assets

# Linux/macOS
./download_atx_agent.sh

# Windows
.\download_atx_agent.ps1

cd ..
```

### 验证环境

```bash
# 构建项目
cargo build

# 运行测试
cargo test --lib

# 运行示例
cargo run -- --help
```

## TDD 开发流程

本项目严格遵循 **测试驱动开发（TDD）** 方法。

### TDD 三步骤

#### 1. Red（红）- 先写测试

在实现功能之前，先编写测试用例。

```bash
# 创建测试文件
touch tests/my_feature_test.rs
```

```rust
// tests/my_feature_test.rs
#[test]
fn test_my_new_feature() {
    // 编写测试逻辑
    let result = my_new_feature();
    assert_eq!(result, expected_value);
}
```

```bash
# 运行测试（应该失败）
cargo test my_new_feature
# ❌ 测试失败 - 这是预期的！
```

#### 2. Green（绿）- 实现功能

编写最小化的代码使测试通过。

```rust
// src/lib.rs 或相关模块
pub fn my_new_feature() -> ReturnType {
    // 实现功能
    // 只写足够让测试通过的代码
}
```

```bash
# 再次运行测试
cargo test my_new_feature
# ✅ 测试通过！
```

#### 3. Refactor（重构）- 优化代码

在保持测试通过的前提下，优化代码质量。

```rust
// 重构代码，提高可读性和性能
pub fn my_new_feature() -> ReturnType {
    // 优化后的实现
}
```

```bash
# 确保所有测试仍然通过
cargo test
# ✅ 所有测试通过！
```

### TDD 实践示例

假设我们要添加一个新功能：验证资源文件的完整性。

**步骤 1: 写测试（Red）**

```rust
// tests/resources_test.rs
#[test]
fn test_verify_resource_integrity() {
    let resources = EmbeddedResources::get();
    let result = resources.verify_integrity();
    assert!(result.is_ok());
}
```

```bash
cargo test test_verify_resource_integrity
# ❌ 编译错误：verify_integrity 方法不存在
```

**步骤 2: 实现功能（Green）**

```rust
// src/resources.rs
impl EmbeddedResources {
    pub fn verify_integrity(&self) -> Result<(), String> {
        let computed_md5 = format!("{:x}", md5::compute(self.atx_agent));
        if computed_md5 != self.atx_agent_md5 {
            return Err("MD5 不匹配".to_string());
        }
        Ok(())
    }
}
```

```bash
cargo test test_verify_resource_integrity
# ✅ 测试通过！
```

**步骤 3: 重构（Refactor）**

```rust
// src/resources.rs
impl EmbeddedResources {
    pub fn verify_integrity(&self) -> Result<(), String> {
        self.verify_file_integrity(
            self.atx_agent,
            self.atx_agent_md5,
            "atx-agent"
        )?;
        self.verify_file_integrity(
            self.app_uiautomator_apk,
            self.app_uiautomator_apk_md5,
            "app-uiautomator.apk"
        )?;
        Ok(())
    }
    
    fn verify_file_integrity(
        &self,
        data: &[u8],
        expected_md5: &str,
        name: &str
    ) -> Result<(), String> {
        let computed_md5 = format!("{:x}", md5::compute(data));
        if computed_md5 != expected_md5 {
            return Err(format!("{} MD5 不匹配", name));
        }
        Ok(())
    }
}
```

```bash
cargo test
# ✅ 所有测试通过！
```

## 代码规范

### Rust 代码风格

遵循 Rust 官方代码风格指南。

```bash
# 格式化代码
cargo fmt

# 检查代码规范
cargo clippy

# 修复 clippy 警告
cargo clippy --fix
```

### 命名约定

- **函数和变量**: `snake_case`
- **类型和 trait**: `PascalCase`
- **常量**: `SCREAMING_SNAKE_CASE`
- **模块**: `snake_case`

```rust
// ✅ 好的命名
fn install_atx_agent() -> Result<()> { }
struct DeviceInfo { }
const DEFAULT_PORT: u16 = 7912;

// ❌ 不好的命名
fn InstallATXAgent() -> Result<()> { }
struct device_info { }
const defaultPort: u16 = 7912;
```

### 错误处理

使用 `Result` 和 `anyhow` 进行错误处理。

```rust
use anyhow::{Result, Context};

fn my_function() -> Result<()> {
    let value = some_operation()
        .context("操作失败")?;
    Ok(())
}
```

### 文档注释

为公共 API 添加文档注释。

```rust
/// 初始化设备并安装 ATX-Agent
///
/// # Arguments
///
/// * `serial` - 设备序列号（可选）
/// * `force` - 是否强制重新安装
///
/// # Examples
///
/// ```
/// let installer = Installer::new(None).await?;
/// installer.install(false).await?;
/// ```
pub async fn install(&self, force: bool) -> Result<()> {
    // 实现
}
```

## 测试指南

### 测试类型

#### 1. 单元测试

测试单个函数或模块。

```rust
// src/resources.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_embedded_resources_exist() {
        let resources = EmbeddedResources::get();
        assert!(!resources.atx_agent.is_empty());
    }
}
```

#### 2. 集成测试

测试完整的功能流程。

```rust
// tests/integration_init_test.rs
#[tokio::test]
#[ignore] // 需要真实设备
async fn test_full_init_workflow() {
    let result = commands::init::execute(None, false).await;
    assert!(result.is_ok());
}
```

#### 3. 属性测试

使用 `proptest` 测试通用属性。

```rust
// tests/property_resources_test.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_resource_md5_always_matches(seed in any::<u64>()) {
        let resources = EmbeddedResources::get();
        let computed_md5 = format!("{:x}", md5::compute(resources.atx_agent));
        prop_assert_eq!(computed_md5, resources.atx_agent_md5);
    }
}
```

### 运行测试

```bash
# 运行所有单元测试
cargo test --lib

# 运行特定测试
cargo test test_name

# 运行集成测试（需要设备）
cargo test --test integration_init_test -- --ignored

# 运行属性测试
cargo test property

# 运行所有测试
cargo test --all

# 显示测试输出
cargo test -- --nocapture
```

### 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html

# 查看报告
open tarpaulin-report.html
```

### 测试最佳实践

1. **每个功能都要有测试**
2. **测试应该独立且可重复**
3. **使用描述性的测试名称**
4. **测试边界条件和错误情况**
5. **保持测试简单明了**

```rust
// ✅ 好的测试
#[test]
fn test_installer_detects_no_device_error() {
    // 清晰的测试目的
    // 独立的测试数据
    // 明确的断言
}

// ❌ 不好的测试
#[test]
fn test1() {
    // 不清楚测试什么
    // 依赖外部状态
    // 模糊的断言
}
```

## 提交代码

### 分支策略

- `main`: 稳定版本
- `develop`: 开发分支
- `feature/*`: 新功能分支
- `bugfix/*`: 修复 bug 分支

### 工作流程

1. **Fork 仓库**

   在 GitHub 上 Fork 本仓库到你的账号。

2. **创建分支**

   ```bash
   git checkout -b feature/my-awesome-feature
   ```

3. **开发功能**

   遵循 TDD 流程开发。

4. **提交代码**

   ```bash
   git add .
   git commit -m "feat: 添加新功能"
   ```

5. **推送分支**

   ```bash
   git push origin feature/my-awesome-feature
   ```

6. **创建 Pull Request**

   在 GitHub 上创建 PR，描述你的更改。

### Commit 消息规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范。

格式：
```
<type>(<scope>): <subject>

<body>

<footer>
```

类型（type）：
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `test`: 添加测试
- `chore`: 构建或辅助工具更改

示例：
```bash
# 新功能
git commit -m "feat(installer): 添加设备自动检测功能"

# 修复 bug
git commit -m "fix(cli): 修复 --serial 参数解析错误"

# 文档更新
git commit -m "docs: 更新 README 安装说明"

# 测试
git commit -m "test: 添加资源完整性属性测试"
```

### Pull Request 检查清单

在提交 PR 之前，确保：

- [ ] 所有测试通过：`cargo test`
- [ ] 代码格式正确：`cargo fmt`
- [ ] 没有 clippy 警告：`cargo clippy`
- [ ] 添加了必要的测试
- [ ] 更新了相关文档
- [ ] Commit 消息符合规范
- [ ] PR 描述清晰

### Code Review

PR 提交后，维护者会进行 Code Review。请：

- 及时回复评论
- 根据反馈修改代码
- 保持友好和专业的态度

## 发布流程

### 版本号规范

遵循 [Semantic Versioning](https://semver.org/)：

- `MAJOR.MINOR.PATCH`
- `MAJOR`: 不兼容的 API 更改
- `MINOR`: 向后兼容的新功能
- `PATCH`: 向后兼容的 bug 修复

### 发布步骤

1. **更新版本号**

   ```toml
   # Cargo.toml
   [package]
   version = "0.2.0"
   ```

2. **更新 CHANGELOG**

   ```markdown
   # CHANGELOG.md
   
   ## [0.2.0] - 2024-01-18
   
   ### Added
   - 新功能 A
   - 新功能 B
   
   ### Fixed
   - 修复 bug X
   ```

3. **运行所有测试**

   ```bash
   cargo test --all
   ```

4. **构建发布版本**

   ```bash
   cargo build --release
   ```

5. **创建 Git 标签**

   ```bash
   git tag -a v0.2.0 -m "Release version 0.2.0"
   git push origin v0.2.0
   ```

6. **GitHub Actions 自动发布**

   推送标签后，GitHub Actions 会自动：
   - 构建多平台二进制文件
   - 创建 GitHub Release
   - 上传构建产物

## 获取帮助

如果你在贡献过程中遇到问题：

1. **查看文档**
   - [README.md](README.md)
   - [FAQ.md](FAQ.md)
   - 本文档

2. **搜索 Issues**
   - 可能已有人遇到相同问题

3. **创建 Issue**
   - 描述你的问题
   - 提供复现步骤

4. **加入讨论**
   - GitHub Discussions

## 行为准则

请遵守我们的 [行为准则](CODE_OF_CONDUCT.md)：

- 尊重他人
- 保持专业
- 接受建设性批评
- 关注对社区最有利的事情

---

感谢你的贡献！🎉
> **Canonical release gate entrypoint (repository root)**  
> `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\trigger-gh-release-gate.ps1 -Repo iamsevens/uiautomator-rs -Ref main`  
>  
> This command is the single release gate entrypoint (`Release Check` + `Publish Dry Run`).  
> If this document conflicts with root-level `CONTRIBUTING.md` / `PUBLISHING.md`, follow root docs.
