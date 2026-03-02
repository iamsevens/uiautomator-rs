Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Add-PathEntry {
    param([Parameter(Mandatory = $true)][string]$PathEntry)

    if (-not (Test-Path $PathEntry)) {
        return $false
    }

    $normalized = [System.IO.Path]::GetFullPath($PathEntry.TrimEnd('\'))
    $segments = @($env:PATH -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })

    foreach ($segment in $segments) {
        try {
            if ([string]::Equals([System.IO.Path]::GetFullPath($segment.TrimEnd('\')), $normalized, [System.StringComparison]::OrdinalIgnoreCase)) {
                return $true
            }
        }
        catch {
            # ignore malformed PATH segments
        }
    }

    $env:PATH = "$normalized;$env:PATH"
    if (-not [string]::IsNullOrWhiteSpace($env:GITHUB_PATH)) {
        $normalized | Out-File -FilePath $env:GITHUB_PATH -Append -Encoding utf8
    }
    Write-Host "added Rust path: $normalized"
    return $true
}

function Convert-HttpContentToText {
    param([Parameter(Mandatory = $true)]$Content)

    if ($Content -is [byte[]]) {
        return [System.Text.Encoding]::UTF8.GetString($Content)
    }

    return [string]$Content
}

function Get-ExpectedSha256FromUrl {
    param([Parameter(Mandatory = $true)][string]$Sha256Url)

    $response = Invoke-WebRequest -Uri $Sha256Url -UseBasicParsing
    $text = Convert-HttpContentToText -Content $response.Content
    $line = ($text -split "`r?`n" |
        ForEach-Object { $_.Trim() } |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
        Select-Object -First 1)

    if ([string]::IsNullOrWhiteSpace($line)) {
        throw "checksum response is empty: $Sha256Url"
    }

    $sha = ($line -split '\s+')[0].Trim().ToLowerInvariant()
    if ($sha -notmatch '^[0-9a-f]{64}$') {
        throw "invalid sha256 format from $Sha256Url : $line"
    }

    return $sha
}

function Assert-FileSha256 {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter(Mandatory = $true)][string]$ExpectedSha256,
        [Parameter(Mandatory = $true)][string]$Label
    )

    $actual = (Get-FileHash -Path $FilePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $ExpectedSha256.ToLowerInvariant()) {
        throw "$Label checksum mismatch. expected=$ExpectedSha256 actual=$actual file=$FilePath"
    }

    Write-Host "$Label checksum verified: $actual"
}

function Resolve-RustToolchain {
    $rustc = Get-Command rustc -ErrorAction SilentlyContinue
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if ($null -ne $rustc -and $null -ne $cargo) {
        return
    }

    $candidates = New-Object System.Collections.Generic.List[string]

    if (-not [string]::IsNullOrWhiteSpace($env:RUST_BIN_DIR)) {
        $candidates.Add($env:RUST_BIN_DIR)
    }
    if (-not [string]::IsNullOrWhiteSpace($env:CARGO_HOME)) {
        $candidates.Add((Join-Path $env:CARGO_HOME "bin"))
    }
    if (-not [string]::IsNullOrWhiteSpace($env:USERPROFILE)) {
        $candidates.Add((Join-Path $env:USERPROFILE ".cargo\\bin"))
    }

    $userRoot = Join-Path $env:SystemDrive "Users"
    if (Test-Path $userRoot) {
        $userDirs = Get-ChildItem -Path $userRoot -Directory -ErrorAction SilentlyContinue
        foreach ($dir in $userDirs) {
            $bin = Join-Path $dir.FullName ".cargo\\bin"
            try {
                if (Test-Path (Join-Path $bin "rustc.exe") -ErrorAction Stop) {
                    $candidates.Add($bin)
                }
            }
            catch {
                # Ignore access-denied directories under C:\Users for service accounts.
                continue
            }
        }
    }

    foreach ($candidate in ($candidates | Select-Object -Unique)) {
        if (Add-PathEntry -PathEntry $candidate) {
            $rustc = Get-Command rustc -ErrorAction SilentlyContinue
            $cargo = Get-Command cargo -ErrorAction SilentlyContinue
            if ($null -ne $rustc -and $null -ne $cargo) {
                return
            }
        }
    }

    Install-RustToolchain

    $rustc = Get-Command rustc -ErrorAction SilentlyContinue
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if ($null -eq $rustc -or $null -eq $cargo) {
        throw "rust toolchain bootstrap failed. set RUST_BIN_DIR/CARGO_HOME or install Rust for the runner account"
    }
}

function Install-RustToolchain {
    Write-Host "rust toolchain not found in PATH; bootstrapping stable toolchain..."

    $baseDir = if (-not [string]::IsNullOrWhiteSpace($env:UIAUTOMATOR_RUST_HOME)) {
        $env:UIAUTOMATOR_RUST_HOME
    }
    elseif (-not [string]::IsNullOrWhiteSpace($env:RUNNER_TEMP)) {
        Join-Path $env:RUNNER_TEMP "uiautomator-rust"
    }
    else {
        Join-Path $env:TEMP "uiautomator-rust"
    }

    $cargoHome = Join-Path $baseDir "cargo"
    $rustupHome = Join-Path $baseDir "rustup"
    $cargoBin = Join-Path $cargoHome "bin"
    $rustupInit = Join-Path $baseDir "rustup-init.exe"
    $rustupUrl = "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
    $rustupSha256Url = "$rustupUrl.sha256"

    New-Item -ItemType Directory -Path $baseDir -Force | Out-Null
    New-Item -ItemType Directory -Path $cargoHome -Force | Out-Null
    New-Item -ItemType Directory -Path $rustupHome -Force | Out-Null

    $env:CARGO_HOME = $cargoHome
    $env:RUSTUP_HOME = $rustupHome

    if (-not [string]::IsNullOrWhiteSpace($env:GITHUB_ENV)) {
        "CARGO_HOME=$cargoHome" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
        "RUSTUP_HOME=$rustupHome" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8
    }

    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $expectedSha256 = Get-ExpectedSha256FromUrl -Sha256Url $rustupSha256Url
    $needsDownload = -not (Test-Path $rustupInit)

    if (-not $needsDownload) {
        try {
            Assert-FileSha256 -FilePath $rustupInit -ExpectedSha256 $expectedSha256 -Label "cached rustup-init.exe"
        }
        catch {
            Write-Host "cached rustup-init.exe failed checksum validation, redownloading."
            Remove-Item -Path $rustupInit -Force -ErrorAction SilentlyContinue
            $needsDownload = $true
        }
    }

    if ($needsDownload) {
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupInit -UseBasicParsing
        Assert-FileSha256 -FilePath $rustupInit -ExpectedSha256 $expectedSha256 -Label "downloaded rustup-init.exe"
    }

    & $rustupInit -y --default-toolchain stable --profile minimal --no-modify-path
    if ($LASTEXITCODE -ne 0) {
        throw "rustup-init failed with exit code $LASTEXITCODE"
    }

    $null = Add-PathEntry -PathEntry $cargoBin
}

Resolve-RustToolchain

rustc --version
if ($LASTEXITCODE -ne 0) {
    throw "rustc --version failed with exit code $LASTEXITCODE"
}

cargo --version
if ($LASTEXITCODE -ne 0) {
    throw "cargo --version failed with exit code $LASTEXITCODE"
}
