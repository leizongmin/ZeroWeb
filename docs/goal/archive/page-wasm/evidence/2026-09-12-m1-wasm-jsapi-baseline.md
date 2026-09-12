# M1 / DC-1 — wasm/jsapi window 子集通过率基线（stub 导出面假设被推翻）

**日期**: 2026-09-12
**套件**: `make testharness-wasm`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-12-m1-wasm-jsapi-baseline.json](2026-09-12-m1-wasm-jsapi-baseline.json)

## 结果

| 指标 | 值 |
|---|---|
| 导入用例 | 31 案（wasm/jsapi 标准 JS API 面，非 tentative） |
| subtests | **715/719 = 99.4% Pass** |
| 全绿用例 | 27/31 |
| 非 Pass 状态 | Timeout × 4（零 Fail/Unsupported） |

全部 4 个非绿都在 `constructor/` 域，失败形态一致：testharness **completion callback
未调用**，state 里 `pending:N`（N 个 promise_test 的 promise 永不 settle）：

| 用例 | 绿/总 | pending |
|---|---|---|
| constructor/compile.any.js | 3/4 | 6 |
| constructor/instantiate.any.js | 2/3 | 55 |
| constructor/instantiate-bad-imports.any.js | 113/114 | 99 |
| constructor/multi-value.any.js | 0/1 | 3 |

## 关键发现一：页面路径跑的是 V8 原生 WebAssembly（goal 基线假设修正）

goal 立项基线假设「页面 WASM = dom_bridge polyfill stub → 导出面/类型是瓶颈」。实测：

- 生产页面路径 `run_page_scripts`（js_dom_shim + native 绑定）**不安装**
  `generate_dom_api_polyfill()`——`globalThis.WebAssembly` 就是 V8 原生完整实现。
- polyfill（stub 导出面/仅 I32）只装在 `execute_script_with_dom`（embedder execute
  路径 + `tests/wpt-runner` wasm_bridge 类测试消费）。

因此 WPT window 面一次即 99.4%：M1 原计划的「类型映射/exports 真实化」切片针对的
polyfill **不在这条验收路径上**。后续切片优先级按本证据重排（见 master.md）。

## 关键发现二：4 案 Timeout 根因 = V8 前台消息循环从不泵（跨流卡点）

**证据链**：

1. 全绿 27 案只用**同步**面 `new WebAssembly.Module/Instance`
   （`grep -c "await WebAssembly|WebAssembly.compile("` 全 0）。
2. 非 Green 4 案全部依赖**异步** `WebAssembly.compile()/instantiate()` 返回的
   promise——runner probe 循环 10s 内反复 `execute_script`（每次收尾都有
   `perform_microtask_checkpoint`）仍不 settle。
3. `crates/script-sandbox/src/v8_runtime.rs` `ensure_v8_initialized` 用
   `v8::new_default_platform`，`execute` 收尾只 `perform_microtask_checkpoint()`，
   **全仓无一处 `PumpMessageLoop`**。

**机制**：V8 异步 wasm 编译在后台线程完成后，把「resolve promise」的前台任务投递到
platform 默认 foreground task queue；embedder 只泵 microtask、从不泵消息循环 →
resolve 任务永不执行 → promise 永挂 → `pending:N`。这也解释了为何同一 runner 里
同步 wasm 全绿而异步面零绿，且影响面不止 wasm——一切以 V8 前台任务队列收尾的
异步 builtin 都挂。

**修复位置**：`crates/script-sandbox`（v8_runtime 泵消息循环）——按 run-rules §9
该 crate 属 event-loop-spec 流域，本流**不硬解**，记 master.md 待协调（消息循环泵
本就是 event loop spec 化的天然组成件）。

## M1 切片 1 交付物

- `tests/wpt-runner/scripts/fetch-wasm-subset.sh`（pinned rev；jsdelivr 首选 + raw
  回落——raw.githubusercontent.com 本网络下 30s 仅 7KB，GitHub API 限速 403）
- runner `testharness-wasm` 子命令 + `WASM_CASES` 清单（META script 自动解析）
- `make fetch-wpt-wasm` / `make testharness-wasm`
- `imported-testharness.txt` +31 行（PWASM-M1-baseline）
- skip 域（fetch 脚本头注释）：esm-integration / jspi / js-string / gc / exception /
  tag / function(tentative) / functions(realm harness) / idlharness / memory tentative 面
