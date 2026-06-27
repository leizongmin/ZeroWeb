# 以 CPU 渲染模式启动 ZeroBrowser（Windows 版，等价于 make browser-cpu）
#
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts\browser-cpu.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\browser-cpu.ps1 -- --headless
#
# 默认 --wpt-parity（CPU + scale 1.0，与 WPT/product-smoke 肉眼对齐）。
# 显式 --renderer / --scale 可覆盖。
# 会先下载 rusty_v8，再 release 编译 zero-browser + zero-renderer，最后运行已构建的二进制
# （不用 cargo run，确保与 zero-renderer.exe 同目录，多进程模式可 spawn 子进程）。

param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$BrowserArgs
)

$ErrorActionPreference = "Stop"

$ProjectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$DownloadScript = Join-Path $PSScriptRoot "download-rusty-v8.ps1"
$BrowserBin = Join-Path $ProjectRoot "target\release\zero-browser.exe"
$RendererBin = Join-Path $ProjectRoot "target\release\zero-renderer.exe"

Push-Location $ProjectRoot
try {
    Write-Host "ZeroBrowser: preparing rusty_v8..."
    & $DownloadScript
    if ($LASTEXITCODE -ne 0) {
        Write-Error "rusty_v8 setup failed with exit code $LASTEXITCODE"
    }

    Write-Host "ZeroBrowser: building zero-browser and zero-renderer (release)..."
    cargo build --release -p zero-browser -p zero-renderer
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    if (-not (Test-Path -LiteralPath $BrowserBin)) {
        Write-Error "zero-browser not found at $BrowserBin"
    }
    if (-not (Test-Path -LiteralPath $RendererBin)) {
        Write-Error "zero-renderer not found at $RendererBin (required for default multi-process mode)"
    }

    Write-Host "ZeroBrowser: starting $BrowserBin (WPT parity: CPU, scale 1.0)"
    $env:RUST_BACKTRACE = "1"
    $launchArgs = @("--wpt-parity") + $BrowserArgs
    & $BrowserBin @launchArgs
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
