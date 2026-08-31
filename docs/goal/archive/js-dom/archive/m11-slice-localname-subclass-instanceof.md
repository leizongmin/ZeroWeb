# M4 R11 切片 — element.localName getter + HTML 元素子类 instanceof

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**前置**: R10（polyfill Proxy getPrototypeOf 解 instanceof）
**commit**: 见 `git log`（feat(js-dom): element.localName getter + HTML element subclass instanceof）

## 背景

R10 解了 instanceof Element/HTMLElement/Node（cloneNode 0P→51P）。剩余：localName getter 缺失（createElement 用例）+ 具体 HTML 元素子类 instanceof（cloneNode 用例 create_element_and_check 对 ~64 个 tag 查对应 HTML*Element 接口）。

## 改动（3 文件）

### 1. element.localName getter（part04 get trap）

`localName`（spec `dom-element-localname`）：HTML 元素 = tagName 小写；带 prefix（`svg:rect`）去 prefix 取冒号后；非 Element → null。

### 2. HTML 元素子类构造器 + tag 映射（part03）

- ~64 个 `HTML*Element` 构造器（spec HTML 元素接口表），prototype → HTMLElement.prototype。
- `__zwHtmlTagIface`：tag→接口名映射（div→HTMLDivElement 等，覆盖 spec 全表，未知/自定义→HTMLUnknownElement）。

### 3. getPrototypeOf 按 tag 返子类 prototype（part05）

element 分支：按 `_realTag` 小写查 `__zwHtmlTagIface`，返对应 `HTML*Element.prototype`（链 HTMLElement→Element→Node）；无映射回落 HTMLElement.prototype。

### 4. 单测（part07）

`test_element_local_name_r11` + `test_html_element_subclass_instanceof_r11`。

## 基线（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R10 | R11 | Δ |
|------|----|----|---|
| polyfill | 39.23% | 40.89% | +1.66pp |
| native | 38.96% | 40.63% | +1.67pp |

cloneNode 用例 polyfill 51P→121P（+70）。双路径对等差 0.26pp。

## 验证

engine v8 2082 / quickjs 1408 单测；fmt + clippy（v8 + quickjs）零警告。

## 已知边缘

`createElement('canvas') instanceof HTMLCanvasElement` 仍 false——canvas 经 `_zwMakeCanvas()` 特殊 proxy（canvas 流专用路径），不走 getPrototypeOf。记未解决问题，canvas proxy instanceof 待 canvas 流或专项。

## 下一步

- createElement/cloneNode 剩余：XML/XHTML iframe contentDocument（~大头，html-compat 域）。
- iframe.contentDocument（createElementNS/case ~390）。
