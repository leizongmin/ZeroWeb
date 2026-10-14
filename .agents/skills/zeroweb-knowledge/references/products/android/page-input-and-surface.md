# 触摸、IME、视口与页面帧

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 Android 页面坐标、软键盘、滚动、位图与标签预览。MainActivity 换算 display/Compose 输入，JNI 使用本槽 viewport 下发消息并读取 compositor 帧；当前显示链是定时抓取预览位图，不是连续 Android Surface。

关键入口：`computePageViewport`、`validate_page_viewport`、`encode_page_frame`、`PageInputView`、`toPageBitmap`。底层输入见[交互契约](../../runtime/page-runtime/interaction-contract.md)，帧合法性见[帧交付](../../runtime/protocol/frame-delivery.md)。

## Gotchas
- computePageViewport 用 display 物理宽/density 推算 CSS 宽并夹到 320..480，高按纵横比夹到 320..1024，density 夹到 1..3 并考虑单边 1280 目标；这是有界近似，不是 Compose 页面实际可用高度的直接测量。 (`apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`)
- native viewport 拒绝零尺寸、单边大于 4096、非有限 density 或不在 (0,8] 的 density；Kotlin 的正常计算范围不能代替 JNI 信任边界校验。 (`apps/android-browser/rust/src/lib.rs`)
- 点击先归一化到 0..1，再乘当前槽 CSS viewport，滚动要求有限且绝对值不超过 4096，并以视口中心定位；用 slot0 的尺寸代替当前槽会让多标签输入偏移。 (`apps/android-browser/rust/src/lib.rs`)
- 滚动同时发送 renderer ScrollEvent 与 compositor 滚动/取帧请求，只有一侧变化会导致输入已生效但预览仍旧，或预览偏移却页面状态未更新。 (`apps/android-browser/rust/src/lib.rs`, `crates/protocol/src/message.rs`)
- 页面文本输入非空且最多 4096 个 UTF-8 字节，经 Ime Commit 转发；特殊键只支持 Backspace/Enter，并分别发送 keydown 与 keyup，其他软键盘操作不能按完整键盘能力推断。 (`apps/android-browser/rust/src/lib.rs`, `apps/android-browser/app/src/main/java/com/leizm/zeroweb/NativeBridge.kt`)
- PageInputView 是 1dp 的输入连接宿主，键盘由显式开关控制，当前不转发 preedit；deleteSurroundingText 忽略请求删除长度，只发送一次 Backspace。 (`apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`)
- compositor 帧必须尺寸合法且 rgba.len()==width*height*4，JNI 输出 8 字节小端 width/height 加 RGBA，Kotlin 再复核；不能再按旧的固定 320×180 解码。 (`apps/android-browser/rust/src/lib.rs`, `apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`, `git:a99e4ed33553d98518b9c422d63cc26b066af3bb`)
- frame 按 surface_id==slot 归档，指定槽没有帧就返回无帧，不能拿 slot0 的帧兜底，否则会把另一标签页面当成当前内容。 (`apps/android-browser/rust/src/lib.rs`)
- 页面预览在 1/5/10 秒延迟抓取；非活跃标签缩略图首次取 112px 图，预览有内容不证明后续动画持续更新。 (`apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`)
- refreshRendererPreview 的延迟任务捕获当次 targetSlot，执行时直接更新公共 rendererPreview，没有复核活动标签或槽；切标签后旧任务仍可覆盖当前预览，不能把按槽 JNI 缓存正确推断成 Compose 显示身份正确。 (`apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`)

## Architecture
- 槽 viewport 同时决定 SetViewport、点击坐标和页面帧解码，清槽时同步清 viewport；这三处由固定小画面迁移成真实 display 推算时共同改变，不能只改截图尺寸。 (`apps/android-browser/rust/src/lib.rs`, `apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`, `git:a99e4ed33553d98518b9c422d63cc26b066af3bb`)

## Testing Strategy
- 帧测试分开检查 page_frame_dims 的合法性和 encode_page_frame 的字节格式；后者特意用不匹配 320×180 的四字节载荷证明它只是编码器，生产调用点须先校验尺寸与载荷。 (`apps/android-browser/rust/src/lib.rs`)
- JNI 纯函数测试覆盖 NaN/越界点击、412×914×2.625 视口、4096/4097 字节输入边界与键白名单，但不覆盖 Android InputConnection、Service 或位图显示；宿主测试不能替代设备输入验收。 (`apps/android-browser/rust/src/lib.rs`, `apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`)
