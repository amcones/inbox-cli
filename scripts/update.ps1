param(
    [string]$Version = "latest",
    [string]$BinDir = "",
    [string]$ReleaseBaseUrl = $(if ($env:INBOX_RELEASE_BASE_URL) { $env:INBOX_RELEASE_BASE_URL } else { "https://github.com/amcones/inbox-cli/releases" })
)

$ErrorActionPreference = "Stop"
if ($Version -ne "latest" -and $Version -notmatch '^v\d+\.\d+\.\d+$') {
    throw "-Version must be latest or vMAJOR.MINOR.PATCH"
}
if ([Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne [Runtime.InteropServices.Architecture]::X64) {
    throw "Only Windows x86_64 is currently supported"
}

if ($BinDir) {
    $InstalledBinary = Join-Path $BinDir "inbox.exe"
} else {
    $Command = Get-Command inbox.exe -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $Command) {
        throw "inbox.exe is not on PATH; use -BinDir to identify its installation directory"
    }
    $InstalledBinary = $Command.Source
    $BinDir = Split-Path -Parent $InstalledBinary
}
if (-not (Test-Path -LiteralPath $InstalledBinary -PathType Leaf)) {
    throw "$InstalledBinary is not an installed inbox binary"
}

$Asset = "inbox-windows-x86_64"
$Archive = "$Asset.zip"
$WorkDir = Join-Path ([IO.Path]::GetTempPath()) ("inbox-update-" + [guid]::NewGuid())
$Staged = $null
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
    $RelativeBase = if ($Version -eq "latest") { "latest\download" } else { "download\$Version" }
    Receive-ReleaseFile "$RelativeBase\$Archive" $ArchivePath
    Receive-ReleaseFile "$RelativeBase\$Archive.sha256" $ChecksumPath
    $Expected = ((Get-Content -LiteralPath $ChecksumPath -Raw).Trim() -split '\s+')[0].ToLowerInvariant()
    $Actual = (Get-FileHash -LiteralPath $ArchivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if (-not $Expected -or $Actual -ne $Expected) {
        throw "checksum verification failed"
    }

    Expand-Archive -LiteralPath $ArchivePath -DestinationPath $WorkDir
    $Downloaded = Join-Path $WorkDir "$Asset\inbox.exe"
    if (-not (Test-Path -LiteralPath $Downloaded -PathType Leaf)) {
        throw "release archive does not contain inbox.exe"
    }
    $NewVersion = & $Downloaded --version
    if ($Version -ne "latest" -and $NewVersion -ne "inbox $($Version.Substring(1))") {
        throw "downloaded $NewVersion, expected inbox $($Version.Substring(1))"
    }

    $Staged = "$InstalledBinary.new"
    Copy-Item -LiteralPath $Downloaded -Destination $Staged -Force
    Move-Item -LiteralPath $Staged -Destination $InstalledBinary -Force

    $CompletionSource = Join-Path $WorkDir "$Asset\completion.powershell"
    $CompletionDestination = Join-Path $BinDir "inbox-completion.ps1"
    if ((Test-Path -LiteralPath $CompletionSource -PathType Leaf) -and (Test-Path -LiteralPath $CompletionDestination -PathType Leaf)) {
        Copy-Item -LiteralPath $CompletionSource -Destination $CompletionDestination -Force
    }
    $UpdaterSource = Join-Path $WorkDir "$Asset\update.ps1"
    $UpdaterDestination = Join-Path $BinDir "inbox-update.ps1"
    if ((Test-Path -LiteralPath $UpdaterSource -PathType Leaf) -and (Test-Path -LiteralPath $UpdaterDestination -PathType Leaf)) {
        Copy-Item -LiteralPath $UpdaterSource -Destination $UpdaterDestination -Force
    }

    Write-Output "Updated $InstalledBinary to $NewVersion"
} finally {
    if ($Staged) { Remove-Item -LiteralPath $Staged -Force -ErrorAction SilentlyContinue }
    Remove-Item -LiteralPath $WorkDir -Recurse -Force -ErrorAction SilentlyContinue
}
