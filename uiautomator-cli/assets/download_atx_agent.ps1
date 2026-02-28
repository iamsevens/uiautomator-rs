# Download ATX-Agent binaries (multi-arch) and APK assets for uiautomator-cli.
$ErrorActionPreference = "Stop"

$AssetsDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $AssetsDir

$AtxAgentVersion = if ($env:ATX_AGENT_VERSION) { $env:ATX_AGENT_VERSION } else { "0.10.0" }
$UiAutomatorVersion = if ($env:UIAUTOMATOR_VERSION) { $env:UIAUTOMATOR_VERSION } else { "2.3.6" }

function Get-ExpectedAtxSha256 {
    param([Parameter(Mandatory = $true)][string]$Arch)
    switch ("$AtxAgentVersion|$Arch") {
        "0.10.0|armv7" { "4157ec30b7125266370782e03eba53edfee1e719dc8572c3e9565c212668b0f8" }
        "0.10.0|arm64" { "458bc5bacaae32abbe658262257b1a42345a566c684f93babd2dc0778ca6d78f" }
        "0.10.0|amd64" { "e338480e34fdaa9f0bedbf8d9c7e6c15e1335805c0e1c6d1d209f528590be3c9" }
        "0.10.0|386" { "bfde550ff7fdfe4926d96f6d23d15ace099cec1be9e2c52455efc8119a97f8a7" }
        default { "" }
    }
}

function Get-ExpectedApkSha256 {
    param([Parameter(Mandatory = $true)][string]$FileName)
    switch ("$UiAutomatorVersion|$FileName") {
        "2.3.6|app-uiautomator.apk" { "6f85594700ad96de89d012b3767049c2c6988510b68b31b439dd2a6dd93a30c9" }
        "2.3.6|app-uiautomator-test.apk" { "b768dfa7085389234feffc9246275ad5c3301db98424634bd9e06d916df0e3e4" }
        default { "" }
    }
}

function Test-Checksum {
    param(
        [Parameter(Mandatory = $true)][string]$FileName,
        [Parameter(Mandatory = $true)][string]$ExpectedSha256
    )

    if (-not $ExpectedSha256) {
        Write-Warning "Skip checksum for $FileName (no expected hash for selected version)"
        return
    }

    $actual = (Get-FileHash -Path $FileName -Algorithm SHA256).Hash.ToLower()
    if ($actual -ne $ExpectedSha256.ToLower()) {
        throw "Checksum mismatch for $FileName`nexpected: $ExpectedSha256`nactual  : $actual"
    }
}

function Download-AtxAgent {
    param(
        [Parameter(Mandatory = $true)][string]$Arch,
        [Parameter(Mandatory = $true)][string]$OutputName
    )

    $archiveName = "atx-agent_${AtxAgentVersion}_linux_${Arch}.tar.gz"
    $url = "https://github.com/openatx/atx-agent/releases/download/$AtxAgentVersion/$archiveName"

    Write-Host "Downloading $archiveName ..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri $url -OutFile $archiveName
    tar -xzf $archiveName atx-agent
    Move-Item -Force atx-agent $OutputName
    Remove-Item -Force $archiveName
    Test-Checksum -FileName $OutputName -ExpectedSha256 (Get-ExpectedAtxSha256 -Arch $Arch)
}

function Download-OrCopyApk {
    param(
        [Parameter(Mandatory = $true)][string]$FileName,
        [Parameter(Mandatory = $true)][string]$Url
    )

    if (Test-Path $FileName) {
        Write-Host "Skip $FileName (already exists)" -ForegroundColor DarkYellow
        Test-Checksum -FileName $FileName -ExpectedSha256 (Get-ExpectedApkSha256 -FileName $FileName)
        return
    }

    $source = Join-Path "..\..\uiautomator\assets" $FileName
    if (Test-Path $source) {
        Write-Host "Copy $FileName from ../../uiautomator/assets" -ForegroundColor DarkYellow
        Copy-Item -Force $source $FileName
        Test-Checksum -FileName $FileName -ExpectedSha256 (Get-ExpectedApkSha256 -FileName $FileName)
        return
    }

    Write-Host "Downloading $FileName ..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri $Url -OutFile $FileName
    Test-Checksum -FileName $FileName -ExpectedSha256 (Get-ExpectedApkSha256 -FileName $FileName)
}

Write-Host "Downloading ATX-Agent assets (version $AtxAgentVersion) ..." -ForegroundColor Green
Download-AtxAgent -Arch "armv7" -OutputName "atx-agent-armv7"
Download-AtxAgent -Arch "arm64" -OutputName "atx-agent-arm64"
Download-AtxAgent -Arch "amd64" -OutputName "atx-agent-amd64"
Download-AtxAgent -Arch "386" -OutputName "atx-agent-386"

Copy-Item -Force "atx-agent-armv7" "atx-agent"
Test-Checksum -FileName "atx-agent" -ExpectedSha256 (Get-ExpectedAtxSha256 -Arch "armv7")

Write-Host "Preparing UiAutomator APK assets (version $UiAutomatorVersion) ..." -ForegroundColor Green
Download-OrCopyApk -FileName "app-uiautomator.apk" -Url "https://github.com/openatx/android-uiautomator-server/releases/download/$UiAutomatorVersion/app-uiautomator.apk"
Download-OrCopyApk -FileName "app-uiautomator-test.apk" -Url "https://github.com/openatx/android-uiautomator-server/releases/download/$UiAutomatorVersion/app-uiautomator-test.apk"

Write-Host "Done." -ForegroundColor Green
$files = @(
    "atx-agent",
    "atx-agent-armv7",
    "atx-agent-arm64",
    "atx-agent-amd64",
    "atx-agent-386",
    "app-uiautomator.apk",
    "app-uiautomator-test.apk"
)
Get-ChildItem -Path . -File | Where-Object { $files -contains $_.Name } | Select-Object Name, Length
