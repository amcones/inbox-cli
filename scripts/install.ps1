param(
    [string]$BinDir = $(if ($env:INBOX_BIN_DIR) { $env:INBOX_BIN_DIR } else { Join-Path $env:LOCALAPPDATA "Programs\inbox" }),
    [ValidateSet("powershell", "none")]
    [string]$Shell = "powershell",
    [switch]$NoBuild
)

$ErrorActionPreference = "Stop"
$ProjectDir = Split-Path -Parent $PSScriptRoot
$Binary = Join-Path $ProjectDir "target\release\inbox.exe"

if (-not $NoBuild) {
    & (Join-Path $PSScriptRoot "build.ps1") | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "build.ps1 failed with exit code $LASTEXITCODE"
    }
} elseif (-not (Test-Path -LiteralPath $Binary -PathType Leaf)) {
    throw "$Binary does not exist; run without -NoBuild"
}

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
$InstalledBinary = Join-Path $BinDir "inbox.exe"
Copy-Item -LiteralPath $Binary -Destination $InstalledBinary -Force

if ($Shell -eq "powershell") {
    $CompletionPath = Join-Path $BinDir "inbox-completion.ps1"
    & $InstalledBinary completions powershell | Set-Content -Encoding utf8 $CompletionPath
    Write-Output "Installed PowerShell completion to $CompletionPath"
}

Write-Output "Installed inbox to $InstalledBinary"
$PathEntries = $env:PATH -split [IO.Path]::PathSeparator
if ($BinDir -notin $PathEntries) {
    Write-Output "Add $BinDir to PATH before using inbox."
}
