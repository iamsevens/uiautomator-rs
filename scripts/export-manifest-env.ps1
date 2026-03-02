param(
    [Parameter(Mandatory = $true)]
    [string]$ManifestPath,

    [string]$Serial = "",

    [string]$Context = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Test-Path $ManifestPath)) {
    throw "manifest not found: $ManifestPath"
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
