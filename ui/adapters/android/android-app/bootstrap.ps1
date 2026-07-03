# Bootstrap: download gradle-wrapper.jar from official Gradle GitHub release.
# Run once if gradle/wrapper/gradle-wrapper.jar is missing or corrupted.
#
# The jar is COMMITTED to the repository (43 KB binary, standard Gradle convention).
# This script is a fallback for re-downloading.

$ErrorActionPreference = "Stop"
$projectDir = Split-Path $PSScriptRoot -Parent
$wrapperDir = "$projectDir\gradle\wrapper"
$jarPath = "$wrapperDir\gradle-wrapper.jar"

if (Test-Path $jarPath) {
    $size = (Get-Item $jarPath).Length
    if ($size -gt 40000) {
        Write-Host "gradle-wrapper.jar already exists ($size bytes), skipping download."
        exit 0
    }
    Write-Host "gradle-wrapper.jar exists but is too small ($size bytes), re-downloading..."
}

# Official source: Gradle GitHub release (v8.12.0)
$url = "https://raw.githubusercontent.com/gradle/gradle/v8.12.0/gradle/wrapper/gradle-wrapper.jar"

Write-Host "Downloading gradle-wrapper.jar from $url ..."
New-Item -Path $wrapperDir -ItemType Directory -Force | Out-Null
Invoke-WebRequest -Uri $url -OutFile $jarPath
Write-Host "Downloaded OK ($((Get-Item $jarPath).Length) bytes)"
