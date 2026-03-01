param(
    [Parameter(Mandatory = $true)]
    [string]$Serial,

    [string]$TargetName = "",

    [string]$ExpectedAbi = "",

    [int]$ExpectedAndroidMajor = 0,

    [int]$StepTimeoutMinutes = 45,

    [string]$LogRoot = "internal/testlogs/full-device",

    [string]$TestAppApk = "test-app/app/build/outputs/apk/debug/app-debug.apk",

    [string]$OutputManifestPath = "",

    [switch]$SkipCleanup,

    [switch]$SkipInit,

    [switch]$SkipTestAppInstall
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
$debugDumpDir = Join-Path $runLogDir "debug-dumps"
New-Item -ItemType Directory -Force -Path $runLogDir | Out-Null

$summary = New-Object System.Collections.Generic.List[object]
$deviceProfile = [ordered]@{
    abi             = ""
    android_release = ""
    android_major   = 0
    sdk_int         = ""
}

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
        [string]$StderrPath = ""
    )
    $summary.Add([PSCustomObject]@{
            Step            = $Step
            Status          = $Status
            Detail          = $Detail
            DurationSeconds = [Math]::Round($DurationSeconds, 3)
            ExitCode        = $ExitCode
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
        device_profile    = $deviceProfile
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
    [void]$builder.AppendLine(('  <testsuite name="device-full-test" tests="{0}" failures="{1}" skipped="{2}" errors="0" time="{3}" timestamp="{4}">' -f $stepsArray.Count, $failedCount, $skippedCount, $durationSeconds, $runFinishedAt.ToString("o")))
    [void]$builder.AppendLine('    <properties>')
    [void]$builder.AppendLine(('      <property name="serial" value="{0}" />' -f (Escape-XmlText $Serial)))
    [void]$builder.AppendLine(('      <property name="run_id" value="{0}" />' -f (Escape-XmlText "$runId-$safeSerialForPath")))
    [void]$builder.AppendLine(('      <property name="run_status" value="{0}" />' -f (Escape-XmlText $RunStatus)))
    [void]$builder.AppendLine(('      <property name="log_dir" value="{0}" />' -f (Escape-XmlText $runLogDir)))
    [void]$builder.AppendLine('    </properties>')

    foreach ($item in $stepsArray) {
        $stepName = Escape-XmlText $item.Step
        $detailText = Escape-XmlText $item.Detail
        $timeValue = [Math]::Round([double]$item.DurationSeconds, 3)
        [void]$builder.AppendLine(('    <testcase classname="device-full-test" name="{0}" time="{1}">' -f $stepName, $timeValue))

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
        schema_version   = 1
        run_id           = "$runId-$safeSerialForPath"
        target_name      = $TargetName
        serial           = $Serial
        status           = $RunStatus
        run_log_dir      = $runLogDir
        summary_json     = $StructuredSummary.JsonPath
        summary_junit    = $StructuredSummary.JunitPath
        device_profile   = $deviceProfile
        failure_message  = $FailureMessage
        generated_at     = (Get-Date).ToString("o")
    }

    $manifest | ConvertTo-Json -Depth 8 | Set-Content -Path $OutputManifestPath -Encoding utf8
}

function Move-RootDebugXmlArtifacts {
    param(
        [string]$DestinationDir
    )

    $patterns = @(
        "window_dump*.xml",
        "atx_dump*.xml",
        "__hierarchy*.xml"
    )

    $moved = New-Object System.Collections.Generic.List[string]

    foreach ($pattern in $patterns) {
        $files = Get-ChildItem -Path $repoRoot -Filter $pattern -File -ErrorAction SilentlyContinue
        foreach ($file in $files) {
            if (-not (Test-Path $DestinationDir)) {
                New-Item -ItemType Directory -Force -Path $DestinationDir | Out-Null
            }

            $targetPath = Join-Path $DestinationDir $file.Name
            if (Test-Path $targetPath) {
                $baseName = [System.IO.Path]::GetFileNameWithoutExtension($file.Name)
                $extension = [System.IO.Path]::GetExtension($file.Name)
                $targetPath = Join-Path $DestinationDir ("{0}-{1}{2}" -f $baseName, (Get-Date -Format "yyyyMMdd-HHmmssfff"), $extension)
            }

            Move-Item -Path $file.FullName -Destination $targetPath -Force
            $moved.Add((Split-Path -Leaf $targetPath))
        }
    }

    return ,$moved
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
        Start-Sleep -Seconds 20
        $elapsed = ((Get-Date) - $started).TotalMinutes
        Write-Host ("[{0}] running {1:N1}m" -f $Name, $elapsed)

        if (Test-Path $stdout) {
            Get-Content $stdout -Encoding utf8 -Tail 6 | ForEach-Object { Write-Host "  $_" }
        }
        if (Test-Path $stderr) {
            Get-Content $stderr -Encoding utf8 -Tail 6 | ForEach-Object { Write-Host "  [err] $_" }
        }

        if ((Get-Date) -gt $deadline) {
            Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
            throw "step '$Name' timed out after $TimeoutMinutes minutes. Logs: $stdout"
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
        throw "step '$Name' finished but exit code is unavailable. Logs: $stdout"
    }

    if ($exitCode -ne 0) {
        $tail = ""
        if (Test-Path $stderr) {
            $tail = (Get-Content $stderr -Encoding utf8 -Tail 40) -join "`n"
        }
        elseif (Test-Path $stdout) {
            $tail = (Get-Content $stdout -Encoding utf8 -Tail 40) -join "`n"
        }
        throw "step '$Name' failed (exit=$exitCode). Logs: $stdout`n$tail"
    }

    $ended = Get-Date

    return [PSCustomObject]@{
        Name            = $Name
        Stdout          = $stdout
        Stderr          = $stderr
        ExitCode        = $exitCode
        StartedAt       = $started.ToString("o")
        FinishedAt      = $ended.ToString("o")
        DurationSeconds = [Math]::Round(($ended - $started).TotalSeconds, 3)
    }
}

function Get-InstalledPackageApkPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$PackageName
    )

    $pkg = Invoke-Adb -CmdArgs @("shell", "pm", "path", $PackageName) -AllowFailure
    if ($pkg.ExitCode -ne 0) {
        return $null
    }

    $paths = New-Object System.Collections.Generic.List[string]
    foreach ($line in ($pkg.Output -split "`r?`n")) {
        $trimmed = $line.Trim()
        if ($trimmed -match '^package:(?<path>.+)$') {
            $paths.Add($matches.path.Trim())
        }
    }

    if ($paths.Count -eq 0) {
        return $null
    }

    $baseApk = $paths | Where-Object { $_ -like "*/base.apk" } | Select-Object -First 1
    if ($baseApk) {
        return $baseApk
    }

    return $paths[0]
}

function Get-RemoteFileHash {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RemotePath
    )

    $candidates = @(
        @("shell", "toybox", "sha256sum", $RemotePath),
        @("shell", "sha256sum", $RemotePath),
        @("shell", "toybox", "md5sum", $RemotePath),
        @("shell", "md5sum", $RemotePath)
    )

    foreach ($candidate in $candidates) {
        $result = Invoke-Adb -CmdArgs $candidate -AllowFailure
        if ($result.ExitCode -ne 0) {
            continue
        }

        foreach ($line in ($result.Output -split "`r?`n")) {
            $trimmed = $line.Trim()
            if ($trimmed -match '^(?<hash>[0-9a-fA-F]{32}|[0-9a-fA-F]{64})\s+\*?.+$') {
                $hashText = $matches.hash.ToLowerInvariant()
                return [PSCustomObject]@{
                    Hash      = $hashText
                    Algorithm = if ($hashText.Length -eq 64) { "SHA256" } else { "MD5" }
                }
            }
        }
    }

    return $null
}

try {
    $preRunArtifacts = Move-RootDebugXmlArtifacts -DestinationDir $debugDumpDir
    if ($preRunArtifacts.Count -gt 0) {
        Write-Host ("Moved {0} pre-existing debug xml file(s) to {1}" -f $preRunArtifacts.Count, $debugDumpDir)
        Add-Summary -Step "collect_debug_xml_pre" -Status "ok" -Detail ("moved {0} file(s) to debug-dumps" -f $preRunArtifacts.Count)
    }
    else {
        Add-Summary -Step "collect_debug_xml_pre" -Status "ok" -Detail "no debug xml files found in repo root"
    }

    Write-Step "Validate target device"
    $state = Invoke-Adb -CmdArgs @("get-state")
    if ($state.Output.Trim() -ne "device") {
        throw "target device '$Serial' is not in 'device' state: $($state.Output.Trim())"
    }
    Add-Summary -Step "validate_device" -Status "ok" -Detail "state=device"

    $abi = (Invoke-Adb -CmdArgs @("shell", "getprop", "ro.product.cpu.abi")).Output.Trim()
    $androidRelease = (Invoke-Adb -CmdArgs @("shell", "getprop", "ro.build.version.release")).Output.Trim()
    $sdkInt = (Invoke-Adb -CmdArgs @("shell", "getprop", "ro.build.version.sdk")).Output.Trim()
    $androidMajor = 0
    if ($androidRelease -match '^(?<major>\d+)') {
        $androidMajor = [int]$matches.major
    }

    $deviceProfile.abi = $abi
    $deviceProfile.android_release = $androidRelease
    $deviceProfile.android_major = $androidMajor
    $deviceProfile.sdk_int = $sdkInt

    if (-not [string]::IsNullOrWhiteSpace($ExpectedAbi) -and ($abi -ne $ExpectedAbi)) {
        throw "target device '$Serial' abi mismatch: expected '$ExpectedAbi', actual '$abi'"
    }

    if ($ExpectedAndroidMajor -gt 0 -and ($androidMajor -ne $ExpectedAndroidMajor)) {
        throw "target device '$Serial' Android major mismatch: expected '$ExpectedAndroidMajor', actual '$androidMajor' (release=$androidRelease)"
    }

    Add-Summary -Step "validate_device_profile" -Status "ok" -Detail "abi=$abi android=$androidRelease major=$androidMajor sdk=$sdkInt"

    if (-not $SkipCleanup) {
        Write-Step "Cleanup ATX/uiautomator state on target"

        try {
            $cleanupCliResult = Start-StepProcess `
                -Name "cleanup_cli_uninstall" `
                -WorkingDirectory (Join-Path $repoRoot "uiautomator-cli") `
                -Command "cargo run -- uninstall -s $Serial" `
                -TimeoutMinutes 10
            Add-Summary -Step "cleanup_cli_uninstall" -Status "ok" -Detail "cargo run -- uninstall -s $Serial" -DurationSeconds $cleanupCliResult.DurationSeconds -ExitCode $cleanupCliResult.ExitCode -StdoutPath $cleanupCliResult.Stdout -StderrPath $cleanupCliResult.Stderr
        }
        catch {
            Write-Host "cleanup uninstall returned non-zero, continue with manual cleanup"
            Add-Summary -Step "cleanup_cli_uninstall" -Status "skipped" -Detail "non-blocking: $($_.Exception.Message)"
        }

        $pidResult = Invoke-Adb -CmdArgs @("shell", "pidof", "atx-agent") -AllowFailure
        $pidText = $pidResult.Output.Trim()
        if ($pidText) {
            foreach ($pid in ($pidText -split "\s+")) {
                if ($pid) {
                    Invoke-Adb -CmdArgs @("shell", "kill", "-9", $pid) -AllowFailure | Out-Null
                }
            }
        }

        Invoke-Adb -CmdArgs @("shell", "rm", "-f", "/data/local/tmp/atx-agent", "/data/local/tmp/app-uiautomator.apk", "/data/local/tmp/app-uiautomator-test.apk", "/data/local/tmp/minicap", "/data/local/tmp/minitouch") -AllowFailure | Out-Null
        Invoke-Adb -CmdArgs @("shell", "am", "force-stop", "com.github.uiautomator") -AllowFailure | Out-Null
        Invoke-Adb -CmdArgs @("shell", "am", "force-stop", "com.github.uiautomator.test") -AllowFailure | Out-Null
        Invoke-Adb -CmdArgs @("shell", "pm", "uninstall", "com.github.uiautomator") -AllowFailure | Out-Null
        Invoke-Adb -CmdArgs @("shell", "pm", "uninstall", "com.github.uiautomator.test") -AllowFailure | Out-Null
        Invoke-Adb -CmdArgs @("forward", "--remove-all") -AllowFailure | Out-Null

        $checkAtx = Invoke-Adb -CmdArgs @("shell", "if [ -f /data/local/tmp/atx-agent ]; then echo yes; else echo no; fi")
        if ($checkAtx.Output.Trim() -eq "yes") {
            throw "cleanup failed: /data/local/tmp/atx-agent still exists"
        }

        Add-Summary -Step "cleanup" -Status "ok" -Detail "target cleanup completed"
    }
    else {
        Add-Summary -Step "cleanup" -Status "skipped" -Detail "SkipCleanup"
    }

    if (-not $SkipInit) {
        Write-Step "Initialize ATX-Agent from empty state"
        $initResult = Start-StepProcess `
            -Name "init_force" `
            -WorkingDirectory (Join-Path $repoRoot "uiautomator-cli") `
            -Command "cargo run -- init -f -s $Serial" `
            -TimeoutMinutes 15
        Add-Summary -Step "init_force" -Status "ok" -Detail "uiautomator-cli init -f -s $Serial" -DurationSeconds $initResult.DurationSeconds -ExitCode $initResult.ExitCode -StdoutPath $initResult.Stdout -StderrPath $initResult.Stderr
    }
    else {
        Add-Summary -Step "init_force" -Status "skipped" -Detail "SkipInit"
    }

    if (-not $SkipTestAppInstall) {
        Write-Step "Ensure test-app is installed"
        $apkPath = Join-Path $repoRoot $TestAppApk
        if (-not (Test-Path $apkPath)) {
            throw "test-app apk not found: $apkPath"
        }

        $packageName = "com.uiautomator.testapp"
        $localSha256 = (Get-FileHash -Path $apkPath -Algorithm SHA256).Hash.ToLowerInvariant()
        $localMd5 = (Get-FileHash -Path $apkPath -Algorithm MD5).Hash.ToLowerInvariant()
        $remoteApkPath = Get-InstalledPackageApkPath -PackageName $packageName
        $remoteHash = $null
        $shouldInstall = $true
        $reason = ""

        if (-not $remoteApkPath) {
            $reason = "not installed"
        }
        else {
            $remoteHash = Get-RemoteFileHash -RemotePath $remoteApkPath
            if ($null -eq $remoteHash) {
                $reason = "installed but hash unavailable"
            }
            elseif ($remoteHash.Algorithm -eq "SHA256") {
                if ($remoteHash.Hash -eq $localSha256) {
                    $shouldInstall = $false
                    $reason = "installed sha256 match"
                }
                else {
                    $reason = "sha256 mismatch"
                }
            }
            else {
                if ($remoteHash.Hash -eq $localMd5) {
                    $shouldInstall = $false
                    $reason = "installed md5 match"
                }
                else {
                    $reason = "md5 mismatch"
                }
            }
        }

        if ($shouldInstall) {
            $installOutput = & adb -s $Serial install -r $apkPath 2>&1
            if ($LASTEXITCODE -ne 0) {
                $text = ($installOutput | ForEach-Object { $_.ToString() }) -join "`n"
                throw "test-app install failed on $Serial`n$text"
            }
            Add-Summary -Step "install_test_app" -Status "ok" -Detail "installed ($reason)"
        }
        else {
            Add-Summary -Step "install_test_app" -Status "ok" -Detail "already installed ($reason)"
        }
    }
    else {
        Add-Summary -Step "install_test_app" -Status "skipped" -Detail "SkipTestAppInstall"
    }

    Write-Step "Run full test matrix with serial pinning"
    $testSteps = @(
        @{
            Name    = "uiautomator-cli_nonignored"
            WorkDir = Join-Path $repoRoot "uiautomator-cli"
            Cmd     = "`$env:TEST_DEVICE_SERIAL='$Serial'; `$env:ANDROID_SERIAL='$Serial'; `$env:RUST_TEST_THREADS='1'; cargo test -- --nocapture --test-threads=1"
        },
        @{
            Name    = "uiautomator-cli_ignored"
            WorkDir = Join-Path $repoRoot "uiautomator-cli"
            Cmd     = "`$env:TEST_DEVICE_SERIAL='$Serial'; `$env:ANDROID_SERIAL='$Serial'; `$env:RUST_TEST_THREADS='1'; cargo test -- --ignored --nocapture --test-threads=1"
        },
        @{
            Name    = "uiautomator_nonignored"
            WorkDir = Join-Path $repoRoot "uiautomator"
            Cmd     = "`$env:TEST_DEVICE_SERIAL='$Serial'; `$env:ANDROID_SERIAL='$Serial'; `$env:RUST_TEST_THREADS='1'; cargo test -- --nocapture --test-threads=1"
        },
        @{
            Name    = "uiautomator_ignored"
            WorkDir = Join-Path $repoRoot "uiautomator"
            Cmd     = "`$env:TEST_DEVICE_SERIAL='$Serial'; `$env:ANDROID_SERIAL='$Serial'; `$env:RUST_TEST_THREADS='1'; cargo test -- --ignored --nocapture --test-threads=1"
        }
    )

    foreach ($step in $testSteps) {
        $stepResult = Start-StepProcess `
            -Name $step.Name `
            -WorkingDirectory $step.WorkDir `
            -Command $step.Cmd `
            -TimeoutMinutes $StepTimeoutMinutes
        Add-Summary -Step $step.Name -Status "ok" -Detail "passed" -DurationSeconds $stepResult.DurationSeconds -ExitCode $stepResult.ExitCode -StdoutPath $stepResult.Stdout -StderrPath $stepResult.Stderr
    }

    $postRunArtifacts = Move-RootDebugXmlArtifacts -DestinationDir $debugDumpDir
    if ($postRunArtifacts.Count -gt 0) {
        Write-Host ("Moved {0} post-run debug xml file(s) to {1}" -f $postRunArtifacts.Count, $debugDumpDir)
        Add-Summary -Step "collect_debug_xml_post" -Status "ok" -Detail ("moved {0} file(s) to debug-dumps" -f $postRunArtifacts.Count)
    }
    else {
        Add-Summary -Step "collect_debug_xml_post" -Status "ok" -Detail "no post-run debug xml files found in repo root"
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
    try {
        $postRunArtifacts = Move-RootDebugXmlArtifacts -DestinationDir $debugDumpDir
        if ($postRunArtifacts.Count -gt 0) {
            Write-Host ("Moved {0} post-failure debug xml file(s) to {1}" -f $postRunArtifacts.Count, $debugDumpDir)
            Add-Summary -Step "collect_debug_xml_post" -Status "ok" -Detail ("moved {0} file(s) to debug-dumps" -f $postRunArtifacts.Count)
        }
    }
    catch {
        Write-Host "Failed to collect debug xml artifacts after failure: $($_.Exception.Message)"
    }

    Add-Summary -Step "run" -Status "failed" -Detail $failureMessage
    $structured = Write-StructuredSummary -RunStatus "failed" -FailureMessage $failureMessage
    Write-RunManifest -RunStatus "failed" -StructuredSummary $structured -FailureMessage $failureMessage
    Write-Host ""
    Write-Host "FAILED"
    $summary | Select-Object Step, Status, Detail | Format-Table -Wrap -AutoSize | Out-String | Write-Host
    Write-Host "summary json: $($structured.JsonPath)"
    Write-Host "summary junit: $($structured.JunitPath)"
    if (-not [string]::IsNullOrWhiteSpace($OutputManifestPath)) {
        Write-Host "manifest: $OutputManifestPath"
    }
    Write-Host "logs: $runLogDir"
    throw
}
