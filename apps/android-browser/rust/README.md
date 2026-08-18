# ZeroWeb Android Browser (`zero-android-browser`)

> ZeroWeb Android 浏览器主机的 JNI 桥接层 — 在 Kotlin Activity 宿主中按角色复用共享 Rust 多进程运行循环

## 概述

`zero-android-browser` 是 Android 端浏览器（应用 ID `com.leizm.zeroweb`）的原生库（`cdylib`），为 Kotlin/Jetpack Compose 宿主提供 JNI 入口：校验服务角色、把 detached socket FD 交给共享 Rust 角色循环，并暴露引导屏版本号与 decoder/compositor socket 探针。它保持 renderer、compositor、image-decoder 的物理多进程隔离，其中 decoder 与 compositor 已走原生线程运行（复用 `zero-image-decoder` / `zero-compositor` 的 `run_role`），renderer 暂保持 Kotlin Service 拓扑，Android transport adapter 在后续 M1 切片完成。

设计背景见 [docs/specs/android-browser-spec-rfc.md](../../../docs/specs/android-browser-spec-rfc.md)。

## 主要功能

- **nativeVersion** — 返回引导屏显示的版本串（`ZeroWeb Android M0`）
- **nativeStartRole** — Service 进程报告就绪前校验 role（仅接受 `renderer` / `compositor` / `image-decoder`）
- **nativeRunRole** — 取得 detached socket FD（`zero_protocol::android_socket_transport_from_fd`），按角色起线程运行 `zero_image_decoder::run_role` / `zero_compositor::run_role`；FD 所有权由 Kotlin 侧 `ParcelFileDescriptor.detachFd()` 移交，失败路径负责 `close`
- **nativeProbeDecoder** — 经 socket 发送畸形 `ImageDecodeRequest`，校验错误回复
- **nativeProbeCompositor** — `RegisterUiSurface → Ok / UiFrame → Ok / GetCompositorUiFrame` 帧数据往返校验
- **android_main 锚点** — 满足 winit native-activity 链接契约；项目使用 Kotlin Activity，Android 从不调用此符号

## Kotlin 侧契约

- `app/src/main/java/com/leizm/zeroweb/NativeBridge.kt` — `System.loadLibrary("zero_android_browser")`，声明上述五个 `external` 方法
- `NativeRoleService.kt` — AIDL `IRoleService.Stub`：`onCreate` 调 `nativeStartRole` 校验，`start(socket: ParcelFileDescriptor)` 调 `nativeRunRole` 并 `socket.detachFd()` 移交所有权；仅 decoder/compositor 两个 role 走 native 线程，其余拒绝

## 构建

由 Gradle 集成构建（`app/build.gradle.kts` 以本目录为 inputs 调 cargo）：

```bash
# 在 apps/android-browser 下经 Gradle 构建（arm64 Release / x86_64 Debug）
./gradlew :app:assembleDebug
```

## 测试

src 内嵌单测 `only_declared_process_roles_are_accepted`（角色白名单校验）；JNI 导出大多 `#[cfg(target_os = "android")]` 门控，桌面 `cargo test` 覆盖角色校验逻辑：

```bash
cargo test -p zero-android-browser
```

## 相关文档

- Android 浏览器实现 RFC：`docs/specs/android-browser-spec-rfc.md`
- 多进程协议与 Android socket 传输：`crates/protocol/src/transport.rs`（`android_socket_transport_from_fd`）
