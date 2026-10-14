# Compose 界面、Rust 状态与系统集成

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 Android 标签/收藏/历史/下载列表、外部 Intent、通知、SAF 导出与宿主恢复。MainActivity 消费 NativeBridge 的 JSON 快照，业务状态经 `facade.rs` 进入 BrowserShell；Kotlin 不是第二套持久业务模型。

关键入口：`load_profile`、`mutate`、`snapshot_locked`、MainActivity 的 Intent/通知/导出处理。[profile 数据模型](../browser-state/profile-and-tabs.md)负责持久化细节，[槽位恢复](roles-and-jni.md)负责进程连接。

验证导航：[Android 功能地图](../../../../zeroweb-verify/features/android.md)；需要设备核验 UI、权限和文件副作用。本轮没有设备实跑，地图为工作区新增，不属于上述 HEAD。

## Gotchas
- profile 根来自 filesDir/profile，恢复 tabs、bookmarks、history、downloads 各自失败降级；某类损坏不阻止其他类别恢复，不应通过“全部清空重建”实现单类错误恢复。 (`apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`, `apps/android-browser/rust/src/facade.rs`, `crates/browser-shell/src/browser.rs`)
- facade mutate 先改变内存，再保存 profile，保存成功才 revision++；JNI 返回 false 不表示内存回滚，因此不能仅凭布尔值恢复 Kotlin 旧快照。 (`apps/android-browser/rust/src/facade.rs`, `apps/android-browser/app/src/main/java/com/leizm/zeroweb/NativeBridge.kt`)
- snapshot 的 tabs 全量输出，但收藏、历史和下载各最多 200 项，Count 字段显示全量数量；列表长度与总数不相等不是解析失败。 (`apps/android-browser/rust/src/facade.rs`)
- Rust 导航校验先 trim，长度为 1..16384 个 UTF-8 字节，只接受大小写敏感 http:// 或 https:// 前缀，并非完整 URL 语法校验；Kotlin 外部 Intent 的 length 是 UTF-16 长度，最终拒绝规则仍以 Rust 为准。 (`apps/android-browser/rust/src/facade.rs`, `apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`)
- Android 关闭最后一个标签后会补 new_tab(None)，这是 facade 的宿主规则，不是底层 TabManager 永远非空的保证。 (`apps/android-browser/rust/src/facade.rs`, `crates/browser-shell/src/tab.rs`)
- profile 恢复会生成新的 TabId，tab→renderer slot 亲和表不落盘；恢复业务列表不表示旧进程槽仍可复用。 (`apps/android-browser/rust/src/facade.rs`, `crates/browser-shell/src/browser.rs`)
- 下载落到 profile/downloads 下，以 ID 与净化后的 filename 组合路径，失败记 Failed、成功记 Completed；净化在 JNI 网络接管入口完成而不是 facade，直接调用 record_download 的内部调用方必须提供已处理文件名。 (`apps/android-browser/rust/src/facade.rs`, `apps/android-browser/rust/src/lib.rs`)
- 下载通知的首个快照只建立完成记录水位，避免启动时重发历史通知；API 33 未授予通知权限时跳过，不表示下载本身失败。 (`apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`, `apps/android-browser/app/src/main/AndroidManifest.xml`)
- SAF 导出使用用户选择的 URI；FileProvider 只暴露 profile/downloads/ 且通过只读授权打开，不能把整个 profile 作为共享根。 (`apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`, `apps/android-browser/app/src/main/res/xml/file_paths.xml`, `apps/android-browser/app/src/main/AndroidManifest.xml`)

## Architecture
- NativeBridge 用显式 JNI 方法接业务动作和快照，Compose 页面根据快照渲染而非自行维护标签历史；更改 JSON 字段必须同时核对 Kotlin 解析与 Rust 输出。 (`apps/android-browser/app/src/main/java/com/leizm/zeroweb/NativeBridge.kt`, `apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`, `apps/android-browser/rust/src/facade.rs`)

## Patterns
- 下载文件名只解析大小写敏感 filename=，不解码 filename*；依次取 Content-Disposition、去 query/fragment 后的 URL 末段、download，过滤路径/控制/通配字符及全点名并限制 100 字符，不能按完整 RFC 文件名协商能力推断。 (`apps/android-browser/rust/src/lib.rs`)

## Conventions
- UI 文案与无障碍描述使用默认英文及 values-zh-rCN 同键资源，底层 JNI 失败与日志保持诊断文本；新增可见错误需在展示边界映射，而不是把 Rust 错误串直接当 UI 文案。 (`apps/android-browser/app/src/main/res/values/strings.xml`, `apps/android-browser/app/src/main/res/values-zh-rCN/strings.xml`, `apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`)
