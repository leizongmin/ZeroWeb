# media-audio D2 — libasound2-dev 安装后 cpal 编译实测

**日期**: 2026-09-01
**前置**: [M0 环境探测](2026-09-01-m0-environment-probe.md)（当时 libasound2-dev 缺失，
cpal 编译面阻塞）→ 用户批准 D2 → 安装 → 本实测。
**方法**: workspace 外探针 crate（`/tmp/cpal-probe`，依赖 `cpal = "0.16"`，默认 feature），
不污染主工作区；编译 + 运行设备枚举。

## 实测结果

| 项 | 结果 |
|---|---|
| `libasound2-dev` 安装 | ✅ 1.2.14（`dpkg -s` install ok；`/usr/include/alsa/asoundlib.h` 在；`pkg-config --modversion alsa` = 1.2.14） |
| cpal 0.16.0 编译（默认 feature，ALSA host） | ✅ **通过**——零 warning，纯 cargo 直依赖 |
| 默认 host | `ALSA` |
| 输出设备枚举 | ✅ 2 个：`default` + `HDA Intel PCH`（与 M0 探测的内核层 HDA 声卡一致） |

**枚举噪音说明**（不影响判定）：stderr 出现 `Unknown PCM pipewire` /
`PulseAudio: Connection refused` / `jack server is not running` / `Cannot open
device /dev/dsp`——ALSA 枚举设备时逐个探测 pipewire/pulse/jack/OSS 插件后端的正常
行为，WSL2 环境下均不可用，主设备枚举不受影响。

## 结论

- **D2 收口**：cpal 编译面阻塞解除，CpalSink（feature-gated `audio-cpal`）在本环境
  可编译、可枚举设备。
- M0 遗留的「cpal 编译验证」待办项闭环——media-audio 全部门控解除。
- 运行时真出声仍受 WSLg 音频桥限制（pulse server 不可用），但 CpalSink 冒烟的
  **可编译面 + 设备枚举面**已验证；真出声冒烟留待桌面环境可选执行（M0 报告
  §3 既定策略：M1 验收 = NullSink 可观测断言常驻 CI，CpalSink 冒烟可选）。
