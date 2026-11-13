---
date: 2026-11-14
modules: css-parser
---

# debug 单测 SIGILL：先 cargo clean -p 排增量缓存毒化，再查逻辑

## 问题

`cargo test -p zero-css-parser --lib` 进程以 SIGILL（signal 4）死，无 panic
输出、无失败测试名。单线程过滤到具体测试
（`sibling_tree_counting_tests::tree_counting_fold_number_and_unpack`）后
**隔离复现** SIGILL。该测试是纯断言（fold_math_specified 字符串比较），
无 unsafe、无并发。

## 根因

增量编译缓存毒化。判据链：

1. stash 全部工作树改动，干净 HEAD 上同一测试仍 SIGILL——排除本批改动；
2. `cargo test --release` 同一测试通过——排除测试逻辑错误；
3. `cargo clean -p zero-css-parser` 丢掉该 crate 的增量产物后，debug
   profile **3,189 全绿**，一次复跑即愈。

rustc 1.99.1 debug codegen 在增量重编译（本轮 attr.rs 修改触发同 crate
重编）后为无关测试产出非法指令（`ud2` 类），属工具链 artifact，非源码
缺陷。

## 解决方案

workspace 单测进程 SIGILL / 无诊断信息崩溃时的排查序：

1. 先 `cargo clean -p <crate>` 后重跑——增量缓存毒化零成本排除；
2. 仍复现再 stash 对照干净 HEAD，区分本批改动与预存问题；
3. debug SIGILL + release 通过 = 毒化定谳（逻辑 bug 两 profile 同死），
   不需要下溯汇编。

不要在 SIGILL 上直接归因本批改动（本次干净 HEAD 同样复现）；也不要为
它回滚正确代码。
