# Android 浏览器可用化 — M2 到日常可用的平台适配目标

**版本**: v1.0
**日期**: 2026-09-07
**状态**: Active
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（M14「Android 平台适配」未标注项 + Mission「架构从第一天预留」）

> **说明**
> 本文档是 ZeroWeb「Android 浏览器可用化」专项目标执行契约。Android 线**远超 M0 骨架**——
> 22 个 JNI 导出、四类 native 进程角色（renderer×8/compositor/image-decoder）、compositor
> 帧桥接、滚动、网络代理均已落地（`NATIVE_VERSION = "ZeroWeb Android M2"`），但治理缺位：
> 无 CI、无 goal 追踪、renderer transport adapter 未完成、无真机验收记录。目标是以可验证的
> 里程碑把 Android 线推进到「可构建、可安装、基础网页可用」。本文定义 Mission、边界、
> Done Criteria、执行协议和文档治理规则，供后续 `rally run` 会话作为稳定输入。日常进展、
> evidence、active milestone 更新写入 `master.md`。
>
> **▶ 拆分动机（2026-09-07 用户决策）**：从父目标 M14 拆出。理由：① Android 是父目标
> Mission 明确的平台目标（P1 后续），且代码已有实质投入（M2 级），是唯一「功能中期、治理
> 缺位」的平台线——无 goal 追踪意味着投入无验收标准；② 改动域（apps/android-browser 独立
> Cargo 工程 + Kotlin）与 rendering-compat 渲染流域**零重叠**（读消费，不改布局/渲染逻辑）；
> ③ 无 CI 覆盖是明确的第一缺口——先建立构建/测试门禁再谈功能，符合父目标质量门禁精神。
>
> **▶ 基线事实（2026-09-07 实测）**：
> - **Kotlin 侧**：`apps/android-browser/app/src/main/java/com/leizm/zeroweb/`——
>   `MainActivity.kt`（478 行，Compose UI、外部 intent、返回键、双预览 Bitmap）、
>   `NativeBridge.kt`（77 行，22 个 `external fun` JNI 声明）、`NativeRoleService.kt`
>   （51 行，RendererService0..7 + CompositorService + ImageDecoderService）、
>   AIDL `IRoleService.aidl`、中英文资源。
> - **Rust 侧**：`apps/android-browser/rust/`（独立 Cargo 工程，workspace member）——
>   `lib.rs`（949 行）22 个 JNI 导出（navigate/newTab/goBack/goForward/bookmark/history/
>   scroll/snapshot/startRole/runRole/attachCompositor/attachRenderer/latestPageFrame/
>   probeDecoder/probeCompositor 等），`facade.rs`（228 行）复用 `zero_browser_shell::
>   BrowserShell`；内部经 `zero_protocol::PipeTransport<UnixStream>` 走多进程角色。
> - **已落地能力**：renderer fetch 代理（`proxy_renderer_fetch` L568）、renderer→compositor
>   帧转发（L655）、compositor 帧回传 Bitmap（L458）、滚动转发（L489）、WSL renderer 构建
>   产物（7e2a1ab34）。
> - **明确缺口**：① `.github/workflows/` 8 个 yml **零 Android job**（无 NDK 构建/单测/
>   APK 产物门禁）；② rust/README.md 自述「renderer 暂保持 Kotlin Service 拓扑，Android
>   transport adapter 在后续 M1 切片完成」；③ RFC `docs/specs/android-browser-spec-rfc.md`
>   （1097 行，状态「待确认」v0.1）的 FR-006 下载/FR-007 生命周期/FR-009 系统集成权限
>   未见对应代码；④ 无 arm64 真机验收记录、无 Release APK 交付；⑤ 版本串 `M2` 与
>   README 的 `M0` 描述不一致（文档滞后）。
> - **架构边界注记**：renderer 的 Android transport adapter 属多进程架构深化——本目标
>   **只做治理与可用性**（构建门禁、构建修复、已有功能回归保护），transport adapter 深化
>   与 RFC 批准走「待用户决策」，不自主开工。

---

## Mission

把 Android 线从「功能中期、治理缺位」推进到「**可构建、可安装、基础网页可用**」：
建立 CI 构建与测试门禁、修齐构建/文档不一致、保护已有功能不回归，并以模拟器/真机冒烟
记录验收证据。分阶段里程碑校准执行预期：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **治理就位** | Android CI job（NDK 构建 + Rust 侧单测 + APK assemble 门禁） |
| 第二阶段 | **可安装** | Release APK 产物 + 构建文档 + 版本串统一 |
| 长期 | **基础可用** | 模拟器冒烟（导航/标签页/滚动/书签）+ 已有功能回归保护 + RFC 待确认项决策清单 |

**关键约束**：Android 线的所有验收以**可重复的构建/测试命令**为证据（CI job 或本地脚本
+ 记录产物），不以「代码写完了」为完成标准。无法在本环境验证的项（真机）记入
「待用户决策」清单，等用户提供设备，不充数。

覆盖范围：

1. **CI 门禁** — Android NDK 交叉编译 Rust 侧 + Gradle assemble + Kotlin 编译（debug
   先行，release 跟进）；Rust 侧单测（host 可跑部分）
2. **构建一致性** — 版本串统一（M2 vs M0 文档滞后）、构建文档、`make`/脚本入口
3. **功能回归保护** — 已有 22 个 JNI 导出与四类进程角色的桥接测试加固
4. **冒烟验收** — 模拟器（CI 或本地）导航/标签页/滚动冒烟 + 证据记录
5. **RFC 待确认项** — FR-006/007/009 与 transport adapter 的决策清单整理（整理，不实施）

执行方式：**治理先行** — CI/构建门禁是后续一切验收的前提。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| CI 集成 | `.github/workflows/` 新增 Android job（NDK + Gradle） | 只增不改既有 8 个 job |
| Rust 侧 | `apps/android-browser/rust/` 的编译修复、单测加固 | JNI 桥接测试（host 侧可跑的 mock 层） |
| Kotlin 侧 | 构建配置、依赖版本、编译错误修复 | 不重构 UI 架构 |
| 构建脚本 | Android 构建入口（脚本或 Makefile target） | 照 package-linux.sh 先例 |
| 冒烟测试 | 模拟器冒烟脚本 + 证据记录 | 照 browser-compositor-smoke.sh 精神 |
| 文档对齐 | README/版本串/RFC 状态注记 | 只对齐事实，不改 RFC 结论 |

### 不在范围内（明确排除）

- **renderer Android transport adapter 实施** — 多进程架构深化，RFC（待确认）域；
  记入「待用户决策」等用户点名
- **FR-006 下载 / FR-007 生命周期 / FR-009 系统权限的功能实施** — 同上，只整理决策清单
- **触摸输入优化 / IME / 移动端 UI 重设计** — RFC UI 域，等 RFC 批准
- **鸿蒙 PC 适配** — 独立平台，不在本目标
- **桌面三平台的行为变更** — 任何影响 linux/macos/windows 主线构建的改动都超出本目标边界

### 依赖约束

- **与 rendering-compat 流边界（run-rules §9）**：本流改动域 = `apps/android-browser/` +
  CI workflow 新增 job + 本 goal 控制面，与渲染流域 crate 域**零重叠**。android-browser/rust
  依赖 zero-protocol/zero-compositor/zero-image-decoder 的**公开 API**——只读消费；若需
  这些 crate 变更则停下记录（属其他流域），转做本流其他面。
- **Cargo.lock 属共享面**：android-browser/rust 依赖变更会碰 Cargo.lock——变更前 `git log`
  核对，若与他流冲突即暂缓（碰头信号，暂停一边记入 master.md，不硬解）。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: CI 构建与测试门禁就位

- [ ] CI 新增 Android job：NDK 交叉编译 android-browser/rust 成功
- [ ] Kotlin/Gradle assemble（debug APK）成功入 job
- [ ] Rust 侧可 host 运行的单测入 job（jni 不可 mock 的部分如实标注）
- [ ] job 失败即红灯（不 continue-on-error——windows-aarch64 环境问题先例不适用于新 job）

### DC-2: 可安装产物

- [ ] Release/Debug APK 构建入口脚本就位（本地一条命令可构建）
- [ ] 构建文档（依赖、命令、产物路径）写入 apps/android-browser/README
- [ ] 版本串统一（NATIVE_VERSION 与 README 描述一致），文档滞后清零

### DC-3: 已有功能回归保护

- [ ] 22 个 JNI 导出有桥接测试覆盖（host 侧 mock JNI 层；不可 mock 的如实标注并给出
      手工验证清单）
- [ ] 四类进程角色启动路径（renderer/compositor/image-decoder）有冒烟断言

### DC-4: 测试与质量不可退让

- [ ] `make test` 全绿，零失败（桌面主线不受 Android 改动影响）
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] Android 冒烟证据持久化到 `docs/goal/android-browser/evidence/`

---

## 活跃里程碑

### M1 — CI 门禁 + 构建修复

**目标**：Android CI job（NDK 构建 + assemble）绿；构建错误修齐；版本串统一。

**切片建议**：
1. 本地先打通：NDK 交叉编译 android-browser/rust + Gradle assemble（记录依赖与命令）
2. CI job 落地（照既有 build-and-test 矩阵风格）
3. README/版本串对齐

### M2 — 回归保护 + 可安装产物

**目标**：JNI 桥接测试加固、进程角色冒烟断言、APK 构建入口脚本。

### M3 — 冒烟验收 + 决策清单

**目标**：模拟器冒烟（导航/标签/滚动）+ 证据记录；RFC 待确认项（FR-006/007/009、
transport adapter）决策清单整理 → 提交用户决策。

> **门控注记**：M3 的真机验收（真实设备流畅度、触摸/IME）等用户提供设备（同父目标 P3
> GPU 物理机门控模式），模拟器冒烟不因此阻塞。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用（如 NDK 依赖不可下载且无替代） |

### DONE 允许条件

**同时满足**：DC-1~4 全部满足；CI job 稳定绿（非侥幸单次通过）；`make test` +
`cargo clippy` 全通过；master.md 内部自洽，archive 已建立。真机验收项按门控注记
明确记录为「等设备」，不算未满足 DC。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**当前构建链的确切状态（NDK 版本要求、Gradle 配置、依赖闭包）
2. **自主打通**本地构建，记录可重复命令
3. **自主落地**CI job 与门禁
4. **自主修复**构建/编译问题，不等待用户逐步指令
5. **自主加固**桥接测试，新修复必须有对应测试
6. **自主验证**：`make test` + clippy + Android job 绿
7. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量修复**：构建错误、文档不一致、测试缺口——根因清楚、改动面小。
2. **永不停**：遇需拍板事项（RFC 批准、真机、transport adapter）记「待用户决策」清单
   并跳过，继续下一个轻量修复。
3. **碰撞管理**：Cargo.lock 变更前 `git log` 核对；只读消费其他 crate 公开 API，
   需要改它们时停下记录（属其他流域）。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。当作当前任务的一部分修复，直到稳定可重复。
2. **构建失败分析**：每次构建失败必须定位根因（NDK 版本？依赖特性门？Gradle 配置？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/android-browser/master.md`：当前真实状态的唯一控制面板。
  治理规则：持续演进、不允许无限增长（过时内容压缩或归档）、各章节必须自洽。
- **归档区域** `docs/goal/android-browser/archive/`：只追加不修改。
- **证据区域** `docs/goal/android-browser/evidence/`：构建/冒烟证据，持续追加。
