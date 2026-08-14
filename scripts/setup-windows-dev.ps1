# 配置 ZeroWeb Windows 开发依赖中需要显式发现的 libclang。
#
# 用法：
#   powershell -ExecutionPolicy Bypass -File scripts\setup-windows-dev.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\setup-windows-dev.ps1 -InstallPortable -Persist
#   powershell -ExecutionPolicy Bypass -File scripts\setup-windows-dev.ps1 -Persist

[CmdletBinding()]
param(
    [switch]$InstallPortable,
    [switch]$Persist
)

$ErrorActionPreference = "Stop"

if ($env:OS -ne "Windows_NT") {
    throw "setup-windows-dev.ps1 only supports Windows."
}

function Find-LibClangDirectory {
    $candidates = [System.Collections.Generic.List[string]]::new()

    if ($env:LIBCLANG_PATH) {
        $candidates.Add($env:LIBCLANG_PATH)
    }
    if ($env:LOCALAPPDATA) {
        $candidates.Add((Join-Path $env:LOCALAPPDATA "ZeroWeb\tools\libclang\21.1.8\runtimes\win-x64\native"))
    }

    $clang = Get-Command clang.exe -ErrorAction SilentlyContinue
    if ($clang) {
        $candidates.Add((Split-Path -Parent $clang.Source))
    }

    foreach ($root in @($env:ProgramW6432, $env:ProgramFiles, ${env:ProgramFiles(x86)})) {
        if ($root) {
            $candidates.Add((Join-Path $root "LLVM\bin"))
        }
    }

    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path -LiteralPath $vswhere) {
        $installations = & $vswhere -products * -requires Microsoft.VisualStudio.Component.VC.Llvm.Clang -property installationPath
        foreach ($installation in $installations) {
            if (-not $installation) {
                continue
            }
            foreach ($relative in @("VC\Tools\Llvm\x64\bin", "VC\Tools\Llvm\ARM64\bin", "VC\Tools\Llvm\bin")) {
                $candidates.Add((Join-Path $installation $relative))
            }
        }
    }

    foreach ($candidate in $candidates) {
        if ($candidate -and (Test-Path -LiteralPath (Join-Path $candidate "libclang.dll"))) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }
    return $null
}

function Install-PortableLibClang {
    $version = "21.1.8"
    $packageName = "libclang.runtime.win-x64.$version.nupkg"
    $packageRoot = Join-Path $env:LOCALAPPDATA "ZeroWeb\tools\libclang\$version"
    $packagePath = Join-Path $env:TEMP $packageName
    $baseUri = "https://api.nuget.org/v3-flatcontainer/libclang.runtime.win-x64/$version/$packageName"
    $registrationUri = "https://api.nuget.org/v3/registration5-gz-semver2/libclang.runtime.win-x64/$version.json"

    New-Item -ItemType Directory -Force -Path $packageRoot | Out-Null
    Write-Host "Downloading portable libclang $version from NuGet..."
    Invoke-WebRequest -UseBasicParsing -Uri $baseUri -OutFile $packagePath
    $registration = Invoke-RestMethod -Uri $registrationUri
    $catalog = Invoke-RestMethod -Uri $registration.catalogEntry
    if ($catalog.packageHashAlgorithm -ne "SHA512" -or -not $catalog.packageHash) {
        throw "NuGet metadata did not provide a SHA-512 package hash."
    }
    $expectedHash = $catalog.packageHash.Trim()
    $actualHashHex = (Get-FileHash -Algorithm SHA512 -LiteralPath $packagePath).Hash
    $actualHashBytes = [byte[]]::new($actualHashHex.Length / 2)
    for ($index = 0; $index -lt $actualHashBytes.Length; $index++) {
        $actualHashBytes[$index] = [Convert]::ToByte($actualHashHex.Substring($index * 2, 2), 16)
    }
    $actualHash = [Convert]::ToBase64String($actualHashBytes)
    if ($actualHash -ne $expectedHash) {
        Remove-Item -LiteralPath $packagePath -Force
        throw "NuGet SHA-512 verification failed for $packageName."
    }

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::ExtractToDirectory($packagePath, $packageRoot)
    Remove-Item -LiteralPath $packagePath -Force
    $dll = Get-ChildItem -LiteralPath $packageRoot -Filter libclang.dll -Recurse -File | Select-Object -First 1
    if (-not $dll) {
        throw "The verified NuGet package did not contain libclang.dll."
    }
    return $dll.Directory.FullName
}

$libClangDirectory = Find-LibClangDirectory
if (-not $libClangDirectory -and $InstallPortable) {
    $libClangDirectory = Install-PortableLibClang
}
if (-not $libClangDirectory) {
    throw @"
libclang.dll was not found. Install LLVM first:
  winget install --id LLVM.LLVM --exact
or add the "C++ Clang tools for Windows" component in Visual Studio Installer.
For a non-admin user-local install, add -InstallPortable.
Then run this script again. See docs/development/windows.md.
"@
}

$env:LIBCLANG_PATH = $libClangDirectory
Write-Host "libclang.dll: $(Join-Path $libClangDirectory 'libclang.dll')"
Write-Host "LIBCLANG_PATH configured for this process."

if ($Persist) {
    [Environment]::SetEnvironmentVariable("LIBCLANG_PATH", $libClangDirectory, "User")
    Write-Host "LIBCLANG_PATH saved to the current user environment. Open a new terminal before building."
}
