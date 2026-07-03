# Build script: ZeroBrowser Android APK
# Usage: .\build.ps1 [debug|release]
#
# gradle-wrapper.jar is COMMITTED to the repository (standard Gradle convention).
# If missing or corrupted, run:  .\bootstrap.ps1  to re-download from official source.
param(
    [string]$Mode = "debug"
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path $PSScriptRoot -Parent

$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
$env:ANDROID_NDK_HOME = "$env:ANDROID_HOME\ndk\30.0.14904198"

$rustTarget = "aarch64-linux-android"
$linker = "$env:ANDROID_NDK_HOME\toolchains\llvm\prebuilt\windows-x86_64\bin\$rustTarget-clang.cmd"
$rustMode = if ($Mode -eq "release") { "--release" } else { "" }
$buildType = if ($Mode -eq "release") { "release" } else { "debug" }
$soName = "libzero_ui_adapter_android.so"
$rustOutDir = "..\..\..\..\target\$rustTarget\$buildType"
$jniLibsDir = "$PSScriptRoot\app\src\main\jniLibs\arm64-v8a"

# Step 1: Build Rust .so
Write-Host "=== Building Rust .so for $rustTarget ($Mode) ==="
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = $linker
Push-Location $projectRoot
try {
    cargo ndk -t arm64-v8a build -p zero-ui-adapter-android $rustMode.Split(' ')
    if ($LASTEXITCODE -ne 0) { throw "cargo ndk failed" }
    Write-Host "cargo ndk OK"
} finally {
    Pop-Location
}

# Step 2: Copy .so to jniLibs
Write-Host "=== Copying .so to jniLibs ==="
New-Item -Path $jniLibsDir -ItemType Directory -Force | Out-Null
$so = Join-Path $projectRoot "target\$rustTarget\$buildType\$soName"
if (-not (Test-Path $so)) {
    $so = Join-Path $projectRoot "..\..\..\target\$rustTarget\$buildType\$soName"
}
Copy-Item $so $jniLibsDir -Force
Write-Host "Copied $so -> $jniLibsDir"

# Step 3: Build APK with gradle
Write-Host "=== Building APK with gradle ==="
$env:ANDROID_SDK_ROOT = $env:ANDROID_HOME
& "$PSScriptRoot\gradlew.bat" assembleDebug
if ($LASTEXITCODE -ne 0) { throw "gradle build failed" }

Write-Host "=== Build SUCCESS ==="
$apk = Get-ChildItem "$PSScriptRoot\app\build\outputs\apk\debug\*.apk" | Select-Object -First 1
Write-Host "APK: $($apk.FullName)"
Write-Host ""
Write-Host "To install and run:"
Write-Host "  adb install -r `"$($apk.FullName)`""
Write-Host "  adb shell am start -n com.zeroweb.ui/.MainActivity"
Write-Host "  adb logcat -s ZeroBrowser:V"
