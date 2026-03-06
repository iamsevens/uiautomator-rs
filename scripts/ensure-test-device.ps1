param(
    [Parameter(Mandatory = $true)]
    [string]$Serial,

    [string]$LdplayerStartCommand = "",

    [string]$LdplayerStopCommand = "",

    [string]$MumuStartCommand = "",

    [string]$MumuStopCommand = "",

    [string]$MumuConnectEndpoints = "",

    [string]$LauncherStatePath = "",

    [switch]$RegisterStopWhenAlreadyOnline,

    [int]$WaitTimeoutSeconds = 360,

    [int]$PollIntervalSeconds = 5
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$defaultMumuEndpoints = "127.0.0.1:16384,127.0.0.1:5555"
$isTcpSerial = $Serial -match '^\d{1,3}(\.\d{1,3}){3}:\d+$'
$isEmulatorSerial = $Serial -like "emulator-*"
$shouldHandleEmulator = $isTcpSerial -or $isEmulatorSerial

if ([string]::IsNullOrWhiteSpace($LdplayerStartCommand) -and -not [string]::IsNullOrWhiteSpace($env:LDPLAYER_START_CMD)) {
    $LdplayerStartCommand = $env:LDPLAYER_START_CMD
}

if ([string]::IsNullOrWhiteSpace($LdplayerStopCommand) -and -not [string]::IsNullOrWhiteSpace($env:LDPLAYER_STOP_CMD)) {
    $LdplayerStopCommand = $env:LDPLAYER_STOP_CMD
}

if ([string]::IsNullOrWhiteSpace($MumuStartCommand) -and -not [string]::IsNullOrWhiteSpace($env:MUMU_START_CMD)) {
    $MumuStartCommand = $env:MUMU_START_CMD
}

if ([string]::IsNullOrWhiteSpace($MumuStopCommand) -and -not [string]::IsNullOrWhiteSpace($env:MUMU_STOP_CMD)) {
    $MumuStopCommand = $env:MUMU_STOP_CMD
}

function Resolve-MumuIndex {
    param([string]$Command)

    if (-not [string]::IsNullOrWhiteSpace($Command) -and $Command -match '(?i)(--vmindex|-v)\s+(\d+)') {
        return [int]$matches[2]
    }

    return 0
}

function Resolve-LdplayerConsolePath {
    param([string]$StartCommand)

    if (-not [string]::IsNullOrWhiteSpace($StartCommand) -and $StartCommand -match "(?i)'([^']*ldconsole\.exe)'") {
        return $matches[1]
    }

    $defaultLdconsolePath = "D:\leidian\LDPlayer9\ldconsole.exe"
    if (Test-Path $defaultLdconsolePath) {
        return $defaultLdconsolePath
    }

    return ""
}

function Resolve-LdplayerIndex {
    param([string]$StartCommand)

    if (-not [string]::IsNullOrWhiteSpace($StartCommand) -and $StartCommand -match '(?i)--index\s+(\d+)') {
        return [int]$matches[1]
    }

    return 0
}

$mumuIndex = Resolve-MumuIndex -Command $MumuStartCommand
$ldplayerIndex = Resolve-LdplayerIndex -StartCommand $LdplayerStartCommand
$ldconsoleResolvedPath = Resolve-LdplayerConsolePath -StartCommand $LdplayerStartCommand

$defaultMumuManagerPath = "C:\Program Files\Netease\MuMu\nx_main\MuMuManager.exe"
if ($isTcpSerial) {
    if (-not [string]::IsNullOrWhiteSpace($MumuStartCommand) -and $MumuStartCommand -match '(?i)MuMuNxMain\.exe' -and (Test-Path $defaultMumuManagerPath)) {
        Write-Host "[MuMu] normalize start command from MuMuNxMain to MuMuManager launch."
        $MumuStartCommand = "& '$defaultMumuManagerPath' control -v $mumuIndex launch"
    }
    if ([string]::IsNullOrWhiteSpace($MumuStopCommand) -and (Test-Path $defaultMumuManagerPath)) {
        $MumuStopCommand = "& '$defaultMumuManagerPath' control -v $mumuIndex shutdown"
    }
}

if ([string]::IsNullOrWhiteSpace($LdplayerStopCommand)) {
    if (-not [string]::IsNullOrWhiteSpace($ldconsoleResolvedPath)) {
        $LdplayerStopCommand = "& '$ldconsoleResolvedPath' quit --index $ldplayerIndex"
    }
    else {
        $LdplayerStopCommand = ""
    }
}

if ([string]::IsNullOrWhiteSpace($MumuConnectEndpoints)) {
    if (-not [string]::IsNullOrWhiteSpace($env:MUMU_ADB_ENDPOINTS)) {
        $MumuConnectEndpoints = $env:MUMU_ADB_ENDPOINTS
    }
    elseif ($isTcpSerial) {
        $MumuConnectEndpoints = $defaultMumuEndpoints
    }
    else {
        $MumuConnectEndpoints = ""
    }
}

function Wait-LdplayerRunning {
    param(
        [string]$LdconsolePath,
        [int]$Index = 0,
        [int]$TimeoutSeconds = 75,
        [int]$PollIntervalSeconds = 3
    )

    if ([string]::IsNullOrWhiteSpace($LdconsolePath) -or -not (Test-Path $LdconsolePath)) {
        return $false
    }

    $deadline = (Get-Date).AddSeconds([Math]::Max($TimeoutSeconds, 1))
    while ((Get-Date) -lt $deadline) {
        $res = & $LdconsolePath isrunning --index $Index 2>&1
        $statusText = (($res | ForEach-Object { $_.ToString().Trim() }) -join " ").ToLowerInvariant()
        if ($LASTEXITCODE -eq 0 -and $statusText -match '\brunning\b') {
            return $true
        }
        Start-Sleep -Seconds ([Math]::Max($PollIntervalSeconds, 1))
    }

    return $false
}

function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "==> $Message"
}

function Invoke-AdbRaw {
    param(
        [string[]]$CmdArgs,
        [switch]$AllowFailure
    )

    $prevErrorAction = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $output = & adb @CmdArgs 2>&1
        $code = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $prevErrorAction
    }

    $text = ($output | ForEach-Object { $_.ToString() }) -join "`n"
    if (-not $AllowFailure -and $code -ne 0) {
        throw "adb $($CmdArgs -join ' ') failed (exit=$code)`n$text"
    }

    return [PSCustomObject]@{
        ExitCode = $code
        Output   = $text
    }
}

function Start-LauncherCommand {
    param(
        [string]$Name,
        [string]$Command,
        [string]$StopCommand = ""
    )

    if ([string]::IsNullOrWhiteSpace($Command)) {
        Write-Host "[$Name] start command is empty. skip."
        return $null
    }

    Write-Host "[$Name] start command: $Command"
    $proc = Start-Process `
        -FilePath "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe" `
        -ArgumentList @("-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", $Command) `
        -WindowStyle Hidden `
        -PassThru
    Write-Host "[$Name] launcher pid: $($proc.Id)"
    return [PSCustomObject]@{
        Name        = $Name
        Pid         = $proc.Id
        Command     = $Command
        StopCommand = $StopCommand
    }
}

function Invoke-LauncherCommandSync {
    param(
        [string]$Name,
        [string]$Command,
        [switch]$AllowFailure
    )

    if ([string]::IsNullOrWhiteSpace($Command)) {
        Write-Host "[$Name] command is empty. skip."
        return $true
    }

    Write-Host "[$Name] run command: $Command"
    $res = & "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -Command $Command 2>&1
    $code = $LASTEXITCODE
    if ($code -ne 0) {
        $text = ($res | ForEach-Object { $_.ToString() }) -join "`n"
        if ($AllowFailure) {
            Write-Host "[$Name] command failed but ignored (exit=$code): $text"
            return $false
        }
        throw "[$Name] command failed (exit=$code): $text"
    }
    return $true
}

function Get-AdbDeviceStates {
    $res = Invoke-AdbRaw -CmdArgs @("devices")
    $states = @{}

    $lines = $res.Output -split "`r?`n"
    foreach ($line in $lines) {
        $trimmed = $line.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed)) {
            continue
        }

        if ($trimmed -like "List of devices attached*") {
            continue
        }

        if ($trimmed -match '^(\S+)\s+(\S+)$') {
            $states[$matches[1]] = $matches[2]
        }
    }

    return $states
}

function Connect-Endpoints {
    param([string]$EndpointsCsv)

    if ([string]::IsNullOrWhiteSpace($EndpointsCsv)) {
        return
    }

    $endpoints = $EndpointsCsv.Split(",") |
        ForEach-Object { $_.Trim() } |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
        Select-Object -Unique

    foreach ($endpoint in $endpoints) {
        $res = Invoke-AdbRaw -CmdArgs @("connect", $endpoint) -AllowFailure
        if (-not [string]::IsNullOrWhiteSpace($res.Output)) {
            Write-Host "[adb connect] $endpoint -> $($res.Output.Trim())"
        }
    }
}

if ($PollIntervalSeconds -lt 1) {
    throw "PollIntervalSeconds must be >= 1"
}

if ($WaitTimeoutSeconds -lt $PollIntervalSeconds) {
    $WaitTimeoutSeconds = $PollIntervalSeconds
}

Write-Step "Ensure adb server is running"
$null = Invoke-AdbRaw -CmdArgs @("start-server")

function Get-TargetState {
    param([string]$TargetSerial)

    $states = Get-AdbDeviceStates
    if ($states.ContainsKey($TargetSerial)) {
        return [string]$states[$TargetSerial]
    }
    return ""
}

$targetInitialState = Get-TargetState -TargetSerial $Serial
$targetAlreadyOnline = $targetInitialState -eq "device"

Write-Step "Optionally start emulators"
$startedLaunchers = New-Object System.Collections.Generic.List[object]
if ($shouldHandleEmulator) {
    if ($targetAlreadyOnline) {
        Write-Host "target serial already online: $Serial state=$targetInitialState; skip emulator auto-start."
        if ($RegisterStopWhenAlreadyOnline) {
            $stopCommand = if ($isTcpSerial) { $MumuStopCommand } elseif ($isEmulatorSerial) { $LdplayerStopCommand } else { "" }
            if (-not [string]::IsNullOrWhiteSpace($stopCommand)) {
                $startedLaunchers.Add([PSCustomObject]@{
                        Name        = if ($isTcpSerial) { "MuMu" } else { "LDPlayer" }
                        Pid         = 0
                        Command     = ""
                        StopCommand = $stopCommand
                })
                Write-Host "registered stop command for already-online target: $Serial"
            }
        }
    }
    else {
        if ($isTcpSerial) {
            $launcher = Start-LauncherCommand -Name "MuMu" -Command $MumuStartCommand -StopCommand $MumuStopCommand
            if ($null -ne $launcher) {
                $startedLaunchers.Add($launcher)
            }
        }
        elseif ($isEmulatorSerial) {
            $launcher = Start-LauncherCommand -Name "LDPlayer" -Command $LdplayerStartCommand -StopCommand $LdplayerStopCommand
            if ($null -ne $launcher) {
                $startedLaunchers.Add($launcher)
            }

            $ldconsolePath = Resolve-LdplayerConsolePath -StartCommand $LdplayerStartCommand
            $ldIndex = Resolve-LdplayerIndex -StartCommand $LdplayerStartCommand
            $ldReady = Wait-LdplayerRunning -LdconsolePath $ldconsolePath -Index $ldIndex -TimeoutSeconds 75 -PollIntervalSeconds ([Math]::Min([Math]::Max($PollIntervalSeconds, 1), 5))
            if (-not $ldReady) {
                $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
                throw "LDPlayer launch probe failed within 75s (index=$ldIndex). Service account may lack interactive desktop rights. current_identity=$identity"
            }
            Write-Host "[LDPlayer] running probe passed: index=$ldIndex"
        }
    }
}
else {
    Write-Host "target serial is not emulator/tcp. skip emulator auto-start."
}

if (-not [string]::IsNullOrWhiteSpace($LauncherStatePath)) {
    $stateDir = Split-Path -Parent $LauncherStatePath
    if (-not [string]::IsNullOrWhiteSpace($stateDir)) {
        New-Item -ItemType Directory -Force -Path $stateDir | Out-Null
    }
    $state = [ordered]@{
        serial    = $Serial
        started_at = (Get-Date).ToString("o")
        launchers = $startedLaunchers.ToArray()
    }
    $state | ConvertTo-Json -Depth 6 | Set-Content -Path $LauncherStatePath -Encoding utf8
    Write-Host "launcher state path: $LauncherStatePath"
}

Write-Step "Try initial adb connect"
if ($isTcpSerial) {
    Connect-Endpoints -EndpointsCsv $MumuConnectEndpoints
    $null = Invoke-AdbRaw -CmdArgs @("connect", $Serial) -AllowFailure
}

Write-Step "Wait for target serial online: $Serial"
$deadline = (Get-Date).AddSeconds($WaitTimeoutSeconds)
$recoverAfterSeconds = [Math]::Min([Math]::Max([int]($WaitTimeoutSeconds / 3), 30), 120)
$recoverDeadline = (Get-Date).AddSeconds($recoverAfterSeconds)
$ldplayerRecovered = $false
$attempt = 0
$lastState = ""

while ((Get-Date) -lt $deadline) {
    $attempt++

    # Reconnect periodically for tcp endpoints.
    if ($isTcpSerial -and (($attempt -eq 1) -or ($attempt % [Math]::Max([int](30 / $PollIntervalSeconds), 1) -eq 0))) {
        Connect-Endpoints -EndpointsCsv $MumuConnectEndpoints
        if ($isTcpSerial) {
            $null = Invoke-AdbRaw -CmdArgs @("connect", $Serial) -AllowFailure
        }
    }

    $states = Get-AdbDeviceStates
    if ($states.ContainsKey($Serial)) {
        $state = [string]$states[$Serial]
        if ($state -eq "device") {
            Write-Host "target serial online: $Serial"
            exit 0
        }

        if ($state -ne $lastState) {
            Write-Host "target serial present but not ready: $Serial state=$state"
            $lastState = $state
        }
    }
    else {
        if ($isEmulatorSerial -and $shouldHandleEmulator -and -not $ldplayerRecovered -and (Get-Date) -ge $recoverDeadline) {
            Write-Host "[LDPlayer] serial still missing after ${recoverAfterSeconds}s; trying one auto-recovery restart."
            $ldplayerRecovered = $true

            $null = Invoke-LauncherCommandSync -Name "LDPlayer-stop-recovery" -Command $LdplayerStopCommand -AllowFailure
            Start-Sleep -Seconds ([Math]::Min([Math]::Max($PollIntervalSeconds, 2), 8))

            $launcher = Start-LauncherCommand -Name "LDPlayer-restart" -Command $LdplayerStartCommand -StopCommand $LdplayerStopCommand
            if ($null -ne $launcher) {
                $startedLaunchers.Add($launcher)
            }

            $ldconsolePath = Resolve-LdplayerConsolePath -StartCommand $LdplayerStartCommand
            $ldIndex = Resolve-LdplayerIndex -StartCommand $LdplayerStartCommand
            $ldReady = Wait-LdplayerRunning -LdconsolePath $ldconsolePath -Index $ldIndex -TimeoutSeconds 60 -PollIntervalSeconds ([Math]::Min([Math]::Max($PollIntervalSeconds, 1), 5))
            if (-not $ldReady) {
                Write-Host "[LDPlayer] recovery restart probe failed (index=$ldIndex), continue waiting for adb state."
            }
            else {
                Write-Host "[LDPlayer] recovery restart probe passed: index=$ldIndex"
            }
        }

        if ($attempt % [Math]::Max([int](20 / $PollIntervalSeconds), 1) -eq 0) {
            Write-Host "waiting for serial: $Serial"
        }
    }

    Start-Sleep -Seconds $PollIntervalSeconds
}

$finalStates = Get-AdbDeviceStates
$pairs = @()
foreach ($k in $finalStates.Keys) {
    $pairs += ("{0}:{1}" -f $k, $finalStates[$k])
}
$stateSummary = if ($pairs.Count -gt 0) { $pairs -join ", " } else { "<none>" }
throw "target serial did not become online within ${WaitTimeoutSeconds}s: $Serial; adb states: $stateSummary"
