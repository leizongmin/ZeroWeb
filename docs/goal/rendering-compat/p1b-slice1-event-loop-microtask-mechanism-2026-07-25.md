# P1b 首切片机制方案 — event loop microtask queue

**日期**：2026-07-25
**性质**：P1b（JS Bridge 原生化）轨的可执行首切片机制方案（**非完整 P1b RFC**，首切片探索）。master.md 列 P1b 需独立 RFC，本文档识别最底层可执行首切片。
**关联**：[blockers-resolution-plan-2026-07-25.md](blockers-resolution-plan-2026-07-25.md) §4、[js-dom-bridge-design.md](js-dom-bridge-design.md)

---

## 目标

让 **Promise.then / microtask 真实化**——这是 fetch / MutationObserver / setTimeout 真实化的**共同基础**（三者都依赖 microtask 交付回调）。当前 dom_bridge 的 `Promise.resolve` 是同步 polyfill（dom_bridge.rs:975/977/997），`execute_script_direct` 同步执行无 microtask queue。

## 注入点（基于已有代码评估）

- `page-runtime` `JsExecutor::execute_script_direct`（同步执行 JS 串）
- 调用点：`tab_scripts.rs:156/211`（browser）+ `page_scripts.rs:105/144`（renderer）—— execute 后即返回
- **microtask drain 注入点**：execute_script_direct 返回后，Rust 循环调 JS 全局 `__zwDrainMicrotasks` 直到 queue 空

## 机制（方向，非代码）

1. **JS 侧 dom_bridge**：加 microtask queue（数组）+ `Promise.then` 注册回调到 queue（替代当前 `Promise.resolve` 同步执行 then）
2. **Rust 侧**：每次 `execute_script_direct` 返回后，drain loop——调 `__zwDrainMicrotasks`（执行 pending callback），直到 queue 空
3. **microtask 语义**：microtask 内产生的新 microtask 在同 drain 循环执行（当前 task 结束前清空，≈ HTML spec microtask checkpoint）

## 验证

- `Promise.resolve("a").then(v=>log(v)).then(...)` 链式真实触发（单测：回调顺序 a→b→...）
- 现有集成测试零回归（对不用 Promise.then 的脚本，execute 行为不变）
- product smoke（welcome/wintertc）字节一致

## 风险

- **架构级**：dom_bridge shim 路径，master.md 列 P1b「需独立 RFC」
- register_callback 当前同步；microtask drain 在 execute 后同步调用——**可避免真正异步**（仍是 execute→drain 同步序列），但属 shim 增强，非 V8 原生 event loop
- **本机制是首切片探索，非完整 P1b RFC**：节点身份（selector→NodeId）、fetch 真实化、MutationObserver、setTimeout 真实延迟是后续切片（依赖本 microtask 基础）
- 净负即回退

## 与 P1b 整体的关系

本首切片是 P1b 的**最底层基础**（microtask queue），**不解**节点身份 / fetch / Observer / setTimeout（后续切片依赖本基础）。成功 = Promise.then 真实 + 为 fetch/Observer 铺路。完整 P1b（V8 原生绑定 + 真实 event loop）仍需独立 RFC。

## 依据

- 代码：`page-runtime/src/lib.rs`（JsExecutor trait）+ `tab_scripts.rs:156/211` + `dom_bridge.rs:975/977/997`（Promise.resolve 同步 polyfill）
- 评估：R2025 P1a 摸底结论（fetch/MutationObserver/事件循环都依赖异步 microtask，当前 selector-shim + 同步 execute 模型下只能 stub）
