Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

function Get-GradleDistributionInfo {
    param([Parameter(Mandatory = $true)][string]$WrapperPropertiesPath)

    $wrapperDir = Split-Path -Parent $WrapperPropertiesPath
    $distributionUrl = ""

    foreach ($line in (Get-Content $WrapperPropertiesPath -Encoding utf8)) {
        $trimmed = $line.Trim()
        if ($trimmed.StartsWith("#") -or [string]::IsNullOrWhiteSpace($trimmed)) {
            continue
        }
        if ($trimmed -like "distributionUrl=*") {
            $distributionUrl = $trimmed.Substring("distributionUrl=".Length)
            break
        }
    }

    if ([string]::IsNullOrWhiteSpace($distributionUrl)) {
        return $null
    }

    $normalizedUrl = $distributionUrl -replace '\\:', ':'
    $isFileRelative = -not ($normalizedUrl -match '^[a-zA-Z][a-zA-Z0-9+\.-]*://')
    if (-not $isFileRelative) {
        return $null
    }

    $distributionPath = [System.IO.Path]::GetFullPath((Join-Path $wrapperDir $normalizedUrl))
    $zipName = [System.IO.Path]::GetFileName($distributionPath)
    if ([string]::IsNullOrWhiteSpace($zipName)) {
        return $null
    }

    return [PSCustomObject]@{
        DistributionPath = $distributionPath
        ZipName          = $zipName
    }
}

function Ensure-GradleDistributionZip {
    param(
        [Parameter(Mandatory = $true)]
        [string]$WrapperPropertiesPath
    )

    $info = Get-GradleDistributionInfo -WrapperPropertiesPath $WrapperPropertiesPath
    if ($null -eq $info) {
        return
    }

    if (Test-Path $info.DistributionPath) {
        Write-Host "gradle distribution zip found: $($info.DistributionPath)"
        return
    }

    $downloadUrl = if (-not [string]::IsNullOrWhiteSpace($env:GRADLE_DIST_DOWNLOAD_URL)) {
        $env:GRADLE_DIST_DOWNLOAD_URL
    }
    else {
        "https://services.gradle.org/distributions/$($info.ZipName)"
    }

    $targetDir = Split-Path -Parent $info.DistributionPath
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null

    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Write-Host "downloading gradle distribution: $downloadUrl"
    Invoke-WebRequest -Uri $downloadUrl -OutFile $info.DistributionPath -UseBasicParsing

    if (-not (Test-Path $info.DistributionPath)) {
        throw "failed to prepare gradle distribution zip: $($info.DistributionPath)"
    }
    Write-Host "gradle distribution zip prepared: $($info.DistributionPath)"
}

if (-not (Test-Path ".\test-app\gradlew.bat")) {
    throw "test-app\gradlew.bat not found"
}

$wrapperProperties = Join-Path $repoRoot "test-app\gradle\wrapper\gradle-wrapper.properties"
if (Test-Path $wrapperProperties) {
    Ensure-GradleDistributionZip -WrapperPropertiesPath $wrapperProperties
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
