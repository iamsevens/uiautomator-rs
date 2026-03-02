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
    if (-not (Test-Path $rustupInit)) {
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInit -UseBasicParsing
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
