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
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot "completion.powershell") -Destination $CompletionPath -Force
    Write-Output "Installed PowerShell completion to $CompletionPath"

    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $UserEntries = @($UserPath -split [IO.Path]::PathSeparator | Where-Object { $_ })
    if ($BinDir -notin $UserEntries) {
        [Environment]::SetEnvironmentVariable("Path", (($UserEntries + $BinDir) -join [IO.Path]::PathSeparator), "User")
        Write-Output "Added $BinDir to the user PATH. Open a new PowerShell session to use it."
    }
    $ProfilePath = $PROFILE
    $ProfileDir = Split-Path -Parent $ProfilePath
    New-Item -ItemType Directory -Force -Path $ProfileDir | Out-Null
    $EscapedCompletion = $CompletionPath.Replace("'", "''")
    $ProfileLine = ". '$EscapedCompletion'"
    if (-not (Test-Path -LiteralPath $ProfilePath -PathType Leaf) -or -not (Select-String -LiteralPath $ProfilePath -SimpleMatch -Pattern $ProfileLine -Quiet)) {
        Add-Content -LiteralPath $ProfilePath -Value $ProfileLine
        Write-Output "Updated PowerShell profile $ProfilePath"
    }
}

Write-Output "Installed inbox to $InstalledBinary"
