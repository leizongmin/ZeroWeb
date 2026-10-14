# 构建矩阵、依赖准备与 Android 工具链

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 Rust/V8 下载、单包构建、Android Gradle→native 的工具链边界；feature 运行语义由 runtime conditions 维护。关键词：`RUSTY_V8_ARCHIVE`、`cargo ndk`、`useWslRenderer`。本轮不执行构建。

## Gotchas
- `.cargo/config.toml` 把 archive 定到相对仓库的 `.cargo/rusty_v8/archive`，下载脚本从 Cargo.lock 读取实际 V8 版本并按 target/profile 缓存；仅改 workspace 版本声明不能证明本机 archive 匹配。 (`.cargo/config.toml`, `scripts/download-rusty-v8.sh`)
- 下载器接受 ar 或 gzip 文件头，清理非法缓存与 partial 后才续传；Windows 使用 `.lib.gz`、其他平台 `.a.gz`，不能跨 target 复用同一缓存文件。 (`scripts/download-rusty-v8.sh`)
- `make build` 先构建除 browser 的 workspace 再单独 browser，`browser-build` 另外刷新三个 release 子角色；只构建 browser executable 不代表运行所需同版本子进程齐备。 (`Makefile`)
- workspace feature 并集会遮蔽无引擎和 QuickJS 单包错误，CI 单独检查 browser 的 no-default 与 quickjs 两臂；全 workspace 检查通过不能替代这些臂。 (`.github/workflows/ci.yml`, `Cargo.toml`)
- Gradle 根据请求任务名是否包含 `Emulator` 选 x86_64，否则 arm64-v8a；emulator release variant 被禁用，不能把 emulator debug 构建推定为真机 release。 (`apps/android-browser/app/build.gradle.kts`)
- `useWslRenderer` 按 Gradle property 是否存在启用；普通 native 路径不显式开启 `android-renderer`，WSL/Linux 脚本才启用该 feature，宿主 APK 构建不等于真实 renderer APK。 (`apps/android-browser/app/build.gradle.kts`, `apps/android-browser/rust/Cargo.toml`, `scripts/android/build-native-wsl.sh`)

## Architecture
- WSL native 脚本要求 rusty_v8 150.2.0、Chromium clang、Linux NDK、libclang、gn、ninja 六项输入，并把 v8 patch、NDK 链接及 libc++_shared.so 交付纳入同一路径；只配置 cargo target 不够。 (`scripts/android/build-native-wsl.sh`)
- renderer 构建的临时 workspace 限定在用户 cache 子目录，target 可独立复用；脚本复制源码后修改外部 V8 checkout 并应用 patch，不能把它当无副作用的依赖预检。 (`scripts/android/build-native-wsl.sh`)
- JNI 产物进入 Gradle generated/jniLibs，merge NativeLibs/JniLibFolders 任务依赖 buildRustNative，后者启动前会删除旧输出目录；增量构建需核对 inputs 与 feature property。 (`apps/android-browser/app/build.gradle.kts`)
- workspace 明确固定 unicode-segmentation 1.12.0，并通过 patch 接本地 Taffy；升级依赖时需区分版本更新与本地算法补丁，不应直接绕过补丁换成上游。 (`Cargo.toml`)
