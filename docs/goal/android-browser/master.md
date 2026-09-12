# Android 浏览器可用化 — 运行时控制面板（master.md）

**入口文档**: [../android-browser.md](../android-browser.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-13（P5 chrome 级模拟器冒烟 CI 落地并稳定绿：android-emulator-smoke
工作流三轮根因修复后连续两轮全绿，本机 KVM 门控对 chrome 级验收解除；M3 收口，M5 模拟器
侧解锁）

---

## 当前状态

**专项定位**：Android 线治理与可用化。**现状**：治理面（CI 门禁/构建入口/签名/文档/测试）
与 RFC M0→M4 代码面全部落地（28 个 JNI 导出、断连恢复×2、多标签换槽、点击/滚动/键盘 IME、
真实 viewport、下载全链路、缩略图、双语+无障碍）；**chrome 级模拟器冒烟已在 CI 落地并
稳定绿**（GitHub runner KVM，无需本机授权，见 evidence/ci-emulator-smoke.md）。
剩余：M5 质量面（性能/内存/chaos/安全负测试——CI 模拟器已解锁可推进）与真机端到端验收
（等设备）。

**与兄弟 goal 的边界**：
- rendering-compat — 渲染流域 crate 域零重叠；android-browser/rust 只读消费
  zero-protocol/zero-compositor/zero-image-decoder 公开 API，需要改它们时停下记录
- page-wasm / storage-opfs / webdriver — 无共享面
- 共享面：Cargo.lock（依赖变更前 `git log` 核对，冲突即暂缓，run-rules §9）

**历史**：立项基线与切片 1-13 流水见
[archive/2026-09-12-m0-to-m4-code-slices.md](archive/2026-09-12-m0-to-m4-code-slices.md)。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — CI 门禁 + 构建修复 | ✅ 完成（CI job 全绿、本地双路径打通、README/版本串对齐） |
| M2 — 回归保护 + 可安装产物 | ✅ 完成（构建入口/签名/构建文档；宿主侧纯逻辑单测 16 项，进程拓扑断言入 install-smoke） |
| M3 — 冒烟验收 + 决策清单 | ✅ 完成（决策清单 ✅ RFC 批准；**chrome 级模拟器冒烟 CI 稳定绿**——拓扑/UID 隔离/双 probe，连续两轮，evidence/ci-emulator-smoke.md；页面级/真机归 M4 验收） |
| M4 — 完整首期功能（RFC 路线） | 🔶 代码面全部落地（FR-006 全链路/缩略图/双语+无障碍）；**端到端待设备** |
| M5 — 质量与交付 | 🔶 验收清单/许可证清单 ✅；性能/内存/chaos/安全负测试**模拟器已解锁待执行**（CI 模拟器可跑）；验收报告待真机 |

## 缺口清单（当前）

| # | 缺口 | 状态 |
|---|------|------|
| P1 | Android CI job | ✅ ci.yml android job 全绿（34665015108） |
| P2 | 版本串/README/RFC 文档滞后 | ✅ 全部对齐（RFC 2026-09-12 批准） |
| P3 | JNI 桥接测试 + 进程角色冒烟断言 | ✅ 宿主侧纯逻辑单测 16 项 + install-smoke.sh 拓扑/UID/probe 断言 |
| P4 | APK 构建入口 + 构建文档 | ✅ make 三入口（test-guard 包裹）+ 签名 + evidence 文档 |
| P5 | 模拟器/真机冒烟**执行**与证据 | ✅ chrome 级：CI 模拟器稳定绿（34708223825/34708752317，evidence/ci-emulator-smoke.md）；真机端到端仍待设备 |

## 下一步（按序执行）

1. **M5 质量面（CI 模拟器已解锁）**：基于
   android-emulator-smoke job 扩展性能/内存/chaos/安全负测试——冷启动时长、
   断连恢复（kill renderer/compositor/decoder 进程树）、下载中断、内存峰值观测；
   证据入 evidence/。
2. **真机验收**（设备到位后）：`make android-renderer-apk` 产物安装 →
   按 [evidence/device-acceptance-checklist.md](evidence/device-acceptance-checklist.md)
   清单全项 + FR-006 端到端（下载/通知/导出/打开）+ FR-007 旋转/后台回收。
3. **发布前**：重跑 `scripts/android/license-manifest.sh` 刷新许可证清单；
   `make android-release-apk` 签名产物 + 验收报告入 evidence。

**碰撞管理**：Cargo.lock 变更前 `git log --since="14 days ago" -- Cargo.lock` 核对；
只读消费其他 crate 公开 API。

## 待用户决策

| 项 | 状态 | 说明 |
|----|------|------|
| android-browser-spec-rfc.md 批准 | ✅ 已批准（2026-09-12） | transport adapter、FR-006/007/009 解锁，按路线 M2→M4 排期 |
| 真机验收设备 | ⬜ 等设备 | 同父目标 P3 GPU 物理机门控模式；chrome 级冒烟已不阻塞（CI 模拟器） |
| 本机模拟器 KVM 授权 | ⬜ 等用户一次性授权（已征询；**非阻塞**） | CI 模拟器已覆盖 chrome 级验收，本机授权仅影响本地模拟器调试便利性。WSL2 `/dev/kvm` 存在但用户不在 kvm 组且 sudo 需密码（详情 evidence/emulator-feasibility.md）。**GB-20260912 巡检飞书征询 msg `om_x100b6561b60c38a0c27902f51665a6e`（2026-09-12）** |

## 验证基线

- 测试基线：`make guarded-test` 全绿（test-guard 包裹；禁止裸跑 cargo test）
- Android 构建：CI android job + 本地 `make android-apk/release-apk/renderer-apk`（签名产物）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
- Android 质量口径：host/android-target 双配置 clippy + crate 宿主单测 + renderer APK 构建与 JNI 导出核对

## 治理

- **归档区域** `archive/`：只追加不修改（当前
  [2026-09-12-m0-to-m4-code-slices.md](archive/2026-09-12-m0-to-m4-code-slices.md)）
- **证据区域** `evidence/`：构建/冒烟/清单证据持续追加
  （local-toolchain-bootstrap、v8-renderer-path、emulator-feasibility、
  device-acceptance-checklist、license-manifest、ci-emulator-smoke）
