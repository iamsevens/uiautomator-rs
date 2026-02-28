#!/bin/bash
# 离线环境测试脚本

echo "=========================================="
echo "  uiautomator-cli 离线环境测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查是否以 root 运行（某些网络隔离需要）
if [ "$EUID" -ne 0 ] && command -v iptables &> /dev/null; then
    echo -e "${YELLOW}警告: 某些网络隔离功能需要 root 权限${NC}"
    echo "建议使用: sudo $0"
    echo ""
fi

# 测试计数
PASSED=0
FAILED=0

echo "=========================================="
echo -e "${BLUE}准备离线环境${NC}"
echo "=========================================="
echo ""

# 检测操作系统
OS=$(uname -s)
echo "操作系统: $OS"
echo ""

# 保存当前网络状态
echo "保存当前网络配置..."
if [ "$OS" = "Linux" ]; then
    # Linux: 使用 iptables
    if command -v iptables &> /dev/null && [ "$EUID" -eq 0 ]; then
        echo "使用 iptables 阻止网络访问"
        NETWORK_METHOD="iptables"
        
        # 保存当前规则
        iptables-save > /tmp/iptables-backup.rules
        
        # 阻止所有出站连接（除了本地）
        iptables -A OUTPUT -d 127.0.0.0/8 -j ACCEPT
        iptables -A OUTPUT -d 10.0.0.0/8 -j ACCEPT
        iptables -A OUTPUT -d 172.16.0.0/12 -j ACCEPT
        iptables -A OUTPUT -d 192.168.0.0/16 -j ACCEPT
        iptables -A OUTPUT -j DROP
        
        echo -e "${GREEN}✓ 网络已隔离${NC}"
    else
        echo -e "${YELLOW}警告: 无法使用 iptables 隔离网络${NC}"
        echo "请手动断开网络连接，然后按 Enter 继续..."
        read
        NETWORK_METHOD="manual"
    fi
elif [ "$OS" = "Darwin" ]; then
    # macOS
    echo -e "${YELLOW}macOS 系统${NC}"
    echo "请手动断开网络连接（Wi-Fi 或以太网），然后按 Enter 继续..."
    read
    NETWORK_METHOD="manual"
else
    # 其他系统
    echo -e "${YELLOW}未知系统${NC}"
    echo "请手动断开网络连接，然后按 Enter 继续..."
    read
    NETWORK_METHOD="manual"
fi
echo ""

# 验证网络已断开
echo "=========================================="
echo -e "${BLUE}验证网络状态${NC}"
echo "=========================================="
echo ""

echo "测试外网连接..."
if ping -c 1 -W 2 8.8.8.8 &> /dev/null; then
    echo -e "${RED}✗ 警告: 仍然可以访问外网${NC}"
    echo "网络隔离可能不完全"
else
    echo -e "${GREEN}✓ 无法访问外网（符合预期）${NC}"
fi
echo ""

echo "测试 HTTP 连接..."
if curl -s --connect-timeout 2 https://www.google.com &> /dev/null; then
    echo -e "${RED}✗ 警告: 仍然可以进行 HTTP 连接${NC}"
    echo "网络隔离可能不完全"
else
    echo -e "${GREEN}✓ 无法进行 HTTP 连接（符合预期）${NC}"
fi
echo ""

# 检查设备连接
echo "=========================================="
echo -e "${BLUE}检查设备连接${NC}"
echo "=========================================="
echo ""

if ! adb devices | grep -q "device$"; then
    echo -e "${RED}错误: 未找到连接的设备${NC}"
    echo "请连接设备后重试"
    
    # 恢复网络
    if [ "$NETWORK_METHOD" = "iptables" ] && [ "$EUID" -eq 0 ]; then
        echo "恢复网络配置..."
        iptables-restore < /tmp/iptables-backup.rules
        rm /tmp/iptables-backup.rules
    fi
    
    exit 1
fi

device=$(adb devices | grep "device$" | head -1 | awk '{print $1}')
echo -e "${GREEN}✓ 找到设备: $device${NC}"
echo ""

# 开始离线测试
echo "=========================================="
echo -e "${BLUE}离线环境功能测试${NC}"
echo "=========================================="
echo ""

# 测试 1: version 命令
echo "1. 测试 version 命令（不需要网络）"
if uiautomator version; then
    echo -e "${GREEN}✓ version 命令成功${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ version 命令失败${NC}"
    ((FAILED++))
fi
echo ""

# 测试 2: help 命令
echo "2. 测试 help 命令（不需要网络）"
if uiautomator --help > /dev/null; then
    echo -e "${GREEN}✓ help 命令成功${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ help 命令失败${NC}"
    ((FAILED++))
fi
echo ""

# 测试 3: init 命令（关键测试）
echo "3. 测试 init 命令（使用内置资源）"
echo "这是最重要的测试，验证是否使用内置资源文件..."
if uiautomator --serial "$device" init --force; then
    echo -e "${GREEN}✓ init 命令成功（完全离线）${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ init 命令失败${NC}"
    echo "可能尝试访问网络或资源文件有问题"
    ((FAILED++))
fi
echo ""

# 测试 4: status 命令
echo "4. 测试 status 命令"
if uiautomator --serial "$device" status; then
    echo -e "${GREEN}✓ status 命令成功${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ status 命令失败${NC}"
    ((FAILED++))
fi
echo ""

# 测试 5: restart 命令
echo "5. 测试 restart 命令"
if uiautomator --serial "$device" restart; then
    echo -e "${GREEN}✓ restart 命令成功${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ restart 命令失败${NC}"
    ((FAILED++))
fi
echo ""

# 测试 6: 再次 status
echo "6. 再次测试 status 命令"
if uiautomator --serial "$device" status; then
    echo -e "${GREEN}✓ status 命令成功${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ status 命令失败${NC}"
    ((FAILED++))
fi
echo ""

# 测试 7: 幂等性测试
echo "7. 测试幂等性（重复 init）"
if uiautomator --serial "$device" init; then
    echo -e "${GREEN}✓ 重复 init 成功${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 重复 init 失败${NC}"
    ((FAILED++))
fi
echo ""

# 恢复网络
echo "=========================================="
echo -e "${BLUE}恢复网络配置${NC}"
echo "=========================================="
echo ""

if [ "$NETWORK_METHOD" = "iptables" ] && [ "$EUID" -eq 0 ]; then
    echo "恢复 iptables 规则..."
    iptables-restore < /tmp/iptables-backup.rules
    rm /tmp/iptables-backup.rules
    echo -e "${GREEN}✓ 网络已恢复${NC}"
else
    echo "请手动恢复网络连接，然后按 Enter 继续..."
    read
fi
echo ""

# 验证网络已恢复
echo "验证网络连接..."
if ping -c 1 -W 2 8.8.8.8 &> /dev/null; then
    echo -e "${GREEN}✓ 网络已恢复${NC}"
else
    echo -e "${YELLOW}警告: 网络可能未完全恢复${NC}"
fi
echo ""

# 总结
echo "=========================================="
echo "测试总结"
echo "=========================================="
echo -e "通过: ${GREEN}${PASSED}${NC}"
echo -e "失败: ${RED}${FAILED}${NC}"
echo "=========================================="

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有离线测试通过！${NC}"
    echo ""
    echo "结论: CLI 工具可以在完全离线环境下正常工作"
    exit 0
else
    echo -e "${RED}✗ 有测试失败${NC}"
    echo ""
    echo "请检查:"
    echo "1. 资源文件是否正确嵌入"
    echo "2. 是否有代码尝试访问网络"
    echo "3. 错误日志中的详细信息"
    exit 1
fi
