#!/bin/bash
# 错误场景测试脚本

echo "=========================================="
echo "  uiautomator-cli 错误场景测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试计数
PASSED=0
FAILED=0

# 测试函数
test_error_handling() {
    local test_name="$1"
    local command="$2"
    local expected_pattern="$3"
    
    echo "测试: $test_name"
    echo "命令: $command"
    
    output=$(eval "$command" 2>&1 || true)
    
    if echo "$output" | grep -qi "$expected_pattern"; then
        echo -e "${GREEN}✓ 正确处理错误${NC}"
        echo "输出包含预期的错误信息: $expected_pattern"
        ((PASSED++))
    else
        echo -e "${RED}✗ 错误处理不正确${NC}"
        echo "预期包含: $expected_pattern"
        echo "实际输出:"
        echo "$output"
        ((FAILED++))
    fi
    echo ""
}

echo "=========================================="
echo -e "${BLUE}1. 命令行错误${NC}"
echo "=========================================="
echo ""

test_error_handling \
    "无效命令" \
    "uiautomator invalid-command" \
    "error\|invalid\|unknown"

test_error_handling \
    "无效选项" \
    "uiautomator init --invalid-option" \
    "error\|invalid\|unknown"

test_error_handling \
    "缺少参数值" \
    "uiautomator --serial" \
    "error\|requires\|expected"

echo "=========================================="
echo -e "${BLUE}2. 设备错误${NC}"
echo "=========================================="
echo ""

test_error_handling \
    "无效设备序列号" \
    "uiautomator --serial invalid-device-12345 status" \
    "未找到\|not found\|error\|device"

# 检查是否有设备连接
if ! adb devices | grep -q "device$"; then
    echo -e "${YELLOW}注意: 未找到连接的设备${NC}"
    echo "以下测试需要设备连接，将被跳过"
    echo ""
else
    echo "=========================================="
    echo -e "${BLUE}3. 服务错误${NC}"
    echo "=========================================="
    echo ""
    
    # 获取第一个设备
    device=$(adb devices | grep "device$" | head -1 | awk '{print $1}')
    
    echo "使用设备: $device"
    echo ""
    
    # 确保设备未初始化（尝试卸载）
    echo "准备测试环境（卸载服务）..."
    uiautomator --serial "$device" uninstall 2>/dev/null || true
    echo ""
    
    test_error_handling \
        "服务未安装时重启" \
        "uiautomator --serial $device restart" \
        "未安装\|not installed\|not found\|init"
    
    test_error_handling \
        "服务未安装时查看状态" \
        "uiautomator --serial $device status" \
        "未运行\|not running\|stopped"
fi

echo "=========================================="
echo -e "${BLUE}4. 帮助信息测试${NC}"
echo "=========================================="
echo ""

echo "测试: 主帮助信息"
if uiautomator --help | grep -q "Usage\|USAGE\|Commands"; then
    echo -e "${GREEN}✓ 帮助信息正常显示${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 帮助信息显示异常${NC}"
    ((FAILED++))
fi
echo ""

echo "测试: init 命令帮助"
if uiautomator init --help | grep -q "init\|初始化"; then
    echo -e "${GREEN}✓ init 帮助信息正常显示${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ init 帮助信息显示异常${NC}"
    ((FAILED++))
fi
echo ""

echo "=========================================="
echo -e "${BLUE}5. 版本信息测试${NC}"
echo "=========================================="
echo ""

echo "测试: version 命令"
if uiautomator version | grep -q "version\|版本\|[0-9]\+\.[0-9]\+"; then
    echo -e "${GREEN}✓ 版本信息正常显示${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 版本信息显示异常${NC}"
    ((FAILED++))
fi
echo ""

echo "=========================================="
echo "测试总结"
echo "=========================================="
echo -e "通过: ${GREEN}${PASSED}${NC}"
echo -e "失败: ${RED}${FAILED}${NC}"
echo "=========================================="

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有错误场景测试通过！${NC}"
    exit 0
else
    echo -e "${RED}✗ 有测试失败${NC}"
    exit 1
fi
