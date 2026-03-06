param(
    [string]$SourceRepoRoot = "D:\dev\uiautomator",

    [Parameter(Mandatory = $true)]
    [string]$DestinationRepoRoot,

    [Parameter(Mandatory = $true)]
    [string]$CommitSha
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Test-Path (Join-Path $SourceRepoRoot ".git"))) {
    throw "source repo is not a git repository: $SourceRepoRoot"
}

$null = & git -C $SourceRepoRoot cat-file -e "$CommitSha^{commit}" 2>$null
if ($LASTEXITCODE -ne 0) {
    throw "commit not found in source repo: $CommitSha"
}

if (Test-Path $DestinationRepoRoot) {
    Get-ChildItem -Force $DestinationRepoRoot -ErrorAction SilentlyContinue |
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
}
else {
    New-Item -ItemType Directory -Path $DestinationRepoRoot | Out-Null
}

$archivePath = Join-Path $env:TEMP ("uiautomator-local-checkout-{0}.zip" -f [Guid]::NewGuid().ToString("N"))

try {
    & git -C $SourceRepoRoot archive --format=zip -o $archivePath $CommitSha
    if ($LASTEXITCODE -ne 0) {
        throw "git archive failed for commit: $CommitSha"
    }

    Expand-Archive -Path $archivePath -DestinationPath $DestinationRepoRoot -Force
}
finally {
    Remove-Item $archivePath -Force -ErrorAction SilentlyContinue
}

[ordered]@{
    source_repo_root = $SourceRepoRoot
    destination_repo_root = $DestinationRepoRoot
    commit_sha = $CommitSha
} | ConvertTo-Json -Depth 4
