# Android 浏览器可用化 — 运行时控制面板（master.md）

**入口文档**: [../android-browser.md](../android-browser.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-12（M3 切片 6 键盘/IME 输入通路落地——复用桌面 ImeEvent IPC；模拟器环境核查记入 evidence/emulator-feasibility.md）

---

## 当前状态

**专项定位**：Android 线治理与可用化。现状 = 功能中期（M2 级：22 JNI 导出、四类进程角色、
帧桥接/滚动/网络代理已落地）、治理缺位（零 CI、零 goal 追踪、文档滞后）。先治理（CI 门禁）
后验收（冒烟证据）。

**与兄弟 goal 的边界**：
- rendering-compat — 渲染流域 crate 域零重叠；android-browser/rust 只读消费
  zero-protocol/zero-compositor/zero-image-decoder 公开 API，需要改它们时停下记录
- page-wasm / storage-opfs / webdriver — 无共享面
- 共享面：Cargo.lock（依赖变更前 `git log` 核对，冲突即暂缓，run-rules §9）

## 实测基线（2026-09-07 立项时）

### 现有实现

- ✅ Kotlin 侧：MainActivity.kt（478 行 Compose UI）+ NativeBridge.kt（22 external fun）
  + NativeRoleService.kt（renderer×8/compositor/image-decoder）+ AIDL + 中英资源
- ✅ Rust 侧：apps/android-browser/rust（workspace member）lib.rs 949 行 22 个 JNI 导出 +
  facade.rs 228 行（复用 BrowserShell）；PipeTransport 多进程角色；`NATIVE_VERSION =
  "ZeroWeb Android M2"`
- ✅ 已落地：renderer fetch 代理、renderer→compositor 帧转发、compositor 帧 Bitmap 回传、
  滚动转发、WSL renderer 构建产物
- ⚠️ CI：`.github/workflows/` 8 个 yml **零 Android job**
- ⚠️ renderer Android transport adapter 未完成（README 自述「后续 M1 切片」——RFC 域，
  本 goal 不实施）
  → 2026-09-12 核实修正：adapter 已于 `73f2d7d57` 落地（`run_android_role` 完整 runtime
  管线），Kotlin `RendererService0-7` 经 `nativeRunRole(role, slot, fd)` 交 FD，slot→
  `renderer_id` 透传本轮补齐；剩余真缺口=多标签换槽接线（MainActivity 现只绑 slot 0）
  与真机验证（M3）。默认 release 构建不含 renderer feature（renderer-less shell 产物，
  `nativeRendererLinked()` 门控），完整版走 `make android-renderer-apk`
  → 2026-09-12 续：**M3 切片 1 renderer 断连恢复落地**（`6b1ee64ce`）——recv 断连清槽 +
  attach 握手失败清槽 + `nativeIsRendererAttached` 查询 + Kotlin 导航失败重绑
  RendererService0
  → 2026-09-12 续：**M3 切片 2 多标签换槽落地**（`e226fe361`）——facade tab→slot 亲和表
  （空闲优先/LRU 逐出/关闭回收，host 测试覆盖）、native 8 槽注册表（transport/帧缓冲/
  meta 全部按槽），navigate 路由活动槽、fetch 响应按来路槽回、snapshot 增
  `activeRendererSlot` 驱动 Kotlin 按槽绑槽与预览
  → 2026-09-12 续：**被逐标签切回自动重导航落地**——快照驱动恢复（有 URL 无附着槽 →
  导航分配槽 → 失败按槽重绑 → attach 后补导航），启动恢复上次会话活动标签、外部
  intent 新标签同链路；rebind 节流（3 次/选择周期）防循环；被逐标签预览留空不再
  错拿他槽帧。多标签换槽功能面就此闭环，余真机冒烟（M3 收口）
  → 2026-09-12 续：**M3 切片 3 compositor 断连恢复落地**（RFC M3「renderer/compositor
  death 处理」后半，对称切片 1）——三处 compositor send/recv 往返收敛进 `compositor_round`
  助手，传输失败即清槽 + `tracing::warn`；新增 `nativeDetachCompositor`；Kotlin
  `onServiceDisconnected(CompositorService)` 作废附着标记并清 native 僵尸 transport，
  BIND_AUTO_CREATE 重启后 `onServiceConnected` 重新走 attach 协议。宿主 7/7 + android
  clippy + renderer APK（25 JNI 导出含新导出）构建通过
- ⚠️ 版本串不一致：lib.rs `M2` vs README `M0`（文档滞后）
- ⚠️ RFC `docs/specs/android-browser-spec-rfc.md`（1097 行）状态「待确认」；
  FR-006/007/009 未见对应代码
- ⚠️ 无 arm64 真机验收记录、无 Release APK 交付记录

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | Android CI job（NDK 构建 + assemble + 单测） | ✅ M1——ci.yml android job（34665015108 全绿） |
| P2 | 版本串/README/RFC 状态文档滞后 | 🔶 M1——README/版本串已对齐（601138470）；RFC 状态待批准 |
| P3 | JNI 桥接测试覆盖 + 进程角色冒烟断言 | 🔶 M2——宿主侧纯逻辑单测 5 项（roles/尺寸契约/版本串）已入；真机/模拟器冒烟属 M3 |
| P4 | APK 构建入口 + 构建文档 | ✅ M2——`make android-apk/release-apk/renderer-apk`（test-guard 包裹）+ 签名（local.properties keystore，回退 unsigned）+ evidence 两篇 |
| P5 | 模拟器冒烟证据 + RFC 决策清单 | ⬜ M3 |

## 下一步计划

1. **M1 切片 1**：本地打通 NDK 交叉编译 android-browser/rust + Gradle assemble，
   记录依赖与可重复命令到 evidence/
   → 2026-09-12 基础路径已打通（默认 feature、无 V8）：cargo 交叉编译 277 crate 通过 +
   `make android-release-apk` 产出 unsigned APK；依赖与可重复命令见
   [evidence/local-toolchain-bootstrap.md](evidence/local-toolchain-bootstrap.md)。
   剩余：renderer feature 路径（V8 从源码，5 项前置缺 3）、gradlew 可执行位入库修复
   → 2026-09-12 续：**renderer/V8 路径亦已本地打通**——六项前置装齐、`build-native-wsl.sh
   arm64-v8a` 产出 90M 含 V8 的 .so（29m33s，三轮踩坑修复，含补丁新增 BUILDCONFIG
   declare_args hunk），见 [evidence/v8-renderer-path.md](evidence/v8-renderer-path.md)
2. **M1 切片 2**：CI Android job 落地（照既有 build-and-test 矩阵风格；不 continue-on-error）
   → 2026-09-12 ✅ ci.yml android job 首跑全绿（run 34665015108）：clippy android target
   （经 cargo ndk 包裹）+ 宿主单测 + assembleArm64Release + APK artifact。修复两处
   android-cfg clippy 盲区 lint（3f5fad847）。M1 全部切片完成
3. **M1 切片 3**：README/版本串对齐
   → 2026-09-12 README 两处已对齐（版本串 M2、transport adapter 表述改待 RFC）；RFC 状态行仍待用户批准
4. **RFC M3 剩余功能面**（RFC 批准后按 M2→M4 排期推进）
   → 2026-09-12 ✅ 切片 3 compositor 断连恢复（见上）
   → 2026-09-12 ✅ 切片 4 预览点击 → DOM click：Kotlin `detectTapGestures` 按预览显示区
   归一化坐标 → `nativePageTap(normX, normY)`（纯函数 `tap_viewport_point` 校验
   0..=1/有限值，宿主测试覆盖）→ 活动槽 `MouseEvent(Click)`，复用 renderer 桌面
   hit-test/focus/表单提交语义
   → 2026-09-12 ✅ 切片 5 viewport 真实尺寸贯通——替换写死 320×180：Kotlin
   `computePageViewport()` 按真实 display 推算（CSS 宽 = 物理宽/density 夹在 320-480，
   高按纵横比，密度单边帧上限 1280 内下调）；`nativeAttachRenderer` 增 (w,h,density)
   参数，`validate_page_viewport` 纯函数校验（宿主测试）后作该槽 SetViewport 与每槽
   视口注册表（tap/滚动光标换算同源）；帧出槽带 8 字节小端尺寸头
   （`encode_page_frame`），`page_frame_dims` 校验 compositor 回帧有界自洽；清槽点
   收敛 `clear_renderer_slot`（transport+视口同清）
   → 2026-09-12 ✅ 切片 6 键盘/IME 输入通路——零 renderer/protocol 改动，复用桌面
   `ImeEvent` IPC：Kotlin `PageInputView`（1dp 隐形 AndroidView + `BaseInputConnection`，
   「键盘」开关按钮弹/收软键盘）→ `commitText` → `nativePageText`（ImeEvent::Commit，
   CJK 主通路）、删除/回车 → `nativePageKey`（Backspace/Enter 白名单 → keydown+keyup，
   renderer 侧默认动作删字/提交表单）。纯函数校验（文本 ≤4096 字节、键白名单）宿主
   测试覆盖。RFC M3「焦点、IME」项就此闭合（焦点已随点击落位）
5. **模拟器/真机冒烟（M3 收口）**：等 KVM 授权或设备（待用户决策，不阻塞功能切片）

**碰撞管理**：Cargo.lock 变更前 `git log --since="14 days ago" -- Cargo.lock` 核对；
只读消费其他 crate 公开 API。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — CI 门禁 + 构建修复 | ✅ 全切片完成（CI job 全绿 34665015108、本地双路径打通、README/版本串对齐） |
| M2 — 回归保护 + 可安装产物 | 🔶 APK 入口/签名/构建文档 ✅（P4）；JNI 桥接测试宿主侧 7 项已入、真机/模拟器冒烟断言待 M3 设备面（P3） |
| M3 — 冒烟验收 + 决策清单 | 🔶 决策清单 ✅（RFC 已批准）；RFC M3 功能面（断连恢复×2/多标签换槽/点击/viewport/键盘 IME）✅ 全部落地，模拟器冒烟受 KVM 环境阻塞（见待用户决策），真机待设备 |

## 待用户决策

| 项 | 状态 | 说明 |
|----|------|------|
| android-browser-spec-rfc.md 批准 | ✅ 已批准（2026-09-12） | transport adapter、FR-006/007/009 解锁，按路线 M2→M4 排期 |
| 真机验收设备 | ⬜ 等设备 | 同父目标 P3 GPU 物理机门控模式；模拟器冒烟不阻塞 |
| 本机模拟器 KVM 授权 | ⬜ 等用户一次性授权 | WSL2 `/dev/kvm` 存在但用户不在 kvm 组且 sudo 需密码，模拟器无法启动；`sudo usermod -aG kvm lei` 一次即可解锁（详情 evidence/emulator-feasibility.md）。解锁前 RFC M3 功能切片继续推进，不阻塞 |

## 验证基线

- 测试基线：立项时点全绿（`make test` 入口，经 test-guard 包裹；禁止裸跑 cargo test）
- Android 构建：无 CI 基线（M1 建立）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
