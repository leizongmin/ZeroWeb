# 归档：M4 切片 6 / R6 — DOMException identity 三重根因修复（native 追平 polyfill）

**日期**: 2026-08-14
**轮次**: R6
**Milestone**: M4（WPT dom 上游基线 + 按聚类驱动修复）
**切片**: M4 切片 6（native DOMException identity 闭环 R5）
**基线**: `e7684e42`（R5 land 后）

## 切片目标

R5 定位 native "wrong global" 根因（prototype.constructor 缺失）但修复 V8 Fatal 回退，native 卡 41.25%。本轮用 webview 叠加诊断测试精确定位**真正的三重根因**并全部修复，使 native 追平 polyfill。

## 三重根因与修复

R5 只发现根因 1（prototype.constructor）。R6 webview overlap 诊断测试（`test_native_dom_exception_identity_overlap_r6`）揭示了 R5 漏掉的根因 2（关键）。

### 根因 1：DOMException.prototype 缺 constructor 属性
FunctionTemplate prototype template 的 set 不接受 Local<Function>（V8 Fatal），传 tmpl 自身循环 CHECK。
**修复**：`build_and_register` 末尾取构造器 function 的 `prototype` **对象**（`func.get("prototype")`），对象 set constructor（接受任意 Value）。

### 根因 2（R5 漏掉的真因）：install_dom_bindings 多次调用建不同构造器
webview native_dom=true 路径下 `install_native_dom_bindings` 被多次调（run_page_scripts + execute_script 各一次），每次建新 FunctionTemplate 覆盖全局 DOMException。classList 抛的异常持有第一次 install 的构造器，全局是后一次的 → `e.constructor === DOMException` false（两者都是 native function 但是不同实例）。

诊断证据：`__ex.constructor` 与全局 `DOMException` 都 `[native code]` 但 `!==`。

**修复**：`dom_exception::build_and_register` 幂等——全局已有 DOMException 则跳过重建（复用首次 install 构造器）。

### 根因 3：polyfill shim classList 抛词法作用域 DOMException
shim part03.js `check()` `throw new DOMException` 的 `DOMException` 解析到 part01b 词法作用域（非全局 native）。叠加路径下 shim 实例 constructor（part01b）≠ 全局 native。
**修复**：part03.js `check()` 改用 `globalThis.DOMException`。

## 实现产物

- **`dom_bindings/dom_exception.rs`**：prototype.constructor 补齐（prototype 对象 set）+ `build_and_register` 幂等（全局已有则跳过）
- **`js_dom_shim/part03.js`**：classList `check()` 用 `globalThis.DOMException`
- **`webview/tests/coverage.rs`**：`test_native_dom_exception_identity_overlap_r6`（webview 叠加路径 DOMException identity 断言，R6 从诊断转正式）
- **`dom_bindings/tests_collections.rs`**：`native_dom_exception_identity_r6`（纯 native identity）

## 验证证据

| 矩阵 | 命令 | 结果 |
|------|------|------|
| zero-engine v8 lib | `cargo test -p zero-engine --features v8 --lib` | ✅ 2073 passed |
| zero-webview webview_coverage | `cargo test -p zero-webview --test webview_coverage` | ✅ 17 passed（含 overlap 测试） |
| clippy v8 + quickjs | `cargo clippy ...` | ✅ 双矩阵零警告 |
| **testharness-dom native 全量** | `make testharness-dom-native` | **41.25% → 56.08%（+14.83pp，400 subtest 净 pass，0 回归）** |
| native classList 单用例 | `testharness-dom-native Element-classlist.html` | 52.1% → **80.3%**（=polyfill） |

**核心结论**: native 路径 dom/nodes 追平 polyfill（56.08% vs 56.45%，差仅 0.37pp）。classList/createElement/node mutation 全部 DOMException 抛出点在 native 路径转 pass。

## 关键洞察

1. **R5 根因定位不完整**：R5 只发现 prototype.constructor，漏掉更关键的「多次 install 建不同构造器」。R6 webview 叠加诊断测试（模拟真实 testharness 路径，而非裸 Isolate）才暴露根因 2——**dom_bindings 单测（裸 Isolate）无法发现 webview 叠加路径问题**，webview 层测试不可或缺。
2. **install_dom_bindings 幂等的普遍性**：Event/HTMLElement 等其他构造器也可能有多次 install 问题（只是 DOMException 因 assert_throws_dom constructor 检查暴露）。后续可推广幂等模式。
3. **native 追平 polyfill 是 default-on 关键前置**：DC-1 default-on 要求 native 路径规范合规，R6 证明 native dom/nodes 与 polyfill 对等。

## 下一步（M4 切片 7 候选）

按剩余 ROI：① createProcessingInstruction（44）② instanceof HTMLElement/Element 原型链（~88）③ 扩 dom/events。
