# 日期版本、图标、平台打包与发布

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖日期版本共享逻辑、图标生成与平台包布局；打包脚本不是本轮执行对象。关键词：`ZERO_BUILD_VERSION`、`SOURCE_DATE_EPOCH`、Helper、codesign、iconset。

### User Task Navigation
| 任务 | 源码入口 | 检查边界 |
|---|---|---|
| CLI/Windows 版本不一致 | `build-support/product_version.rs`、`apps/browser/build.rs` | 检查构建输入，不据 Cargo package version 推定产品版本 |
| 包内缺子进程 | 三个平台 package 脚本 | 核对四角色、安装位置和实际启动，见项目 verify |
| 图标缺失 | `tools/icon-gen/src/main.rs`、browser build.rs | 生成资产与资源嵌入是两阶段 |
| macOS Gatekeeper | `scripts/package-macos.sh` | ad-hoc、Developer ID、公证是不同状态 |

## Gotchas
- Rust 版本优先显式 `ZERO_BUILD_VERSION`，其次 SOURCE_DATE_EPOCH 的 UTC 日期，再本地日期，日期命令失败才退 UTC；相同源码在不同构建日可以得到不同产品版本。 (`build-support/product_version.rs`)
- Cargo 只对显式版本和时间戳变化注册 rerun，时间跨日自身不是 rerun 输入；重用构建缓存不能保证嵌入版本自动更新到当天。 (`build-support/product_version.rs`, `crates/product-version/build.rs`)
- Windows 图标存在才编译资源，winres 失败仅警告并继续，版本常量成功不证明 exe 已含图标与 Windows 资源版本。 (`apps/browser/build.rs`, `crates/product-version/src/lib.rs`)
- macOS 脚本以 `--version` 或日期生成版本再导出 ZERO_BUILD_VERSION，不直接用已有该环境变量作首选；传入外部 binary 时修改 bundle 版本不会重写 binary 内嵌版本。 (`scripts/package-macos.sh`)
- Linux/macOS shell 先只校验月 1..12、日 1..31，Rust 共享解析器会核对真实日历；使用预建 binary 的路径不能靠 shell 初检证明版本是有效日期。 (`scripts/package-linux.sh`, `scripts/package-macos.sh`, `build-support/product_version.rs`)

## Architecture
- 产品版本文本规范为 YY.M.D，Windows 四段数值编码 YY/M/D/0，SOURCE_DATE_EPOCH 仅支持 2000..2099；不是 workspace 的 0.1.0 语义版本。 (`build-support/product_version.rs`, `crates/product-version/src/lib.rs`)
- 三个平台都交付 browser、renderer、compositor、image-decoder；Linux/Windows 同目录，macOS 子角色在 Contents/Frameworks 三个独立 Helper.app，不能把一个 browser binary 当完整安装包。 (`scripts/package-linux.sh`, `scripts/package-windows.ps1`, `scripts/package-macos.sh`)
- 图标生成器从同一 SVG 缓存各尺寸 RGBA，再生成 PNG/ICO/iconset/窗口 RGBA；小图有超采样，macOS 1024 的 @2x 图由打包脚本补齐后交 iconutil。 (`tools/icon-gen/src/main.rs`, `scripts/package-macos.sh`)
- macOS Developer ID 路径给 Helper 与主 app 逐级加 runtime/timestamp/allow-jit 并最终严格核验；无身份走 ad-hoc，公证另需 notarytool/staple 后重建 archive。 (`scripts/package-macos.sh`)

## Patterns
- release matrix 显式按 V8/QuickJS 分包，browser 与 renderer 分别传 feature；Windows ARM64 仅有 QuickJS，V8 archive 状态 job 只报告可用性，不会自动增加发布矩阵。 (`.github/workflows/release.yml`)
- Windows 包文本通过 .NET UTF8 无 BOM 写入并统一控制台编码，避免 PowerShell 5.x 的 ACP/BOM 行为污染 README 与安装器脚本。 (`scripts/package-windows.ps1`)
