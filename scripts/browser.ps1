# 启动 ZeroBrowser（Windows 版，等价于 make browser）
#
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts\browser.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\browser.ps1 -- --headless
#
# 会先下载 rusty_v8 预构建产物，再执行 cargo run --release -p zero-browser。

param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$BrowserArgs
)

$ErrorActionPreference = "Stop"

$ProjectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$DownloadScript = Join-Path $PSScriptRoot "download-rusty-v8.ps1"

Push-Location $ProjectRoot
try {
    Write-Host "ZeroBrowser: preparing rusty_v8..."
    & $DownloadScript
    if ($LASTEXITCODE -ne 0) {
        Write-Error "rusty_v8 setup failed with exit code $LASTEXITCODE"
    }

    Write-Host "ZeroBrowser: building and starting (release, first run may take several minutes)..."
    if ($BrowserArgs.Count -gt 0) {
        cargo run --release -p zero-browser -- @BrowserArgs
    }
    else {
        cargo run --release -p zero-browser
    }
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
