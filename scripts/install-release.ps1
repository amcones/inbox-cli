param(
    [string]$BinDir = $(if ($env:INBOX_BIN_DIR) { $env:INBOX_BIN_DIR } else { Join-Path $env:LOCALAPPDATA "Programs\inbox" }),
    [ValidateSet("powershell", "none")]
    [string]$Shell = "powershell",
    [string]$ReleaseBaseUrl = $(if ($env:INBOX_RELEASE_BASE_URL) { $env:INBOX_RELEASE_BASE_URL } else { "https://github.com/amcones/inbox-cli/releases" })
)

$ErrorActionPreference = "Stop"
if ([Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne [Runtime.InteropServices.Architecture]::X64) {
    throw "Only Windows x86_64 is currently supported"
}
$Asset = "inbox-windows-x86_64"
$Archive = "$Asset.zip"
$WorkDir = Join-Path ([IO.Path]::GetTempPath()) ("inbox-install-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $WorkDir | Out-Null
try {
    function Receive-ReleaseFile([string]$RelativePath, [string]$Destination) {
        if (Test-Path -LiteralPath $ReleaseBaseUrl -PathType Container) {
            Copy-Item -LiteralPath (Join-Path $ReleaseBaseUrl $RelativePath) -Destination $Destination
        } else {
            Invoke-WebRequest -Uri "$ReleaseBaseUrl/$($RelativePath.Replace('\', '/'))" -OutFile $Destination
        }
    }
    $ArchivePath = Join-Path $WorkDir $Archive
    $ChecksumPath = "$ArchivePath.sha256"
    Receive-ReleaseFile "latest\download\$Archive" $ArchivePath
    Receive-ReleaseFile "latest\download\$Archive.sha256" $ChecksumPath
    $Expected = ((Get-Content -LiteralPath $ChecksumPath -Raw).Trim() -split '\s+')[0].ToLowerInvariant()
    $Actual = (Get-FileHash -LiteralPath $ArchivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if (-not $Expected -or $Actual -ne $Expected) { throw "checksum verification failed" }
    Expand-Archive -LiteralPath $ArchivePath -DestinationPath $WorkDir
    $PackageDir = Join-Path $WorkDir $Asset
    if (-not (Test-Path -LiteralPath (Join-Path $PackageDir "install.ps1") -PathType Leaf)) {
        throw "release archive is incomplete"
    }
    & (Join-Path $PackageDir "install.ps1") -PackageDir $PackageDir -BinDir $BinDir -Shell $Shell
} finally {
    Remove-Item -LiteralPath $WorkDir -Recurse -Force -ErrorAction SilentlyContinue
}
