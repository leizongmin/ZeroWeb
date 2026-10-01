---
date: 2026-10-02
modules: engine
---

# textContent= 突变后 gBCR 返回文本量盒——写路径注册副作用劫持读路径语义

## 问题描述

页面脚本对任意 selector-identity 元素（querySelector/getElementById 取得的已渲染元素）执行 `textContent = "..."` 后，该元素的 `getBoundingClientRect()` 从真实布局盒（如 `[0,260,70,19]`）变成 0 基文本量盒 `[0,0,文字宽,ascent+descent]`（如 `[0,0,6,10]`），且三条身份路径（getElementById / querySelector / 位置索引）返回同一个错值。绘制（截图）完全正确——只有几何查询错。

同场景还有第二个独立缺陷：非突变元素的锚 gBCR 塌缩成行盒全宽 `(0,10,800,23)`、absolute 兄弟整体 +10px 漂移（这是布局层问题，由 SetText 增量布局臂缺全量后处理族导致，见 pipeline `render_with_dom_mutations_persistent` 注释）。本文聚焦第一层：布局已经正确、gBCR 仍错。

## 根因分析

shim 的 gBCR trap（`js_dom_shim/part04.js`）在查真实布局 rect **之前**先查 R34xx 文本几何注册表：

```js
if (prop === 'getBoundingClientRect') {
  return function() {
    if (typeof _zwTextElBoundingRect === 'function') {
      var _zwR = _zwTextElBoundingRect(sel, handle);   // 先查注册表 ← 劫持点
      if (_zwR) return _zwR;
    }
    return _domRectFromId(sel || handle) || _makeDomRect(0, 0, 0, 0);
  };
}
```

而 `textContent=` / `innerHTML=` 的 setter（part04）会把目标元素注册进该注册表（part06 `_zwRegisterTextEl`，R34xx 为 selection-rects / caret 测试建的服务，对**created/detached** 元素返回本地 0 基文本量几何）。注册表本没有覆盖"已渲染的 selector-identity 元素"的意图，但注册条件只判 `_tcVal !== ''`，不区分元素是否在真实布局里。于是：

- 突变前：元素未注册 → gBCR 走 `_domRectFromId` → 正确；
- 突变后：元素入表 → gBCR 先命中 `_zwTextElBoundingRect` → 恒返 `DOMRect(0, 0, textWidth, ascent+descent)`；
- `getClientRects` 未查注册表 → 同页同元素两 API 不同源（矛盾暴露点）。

深层模式：**写路径（textContent setter）为实现细节（本地文本子注册）附带了一个读路径可见的身份副作用（进 gBCR 特查表），且该表优先级高于权威数据源**。排查时"布局对、查询错"的组合把注意力引向 rect 桥/身份解析，实际劫持点在 trap 的优先级顺序。

定位关键：`_zwTextElBoundingRect` 返回值形状（x=y=0、宽=文本量宽、高=em 盒）与错值逐字节同形；身份三路径同值排除 JS 侧 wrapper 分裂；截图正确排除布局/绘制层。

## 解决方案

gBCR trap 改真实布局 rect 优先，注册表本地几何仅兜底无布局元素（`_domRectFromId` 未命中时）——R34xx 原始受众（created/detached）不变：

```js
var _zwLayoutR = _domRectFromId(sel || handle);
if (_zwLayoutR) return _zwLayoutR;
if (typeof _zwTextElBoundingRect === 'function') { ... }
```

spec 依据：CSSOM View `getBoundingClientRect` 返回布局盒。回归测试 `test_element_gbcr_real_layout_rect_beats_text_registry_slice14`（part08.rs）双向负控制：旧序 RED `"0,0,30,10"` ≠ `"10,20,100,50"`，新序 GREEN 且 detached 兜底保留。slice14 修复见 PR #56。

## 如何避免

- 给"写路径"加注册副作用时，检查它是否被任何读路径消费、以及消费优先级是否高于权威数据源；副作用表只能兜底、不能压过真实数据。
- 同语义 API（gBCR / getClientRects / offsetWidth）应同源；两 API 分叉是最便宜的报警器，新几何读路径接入时先对表。
- 排查"渲染对、查询错"时，先看错值的数值形状（原点、文本量宽、em 高）反推数据来源，形状匹配比逐层追踪更快。
