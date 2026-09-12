# CI chaos 冒烟执行证据（M5：断连恢复负测试）

**日期**: 2026-09-13
**结论**: 角色进程 kill → 断连恢复重绑（RFC M3 death recovery 语义）已在 CI 模拟器
**执行验证并通过**——compositor/decoder/renderer0 逐角色 root-kill 后，BIND_AUTO_CREATE
全部拉起新进程，compositor 重连协议（detach → re-attach）与 decoder 重探针均成功。

## 脚本与接线

- `scripts/android/chaos-smoke.sh`：每轮 kill 前 `logcat -c`，断言①新 pid（同名进程，
  60s 轮询）②ZeroWebRole 成功日志再现（重连证据；抗噪同 install-smoke：`-G 4M` 扩环 +
  `-s ZeroWebRole:*` 过滤读）
- kill 应用子进程需 root：google_apis 非 playstore 镜像 `adb root` 可用（脚本内自取，
  kill 被拒时显式报 root 要求）
- CI：android-emulator-smoke 工作流在 install-smoke 后追加 chaos 步骤；本地入口
  `make android-chaos-smoke`（依赖 android-install-smoke，须运行中的模拟器/真机）

## 通过证据（run 34711012620，2026-09-12T18:29Z）

```text
chaos: killing com.leizm.zeroweb:compositor (pid 2385)
chaos: com.leizm.zeroweb:compositor restarted with new pid 3062
chaos: re-connection confirmed: compositor bridge ready
chaos: killing ...ImageDecoderService (pid 2389)
chaos: ...ImageDecoderService restarted with new pid 3432
chaos: re-connection confirmed: decoder probe succeeded
chaos: killing ...RendererService0 (pid 2381)
chaos: ...RendererService0 restarted with new pid 3675
chaos-smoke PASS
```

语义对应：compositor 死亡走 `onServiceDisconnected` → `nativeDetachCompositor` 作废
僵尸 transport → 重启后 `onServiceConnected` 重新走 attach 协议（对 renderer 断连恢复
对称，RFC M3）；decoder 重启后探针线程重跑。浏览器主进程全程存活（拓扑断言未被破坏）。

## 边界与遗留

- renderer0 断言仅到进程重启（renderer-less 构建 `nativeRendererLinked()=false`，
  socket/attach 路径按设计旁路）；renderer socket 断连恢复待真机 renderer APK 路径。
- 冷启动观测（am start -W TotalTime）：CI 模拟器 2.8-6.5s（软渲染，真机预期显著更低），
  正式性能基线归真机验收。
