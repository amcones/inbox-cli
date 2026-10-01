$ErrorActionPreference = "Stop"
$ProjectDir = Split-Path -Parent $PSScriptRoot
$TestDir = Join-Path ([IO.Path]::GetTempPath()) ("inbox-update-test-" + [guid]::NewGuid())
try {
    $Asset = "inbox-windows-x86_64"
    $ReleaseDir = Join-Path $TestDir "releases\download\v0.6.2"
    $PackageParent = Join-Path $TestDir "package"
    $PackageDir = Join-Path $PackageParent $Asset
    $BinDir = Join-Path $TestDir "bin"
    New-Item -ItemType Directory -Force $ReleaseDir, $PackageDir, $BinDir | Out-Null
    Copy-Item (Join-Path $ProjectDir "target\release\inbox.exe") (Join-Path $PackageDir "inbox.exe")
    Copy-Item (Join-Path $PSScriptRoot "update.ps1") (Join-Path $PackageDir "update.ps1")
    Copy-Item (Join-Path $PSScriptRoot "install.ps1") (Join-Path $PackageDir "install.ps1")
    Copy-Item (Join-Path $PSScriptRoot "completion.powershell") (Join-Path $PackageDir "completion.powershell")
    $Archive = Join-Path $ReleaseDir "$Asset.zip"
    Compress-Archive -Path $PackageDir -DestinationPath $Archive
    $Hash = (Get-FileHash $Archive -Algorithm SHA256).Hash.ToLowerInvariant()
    "$Hash  $Asset.zip" | Set-Content -NoNewline "$Archive.sha256"
    $LatestDir = Join-Path $TestDir "releases\latest\download"
    New-Item -ItemType Directory -Force $LatestDir | Out-Null
    Copy-Item $Archive, "$Archive.sha256" $LatestDir
    New-Item -ItemType File (Join-Path $BinDir "inbox.exe"), (Join-Path $BinDir "inbox-update.ps1"), (Join-Path $BinDir "inbox-completion.ps1") | Out-Null

    $CorrectChecksum = Get-Content "$Archive.sha256" -Raw
    "$('0' * 64)  $Asset.zip" | Set-Content -NoNewline "$Archive.sha256"
    try {
        & (Join-Path $PSScriptRoot "update.ps1") -Version v0.6.2 -BinDir $BinDir -ReleaseBaseUrl (Join-Path $TestDir "releases")
        throw "updater accepted a damaged checksum"
    } catch {
        if ($_.Exception.Message -eq "updater accepted a damaged checksum") { throw }
    }
    if ((Get-Item (Join-Path $BinDir "inbox.exe")).Length -ne 0) { throw "failed verification changed the installed binary" }
    $CorrectChecksum | Set-Content -NoNewline "$Archive.sha256"

    & (Join-Path $PSScriptRoot "update.ps1") -Version v0.6.2 -BinDir $BinDir -ReleaseBaseUrl (Join-Path $TestDir "releases")
    if ((& (Join-Path $BinDir "inbox.exe") --version) -ne "inbox 0.6.2") { throw "binary was not updated" }
    if ((Get-FileHash (Join-Path $BinDir "inbox-update.ps1")).Hash -ne (Get-FileHash (Join-Path $PSScriptRoot "update.ps1")).Hash) { throw "updater was not refreshed" }
    if ((Get-FileHash (Join-Path $BinDir "inbox-completion.ps1")).Hash -ne (Get-FileHash (Join-Path $PSScriptRoot "completion.powershell")).Hash) { throw "completion was not refreshed" }
    $InstallBin = Join-Path $TestDir "install-bin"
    & (Join-Path $PSScriptRoot "install-release.ps1") -BinDir $InstallBin -Shell none -ReleaseBaseUrl (Join-Path $TestDir "releases")
    if ((& (Join-Path $InstallBin "inbox.exe") --version) -ne "inbox 0.6.2") { throw "binary was not installed" }
    Write-Output "Windows updater test passed"
} finally {
    Remove-Item -LiteralPath $TestDir -Recurse -Force -ErrorAction SilentlyContinue
}
