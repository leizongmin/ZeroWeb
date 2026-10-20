# 归档：立项基线与 M0→M4 代码切片流水（2026-09-07 ~ 2026-09-12）

**归档日期**: 2026-09-12（master.md 压缩时移入；只追加不修改）
**内容**: 立项实测基线 + 13 个切片的执行流水，权威结论已并入 master.md 当前状态表。

## 立项实测基线（2026-09-07）

- ✅ Kotlin 侧：MainActivity.kt（478 行 Compose UI）+ NativeBridge.kt（22 external fun）
  + NativeRoleService.kt（renderer×8/compositor/image-decoder）+ AIDL + 中英资源
- ✅ Rust 侧：apps/android-browser/rust lib.rs 949 行 22 个 JNI 导出 + facade.rs 228 行
  （复用 BrowserShell）；PipeTransport 多进程角色；`NATIVE_VERSION = "ZeroWeb Android M2"`
- ✅ 已落地：renderer fetch 代理、renderer→compositor 帧转发、compositor 帧 Bitmap 回传、
  滚动转发、WSL renderer 构建产物
- ⚠️→✅ CI：8 个 yml 零 Android job → M1 切片 2 补齐（run 34665015108 全绿）
- ⚠️→✅ renderer transport adapter：`73f2d7d57` 已落地（立项时误判未完成，2026-09-12 核实修正）
- ⚠️→✅ 版本串/README 滞后：M1 切片 3 对齐
- ⚠️→✅ RFC 待确认：2026-09-12 用户批准，FR-006/007/009 解锁

## 切片流水（2026-09-12 单日执行）

| # | 里程碑 | 内容 | 提交/证据 |
|---|--------|------|-----------|
| 1 | M1 | 本地打通 NDK 交叉编译 + Gradle assemble（277 crate）；renderer/V8 路径二次打通（V8 150.2.0 源码 29m33s，三轮踩坑修复） | evidence/local-toolchain-bootstrap.md、v8-renderer-path.md |
| 2 | M1 | ci.yml android job（cargo ndk clippy + 宿主单测 + assembleArm64Release + artifact）首跑全绿；修 android-cfg clippy 盲区 | 3f5fad847、eb5d3014e |
| 3 | M1 | README/版本串对齐（M2、transport adapter 表述） | 601138470 |
| — | M2 | renderer APK Linux 出包 + release 签名（keystore 三项，缺省回退 unsigned）；make android-renderer-apk | ddfea869a |
| — | M2 | 宿主侧纯逻辑单测入库（roles/尺寸契约/版本串） | dc135308a |
| 1' | M3 | renderer 断连恢复：recv 断连/握手失败清槽 + nativeIsRendererAttached + Kotlin 导航失败重绑（6b1ee64ce） | |
| 2' | M3 | 多标签换槽：facade tab→slot 亲和表（LRU 逐出/回收）、native 8 槽注册表、snapshot activeRendererSlot（e226fe361）+ 被逐标签切回自动重导航 + rebind 节流（9f61b903b） | |
| 3 | M3 | compositor 断连恢复：compositor_round 收敛三处往返、失败清槽 + nativeDetachCompositor + Kotlin onServiceDisconnected 重走 attach（ff71067ad） | |
| 4 | M3 | 预览点击 → DOM click：detectTapGestures 归一化 → nativePageTap → MouseEvent(Click)，复用 renderer hit-test/focus/表单语义（0ac92f9d4） | |
| 5 | M3 | viewport 真实尺寸贯通：computePageViewport 按 display 注入 (w,h,density)、每槽视口注册表、帧带 8 字节小端尺寸头、clear_renderer_slot 收敛清槽（4f8b01134）；修 android-cfg E0515 | |
| 6 | M3 | 键盘/IME 输入通路：PageInputView（BaseInputConnection）→ nativePageText（ImeEvent::Commit，CJK）/nativePageKey（Backspace/Enter 白名单）（b522b1b41） | |
| 7 | M4 | FR-006 下载接管：attachment 拦截 → facade::record_download 落盘 + DownloadManager 完成态；derive_download_filename 消毒（bf06248ba）；修 android-cfg E0382 | |
| 8 | M4 | 多标签缩略图：snapshot tabs 增 rendererSlot + Kotlin 112px 缩略图缓存（06718bb8e） | |
| 9 | M4 | 双语资源与 chrome 无障碍：35 键 EN/zh-rCN 双资源、32 处硬编码中文迁移（45924862f） | |
| 10 | M4 | 下载系统通知：快照水位线驱动 + NotificationChannel + POST_NOTIFICATIONS 静默降级（4ce04fff7） | |
| 11 | M4 | FR-006 SAF 导出与打开：CreateDocument + FileProvider/ACTION_VIEW + file_paths.xml（744be140b） | |
| 12 | M3/M5 | 验收就绪包：install-smoke.sh Linux 移植 + make android-install-smoke 接通 + device-acceptance-checklist.md（6171653bc） | |
| 13 | M5 | FR-010 依赖与许可证清单：license-manifest.sh 可重复生成，341 包全宽松许可（1ecff8051） | |

**共性验证口径**（每切片）：cargo fmt + host/android-target/workspace clippy（-D warnings）+
crate 宿主单测 + `make guarded-test` 全绿 + `make android-renderer-apk`（JNI 导出 nm 验证）。
