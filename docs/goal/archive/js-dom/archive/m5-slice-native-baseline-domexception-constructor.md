# 归档：M4 切片 5 / R5 — testharness-dom native 路径对照 + native DOMException constructor 修复（部分）

**日期**: 2026-08-14
**轮次**: R5
**Milestone**: M4（WPT dom 上游基线 + 按聚类驱动修复）
**切片**: M4 切片 5（DC-3 native 路径对照 + native DOMException instance identity 修复）
**基线**: `cde39b6b`（R4 land 后）

## 切片目标

R4 发现 testharness-dom 仅测 polyfill 路径，R2/R3/R4 的 native 修复基线不可见。本轮：(1) 给 runner 加 native 路径选项，建立 polyfill vs native 双基线对照（DC-3 硬要求）；(2) 顺带修复发现的 native DOMException instance identity 问题。

## 实现产物

### 1. testharness-dom native 路径入口
- **`tests/wpt-runner/src/testharness.rs`** `run_testharness_html_inner`：读 env `ZW_NATIVE_DOM=1` → `WebViewConfig.native_dom=true`（默认 false polyfill）。零签名扩散（canvas/html-interaction 不设 env 不受影响）。
- **Makefile**：`testharness-dom-native` 目标（`ZW_NATIVE_DOM=1` 前缀）+ `.PHONY`。
- 命令：`make testharness-dom`（polyfill）/ `make testharness-dom-native`（native）。

### 2. native DOMException constructor 修复（部分）
R5 跑 native 基线发现：native 路径 assert_throws_dom 报 **"threw an exception from the wrong global"**（~414 失败）。根因：R2 的 `throw_dom_exception`/构造器用 `v8::Object::new` 建裸对象，instance prototype 是 Object.prototype → `e.constructor === Object` ≠ DOMException。

**修复（dom_exception.rs，保留）**：
- `throw_dom_exception`：改为取全局 DOMException 构造器 `ctor.new_instance`（instance prototype 正确 = DOMException.prototype）
- 构造器 invoke：改用 `args.this()`（new 调用 This prototype 正确）set 属性
- 新增 `fill_instance`（在给定对象上 set name/message/code/stack）

**未完成（回退，留下轮）**：DOMException.prototype 缺 `constructor` 属性 → `e.constructor` 走原型链到 Object.prototype.constructor。补 prototype.constructor 的尝试：
- `proto.set("constructor", Local<Function>)` → V8 Fatal "must be primitive or Template"
- `proto.set("constructor", tmpl)` → V8 CHECK 崩溃（循环引用）
- **回退**，记未解决问题。修复方向：install_dom_bindings 末尾取 prototype **对象**（非 template）set constructor。

## 双基线对照（DC-3 达成）

| 路径 | Pass | Fail | Timeout | Pass 率 |
|------|------|------|---------|---------|
| polyfill | 1512 | 1172 | 12 | **56.45%** |
| native | 1112 | 1572 | 12 | **41.25%** |
| 差距 | -400 | +400 | 0 | **-15.20pp** |

详见 [evidence/2026-08-14-r5-native-vs-polyfill-baseline.md](../evidence/2026-08-14-r5-native-vs-polyfill-baseline.md)。

## 验证证据

| 矩阵 | 命令 | 结果 |
|------|------|------|
| dom_bindings 全测（v8） | `cargo test -p zero-engine --features v8 --lib dom_bindings` | ✅ 197 passed（constructor 改动无回归） |
| clippy v8 + quickjs | `cargo clippy -p zero-engine ...` | ✅ 双矩阵零警告 |
| testharness-dom polyfill | `make testharness-dom` | 56.45%（R3 值维持） |
| testharness-dom native | `make testharness-dom-native` | 41.25%（不崩溃，回退后） |

## 关键洞察

1. **native 落后 polyfill 15.2pp 主因 = DOMException prototype.constructor 缺失**：classList（405）/createElement（10）/node mutation 等 assert_throws_dom 全部因 "wrong global" 失败。修复 prototype.constructor 后 native 预计 +400 subtest（最高 ROI 下轮）。
2. **V8 Template::Set 限制**：prototype template 的 set 只接受 primitive/Template，不接受实例 Function；循环 Template 引用触发 CHECK。须绕开 template，用 prototype 对象实例化后 set。
3. **R2 的 Object::new 是 bug**：dom_bindings 单测只查 `.name`（不查 constructor）所以未暴露；testharness webview 路径暴露。本轮 throw_dom_exception/构造器用 This 的修复是 constructor 修复的前置（prototype 已正确）。

## 下一步（M4 切片 6，最高 ROI）

补 DOMException.prototype.constructor（prototype 对象实例化后 set）→ native 路径预计 +400 subtest，classList/createElement/node mutation DOMException 全转 pass，native 基线有望追平 polyfill。
