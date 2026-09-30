---
date: 2026-09-30
modules: engine,paint,layout-engine
---

# cargo test --workspace 深嵌套渲染单测栈溢出：feature 统一改变栈帧量级，单包复现会误导归因

## 问题描述

t7 修复（js-dom P15，改动仅 js_dom_bridge/shim 7 文件，不涉 paint/feature）后跑
`cargo test --workspace`，`zero-engine` 的 `test_pipeline_deeply_nested_html`
（仅 20 层 `<div>`）稳定栈溢出 abort（3/3）。但 `cargo test -p zero-engine --lib`
全量 2760 用例两轮全绿——现象只出现在 workspace 模式，极易误判为新改动引入的回归。

## 根因分析

1. cargo 按**调用面**做 feature unification：`cargo test --workspace` 会把其他
   workspace member 开启的 feature 合并进 zero-engine 的编译（产物 metadata hash
   与单包调用不同，二进制不同），paint/layout 侧帧量级变大。
2. `painter` 递归按盒树逐元素下探，栈余量「以字节计」（painter/mod.rs R4248 注释
   自证）；libtest 测试线程默认栈 2MiB，workspace 统一编译后 20 层递归越界。
3. 判别实验链（复用同一思路可省一半排查时间）：
   - 直接跑 workspace 产物单测 → 通过 → 排除调度/并发，锁定二进制差异；
   - `git stash` 后 `cargo test --workspace --no-run`，产物 hash 不变（metadata
     hash 只由 crate 名+feature+flags 决定，与源码内容无关），覆盖即基线同型产物
     → 基线同样溢出 → **pre-existing**，与本改动无关；
   - `RUST_MIN_STACK=33554432` 重跑溢出二进制 → 通过 → 纯栈量级问题。

## 解决方案

- 门禁跑法：`RUST_MIN_STACK=33554432 cargo test --workspace`（libtest 官方出口，
  不改产品代码）；本轮 t7 即以此通过并在提交说明中注明覆盖口径。
- 根治方向（未实施，另立切片）：`test_pipeline_deeply_nested_html` 类深递归单测
  自管线程栈（`thread::Builder::stack_size`），或 painter 深树改显式栈迭代——
  两者都不属于渲染修复切片的最小边界。

## 如何避免

- 「单包过、workspace 挂」时先怀疑 feature unification 而不是自己的 diff：比对
  两个产物二进制（`target/debug/deps/<crate>-<hash>` 的 mtime + hash）即可定性，
  不要直接开始改代码。
- metadata hash 与源码内容无关这一点可反向利用：同 hash 二进制被重写 = 产物已按
  当前 feature 集重编，可直接用于基线判别，不必重新 stash 构建。
