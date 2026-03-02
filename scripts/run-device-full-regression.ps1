param(
    [Parameter(Mandatory = $true)]
    [string]$Serial,

    [Parameter(Mandatory = $true)]
    [string]$TargetName,

    [int]$StepTimeoutMinutes = 45,

    [Parameter(Mandatory = $true)]
    [string]$OutputManifestPath,

    [string]$ExpectedAbi = "",

    [string]$ExpectedAndroidMajor = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptPath = Join-Path $PSScriptRoot "device-full-test.ps1"
if (-not (Test-Path $scriptPath)) {
    throw "device-full-test.ps1 not found: $scriptPath"
}

$args = @{
    Serial             = $Serial
    TargetName         = $TargetName
    StepTimeoutMinutes = $StepTimeoutMinutes
    OutputManifestPath = $OutputManifestPath
}

if (-not [string]::IsNullOrWhiteSpace($ExpectedAbi)) {
    $args["ExpectedAbi"] = $ExpectedAbi
}

if (-not [string]::IsNullOrWhiteSpace($ExpectedAndroidMajor)) {
    $major = 0
    if (-not [int]::TryParse($ExpectedAndroidMajor, [ref]$major)) {
        throw "ExpectedAndroidMajor must be an integer: '$ExpectedAndroidMajor'"
    }
    if ($major -gt 0) {
        $args["ExpectedAndroidMajor"] = $major
    }
}

& $scriptPath @args
