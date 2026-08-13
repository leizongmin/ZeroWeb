# 归档：M4 切片 2 — classList token 校验抛 DOMException（双路径）+ native DOMException 构造器

**日期**: 2026-08-13
**轮次**: R2
**Milestone**: M4（WPT dom 上游基线 + 按聚类驱动修复）
**切片**: M4 切片 2（基线暴露的最高 ROI 聚类修复：classList DOMException）
**基线**: `a1feb5dc`（R1 land 后）

## 切片目标

R1 基线聚类分析暴露：`assert_throws_dom` 443 失败中 **405（91%）集中在 `Element-classlist.html`**——classList token 校验（add/remove/toggle/contains 的空/含空白 token）当前抛 `TypeError`，而 WPT `assert_throws_dom` 要求抛真正 `DOMException`（按 name 区分：空→SyntaxError、空白→InvalidCharacterError）。这是 dom/nodes 最高 ROI 修复点。

## 实现产物

### 1. native DOMException 构造器（新基础设施）
**`crates/engine/src/dom_bindings/dom_exception.rs`**（新，~170 行）：
- 全局 `DOMException` 构造器（`new DOMException(message, name)`）——`ObjectTemplate` 纯数据对象（name/message/code/stack 自有属性），无 internal slot 指向 Rust
- `code_for_name(name)`——spec error-names-table 映射（SyntaxError→12、InvalidCharacterError→5 等 20 项）
- 原型 `toString()`（`"name: message"`，spec）+ legacy code 常量（`DOMException.SYNTAX_ERR=12` 等）
- **`pub(super) fn throw_dom_exception(scope, name, msg)`**——Rust 侧校验失败抛 DOMException 实例（供各 dom_bindings 校验点调）
- 注册到 `install_dom_bindings`（mod.rs）

### 2. classList 校验抛 DOMException（双路径同步）
- **native** `dom_token_list.rs require_valid_token`：空 token → `throw_dom_exception("SyntaxError", ...)`；含 ASCII 空白 → `throw_dom_exception("InvalidCharacterError", ...)`（此前抛 `v8::Exception::type_error`）
- **polyfill** `js_dom_shim/part03.js check()`：同 spec 区分 name 抛 `new DOMException(msg, name)`（此前抛 `TypeError`；DOMException 构造器在 part01b.js 已存在）

### 3. A/B 门异常路径扩展
**`tests_ab_compare.rs`**：新增 `ab_catch(html, expr)` helper（try/catch 返 `threw|<name>` 或 `no-throw|<value>`）+ `ab_classlist_token_validation_throws_dom_exception` 测试——断言两路径 classList 异常抛相同 name DOMException（读路径 + 异常路径双重等价守）。

### 4. 既有测试增强
**`tests_collections.rs native_class_list_token_validation_r3145`**：从"只看抛不抛"增强为"断言 name 正确"（空→SyntaxError、空白→InvalidCharacterError），验证新行为。

## 验证证据

| 矩阵 | 命令 | 结果 |
|------|------|------|
| zero-engine v8 lib | `cargo test -p zero-engine --features v8 --lib` | ✅ 2065 passed（+2：A/B 异常测试 + token 增强重排） |
| zero-engine quickjs lib | `cargo test -p zero-engine --no-default-features --features quickjs --lib` | ✅ 1406 passed（A/B 门 + dom_exception 均 cfg(v8) 排除） |
| zero-wpt-runner | `cargo test -p zero-wpt-runner` | ✅ 167 passed |
| clippy v8 + quickjs | `cargo clippy -p zero-engine ...` | ✅ 双矩阵零警告 |
| **testharness-dom 全量复跑** | `make testharness-dom` | **dom/nodes 41.25% → 56.08%（+14.83pp，400 subtest 净 pass，0 回归）** |
| Element-classlist.html 单用例 | `testharness-dom Element-classlist.html` | 80.3% pass（1140/1420） |

**核心结论**: classList DOMException 修复是大幅净正收益（+14.83pp），且 native DOMException 构造器基建（`throw_dom_exception`）为后续其余 DOMException 抛出点（createElement 非法标签、appendChild 闭环等）铺平路。

## 关键决策

1. **双路径同步而非 polyfill-only**：DC-4 A/B 等价是硬约束；只改 polyfill 会破坏等价。native DOMException 基建是一次性投入，后续扩点成本极低（调 `throw_dom_exception`）。
2. **DOMException 用 ObjectTemplate 纯数据对象**：无 internal slot 指向 Rust（不像 Node/Element 持 NodeId），故用最简 `Object::new` + set 属性，不需 FunctionTemplate 继承链。
3. **既有 native classList 测试不绑定异常类型**（只看抛不抛）→ 改 DOMException 不破坏它们；顺带增强为 name 断言验证新行为。

## 未跑

- `make test` 全量（>580s 超时）；聚焦验证覆盖变更面（dom_bindings 新模块 + part03.js + 测试，无渲染热路径）。非 JS 桥热路径代码变更，product-smoke/bench-gate 豁免。

## 下一步（M4 切片 3 候选）

按剩余 ROI：① 其余 DOMException 抛出点（createElement 非法标签 ~10 + appendChild 闭环，`throw_dom_exception` 已就位）② createProcessingInstruction（44）③ 扩 dom/events。
