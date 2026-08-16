# ZeroBrowser Windows 打包脚本
#
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
#
# 生成 ZeroBrowser 安装包到 target\packages\ 目录。
# 必须在 Windows 上运行，需要 cargo 和 MSVC 工具链。

param(
    [switch]$Installer = $false
)

$ErrorActionPreference = "Stop"

# 强制 UTF-8：脚本含中文字面量，且打包输出/README 需跨平台无乱码。
# PowerShell 5.x 默认按系统 ACP（中文系统是 cp936/GBK）输出，会导致控制台与
# 写出的文本文件出现乱码。这里把控制台与管道编码统一改为 UTF-8。
try {
    [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
    [Console]::InputEncoding = [System.Text.UTF8Encoding]::new($false)
} catch {}
$OutputEncoding = [System.Text.UTF8Encoding]::new($false)
# chcp 65001 让传统 cmd 子进程（如 cargo 间接调用的工具）也走 UTF-8。
try { chcp 65001 > $null } catch {}

# 写 UTF-8（无 BOM）文本文件的辅助函数。
# PowerShell 5.x 的 Out-File -Encoding UTF8 会写入 BOM，部分查看器/解压工具会
# 把 BOM 当成普通字节显示乱码；这里用 .NET 显式构造无 BOM 编码器。
function Write-Utf8NoBom {
    param(
        [Parameter(Mandatory)] [string] $Path,
        [Parameter(Mandatory)] [string] $Content
    )
    $enc = [System.Text.UTF8Encoding]::new($false)
    [System.IO.File]::WriteAllText($Path, $Content, $enc)
}

$ProjectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$PackageDir = Join-Path $ProjectRoot "target\packages"

# 获取产品构建日期版本。
if ($env:ZERO_BUILD_VERSION) {
    $Version = $env:ZERO_BUILD_VERSION
} elseif ($env:SOURCE_DATE_EPOCH) {
    try {
        $BuildDate = [DateTimeOffset]::FromUnixTimeSeconds(
            [Int64]::Parse($env:SOURCE_DATE_EPOCH)
        ).UtcDateTime
    } catch {
        throw "Invalid SOURCE_DATE_EPOCH: $env:SOURCE_DATE_EPOCH"
    }
    $Version = "{0}.{1}.{2}" -f ($BuildDate.Year % 100), $BuildDate.Month, $BuildDate.Day
} else {
    $BuildDate = [DateTime]::Now
    $Version = "{0}.{1}.{2}" -f ($BuildDate.Year % 100), $BuildDate.Month, $BuildDate.Day
}
if ($Version -notmatch '^\d{1,2}\.\d{1,2}\.\d{1,2}$') {
    throw "Version must use YY.M.D: $Version"
}
try {
    $VersionParts = $Version.Split('.') | ForEach-Object { [Int32]::Parse($_) }
    $null = [DateTime]::new(
        2000 + $VersionParts[0],
        $VersionParts[1],
        $VersionParts[2]
    )
} catch {
    throw "Version contains an invalid date: $Version"
}
$env:ZERO_BUILD_VERSION = $Version

Write-Host "[INFO] ZeroBrowser v$Version Windows 打包" -ForegroundColor Green

# 编译（browser、renderer 与 compositor 须在同一输出目录，供多进程 spawn）
Write-Host "[INFO] 编译 release 二进制..." -ForegroundColor Green

if ($env:CFLAGS -notmatch "zlib") {
    $zlibPaths = @(
        "C:\Strawberry\c\include",
        "C:\Program Files\Git\mingw64\include",
        "C:\vcpkg\installed\x64-windows-static-md\include"
    )
    foreach ($p in $zlibPaths) {
        if (Test-Path "$p\zlib.h") {
            $env:CFLAGS = "-I$p $env:CFLAGS".Trim()
            Write-Host "[INFO] auto-detected zlib.h at $p"
            break
        }
    }
}

Push-Location $ProjectRoot
cargo build --release -p zero-browser
cargo build --release -p zero-renderer -p zero-compositor -p zero-image-decoder
Pop-Location

$BrowserBin = Join-Path $ProjectRoot "target\release\zero-browser.exe"
$RendererBin = Join-Path $ProjectRoot "target\release\zero-renderer.exe"
$CompositorBin = Join-Path $ProjectRoot "target\release\zero-compositor.exe"
$ImageDecoderBin = Join-Path $ProjectRoot "target\release\zero-image-decoder.exe"
if (-not (Test-Path $BrowserBin)) {
    Write-Host "[ERROR] 编译失败: zero-browser" -ForegroundColor Red
    exit 1
}
if (-not (Test-Path $RendererBin)) {
    Write-Host "[ERROR] 编译失败: zero-renderer" -ForegroundColor Red
    exit 1
}
if (-not (Test-Path $CompositorBin)) {
    Write-Host "[ERROR] 编译失败: zero-compositor" -ForegroundColor Red
    exit 1
}
if (-not (Test-Path $ImageDecoderBin)) {
    Write-Host "[ERROR] 编译失败: zero-image-decoder" -ForegroundColor Red
    exit 1
}

$Binary = $BrowserBin

$BinarySize = (Get-Item $Binary).Length / 1MB
Write-Host "[INFO] 二进制大小: $([math]::Round($BinarySize, 1)) MB"

# 创建输出目录
New-Item -ItemType Directory -Force -Path $PackageDir | Out-Null
$DistDir = Join-Path $PackageDir "ZeroBrowser-$Version-win64"
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

# 复制二进制（多进程：renderer/compositor/image-decoder 与 browser 同目录）
Copy-Item $BrowserBin (Join-Path $DistDir "ZeroBrowser.exe")
Copy-Item $RendererBin (Join-Path $DistDir "zero-renderer.exe")
Copy-Item $CompositorBin (Join-Path $DistDir "zero-compositor.exe")
Copy-Item $ImageDecoderBin (Join-Path $DistDir "zero-image-decoder.exe")

# 创建 README
$readme = @"
ZeroBrowser v$Version
====================

ZeroBrowser 是一个基于 Rust 构建的跨平台浏览器。

运行方式：
  双击 ZeroBrowser.exe 启动浏览器。

命令行选项：
  ZeroBrowser.exe [URL]              打开指定 URL
  ZeroBrowser.exe --headless          无头模式运行
  ZeroBrowser.exe --remote-debugging-port=9222  启用远程调试

系统要求：
  - Windows 10 或更高版本
  - 支持的 GPU（可选，也支持 CPU 渲染）

许可证：MIT
"@
Write-Utf8NoBom -Path (Join-Path $DistDir "README.txt") -Content $readme

# 创建 ZIP
$ZipFile = Join-Path $PackageDir "ZeroBrowser-$Version-win64.zip"
if (Test-Path $ZipFile) { Remove-Item $ZipFile }
Compress-Archive -Path $DistDir -DestinationPath $ZipFile
Write-Host "[INFO] .zip 已生成: $ZipFile" -ForegroundColor Green

# 可选：创建 NSIS 安装程序
if ($Installer) {
    Write-Host "[INFO] 创建安装程序需要 NSIS：https://nsis.sourceforge.io/" -ForegroundColor Yellow
    $NsiPath = Join-Path $PackageDir "installer.nsi"
    $nsis = @"
!define APPNAME "ZeroBrowser"
!define APPVERSION "$Version"
!define APPEXE "ZeroBrowser.exe"

Name "`${APPNAME} `${APPVERSION}"
OutFile "ZeroBrowser-`${APPVERSION}-setup.exe"
InstallDir "`$PROGRAMFILES\`${APPNAME}"

Section "Install"
  SetOutPath `$INSTDIR
  File "`$DISTDIR\`${APPEXE}"
  CreateShortcut "`$DESKTOP\`${APPNAME}.lnk" "`$INSTDIR\`${APPEXE}"
  CreateDirectory "`$SMPROGRAMS\`${APPNAME}"
  CreateShortcut "`$SMPROGRAMS\`${APPNAME}\`${APPNAME}.lnk" "`$INSTDIR\`${APPEXE}"
  CreateShortcut "`$SMPROGRAMS\`${APPNAME}\Uninstall.lnk" "`$INSTDIR\uninstall.exe"
  WriteUninstaller "`$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "`$INSTDIR\`${APPEXE}"
  Delete "`$INSTDIR\uninstall.exe"
  Delete "`$DESKTOP\`${APPNAME}.lnk"
  Delete "`$SMPROGRAMS\`${APPNAME}\`${APPNAME}.lnk"
  Delete "`$SMPROGRAMS\`${APPNAME}\Uninstall.lnk"
  RMDir "`$SMPROGRAMS\`${APPNAME}"
  RMDir "`$INSTDIR"
SectionEnd
"@
    Write-Utf8NoBom -Path $NsiPath -Content $nsis
    Write-Host "[INFO] NSIS 脚本已生成: $NsiPath"
}

# 清理临时目录
Remove-Item $DistDir -Recurse -Force
Write-Host ""
Write-Host "[INFO] 打包完成！产物在 $PackageDir\" -ForegroundColor Green
Get-ChildItem $PackageDir | Format-Table Name, Length
