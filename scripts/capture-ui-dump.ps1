param(
    [Parameter(Mandatory = $true)]
    [string]$Serial,

    [string]$OutputRoot = "internal/testlogs/ui-dumps"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$safeSerialForPath = ($Serial -replace '[\\/:*?"<>|]', '_')
$runId = Get-Date -Format "yyyyMMdd-HHmmss"
$outputDir = Join-Path $repoRoot (Join-Path $OutputRoot "$runId-$safeSerialForPath")
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

$remotePath = "/sdcard/__u2_window_dump.xml"
$localPath = Join-Path $outputDir "window_dump.xml"

Write-Host "==> Validate target device"
$state = & adb -s $Serial get-state 2>&1
if ($LASTEXITCODE -ne 0 -or ($state | Out-String).Trim() -ne "device") {
    $text = ($state | ForEach-Object { $_.ToString() }) -join "`n"
    throw "target device '$Serial' is not ready:`n$text"
}

Write-Host "==> Dump hierarchy on device"
$dumpOut = & adb -s $Serial shell uiautomator dump $remotePath 2>&1
if ($LASTEXITCODE -ne 0) {
    $text = ($dumpOut | ForEach-Object { $_.ToString() }) -join "`n"
    throw "uiautomator dump failed:`n$text"
}

Write-Host "==> Pull hierarchy xml to log directory"
$pullOut = & adb -s $Serial pull $remotePath $localPath 2>&1
if ($LASTEXITCODE -ne 0) {
    $text = ($pullOut | ForEach-Object { $_.ToString() }) -join "`n"
    throw "adb pull failed:`n$text"
}

& adb -s $Serial shell rm -f $remotePath 2>&1 | Out-Null

Write-Host "saved: $localPath"
