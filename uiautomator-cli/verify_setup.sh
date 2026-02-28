#!/bin/bash
# 验证 uiautomator-cli 项目设置

echo "验证 uiautomator-cli 项目设置..."
echo ""

# 检查资源文件
echo "检查资源文件..."
if [ -f "assets/atx-agent" ]; then
    echo "✓ atx-agent 存在"
    ls -lh assets/atx-agent
else
    echo "✗ atx-agent 不存在"
    exit 1
fi

if [ -f "assets/app-uiautomator.apk" ]; then
    echo "✓ app-uiautomator.apk 存在"
    ls -lh assets/app-uiautomator.apk
else
    echo "✗ app-uiautomator.apk 不存在"
    exit 1
fi

if [ -f "assets/app-uiautomator-test.apk" ]; then
    echo "✓ app-uiautomator-test.apk 存在"
    ls -lh assets/app-uiautomator-test.apk
else
    echo "✗ app-uiautomator-test.apk 不存在"
    exit 1
fi

echo ""
echo "尝试构建项目..."
cargo build

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ 项目构建成功！"
    echo ""
    echo "运行项目..."
    cargo run
else
    echo "✗ 项目构建失败"
    exit 1
fi
