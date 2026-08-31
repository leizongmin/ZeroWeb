# M6 S5q 完整化 — observedAttributes 过滤 + oldValue 写前捕获（R68）

**日期**: 2026-08-16
**Commit**: `d6dc00d5`
**前置**: R67（S4q 三阶段派发，`22574a08` rebase 后）
**证据**: [evidence/2026-08-16-r68-quickjs-s5q-observed-attrs.json](../evidence/2026-08-16-r68-quickjs-s5q-observed-attrs.json)

## 背景

R65 落地 attributeChangedCallback 时两个简化注记：oldValue 恒 null（未写前捕获）+ observedAttributes 不过滤（全部派发）。本切片闭合两项，master.md「R67 后下一步首选」第 ③ 项。

## 实现

1. **oldValue 写前捕获**：`set_attribute_method` 在 `set_reflected_attr` 前读 `d.get_attribute(id, name)`，作为 `old` 传入派发（缺失 → JS null）。
2. **removeAttribute 派发**：移除已存在属性 → `attributeChangedCallback(name, old, null)`；缺失属性移除 no-op 不派发（spec 仅已存在属性的移除才是变更）。
3. **observedAttributes 过滤**：`dispatch_attribute_changed` 内求值 ctor 的 `observedAttributes`（`get::<Array>` 泛型调用——静态属性/getter 均可），name ∈ 数组才派发；缺失/非数组 → observe-all（R65 行为兼容）。
4. **同值 set 仍派发**（spec 无值变化短路——区别于 MutationObserver 的 same-value 语义）。

## 验证

- PoC 断言两组：`data-k:v1:v2|data-k:v2:v2|data-k:v2:null`（old 捕获 + 同值派发 + remove new=null + 缺失 remove 不派发）；`observedAttributes = ['data-obs']` 时 `data-skip` 跳过、`data-obs:null:y` 派发
- engine quickjs **1419** / v8 **2153** 全绿零回归；clippy 双矩阵零警告；fmt 无 diff
- pre-commit-guard PASS

## M6 剩余

S0q 续 weak/finalizer → S1q 复合对象（attributes/classList/dataset 二级身份缓存）→ Event 构造器 → DOMException 构造器 instanceof 面 → whenDefined 真 pending。
