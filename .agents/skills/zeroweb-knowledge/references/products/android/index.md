# Android 浏览器宿主

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 Kotlin/Compose 与 Rust JNI 的移动产品边界。`MainActivity` 管系统 UI 与 Service 绑定，`facade` 管可恢复业务状态及租户表，`lib.rs` 管角色 transport、输入与帧；APK/工具链配方见[打包知识](../../engineering/build-and-release/packaging.md)。

### User Task Navigation
- 标签/收藏/历史/下载、Intent、通知或导出：`MainActivity`/`snapshot_locked` → [界面与状态](chrome-and-state.md)。
- 断连、9 个以上标签或切回空白：`assign_active_tab_slot`/`clear_renderer_slot` → [角色与 JNI](roles-and-jni.md)。
- 点击偏移、键盘或旧预览：`computePageViewport`/`PageInputView`/`encode_page_frame` → [输入与页面帧](page-input-and-surface.md)。
- 真实设备判据查 [Android 验证地图](../../../../zeroweb-verify/features/android.md)；本轮没有设备运行，地图为工作区新增，不归上述 HEAD。

## Gotchas
- 业务 profile 恢复、Service attach 成功与用户页面帧到达是三个不同阶段；页面预览初始 smoke 帧不证明用户 URL 已经恢复。 (`apps/android-browser/rust/src/facade.rs`, `apps/android-browser/rust/src/lib.rs`, `apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`)

## Architecture
- 槽亲和与系统进程连接为易失状态，Shell 的可序列化标签是长期状态；恢复/逐出应重建连接而不清掉标签业务记录。 (`apps/android-browser/rust/src/facade.rs`, `apps/android-browser/app/src/main/java/com/leizm/zeroweb/NativeRoleService.kt`, `apps/android-browser/rust/src/lib.rs`)

## Child Knowledge Nodes
- [Compose 界面、Rust 状态与系统集成](chrome-and-state.md) — 快照、profile、Intent、通知、SAF 与文案边界。
- [JNI、隔离 Service、槽位与恢复](roles-and-jni.md) — FD 所有权、隔离级别、8 槽 LRU、清理及重绑。
- [触摸、IME、视口与页面帧](page-input-and-surface.md) — CSS/物理坐标、软键盘输入、帧编码与定时预览。
