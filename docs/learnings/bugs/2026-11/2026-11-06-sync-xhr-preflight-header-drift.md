---
date: 2026-11-06
modules: zero-engine（js_dom_shim part01/part03）
---

# XHR 同步路径 preflight 头集与响应过滤的异步镜像漂移

## 问题

`access-control-preflight-headers-sync.htm`（上游断言：preflight OPTIONS
不得携带自定义头 x-test，携则端点 400）恒 Fail：同步 XHR 跨源自定义头
请求全部网络错误。同族的 async 版本早已通过。

## 根因

引擎 XHR 是**同步/异步两套独立实现**（part03.js sync `__zw_fetch` 契约
循环 vs part01.js `_zwFetch` Promise 管线）。异步侧跨版本迭代出的语义
不会自动落到同步侧，形成三处漂移：

1. **preflight 头集**：异步侧 M4-S26 已改最小集（Origin + ACRM +
   `Accept: */*` + UA + ACRH）；同步侧仍是初版全转发 `syncHeaders`——
   OPTIONS 携带原请求自定义头，被端点探测臂 400 拒绝。
2. **ACRH 形态**：同步 `join(', ')`（带空格）vs 异步 `join(',')`——
   `access-control-preflight-request-header-sorted.py` 类精确比对端点
   会因空格拒绝。
3. **终响应 CORS 门**：异步终局走 `_zwFetchApplyFilteredResponse`
   （cors check + cors-filtered 头过滤）；同步终响应直填——跨源无 ACAO
   响应可读、非 safelisted 头（x-test）暴露，basic-denied 族凑成 vacuous
   F 与 safelisted-response 过滤断言失败。

同构漂移在历史上反复出现（M4-S18 sync preflight 引入时即全转发、
M5-S11 sync bad-chunk 才补同步抛错），属结构性缺口而非单点笔误。

## 解决

- E2：sync preflight 头集改最小集，注释标明「镜像异步 M4-S26」；ACRH
  join 本次不改（无案面驱动，避免既有 sync 案外溢）。
- E6：sync 无重定向终响应接 `_zwFetchApplyFilteredResponse`（函数内部
  自判同源直通），TypeError → 同步抛 NetworkError。重定向终跳留 M3。
- E1 配套：`_makeResponse` 错误 wire 挂 `__zwFetchErrorTag`，同步侧按
  tag 窄化抛 NetworkError（其余错误维持 error 事件语义，防止外溢）。

## 避免方式

改异步 fetch 语义（preflight 形态、cors check、头过滤、错误处置）时，
把「sync 镜像面是否同步改」列入同一提交的核对清单：grep `part03.js`
的 sync 循环段，确认四处锚点（preflight 头集、ACRH 形态、终响应
过滤、错误处置）与异步侧一致；不一致且有案面驱动则同批修，无案面
驱动则显式注释挂账，不留隐性漂移。fixture 端点带探测臂
（如 no-custom-header-on-preflight 的「OPTIONS 携自定义头即 400」）是
捕获此类漂移的试金石——扩表时优先引入。
