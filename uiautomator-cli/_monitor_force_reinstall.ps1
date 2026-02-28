param(
    [string]$Command = 'cargo test --test installer_test test_force_reinstall -- --ignored --nocapture --test-threads=1',
    [string]$LogFile = 'force_reinstall_test.log',
    [string]$DeviceSerial = 'emulator-5554',
    [int]$DeadlineMinutes = 25,
    [int]$NoActivityMinutes = 10,
    [int]$PollSeconds = 5
)

$ErrorActionPreference = 'Stop'
$workDir = Get-Location
$logPath = Join-Path $workDir $LogFile

if (Test-Path $logPath) {
    try {
        Remove-Item $logPath -Force -ErrorAction Stop
    }
    catch {
        $stamp = Get-Date -Format 'yyyyMMdd_HHmmss'
        $base = [System.IO.Path]::GetFileNameWithoutExtension($LogFile)
        $ext = [System.IO.Path]::GetExtension($LogFile)
        $LogFile = "${base}_${stamp}${ext}"
        $logPath = Join-Path $workDir $LogFile
        Write-Host "[monitor] existing log is locked, switch to: $logPath"
    }
}

$env:TEST_DEVICE_SERIAL = $DeviceSerial

$cmd = "$Command > `"$LogFile`" 2>&1"
Write-Host "[monitor] start command: $Command"
Write-Host "[monitor] log file: $logPath"
Write-Host "[monitor] deadline: ${DeadlineMinutes}min, no-activity: ${NoActivityMinutes}min"

$proc = Start-Process -FilePath 'cmd.exe' -ArgumentList '/c', $cmd -WorkingDirectory $workDir -PassThru
$deadline = (Get-Date).AddMinutes($DeadlineMinutes)
$lastChange = Get-Date
$lastLen = 0L
$lastReadPos = 0L
$timedOut = $false
$noActivityKill = $false

while ($true) {
    $proc.Refresh()
    if ($proc.HasExited) { break }

    if ((Get-Date) -ge $deadline) {
        $timedOut = $true
        break
    }

    if (Test-Path $logPath) {
        $item = Get-Item $logPath
        $len = $item.Length

        if ($len -gt $lastLen) {
            $lastChange = Get-Date
            $fs = [System.IO.File]::Open($logPath, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
            try {
                if ($lastReadPos -gt $fs.Length) { $lastReadPos = 0 }
                $fs.Seek($lastReadPos, [System.IO.SeekOrigin]::Begin) | Out-Null
                $sr = New-Object System.IO.StreamReader($fs)
                $delta = $sr.ReadToEnd()
                $lastReadPos = $fs.Position
                if ($delta) {
                    Write-Host $delta -NoNewline
                }
            }
            finally {
                $fs.Dispose()
            }
            $lastLen = $len
        }

        if (((Get-Date) - $lastChange).TotalMinutes -ge $NoActivityMinutes) {
            $noActivityKill = $true
            break
        }
    }

    Start-Sleep -Seconds $PollSeconds
}

if (-not $proc.HasExited) {
    try {
        cmd.exe /c "taskkill /PID $($proc.Id) /T /F >nul 2>&1" | Out-Null
    } catch {}
    try {
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    } catch {}
    Start-Sleep -Seconds 1
    $proc.Refresh()
}

$exitCode = if ($proc.HasExited) { $proc.ExitCode } else { -1 }
Write-Host "`n[monitor] PROCESS_EXITED=$($proc.HasExited) EXIT_CODE=$exitCode TIMED_OUT=$timedOut NO_ACTIVITY_KILL=$noActivityKill"

if (Test-Path $logPath) {
    Write-Host '----- LOG TAIL (200) -----'
    Get-Content -Path $logPath -Tail 200
} else {
    Write-Host 'LOG_FILE_NOT_FOUND'
}

if ($timedOut -or $noActivityKill) {
    exit 124
}

exit $exitCode

