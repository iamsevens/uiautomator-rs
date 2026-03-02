param(
    [string]$ManifestPath = "",

    [string]$Serial = "",

    [string]$Context = "",

    [switch]$AllowMissingManifest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($ManifestPath) -or -not (Test-Path $ManifestPath)) {
    if (-not $AllowMissingManifest) {
        throw "manifest not found: $ManifestPath"
    }

    if ([string]::IsNullOrWhiteSpace($env:GITHUB_ENV)) {
        throw "GITHUB_ENV is not set"
    }

    $safeSerial = if ([string]::IsNullOrWhiteSpace($Serial)) {
        "unknown"
    }
    else {
        $Serial -replace '[:\\\/<>"\|\*\?]', '_'
    }

    "RUN_STATUS=failed" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
    "RUN_LOG_DIR=" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
    "RUN_SUMMARY_JSON=" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
    "RUN_SUMMARY_JUNIT=" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
    "SAFE_SERIAL=$safeSerial" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8

    if (-not [string]::IsNullOrWhiteSpace($Context)) {
        Write-Host "$Context status=failed (manifest missing)"
    }
    else {
        Write-Host "status=failed (manifest missing)"
    }
    return
}

$manifest = Get-Content $ManifestPath -Raw -Encoding utf8 | ConvertFrom-Json

if ([string]::IsNullOrWhiteSpace($Serial)) {
    if ($manifest.PSObject.Properties.Name -contains "serial") {
        $Serial = [string]$manifest.serial
    }
}

$safeSerial = if ([string]::IsNullOrWhiteSpace($Serial)) {
    "unknown"
}
else {
    $Serial -replace '[:\\\/<>"\|\*\?]', '_'
}

if ([string]::IsNullOrWhiteSpace($env:GITHUB_ENV)) {
    throw "GITHUB_ENV is not set"
}

"RUN_STATUS=$($manifest.status)" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
"RUN_LOG_DIR=$($manifest.run_log_dir)" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
"RUN_SUMMARY_JSON=$($manifest.summary_json)" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
"RUN_SUMMARY_JUNIT=$($manifest.summary_junit)" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
"SAFE_SERIAL=$safeSerial" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8

if (-not [string]::IsNullOrWhiteSpace($Context)) {
    Write-Host "$Context status=$($manifest.status)"
}
else {
    Write-Host "status=$($manifest.status)"
}
Write-Host "log_dir=$($manifest.run_log_dir)"
