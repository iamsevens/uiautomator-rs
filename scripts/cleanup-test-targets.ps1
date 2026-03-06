param(
    [int[]]$LdplayerIndexes = @(0),

    [int[]]$MumuIndexes = @(0),

    [string]$LdconsolePath = "D:\leidian\LDPlayer9\ldconsole.exe",

    [string]$MumuManagerPath = "C:\Program Files\Netease\MuMu\nx_main\MuMuManager.exe",

    [string]$MumuAdbEndpoints = "127.0.0.1:16384,127.0.0.1:16385,127.0.0.1:5555"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Invoke-ExternalCommand {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FilePath,

        [Parameter(Mandatory = $true)]
        [string[]]$Arguments,

        [switch]$AllowFailure
    )

    $nativeErrorPrefVar = Get-Variable -Name PSNativeCommandUseErrorActionPreference -ErrorAction SilentlyContinue
    $prevNativeErrorPref = $null
    if ($null -ne $nativeErrorPrefVar) {
        $prevNativeErrorPref = [bool]$nativeErrorPrefVar.Value
        $script:PSNativeCommandUseErrorActionPreference = $false
    }

    try {
        $output = & $FilePath @Arguments 2>&1
        $code = $LASTEXITCODE
        $text = ($output | ForEach-Object { $_.ToString() }) -join "`n"
    }
    catch {
        if ($AllowFailure) {
            Write-Host "::warning::$FilePath $($Arguments -join ' ') threw: $($_.Exception.Message)"
            return
        }
        throw
    }
    finally {
        if ($null -ne $nativeErrorPrefVar) {
            $script:PSNativeCommandUseErrorActionPreference = $prevNativeErrorPref
        }
    }

    if ($code -ne 0) {
        if ($AllowFailure) {
            Write-Host "::warning::$FilePath $($Arguments -join ' ') failed (exit=$code): $text"
            return
        }
        throw "$FilePath $($Arguments -join ' ') failed (exit=$code): $text"
    }

    if (-not [string]::IsNullOrWhiteSpace($text)) {
        Write-Host $text.Trim()
    }
}

Write-Host "==> cleanup test targets"

if (Test-Path $LdconsolePath) {
    foreach ($index in ($LdplayerIndexes | Select-Object -Unique)) {
        Write-Host "[LDPlayer] quit --index $index"
        Invoke-ExternalCommand -FilePath $LdconsolePath -Arguments @("quit", "--index", [string]$index) -AllowFailure
    }
}
else {
    Write-Host "::warning::ldconsole not found: $LdconsolePath"
}

if (Test-Path $MumuManagerPath) {
    foreach ($index in ($MumuIndexes | Select-Object -Unique)) {
        Write-Host "[MuMu] control -v $index shutdown"
        Invoke-ExternalCommand -FilePath $MumuManagerPath -Arguments @("control", "-v", [string]$index, "shutdown") -AllowFailure
    }
}
else {
    Write-Host "::warning::MuMuManager not found: $MumuManagerPath"
}

Invoke-ExternalCommand -FilePath "adb" -Arguments @("start-server") -AllowFailure

$endpoints = $MumuAdbEndpoints.Split(",") |
    ForEach-Object { $_.Trim() } |
    Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
    Select-Object -Unique

foreach ($endpoint in $endpoints) {
    Write-Host "[adb] disconnect $endpoint"
    Invoke-ExternalCommand -FilePath "adb" -Arguments @("disconnect", $endpoint) -AllowFailure
}

Write-Host ""
Write-Host "==> adb devices"
Invoke-ExternalCommand -FilePath "adb" -Arguments @("devices") -AllowFailure
