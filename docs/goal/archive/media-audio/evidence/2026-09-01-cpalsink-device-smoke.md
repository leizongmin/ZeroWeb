# CpalSink 真设备冒烟实测（media-audio M1 收尾验证）

**日期**: 2026-09-01
**环境**: 本机（Linux 6.18.25-x64v3-xanmod1，内核层 HDA 声卡在，libasound2-dev
1.2.14-1 已装——media-audio D2 获批项）

## 实测内容

`cargo test -p zero-media --features audio-cpal`（经 make test 等价 feature 矩阵）：

- **编译**：cpal 0.16 + ALSA host 编译通过（`audio-cpal` feature）。
- **全测**：39 passed / 0 failed（38 default + 1 CpalSink 环境自适应冒烟）。
- **冒烟分支**：`cpalsink_constructs_or_reports_device_error` 走 **Ok 分支**
  （无 "unavailable" 输出——eprintln 探针确认）——真设备流构造成功：
  `CpalSink::new(48kHz/2ch)` → `start` → `write 480 样本（10 立体声帧）` →
  `pause` → 暂停期 write 拒收（`NotStarted`）→ `resume` 全链通过。
- **真出声验证**：写入为静音样本（0.0）——链路/流控验证面（声音可听性属人工
  面不在 CI 可断言范围；ALSAmixer 静音态亦不拒流）。真声采样回放留桌面环境
  人工冒烟（可选，不阻塞）。

## 结论

- media-audio D2 获批项闭环：cpal 编译 + 枚举 + 真设备流构建/流控全通。
- `CpalSink` 从「编译面已验证」升级为「本环境真设备流全链验证」。
- M1 的设备面验证完成；NullSink 可观测断言面维持 CI 常驻。

## 关联

- master.md M1 里程碑 / 下一步 #1（Mixer 接线挂 CpalSink 真出声切片）
- [M0 环境探测 evidence](2026-09-01-m0-environment-probe.md)
