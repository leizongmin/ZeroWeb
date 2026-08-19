[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$ApkPath,
    [switch]$RequireRendererLinked
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($env:ANDROID_HOME)) {
    throw "ANDROID_HOME must point to an Android SDK installation."
}

$adb = Join-Path $env:ANDROID_HOME "platform-tools\adb.exe"
if (-not (Test-Path -LiteralPath $adb)) {
    throw "Android platform tools are missing under ANDROID_HOME."
}
if (-not (Test-Path -LiteralPath $ApkPath)) {
    throw "APK does not exist: $ApkPath"
}
if ($RequireRendererLinked) {
    $apkEntries = & tar -tf $ApkPath
    if (-not ($apkEntries | Select-String "lib/x86_64/libc\+\+_shared\.so")) {
        throw "Renderer-enabled APK must package libc++_shared.so."
    }
}

& $adb wait-for-device
& $adb logcat -c
& $adb install -r $ApkPath
& $adb shell am start -W -n com.leizm.zeroweb/.MainActivity
Start-Sleep -Seconds 2

$processes = & $adb shell ps -A -o USER,NAME | Select-String "com.leizm.zeroweb"
$browser = $processes | Where-Object { $_.Line -match "\scom\.leizm\.zeroweb$" } | Select-Object -First 1
$renderer = $processes | Where-Object { $_.Line -match "RendererService0$" } | Select-Object -First 1
$compositor = $processes | Where-Object { $_.Line -match "com\.leizm\.zeroweb:compositor$" } | Select-Object -First 1
$decoder = $processes | Where-Object { $_.Line -match "ImageDecoderService$" } | Select-Object -First 1

if (-not $browser -or -not $renderer -or -not $compositor -or -not $decoder) {
    throw "Expected browser, renderer, compositor, and image-decoder process roles."
}

$browserUid = ($browser.Line -split "\s+")[0]
$rendererUid = ($renderer.Line -split "\s+")[0]
$compositorUid = ($compositor.Line -split "\s+")[0]
$decoderUid = ($decoder.Line -split "\s+")[0]
if ($rendererUid -eq $browserUid -or $decoderUid -eq $browserUid -or $compositorUid -ne $browserUid) {
    throw "Android process UID isolation does not match the required browser/renderer/compositor/decoder topology."
}

$processes | ForEach-Object { Write-Output $_.Line }

$probes = & $adb logcat -d -t 500
foreach ($probe in "decoder probe succeeded", "compositor probe succeeded") {
    if (-not ($probes | Select-String $probe)) {
        throw "Android socket probe did not report success: $probe"
    }
}
if ($RequireRendererLinked -and -not ($probes | Select-String "renderer socket connected")) {
    throw "Renderer-enabled APK did not connect its native renderer socket."
}
