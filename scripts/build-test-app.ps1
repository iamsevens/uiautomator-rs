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

    $cacheRoot = if (-not [string]::IsNullOrWhiteSpace($env:UIAUTOMATOR_CACHE_DIR)) {
        $env:UIAUTOMATOR_CACHE_DIR
    }
    elseif (-not [string]::IsNullOrWhiteSpace($env:RUNNER_TEMP)) {
        Join-Path $env:RUNNER_TEMP "uiautomator-cache"
    }
    else {
        Join-Path $env:TEMP "uiautomator-cache"
    }
    $cachedZip = Join-Path $cacheRoot "gradle\$($info.ZipName)"
    New-Item -ItemType Directory -Path (Split-Path -Parent $cachedZip) -Force | Out-Null

    if (-not (Test-Path $cachedZip)) {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Write-Host "downloading gradle distribution: $downloadUrl"
        Invoke-WebRequest -Uri $downloadUrl -OutFile $cachedZip -UseBasicParsing
    }
    else {
        Write-Host "using cached gradle distribution: $cachedZip"
    }

    Copy-Item -Path $cachedZip -Destination $info.DistributionPath -Force

    if (-not (Test-Path $info.DistributionPath)) {
        throw "failed to prepare gradle distribution zip: $($info.DistributionPath)"
    }
    Write-Host "gradle distribution zip prepared: $($info.DistributionPath)"
}

function Resolve-AndroidSdkDir {
    $candidates = New-Object System.Collections.Generic.List[string]

    if (-not [string]::IsNullOrWhiteSpace($env:ANDROID_HOME)) {
        $candidates.Add($env:ANDROID_HOME)
    }
    if (-not [string]::IsNullOrWhiteSpace($env:ANDROID_SDK_ROOT)) {
        $candidates.Add($env:ANDROID_SDK_ROOT)
    }

    $adbCmd = Get-Command adb -ErrorAction SilentlyContinue
    if ($null -ne $adbCmd) {
        $adbPath = $adbCmd.Source
        if (-not [string]::IsNullOrWhiteSpace($adbPath) -and $adbPath -match '([\\/])platform-tools\1adb(\.exe)?$') {
            $sdkFromAdb = Split-Path -Parent (Split-Path -Parent $adbPath)
            if (-not [string]::IsNullOrWhiteSpace($sdkFromAdb)) {
                $candidates.Add($sdkFromAdb)
            }
        }
    }

    if (-not [string]::IsNullOrWhiteSpace($env:LOCALAPPDATA)) {
        $candidates.Add((Join-Path $env:LOCALAPPDATA "Android\Sdk"))
    }
    if (-not [string]::IsNullOrWhiteSpace($env:USERPROFILE)) {
        $candidates.Add((Join-Path $env:USERPROFILE "AppData\Local\Android\Sdk"))
    }
    $candidates.Add("C:\Android\Sdk")
    $candidates.Add("D:\Android\Sdk")

    $usersRoot = Join-Path $env:SystemDrive "Users"
    if (Test-Path $usersRoot) {
        $userDirs = Get-ChildItem -Path $usersRoot -Directory -ErrorAction SilentlyContinue
        foreach ($dir in $userDirs) {
            $candidates.Add((Join-Path $dir.FullName "AppData\Local\Android\Sdk"))
        }
    }

    foreach ($candidate in ($candidates | Select-Object -Unique)) {
        if ([string]::IsNullOrWhiteSpace($candidate)) {
            continue
        }
        try {
            $platformToolsAdb = Join-Path $candidate "platform-tools\adb.exe"
            if (Test-Path $platformToolsAdb -ErrorAction Stop) {
                return [System.IO.Path]::GetFullPath($candidate)
            }
        }
        catch {
            continue
        }
    }

    return ""
}

function Ensure-AndroidLocalProperties {
    param(
        [Parameter(Mandatory = $true)]
        [string]$TestAppRoot
    )

    $localPropertiesPath = Join-Path $TestAppRoot "local.properties"
    if (Test-Path $localPropertiesPath) {
        foreach ($line in (Get-Content $localPropertiesPath -Encoding utf8)) {
            $trimmed = $line.Trim()
            if ($trimmed -like "sdk.dir=*") {
                $raw = $trimmed.Substring("sdk.dir=".Length)
                $fromLocalProperties = ($raw -replace '\\:', ':' -replace '\\\\', '\')
                if (-not [string]::IsNullOrWhiteSpace($fromLocalProperties)) {
                    $env:ANDROID_HOME = $fromLocalProperties
                    $env:ANDROID_SDK_ROOT = $fromLocalProperties
                }
                break
            }
        }
    }

    $sdkDir = Resolve-AndroidSdkDir
    if ([string]::IsNullOrWhiteSpace($sdkDir)) {
        throw "Android SDK not found for runner account. Set ANDROID_HOME/ANDROID_SDK_ROOT or install SDK in a shared path."
    }

    $env:ANDROID_HOME = $sdkDir
    $env:ANDROID_SDK_ROOT = $sdkDir
    if (-not [string]::IsNullOrWhiteSpace($env:GITHUB_ENV)) {
        "ANDROID_HOME=$sdkDir" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
        "ANDROID_SDK_ROOT=$sdkDir" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
    }

    $sdkDirForProps = $sdkDir -replace '\\', '/'
    "sdk.dir=$sdkDirForProps" | Set-Content -Path $localPropertiesPath -Encoding ascii
    Write-Host "android sdk dir: $sdkDir"
}

if (-not (Test-Path ".\test-app\gradlew.bat")) {
    throw "test-app\gradlew.bat not found"
}

$wrapperProperties = Join-Path $repoRoot "test-app\gradle\wrapper\gradle-wrapper.properties"
if (Test-Path $wrapperProperties) {
    Ensure-GradleDistributionZip -WrapperPropertiesPath $wrapperProperties
}
Ensure-AndroidLocalProperties -TestAppRoot (Join-Path $repoRoot "test-app")

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
