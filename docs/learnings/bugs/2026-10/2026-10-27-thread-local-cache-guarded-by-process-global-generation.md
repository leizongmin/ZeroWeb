---
date: 2026-10-27
modules: crates/engine/src/dom_bindings/mod.rs, crates/webview/src/webview.rs, tests/wpt-runner
---

# 进程级代际计数守护线程局部缓存——并行化 runner 时在跑案绑定状态被误重装

**触发轮**：perf/wpt-runner-testharness-speed（html-semantics parallel_map 并行化 A/B）

## 问题描述

html-semantics 套件并行化（rayon `parallel_map`，每 worker 线程串行跑各自分到的案）
后，A/B 等价比对发现 jobs>=2 下 `forms/the-label-element` 的 clicking-noninteractive-
unlabelable-content 与 proxy-modifier-click-to-associated-element 两案 6 子测**确定性**
Fail（`srcElement.cloneNode is not a function`、`assert_true: expected true got false`）；
jobs=1 稳 Pass，j2/j3/j8 差的是同一批键——不是概率 flake，是逻辑分支。

## 根因分析

`dom_bindings::NATIVE_STATE_GEN` 原是**进程级** `AtomicU64`，但它守护的 V8 模板/身份
缓存（`gc.rs` thread_local `Global` Handle）是**线程局部**的，而 `reset_native_state`
（Drop 时 bump 代际）只在 WebView 拥有线程执行（`WebView` !Send；js_worker 单线程）。
R384 的原语义「install 前代际不符 ⟺ 缓存属于已销毁 Isolate」只在**同线程串行**下
成立。并行下：

1. worker T1 的 WebView A Drop → 进程级代际 1→2（影响所有线程）；
2. worker T2 在跑案 B 的下一次 install（每次 `execute_script_with_dom` 都调）比对
   `native_state_gen(1) != 2` → 误判「缓存属于别的 Isolate」→ reset + 全量重装；
3. 重装重建模板/身份映射，案 B JS 已捕获的对象（`template.content.children` 的元素、
   事件面包装对象）与重建后的原型/映射脱节 → 方法缺失、断言面碎。

失败案恰好集中在 click/模板 面，是因为只有断言路径踩中「旧对象 + 新模板」错位的
API 才翻红——损害面比失败集大，失败集只是可观测子集。

## 解决方案 / 如何避免

- 代际计数改 `thread_local Cell<u64>`：计数与被守护缓存同线程局部性，Drop 只 bump
  本线程，worker 互不干扰；同线程串行语义逐位不变（R384 双 WebView 共存、Drop
  reset 链路测试全绿可证）。edition 2024 注意：`gen` 是保留字，闭包参数不能叫 `gen`。
- **判别方法可复用**：确定性差异（多 jobs 臂差同一批键）先做 `jobs=1 vs jobs=2`
  小集判别实验复现，再从失败 message（`cloneNode is not a function` 这类 JS 面
  错误）反推绑定安装路径，而不是先怀疑竞态时序。
- **推广审查**：任何「进程级计数/开关守护线程局部状态」的组合，在引入多线程执行
  （rayon worker 复用线程）前必须核对语义前提——同线程串行下成立的蕴含关系
  （代际不符 ⟹ 缓存失效）在并行下可能 silent break。wpt-runner 已并行化的套件
  （reftest、html-semantics）共享此前提的 engine 状态只有此一处（审计于 2026-10-27）。
