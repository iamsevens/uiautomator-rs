Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

rustc --version
if ($LASTEXITCODE -ne 0) {
    throw "rustc --version failed with exit code $LASTEXITCODE"
}

cargo --version
if ($LASTEXITCODE -ne 0) {
    throw "cargo --version failed with exit code $LASTEXITCODE"
}
