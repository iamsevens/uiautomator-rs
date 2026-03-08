# Local build test script for Windows
# UTF-8 encoding

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Testing uiautomator-cli build process" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# Change to project directory
Set-Location $PSScriptRoot

Write-Host ""
Write-Host "Step 1: Check resource files" -ForegroundColor Yellow
if (-not (Test-Path "assets\atx-agent")) {
    Write-Host "[X] Resource files not found, downloading..." -ForegroundColor Red
    & powershell -ExecutionPolicy Bypass -File assets\download_atx_agent.ps1
} else {
    Write-Host "[OK] Resource files exist" -ForegroundColor Green
}

Write-Host ""
Write-Host "Step 2: Run code format check" -ForegroundColor Yellow
$formatResult = cargo fmt -- --check
if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] Code format is correct" -ForegroundColor Green
} else {
    Write-Host "[X] Code format is incorrect, please run 'cargo fmt'" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Step 3: Run Clippy check" -ForegroundColor Yellow
$clippyResult = cargo clippy -- -D warnings
if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] Clippy check passed" -ForegroundColor Green
} else {
    Write-Host "[X] Clippy check failed" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Step 4: Run unit tests" -ForegroundColor Yellow
$testResult = cargo test --lib
if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] Unit tests passed" -ForegroundColor Green
} else {
    Write-Host "[X] Unit tests failed" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Step 5: Run integration tests (excluding device tests)" -ForegroundColor Yellow
$integrationResult = cargo test --test resources_test --test cli_test --test error_test
if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] Integration tests passed" -ForegroundColor Green
} else {
    Write-Host "[X] Integration tests failed" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Step 6: Run property tests" -ForegroundColor Yellow
$propertyResult = cargo test --test property_resources_test --test property_idempotent_test
if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] Property tests passed" -ForegroundColor Green
} else {
    Write-Host "[X] Property tests failed" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Step 7: Build Release version" -ForegroundColor Yellow
$buildResult = cargo build --release
if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] Release build succeeded" -ForegroundColor Green
} else {
    Write-Host "[X] Release build failed" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Step 8: Verify binary file" -ForegroundColor Yellow
if (Test-Path "target\release\uiautomator.exe") {
    Write-Host "[OK] Binary file exists" -ForegroundColor Green
    
    # Test version command
    Write-Host "Testing version command:"
    & .\target\release\uiautomator.exe version
} else {
    Write-Host "[X] Binary file does not exist" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Green
Write-Host "[OK] All checks passed!" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:"
Write-Host "1. If all tests pass, you can commit the code"
Write-Host "2. Create a tag to release a new version:"
Write-Host "   git tag vX.Y.Z"
Write-Host "   git push origin vX.Y.Z"
Write-Host "3. GitHub Actions will automatically build and release"

