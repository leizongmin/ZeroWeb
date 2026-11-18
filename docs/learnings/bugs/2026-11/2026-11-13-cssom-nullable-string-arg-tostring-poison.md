---
date: 2026-11-13
modules: engine, js-dom-shim
---

# WebIDL nullable 字符串参数经 ToString 的 null 毒化——双 face 同病 cleanup 失效

## 问题

`element.style.setProperty(prop, null)` 在 ZeroWeb 中从不移除声明（computed
值残留），而 `removeProperty(prop)` 与 `setAttribute('style', '')` 均正常。
WPT css-values 语料 `test_invalid_attr` 每测试尾部依赖该调用做 cleanup，
失败使 in-run 捕获值全部退化为上一测试的残留值，级联毒化整页断言基线
（attr-all-types 103F；修复后同页 +86，calc 两页合计 +61——共享路径的
放大效应远超直接受害面）。

## 根因

三个 style 写 face 对 value 参的 null 处理不对称：

- shim `_zwMStyleProxy` 物化面：`String(value == null ? '' : value)` ✓
- shim `_styleProxy` sel/handle 臂（语料实走路径）：`String(value)` ——
  `String(null)` = `"null"` 四字符 → 非空 → 走值校验 → `width: null` 非法
  → 整体拒绝 → **既有声明原地保留**；
- native `native_style_set_property_invoke`：`string_arg` 用 v8
  `args.get(idx).to_string(scope)`，JS null 的 ToString 同样得 `"null"`。

spec（cssom-1 `dom-cssstyledeclaration-setproperty`）value 是 nullable
`CSSOMString? = ""`：null 归空串，空串臂 ≡ removeProperty。WebIDL
**non-nullable** DOMString（如 IDL 属性 setter `el.style.width = null`）
的 ToString(null)="null" 才是正确语义——两类参数不可共用同一个取参 helper。

代码库已有正确范式：`dom_bindings/mod.rs` 的
`local_value_to_string_null_as_empty`（null → 空串），证明该坑此前已在别处
踩过并修复，但新写面（sel/handle 臂、native setProperty）没有复用该约定。

## 解决

- shim setProperty 臂：`String(value == null ? '' : value)`（与物化面同形）。
- native set_property_invoke：改用 `local_value_to_string_null_as_empty`。

## 避免方式

- 新写 face 处理 WebIDL nullable 参数时，取参必须走 null-aware helper，
  不裸用 `String()`/`ToString()`；检索既有 `*_null_as_empty` 命名范式复用。
- 同一 DOM API 有多个实现 face（物化面/sel-handle 代理面/native 绑定）时，
  参数语义修复须三面同查——本例物化面正确反而掩盖了问题面（语料只走
  sel/handle 臂），行为差异最初被误判为「引擎级 cleanup 缺陷」。
- 排查「值校验拒」类 bug 时先核对**实际到达校验的字符串**：`"null"` 是
  ToString 毒化的指纹（同理 `"undefined"`）。
- 修复验证用三角探针：`setProperty(null)` / `removeProperty` /
  `setAttribute(空)` 三路分离（本例三路两过一挂，直接定位到 setProperty
  取参臂而非存储层）。
