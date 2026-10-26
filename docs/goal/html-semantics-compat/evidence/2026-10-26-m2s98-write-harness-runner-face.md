# M2 切片 98：write 注入 harness 的 runner 面完成探测接线

日期：2026-10-26。范围：`tests/wpt-runner/src/testharness.rs` fetch 通道静态服务面——execution-timing/102（write 注入 harness 全家桶）runner 完成探测销账。上轮：[M2 切片 97](2026-10-26-m2s97-document-write-run-face.md)。

## 1. 入场核账

- s97 基线（/tmp/s97-scripting1-final.txt，scripting-1 口径 574 行）：311P/229F/16T + 18 NotRun。
- 102 挂账定谳（s97）：页脚本全执行（写入 harness 全家桶依序跑通），但 write 注入的 testharness.js 经 fetch 通道取回 RAW 副本——无 `__zw_mark_harness_loaded`/`__zw_harness_state` 完成探测钩子，超时转储 `hook=undefined/state=null`。
- 完成机制核实（pinned testharness.js）：`Tests.prototype.notify_result` → `all_done()`（要求 `test_environment.all_loaded && num_pending===0`）→ `complete()` → completion 回调 → 探针全局 `__zw_harness_complete`。runner 终态判据 `terminal_harness_state`（phase=4 + pending=0 + tests=结果数）或 `state 为 null && __zw_harness_complete`。
- 静态路径 vs write 路径不对称：`prepare_harness_html` 把静态 `<script src="/resources/testharness.js">` 标签替换为补丁源内联（`__zw_mark_harness_loaded` replacen + reporter 装配快照），生命周期脚本尾补调钩子（全部页面脚本后同步置 all_loaded）；102 初始文档无静态 harness 标签（只有 write 串里的 src），补丁与 reporter 都到不了注入副本——且 `document.open()` 换文档后 load 事件不重发，注入副本的 `all_loaded` 恒 false，末测 done 后永不 complete。

## 2. 接线（fetch 通道静态服务特判）

`wpt_data_fetch_handler` 通用静态回退（`std::fs::read(root.join(clean))` Ok 臂）内：

- `resources/testharness.js` → `apply_harness_completion_patch` 补丁源（与静态内联同一 replacen，抽公共函数防两路漂移）+ **eval 尾自调** `__zw_mark_harness_loaded()`（静态路径由生命周期脚本补调；fetch 服务源须自带——注入副本求值完成即置 all_loaded；此刻 `tests.length===0`，all_done 为 false，无误完成）。
- `resources/testharnessreport.js` → reporter 装配快照（`setup({output:false})` + GLOBAL shim + `__zw_harness_results`/`__zw_harness_complete` 回调装配）——上游 report 是裸 loader，注入副本经它注册的回调到不了探针全局；与静态路径移除 report 标签内联 reporter 等价。
- 装配去重：reporter `r#""#` 提升为 `HARNESS_REPORTER_SOURCE` 模块常量、replacen 提为 `apply_harness_completion_patch`——静态内联（prepare_harness_html）与 fetch 服务共用同一装配面。

## 3. 影响面

- 全语料核查：write/dynamic 注入这两个路径的页面仅在 scripting-1（102 形）——静态标签一律被 prepare_harness_html 内联替换走不到 fetch 通道，XHR/fetch 断言字节面无已知用例；blast radius 有界。
- 时序语义：补丁源尾自调等价真实浏览器「document.open 新文档 load 事件置 all_loaded」的近似（本引擎 open 后不重发 load）。

## 4. 验证

- execution-timing 全目录：s97 97P/51F/2T → **98P/51F/1T**（/tmp/s97-exec-timing-final.txt vs /tmp/s98-exec-timing.txt 逐行 diff 仅 102 行）。
- 全子树终扫（scripting-1 口径 574 行）：s97 311P/229F/16T + 18 NotRun → **312P/229F/15T + 18 NotRun**，净 +1 Pass，行级 diff **仅 102 Timeout→Pass**、零 P→非P（/tmp/s97-s1-sorted.txt vs /tmp/s98-s1-rows.txt）。
- 102 结果行 `Pass :: scheduler: defer script after initial onload event`——写入 harness + report + testlib 依序执行、async_test 断言 eventOrder 通过后 t.done() 完成 harness（非空转判绿）。
- zero-wpt-runner bin 单测 **226 passed / 0 failed**（guard；fetch handler fixture 单测在内）。
- 纯 runner .rs 变更：fmt/clippy 不涉 render/layout 面，reftest 沿 s91-s97 口径未跑。

## 5. 范围与挂账

- 触碰面：testharness.rs ×1（fetch 静态服务特判 + 装配常量/公共函数提升）；engine shim 零变更。
- 残余挂账（承袭 s97）：116 frameset 文档语义缺口；063 Timeout 承袭；module 尾片（.py 前置）；charset 12F（.py）；svg-script-fragment-clone 3F；nomodule-async 2F；load-error-events 族；FIXME 组（bufTop 漂移 / setAttribute('src') / type 门 / children-changed 其余入口）；接口对象基建面。
