#!/bin/bash
# 多设备测试脚本

set -e

echo "=========================================="
echo "  uiautomator-cli 多设备测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 获取所有设备
echo "正在检测连接的设备..."
devices=$(adb devices | grep -v "List" | grep "device$" | awk '{print $1}')

if [ -z "$devices" ]; then
    echo -e "${RED}错误: 未找到连接的设备${NC}"
    echo "请连接至少一个 Android 设备或启动模拟器"
    exit 1
fi

# 统计设备数量
device_count=$(echo "$devices" | wc -l)
echo -e "${GREEN}✓ 找到 ${device_count} 个设备${NC}"
echo ""

echo "设备列表:"
echo "$devices" | nl
echo ""

# 测试计数
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 为每个设备执行测试
device_num=1
for device in $devices; do
    echo "=========================================="
    echo -e "${BLUE}测试设备 ${device_num}/${device_count}: ${device}${NC}"
    echo "=========================================="
    echo ""
    
    # 测试 1: 初始化
    echo "1. 初始化设备..."
    ((TOTAL_TESTS++))
    if uiautomator --serial "$device" init; then
        echo -e "${GREEN}✓ 初始化成功${NC}"
        ((PASSED_TESTS++))
    else
        echo -e "${RED}✗ 初始化失败${NC}"
        ((FAILED_TESTS++))
    fi
    echo ""
    
    # 测试 2: 查看状态
    echo "2. 查看状态..."
    ((TOTAL_TESTS++))
    if uiautomator --serial "$device" status; then
        echo -e "${GREEN}✓ 状态查询成功${NC}"
        ((PASSED_TESTS++))
    else
        echo -e "${RED}✗ 状态查询失败${NC}"
        ((FAILED_TESTS++))
    fi
    echo ""
    
    # 测试 3: 重启服务
    echo "3. 重启服务..."
    ((TOTAL_TESTS++))
    if uiautomator --serial "$device" restart; then
        echo -e "${GREEN}✓ 重启成功${NC}"
        ((PASSED_TESTS++))
    else
        echo -e "${RED}✗ 重启失败${NC}"
        ((FAILED_TESTS++))
    fi
    echo ""
    
    # 测试 4: 再次查看状态
    echo "4. 再次查看状态..."
    ((TOTAL_TESTS++))
    if uiautomator --serial "$device" status; then
        echo -e "${GREEN}✓ 状态查询成功${NC}"
        ((PASSED_TESTS++))
    else
        echo -e "${RED}✗ 状态查询失败${NC}"
        ((FAILED_TESTS++))
    fi
    echo ""
    
    ((device_num++))
done

# 如果有多个设备，测试设备独立性
if [ $device_count -gt 1 ]; then
    echo "=========================================="
    echo -e "${BLUE}测试设备独立性${NC}"
    echo "=========================================="
    echo ""
    
    # 获取第一个和第二个设备
    device1=$(echo "$devices" | sed -n '1p')
    device2=$(echo "$devices" | sed -n '2p')
    
    echo "同时查询两个设备的状态..."
    echo ""
    
    echo "设备 1 ($device1):"
    ((TOTAL_TESTS++))
    if uiautomator --serial "$device1" status; then
        echo -e "${GREEN}✓ 设备 1 状态查询成功${NC}"
        ((PASSED_TESTS++))
    else
        echo -e "${RED}✗ 设备 1 状态查询失败${NC}"
        ((FAILED_TESTS++))
    fi
    echo ""
    
    echo "设备 2 ($device2):"
    ((TOTAL_TESTS++))
    if uiautomator --serial "$device2" status; then
        echo -e "${GREEN}✓ 设备 2 状态查询成功${NC}"
        ((PASSED_TESTS++))
    else
        echo -e "${RED}✗ 设备 2 状态查询失败${NC}"
        ((FAILED_TESTS++))
    fi
    echo ""
fi

# 测试默认设备选择
echo "=========================================="
echo -e "${BLUE}测试默认设备选择${NC}"
echo "=========================================="
echo ""

echo "不指定 --serial 参数，应该使用第一个设备..."
((TOTAL_TESTS++))
if uiautomator status; then
    echo -e "${GREEN}✓ 默认设备选择成功${NC}"
    ((PASSED_TESTS++))
else
    echo -e "${RED}✗ 默认设备选择失败${NC}"
    ((FAILED_TESTS++))
fi
echo ""

# 测试无效设备序列号
echo "=========================================="
echo -e "${BLUE}测试错误处理${NC}"
echo "=========================================="
echo ""

echo "使用无效的设备序列号..."
((TOTAL_TESTS++))
if uiautomator --serial "invalid-device-12345" status 2>&1 | grep -q "未找到\|not found\|error"; then
    echo -e "${GREEN}✓ 正确处理无效设备序列号${NC}"
    ((PASSED_TESTS++))
else
    echo -e "${RED}✗ 未正确处理无效设备序列号${NC}"
    ((FAILED_TESTS++))
fi
echo ""

# 总结
echo "=========================================="
echo "测试总结"
echo "=========================================="
echo "设备数量: $device_count"
echo "总测试数: $TOTAL_TESTS"
echo -e "通过: ${GREEN}${PASSED_TESTS}${NC}"
echo -e "失败: ${RED}${FAILED_TESTS}${NC}"
echo "=========================================="

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过！${NC}"
    exit 0
else
    echo -e "${RED}✗ 有测试失败${NC}"
    exit 1
fi
