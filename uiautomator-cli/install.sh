#!/bin/bash
# uiautomator-cli 安装脚本
# 支持 Linux 和 macOS

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# 检测操作系统和架构
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    case "$OS" in
        linux)
            OS="linux"
            ;;
        darwin)
            OS="macos"
            ;;
        *)
            print_error "不支持的操作系统: $OS"
            exit 1
            ;;
    esac
    
    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            print_error "不支持的架构: $ARCH"
            exit 1
            ;;
    esac
    
    print_info "检测到平台: $OS-$ARCH"
}

# 下载二进制文件
download_binary() {
    BINARY_NAME="uiautomator-${OS}-${ARCH}"
    
    # TODO: 替换为实际的 GitHub Release URL
    GITHUB_REPO="your-username/uiautomator"
    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/${BINARY_NAME}"
    
    print_info "正在下载 uiautomator CLI 工具..."
    
    # 检查是否安装了 curl 或 wget
    if command -v curl &> /dev/null; then
        curl -L "$DOWNLOAD_URL" -o /tmp/uiautomator
    elif command -v wget &> /dev/null; then
        wget -O /tmp/uiautomator "$DOWNLOAD_URL"
    else
        print_error "需要 curl 或 wget 来下载文件"
        exit 1
    fi
    
    if [ $? -ne 0 ]; then
        print_error "下载失败"
        exit 1
    fi
    
    print_info "下载完成"
}

# 安装二进制文件
install_binary() {
    chmod +x /tmp/uiautomator
    
    # 尝试安装到 /usr/local/bin
    INSTALL_DIR="/usr/local/bin"
    
    if [ -w "$INSTALL_DIR" ]; then
        mv /tmp/uiautomator "$INSTALL_DIR/uiautomator"
        print_info "已安装到 $INSTALL_DIR/uiautomator"
    else
        print_warning "$INSTALL_DIR 需要管理员权限"
        sudo mv /tmp/uiautomator "$INSTALL_DIR/uiautomator"
        print_info "已安装到 $INSTALL_DIR/uiautomator"
    fi
}

# 验证安装
verify_installation() {
    if command -v uiautomator &> /dev/null; then
        VERSION=$(uiautomator version 2>/dev/null || echo "unknown")
        print_info "安装成功！版本: $VERSION"
        echo ""
        echo "使用方法:"
        echo "  uiautomator init      # 初始化设备"
        echo "  uiautomator status    # 查看状态"
        echo "  uiautomator --help    # 查看帮助"
    else
        print_error "安装验证失败"
        exit 1
    fi
}

# 主函数
main() {
    echo "uiautomator-cli 安装程序"
    echo "========================"
    echo ""
    
    detect_platform
    download_binary
    install_binary
    verify_installation
    
    echo ""
    print_info "安装完成！"
}

# 运行主函数
main
