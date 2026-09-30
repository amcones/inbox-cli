param(
    [string]$Target
)

$ErrorActionPreference = "Stop"
$ProjectDir = Split-Path -Parent $PSScriptRoot
Push-Location $ProjectDir
try {
    $BuildArgs = @("build", "--release", "--locked")
    if ($Target) {
        $BuildArgs += @("--target", $Target)
    }
    & cargo @BuildArgs
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }
    if ($Target) {
        $Executable = if ($Target -match "windows") { "inbox.exe" } else { "inbox" }
        Join-Path $ProjectDir "target\$Target\release\$Executable"
    } else {
        Join-Path $ProjectDir "target\release\inbox.exe"
    }
} finally {
    Pop-Location
}
