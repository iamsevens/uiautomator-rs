param(
    [Parameter(Mandatory = $true)]
    [string]$Serial,

    [string]$LdplayerStartCommand = "",

    [string]$MumuStartCommand = "",

    [string]$MumuConnectEndpoints = "",

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

if ([string]::IsNullOrWhiteSpace($MumuStartCommand) -and -not [string]::IsNullOrWhiteSpace($env:MUMU_START_CMD)) {
    $MumuStartCommand = $env:MUMU_START_CMD
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
        [string]$Command
    )

    if ([string]::IsNullOrWhiteSpace($Command)) {
        Write-Host "[$Name] start command is empty. skip."
        return
    }

    Write-Host "[$Name] start command: $Command"
    $proc = Start-Process `
        -FilePath "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe" `
        -ArgumentList @("-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", $Command) `
        -WindowStyle Hidden `
        -PassThru
    Write-Host "[$Name] launcher pid: $($proc.Id)"
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

Write-Step "Optionally start emulators"
if ($shouldHandleEmulator) {
    Start-LauncherCommand -Name "LDPlayer" -Command $LdplayerStartCommand
    Start-LauncherCommand -Name "MuMu" -Command $MumuStartCommand
}
else {
    Write-Host "target serial is not emulator/tcp. skip emulator auto-start."
}

Write-Step "Try initial adb connect"
Connect-Endpoints -EndpointsCsv $MumuConnectEndpoints
if ($isTcpSerial) {
    $null = Invoke-AdbRaw -CmdArgs @("connect", $Serial) -AllowFailure
}

Write-Step "Wait for target serial online: $Serial"
$deadline = (Get-Date).AddSeconds($WaitTimeoutSeconds)
$attempt = 0
$lastState = ""

while ((Get-Date) -lt $deadline) {
    $attempt++

    # Reconnect periodically for tcp endpoints.
    if (($attempt -eq 1) -or ($attempt % [Math]::Max([int](30 / $PollIntervalSeconds), 1) -eq 0)) {
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
