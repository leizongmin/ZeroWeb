# Android 浏览器可用化 — 运行时控制面板（master.md）

**入口文档**: [../android-browser.md](../android-browser.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-07（立项——M1 待启动）

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
- ⚠️ 版本串不一致：lib.rs `M2` vs README `M0`（文档滞后）
- ⚠️ RFC `docs/specs/android-browser-spec-rfc.md`（1097 行）状态「待确认」；
  FR-006/007/009 未见对应代码
- ⚠️ 无 arm64 真机验收记录、无 Release APK 交付记录

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | Android CI job（NDK 构建 + assemble + 单测） | ⬜ M1 |
| P2 | 版本串/README/RFC 状态文档滞后 | ⬜ M1 |
| P3 | JNI 桥接测试覆盖 + 进程角色冒烟断言 | ⬜ M2 |
| P4 | APK 构建入口 + 构建文档 | ⬜ M2 |
| P5 | 模拟器冒烟证据 + RFC 决策清单 | ⬜ M3 |

## 下一步计划

1. **M1 切片 1**：本地打通 NDK 交叉编译 android-browser/rust + Gradle assemble，
   记录依赖与可重复命令到 evidence/
2. **M1 切片 2**：CI Android job 落地（照既有 build-and-test 矩阵风格；不 continue-on-error）
3. **M1 切片 3**：README/版本串对齐

**碰撞管理**：Cargo.lock 变更前 `git log --since="14 days ago" -- Cargo.lock` 核对；
只读消费其他 crate 公开 API。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — CI 门禁 + 构建修复 | ⬜ 待启动 |
| M2 — 回归保护 + 可安装产物 | ⬜ |
| M3 — 冒烟验收 + 决策清单 | ⬜ |

## 待用户决策

| 项 | 状态 | 说明 |
|----|------|------|
| android-browser-spec-rfc.md 批准 | ⬜ 待确认 | transport adapter、FR-006/007/009 实施的前提 |
| 真机验收设备 | ⬜ 等设备 | 同父目标 P3 GPU 物理机门控模式；模拟器冒烟不阻塞 |

## 验证基线

- 测试基线：立项时点全绿（`make test` 入口，经 test-guard 包裹；禁止裸跑 cargo test）
- Android 构建：无 CI 基线（M1 建立）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
