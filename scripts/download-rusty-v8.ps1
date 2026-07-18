# 下载 rusty_v8 预构建产物（Windows 版，等价于 scripts/download-rusty-v8.sh）
#
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts\download-rusty-v8.ps1
#
# 缓存目录：%LOCALAPPDATA%\zero-web\rusty_v8（可用 RUSTY_V8_CACHE_DIR 覆盖）
# 本地链接：.cargo\rusty_v8\archive（可用 RUSTY_V8_ARCHIVE 覆盖为本地路径）

param()

$ErrorActionPreference = "Stop"

function Remove-FileIfPresent {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return
    }

    try {
        Remove-Item -LiteralPath $Path -Force
    }
    catch {
        Write-Warning "Could not remove locked file $Path; continuing with existing cache."
    }
}

function Test-ValidArchive {
    param(
        [string]$Path,
        [long]$ExpectedSize = 0
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        return $false
    }

    $length = (Get-Item -LiteralPath $Path).Length
    if ($length -le 1MB) {
        return $false
    }

    if ($ExpectedSize -gt 0 -and $length -ne $ExpectedSize) {
        return $false
    }

    return $true
}

function Get-ExpectedArchiveSize {
    param([string]$DownloadUrl)

    $curl = Get-Command curl.exe -ErrorAction SilentlyContinue
    if (-not $curl) {
        return 0
    }

    $headers = & curl.exe -sIL --connect-timeout 10 --max-time 30 $DownloadUrl 2>$null
    if ($LASTEXITCODE -ne 0) {
        return 0
    }

    $contentLength = $headers | Select-String -Pattern '^Content-Length:\s*(\d+)$' | Select-Object -Last 1
    if (-not $contentLength) {
        return 0
    }

    return [long]$contentLength.Matches.Groups[1].Value
}

function Get-RustyV8Version {
    param([string]$CargoLockPath)

    $lines = Get-Content -LiteralPath $CargoLockPath
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -eq 'name = "v8"') {
            if ($lines[$i + 1] -match '^version = "([^"]+)"$') {
                return $Matches[1]
            }
            break
        }
    }

    throw "Failed to determine v8 version from Cargo.lock"
}

function Get-HostTargetTriple {
    $hostLine = rustc -vV | Select-String -Pattern '^host: (.+)$' | Select-Object -First 1
    if (-not $hostLine) {
        throw "Failed to determine host target triple"
    }
    return $hostLine.Matches.Groups[1].Value
}

function Install-LocalArchive {
    param(
        [string]$Source,
        [string]$Destination
    )

    $destDir = Split-Path -Parent $Destination
    if (-not (Test-Path -LiteralPath $destDir)) {
        New-Item -ItemType Directory -Force -Path $destDir | Out-Null
    }

    if (Test-Path -LiteralPath $Destination) {
        Remove-Item -LiteralPath $Destination -Force
    }

    try {
        New-Item -ItemType HardLink -Path $Destination -Target $Source -Force | Out-Null
    }
    catch {
        Copy-Item -LiteralPath $Source -Destination $Destination -Force
    }
}

$ProjectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Write-Host "Checking rusty_v8 archive..."
$localArchive = if ($env:RUSTY_V8_ARCHIVE) {
    $env:RUSTY_V8_ARCHIVE
}
else {
    Join-Path $ProjectRoot ".cargo\rusty_v8\archive"
}

if ($localArchive -match '^https?://') {
    Write-Host "RUSTY_V8_ARCHIVE is a URL; skipping local download."
    exit 0
}

if (-not [System.IO.Path]::IsPathRooted($localArchive)) {
    $localArchive = Join-Path $ProjectRoot $localArchive
}

$version = Get-RustyV8Version -CargoLockPath (Join-Path $ProjectRoot "Cargo.lock")
$targetTriple = if ($env:RUSTY_V8_TARGET) { $env:RUSTY_V8_TARGET } else { Get-HostTargetTriple }

if ($targetTriple -notmatch 'windows') {
    Write-Error "download-rusty-v8.ps1 only supports Windows targets (host: $targetTriple)."
}

$archiveName = "rusty_v8_release_${targetTriple}.lib.gz"

$cacheRoot = if ($env:RUSTY_V8_CACHE_DIR) {
    $env:RUSTY_V8_CACHE_DIR
}
else {
    Join-Path $env:LOCALAPPDATA "zero-web\rusty_v8"
}

$cacheArchive = Join-Path $cacheRoot "v$version\$archiveName"
$cacheTmp = "$cacheArchive.tmp"
$cacheSizeFile = "$cacheArchive.size"

foreach ($dir in @((Split-Path -Parent $cacheArchive), (Split-Path -Parent $localArchive))) {
    if (-not (Test-Path -LiteralPath $dir)) {
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
    }
}

function Get-RecordedArchiveSize {
    param([string]$SizeFilePath)

    if (-not (Test-Path -LiteralPath $SizeFilePath)) {
        return 0
    }

    $raw = (Get-Content -LiteralPath $SizeFilePath -Raw).Trim()
    if ($raw -match '^\d+$') {
        return [long]$raw
    }

    return 0
}

function Set-RecordedArchiveSize {
    param(
        [string]$SizeFilePath,
        [long]$Size
    )

    Set-Content -LiteralPath $SizeFilePath -Value $Size -NoNewline
}

$recordedSize = Get-RecordedArchiveSize -SizeFilePath $cacheSizeFile
if ((Test-Path -LiteralPath $cacheArchive) -and (Test-ValidArchive $cacheArchive $recordedSize)) {
    Install-LocalArchive -Source $cacheArchive -Destination $localArchive
    Write-Host "rusty_v8 archive already cached: $cacheArchive"
    exit 0
}

$baseUrl = if ($env:RUSTY_V8_MIRROR) { $env:RUSTY_V8_MIRROR } else { "https://github.com/denoland/rusty_v8/releases/download" }
$downloadUrl = "$baseUrl/v$version/$archiveName"
Write-Host "Querying release metadata..."
$expectedSize = Get-ExpectedArchiveSize -DownloadUrl $downloadUrl
if ($expectedSize -gt 0) {
    Set-RecordedArchiveSize -SizeFilePath $cacheSizeFile -Size $expectedSize
}

if ((Test-Path -LiteralPath $cacheArchive) -and -not (Test-ValidArchive $cacheArchive $expectedSize)) {
    Write-Host "Removing invalid cached archive $cacheArchive"
    Remove-FileIfPresent $cacheArchive
    Remove-FileIfPresent $cacheSizeFile
}

if ((Test-Path -LiteralPath $cacheTmp) -and -not (Test-ValidArchive $cacheTmp $expectedSize)) {
    Write-Host "Removing invalid partial archive $cacheTmp"
    Remove-FileIfPresent $cacheTmp
}

if (Test-ValidArchive $cacheArchive $expectedSize) {
    Install-LocalArchive -Source $cacheArchive -Destination $localArchive
    Write-Host "rusty_v8 archive already cached: $cacheArchive"
    exit 0
}

if (Test-Path -LiteralPath $cacheTmp) {
    if (Test-ValidArchive $cacheTmp $expectedSize) {
        Write-Host "Resuming download: $downloadUrl"
    }
    else {
        Write-Host "Removing invalid partial archive $cacheTmp"
        Remove-Item -LiteralPath $cacheTmp -Force
        Write-Host "Downloading: $downloadUrl"
    }
}
else {
    Write-Host "Downloading: $downloadUrl"
}

$curl = Get-Command curl.exe -ErrorAction SilentlyContinue
if (-not $curl) {
    throw "curl.exe not found. Install curl or download manually: $downloadUrl"
}

$curlArgs = @("-fL", "--connect-timeout", "10", "--max-time", "3600", "--progress-bar", "-o", $cacheTmp, $downloadUrl)
if (Test-Path -LiteralPath $cacheTmp) {
    $curlArgs = @("-fL", "-C", "-", "--connect-timeout", "10", "--max-time", "3600", "--progress-bar", "-o", $cacheTmp) + $downloadUrl
}

& curl.exe @curlArgs
if ($LASTEXITCODE -ne 0) {
    throw "curl failed with exit code $LASTEXITCODE"
}

if (-not (Test-ValidArchive $cacheTmp $expectedSize)) {
    Remove-Item -LiteralPath $cacheTmp -Force -ErrorAction SilentlyContinue
    throw "Downloaded archive size mismatch (expected $expectedSize bytes). Removed partial file; please retry."
}

Copy-Item -LiteralPath $cacheTmp -Destination $cacheArchive -Force
Remove-FileIfPresent $cacheTmp
Set-RecordedArchiveSize -SizeFilePath $cacheSizeFile -Size $expectedSize
Install-LocalArchive -Source $cacheArchive -Destination $localArchive

Write-Host "rusty_v8 archive ready at $cacheArchive"
