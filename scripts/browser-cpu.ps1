# 以 CPU 渲染模式启动 ZeroBrowser（Windows 版，等价于 make browser-cpu）
#
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts\browser-cpu.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\browser-cpu.ps1 -- --headless
#
# 默认 --wpt-parity（CPU + scale 1.0，与 WPT/product-smoke 肉眼对齐）。
# 显式 --renderer / --scale 可覆盖。
# 会先下载 rusty_v8，再 release 编译 browser、renderer 与 compositor，最后运行已构建的二进制
# （不用 cargo run，确保三个进程的可执行文件同目录）。

param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$BrowserArgs
)

$ErrorActionPreference = "Stop"

$ProjectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$DownloadScript = Join-Path $PSScriptRoot "download-rusty-v8.ps1"
$BrowserBin = Join-Path $ProjectRoot "target\release\zero-browser.exe"
$RendererBin = Join-Path $ProjectRoot "target\release\zero-renderer.exe"
$CompositorBin = Join-Path $ProjectRoot "target\release\zero-compositor.exe"

Push-Location $ProjectRoot
try {
    Write-Host "ZeroBrowser: preparing rusty_v8..."
    & $DownloadScript
    if ($LASTEXITCODE -ne 0) {
        Write-Error "rusty_v8 setup failed with exit code $LASTEXITCODE"
    }

    Write-Host "ZeroBrowser: building zero-browser, zero-renderer, and zero-compositor (release)..."

    if ($env:CFLAGS -notmatch "zlib") {
        $zlibPaths = @(
            "C:\Strawberry\c\include",
            "C:\Program Files\Git\mingw64\include",
            "C:\vcpkg\installed\x64-windows-static-md\include"
        )
        foreach ($p in $zlibPaths) {
            if (Test-Path "$p\zlib.h") {
                $env:CFLAGS = "-I$p $env:CFLAGS".Trim()
                Write-Host "  (auto-detected zlib.h at $p)"
                break
            }
        }
    }

    # 启用 windows-console feature：让 zero-browser 走 console 子系统，
    # tracing 日志输出到当前控制台、Ctrl+C 可终止；打包构建默认 GUI 子系统。
    cargo build --release -p zero-browser -p zero-renderer -p zero-compositor --features zero-browser/windows-console
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    if (-not (Test-Path -LiteralPath $BrowserBin)) {
        Write-Error "zero-browser not found at $BrowserBin"
    }
    if (-not (Test-Path -LiteralPath $RendererBin)) {
        Write-Error "zero-renderer not found at $RendererBin (required for default multi-process mode)"
    }
    if (-not (Test-Path -LiteralPath $CompositorBin)) {
        Write-Error "zero-compositor not found at $CompositorBin (required for compositor mode)"
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
