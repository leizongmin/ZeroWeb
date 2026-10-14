# JNI、隔离 Service、槽位与恢复

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 Android FD 所有权、Service 断连、8 槽 LRU、重绑及恢复导航。facade 管 tab→slot 租户，JNI 管每槽 transport/viewport/frame，NativeRoleService 通过 AIDL 启动对应 Rust 角色。

关键入口：`assign_active_tab_slot`、`touch_active_tab_slot`、`clear_renderer_slot`、`nativeAttachRenderer`、MainActivity 的 `pendingRestoreUrl`。消息契约见[IPC 生命周期](../../runtime/protocol/messages-and-lifecycle.md)，帧协议见[帧交付](../../runtime/protocol/frame-delivery.md)。

## Gotchas
- Manifest 预声明 RendererService0..7 为 isolatedProcess，image-decoder 也隔离；compositor 仅运行在私有 :compositor 进程，不带 isolatedProcess，不能把“进程外”解释成相同 UID 隔离级别。 (`apps/android-browser/app/src/main/AndroidManifest.xml`)
- Service 通过 socketpair 和 ParcelFileDescriptor 交 AIDL，detachFd 后交给 Rust 接管；Kotlin 再关闭同一个 detached 原始 FD 会破坏 transport 所有权。 (`apps/android-browser/app/src/main/java/com/leizm/zeroweb/NativeRoleService.kt`, `apps/android-browser/app/src/main/aidl/com/leizm/zeroweb/IRoleService.aidl`, `apps/android-browser/rust/src/lib.rs`)
- 槽优先取空闲，8 槽满后逐出最久未用租户，切回已有槽必须 touch 时钟；逐出不删除标签业务记录，切回被逐标签需要重新导航。 (`apps/android-browser/rust/src/facade.rs`, `git:571bc37de179e360ca88e781a2753b9b4eedd2df`)
- assign_active_tab_slot 返回槽号与被逐出的 TabId，实际 transport 清理由 JNI 执行；仅更新 facade 租户表会保留旧连接。 (`apps/android-browser/rust/src/facade.rs`, `apps/android-browser/rust/src/lib.rs`)
- clear_renderer_slot 同时清 transport 和 viewport，reader EOF 也走清理；只把 Kotlin attached 标志改为 false 无法消除 Rust 僵尸连接。 (`apps/android-browser/rust/src/lib.rs`)
- renderer 重绑最多重试 3 次，attach 成功后补 pendingRestoreUrl；Service 已绑定不等于新槽已经完成用户 URL 导航。 (`apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`)
- compositor send/recv 失败与 ServiceDisconnected 会清僵尸 transport 以允许重新 attach，当前恢复链不回退进程内 compositor。 (`apps/android-browser/rust/src/lib.rs`, `apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`)
- renderer attach 顺序是 SetViewport、SetFramePublishMode(Compositor)、内联 smoke LoadHtml，握手失败清槽；attach 后出现的测试帧不能作为用户网页加载成功证据。 (`apps/android-browser/rust/src/lib.rs`)
- attach 握手的“成功”仅是 send_renderer_to_slot 写出消息成功，没有等待 renderer 确认 SetViewport；视口注册随后写入，不能把 nativeAttachRenderer=true 表述成对端已完成布局。 (`apps/android-browser/rust/src/lib.rs`)

## Architecture
- 业务状态和连接状态分层：facade 保存可恢复标签与 LRU 亲和，JNI 注册表保存易失连接，MainActivity 负责系统绑定；清 transport 不应顺带清 profile 标签。 (`apps/android-browser/rust/src/facade.rs`, `apps/android-browser/rust/src/lib.rs`, `apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`)
- Android native 库为 cdylib，android-renderer feature 才接可选 zero-renderer，且 renderer 依赖位于 Android target 配置；宿主 facade 测试能通过不证明角色入口在 APK 已接线。 (`apps/android-browser/rust/Cargo.toml`, `apps/android-browser/app/src/main/java/com/leizm/zeroweb/NativeRoleService.kt`)

## Patterns
- facade 的 LRU 用例在一个测试函数里串行驱动全链，因为 BROWSER 是进程级 OnceLock/Mutex 单例；拆成并行用例会互相替换 profile 与租户表。 (`apps/android-browser/rust/src/facade.rs`)
- snapshot_locked 已持有 browser 锁，activeRendererSlot 必须内联读 tab_slots，不调用再次取同一 Mutex 的 active_tab_slot；只读查询也可能造成单线程重入死锁。 (`apps/android-browser/rust/src/facade.rs`)
- LRU 判据不是“第九个标签分到某槽”，而是先占满 0..7、touch 槽0、再断言逐出槽1及其旧 TabId，并核对快照各标签 rendererSlot；仅计数测试会漏掉切回标签未更新时间的错误。 (`apps/android-browser/rust/src/facade.rs`)
