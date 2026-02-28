#!/bin/bash
# 快速功能测试脚本

set -e

echo "=========================================="
echo "  uiautomator-cli 快速功能测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试计数
PASSED=0
FAILED=0

# 测试函数
test_command() {
    local test_name="$1"
    local command="$2"
    
    echo -n "测试: $test_name ... "
    
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ 通过${NC}"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}✗ 失败${NC}"
        ((FAILED++))
        return 1
    fi
}

# 开始测试
echo "1. 检查 CLI 工具是否可用"
if ! command -v uiautomator &> /dev/null; then
    echo -e "${RED}错误: uiautomator 命令未找到${NC}"
    echo "请确保已编译并安装 CLI 工具"
    exit 1
fi
echo -e "${GREEN}✓ CLI 工具已安装${NC}"
echo ""

echo "2. 测试基本命令"
test_command "version 命令" "uiautomator version"
test_command "help 命令" "uiautomator --help"
echo ""

echo "3. 检查设备连接"
if ! adb devices | grep -q "device$"; then
    echo -e "${YELLOW}警告: 未找到连接的设备${NC}"
    echo "跳过需要设备的测试"
    echo ""
    echo "=========================================="
    echo "测试结果: ${GREEN}${PASSED} 通过${NC}, ${RED}${FAILED} 失败${NC}"
    echo "=========================================="
    exit 0
fi

device_count=$(adb devices | grep "device$" | wc -l)
echo -e "${GREEN}✓ 找到 ${device_count} 个设备${NC}"
echo ""

echo "4. 测试设备操作"
echo "执行: uiautomator init"
if uiautomator init; then
    echo -e "${GREEN}✓ 初始化成功${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 初始化失败${NC}"
    ((FAILED++))
fi
echo ""

echo "执行: uiautomator status"
if uiautomator status; then
    echo -e "${GREEN}✓ 状态查询成功${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 状态查询失败${NC}"
    ((FAILED++))
fi
echo ""

echo "执行: uiautomator restart"
if uiautomator restart; then
    echo -e "${GREEN}✓ 重启成功${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 重启失败${NC}"
    ((FAILED++))
fi
echo ""

echo "再次执行: uiautomator status"
if uiautomator status; then
    echo -e "${GREEN}✓ 状态查询成功${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 状态查询失败${NC}"
    ((FAILED++))
fi
echo ""

echo "=========================================="
echo "测试结果: ${GREEN}${PASSED} 通过${NC}, ${RED}${FAILED} 失败${NC}"
echo "=========================================="

if [ $FAILED -eq 0 ]; then
    exit 0
else
    exit 1
fi
