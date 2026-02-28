# uiautomator-cli 安装脚本
# 支持 Windows

$ErrorActionPreference = "Stop"

# 颜色定义
function Write-Success {
    param([string]$Message)
    Write-Host "✓ $Message" -ForegroundColor Green
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "✗ $Message" -ForegroundColor Red
}

function Write-Warning-Custom {
    param([string]$Message)
    Write-Host "⚠ $Message" -ForegroundColor Yellow
}

# 检测架构
function Get-Architecture {
    $arch = $env:PROCESSOR_ARCHITECTURE
    
    switch ($arch) {
        "AMD64" { return "x86_64" }
        "ARM64" { return "aarch64" }
        default {
            Write-Error-Custom "不支持的架构: $arch"
            exit 1
        }
    }
}

# 下载二进制文件
function Download-Binary {
    param([string]$Architecture)
    
    $BinaryName = "uiautomator-windows-$Architecture.exe"
    
    # TODO: 替换为实际的 GitHub Release URL
    $GitHubRepo = "your-username/uiautomator"
    $DownloadUrl = "https://github.com/$GitHubRepo/releases/latest/download/$BinaryName"
    
    Write-Success "正在下载 uiautomator CLI 工具..."
    
    $TempFile = "$env:TEMP\uiautomator.exe"
    
    try {
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempFile -UseBasicParsing
        Write-Success "下载完成"
        return $TempFile
    }
    catch {
        Write-Error-Custom "下载失败: $_"
        exit 1
    }
}

# 安装二进制文件
function Install-Binary {
    param([string]$TempFile)
    
    # 安装到用户的 PATH 中
    $InstallDir = "$env:LOCALAPPDATA\Microsoft\WindowsApps"
    $InstallPath = "$InstallDir\uiautomator.exe"
    
    # 确保目录存在
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    
    # 复制文件
    Copy-Item -Path $TempFile -Destination $InstallPath -Force
    Remove-Item -Path $TempFile -Force
    
    Write-Success "已安装到 $InstallPath"
    
    # 检查 PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$InstallDir*") {
        Write-Warning-Custom "$InstallDir 不在 PATH 中"
        Write-Host "请手动添加到 PATH 或重启终端"
    }
}

# 验证安装
function Test-Installation {
    try {
        $version = & uiautomator version 2>$null
        Write-Success "安装成功！版本: $version"
        Write-Host ""
        Write-Host "使用方法:"
        Write-Host "  uiautomator init      # 初始化设备"
        Write-Host "  uiautomator status    # 查看状态"
        Write-Host "  uiautomator --help    # 查看帮助"
    }
    catch {
        Write-Warning-Custom "安装完成，但无法验证（可能需要重启终端）"
    }
}

# 主函数
function Main {
    Write-Host "uiautomator-cli 安装程序" -ForegroundColor Cyan
    Write-Host "========================" -ForegroundColor Cyan
    Write-Host ""
    
    $arch = Get-Architecture
    Write-Success "检测到架构: $arch"
    
    $tempFile = Download-Binary -Architecture $arch
    Install-Binary -TempFile $tempFile
    Test-Installation
    
    Write-Host ""
    Write-Success "安装完成！"
}

# 运行主函数
Main
