---
date: 2026-10-11
modules: engine,tests/wpt-runner
---

# A/B 基线臂重建后恢复源码未重建 release 二进制——守成跑测到旧产物出假回归

## 问题描述

navigation-compat M2-S9 轮做了零回归双臂测量：先带修复构建 release runner 跑证据轮，
再 stash 源码修复 → **重建 release**（pre-change 基线臂）跑对照轮 → stash pop 恢复源码。
恢复后只跑了 debug/test profile 的 `make test`，**没有再重建 `target/release/zero-wpt-runner`**。
下一轮的组成态守成验证直接跑了盘上的 release 二进制：S9 已修复翻绿的
`navigate-anchor-cross-origin` 退回 Timeout，407/476 vs 证据 408/476——形似真回归。

## 根因分析

盘上的 release 二进制停在**基线臂构建**（无修复），与工作树源码（有修复）脱钩。
二进制 provenance 与源码状态无任何强制绑定：`make test` 类入口只编 test profile，
不会顺手重建 release 产物；测试入口（`testharness-navigation` 等）虽依赖
`zero-wpt-runner-release`，但手动绕过 make 直接跑 `./target/test-guard -- ./target/release/zero-wpt-runner`
时 cargo 不参与，陈旧产物静默上测。

## 解决方案

重建 release 后复跑：408/476，与 S9 evidence 逐行恒等——组成态零漂移，假回归排除。

**流程规则**：A/B 双臂测量（stash → rebuild → 跑基线 → pop）结束后，**必须用恢复后的
源码重建被测二进制**再做任何后续测量；或跑测前核对二进制 mtime ≥ 全部相关源码 mtime
（本轮 `ls -la target/release/zero-wpt-runner` 的 00:05 vs part03.js 的 00:57 一眼定性）。
统一走 make 入口（`testharness-navigation` target 自带 release 依赖）可结构性避免。

## 如何避免

- A/B 测量的恢复步骤写全：pop 之后紧接着 `make zero-wpt-runner-release`（或本轮后续
  跑测一律经 make target，让依赖图兜底）。
- 见到「已修复用例退回旧状态」的守成红灯，先查二进制 provenance（mtime/sha）再查代码——
  归因顺序应为：产物陈旧 → 环境（平行 clone 负载 flake）→ 代码组合态。
