param(
    [Parameter(Mandatory = $true)]
    [string]$Serial,

    [string]$TargetName = "",

    [int]$StepTimeoutMinutes = 20,

    [string]$LogRoot = "internal/testlogs/install-smoke",

    [string]$OutputManifestPath = "",

    [string]$CargoInstallRoot = ""
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[Console]::InputEncoding = $utf8NoBom
[Console]::OutputEncoding = $utf8NoBom
$OutputEncoding = $utf8NoBom
$PSDefaultParameterValues["Out-File:Encoding"] = "utf8"
$PSDefaultParameterValues["Set-Content:Encoding"] = "utf8"
$PSDefaultParameterValues["Add-Content:Encoding"] = "utf8"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$runStartedAt = Get-Date
$runId = Get-Date -Format "yyyyMMdd-HHmmss"
$safeSerialForPath = ($Serial -replace '[\\/:*?"<>|]', '_')
$runLogDir = Join-Path $repoRoot (Join-Path $LogRoot "$runId-$safeSerialForPath")
New-Item -ItemType Directory -Force -Path $runLogDir | Out-Null

if ([string]::IsNullOrWhiteSpace($CargoInstallRoot)) {
    $CargoInstallRoot = Join-Path $runLogDir "cargo-root"
}
$CargoInstallRoot = [System.IO.Path]::GetFullPath($CargoInstallRoot)
$CargoBinDir = Join-Path $CargoInstallRoot "bin"
$InstalledCliPath = Join-Path $CargoBinDir "uiautomator.exe"

$summary = New-Object System.Collections.Generic.List[object]

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
        [int]$ExitCode = 0,
        [string]$StdoutPath = "",
        [string]$StderrPath = "",
        [string]$Command = ""
    )

    $summary.Add([PSCustomObject]@{
            Step            = $Step
            Status          = $Status
            Detail          = $Detail
            DurationSeconds = [Math]::Round($DurationSeconds, 3)
            ExitCode        = $ExitCode
            Command         = $Command
            StdoutPath      = $StdoutPath
            StderrPath      = $StderrPath
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

function Write-StructuredSummary {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RunStatus,
        [string]$FailureMessage = ""
    )

    $runFinishedAt = Get-Date
    $durationSeconds = [Math]::Round(($runFinishedAt - $runStartedAt).TotalSeconds, 3)
    $stepsArray = $summary.ToArray()
    if ($null -eq $stepsArray) {
        $stepsArray = @()
    }

    $jsonPath = Join-Path $runLogDir "summary.json"
    $junitPath = Join-Path $runLogDir "summary.junit.xml"

    $jsonObject = [ordered]@{
        schema_version    = 1
        run_id            = "$runId-$safeSerialForPath"
        target_name       = $TargetName
        serial            = $Serial
        status            = $RunStatus
        started_at        = $runStartedAt.ToString("o")
        finished_at       = $runFinishedAt.ToString("o")
        duration_seconds  = $durationSeconds
        log_dir           = $runLogDir
        install_root      = $CargoInstallRoot
        cli_path          = $InstalledCliPath
        failure_message   = $FailureMessage
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
    [void]$builder.AppendLine(('  <testsuite name="post-install-smoke" tests="{0}" failures="{1}" skipped="{2}" errors="0" time="{3}" timestamp="{4}">' -f $stepsArray.Count, $failedCount, $skippedCount, $durationSeconds, $runFinishedAt.ToString("o")))

    foreach ($item in $stepsArray) {
        $stepName = Escape-XmlText $item.Step
        $detailText = Escape-XmlText $item.Detail
        $timeValue = [Math]::Round([double]$item.DurationSeconds, 3)
        [void]$builder.AppendLine(('    <testcase classname="post-install-smoke" name="{0}" time="{1}">' -f $stepName, $timeValue))

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

function Write-RunManifest {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RunStatus,
        [Parameter(Mandatory = $true)]
        [object]$StructuredSummary,
        [string]$FailureMessage = ""
    )

    if ([string]::IsNullOrWhiteSpace($OutputManifestPath)) {
        return
    }

    $manifestDir = Split-Path -Parent $OutputManifestPath
    if ($manifestDir) {
        New-Item -ItemType Directory -Force -Path $manifestDir | Out-Null
    }

    $manifest = [ordered]@{
        schema_version  = 1
        run_id          = "$runId-$safeSerialForPath"
        target_name     = $TargetName
        serial          = $Serial
        status          = $RunStatus
        run_log_dir     = $runLogDir
        summary_json    = $StructuredSummary.JsonPath
        summary_junit   = $StructuredSummary.JunitPath
        install_root    = $CargoInstallRoot
        cli_path        = $InstalledCliPath
        failure_message = $FailureMessage
        generated_at    = (Get-Date).ToString("o")
    }

    $manifest | ConvertTo-Json -Depth 8 | Set-Content -Path $OutputManifestPath -Encoding utf8
}

function Invoke-Adb {
    param(
        [string[]]$CmdArgs,
        [switch]$AllowFailure
    )

    $prevErrorAction = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $output = & adb -s $Serial @CmdArgs 2>&1
        $code = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $prevErrorAction
    }

    $text = ($output | ForEach-Object { $_.ToString() }) -join "`n"

    if (-not $AllowFailure -and $code -ne 0) {
        throw "adb -s $Serial $($CmdArgs -join ' ') failed (exit=$code)`n$text"
    }

    return [PSCustomObject]@{
        ExitCode = $code
        Output   = $text
    }
}

function Start-StepProcess {
    param(
        [string]$Name,
        [string]$WorkingDirectory,
        [string]$Command,
        [int]$TimeoutMinutes
    )

    $safeName = ($Name -replace "[^A-Za-z0-9._-]", "_")
    $stdout = Join-Path $runLogDir "$safeName.log"
    $stderr = Join-Path $runLogDir "$safeName.err.log"
    $wrapper = '$ErrorActionPreference=''Stop''; $utf8 = New-Object System.Text.UTF8Encoding($false); [Console]::InputEncoding = $utf8; [Console]::OutputEncoding = $utf8; $OutputEncoding = $utf8; try { ' + $Command + '; $code = if ($null -eq $LASTEXITCODE) { 0 } else { [int]$LASTEXITCODE } } catch { Write-Error ($_ | Out-String); $code = 1 }; Write-Output "__CODEX_EXIT__=$code"; exit $code'

    Write-Host "[$Name] command: $Command"
    Write-Host "[$Name] log: $stdout"

    $started = Get-Date
    $proc = Start-Process `
        -FilePath "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe" `
        -ArgumentList @("-NoProfile", "-Command", $wrapper) `
        -WorkingDirectory $WorkingDirectory `
        -RedirectStandardOutput $stdout `
        -RedirectStandardError $stderr `
        -PassThru

    $deadline = $started.AddMinutes($TimeoutMinutes)

    while (-not $proc.HasExited) {
        Start-Sleep -Seconds 10
        if ((Get-Date) -gt $deadline) {
            Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
            throw "step '$Name' timed out after $TimeoutMinutes minutes. cmd: $Command. logs: $stdout"
        }
    }

    $null = $proc.WaitForExit()
    $proc.Refresh()

    $exitCode = $null
    $markerPattern = "__CODEX_EXIT__=(\d+)"
    for ($retry = 0; $retry -lt 20 -and $null -eq $exitCode; $retry++) {
        foreach ($path in @($stdout, $stderr)) {
            if (Test-Path $path) {
                $raw = Get-Content $path -Encoding utf8 -Raw -ErrorAction SilentlyContinue
                if ($raw) {
                    $all = [regex]::Matches($raw, $markerPattern)
                    if ($all.Count -gt 0) {
                        $exitCode = [int]$all[$all.Count - 1].Groups[1].Value
                        break
                    }
                }
            }
        }
        if ($null -eq $exitCode) {
            Start-Sleep -Milliseconds 300
        }
    }

    if ($null -eq $exitCode) {
        $exitCode = $proc.ExitCode
    }
    if ($null -eq $exitCode) {
        throw "step '$Name' finished but exit code is unavailable. cmd: $Command. logs: $stdout"
    }

    if ($exitCode -ne 0) {
        $tail = ""
        if (Test-Path $stderr) {
            $tail = (Get-Content $stderr -Encoding utf8 -Tail 40) -join "`n"
        }
        elseif (Test-Path $stdout) {
            $tail = (Get-Content $stdout -Encoding utf8 -Tail 40) -join "`n"
        }
        throw "step '$Name' failed (exit=$exitCode). cmd: $Command. logs: $stdout`n$tail"
    }

    $ended = Get-Date

    return [PSCustomObject]@{
        Name            = $Name
        Command         = $Command
        Stdout          = $stdout
        Stderr          = $stderr
        ExitCode        = $exitCode
        StartedAt       = $started.ToString("o")
        FinishedAt      = $ended.ToString("o")
        DurationSeconds = [Math]::Round(($ended - $started).TotalSeconds, 3)
    }
}

try {
    Write-Step "Validate target device"
    $state = Invoke-Adb -CmdArgs @("get-state")
    if ($state.Output.Trim() -ne "device") {
        throw "target device '$Serial' is not in 'device' state: $($state.Output.Trim())"
    }
    Add-Summary -Step "validate_device" -Status "ok" -Detail "state=device"

    Write-Step "Cleanup ATX/uiautomator state before smoke"
    Invoke-Adb -CmdArgs @("shell", "pidof", "atx-agent") -AllowFailure | Out-Null
    Invoke-Adb -CmdArgs @("shell", "rm", "-f", "/data/local/tmp/atx-agent", "/data/local/tmp/app-uiautomator.apk", "/data/local/tmp/app-uiautomator-test.apk", "/data/local/tmp/minicap", "/data/local/tmp/minitouch") -AllowFailure | Out-Null
    Invoke-Adb -CmdArgs @("shell", "am", "force-stop", "com.github.uiautomator") -AllowFailure | Out-Null
    Invoke-Adb -CmdArgs @("shell", "am", "force-stop", "com.github.uiautomator.test") -AllowFailure | Out-Null
    Invoke-Adb -CmdArgs @("shell", "pm", "uninstall", "com.github.uiautomator") -AllowFailure | Out-Null
    Invoke-Adb -CmdArgs @("shell", "pm", "uninstall", "com.github.uiautomator.test") -AllowFailure | Out-Null
    Invoke-Adb -CmdArgs @("forward", "--remove-all") -AllowFailure | Out-Null
    Add-Summary -Step "cleanup" -Status "ok" -Detail "cleanup completed"

    Write-Step "cargo install uiautomator-cli in isolated root"
    if (Test-Path $CargoInstallRoot) {
        Remove-Item -Recurse -Force $CargoInstallRoot -ErrorAction SilentlyContinue
    }
    New-Item -ItemType Directory -Force -Path $CargoInstallRoot | Out-Null

    $installCmd = "cargo install --path uiautomator-cli --locked --force --root '$CargoInstallRoot'"
    $installResult = Start-StepProcess -Name "cargo_install_cli" -WorkingDirectory $repoRoot -Command $installCmd -TimeoutMinutes $StepTimeoutMinutes
    Add-Summary -Step "cargo_install_cli" -Status "ok" -Detail "installed to $CargoInstallRoot" -DurationSeconds $installResult.DurationSeconds -ExitCode $installResult.ExitCode -StdoutPath $installResult.Stdout -StderrPath $installResult.Stderr -Command $installCmd

    if (-not (Test-Path $InstalledCliPath)) {
        throw "installed CLI not found: $InstalledCliPath"
    }

    $cliCommands = @(
        @{ Name = "cli_version"; Command = "& '$InstalledCliPath' version" },
        @{ Name = "cli_init_force"; Command = "& '$InstalledCliPath' init --serial '$Serial' --force" },
        @{ Name = "cli_status"; Command = "& '$InstalledCliPath' status --serial '$Serial'" },
        @{ Name = "cli_uninstall"; Command = "& '$InstalledCliPath' uninstall --serial '$Serial'" }
    )

    foreach ($step in $cliCommands) {
        $result = Start-StepProcess -Name $step.Name -WorkingDirectory $repoRoot -Command $step.Command -TimeoutMinutes $StepTimeoutMinutes
        Add-Summary -Step $step.Name -Status "ok" -Detail "passed" -DurationSeconds $result.DurationSeconds -ExitCode $result.ExitCode -StdoutPath $result.Stdout -StderrPath $result.Stderr -Command $step.Command
    }

    Write-Step "Completed"
    $structured = Write-StructuredSummary -RunStatus "passed"
    Write-RunManifest -RunStatus "passed" -StructuredSummary $structured
    $summary | Select-Object Step, Status, Detail | Format-Table -Wrap -AutoSize | Out-String | Write-Host
    Write-Host "summary json: $($structured.JsonPath)"
    Write-Host "summary junit: $($structured.JunitPath)"
    if (-not [string]::IsNullOrWhiteSpace($OutputManifestPath)) {
        Write-Host "manifest: $OutputManifestPath"
    }
    Write-Host "logs: $runLogDir"
}
catch {
    $failureMessage = $_.Exception.Message
    Add-Summary -Step "run" -Status "failed" -Detail $failureMessage
    $structured = Write-StructuredSummary -RunStatus "failed" -FailureMessage $failureMessage
    Write-RunManifest -RunStatus "failed" -StructuredSummary $structured -FailureMessage $failureMessage

    Write-Host ""
    Write-Host "FAILED"
    $summary | Select-Object Step, Status, Detail, Command, StdoutPath | Format-Table -Wrap -AutoSize | Out-String | Write-Host
    Write-Host "Minimal repro:"
    Write-Host "  1) cargo install --path uiautomator-cli --locked --force --root '$CargoInstallRoot'"
    Write-Host "  2) '$InstalledCliPath' init --serial '$Serial' --force"
    Write-Host "summary json: $($structured.JsonPath)"
    Write-Host "summary junit: $($structured.JunitPath)"
    if (-not [string]::IsNullOrWhiteSpace($OutputManifestPath)) {
        Write-Host "manifest: $OutputManifestPath"
    }
    Write-Host "logs: $runLogDir"
    throw
}
