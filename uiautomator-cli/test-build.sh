#!/bin/bash
# 本地测试构建流程脚本

set -e

echo "=========================================="
echo "测试 uiautomator-cli 构建流程"
echo "=========================================="

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 进入项目目录
cd "$(dirname "$0")"

echo ""
echo "${YELLOW}步骤 1: 检查资源文件${NC}"
if [ ! -f "assets/atx-agent" ]; then
    echo "${RED}✗ 资源文件不存在，正在下载...${NC}"
    bash assets/download_atx_agent.sh
else
    echo "${GREEN}✓ 资源文件已存在${NC}"
fi

echo ""
echo "${YELLOW}步骤 2: 运行代码格式检查${NC}"
if cargo fmt -- --check; then
    echo "${GREEN}✓ 代码格式正确${NC}"
else
    echo "${RED}✗ 代码格式不正确，请运行 'cargo fmt'${NC}"
    exit 1
fi

echo ""
echo "${YELLOW}步骤 3: 运行 Clippy 检查${NC}"
if cargo clippy -- -D warnings; then
    echo "${GREEN}✓ Clippy 检查通过${NC}"
else
    echo "${RED}✗ Clippy 检查失败${NC}"
    exit 1
fi

echo ""
echo "${YELLOW}步骤 4: 运行单元测试${NC}"
if cargo test --lib; then
    echo "${GREEN}✓ 单元测试通过${NC}"
else
    echo "${RED}✗ 单元测试失败${NC}"
    exit 1
fi

echo ""
echo "${YELLOW}步骤 5: 运行集成测试（不包括需要设备的测试）${NC}"
if cargo test --test resources_test --test cli_test --test error_test; then
    echo "${GREEN}✓ 集成测试通过${NC}"
else
    echo "${RED}✗ 集成测试失败${NC}"
    exit 1
fi

echo ""
echo "${YELLOW}步骤 6: 运行属性测试${NC}"
if cargo test --test property_resources_test --test property_idempotent_test; then
    echo "${GREEN}✓ 属性测试通过${NC}"
else
    echo "${RED}✗ 属性测试失败${NC}"
    exit 1
fi

echo ""
echo "${YELLOW}步骤 7: 构建 Release 版本${NC}"
if cargo build --release; then
    echo "${GREEN}✓ Release 构建成功${NC}"
else
    echo "${RED}✗ Release 构建失败${NC}"
    exit 1
fi

echo ""
echo "${YELLOW}步骤 8: 验证二进制文件${NC}"
if [ -f "target/release/uiautomator" ] || [ -f "target/release/uiautomator.exe" ]; then
    echo "${GREEN}✓ 二进制文件存在${NC}"
    
    # 测试 version 命令
    if [ -f "target/release/uiautomator" ]; then
        echo "测试 version 命令："
        ./target/release/uiautomator version
    elif [ -f "target/release/uiautomator.exe" ]; then
        echo "测试 version 命令："
        ./target/release/uiautomator.exe version
    fi
else
    echo "${RED}✗ 二进制文件不存在${NC}"
    exit 1
fi

echo ""
echo "${GREEN}=========================================="
echo "✓ 所有检查通过！"
echo "==========================================${NC}"
echo ""
echo "下一步："
echo "1. 如果所有测试通过，可以提交代码"
echo "2. 创建标签发布新版本："
echo "   git tag vX.Y.Z"
echo "   git push origin vX.Y.Z"
echo "3. GitHub Actions 将自动构建并发布"
