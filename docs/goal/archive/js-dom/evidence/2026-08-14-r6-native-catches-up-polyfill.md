# Evidence: R6 — native dom/nodes 追平 polyfill（DOMException identity 修复）

**日期**: 2026-08-14
**轮次**: R6
**Commit**: 本切片 land commit（见归档）
**分类**: `dom/nodes`（141 用例 / 2696 subtest）

## 测试命令

```bash
make testharness-dom            # polyfill（56.45%）
make testharness-dom-native     # native（R6: 56.08%，R5: 41.25%）
```

## 双路径通过率（R5 → R6）

| 路径 | R5 Pass 率 | R6 Pass 率 | 变化 |
|------|-----------|-----------|------|
| polyfill | 56.45% | 56.45% | —（未动 polyfill） |
| **native** | 41.25% | **56.08%** | **+14.83pp（+400 subtest）** |
| 差距 | -15.20pp | **-0.37pp** | native 几乎追平 polyfill |

## 根因定位与修复（R6 闭环 R5）

R5 定位 native "wrong global" 根因为 prototype.constructor 缺失，但 R5 修复尝试 V8 Fatal 回退。R6 用 webview 叠加诊断测试（`test_native_dom_exception_identity_overlap_r6`）精确定位**真正的双重根因**：

### 根因 1：DOMException.prototype 缺 constructor 属性
FunctionTemplate prototype template 的 `set` 不接受 `Local<Function>`（V8 Fatal），传 tmpl 自身循环引用触发 CHECK。
**R6 修复**：`build_and_register` 末尾取构造器 function 的 `prototype` **对象**（`func.get("prototype")`），在其上 set constructor（对象 set 接受任意 Value）。

### 根因 2（R5 未发现的真因）：install_dom_bindings 多次调用建不同构造器
webview native_dom=true 路径下 `install_native_dom_bindings` 被**多次调**（run_page_scripts + execute_script 各一次），每次建新 FunctionTemplate 覆盖全局 DOMException。classList 抛的异常持有**第一次** install 的构造器，全局是**后一次**的 → `e.constructor === DOMException` false（即便两者都是 native `[native code]` function，但是不同实例）。

诊断证据（R6 webview overlap 测试 eprintln）：
```
constructor===DOMException: false
DOMException.toString: function () { [native code] }   # 全局是 native
__ex.constructor: function () { [native code] }         # 抛出的也是 native，但是不同实例
```

**R6 修复**：`dom_exception::build_and_register` 幂等——全局已有 DOMException 则跳过重建（复用首次 install 的构造器）。

### 根因 3：polyfill shim classList 抛词法作用域 DOMException
shim part03.js `check()` 用 `throw new DOMException(...)`，`DOMException` 解析到 part01b 词法作用域（非全局 native）。叠加路径下 shim 实例 constructor（part01b）≠ 全局 native。
**R6 修复**：part03.js `check()` 改用 `globalThis.DOMException`（叠加路径下 = native）。

## 修复后验证

webview overlap 测试（`test_native_dom_exception_identity_overlap_r6`，模拟 testharness 叠加路径）：
- `classList.add('')` 抛 SyntaxError DOMException ✅
- `e.constructor === DOMException` = **true** ✅
- `e instanceof DOMException` = **true** ✅

native classList 单用例：52.1% → **80.3%**（与 polyfill 持平）。
native dom/nodes 全量：41.25% → **56.08%**（+14.83pp，400 subtest 净 pass，0 回归）。

## DC-3 达成度（R6 更新）

- ✅ **polyfill 基线**：56.45%
- ✅ **native 基线**：56.08%（R6 从 41.25% 追平，差仅 0.37pp）
- ✅ **双路径对照**：native 几乎与 polyfill 对等（DC-3 + 双引擎对等原则的 V8 侧达成）
- 剩余 0.37pp 差距 = polyfill R3 createElement 双路径修复多出的部分（native createElement 校验已在 R3 land，差异来自其他面，待后续定位）

## 结论

R6 闭环 R5 的 native DOMException identity 问题——三重根因（prototype.constructor + 幂等 install + shim 全局 DOMException）全部修复。native 路径 dom/nodes 通过率追平 polyfill，是 default-on（M5）的关键前置（native 路径规范合规性达成）。
