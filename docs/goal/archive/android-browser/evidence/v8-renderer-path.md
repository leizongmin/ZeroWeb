# renderer feature 路径（V8 从源码交叉编译）本地打通

**日期**: 2026-09-12
**环境**: 同 [local-toolchain-bootstrap.md](local-toolchain-bootstrap.md)（WSL2 x86_64、NDK r30、cargo-ndk 4.x）
**结论**: `build-native-wsl.sh arm64-v8a` 全链路打通——V8 150.2.0 从源码交叉编译成功，
产出含 renderer feature 的 `libzero_android_browser.so`（90M，23 个 JNI 导出，`for Android 26, NDK r30`）

## 前置清单（六项）

| 前置 | 值 | 获取方式 |
|------|-----|----------|
| ZERO_V8_SOURCE | `$HOME/v8/rusty_v8` | `git clone --depth 1 --branch v150.2.0 --recursive --shallow-submodules --jobs 8 https://github.com/denoland/rusty_v8.git`（~20min，1.1G+） |
| ZERO_CHROMIUM_CLANG | `$ZERO_V8_SOURCE/third_party/llvm-build/Release+Asserts` | `python3 $ZERO_V8_SOURCE/tools/clang/scripts/update.py`（pinned `clang-llvmorg-23-init-10931-g20b6ec66-11`，落点在 `third_party/llvm-build` 而非 `tools/llvm-build`） |
| ZERO_ANDROID_NDK | `$HOME/Android/Sdk/ndk/30.0.16248370` | 已有（toolchain bootstrap） |
| LIBCLANG_PATH | `/usr/lib/llvm-19/lib` | 系统包 libclang-19-dev（bindgen 用，仅需库不需编译器） |
| ZERO_V8_GN | `$HOME/v8/bin/gn` | 源码编译：`git clone https://gn.googlesource.com/gn.git`（**勿浅克隆**——gen.py 需 `initial-commit` tag）+ `python3 build/gen.py --no-static-libstdc++` + `ninja -C out gn`。宿主无 clang 编译器时用 `CXX=g++` 并 `sed` 掉 `out/build.ninja` 中的 `-Werror`（gn 源码 ASCII 图注释触发 gcc `-Werror=comment`，上游只管 clang） |
| ZERO_V8_NINJA | `/usr/bin/ninja` | 系统已有 |

另需 Debian sysroot（`use_sysroot=true` 时 gn 硬性要求）：
`python3 $ZERO_V8_SOURCE/build/linux/sysroot_scripts/install-sysroot.py --arch=amd64`（214M，装入 checkout 内持久复用）。

## 可重复命令

```bash
export http_proxy=... https_proxy=...   # 全程必需：rust_toolchain.py/gn 子模块克隆/GCS 下载均直连被卡
export ZERO_V8_SOURCE=$HOME/v8/rusty_v8
export ZERO_CHROMIUM_CLANG=$ZERO_V8_SOURCE/third_party/llvm-build/Release+Asserts
export ZERO_ANDROID_NDK=$HOME/Android/Sdk/ndk/30.0.16248370
export LIBCLANG_PATH=/usr/lib/llvm-19/lib
export ZERO_V8_GN=$HOME/v8/bin/gn
export ZERO_V8_NINJA=/usr/bin/ninja

bash scripts/android/build-native-wsl.sh <repo-root> <output-dir> arm64-v8a
# 首跑 ~30-40min（V8 ninja 编译占绝对大头）；产物 <output-dir>/arm64-v8a/
```

注：`.ps1` 是 Windows→WSL 转发壳（`wsl.exe -d Distro`），Linux 直跑 `.sh` 即可。

## 首跑踩坑记录（三轮修复）

| # | 症状 | 根因 | 修复 |
|---|------|------|------|
| 1 | `rust_toolchain.py` 挂死 9min+（ESTAB 连接无流量） | 起构建时未导出代理，GCS 直连被卡 | 杀进程树带 `http_proxy/https_proxy` 重跑；**构建全程需代理** |
| 2 | `gn gen`: `BUILD.gn:37 Undefined identifier ${android_ndk_version}` | 脚本以 `EXTRA_GN_ARGS=android_ndk_version=30` 传参，但 GN 只让 `declare_args()` 声明过的参数进入作用域（未声明参数被静默丢弃）；上游 V8 引用了该变量却从未声明 | 在 `build/` **submodule** 的 `config/BUILDCONFIG.gn`（根作用域，声明后全局可见，已做 GN 作用域实验验证）追加 `declare_args() { android_ndk_version = "" }`，并以路径前缀改写（`--src-prefix=a/build/`）固化进 `scripts/android/patches/rusty-v8-android-bindgen.patch`，脚本 reverse-check 幂等逻辑不受影响 |
| 3 | `gn gen`: `Missing sysroot (debian_bullseye_amd64-sysroot)` | `use_sysroot=true` 下宿主侧工具链需要 Debian sysroot，checkout 内未装 | `install-sysroot.py --arch=amd64` |

**教训**：rusty_v8 仓库的 `build/` 是独立 git submodule——在主仓根 `git diff` 对 submodule 内文件输出为**空**，补丁 hunk 必须在 submodule 内生成并改写路径前缀；direct 编辑 submodule 工作树还可能被 build.rs 的 submodule 操作截断（实测出现过 0 字节），修复必须固化进项目补丁文件。

## 验证证据（2026-09-12）

- 构建：`Finished release profile [optimized] in 29m 33s`（第五次启动成功；前四次败于上表三轮问题）
- 产物：`android-renderer-jniLibs/arm64-v8a/`：`libzero_android_browser.so`（90M，无 V8 基线 14M）+ `libc++_shared.so`（9.1M）
- `file`: `ELF 64-bit ARM aarch64, for Android 26, built by NDK r30` ✓
- `strings` 命中 `v8-version`（V8 静态链接嵌入；`nm -D` 无 v8::internal 动态导出属预期——静态库符号隐藏）
- JNI 导出：`nm -D | grep Java_com_leizm | wc -l` = 23（M2 基线 22，renderer feature 路径新增导出）

## gradle Linux 分支 + 签名 APK（2026-09-12 续）

- `app/build.gradle.kts` 的 `buildRustNative`：`useWslRenderer` 分支按 OS 分叉——Windows 仍走
  `.ps1`→`wsl.exe` 转发壳；**Linux/macOS 直调 `build-native-wsl.sh`**（Exec 继承环境，六项
  ZERO_* 由调用方导出，脚本自校验）。
- Makefile 新增 `make android-renderer-apk`（Linux，test-guard 包裹，3600s 墙钟，见 run-rules #14）。
- **release 签名**：`local.properties`（gitignored）提供 `zeroweb.keystore.path/password/alias`
  三项即启用签名，缺任一回退 unsigned。本地 keystore 生成：
  `keytool -genkeypair -keystore $HOME/.android/zeroweb-release.keystore -alias zeroweb -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=ZeroWeb, O=ZeroWeb"`
  （keystore 与口令仅存本机 local.properties，不入库）。
- 验证：`-PuseWslRenderer :app:assembleArm64Release` → `BUILD SUCCESSFUL in 58s`（V8 缓存热；
  冷缓存加 V8 ninja ~30-40min）→ `app-arm64-release.apk`（107M，无 `-unsigned` 后缀），
  内含 90M `libzero_android_browser.so` + `libc++_shared.so`；
  `apksigner verify --print-certs` → `CN=ZeroWeb, O=ZeroWeb` ✓。
- Kotlin DSL 坑：`.kts` 中 `java.util.Properties` 必须文件顶 `import java.util.Properties`
  （裸写 `java.util` 与 `java {}` 扩展访问器冲突报 Unresolved reference）。

## 边界说明

本 evidence 覆盖「V8 从源码交叉编译 + renderer feature .so 产出 + gradle Linux 出包 + 签名」；
renderer 的 Android transport adapter 属 RFC 域（待 `android-browser-spec-rfc.md` 批准），
未在本轮范围内——即 APK 内 renderer 仍走 Kotlin Service 拓扑，V8 就绪但 transport 未切换。
