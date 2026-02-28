param(
    [switch]$AllowDirty
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$forbiddenRegex = "(?i)(^|/)(TASK_.*\.md|.*_REPORT\.md|.*_SUMMARY\.md|MANUAL_TEST_.*\.md)$"

$crates = @(
    @{
        Name = "uiautomator"
        Path = "uiautomator"
        Required = @(
            "Cargo.toml",
            "README.md",
            "THIRD_PARTY_NOTICES.md",
            "build.rs",
            "src/lib.rs",
            "assets/u2.jar"
        )
    },
    @{
        Name = "uiautomator-cli"
        Path = "uiautomator-cli"
        Required = @(
            "Cargo.toml",
            "README.md",
            "CHANGELOG.md",
            "THIRD_PARTY_NOTICES.md",
            "build.rs",
            "src/main.rs",
            "assets/atx-agent",
            "assets/app-uiautomator.apk",
            "assets/app-uiautomator-test.apk"
        )
    }
)

foreach ($crate in $crates) {
    Write-Host ""
    Write-Host "==> Checking package list for $($crate.Name)"

    Push-Location $crate.Path
    try {
        $args = @("package")
        if ($AllowDirty) {
            $args += "--allow-dirty"
        }
        $args += "--list"

        $rawOutput = & cargo @args
        if ($LASTEXITCODE -ne 0) {
            throw "cargo package --list failed for $($crate.Name)"
        }

        $packageList = @($rawOutput | ForEach-Object { $_.Trim().Replace("\", "/") } | Where-Object {
                $_ -and -not $_.StartsWith("warning:")
            })

        $forbidden = @($packageList | Where-Object { $_ -match $forbiddenRegex })
        if ($forbidden.Count -gt 0) {
            throw "Forbidden files found in $($crate.Name) package:`n$($forbidden -join "`n")"
        }

        $nonReleaseContent = @($packageList | Where-Object {
                $_ -match "^(tests|examples)/"
            })
        if ($nonReleaseContent.Count -gt 0) {
            throw "Non-release content found in $($crate.Name) package:`n$($nonReleaseContent -join "`n")"
        }

        foreach ($required in $crate.Required) {
            if ($packageList -notcontains $required) {
                throw "Required file missing from $($crate.Name) package: $required"
            }
        }

        Write-Host "OK: $($crate.Name) package list passed."
    }
    finally {
        Pop-Location
    }
}

Write-Host ""
Write-Host "All package-list checks passed."
