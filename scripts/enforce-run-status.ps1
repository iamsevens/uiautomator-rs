param(
    [string]$RunStatus = "",

    [string]$FailureCode = "",

    [string]$FailureMessage = "",

    [string]$AllowFailureCodes = "",

    [string]$Context = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RunStatus)) {
    $RunStatus = [string]$env:RUN_STATUS
}
if ([string]::IsNullOrWhiteSpace($FailureCode)) {
    $FailureCode = [string]$env:RUN_FAILURE_CODE
}
if ([string]::IsNullOrWhiteSpace($FailureMessage)) {
    $FailureMessage = [string]$env:RUN_FAILURE_MESSAGE
}

$allowed = @()
if (-not [string]::IsNullOrWhiteSpace($AllowFailureCodes)) {
    $allowed = @(
        $AllowFailureCodes.Split(",") |
            ForEach-Object { $_.Trim() } |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
            Select-Object -Unique
    )
}

$prefix = if ([string]::IsNullOrWhiteSpace($Context)) { "run status check" } else { $Context }

if ($RunStatus -eq "passed") {
    Write-Host "$prefix status=passed"
    exit 0
}

if ($allowed.Count -gt 0 -and $allowed -contains $FailureCode) {
    Write-Host "::warning::$prefix status=$RunStatus failure_code=$FailureCode failure_message=$FailureMessage"
    exit 0
}

throw "$prefix status=$RunStatus failure_code=$FailureCode failure_message=$FailureMessage"
