[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$SourceRoot,
    [Parameter(Mandatory = $true)]
    [string]$OutputDirectory,
    [Parameter(Mandatory = $true)]
    [ValidateSet("x86_64", "arm64-v8a")]
    [string]$Abi,
    [string]$Distro = $(if ($env:ZERO_ANDROID_WSL_DISTRO) { $env:ZERO_ANDROID_WSL_DISTRO } else { "Debian" })
)

$ErrorActionPreference = "Stop"

foreach ($name in "ZERO_V8_SOURCE", "ZERO_CHROMIUM_CLANG", "ZERO_ANDROID_NDK", "LIBCLANG_PATH", "ZERO_V8_GN", "ZERO_V8_NINJA") {
    if ([string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable($name))) {
        throw "$name must be set to the corresponding Linux WSL path."
    }
}

$env:ZERO_ANDROID_WSL_SOURCE_ROOT = [System.IO.Path]::GetFullPath($SourceRoot)
$env:ZERO_ANDROID_WSL_OUTPUT_DIRECTORY = [System.IO.Path]::GetFullPath($OutputDirectory)
$env:ZERO_ANDROID_WSL_SCRIPT = Join-Path $PSScriptRoot "build-native-wsl.sh"
$env:ZERO_ANDROID_WSL_ABI = $Abi

$forwarded = @("ZERO_V8_SOURCE", "ZERO_CHROMIUM_CLANG", "ZERO_ANDROID_NDK", "LIBCLANG_PATH", "ZERO_V8_GN", "ZERO_V8_NINJA", "ZERO_ANDROID_WSL_TARGET_DIR", "ZERO_ANDROID_WSL_ABI")
$convertedPaths = @("ZERO_ANDROID_WSL_SOURCE_ROOT/p", "ZERO_ANDROID_WSL_OUTPUT_DIRECTORY/p", "ZERO_ANDROID_WSL_SCRIPT/p")
$managed = @($forwarded + $convertedPaths | ForEach-Object { ($_ -split "/")[0] })
$existing = @($env:WSLENV -split ":" | Where-Object {
    -not [string]::IsNullOrWhiteSpace($_) -and $managed -notcontains (($_ -split "/")[0])
})
$env:WSLENV = (@($existing + $forwarded + $convertedPaths) | Select-Object -Unique) -join ":"

& wsl.exe -d $Distro -- bash -lc 'tr -d "\r" < "$ZERO_ANDROID_WSL_SCRIPT" | bash -s -- "$ZERO_ANDROID_WSL_SOURCE_ROOT" "$ZERO_ANDROID_WSL_OUTPUT_DIRECTORY" "$ZERO_ANDROID_WSL_ABI"'
exit $LASTEXITCODE
