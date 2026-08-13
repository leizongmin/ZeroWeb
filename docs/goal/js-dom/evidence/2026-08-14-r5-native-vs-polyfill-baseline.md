# Evidence: R5 — dom/nodes polyfill vs native 双路径通过率基线对照（DC-3）

**日期**: 2026-08-14
**轮次**: R5
**Commit**: 本切片 land commit（见归档）
**分类**: `dom/nodes`（141 用例 / 2696 subtest）

## 测试命令

```bash
make testharness-dom            # polyfill 路径（默认 WebViewConfig.native_dom=false）
make testharness-dom-native     # native 路径（ZW_NATIVE_DOM=1 → native_dom=true）
```

## 双路径通过率基线对照

| 路径 | Pass | Fail | Timeout | Pass 率 |
|------|------|------|---------|---------|
| **polyfill**（生产当前路径） | 1512 | 1172 | 12 | **56.45%** |
| **native**（default-on 目标路径） | 1112 | 1572 | 12 | **41.25%** |
| **差距** | -400 | +400 | 0 | **-15.20pp** |

## 关键洞察：native 落后 15.2pp 的根因

native 路径 `assert_throws_dom` 失败 ~414 次（与 polyfill R1 原始基线同量），集中在 `Element-classlist.html`（405）。R2 的 classList DOMException 修复在 **dom_bindings 单测通过**（裸 Isolate），但在 **testharness webview native 路径失败**——报 **"threw an exception from the wrong global"**。

### 根因（已定位）
WPT `assert_throws_dom` 最后一步检查 `e.constructor === self.DOMException`（testharness.js:2441）。native DOMException 实例经构造器 new（prototype = DOMException.prototype，正确），但 **DOMException.prototype 缺 `constructor` 属性** → `e.constructor` 走原型链到 `Object.prototype.constructor === Object` ≠ DOMException → "wrong global" 失败。

### 修复尝试（本轮）
1. ✅ `throw_dom_exception` 改取全局 DOMException 构造器 `new_instance`（instance prototype 正确，保留）
2. ✅ 构造器 invoke 改用 `args.this()`（new 调用 prototype 正确，保留）
3. ❌ prototype template set constructor：`proto.set("constructor", function)` → V8 Fatal（"must be primitive or Template"）；`proto.set("constructor", tmpl)` → V8 CHECK 崩溃（循环引用）。**回退**，留下轮用 prototype 对象实例化后 set 的方式补。

### 本轮保留的改动（语义更正确，不崩溃）
- `throw_dom_exception`：instance 经 DOMException 构造器 new（而非裸 Object::new）→ prototype 正确
- 构造器 invoke：用 `args.this()` set 属性
- 这两项是 constructor 修复的前置，prototype.constructor 属性补齐后即可让 native assert_throws_dom 大量转 pass

## DC-3 达成度

- ✅ **polyfill 基线**：56.45%（R2/R3 双路径修复驱动，41.25%→56.45%）
- ✅ **native 基线对照**：41.25%（本轮建立，DC-3「native 路径对照」硬要求达成）
- ⚠️ **native 落后 polyfill 15.2pp**：定位根因（prototype.constructor 缺失），修复方向明确——native 路径 default-on（M5）前须补齐

## 结论与下一步

- **本轮价值**：建立 polyfill vs native 双基线对照，量化 native 路径差距（15.2pp），定位主因（DOMException prototype.constructor），为 M5 default-on 前的 native 补齐提供精确目标。
- **下轮最高 ROI**：补 DOMException.prototype.constructor 属性（prototype 对象实例化后 set 模式）→ 预计 native 路径 +400 subtest（classList + createElement + node mutation 全部 DOMException 抛出点），native 基线有望追平甚至超越 polyfill。

## 注意

- 用例 gitignored（`fetch-dom-subset.sh` 按需拉取）——基线复现须先 `make fetch-wpt-dom`。
- native 路径基线需 `ZW_NATIVE_DOM=1`（`make testharness-dom-native`）。
