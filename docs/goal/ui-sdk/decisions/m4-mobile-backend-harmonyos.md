# 决策：M4 移动端后端选定 HarmonyOS

**日期**: 2026-07-03
**决策者**: 用户确认
**影响范围**: ui-sdk goal / M4（DC-15）/ docs/goal/ui-sdk.md §Mission DONE 终局
**状态**: Accepted

## 背景

`docs/goal/ui-sdk.md` 的 DONE 终局（第 24 行）和 DC-15（第 179–184 行）要求「至少一个移动端后端（Android/iOS/HarmonyOS 之一）可运行」。三选一需要在 M4 之前确定，以便：
- 提前确认交叉编译工具链可用；
- 在 M1–M3 接口设计阶段为该平台预留正确的抽象点（safe area / soft keyboard / back gesture / 平台 surface）；
- 与 spec §6.7 执行技能路由（goal 文档第 462–470 行）对齐。

## 决策

**M4 选定 HarmonyOS 作为首个（且 M4 阶段唯一）移动端后端。**

DC-15 仅要求「至少一个」移动后端可运行；本决策不排除后续里程碑追加 Android，但 M4 收口范围限定为 HarmonyOS。

## 本机环境证据（2026-07-03 实测）

| 工具链 | 状态 | 说明 |
|--------|------|------|
| DevEco Studio | ✅ 已安装 | `C:\Program Files\Huawei\DevEco Studio`，含 OHOS toolchains |
| OHOS Rust 交叉编译 target | ✅ 已安装 | `aarch64-unknown-linux-ohos`、`x86_64-unknown-linux-ohos`（`rustup target list --installed`） |
| HMS Core | ✅ 已安装 | `C:\Program Files\Huawei\HMS Core` |
| Java 17 | ✅ 已安装 | `C:\Program Files\Microsoft\jdk-17.0.16.8-hotspot\`（DevEco 与 Android 共用） |

被排除的选项：

- **iOS**：当前主机 `win32 10.0.26200`，Xcode 仅在 macOS 运行，物理上不可行。
- **Android**：`ANDROID_HOME` 已设但 `C:\Users\leizo\AppData\Local\Android\Sdk` 实际为空；NDK 缺失（Rust 接 Android 必需）；Android Studio 未安装；`aarch64-linux-android` / `android` Rust target 未安装。补齐成本显著高于 HarmonyOS。

## 选定 HarmonyOS 的理由

1. **零额外安装成本**：DevEco + OHOS toolchains + OHOS Rust target 均已就位，是三选一中唯一开箱即用的。
2. **与 spec 技能路由一致**：goal 文档第 468 行已为 HarmonyOS 挂 `lei-harmonyos6-dev` skill（preferred），ArkTS / ArkUI / `.ets` / Ability / `@ohos` kit 适配路径在 Spec 阶段已预埋。
3. **Rust 侧直接可交叉编译**：`cargo build --target aarch64-unknown-linux-ohos` 即可起步。

## 对 M1–M3 的回溯影响

本决策在 M4 才落地可运行后端，但接口抽象层在 M1 就要预留：

- `ui/platform`、`ui/runtime`、`ui/gestures`、`ui/navigation`、`ui/overlay` 的公共 API 在 M1 设计阶段**至少**覆盖以下 HarmonyOS 关键概念（与 Android/iOS 概念命名上保持平台中立）：
  - safe area（HarmonyOS `avoidArea` / 安全区域避让）
  - soft keyboard（输入法升起时的窗口 insets 变化）
  - text scale（系统字号缩放）
  - 平台 back gesture（OHOS 返回手势）
  - surface / window（Ability 内的窗口与 surface 提供）
- 这些抽象点 **不得**向 widgets / patterns / browser-ui 暴露 OHOS-specific 类型（与 goal 第 88 行「winit 类型不得泄漏」同源约束）。

## 验证清单（M4 推进时复核）

- [ ] `cargo build --target aarch64-unknown-linux-ohos -p <ui-runtime-or-demo>` 可通过
- [ ] DevEco Studio 可创建 Ability 壳工程并调用 Rust 共享库
- [ ] 最小 demo 或 browser 能启动到首帧（满足 DC-15 第一条）
- [ ] PhoneBrowserShell / TabletBrowserShell 与 DesktopBrowserShell 共享 `BrowserChromeModel` + `BrowserAction`（满足 DC-15 第二条）
- [ ] touch / pan / pinch / fling / soft keyboard / safe area / text scale / back gesture 最小适配 skeleton 落地（满足 DC-15 第三条）

## 对入口文档的同步修改

本决策落盘的同时，已将 `docs/goal/ui-sdk.md` 第 24 行（Mission DONE）、第 62 行（Support Envelope 移动端运行时）、第 181 行（DC-15 第一条）、第 324 行（M4）的「Android/iOS/HarmonyOS 之一」收敛为「M4 选定 HarmonyOS」。

## 后续追踪

- 本决策为 M4 选型；M4 推进时若发现 OHOS Rust 工具链或 DevEco 集成存在阻塞，可在 master.md 决策日志中重开评估，但**不得**静默改回三选一。
- 若后续要追加 Android 后端，需新建独立决策记录，并先补齐本机 NDK / Android Studio / `aarch64-linux-android` target。
