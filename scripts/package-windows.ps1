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

$ProjectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$PackageDir = Join-Path $ProjectRoot "target\packages"

# 获取版本号
$CargoToml = Get-Content (Join-Path $ProjectRoot "Cargo.toml") -Raw
if ($CargoToml -match 'version\s*=\s*"([^"]+)"') {
    $Version = $Matches[1]
} else {
    $Version = "0.1.0"
}

Write-Host "[INFO] ZeroBrowser v$Version Windows 打包" -ForegroundColor Green

# 编译（browser 与 renderer 须在同一输出目录，供多进程 spawn）
Write-Host "[INFO] 编译 release 二进制..." -ForegroundColor Green
Push-Location $ProjectRoot
cargo build --release -p zero-browser -p zero-renderer
Pop-Location

$BrowserBin = Join-Path $ProjectRoot "target\release\zero-browser.exe"
$RendererBin = Join-Path $ProjectRoot "target\release\zero-renderer.exe"
if (-not (Test-Path $BrowserBin)) {
    Write-Host "[ERROR] 编译失败: zero-browser" -ForegroundColor Red
    exit 1
}
if (-not (Test-Path $RendererBin)) {
    Write-Host "[ERROR] 编译失败: zero-renderer" -ForegroundColor Red
    exit 1
}

$Binary = $BrowserBin

$BinarySize = (Get-Item $Binary).Length / 1MB
Write-Host "[INFO] 二进制大小: $([math]::Round($BinarySize, 1)) MB"

# 创建输出目录
New-Item -ItemType Directory -Force -Path $PackageDir | Out-Null
$DistDir = Join-Path $PackageDir "ZeroBrowser-$Version-win64"
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

# 复制二进制（多进程：renderer 与 browser 同目录）
Copy-Item $BrowserBin (Join-Path $DistDir "ZeroBrowser.exe")
Copy-Item $RendererBin (Join-Path $DistDir "zero-renderer.exe")

# 创建 README
@"
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
"@ | Out-File (Join-Path $DistDir "README.txt") -Encoding UTF8

# 创建 ZIP
$ZipFile = Join-Path $PackageDir "ZeroBrowser-$Version-win64.zip"
if (Test-Path $ZipFile) { Remove-Item $ZipFile }
Compress-Archive -Path $DistDir -DestinationPath $ZipFile
Write-Host "[INFO] .zip 已生成: $ZipFile" -ForegroundColor Green

# 可选：创建 NSIS 安装程序
if ($Installer) {
    Write-Host "[INFO] 创建安装程序需要 NSIS：https://nsis.sourceforge.io/" -ForegroundColor Yellow
    $NsiPath = Join-Path $PackageDir "installer.nsi"
    @"
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
"@ | Out-File $NsiPath -Encoding UTF8
    Write-Host "[INFO] NSIS 脚本已生成: $NsiPath"
}

# 清理临时目录
Remove-Item $DistDir -Recurse -Force
Write-Host ""
Write-Host "[INFO] 打包完成！产物在 $PackageDir\" -ForegroundColor Green
Get-ChildItem $PackageDir | Format-Table Name, Length
