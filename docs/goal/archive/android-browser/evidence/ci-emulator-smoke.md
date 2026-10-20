# CI 模拟器冒烟执行证据（M3 chrome 级验收）

**日期**: 2026-09-12 ~ 2026-09-13
**结论**: GitHub ubuntu runner（嵌套虚拟化 KVM）上 Android 模拟器冒烟**已可重复执行并通过**——
无需本机 KVM 授权。chrome 级断言全绿：四进程拓扑、UID 隔离（FR-008）、decoder/compositor
socket probe。本机模拟器 KVM 门控（master.md「待用户决策」）对 chrome 级冒烟不再构成阻塞。

## 工作流

`.github/workflows/android-emulator-smoke.yml`（门控与 ci.yml android job 一致：
workflow_dispatch / OWNER 批准的 PR review）：

1. KVM 前置检查（无 `/dev/kvm` 即红，不静默绿）
2. `:app:assembleEmulatorDebug`（x86_64 交叉编译，默认 feature，零 V8 依赖）
3. sdkmanager 装 emulator + `system-images;android-35;google_apis;x86_64`（回落 android-34）
4. avdmanager 建 AVD → emulator 无头启动（swiftshader_indirect，`-cores 4 -memory 4096`）
5. `scripts/android/install-smoke.sh`：安装 → 启动 → 拓扑/UID/probe 断言

## 迭代史（三轮失败 → 两轮绿，每轮根因定位）

| run | 结果 | 根因（证据） | 修复 |
|-----|------|--------------|------|
| 34703425006 | ✗ boot | `adb wait-for-device` 300s 超时；失败路径在 tail 日志前退出，零证据；诊断步骤 adb 对 offline 设备无限 hang | 诊断 timeout 包裹、失败路径固化日志、`adb start-server` 前置 |
| 34704966636 | ✗ boot | artifact 实证：emulator 报 `Unknown AVD name [zeroweb-smoke]` 即刻退出——avdmanager（新 cmdline-tools 走 ANDROID_USER_HOME）与 emulator 37.x（ANDROID_AVD_HOME → ANDROID_SDK_HOME/avd → HOME/.android/avd）默认目录不一致 | job 级 `ANDROID_AVD_HOME` 钉死同一目录；`emulator -list-avds` 预检发现性 |
| 34706845122 | ✗ smoke | **拓扑/UID 全过**（命名实证：isolated 服务进程名 = `packageName:className`，见 AOSP `ActiveServices.getProcessNameForService`，main 与 android-15.0.0_r1 一致）；probe 断言两 attempt 各缺一——`am start -W` 返回（TotalTime 4.2s）后 probe 才落 logcat，固定 sleep 2 窗口误报 | probe 断言改 60s 轮询（911f932d9） |
| 34707541575 | ✓（经 force-stop 重试） | run4 artifact logcat 实证 probe 均在 ~0.5-2.4s 成功落地而读取报缺——模拟器 logcat 噪声 **~1000 行/s**（8529 行/8.2s），默认 ~256KB 主环仅存 ~2s，`-t N` 行数窗口必然竞态 | `logcat -G 4M` 扩环 + `-d -s ZeroWebRole:*` tag 过滤读（399f9ae30） |
| 34708223825 | ✓ 7m32s | — | **单 attempt 干净通过**，无重试 |

## 通过证据（run 34708223825，2026-09-12T17:33:45Z）

```text
u0_a209      com.leizm.zeroweb                                    # browser
u0_i9000     com.leizm.zeroweb:com.leizm.zeroweb.RendererService0 # isolated ✓
u0_a209      com.leizm.zeroweb:compositor                         # 同 browser UID ✓
u0_i9001     com.leizm.zeroweb:com.leizm.zeroweb.ImageDecoderService # isolated ✓
install-smoke PASS
```

断言覆盖：进程四角色齐全、renderer/decoder isolated UID（≠browser）、compositor 与
browser 同 UID、decoder probe 与 compositor bridge 双 probe 成功（FR-008 拓扑 +
M2 探针语义）。

## 边界与遗留

- 本证据为 **chrome 级**：emulator flavor 默认 feature 无 renderer（`nativeNavigate`
  按设计返回失败），不验证页面加载/滚动帧。页面级冒烟需 renderer APK（V8 x86_64
  交叉编译或真机 arm64），归 FR-006/007 真机验收路径。
- 稳定性绿以连续两轮 job 绿为据（34708223825、34708752317）。
- 首轮曾疑「probe 一次性失败」——经 logcat 全量证据推翻，实为读取窗口竞态，Rust 侧
  probe/handshake 无需改动。
