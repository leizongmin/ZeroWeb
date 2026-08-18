[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

function Require-Command([string]$Name) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Missing required command: $Name"
    }
}

Require-Command cargo
Require-Command cargo-ndk

if ([string]::IsNullOrWhiteSpace($env:ANDROID_HOME)) {
    throw "ANDROID_HOME must point to an Android SDK installation."
}

$sdkRoot = [System.IO.Path]::GetFullPath($env:ANDROID_HOME)
$adb = Join-Path $sdkRoot "platform-tools\adb.exe"
if (-not (Test-Path -LiteralPath $adb)) {
    throw "Android platform tools are missing under ANDROID_HOME."
}

$platform = Join-Path $sdkRoot "platforms\android-36"
if (-not (Test-Path -LiteralPath $platform)) {
    throw "Android SDK platform android-36 is required."
}

$ndkR30 = Get-ChildItem (Join-Path $sdkRoot "ndk") -Directory -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -like "30.*" } |
    Select-Object -First 1
if (-not $ndkR30) {
    throw "Android NDK r30 is required."
}

$targets = rustup target list --installed
foreach ($target in "aarch64-linux-android", "x86_64-linux-android") {
    if ($targets -notcontains $target) {
        throw "Missing Rust target: $target"
    }
}

Write-Output "Android preflight passed: SDK platform 36, NDK r30, and Rust arm64/x86_64 targets are available."
