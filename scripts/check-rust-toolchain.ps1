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

    throw "rust toolchain not found. install Rust or provide RUST_BIN_DIR/CARGO_HOME for self-hosted runner"
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
