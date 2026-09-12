# ZeroWeb Android Browser (`zero-android-browser`)

> ZeroWeb Android 浏览器主机的 JNI 桥接层 — 在 Kotlin Activity 宿主中按角色复用共享 Rust 多进程运行循环

## 概述

`zero-android-browser` 是 Android 端浏览器（应用 ID `com.leizm.zeroweb`）的原生库（`cdylib`），为 Kotlin/Jetpack Compose 宿主提供 **28 个 JNI 导出**（与 `NativeBridge.kt` 的 28 个 `external fun` 一一对应），并经 `facade.rs` 把浏览器 chrome 状态（标签页/书签/历史/下载/渲染槽位）收敛到共享 `zero_browser_shell::BrowserShell` 的单一状态入口。它保持 renderer、compositor、image-decoder 的物理多进程隔离，其中 decoder 与 compositor 走原生线程运行（复用 `zero-image-decoder` / `zero-compositor` 的 `run_role`），renderer 经 `zero_renderer::run_android_role` 走原生线程（transport adapter 已落地；RFC 已于 2026-09-12 批准，`RendererService0-7` 的槽位号经 `nativeRunRole` 透传为 `renderer_id`）。

设计背景见 [docs/specs/android-browser-spec-rfc.md](../../../docs/specs/android-browser-spec-rfc.md)。

## JNI 导出面（lib.rs，28 个）

- **元信息** — `nativeVersion`（引导屏版本串，`NATIVE_VERSION = "ZeroWeb Android M2"`，与 `lib.rs` 常量一致）、`nativeRendererLinked`（renderer feature 是否已链接）
- **浏览器 chrome 状态面**（`facade.rs`，复用 `zero_browser_shell::BrowserShell`）— `nativeLoadProfile` / `nativeBrowserSnapshot`（JSON 快照：tabs/bookmarks/history/downloads/revision）、`nativeNavigate` / `nativeNewTab` / `nativeNewTabWithUrl` / `nativeCloseTab` / `nativeSelectTab` / `nativeGoBack` / `nativeGoForward` / `nativeToggleBookmark` / `nativeRemoveBookmark` / `nativeClearHistory`
- **渲染槽位与页面交互** — `nativeAttachRenderer`（slot + FD + viewport + density 绑定）、`nativeIsRendererAttached`、`nativeLatestPageFrame`（多标签缩略图帧）、`nativeScroll` / `nativePageTap` / `nativePageText` / `nativePageKey`（输入经 renderer transport 转发）
- **Compositor 帧** — `nativeAttachCompositor` / `nativeDetachCompositor` / `nativeCompositorTestFrame`
- **角色生命周期** — `nativeStartRole`（Service 进程就绪前校验 role 白名单，仅接受 `renderer` / `compositor` / `image-decoder`）、`nativeRunRole`（`ParcelFileDescriptor.detachFd()` 移交 socket FD → `zero_protocol::android_socket_transport_from_fd` → 按角色起线程；renderer 需 `--features android-renderer`，未链接时该 role 启动被拒且 FD 关闭），失败路径负责 `close`
- **探针** — `nativeProbeDecoder`（经 socket 发送畸形 `ImageDecodeRequest`，校验错误回复）、`nativeProbeCompositor`（`RegisterUiSurface → Ok / UiFrame → Ok / GetCompositorUiFrame` 帧数据往返校验）
- **android_main 锚点** — 满足 winit native-activity 链接契约；项目使用 Kotlin Activity，Android 从不调用此符号

## Kotlin 侧契约

- `app/src/main/java/com/leizm/zeroweb/NativeBridge.kt` — `System.loadLibrary("zero_android_browser")`，声明上述 28 个 `external` 方法
- `NativeRoleService.kt` — AIDL `IRoleService.Stub`：`onCreate` 调 `nativeStartRole` 校验，`start(socket: ParcelFileDescriptor)` 调 `nativeRunRole(role, slot, fd)` 并 `socket.detachFd()` 移交所有权；三个 role 均走 native 线程，未知 role 拒绝

## 构建

无人值守一律走 Makefile 入口（test-guard 包裹 gradle，阈值见 run-rules #14；`ANDROID_HOME` 默认 `$HOME/Android/Sdk`，gradle 侧 SDK 路径另走 `apps/android-browser/local.properties`）：

```bash
make android-apk            # 模拟器 Debug（x86_64）→ app/build/outputs/apk/emulator/debug/app-emulator-debug.apk
make android-release-apk    # arm64 Release → app/build/outputs/apk/arm64/release/app-arm64-release.apk
make android-renderer-apk   # arm64 Release + renderer feature（V8 从源码交叉编译；六项前置环境 + 代理，见 evidence/v8-renderer-path.md）
```

- **依赖**：Android SDK platform 36、NDK（经 cargo-ndk 驱动）、rustup target `aarch64-linux-android` / `x86_64-linux-android`、JDK 17；`make android-preflight` 自校验
- **Rust 侧**：Gradle `buildRustNative` 任务以本目录为 inputs 调 `cargo ndk`（`-PuseWslRenderer` 时改调 `scripts/android/build-native-wsl.sh`），so 产物注入 `app/build/generated/jniLibs`
- **签名**：从 `local.properties` 读 `zeroweb.keystore.path` / `zeroweb.keystore.password` / `zeroweb.keystore.alias`（默认 `zeroweb`），三项齐备才启用 release 签名，否则回退 unsigned；keystore 生成命令见 `app/build.gradle.kts` 头部注释
- **CI**：`.github/workflows/ci.yml` android job — aarch64 clippy（`-D warnings`，cargo ndk 包裹）+ 宿主单测 + `assembleArm64Release` + APK 产物上传

## 测试

宿主单测 16 项（`src/lib.rs` 14 项 + `src/facade.rs` 2 项）：角色白名单（`only_declared_process_roles_are_accepted`）、renderer 槽位映射与 LRU 分配/逐出/回收、页面输入边界（tap/viewport/帧尺寸/文本批量/按键白名单）、compositor 尺寸校验、下载文件名净化、HTTP 头大小写不敏感、URL 校验、版本串前缀等。JNI 导出大多 `#[cfg(target_os = "android")]` 门控，桌面可跑；CI android job 执行 `cargo test -p zero-android-browser`，本地无人值守全量走 `make test`（test-guard 包裹）：

```bash
cargo test -p zero-android-browser
```

安装冒烟（四进程拓扑 / UID 隔离 / probe 断言）与 chaos 冒烟（角色进程 kill → 断连恢复重绑）分别由 `make android-install-smoke` / `make android-chaos-smoke` 驱动，断言脚本为 `scripts/android/install-smoke.sh` / `chaos-smoke.sh`。

## 相关文档

- Android 浏览器实现 RFC：`docs/specs/android-browser-spec-rfc.md`
- 多进程协议与 Android socket 传输：`crates/protocol/src/transport.rs`（`android_socket_transport_from_fd`）
- 冒烟与验收证据：[docs/goal/android-browser/evidence/](../../../docs/goal/android-browser/evidence/)（工具链 bootstrap、renderer 路径、CI 冒烟、真机验收清单等）
