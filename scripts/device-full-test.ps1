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

    [switch]$SkipTestAppInstall,

    [bool]$StrictEnvironmentCheck = $true
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if ($Serial -notmatch '^[A-Za-z0-9._:-]+$') {
    throw "invalid serial format: '$Serial'"
}

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[Console]::InputEncoding = $utf8NoBom
[Console]::OutputEncoding = $utf8NoBom
$OutputEncoding = $utf8NoBom
$PSDefaultParameterValues["Out-File:Encoding"] = "utf8"
$PSDefaultParameterValues["Set-Content:Encoding"] = "utf8"
$PSDefaultParameterValues["Add-Content:Encoding"] = "utf8"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

function Resolve-CacheRoot {
    $candidates = @(
        $env:UIAUTOMATOR_CACHE_DIR,
        $env:RUNNER_TEMP,
        $env:TEMP,
        [System.IO.Path]::GetTempPath()
    ) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique

    foreach ($candidate in $candidates) {
        try {
            New-Item -ItemType Directory -Force -Path $candidate | Out-Null
            return (Resolve-Path -Path $candidate).Path
        }
        catch {
            continue
        }
    }

    throw "unable to resolve a writable cache root for regression artifacts"
}

$runStartedAt = Get-Date
$runId = Get-Date -Format "yyyyMMdd-HHmmss"
$safeSerialForPath = ($Serial -replace '[\\/:*?"<>|]', '_')
$runLogDir = Join-Path $repoRoot (Join-Path $LogRoot "$runId-$safeSerialForPath")
$debugDumpDir = Join-Path $runLogDir "debug-dumps"
New-Item -ItemType Directory -Force -Path $runLogDir | Out-Null

$cacheRoot = Resolve-CacheRoot
$cargoTargetDir = Join-Path $cacheRoot "uiautomator-rs\cargo-target"
New-Item -ItemType Directory -Force -Path $cargoTargetDir | Out-Null
$env:CARGO_TARGET_DIR = $cargoTargetDir

$summary = New-Object System.Collections.Generic.List[object]
$deviceProfile = [ordered]@{
    abi             = ""
    android_release = ""
    android_major   = 0
    sdk_int         = ""
}
$runEnvironment = [ordered]@{
    cache_root       = $cacheRoot
    cargo_target_dir = $cargoTargetDir
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
        run_environment   = $runEnvironment
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
    [void]$builder.AppendLine(('      <property name="cache_root" value="{0}" />' -f (Escape-XmlText $cacheRoot)))
    [void]$builder.AppendLine(('      <property name="cargo_target_dir" value="{0}" />' -f (Escape-XmlText $cargoTargetDir)))
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
        run_environment  = $runEnvironment
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

function Remove-ManagedForwards {
    $forwardList = Invoke-Adb -CmdArgs @("forward", "--list") -AllowFailure
    if ($forwardList.ExitCode -ne 0) {
        return
    }

    foreach ($line in ($forwardList.Output -split "`r?`n")) {
        $trimmed = $line.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed)) {
            continue
        }

        $parts = $trimmed -split "\s+"
        if ($parts.Count -lt 3) {
            continue
        }

        $lineSerial = $parts[0]
        $localPort = $parts[1]
        $remotePort = $parts[2]

        if ($lineSerial -ne $Serial) {
            continue
        }

        if ($remotePort -in @("tcp:7912", "tcp:9008")) {
            Invoke-Adb -CmdArgs @("forward", "--remove", $localPort) -AllowFailure | Out-Null
        }
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
    $exitCodeFile = Join-Path $runLogDir "$safeName.exitcode.txt"
    if (Test-Path $exitCodeFile) {
        Remove-Item -Path $exitCodeFile -Force -ErrorAction SilentlyContinue
    }
    $escapedExitCodeFile = $exitCodeFile -replace "'", "''"
    $wrapper = '$ErrorActionPreference=''Stop''; $utf8 = New-Object System.Text.UTF8Encoding($false); [Console]::InputEncoding = $utf8; [Console]::OutputEncoding = $utf8; $OutputEncoding = $utf8; $exitCodePath = ''' + $escapedExitCodeFile + '''; try { ' + $Command + '; $code = if ($null -eq $LASTEXITCODE) { 0 } else { [int]$LASTEXITCODE } } catch { Write-Error ($_ | Out-String); $code = 1 }; try { Set-Content -Path $exitCodePath -Value $code -Encoding ascii -NoNewline } catch {}; Write-Output "__CODEX_EXIT__=$code"; exit $code'

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
    if (Test-Path $exitCodeFile) {
        $rawExitCode = Get-Content $exitCodeFile -Raw -Encoding ascii -ErrorAction SilentlyContinue
        if ($rawExitCode) {
            $trimmedExitCode = $rawExitCode.Trim()
            if ($trimmedExitCode -match '^-?\d+$') {
                $exitCode = [int]$trimmedExitCode
            }
        }
    }

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
        try {
            $exitCode = [int]$proc.ExitCode
        }
        catch {
            $exitCode = $null
        }
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
        ExitCodeFile    = $exitCodeFile
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

function Install-TestAppWithFallback {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ApkPath,
        [Parameter(Mandatory = $true)]
        [string]$PackageName
    )

    $installResult = Invoke-Adb -CmdArgs @("install", "-r", $ApkPath) -AllowFailure
    if ($installResult.ExitCode -eq 0) {
        return "installed/reinstalled with -r"
    }

    $text = $installResult.Output
    if ($text -match "INSTALL_FAILED_UPDATE_INCOMPATIBLE") {
        Write-Host "test-app signature mismatch detected; uninstalling existing package and retrying."
        $null = Invoke-Adb -CmdArgs @("uninstall", $PackageName) -AllowFailure
        $installRetryResult = Invoke-Adb -CmdArgs @("install", $ApkPath) -AllowFailure
        if ($installRetryResult.ExitCode -eq 0) {
            return "reinstalled after signature mismatch"
        }

        $retryText = $installRetryResult.Output
        throw "test-app reinstall failed on $Serial after uninstall`n$retryText"
    }

    throw "test-app install failed on $Serial`n$text"
}

function Get-AtxAgentProcessStatus {
    $pidResult = Invoke-Adb -CmdArgs @("shell", "pidof", "atx-agent") -AllowFailure
    $pidText = $pidResult.Output.Trim()
    if ($pidResult.ExitCode -eq 0 -and -not [string]::IsNullOrWhiteSpace($pidText)) {
        return [PSCustomObject]@{
            Running = $true
            Detail  = "pidof=$pidText"
        }
    }

    $psCandidates = @(
        @("shell", "ps", "-A"),
        @("shell", "ps")
    )

    foreach ($candidate in $psCandidates) {
        $psResult = Invoke-Adb -CmdArgs $candidate -AllowFailure
        if ($psResult.ExitCode -ne 0) {
            continue
        }

        foreach ($line in ($psResult.Output -split "`r?`n")) {
            if ($line -match '(^|\s)(atx-agent|/data/local/tmp/atx-agent)(\s|$)') {
                return [PSCustomObject]@{
                    Running = $true
                    Detail  = "ps-match: $($line.Trim())"
                }
            }
        }
    }

    $pidDetail = if ($pidResult.ExitCode -eq 0) {
        "pidof returned empty"
    }
    else {
        "pidof exit=$($pidResult.ExitCode)"
    }

    return [PSCustomObject]@{
        Running = $false
        Detail  = $pidDetail
    }
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

    Write-Host "Using cache root: $cacheRoot"
    Write-Host "Using CARGO_TARGET_DIR: $cargoTargetDir"
    Add-Summary -Step "configure_cargo_target" -Status "ok" -Detail "CARGO_TARGET_DIR=$cargoTargetDir"

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

    Write-Step "Preflight environment readiness"
    $shellPing = Invoke-Adb -CmdArgs @("shell", "echo", "__U2_PREFLIGHT_OK__")
    if ($shellPing.Output.Trim() -ne "__U2_PREFLIGHT_OK__") {
        throw "preflight failed: adb shell echo mismatch"
    }

    $pmProbe = Invoke-Adb -CmdArgs @("shell", "pm", "path", "android") -AllowFailure
    if ($pmProbe.ExitCode -ne 0 -or $pmProbe.Output -notmatch "package:") {
        throw "preflight failed: package manager is not ready on device '$Serial'"
    }
    Add-Summary -Step "preflight_environment" -Status "ok" -Detail "adb shell + pm path validated"

    if (-not $SkipCleanup) {
        Write-Step "Cleanup ATX/uiautomator state on target"

        try {
            $cleanupCliResult = Start-StepProcess `
                -Name "cleanup_cli_uninstall" `
                -WorkingDirectory (Join-Path $repoRoot "uiautomator-cli") `
                -Command "cargo run -- uninstall -s '$Serial'" `
                -TimeoutMinutes 10
            Add-Summary -Step "cleanup_cli_uninstall" -Status "ok" -Detail "cargo run -- uninstall -s '$Serial'" -DurationSeconds $cleanupCliResult.DurationSeconds -ExitCode $cleanupCliResult.ExitCode -StdoutPath $cleanupCliResult.Stdout -StderrPath $cleanupCliResult.Stderr
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
        Remove-ManagedForwards

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
            -Command "cargo run -- init -f -s '$Serial'" `
            -TimeoutMinutes 15
        Add-Summary -Step "init_force" -Status "ok" -Detail "uiautomator-cli init -f -s '$Serial'" -DurationSeconds $initResult.DurationSeconds -ExitCode $initResult.ExitCode -StdoutPath $initResult.Stdout -StderrPath $initResult.Stderr

        Write-Step "Verify runtime environment after init"
        $statusResult = Start-StepProcess `
            -Name "verify_cli_status" `
            -WorkingDirectory (Join-Path $repoRoot "uiautomator-cli") `
            -Command "cargo run -- status -s '$Serial'" `
            -TimeoutMinutes 10

        $atxBinary = Invoke-Adb -CmdArgs @("shell", "ls", "/data/local/tmp/atx-agent") -AllowFailure
        if ($atxBinary.ExitCode -ne 0 -or $atxBinary.Output -notmatch "atx-agent") {
            throw "runtime verify failed: /data/local/tmp/atx-agent missing after init"
        }

        $atxProcess = Get-AtxAgentProcessStatus
        if (-not $atxProcess.Running) {
            throw "runtime verify failed: atx-agent process not running after init ($($atxProcess.Detail))"
        }

        $u2MainPkg = Invoke-Adb -CmdArgs @("shell", "pm", "path", "com.github.uiautomator") -AllowFailure
        $u2TestPkg = Invoke-Adb -CmdArgs @("shell", "pm", "path", "com.github.uiautomator.test") -AllowFailure
        if ($u2MainPkg.ExitCode -ne 0 -or $u2MainPkg.Output -notmatch "package:") {
            throw "runtime verify failed: com.github.uiautomator not installed after init"
        }
        if ($u2TestPkg.ExitCode -ne 0 -or $u2TestPkg.Output -notmatch "package:") {
            throw "runtime verify failed: com.github.uiautomator.test not installed after init"
        }

        Add-Summary -Step "verify_runtime_environment" -Status "ok" -Detail "cli status + atx-agent binary/pid + uiautomator packages verified" -DurationSeconds $statusResult.DurationSeconds -ExitCode $statusResult.ExitCode -StdoutPath $statusResult.Stdout -StderrPath $statusResult.Stderr
    }
    else {
        Add-Summary -Step "init_force" -Status "skipped" -Detail "SkipInit"
        Add-Summary -Step "verify_runtime_environment" -Status "skipped" -Detail "SkipInit"
    }

    if (-not $SkipTestAppInstall) {
        Write-Step "Ensure test-app is installed"
        $apkPath = Join-Path $repoRoot $TestAppApk
        if (-not (Test-Path $apkPath)) {
            throw "test-app apk not found: $apkPath"
        }

        $packageName = "com.uiautomator.testapp"

        if ($StrictEnvironmentCheck) {
            $installDetail = Install-TestAppWithFallback -ApkPath $apkPath -PackageName $packageName
            Add-Summary -Step "install_test_app" -Status "ok" -Detail "strict-check: $installDetail"
        }
        else {
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
            $installResult = Invoke-Adb -CmdArgs @("install", "-r", $apkPath) -AllowFailure
            if ($installResult.ExitCode -ne 0) {
                $text = $installResult.Output
                if ($text -match "INSTALL_FAILED_UPDATE_INCOMPATIBLE") {
                    Write-Host "test-app signature mismatch detected; uninstalling existing package and retrying."
                    $null = Invoke-Adb -CmdArgs @("uninstall", $packageName) -AllowFailure

                    $installRetryResult = Invoke-Adb -CmdArgs @("install", $apkPath) -AllowFailure
                    if ($installRetryResult.ExitCode -ne 0) {
                        $retryText = $installRetryResult.Output
                        throw "test-app reinstall failed on $Serial after uninstall`n$retryText"
                    }

                    Add-Summary -Step "install_test_app" -Status "ok" -Detail "installed ($reason; reinstalled after signature mismatch)"
                }
                else {
                    throw "test-app install failed on $Serial`n$text"
                }
            }
            else {
                Add-Summary -Step "install_test_app" -Status "ok" -Detail "installed ($reason)"
            }
        }
        else {
            Add-Summary -Step "install_test_app" -Status "ok" -Detail "already installed ($reason)"
        }
        }
    }
    else {
        Add-Summary -Step "install_test_app" -Status "skipped" -Detail "SkipTestAppInstall"
    }

    Write-Step "Prewarm cargo test binaries"
    $prewarmSteps = @(
        @{
            Name    = "prewarm_uiautomator_cli"
            WorkDir = Join-Path $repoRoot "uiautomator-cli"
            Cmd     = "cargo test --no-run"
        },
        @{
            Name    = "prewarm_uiautomator"
            WorkDir = Join-Path $repoRoot "uiautomator"
            Cmd     = "cargo test --no-run"
        }
    )

    foreach ($step in $prewarmSteps) {
        $stepResult = Start-StepProcess `
            -Name $step.Name `
            -WorkingDirectory $step.WorkDir `
            -Command $step.Cmd `
            -TimeoutMinutes $StepTimeoutMinutes
        Add-Summary -Step $step.Name -Status "ok" -Detail "cargo test --no-run completed" -DurationSeconds $stepResult.DurationSeconds -ExitCode $stepResult.ExitCode -StdoutPath $stepResult.Stdout -StderrPath $stepResult.Stderr
    }

    Write-Step "Run full test matrix with serial pinning"
    $testSteps = @(
        @{
            Name    = "uiautomator-cli_nonignored"
            WorkDir = Join-Path $repoRoot "uiautomator-cli"
            Cmd     = "`$env:TEST_DEVICE_SERIAL='$Serial'; `$env:ANDROID_SERIAL='$Serial'; `$env:RUST_TEST_THREADS='1'; `$env:UIAUTOMATOR_ALLOW_POWER_KEY_TEST='0'; cargo test -- --nocapture --test-threads=1"
        },
        @{
            Name    = "uiautomator-cli_ignored"
            WorkDir = Join-Path $repoRoot "uiautomator-cli"
            Cmd     = "`$env:TEST_DEVICE_SERIAL='$Serial'; `$env:ANDROID_SERIAL='$Serial'; `$env:RUST_TEST_THREADS='1'; `$env:UIAUTOMATOR_ALLOW_POWER_KEY_TEST='0'; cargo test -- --ignored --nocapture --test-threads=1"
        },
        @{
            Name    = "uiautomator_nonignored"
            WorkDir = Join-Path $repoRoot "uiautomator"
            Cmd     = "`$env:TEST_DEVICE_SERIAL='$Serial'; `$env:ANDROID_SERIAL='$Serial'; `$env:RUST_TEST_THREADS='1'; `$env:UIAUTOMATOR_ALLOW_POWER_KEY_TEST='0'; cargo test -- --nocapture --test-threads=1"
        },
        @{
            Name    = "uiautomator_ignored"
            WorkDir = Join-Path $repoRoot "uiautomator"
            Cmd     = "`$env:TEST_DEVICE_SERIAL='$Serial'; `$env:ANDROID_SERIAL='$Serial'; `$env:RUST_TEST_THREADS='1'; `$env:UIAUTOMATOR_ALLOW_POWER_KEY_TEST='0'; cargo test -- --ignored --nocapture --test-threads=1"
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
