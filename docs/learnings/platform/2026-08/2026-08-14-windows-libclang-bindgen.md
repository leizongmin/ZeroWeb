---
date: 2026-08-14
modules: QuickJS feature, rquickjs-sys, Windows 开发环境
---

# Windows 上 bindgen 找不到 libclang

## 问题描述

Windows 执行 QuickJS feature 的 Cargo 构建时，`rquickjs-sys` 在 bindgen 阶段报告找不到 `libclang.dll`。MSVC、Windows SDK 和 Rust MSVC target 均已安装，默认 V8 构建不受影响。

## 根因分析

`rquickjs-sys 0.7.0` 使用 `bindgen 0.69.5`，后者通过 `clang-sys` 在构建脚本运行时动态加载 libclang。MSVC 只提供 C/C++ 编译和链接工具，不提供该 DLL；仅能运行 `clang.exe` 也不能证明动态库在 clang-sys 的搜索路径内。

## 解决方案

安装 LLVM、Visual Studio 的 Clang tools 组件，或使用经过 NuGet SHA-512 校验的用户级 libclang runtime，并把实际包含 `libclang.dll` 的目录设为 `LIBCLANG_PATH`。仓库的 `scripts/setup-windows-dev.ps1` 先验证文件存在，再设置当前进程或用户级环境变量；不在版本控制配置中硬编码本机绝对路径。
