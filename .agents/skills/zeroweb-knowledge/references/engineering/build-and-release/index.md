# 构建、门禁与交付资源

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖编译输入、依赖准备、守卫/CI 和发布包，不替 runtime 维护引擎及平台运行语义。修改 Makefile、下载源、CI、签名或公开质量事实先核对治理 C2 边界。

## Gotchas
- workspace 的 V8/QuickJS 并集不代表发布单包 feature；release matrix 显式分 browser/renderer 构建，普通 workspace 绿灯可能遮蔽单包缺失能力。 (`Cargo.toml`, `Makefile`, `.github/workflows/release.yml`)
- 测试守卫、磁盘前置和 CI job timeout 限制不同资源及阶段；不得用 disk guard 的放行替代进程树内存或总墙钟限制。 (`scripts/test-guard.rs`, `scripts/target-disk-guard.sh`, `.github/workflows/ci.yml`)

## Architecture
- 构建日期版本同时供 Rust VERSION 与 Windows 资源，平台打包还生成 bundle/package 版本；应核对相同显式输入而非把 Cargo 0.1.0 当用户版本。 (`build-support/product_version.rs`, `crates/product-version/src/lib.rs`, `apps/browser/build.rs`, `scripts/package-macos.sh`)
- Android Gradle 的 native 输出与桌面四角色包是不同交付体系，ABI/flavor 和 renderer feature 需各自检查，不把宿主构建成功称为全部平台生产通过。 (`apps/android-browser/app/build.gradle.kts`, `apps/android-browser/rust/Cargo.toml`, `scripts/package-linux.sh`)

## Child Knowledge Nodes
- [构建矩阵、依赖准备与 Android 工具链](toolchains.md) — 查 V8 archive、单包 feature、Gradle ABI 与源码交叉编译。
- [CI、安全、内存、超时与磁盘门禁](guards-and-ci.md) — 查相位、阈值、能力分支、平台缺测和守卫副作用。
- [日期版本、图标、平台打包与发布](packaging.md) — 查版本优先级、图标资源、四角色布局、签名与公证。
