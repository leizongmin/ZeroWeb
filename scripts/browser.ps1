# 启动 ZeroBrowser（Windows 版，等价于 make browser）
#
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts\browser.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\browser.ps1 -- --headless
#   powershell -ExecutionPolicy Bypass -File scripts\browser.ps1 -- --scale=2
#
# 默认 --renderer=gpu。WPT 对齐（CPU + scale 1.0）请用 browser-cpu.ps1 / make browser-cpu。
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
    # 启用 windows-console feature：让 zero-browser 走 console 子系统，
    # tracing 日志输出到当前控制台、Ctrl+C 可终止；打包构建默认 GUI 子系统。
    cargo build --release -p zero-browser -p zero-renderer --features zero-browser/windows-console
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    if (-not (Test-Path -LiteralPath $BrowserBin)) {
        Write-Error "zero-browser not found at $BrowserBin"
    }
    if (-not (Test-Path -LiteralPath $RendererBin)) {
        Write-Error "zero-renderer not found at $RendererBin (required for default multi-process mode)"
    }

    Write-Host "ZeroBrowser: starting $BrowserBin (GPU renderer)"
    $env:RUST_BACKTRACE = "1"
    $launchArgs = @("--renderer=gpu") + $BrowserArgs
    & $BrowserBin @launchArgs
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
