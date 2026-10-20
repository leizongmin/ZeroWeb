# Android 本地工具链 bootstrap + 首次构建打通（M1 切片 1）

**日期**: 2026-09-12
**环境**: Linux x86_64（WSL2），磁盘 1.5T 空闲，内存 46G
**结论**: 默认 feature 路径（无 V8）**全链路打通**——真实 crate 交叉编译 + gradle Release APK 均一次通过

## 已安装组件

| 组件 | 版本 | 位置 | 安装方式 |
|------|------|------|----------|
| OpenJDK | 21.0.12 | 系统 apt | `sudo apt install openjdk-21-jdk`（Gradle 9 支持 17–26，AGP 9.2 最低 17，21 为 LTS） |
| cmdline-tools | 16111833 | `~/Android/Sdk/cmdline-tools/latest` | 官方 zip 手动解压（版本号从 `repository2-3.xml` 数值排序取最新） |
| platform-tools | 37.0.1 | `~/Android/Sdk` | sdkmanager |
| platforms;android-36 | 2.0.0 | `~/Android/Sdk` | sdkmanager |
| build-tools | 36.0.0 | `~/Android/Sdk` | sdkmanager |
| NDK | r30 (30.0.16248370) 稳定版 | `~/Android/Sdk/ndk` | sdkmanager（cargo-ndk 自动发现，无需环境变量） |
| Rust targets | rustc 1.98.1 | rustup | `rustup target add aarch64-linux-android x86_64-linux-android` |
| cargo-ndk | 4.1.2 | `~/.cargo/bin` | `cargo install cargo-ndk` |
| local.properties | — | `apps/android-browser/`（已 gitignore） | `sdk.dir=$HOME/Android/Sdk` |

## 可重复命令

```bash
# 1. SDK 组件（网络需代理时先 export http_proxy/https_proxy）
yes | ~/Android/Sdk/cmdline-tools/latest/bin/sdkmanager --sdk_root=$HOME/Android/Sdk \
  "platform-tools" "platforms;android-36" "build-tools;36.0.0" "ndk;30.0.16248370"

# 2. Rust 侧
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk

# 3. Gradle 入口（Makefile Linux 分支；ANDROID_HOME 为 preflight 硬检查）
export ANDROID_HOME=$HOME/Android/Sdk
# 首次经本机代理下载 gradle 发行版与 AGP 依赖时（PORT 按本机代理端口）：
export GRADLE_OPTS="-Dhttp.proxyHost=127.0.0.1 -Dhttp.proxyPort=$LOCAL_PROXY_PORT -Dhttps.proxyHost=127.0.0.1 -Dhttps.proxyPort=$LOCAL_PROXY_PORT"
make android-release-apk   # → :app:assembleArm64Release
```

注：`make android-preflight` 要求 `$ANDROID_HOME` 环境变量；gradle 本身走 `local.properties`，二者并存不冲突。

## 验证证据（2026-09-12）

1. **cargo 交叉编译**：`cargo ndk -P 26 -t arm64-v8a -o <jniLibs> build --release -p zero-android-browser`
   → 277 crate，release 1m07s，exit=0
   → `libzero_android_browser.so`：`ELF 64-bit LSB shared object, ARM aarch64, for Android 26, built by NDK r30`，14M
2. **gradle 组装**：`make android-release-apk` → `BUILD SUCCESSFUL in 3m 36s`（51 tasks，首次运行含 gradle 发行版 + AGP 依赖下载）
   → `apps/android-browser/app/build/outputs/apk/arm64/release/app-arm64-release-unsigned.apk`（21M）
   → APK 内含 `lib/arm64-v8a/libzero_android_browser.so`（13.9M）
   → badging：`com.leizm.zeroweb`，minSdk 26 / targetSdk 36，INTERNET 权限
3. **工具链冒烟**（安装时）：最小 cdylib 经 `cargo ndk -t arm64-v8a -P 26` 产出 `for Android 26, NDK r30` 的 .so

## 关键认知

- **默认 feature 依赖树不含 v8**（`cargo tree -p zero-android-browser --target aarch64-linux-android | grep " v8 v"` = 0）。
  gradle `buildRustNative` 非 WSL 分支虽然 `V8_FROM_SOURCE=1`，但无 `--features android-renderer`，该 env 是空操作。
  即：**基础 APK 不需要 V8**；V8 只在 renderer feature 路径出现。
- **cargo-ndk 4.x 参数即项目脚本写法**：平台参数是 `-P/--platform`（3.x 为 `-p`），项目 gradle/`build-native-wsl.sh` 均按 4.x 写。

## 遗留缺口

| # | 缺口 | 说明 |
|---|------|------|
| 1 | **renderer feature 路径未打通** | `scripts/android/build-native-wsl.sh` 需 5 项前置：rusty_v8 **150.2.0 递归 checkout**（workspace 锁 150.2.0）、Chromium clang 23、Linux gn、ninja（本机已有 /usr/bin/ninja）、libclang≥19（本机已有 llvm-19）。`download-rusty-v8.sh` 只下载宿主预编译静态库，**不**提供源码 checkout，需另行 `git clone --recursive` |
| 2 | **gradlew 缺可执行位（仓库缺陷）** | git index 中 `apps/android-browser/gradlew` 为 100644，Linux 裸 checkout 跑 `make android-apk` 会 `Permission denied`。本机已 `chmod +x`（工作区）；修复应 `git update-index --chmod=+x apps/android-browser/gradlew` 入库（M1 项） |
| 3 | APK 未签名 | `app-arm64-release-unsigned.apk` 无法直接 `adb install`；M2「可安装产物」需 debug 签名或 keystore 配置 |
