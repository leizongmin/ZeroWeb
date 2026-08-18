---
date: 2026-08-13
modules: Makefile, zero-render-foundation, zero-compositor, 性能门禁
---

# GPU 与性能门禁需要能力匹配

## 问题描述

无 wgpu adapter 的 Linux 主机会让 adapter-only 测试批量 panic；同一主机运行性能门禁时，又把 Xeon 结果与 i5 baseline 比较，产生全局 2.5–8 倍“回归”。

## 根因分析

OS/architecture 相同不代表 GPU capability 或 CPU 性能可比较。GPU 测试默认 unwrap adapter；性能脚本只按 `linux-x86_64` 选择 baseline，忽略报告和 baseline 中已有的 CPU model/core 元数据。

## 解决方案

- workspace 主测试运行平台无关覆盖；先探测 headless adapter，成功后再运行 adapter-only GPU 矩阵。
- GPU 进程测试的共享 env mutex 必须容忍 poison，避免首个断言失败掩盖其余诊断。
- 性能门禁始终执行 retained-form 与页面总时长绝对预算；只有 CPU model/core 匹配时才应用相对 baseline。
- 不因平台不匹配更新或放宽已有 baseline。
