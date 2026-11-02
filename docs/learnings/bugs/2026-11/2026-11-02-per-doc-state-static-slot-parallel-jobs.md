---
date: 2026-11-02
modules: engine, js-dom-bridge
---

# per-document 运行时状态用进程级 static 槽承载 → 并行测试 job 下跨页串态

## 问题

`html/semantics/forms/the-button-element/active-onblur.html` 在 WPT runner 全套件跑中稳定 Fail，断言 `document.querySelector(':active')` 期望 button 却得到 **body**——本页从未有任何路径标记 body 按压。单文件复跑（无并发）却 PASS，失败对并发负载敏感。

## 根因

M2-S196 的 `:active` 按压态槽 `DOC_ACTIVE_SELECTOR` 声明为进程级 `static Mutex<String>`（js_dom_bridge.rs），而按压态本质是 **per-document** 状态。runner 执行走 rayon `par_iter` 多 job（默认 `min(CPU-1, 8)` 线程，每线程独立 WebView 顺序跑多页），8 个线程共享同一 static 槽：

- A 页（checkable-active-onblur-with-click）parent-label 压按因布局零盒命中 body，经 `__zw_set_doc_active` 写入 `"body"`；
- 并发 B 页（active-onblur）断言查询经 `sync_active_state` 读槽，把 A 页的 `"body"` 解析进**自己的**视图文档 → `:active` 假匹配 body。

插桩追时序时失败形态还会翻转成「got null」（读到并发页的清除 `""`）——同一根因的两个投影。该 bug 同样影响同进程多 webview 的产品嵌入场景，不止测试假象。

## 解决

槽改 `thread_local! { RefCell<String> }`，与周边查询基础设施（`LIVE_QUERY_DOC`/`VIEW_DOC_CACHE`/`MUT_APPLIED_CURSOR`）同一归属模型。callbacks.rs 既有注释已写明理由：`zero_dom::Document` 非 Send，状态与查询闭包同线程消费，thread_local 即正确归属。修复后 button 页 1F→P，两次复跑稳定。

## 避免方式

- 新增 per-document / per-page 运行时状态时，归属模型必须对齐既有基础设施：runner 多 job 与同进程多 webview 下**任何**进程级可变槽都会跨页串态。判据：状态的生命周期是「一个文档」→ thread_local（或 per-sandbox 持有），不是 static。
- 排查「断言得到从未在本页出现过的元素」类失败时，先怀疑跨页/跨线程串态：单文件复跑若 PASS 而全套件 Fail，串态嫌疑大于引擎语义 bug。
- shim 侧 wrap `globalThis.__zw_*` 回调看不到 shim IIFE 闭包内的同名引用——JS 侧插桩会静默失明；需要全量可见性时直接在 Rust 回调里加 `eprintln!`（含 `std::thread::current().name()` 验证线程归属）。

## 同族变体（2026-11-03 补充）：进程级 env var 作测试开关

**进程级 env var 也是同型槽**。r5296 门关相位（`paint/tests/r5296_inline_bg_ml_gate_tests.rs:104-108`）以 `std::env::set_var("ZW_INLINE_BG_ML_GATE"/"ZW_INLINE_CA_EXPAND", "0")` 置关态后 paint（painter 绘制中读 env），同二进制 r5270 测试（`r5270_inline_owner_selection_fill_tests.rs:74/105`）并行窗口 set/remove 同变量 → r5296 绘制中途读到 gate-ON 默认几何，门关断言翻铃（make test 实录 zero-engine lib 套件 3718P/1F「门关应复现修复前上溢，实得 y=50 盒顶=0」；隔离复跑 2P 与另一轮同套件 3719P 均绿，并行负载敏感性与本条原案同指纹）。`set_var`/`remove_var` Rust 2024 起 unsafe（进程全局可变，多线程并发调用本身即竞态）。测试开关应在测试内直接传参（paint 函数收 gate 参数）或该组用例串行执行；env-var 置位只留给无法注入的进程边界。真修归 paint/layout 流。
