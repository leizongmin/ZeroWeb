# R11 — element.localName getter + HTML 元素子类 instanceof（M4 / DC-3）

**日期**: 2026-08-14
**轮次**: R11
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**commit**: 见 `git log`（feat(js-dom): element.localName getter + HTML element subclass instanceof）

## 背景

R10 解了 instanceof Element/HTMLElement/Node（cloneNode 0P→51P）。R11 推进两个剩余聚类：
- **localName getter 缺失**（createElement 用例核心断言 `elt.localName` 返 undefined）
- **具体 HTML 元素子类 instanceof**（cloneNode 用例 `create_element_and_check("div","HTMLDivElement")` 的 `el instanceof HTMLDivElement`，~64 个 HTML 元素接口）

## 改动

### 1. element.localName getter（part04 get trap）

`element.localName`（spec `dom-element-localname`）：HTML 元素 = tagName 小写；带 prefix 限定名（`svg:rect`，createElementNS）去 prefix 取冒号后；非 Element（text/comment/PI/fragment）→ null。

### 2. HTML 元素子类构造器 + tag 映射（part03）

- 注册 ~64 个 `HTML*Element` 构造器（spec HTML 元素接口表），prototype → HTMLElement.prototype（HTMLElement 自身已建）。
- `globalThis.__zwHtmlTagIface`：tag→接口名映射表（div→HTMLDivElement, a→HTMLAnchorElement, ... 未知/自定义→HTMLUnknownElement），覆盖 spec HTML 元素接口全表。

### 3. getPrototypeOf 按 tag 返子类 prototype（part05）

element 分支：按 `_realTag` 小写查 `__zwHtmlTagIface`，返对应 `HTML*Element.prototype`（链 HTMLElement→Element→Node）；无映射/构造器缺失回落 HTMLElement.prototype。

### 4. 单测

- `test_element_local_name_r11`：createElement 大写/小写、querySelector、createElementNS 带 prefix、Text/Comment → null。
- `test_html_element_subclass_instanceof_r11`：div→HTMLDivElement（+ HTMLElement/Element/Node 链）、8 tag 对应接口、跨接口不误伤。

## 基线结果（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R10 | R11 | Δ |
|------|----|----|---|
| polyfill | 39.23% | **40.89%** | +1.66pp |
| native | 38.96% | **40.63%** | +1.67pp |

双路径对等差 0.26pp。**cloneNode 用例**：polyfill 51P → **121P**（+70），native 51P → 119P（+68）。polyfill 净 +75 pass。

完整 JSON 快照：`2026-08-14-r11-dom-nodes-polyfill.json` / `2026-08-14-r11-dom-nodes-native.json`。

## 验证

| 门禁 | 结果 |
|------|------|
| engine v8 单测 | ✅ 2082 passed（+2 新测试，0 failed） |
| engine quickjs 单测 | ✅ 1408 passed |
| fmt / clippy（v8 + quickjs 双矩阵） | ✅ 零警告 |

## 已知边缘（非本切片）

- `createElement('canvas') instanceof HTMLCanvasElement` 仍 false：canvas 经 `_zwMakeCanvas()` 特殊 proxy（canvas 流专用路径），不走 `_makeProxy`/getPrototypeOf。记入未解决问题，canvas proxy 的 instanceof 待 canvas 流或专项。

## 下一步

- createElement 用例剩 XML/XHTML iframe contentDocument（`Cannot read documentElement`，~大头，html-compat 域）。
- cloneNode 剩 14F（canvas proxy + 其他边缘）。
- iframe.contentDocument（createElementNS/case 大头 ~390）。
