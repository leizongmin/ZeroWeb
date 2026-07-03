# Build script: cross-compile Rust .so + assemble HAP
# Usage: .\build.ps1 [debug|release]
#
# Prerequisites:
#   - DevEco Studio installed
#   - Rust target aarch64-unknown-linux-ohos installed
#   - cargo-ndk not required (uses plain cargo build)
param(
    [string]$Mode = "debug"
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path $PSScriptRoot -Parent

# ── Tools detection ───────────────────────────────────────────────────────

$devEcoHome = "C:\Program Files\Huawei\DevEco Studio"
$nodeHome = "$devEcoHome\tools\node"
$hvigorBin = "$devEcoHome\tools\hvigor\bin"
$hvigorJs = "$hvigorBin\hvigorw.js"
$nodeExe = "$nodeHome\node.exe"

if (-not (Test-Path $hvigorJs)) {
    throw "hvigor not found at $hvigorJs. Install DevEco Studio."
}
if (-not (Test-Path $nodeExe)) {
    $nodeExe = "node.exe"  # fallback to PATH
}

# ── Build config ──────────────────────────────────────────────────────────

$buildMode = if ($Mode -eq "release") { "release" } else { "debug" }
$productName = "default"
$target = "aarch64-unknown-linux-ohos"
$soName = "libzero_ui_adapter_harmonyos.so"
$rustOutDir = "$projectRoot\..\..\..\..\target\$target\$buildMode"
$libsDir = "$PSScriptRoot\entry\libs\arm64-v8a"

# ── OHOS cross-compile linker ─────────────────────────────────────────────

$sdkNative = "C:\Program Files\Huawei\DevEco Studio\sdk"
$llvmCandidate = Get-ChildItem "$sdkNative\default\openharmony\native\llvm" -Directory -ErrorAction SilentlyContinue | Select-Object -First 1
if ($llvmCandidate) {
    $linker = "$($llvmCandidate.FullName)\bin\clang.exe"
} else {
    $linker = "clang.exe"
}
Write-Host "Linker: $linker" -ForegroundColor DarkGray

# Step 1: Build Rust .so
Write-Host "=== Building Rust .so for $target ($Mode) ===" -ForegroundColor Green
$env:CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_LINKER = $linker
Push-Location $projectRoot\..\..\..\..
try {
    cargo build --target $target -p zero-ui-adapter-harmonyos @(if ($Mode -eq "release") { "--release" })
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
    Write-Host "cargo build OK"
} finally {
    Pop-Location
}

# Step 2: Copy .so to libs
Write-Host "=== Copying .so to entry/libs ===" -ForegroundColor Green
New-Item -Path $libsDir -ItemType Directory -Force | Out-Null
$so = Join-Path $rustOutDir $soName
Copy-Item $so $libsDir -Force
Write-Host "Copied -> $libsDir"

# Step 3: Build HAP via hvigor
Write-Host "=== Building HAP with hvigor ($Mode) ===" -ForegroundColor Green

# hvigorw wrapper needs node, uses project's hvigorfile.ts
$env:NODE_HOME = $nodeHome
$hvigorArgs = @(
    "--mode", "module",
    "-p", "product=$productName",
    "-p", "buildMode=$buildMode",
    "assembleHap"
)

Push-Location $PSScriptRoot
try {
    & $nodeExe $hvigorJs @hvigorArgs
    if ($LASTEXITCODE -ne 0) {
        # hvigor may fail on first run (SDK download). Retry once.
        Write-Host "hvigor exited with $LASTEXITCODE, retrying..." -ForegroundColor Yellow
        & $nodeExe $hvigorJs @hvigorArgs
        if ($LASTEXITCODE -ne 0) { throw "hvigor build failed" }
    }
} finally {
    Pop-Location
}

Write-Host "=== Build SUCCESS ===" -ForegroundColor Green
$hap = Get-ChildItem "$PSScriptRoot\entry\build\$buildMode\outputs\$buildMode\*.hap" | Select-Object -First 1
if ($hap) {
    Write-Host "HAP: $($hap.FullName)"
    Write-Host ""
    Write-Host "To install and run:"
    Write-Host "  hdc install `"$($hap.FullName)`""
    Write-Host "  hdc shell aa start -a EntryAbility -b com.zeroweb.ui"
} else {
    Write-Host "(HAP file not found; check entry/build/*/outputs/*/ for the .hap)"
}
