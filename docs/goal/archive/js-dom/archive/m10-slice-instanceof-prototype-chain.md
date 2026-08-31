# M4 R10 切片 — polyfill Proxy getPrototypeOf 解 instanceof + CSS 回归修复

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**前置**: R9（polyfill createProcessingInstruction + DOMException identity）
**commit**: 见 `git log`（feat(js-dom): polyfill Proxy getPrototypeOf for instanceof + CSS.escape merge fix）

## 背景

R9 记录 instanceof 89 块：polyfill Proxy 节点 instanceof Element/HTMLElement/Node 恒 false（Proxy 无 getPrototypeOf trap，走 target({})=Object.prototype）。集中在 Node-cloneNode（53）+ Document-createElement（36 instanceof Element）。

## 改动（5 文件）

### 1. getPrototypeOf trap（part05 `_makeProxy` handler 闭合处）

handler 原 4 trap（get/has/ownKeys/getOwnPropertyDescriptor）加第 5 个 getPrototypeOf：element→HTMLElement.prototype（链 Element→Node）、PI→ProcessingInstruction、fragment→DocumentFragment、text/comment→Node。仅影响 instanceof/getPrototypeOf/原型链查找，不影响 get/set。

### 2. DOM 原型方法不可枚举（part03）

getPrototypeOf 副作用：原型链含 HTMLElement/Element.prototype，其上 cloneNode/addEventListener/removeEventListener（直接赋值，可枚举）污染 for...in（expando 枚举回归）。改 `Object.defineProperty(enumerable:false)`（spec WebIDL 默认）。

### 3. CSS.escape/supports 合并修复（part06，并行 canvas 流回归）

canvas 流 R34xx part05:920 先建 CSS={percent,deg}（part05 在 part06 前执行）→ part06:77 `CSS || {escape,supports}` 短路 → escape/supports 丢失（CSS.escape is not a function，2 测试回归）。改 part06 合并模式（`if(!CSS.escape)`），保留 percent/deg。

### 4. testharness.rs fetch_handler 编译错误（并行流遗留）

canvas 流 `fetch_handler: wpt_root.and_then(...)` —— `&Path` 无 and_then，release 编译失败阻塞 make test。改 `wpt_data_fetch_handler(wpt_root)`。

### 5. instanceof 单测（part07）

`test_instanceof_prototype_chain_r10`。

## 基线（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R9 | R10 | Δ |
|------|----|----|---|
| polyfill | 38.03% | 39.23% | +1.20pp |
| native | 37.81% | 38.96% | +1.15pp |

cloneNode 用例 polyfill 0P→51P（+51）。双路径对等差 0.27pp。

## 验证

engine v8 2071 / quickjs 1407 单测；fmt + clippy（v8 + quickjs）零警告。

## 归因：6 个 fetch/response 既存失败（非本切片引入）

clean R9 tree 同样 6 失败（test_fetch_abort_signal / fetch_forbidden_headers / fetch_response_binary_body / request_signal_passthrough / response_body_readable_stream / response_request_constructors）。根因 `instanceof Response` = false（fetch 结果非 Response 实例）+ fetch abort/stream 等。归因 fetch/net 域（并行流引入），非 js-dom DOM 桥工作面。本切片修复了同源的 CSS 回归（2 个，part06 工作面内），fetch 6 个记入未解决问题（run-rules §9 工作面不重叠，不硬解跨流）。

## 下一步

- 具体元素子类 instanceof（HTMLDivElement 等，cloneNode 39 行 `orig instanceof type`）需注册子类构造器 + getPrototypeOf 按 tag 返子类 prototype。
- createElement localName getter。
- iframe.contentDocument（createElementNS/case 大头）。
