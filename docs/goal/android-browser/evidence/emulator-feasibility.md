# 模拟器冒烟环境可行性核查（本机 WSL2）

**日期**: 2026-09-12
**结论**: 本机（WSL2 x86_64）当前**无法启动 Android 模拟器**——`/dev/kvm` 存在但属
`root:kvm`，当前用户 `lei` 不在 `kvm` 组，且 `sudo` 需要密码（无人值守环境无法授权）。
模拟器冒烟（goal M3）与 `android-install-smoke` 落地需等待以下任一解锁途径。

## 实测记录（2026-09-12）

| 检查项 | 结果 |
|--------|------|
| `/dev/kvm` | 存在，`crw-rw---- root kvm` |
| 用户组 | 附组 sudo/wheel/audio/video/render——**无 kvm** |
| `head -c0 /dev/kvm` | `Permission denied` |
| `sudo -n true` | 失败（需要密码） |
| SDK 组件 | `platform-tools`(adb) ✓、`platforms/android-36` ✓、`ndk/30.0.16248370` ✓；**emulator 与 system-images 未安装** |
| AVD | `~/.android/avd/` 空 |
| gradle 模拟器产物线 | `assembleEmulatorDebug`（flavor `emulator`，`abiFilters x86_64`）已就绪；`scripts/android/install-smoke.ps1` 已断言四进程拓扑 + UID 隔离 + probe 日志（M0 Windows 路径） |

## 解锁途径（任一即可）

1. **本机 KVM 授权**（最快）：用户一次性输入密码执行
   `sudo usermod -aG kvm $USER`，新 shell（或 `sg kvm -c '...'`）生效后：
   `sdkmanager emulator "system-images;android-36;google_apis;x86_64"` →
   `avdmanager create avd` → 无头启动 `-no-window -gpu swiftshader_indirect`。
   注意 arm64 系统镜像在 x86 宿主**无硬件加速**，必须用 x86_64 镜像。
2. **CI 模拟器**：GitHub Actions ubuntu runner 具备嵌套虚拟化时可在 CI 跑
   emulator + `install-smoke` 等价脚本；但页面渲染冒烟需要 renderer feature
   （V8 x86_64 交叉编译，CI 时长风险高），chrome 级冒烟（四进程拓扑/probe/快照）
   无 renderer 也可跑（renderer-less 构建导航按设计返回失败）。
3. **真机**：`make android-renderer-apk` 产物（arm64 签名 APK）+ adb，待用户提供设备
   （master.md「待用户决策」既有项）。

## 与 renderer-less 构建的边界

`nativeRendererLinked()`=false 时 `nativeNavigate` 有意返回失败（无 renderer 可路由），
因此 chrome 级模拟器冒烟可验证：启动、四进程绑定拓扑、decoder/compositor probe、
标签/书签/历史快照持久化；**不能**验证页面加载/滚动帧。页面级冒烟必须 renderer APK
（本机 x86_64 路径需另做一次 V8 x86_64 交叉编译，arm64 缓存不复用）。
