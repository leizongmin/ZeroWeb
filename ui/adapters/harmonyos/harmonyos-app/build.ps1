# Build script: cross-compile Rust .so for HarmonyOS (aarch64-unknown-linux-ohos)
# then copy to entry/libs/arm64-v8a/.
# The HAP itself must be built in DevEco Studio (File → Open → this project directory).
param(
    [string]$Mode = "debug"
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path $PSScriptRoot -Parent

$target = "aarch64-unknown-linux-ohos"
$ohosRoot = "$env:LOCALAPPDATA\Huawei\DevEcoStudio6.1"
$sdkNative = "$ohosRoot\sdk\default\hms\native"
$rustMode = if ($Mode -eq "release") { "--release" } else { "" }
$buildType = if ($Mode -eq "release") { "release" } else { "debug" }
$soName = "libzero_ui_adapter_harmonyos.so"
$rustOutDir = "$projectRoot\..\..\..\..\target\$target\$buildType"
$libsDir = "$PSScriptRoot\entry\libs\arm64-v8a"

# Find OHOS linker
$llvmDir = Get-ChildItem "$sdkNative\llvm" -Directory -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $llvmDir) {
    $llvmDir = Get-ChildItem "$ohosRoot\sdk\*\native\llvm" -Directory -ErrorAction SilentlyContinue | Select-Object -First 1
}
if ($llvmDir) {
    $linker = "$($llvmDir.FullName)\bin\clang.exe"
    Write-Host "Using linker: $linker"
} else {
    Write-Host "LLVM not found, assuming linker is in PATH"
    $linker = "clang.exe"
}

# Step 1: Build Rust .so
Write-Host "=== Building Rust .so for $target ($Mode) ===" -ForegroundColor Green
$env:CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_LINKER = $linker
Push-Location $projectRoot\..\..\..\..
try {
    cargo build --target $target -p zero-ui-adapter-harmonyos $rustMode.Split(' ')
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
    Write-Host "cargo build OK"
} finally {
    Pop-Location
}

# Step 2: Copy .so to libs
Write-Host "=== Copying .so to entry/libs ===" -ForegroundColor Green
New-Item -Path $libsDir -ItemType Directory -Force | Out-Null
$so = Join-Path $projectRoot "..\..\..\..\target\$target\$buildType\$soName"
Copy-Item $so $libsDir -Force
Write-Host "Copied $so -> $libsDir"
Write-Host ""
Write-Host "=== Done ===" -ForegroundColor Green
Write-Host "Now open this directory in DevEco Studio:"
Write-Host "  File -> Open -> $PSScriptRoot"
Write-Host "Then Build -> Build Hap(s)"
