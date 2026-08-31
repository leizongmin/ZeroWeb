# R10 — polyfill Proxy getPrototypeOf 解 instanceof + CSS 回归修复（M4 / DC-3）

**日期**: 2026-08-14
**轮次**: R10
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**commit**: 见 `git log`（feat(js-dom): polyfill Proxy getPrototypeOf for instanceof + CSS.escape merge fix）

## 背景

R9 记录的 instanceof 89 块：polyfill Proxy 节点默认走 target({})原型（Object.prototype），`el instanceof Element/HTMLElement/Node` 恒 false。集中在 Node-cloneNode（53 instanceof）+ Document-createElement（36 instanceof Element）。

## 改动

### 1. polyfill Proxy getPrototypeOf trap（核心）

`_makeProxy`（part03）的 Proxy handler 加 `getPrototypeOf` trap（part05 handler 闭合处）：
- element 节点 → HTMLElement.prototype（链 Element → Node，覆盖绝大多数 instanceof Element/HTMLElement/Node）
- PI → ProcessingInstruction.prototype / fragment → DocumentFragment.prototype / text/comment → Node.prototype
- 构造器缺失回落 Object.prototype

**安全**：getPrototypeOf 仅影响 `instanceof` / `Object.getPrototypeOf` / 原型链属性查找 fallback，**不影响 get/set**（属性读写仍走 get/set trap）。

### 2. DOM 原型方法不可枚举（修 getPrototypeOf 副作用）

getPrototypeOf 让原型链含 HTMLElement/Element.prototype，其上 cloneNode/addEventListener/removeEventListener（part03 直接赋值，可枚举）会污染 `for...in`（expando 枚举回归，test_expando_enumeration_r3046）。

修复：3 个方法改 `Object.defineProperty(..., {enumerable: false})`（spec WebIDL 操作默认 enumerable:false）。

### 3. CSS.escape/supports 合并修复（并行 canvas 流回归）

**归因**：canvas 流 R34xx 在 part05:920 `if (!globalThis.CSS) globalThis.CSS = {}` + `CSS.percent/deg`（CSS Typed OM）。part05 在 part06 **之前**拼接执行 → part05 先建 CSS = {percent, deg} → part06:77 `globalThis.CSS = globalThis.CSS || {escape, supports}` 短路（CSS 已存在）→ **escape/supports 永不挂载**（CSS.escape is not a function，2 个 css 测试回归）。

修复：part06 CSS 改合并模式（`if(!CSS.escape) CSS.escape = ...`），保留 part05 的 percent/deg。这是 canvas 流 part05 引入的共享面（js_dom_shim）回归，本流 part06 修复。

### 4. testharness.rs fetch_handler 编译错误修复（并行流遗留）

canvas 流在 testharness.rs:350 加 `fetch_handler: wpt_root.and_then(wpt_data_fetch_handler)` —— `wpt_root` 是 `&Path`（非 Option）无 `and_then`，release 编译失败（阻塞 make test/reftest）。改 `wpt_data_fetch_handler(wpt_root)`（镜像 image_source_fetcher 模式）。

### 5. instanceof 单测

`test_instanceof_prototype_chain_r10`：createElement/querySelector instanceof Element/HTMLElement/Node、PI instanceof ProcessingInstruction/Node、不误伤（element instanceof DocumentFragment = false）、getPrototypeOf 不破坏 get trap。

## 基线结果（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R9 | R10 | Δ |
|------|----|----|---|
| polyfill | 38.03% | **39.23%** | +1.20pp |
| native | 37.81% | **38.96%** | +1.15pp |

双路径对等差 0.27pp。**cloneNode 用例**：polyfill 0P → **51P**（+51），native 0P → 49P（instanceof 修复直接解锁）。polyfill 净 +54 pass。

完整 JSON 快照：`2026-08-14-r10-dom-nodes-polyfill.json` / `2026-08-14-r10-dom-nodes-native.json`。

## 验证

| 门禁 | 结果 |
|------|------|
| engine v8 单测 | ✅ 2071 passed（6 fetch 既存失败，非本切片引入——clean R9 tree 同样 6 失败） |
| engine quickjs 单测 | ✅ 1407 passed |
| fmt / clippy（v8 + quickjs 双矩阵） | ✅ 零警告 |

## 未解决问题（遗留，记入 master.md）

- **6 个 fetch/response 测试既存失败**（clean R9 tree 同样存在，并行流引入）：`instanceof Response` = false（fetch 结果非 Response 实例，`_makeResponseFromWire` 路由问题）+ fetch abort/binary/stream/signal/forbidden-headers。归因 fetch/net 域，非 js-dom DOM 桥工作面，记入未解决问题不硬解（run-rules §9 工作面）。

## 下一步

- instanceof 89 块核心已解（cloneNode 51）。剩余：具体元素子类 instanceof（HTMLDivElement 等，cloneNode 用例 39 行）需注册子类构造器 + getPrototypeOf 按 tag 返子类 prototype。
- createElement localName getter（返 undefined，createElement 用例主因之一）。
- iframe.contentDocument（createElementNS/case 大头 ~390）。
