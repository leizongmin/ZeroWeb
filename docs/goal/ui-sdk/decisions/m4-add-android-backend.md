# 决策：M4 追加 Android 作为第二移动端后端

**日期**: 2026-07-03
**决策者**: 用户确认
**影响范围**: ui-sdk goal / M4（DC-15）/ docs/goal/ui-sdk.md
**状态**: Accepted
**关系**: 本决策**追加**于 `m4-mobile-backend-harmonyos.md` 之上，不撤销 HarmonyOS 作为首选后端的定位。

## 背景

`docs/goal/ui-sdk/decisions/m4-mobile-backend-harmonyos.md`（2026-07-03）已锁定 HarmonyOS 为 M4 首个移动后端，理由是 OHOS toolchain + Rust target 在本机开箱即用。随后用户在本机补充安装了 Android SDK，希望同时支持 HarmonyOS 和 Android 两个后端。

## 决策

**M4 在 HarmonyOS 之外，追加 Android 作为第二移动端后端。**

### 范围与 DONE 判定（重要）

为避免无争议地扩大 M4 收口工作量，本决策对 DC-15 的边界作如下澄清：

- **DC-15 硬指标（阻塞 DONE）**：至少一个移动后端可运行——仍由 **HarmonyOS** 担当。HarmonyOS 达标即可判定 DC-15 第一条满足。
- **Android 是 stretch goal**：M4 期间尽量推进到「可启动到首帧」，但如果 Android 因工具链/适配成本在 M4 内无法收口，**不阻塞 DONE**，转入下一阶段继续。
- 此范围划分的依据：DC-15 原文（`ui-sdk.md` 第 181 行）只要求「至少一个」，本决策不放宽 DONE 终局，只是把 Android 纳入 M4 工作面。

> 若后续用户明确要求「Android 也必须是 DONE 硬指标」，需新建决策记录修订 DC-15，并在 master.md 重估 M4 工作量与排期。

## 本机 Android 环境证据（2026-07-03 复测）

| 组件 | 状态 | 说明 |
|------|------|------|
| Android SDK 根目录 | ✅ 已装 | `C:\Users\leizo\AppData\Local\Android\Sdk`，含 build-tools / platform-tools / platforms / emulator / system-images / sources / extras |
| adb | ✅ 已装 | version 1.0.41（`platform-tools\adb.exe`） |
| Android Studio | ✅ 已装 | `C:\Users\leizo\AppData\Local\Programs\Android Studio\bin\studio64.exe` |
| Java 17 | ✅ 已装 | `C:\Program Files\Microsoft\jdk-17.0.16.8-hotspot\`（与 HarmonyOS 共用） |
| **NDK** | ❌ **缺失** | `Sdk\ndk` 不存在——Rust 接 Android 的硬阻塞 |
| **sdkmanager / cmdline-tools** | ❌ 缺失 | 无法用命令行装 NDK，需要 Android Studio GUI 或先装 cmdline-tools |
| `aarch64-linux-android` Rust target | ❌ 未装 | `rustup target list --installed` 中无 android-* |

### 补齐 Android 工具链的待办

在 Android 真正可交叉编译之前，必须完成（**需要用户操作或联网下载**）：

1. **装 NDK**（任选其一）：
   - 在 Android Studio：`Settings → Languages & Frameworks → Android SDK → SDK Tools → NDK (Side by side)`，勾选最新稳定版（推荐 r27 系列），Apply。约 1–1.5 GB。
   - 或命令行：先装 `cmdline-tools`，再 `sdkmanager "ndk;27.0.12077973"`（版本号以 SDK Manager 显示为准）。
2. **装 Rust target**：`rustup target add aarch64-linux-android`（真机 64 位 ARM 主目标；如需 x86_64 模拟器再加 `x86_64-linux-android`，如需 32 位 ARM 老设备再加 `armv7-linux-androideabi`）。
3. **配 cargo linker**：在仓库或用户级 `.cargo/config.toml` 加：

   ```toml
   [target.aarch64-linux-android]
   linker = "<NDK 路径>/toolchains/llvm/prebuilt/windows-x86_64/bin/aarch64-linux-android24-clang.cmd"
   ```

   （API level 24 为广泛兼容下限，可按需调整。）

补齐后，本决策记录的验证清单才能复核。

### 工具链补齐结果（2026-07-03 已完成）

上述三项待办已于 2026-07-03 全部完成，配置方式因 cargo 限制做了调整：

| 步骤 | 实际做法 | 结果 |
|------|----------|------|
| NDK | 用户在 Android Studio 装 **NDK r30.0.14904198** | `Sdk\ndk\30.0.14904198` 存在 ✅ |
| Rust target | `rustup target add aarch64-linux-android` + `x86_64-linux-android` | 两个 target 均已装 ✅ |
| cargo linker | **不用 `.cargo/config.toml`**，改用环境变量 `CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER` / `CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER` | 见下方说明 ✅ |

**为什么不用 `.cargo/config.toml`**：cargo 的 `[target.<triple>] linker` 字段不支持环境变量插值（[rust-lang/cargo#9362](https://github.com/rust-lang/cargo/issues/9362)），写死本机路径会违反 AGENTS.md 第 6 条「路径通用化」（含用户名 `leizo` 的绝对路径不能进 git，仓库曾因此触发 SL-008/SL-010，见 commit `58e74ac8`）。

**实际配置位置**：`C:\Users\leizo\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1`，内容为：

```powershell
$env:ANDROID_NDK_HOME = "C:\Users\leizo\AppData\Local\Android\Sdk\ndk\30.0.14904198"
$ndkToolchain = Join-Path $env:ANDROID_NDK_HOME "toolchains\llvm\prebuilt\windows-x86_64\bin"
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = Join-Path $ndkToolchain "aarch64-linux-android24-clang.cmd"
$env:CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER = Join-Path $ndkToolchain "x86_64-linux-android24-clang.cmd"
```

**API level 选择**：24（Android 7.0，广泛兼容下限）。升级 NDK 时只需改 `ANDROID_NDK_HOME` 一处；切换 API level 需同步改 clang 文件名后缀（如 `android24` → `android28`）。

**交叉编译验证**（2026-07-03）：

```
cargo build --target aarch64-linux-android -p zero-ui-core
→ Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.78s
```

整条链路（rustc → NDK clang → Android `.a`/`.so`）打通，本机 Android 工具链达到与 HarmonyOS 同等的「开箱即用」状态。

## 接口抽象层的影响（M1–M3 回溯）

好消息：M1 阶段 `ui/platform` / `ui/runtime` / `ui/gestures` / `ui/navigation` / `ui/overlay` 的接口设计**本来就要求平台中立**（见 `m4-mobile-backend-harmonyos.md` 「对 M1–M3 的回溯影响」）。Android 的关键概念与 HarmonyOS 几乎一一对应：

| 概念 | HarmonyOS | Android |
|------|-----------|---------|
| 安全区域避让 | `avoidArea` | `WindowInsets` / `fitsSystemWindows` |
| 软键盘 insets | `avoidArea` 类型变化 | `WindowInsets.ime` |
| 系统字号缩放 | 系统文本字号设置 | `fontScale` / `Configuration` |
| 平台返回手势 | OHOS 返回 | `OnBackPressedDispatcher` / predictive back |
| 窗口/surface | Ability 内 window | `Activity` / `SurfaceView` / `Surface` |

因此追加 Android 后端**不要求**改 M1–M3 的通用接口签名，只是 M4 多一个 `ui/adapters/android` 实现。约束依然成立：通用 crate 的公共 API **不得**向 widgets / patterns / browser-ui 暴露 Android- 或 OHOS-specific 类型。

## 新增 crate（M4 实施时）

- `ui/adapters/android`（`zero-ui-adapter-android`）：与 `ui/adapters/harmonyos` 对位，提供：
  - `AndroidRuntime`（对应 `HarmonyosRuntime`）
  - `event_map`（Android `MotionEvent` / `KeyEvent` → SDK `ui/core::event`）
  - surface / window 适配（`Activity` → SDK `WindowMetrics`）
  - IME / soft keyboard insets / safe area 桥接
- 该 crate 可依赖 `ui/core`、`ui/runtime`、`ui/platform`、`ui/gestures`；**不得**被 widgets / patterns / browser-ui 直接依赖（与 HarmonyOS adapter 同一约束）。

注：当前仓库已有 `ui/adapters/harmonyos`（lib.rs 30 行 + runtime.rs 180 行 + event_map.rs 370 行，共 580 行）。Android adapter 起步规模相当，属于 M4 实质工作量。

## 验证清单（M4 推进 Android 时复核）

- [x] NDK 已装（`Sdk\ndk\30.0.14904198`）—— 2026-07-03
- [x] `rustup target add aarch64-linux-android` 完成（含 x86_64-linux-android）—— 2026-07-03
- [x] `cargo build --target aarch64-linux-android -p zero-ui-core` 可链接通过（5.78s）—— 2026-07-03
- [ ] Android Studio 能创建 APK 壳工程并调用 Rust 共享库（`*.so`）
- [ ] 最小 demo 或 browser 能在 Android 设备/模拟器启动到首帧（满足 DC-15 第一条的「Android 这一支」）
- [ ] PhoneBrowserShell / TabletBrowserShell 在 Android 上与 HarmonyOS / Desktop 共享同一 `BrowserChromeModel` + `BrowserAction`
- [ ] Android back gesture / soft keyboard / safe area / text scale / touch gesture 最小适配 skeleton 落地

## 对入口文档的同步修改

本决策落盘同时更新 `docs/goal/ui-sdk.md`：在原「M4 选定 HarmonyOS」措辞后追加「（同时推进 Android 作为第二后端，范围见本决策记录）」。HarmonyOS 仍是 DONE 硬指标，Android 是 stretch goal。

## 后续追踪

- 若 M4 推进中发现 Android 工具链或适配阻塞远超预期，可在 master.md 决策日志记录后**保留 Android 在下一阶段**，不影响 HarmonyOS 单后端满足 DC-15 进而 DONE。
- 若 Android 顺利在 M4 内达标，可在 M4 archive 中标注「DC-15 双后端超额完成」。
