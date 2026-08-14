# Windows 开发环境

本文档说明在 Windows 10/11 x64 上构建和测试 ZeroWeb 所需的环境。仓库使用 MSVC Rust target；不要在 MSYS/MinGW Rust target 下构建。

## 1. 安装基础工具

安装以下组件：

1. Visual Studio 2022 或 Build Tools 2022，勾选“使用 C++ 的桌面开发”，至少包含 MSVC v143 和 Windows 10/11 SDK。
2. Rust stable MSVC 工具链，并安装格式化和静态检查组件：

   ```powershell
   rustup default stable-x86_64-pc-windows-msvc
   rustup component add rustfmt clippy
   ```

3. Node.js 20 或更新版本。Chrome 一致性脚本和 WPT 辅助脚本需要 Node.js。
4. GNU Make。Windows 的 `make test` 会调用 PowerShell/Windows 可执行文件，不要求 Bash。

验证基础工具：

```powershell
rustc --version
cargo --version
node --version
make --version
```

## 2. 安装并配置 LLVM/libclang

QuickJS feature 通过 `rquickjs-sys` 的 bindgen 构建脚本加载 `libclang.dll`。只有 `clang.exe`、MSVC 或 Windows SDK 不够，系统必须能找到 LLVM 的动态库。

只需要构建 ZeroWeb 时，推荐用仓库脚本安装经过 NuGet SHA-512 校验的用户级 libclang runtime，不需要管理员权限：

```powershell
powershell -ExecutionPolicy Bypass -File scripts\setup-windows-dev.ps1 -InstallPortable -Persist
```

该方式把 DLL 放在 `%LOCALAPPDATA%\ZeroWeb\tools\libclang\<version>`，不修改仓库文件。需要完整的 `clang.exe`、LLVM headers 或其他 LLVM 工具时，再用 Windows Package Manager 安装官方 LLVM 包：

```powershell
winget install --id LLVM.LLVM --exact
```

也可以在 Visual Studio Installer 中添加 “C++ Clang tools for Windows”。完整 LLVM 或 VS 组件安装后运行仓库预检脚本：

```powershell
powershell -ExecutionPolicy Bypass -File scripts\setup-windows-dev.ps1 -Persist
```

脚本按以下顺序查找 `libclang.dll`：已有 `LIBCLANG_PATH`、PATH 中的 `clang.exe`、`Program Files\LLVM\bin`、Visual Studio LLVM 组件目录。找不到时，`-InstallPortable` 从 NuGet 官方源下载 `libclang.runtime.win-x64` 并验证服务端 SHA-512。`-Persist` 把确认存在 DLL 的目录写入当前用户的 `LIBCLANG_PATH`；之后需要打开新终端。

验证 QuickJS 构建链：

```powershell
$env:LIBCLANG_PATH
Test-Path (Join-Path $env:LIBCLANG_PATH "libclang.dll")
cargo check -p zero-script-sandbox --no-default-features --features quickjs
```

如果仍提示找不到 DLL，先重新运行不带 `-Persist` 的脚本查看诊断。不要把某台机器的 LLVM 绝对路径写入 `.cargo/config.toml`。

## 3. 准备 rusty_v8

仓库的 `.cargo/config.toml` 默认把 `RUSTY_V8_ARCHIVE` 指向项目内缓存。启动脚本会按当前 Rust target 下载匹配的预构建库：

```powershell
powershell -ExecutionPolicy Bypass -File scripts\download-rusty-v8.ps1
```

一般无需手工设置 `RUSTY_V8_ARCHIVE`。只有使用自建或镜像产物时才覆盖它；值可以是兼容的本地 `.lib` 路径或 release URL。

## 4. 构建、运行和测试

```powershell
# GPU 模式启动浏览器；脚本会准备 rusty_v8 并构建多进程组件
powershell -ExecutionPolicy Bypass -File scripts\browser.ps1

# CPU + scale 1.0 的 WPT 对齐模式
powershell -ExecutionPolicy Bypass -File scripts\browser-cpu.ps1

# 完整测试门禁，包含 QuickJS 编译和测试
make test
```

无人值守测试必须使用 `make test`，不要绕过项目的 `test-guard`。Windows GUI 测试由 Makefile 强制单线程运行，避免多个测试进程互相关闭共享 compositor。

## 5. 常见问题

### `Unable to find libclang` / 找不到 `libclang.dll`

这表示 LLVM 未安装，或 `LIBCLANG_PATH` 没有指向包含 DLL 的目录。重新执行第 2 节的安装和预检；路径应类似 `C:\Program Files\LLVM\bin`，但以脚本实际探测结果为准。

### `rusty_v8` 链接或下载失败

先单独运行 `scripts\download-rusty-v8.ps1` 查看 target 和下载诊断。确认当前 Rust target 是 `x86_64-pc-windows-msvc`，并检查代理是否阻止 GitHub release 下载。

### `zlib.h` 缺失

`browser.ps1`、`browser-cpu.ps1` 和 Windows 打包脚本会探测 Strawberry Perl、Git for Windows 与 vcpkg 的 zlib headers。若直接执行底层 Cargo 命令仍失败，优先安装 vcpkg 的 x64 Windows zlib，或改用仓库启动脚本。
