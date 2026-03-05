param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("smoke", "full")]
    [string]$Mode,

    [Parameter(Mandatory = $true)]
    [string]$Serial,

    [string]$TargetName = "",

    [int]$StepTimeoutMinutes = 45,

    [int]$WaitTimeoutSeconds = 360,

    [int]$PollIntervalSeconds = 5,

    [string]$ExpectedAbi = "",

    [string]$ExpectedAndroidMajor = "",

    [string]$LdplayerStartCommand = "",

    [string]$MumuStartCommand = "",

    [string]$MumuConnectEndpoints = "",

    [string]$LogRoot = "internal/testlogs/validation-gate",

    [string]$OutputManifestPath = "",

    [switch]$SkipRustCheck,

    [switch]$SkipEnsureDevice,

    [switch]$SkipBuildTestApp,

    [switch]$SkipExecution
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[Console]::InputEncoding = $utf8NoBom
[Console]::OutputEncoding = $utf8NoBom
$OutputEncoding = $utf8NoBom
$PSDefaultParameterValues["Out-File:Encoding"] = "utf8"
$PSDefaultParameterValues["Set-Content:Encoding"] = "utf8"
$PSDefaultParameterValues["Add-Content:Encoding"] = "utf8"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

if ($StepTimeoutMinutes -lt 1) {
    throw "StepTimeoutMinutes must be >= 1"
}
if ($WaitTimeoutSeconds -lt 1) {
    throw "WaitTimeoutSeconds must be >= 1"
}
if ($PollIntervalSeconds -lt 1) {
    throw "PollIntervalSeconds must be >= 1"
}

$runStartedAt = Get-Date
$runId = Get-Date -Format "yyyyMMdd-HHmmss"
$safeSerialForPath = ($Serial -replace '[\\/:*?"<>|]', '_')
$targetNameEffective = if ([string]::IsNullOrWhiteSpace($TargetName)) { "$Mode-$safeSerialForPath" } else { $TargetName }
$runLogDir = Join-Path $repoRoot (Join-Path $LogRoot "$runId-$safeSerialForPath-$Mode")
New-Item -ItemType Directory -Force -Path $runLogDir | Out-Null

if ([string]::IsNullOrWhiteSpace($OutputManifestPath)) {
    $OutputManifestPath = Join-Path $runLogDir "gate-manifest.json"
}

$childManifestPath = Join-Path $runLogDir "child-manifest.json"
$launcherStatePath = Join-Path $runLogDir "launcher-state.json"
$summary = New-Object System.Collections.Generic.List[object]
$script:LastFailedStep = ""
$PowerShellExe = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"

function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "==> $Message"
}

function Add-Summary {
    param(
        [string]$Step,
        [string]$Status,
        [string]$Detail = "",
        [double]$DurationSeconds = 0,
        [string]$Command = ""
    )

    $summary.Add([PSCustomObject]@{
            Step            = $Step
            Status          = $Status
            Detail          = $Detail
            DurationSeconds = [Math]::Round($DurationSeconds, 3)
            Command         = $Command
            Timestamp       = (Get-Date).ToString("o")
    })
}

function Escape-XmlText {
    param([string]$Text)
    if ($null -eq $Text) {
        return ""
    }
    return [System.Security.SecurityElement]::Escape($Text)
}

function Invoke-GateStep {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Step,
        [string]$Command = "",
        [switch]$Skip,
        [string]$SkipReason = "",
        [Parameter(Mandatory = $true)]
        [scriptblock]$Action
    )

    if ($Skip) {
        $reason = if ([string]::IsNullOrWhiteSpace($SkipReason)) { "skipped" } else { $SkipReason }
        Add-Summary -Step $Step -Status "skipped" -Detail $reason -Command $Command
        return
    }

    $started = Get-Date
    try {
        & $Action
        $duration = [Math]::Round(((Get-Date) - $started).TotalSeconds, 3)
        Add-Summary -Step $Step -Status "ok" -Detail "passed" -DurationSeconds $duration -Command $Command
    }
    catch {
        $duration = [Math]::Round(((Get-Date) - $started).TotalSeconds, 3)
        $detail = $_.Exception.Message
        Add-Summary -Step $Step -Status "failed" -Detail $detail -DurationSeconds $duration -Command $Command
        $script:LastFailedStep = $Step
        throw
    }
}

function Resolve-FailureCode {
    param(
        [string]$FailedStep,
        [string]$Message
    )

    $msg = if ($null -eq $Message) { "" } else { $Message.ToLowerInvariant() }
    switch ($FailedStep) {
        "preflight_powershell" { return "env_missing_powershell" }
        "preflight_adb" { return "env_missing_adb" }
        "preflight_rust_toolchain" { return "env_missing_rust_toolchain" }
        "ensure_test_device" { return "adb_device_unavailable" }
        "build_test_app" { return "test_app_build_failed" }
    }

    $explicitAdbSignals = @(
        "device offline",
        "device unauthorized",
        "target serial did not become online",
        "target device",
        "cannot connect to",
        "adb version failed",
        "adb start-server failed",
        "no devices/emulators found"
    )
    foreach ($signal in $explicitAdbSignals) {
        if ($msg -like "*$signal*") {
            return "adb_device_unavailable"
        }
    }

    if ($msg -match '\badb\b.+failed') {
        return "adb_device_unavailable"
    }

    if ($FailedStep -in @("run_full", "run_smoke", "validate_child_manifest")) {
        return "test_or_runtime_failure"
    }

    if ($msg -match "timed out|timeout") {
        return "step_timeout"
    }
    if ($msg -match "manifest|summary|convertfrom-json") {
        return "manifest_or_summary_invalid"
    }
    return "test_or_runtime_failure"
}

function Write-GateStructuredSummary {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RunStatus,
        [string]$FailureMessage = "",
        [string]$FailureCode = ""
    )

    $runFinishedAt = Get-Date
    $durationSeconds = [Math]::Round(($runFinishedAt - $runStartedAt).TotalSeconds, 3)
    $stepsArray = $summary.ToArray()
    if ($null -eq $stepsArray) {
        $stepsArray = @()
    }

    $jsonPath = Join-Path $runLogDir "gate-summary.json"
    $junitPath = Join-Path $runLogDir "gate-summary.junit.xml"

    $jsonObject = [ordered]@{
        schema_version    = 1
        run_id            = "$runId-$safeSerialForPath-$Mode"
        mode              = $Mode
        target_name       = $targetNameEffective
        serial            = $Serial
        status            = $RunStatus
        failure_code      = $FailureCode
        failure_message   = $FailureMessage
        started_at        = $runStartedAt.ToString("o")
        finished_at       = $runFinishedAt.ToString("o")
        duration_seconds  = $durationSeconds
        log_dir           = $runLogDir
        total_steps       = $stepsArray.Count
        failed_steps      = @($stepsArray | Where-Object { $_.Status -eq "failed" }).Count
        skipped_steps     = @($stepsArray | Where-Object { $_.Status -eq "skipped" }).Count
        successful_steps  = @($stepsArray | Where-Object { $_.Status -eq "ok" }).Count
        steps             = $stepsArray
    }

    $jsonObject | ConvertTo-Json -Depth 8 | Set-Content -Path $jsonPath -Encoding utf8

    $failedCount = @($stepsArray | Where-Object { $_.Status -eq "failed" }).Count
    $skippedCount = @($stepsArray | Where-Object { $_.Status -eq "skipped" }).Count
    $builder = New-Object System.Text.StringBuilder
    [void]$builder.AppendLine('<?xml version="1.0" encoding="UTF-8"?>')
    [void]$builder.AppendLine('<testsuites>')
    [void]$builder.AppendLine(('  <testsuite name="validation-gate" tests="{0}" failures="{1}" skipped="{2}" errors="0" time="{3}" timestamp="{4}">' -f $stepsArray.Count, $failedCount, $skippedCount, $durationSeconds, $runFinishedAt.ToString("o")))

    foreach ($item in $stepsArray) {
        $stepName = Escape-XmlText $item.Step
        $detailText = Escape-XmlText $item.Detail
        $timeValue = [Math]::Round([double]$item.DurationSeconds, 3)
        [void]$builder.AppendLine(('    <testcase classname="validation-gate" name="{0}" time="{1}">' -f $stepName, $timeValue))

        if ($item.Status -eq "failed") {
            [void]$builder.AppendLine(('      <failure message="{0}">{1}</failure>' -f $detailText, $detailText))
        }
        elseif ($item.Status -eq "skipped") {
            [void]$builder.AppendLine(('      <skipped message="{0}" />' -f $detailText))
        }
        else {
            [void]$builder.AppendLine(('      <system-out>{0}</system-out>' -f $detailText))
        }

        [void]$builder.AppendLine('    </testcase>')
    }

    [void]$builder.AppendLine('  </testsuite>')
    [void]$builder.AppendLine('</testsuites>')
    $builder.ToString() | Set-Content -Path $junitPath -Encoding utf8

    return [PSCustomObject]@{
        JsonPath  = $jsonPath
        JunitPath = $junitPath
    }
}

function Write-GateManifest {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RunStatus,
        [Parameter(Mandatory = $true)]
        [object]$StructuredSummary,
        [string]$FailureMessage = "",
        [string]$FailureCode = "",
        $ChildManifest = $null
    )

    $manifestDir = Split-Path -Parent $OutputManifestPath
    if ($manifestDir) {
        New-Item -ItemType Directory -Force -Path $manifestDir | Out-Null
    }

    $effectiveRunLogDir = $runLogDir
    $effectiveSummaryJson = $StructuredSummary.JsonPath
    $effectiveSummaryJunit = $StructuredSummary.JunitPath
    $childStatus = ""
    $childFailureMessage = ""

    if ($null -ne $ChildManifest) {
        if ($ChildManifest.PSObject.Properties.Name -contains "run_log_dir" -and -not [string]::IsNullOrWhiteSpace([string]$ChildManifest.run_log_dir)) {
            $effectiveRunLogDir = [string]$ChildManifest.run_log_dir
        }
        if ($ChildManifest.PSObject.Properties.Name -contains "summary_json" -and -not [string]::IsNullOrWhiteSpace([string]$ChildManifest.summary_json)) {
            $effectiveSummaryJson = [string]$ChildManifest.summary_json
        }
        if ($ChildManifest.PSObject.Properties.Name -contains "summary_junit" -and -not [string]::IsNullOrWhiteSpace([string]$ChildManifest.summary_junit)) {
            $effectiveSummaryJunit = [string]$ChildManifest.summary_junit
        }
        if ($ChildManifest.PSObject.Properties.Name -contains "status") {
            $childStatus = [string]$ChildManifest.status
        }
        if ($ChildManifest.PSObject.Properties.Name -contains "failure_message") {
            $childFailureMessage = [string]$ChildManifest.failure_message
        }
    }

    $manifest = [ordered]@{
        schema_version      = 1
        run_id              = "$runId-$safeSerialForPath-$Mode"
        mode                = $Mode
        target_name         = $targetNameEffective
        serial              = $Serial
        status              = $RunStatus
        failure_code        = $FailureCode
        failure_message     = $FailureMessage
        run_log_dir         = $effectiveRunLogDir
        summary_json        = $effectiveSummaryJson
        summary_junit       = $effectiveSummaryJunit
        gate_log_dir        = $runLogDir
        gate_summary_json   = $StructuredSummary.JsonPath
        gate_summary_junit  = $StructuredSummary.JunitPath
        child_manifest_path = $childManifestPath
        child_status        = $childStatus
        child_failure_msg   = $childFailureMessage
        generated_at        = (Get-Date).ToString("o")
    }

    $manifest | ConvertTo-Json -Depth 8 | Set-Content -Path $OutputManifestPath -Encoding utf8
}

function Stop-StartedLaunchers {
    param([string]$StatePath)

    if ([string]::IsNullOrWhiteSpace($StatePath) -or -not (Test-Path $StatePath)) {
        return
    }

    try {
        $state = Get-Content -Path $StatePath -Raw -Encoding utf8 | ConvertFrom-Json
        if ($null -eq $state -or $state.PSObject.Properties.Name -notcontains "launchers") {
            return
        }

        foreach ($launcher in @($state.launchers)) {
            $pidText = if ($launcher.PSObject.Properties.Name -contains "Pid") { [string]$launcher.Pid } else { "" }
            if ([string]::IsNullOrWhiteSpace($pidText)) {
                continue
            }

            $launcherPid = 0
            if (-not [int]::TryParse($pidText, [ref]$launcherPid) -or $launcherPid -le 0) {
                continue
            }

            $name = if ($launcher.PSObject.Properties.Name -contains "Name") { [string]$launcher.Name } else { "launcher" }
            $proc = Get-Process -Id $launcherPid -ErrorAction SilentlyContinue
            if ($null -eq $proc) {
                Write-Host "[$name] pid=$launcherPid already exited"
                continue
            }

            Write-Host "[$name] stopping pid=$launcherPid"
            Stop-Process -Id $launcherPid -Force -ErrorAction SilentlyContinue
            Start-Sleep -Milliseconds 500
            if ($null -ne (Get-Process -Id $launcherPid -ErrorAction SilentlyContinue)) {
                Write-Host "::warning::[$name] pid=$launcherPid still running after stop request"
            }
        }
    }
    catch {
        Write-Host "::warning::failed to stop started launchers from '$StatePath': $($_.Exception.Message)"
    }
}

$childManifest = $null
$runStatus = "passed"
$failureMessage = ""
$failureCode = ""

try {
    Invoke-GateStep -Step "preflight_powershell" -Command "Test-Path $PowerShellExe" -Action {
        if (-not (Test-Path $PowerShellExe)) {
            throw "Windows PowerShell not found at '$PowerShellExe'"
        }
        $versionOut = & $PowerShellExe -NoProfile -ExecutionPolicy Bypass -Command '$PSVersionTable.PSVersion' 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "failed to invoke Windows PowerShell. exit=$LASTEXITCODE output=$($versionOut -join ' ')"
        }
        Write-Host ($versionOut | Out-String)
    }

    Invoke-GateStep -Step "preflight_adb" -Command "adb version && adb start-server" -Action {
        $adbCmd = Get-Command adb -ErrorAction SilentlyContinue
        if ($null -eq $adbCmd) {
            throw "adb not found in PATH. Set ANDROID_HOME/ANDROID_SDK_ROOT and include platform-tools."
        }

        $adbVersion = & adb version 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "adb version failed. exit=$LASTEXITCODE output=$($adbVersion -join ' ')"
        }
        Write-Host ($adbVersion | Out-String)

        $adbStart = & adb start-server 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "adb start-server failed. exit=$LASTEXITCODE output=$($adbStart -join ' ')"
        }
    }

    Invoke-GateStep -Step "preflight_rust_toolchain" -Command ".\\scripts\\check-rust-toolchain.ps1" -Skip:$SkipRustCheck -SkipReason "SkipRustCheck" -Action {
        & (Join-Path $PSScriptRoot "check-rust-toolchain.ps1")
    }

    Invoke-GateStep -Step "ensure_test_device" -Command ".\\scripts\\ensure-test-device.ps1 -Serial $Serial" -Skip:$SkipEnsureDevice -SkipReason "SkipEnsureDevice" -Action {
        $ensureArgs = @{
            Serial             = $Serial
            WaitTimeoutSeconds = $WaitTimeoutSeconds
            PollIntervalSeconds = $PollIntervalSeconds
            LauncherStatePath  = $launcherStatePath
        }
        if (-not [string]::IsNullOrWhiteSpace($LdplayerStartCommand)) {
            $ensureArgs["LdplayerStartCommand"] = $LdplayerStartCommand
        }
        if (-not [string]::IsNullOrWhiteSpace($MumuStartCommand)) {
            $ensureArgs["MumuStartCommand"] = $MumuStartCommand
        }
        if (-not [string]::IsNullOrWhiteSpace($MumuConnectEndpoints)) {
            $ensureArgs["MumuConnectEndpoints"] = $MumuConnectEndpoints
        }
        & (Join-Path $PSScriptRoot "ensure-test-device.ps1") @ensureArgs
    }

    $skipBuildByMode = $Mode -ne "full"
    $skipBuild = $SkipBuildTestApp -or $skipBuildByMode
    $skipBuildReason = if ($skipBuildByMode) { "mode=smoke" } elseif ($SkipBuildTestApp) { "SkipBuildTestApp" } else { "" }
    Invoke-GateStep -Step "build_test_app" -Command ".\\scripts\\build-test-app.ps1" -Skip:$skipBuild -SkipReason $skipBuildReason -Action {
        & (Join-Path $PSScriptRoot "build-test-app.ps1")
    }

    Invoke-GateStep -Step "run_$Mode" -Command "child script for mode=$Mode serial=$Serial" -Skip:$SkipExecution -SkipReason "SkipExecution" -Action {
        if (Test-Path $childManifestPath) {
            Remove-Item -Path $childManifestPath -Force -ErrorAction SilentlyContinue
        }

        if ($Mode -eq "smoke") {
            & (Join-Path $PSScriptRoot "post-install-smoke.ps1") `
                -Serial $Serial `
                -TargetName $targetNameEffective `
                -StepTimeoutMinutes $StepTimeoutMinutes `
                -OutputManifestPath $childManifestPath
        }
        else {
            $runArgs = @{
                Serial             = $Serial
                TargetName         = $targetNameEffective
                StepTimeoutMinutes = $StepTimeoutMinutes
                OutputManifestPath = $childManifestPath
            }
            if (-not [string]::IsNullOrWhiteSpace($ExpectedAbi)) {
                $runArgs["ExpectedAbi"] = $ExpectedAbi
            }
            if (-not [string]::IsNullOrWhiteSpace($ExpectedAndroidMajor)) {
                $runArgs["ExpectedAndroidMajor"] = $ExpectedAndroidMajor
            }
            & (Join-Path $PSScriptRoot "run-device-full-regression.ps1") @runArgs
        }
    }

    if (-not $SkipExecution) {
        Invoke-GateStep -Step "validate_child_manifest" -Command "ConvertFrom-Json child manifest" -Action {
            if (-not (Test-Path $childManifestPath)) {
                throw "child manifest not found: $childManifestPath"
            }
            $childManifest = Get-Content -Path $childManifestPath -Raw -Encoding utf8 | ConvertFrom-Json
            if ($null -eq $childManifest) {
                throw "child manifest parse returned null: $childManifestPath"
            }
            if ($childManifest.PSObject.Properties.Name -notcontains "status") {
                throw "child manifest missing status field: $childManifestPath"
            }
            if ([string]$childManifest.status -ne "passed") {
                $childFailure = if ($childManifest.PSObject.Properties.Name -contains "failure_message") { [string]$childManifest.failure_message } else { "unknown child failure" }
                throw "child validation status=$($childManifest.status). $childFailure"
            }
        }
    }
    else {
        Add-Summary -Step "validate_child_manifest" -Status "skipped" -Detail "SkipExecution"
    }
}
catch {
    $runStatus = "failed"
    $failureMessage = $_.Exception.Message
    $failureCode = Resolve-FailureCode -FailedStep $script:LastFailedStep -Message $failureMessage
}

if ($null -eq $childManifest -and (Test-Path $childManifestPath)) {
    try {
        $childManifest = Get-Content -Path $childManifestPath -Raw -Encoding utf8 | ConvertFrom-Json
    }
    catch {
        # keep gate-level failure details
    }
}

if ($runStatus -eq "passed" -and $null -ne $childManifest -and [string]$childManifest.status -ne "passed") {
    $runStatus = "failed"
    $failureMessage = if ($childManifest.PSObject.Properties.Name -contains "failure_message") { [string]$childManifest.failure_message } else { "child status is not passed" }
    $failureCode = "test_or_runtime_failure"
}

$structured = Write-GateStructuredSummary -RunStatus $runStatus -FailureMessage $failureMessage -FailureCode $failureCode
Write-GateManifest -RunStatus $runStatus -StructuredSummary $structured -FailureMessage $failureMessage -FailureCode $failureCode -ChildManifest $childManifest
Stop-StartedLaunchers -StatePath $launcherStatePath

Write-Host ""
Write-Host "Validation gate status: $runStatus"
if (-not [string]::IsNullOrWhiteSpace($failureCode)) {
    Write-Host "Failure code: $failureCode"
}
if (-not [string]::IsNullOrWhiteSpace($failureMessage)) {
    Write-Host "Failure message: $failureMessage"
}
Write-Host "gate summary json: $($structured.JsonPath)"
Write-Host "gate summary junit: $($structured.JunitPath)"
Write-Host "gate manifest: $OutputManifestPath"
Write-Host "gate logs: $runLogDir"

if ($runStatus -ne "passed") {
    throw "validation gate failed ($failureCode): $failureMessage"
}
