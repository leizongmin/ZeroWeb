# M6 S1q 复合对象收口 — dataset DOMStringMap（R71）

**日期**: 2026-08-16
**Commit**: `93eccce4`
**前置**: R70（classList DOMTokenList，`7e4c63c4`）
**证据**: [evidence/2026-08-16-r71-quickjs-s1q-dataset.json](../evidence/2026-08-16-r71-quickjs-s1q-dataset.json)

## 背景

S1q 复合对象第三片（收口）。dataset 是框架状态携带（Vue/Alpine/htmx data-* 属性桥）高频面。

## 实现

**形态决策**：rquickjs 无 V8 `named-property-handler` 的 Rust 等价物（不暴露 `JS_NewProxy`）——动态键拦截改经 **JS Proxy 胶水**：

1. **Rust 原语四件**（install 注册全局）：`__zw_native_ds_get/set/delete/keys`（收 owner ffi + camelCase 键），驼峰↔kebab 转换镜像 V8 `dataset.rs`。
2. **JS 工厂脚本**（install 一次性 eval）：`__zw_native_ds_make(ffi)` 建 Proxy（六 trap：get/set/deleteProperty/ownKeys/getOwnPropertyDescriptor/has）；非 string 键 fallthrough target（对象协议保持）。
3. **dataset_getter**：调工厂 + `DATASET_OBJECTS` 二级身份缓存（R69 模式第三次复用）。
4. **语义细节**：delete miss 键 no-op 成功（spec DOMStringMap——Proxy invariant 要求 trap 返 true 否则 TypeError 抛出，观察语义与 Web 一致）；set/delete 派发 attributeChangedCallback（R68 old 写前捕获）。

## 验证

- PoC 断言四组：身份缓存 / camel 写→data-kebab 元素侧可见 / kebab 读回 camel + ownKeys 枚举 + has / 删除同步 + miss no-op
- engine quickjs **1419** / v8 **2153** 全绿零回归；clippy 双矩阵零警告；fmt 无 diff
- pre-commit-guard PASS

## 过程注记

1. `__ds['x-y-z']` 直访期望 undefined 实得 `w`——`prop_to_attr('x-y-z')` = `data-x-y-z` 恒等往返命中同属性，这是转换函数的正确行为（kebab 键也是合法 dataset 访问形态），期望值修正非实现修正。
2. QuickJS Proxy deleteProperty trap 返 false 会抛 `could not delete property` TypeError——miss 删除必须显式返 true。

## 里程碑注记

**S1q 复合对象三件套（attributes/classList/dataset）全部落地，S1q 收口**。R69 建立的二级身份缓存模式三次复用验证为可复制模式。

## M6 剩余

S0q 续 weak/finalizer（V8 R3133 对等物）→ Event 构造器 → DOMException 构造器 instanceof 面 → whenDefined 真 pending。
