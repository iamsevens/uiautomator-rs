Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

if (-not (Test-Path ".\test-app\gradlew.bat")) {
    throw "test-app\gradlew.bat not found"
}

Push-Location .\test-app
try {
    .\gradlew.bat assembleDebug --no-daemon
    if ($LASTEXITCODE -ne 0) {
        throw "gradlew assembleDebug failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}

$apkPath = Join-Path $repoRoot "test-app\app\build\outputs\apk\debug\app-debug.apk"
if (-not (Test-Path $apkPath)) {
    throw "test-app apk build output not found: $apkPath"
}

Write-Host "test-app apk ready: $apkPath"
