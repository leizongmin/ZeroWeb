# M2-S9 — abort body-abort 族收口（体消费中止 + 流 error + 请求体扰动）（2026-10-29）

**切片**: goal v2.11 小片清单——「abort body-abort 族 ~30F」（承 M2-S6 记账
1/3、M2-S8 记账 1）。**abort/general + abort/request 双文件 27F 全收口**
（general 61/61、request 18/18）。

## 27F 归因分解（终态）

| 组 | 腿数 | 根因 |
|---|---|---|
| response.\*() rejects (with abort reason) if already aborted | 12+1 | 体消费与 signal abort 无关联——abort 后消费照常 fulfill |
| Fetch aborted & connection closed when aborted after calling response.\*() | 7 | 消费 promise 不被 abort 打断（体全量在库、消费即兑现） |
| Stream errors once aborted (with abort reason / after reading) | 3 | 体流为建即闭合的字节背板流，abort 无 error 入口 |
| Calling \*() on an aborted consumed nonempty request（request.any.js） | 5 | fetch(request) 传输体不标 input bodyUsed——再消费不拒 TypeError |

## 实现面

### 1. 消费门 + abort 体错误（`part01.js` 引擎侧，data.json 等原生 fetch 响应）

- `_zwAbortResponseBody(target, reason)`：`_zwAbortErr` 存 reason + 活动体流
  controller error（`_zwBodyStream || _bs` 的 `_zwController`）。幂等；流非
  readable 时 part02 errorStream 内部短路。
- final settle 接线：`signal` 非 null → 注册 abort listener →
  `_zwAbortResponseBody(_finResp, signal.reason)`（settle 后 abort 错误体而非
  fetch promise——spec abort-a-fetch 的 response 已交付分支）。
- `_zwConsumeBodyBytes` 顶部同步门：`_zwAbortErr != null` → 拒 reason
  （**先于 unusable 判定**——「Call text() twice」两度消费同拒 abort reason，
  落 TypeError 会失败）。
- `_zwConsumeBodyChecked(target, f)`：text/json/blob/arrayBuffer/formData 五法
  改经此入口——abort 时**直返已拒 promise**。过程轮教训：五法原在
  `_zwConsumeBodyBytes` 结果上挂 `.then` 派生，已拒 P1 的派生 promise 在调用方
  `.catch` 挂接时尚 pending，reject reaction 晚一 microtask，
  `[reject, next-microtask]` 排序断言必败（bytes() 直返无此跳故单腿先绿）。
  https://fetch.spec.whatwg.org/#abort-a-fetch

### 2. 流 controller 流外可达（`part02.js`）

ReadableStream 构造器 `Object.defineProperty(this, '_zwController',
{ value: controller })`（non-enumerable，自有属性枚举面零扰动）——宿主编排
（引擎 abort 接线、runner 体中止仿真）从流外 error 的唯一入口。

### 3. fetch(request) 传输即扰动（`part01.js`）

`_zwFetchMainImpl` 体提取处：Request-like 载体 + `init.body == null` + 体非空
（string.length/Blob.size/Uint8Array.length > 0，其余类型非 null 即非空）→
`input._bodyUsed = true`。「aborted consumed empty request」fulfillment 腿
（空体传输不扰动）由空体豁免保住；init.body 显式覆盖不动 input 体。

### 4. runner fixture 体改开放流（`testharness.rs` CACHE_ABORT_FETCH_FIXTURE）

infinite-slow-response 端点响应体从全量字节改**开放 ReadableStream**（入队
一 chunk 不 close——上游体无限缓慢、连接持续开放；首 chunk 立即可读、次 read
挂起，消费不悬挂）。abort listener 三翼：stateKey 落 'closed'（连接关闭观测，
M2-S6 既有）+ `bodyCtl.error(signal.reason)`（挂起读/closed 拒 abort reason，
「Stream errors once aborted」面）+ `done._zwAbortErr = signal.reason`（消费门）。
该端点响应绕过引擎 fetch（wrapper 层拦截直返），引擎接线不覆盖——体中止语义
由 wrapper 侧同型实现。cache-abort 页共用 fixture 但无体消费腿（grep 核对，
悬挂风险零）。

## 过程轮（未落提交的中间态）

r1（门内收 + 原方法结构）61 腿中 56P/5F：五法排序断言败于 `.then` 派生跳
（机制见 1.4）。r2 加 `_zwConsumeBodyChecked` 直返已拒 promise 后全收口。

## 验证

- probe：abort/general 39P/22F → **61P/0F 收口**；abort/request
  13P/5F → **18P/0F 收口**。
- 全通道 A/B（`netapi-m2s9-r1.json` vs **直接前轮** `netapi-m2s8-r1.json`，
  逐 subtest）：**+27 增益 / 0 回归 / MISSING 0**（10897 → 10924 pass）；
  per-file Δ 仅 abort/general（+22）与 abort/request（+5），其余全通道逐文件
  零变化（integrity.sub / request-structure / response-consume /
  cache-abort 关联腿零扰动）；stderr **0 panic**，构建 0 warning
  （`m2s9-build-r2.log`）。

## 记账（未清偿）

1. **abort 目录外 signal 关联体中止**（response.any.js 等直接对 Response 构造
   的 signal 关联场景）语料无钉未入门；`_zwAbortResponseBody`/
   `_zwController` 原语已备，随语料扩批再评估。
2. Request 侧 abort → 体流 error（spec abort-a-fetch 同时错误 request 体）：
   request.any.js 现钉仅 TypeError 面（消费扰动门已覆盖）；体流 error 观测腿
   语料无钉未入门。
