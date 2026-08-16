# 启动 ZeroBrowser（Windows 版，等价于 make browser）
#
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts\browser.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\browser.ps1 -- --headless
#   powershell -ExecutionPolicy Bypass -File scripts\browser.ps1 -- --scale=2
#
# 默认 --renderer=gpu。WPT 对齐（CPU + scale 1.0）请用 browser-cpu.ps1 / make browser-cpu。
# 会先下载 rusty_v8，再 release 编译 browser、renderer、compositor 与 image-decoder，最后运行已构建的二进制
# （不用 cargo run，确保四个进程的可执行文件同目录）。

param(
    [switch]$BuildOnly,
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

    Write-Host "ZeroBrowser: checking release build cache (only changed targets will compile)..."

    # 确保 freetype-sys 编译 libpng 时能找到 zlib.h。Windows 上 cc crate 传递
    # 的相对路径 -I "libz-sys/src/zlib" 可能无法正确解析，通过 CFLAGS 提供系统
    # zlib 头文件路径作为 fallback（Strawberry Perl 自带 zlib）。
    # 规范不再要求手动设置环境变量。
    $zlibPaths = @(
        "C:\Strawberry\c\include",
        "C:\Program Files\Git\mingw64\include",
        "C:\vcpkg\installed\x64-windows-static-md\include"
    )
    foreach ($p in $zlibPaths) {
        if (Test-Path "$p\zlib.h") {
            $includeFlag = "-I$p"
            if ($env:CFLAGS -notmatch [regex]::Escape($includeFlag)) {
                $env:CFLAGS = "$includeFlag $env:CFLAGS".Trim()
                Write-Host "  (auto-detected zlib.h at $p)"
            }
            break
        }
    }

    # 启用 windows-console feature：让 zero-browser 走 console 子系统，
    # tracing 日志输出到当前控制台、Ctrl+C 可终止；打包构建默认 GUI 子系统。
    cargo build --release -p zero-browser --features zero-browser/windows-console
    cargo build --release -p zero-renderer -p zero-compositor -p zero-image-decoder
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
    if ($BuildOnly) {
        exit 0
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
