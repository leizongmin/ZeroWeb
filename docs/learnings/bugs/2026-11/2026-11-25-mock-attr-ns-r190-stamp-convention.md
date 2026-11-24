---
date: 2026-11-25
modules: engine, dom
---

# mock 属性 ns 保真必须走 R190 印记约定，写 DOM 字段名被读面静默忽略

## 问题描述

HTML 解析产物（`_zwMEl` plain 节点）的 foreign namespace 属性（`xlink:href`/`xml:space`）在 isEqualNode 与 Attr getter 读面上恒为 null ns，`sanitizer-basic-filtering.html` namespaces/0-9 46F＋javascript-url 同根面整簇失败。引擎发射端与解析器产物均正确（RAW JSON 已含 SVG/MathML 元素 ns 与 adjust 命中属性全三元组），丢失全部发生在 shim JS 物化/重建两端。

## 根因分析

两层叠加，且第一层修复时极易再次踩坑：

1. **读面字段名约定**：`_zwMEl` 在工厂尾部（约 25995 行）用 `Object.defineProperty(node, 'attributes', _zwDescAttributes)` 把构造字面量里的裸 attrs 数组替换为 NamedNodeMap Proxy；Attr 条目经 `_zwMAttrNS` 物化，该读面只消费 **R190 印记约定** `{ns, prefix, local}`（小写短字段，`entry.ns !== undefined` 门控）。给裸条目写 DOM 风格字段名（`namespaceURI`/`localName`）不报错、不生效——物化器直接读不到，静默丢失。
2. **重建域丢 ns**：`_wcRebuildAsHandle` 旧实现 `createElement(tag)`（全 HTML ns）＋attr 循环 `setAttribute(qname)`（null ns），svg/math 元素与带前缀属性在 handle 域双双丢 ns。

诊断侧的二次陷阱：probe 对 mock 节点直接 `JSON.stringify(attr)` 会因真实 Attr 对象的 `ownerElement` 回环抛 Converting circular structure——该报错反而是「读面已物化为真 Attr」的定位信号（attributes accessor 真身暴露）。

null-ns 形（普通 HTML 属性＋表外 `xmlns:foo`）**不印记即正确**：解析器保持 null ns＋整 qname local，bare `_zwMakeAttr` 缺省形（localName=整 qname、ns/prefix=null）恰好是 spec 的 setAttribute 语义。adjust-foreign-attributes 只枚举 xlink:*/xml:*/xmlns/xmlns:xlink，表外任意前缀不在归 ns 范围。

## 解决方案

- 发射端：`deep_attr_entry_json` 统一 attr 条目 JSON（adjust 命中形追加 `ans`/`al`/`ap`），双发射器同源。
- 物化端：`_zwMApplyAttrNsMeta` 只对带 `ans` 条目写 `d.ns`/`d.prefix`/`d.local`（R190 印记字段名），双物化器调用；索引/item/iterator/getNamedItem 读面与 `_zwEqualAttrsOf` isEqualNode 配对自动点亮。
- 重建端：非 HTML ns 元素 `createElementNS`＋stamped attr `setAttributeNS`（失败回落 qname setAttribute）。

避免方式：给 `_zwMEl` plain attr 裸条目补任何元数据前，先查 `_zwMAttrNS`/`_zwEqualAttrsOf` 的消费字段名；消费面约定的字段名（如 R190 印记）与本面自然命名的差异是本类静默丢失的通用根因。判读绑定：`make test test_s10k13_foreign_ns_knife13_pins`（part159 四组钉，含 isEqualNode 反向假恒真守卫）。
