# M2-S6 — abort/general 引擎入口同步取消 + stash 仿真注入（2026-10-29）

**切片**: goal v2.8 小片清单——「abort/general 引擎入口同步 cancel（4F）」
（承 M2-S3 记账 3 / S3b 记账 3）。**二期 M 收口后重开小片**。

## 4F 归因分解（本轮实测修正）

| 腿 | 根因面 | 机制 |
|---|---|---|
| Readable stream synchronously cancels with AbortError if aborted before reading | 引擎 | 入口 abort 无流体同步 cancel |
| Already aborted signal rejects immediately | 引擎 | rejection 晚一格 microtask |
| Request is still 'used' if signal is aborted before fetching | 引擎 | 入口 abort 不置 bodyUsed |
| Already aborted signal does not make request | **runner fixture** | stash-take.py 无路由 → 404 空体 → r.json() SyntaxError 掩盖断言（语义本身已满足——入口 reject 不发请求） |

连带翻绿 2 腿（fixture 连带）：Underlying connection is closed when aborting
after receiving response（http + no-cors）。

## 根因与实现

### 1. abort-a-fetch 同步取消面（`part01.js` `_zwFetchMainImpl` 入口）

fetch spec abort-a-fetch：signal 已 aborted 时 reject reason（默认 AbortError
DOMException）并同步处置 request body。入口分支补齐两翼：

- ① `init.body` 为 ReadableStream → `cancel(signal.reason)` 于 fetch() 返回前
  执行（"cancel called sync" 腿：cancel 回调捕获的 reason 与 fetch rejection
  同一实例）；
- ② input 为 Request-like 且有体 → `_bodyUsed = true`（extract+error 语义，
  与 scheme 分派提取面同型；`init.body == null` 守卫不覆盖显式体覆盖场景）。
  https://fetch.spec.whatwg.org/#abort-fetch

### 2. fetch settle 时序去包装（`globalThis.fetch` RT 发射侧信道化）

Resource Timing 发射此前以 `p.then(emit, emit)` 派生 promise 返回——一切
fetch settle 晚一个 microtask，「Already aborted signal rejects immediately」
期望已 aborted 入口的 rejection 先于调用方排队的下一 microtask（reject 排序
`['fetch-reject', 'next-microtask']`）。改为侧信道挂接（`p.then(...)` 不改返
回值）+ 直接返回 `p`——settle 时序回到引擎自然节奏；RT endMs ≈ settle 时刻
容差不变。

### 3. stash 仿真 fixture 注入 abort/general（`testharness.rs`）

`CACHE_ABORT_FETCH_FIXTURE`（in-page wptserve stash/infinite-slow-response
端点仿真，原仅注入 cache-abort 页）扩注入 `abort/general`：
「Already aborted signal does not make request」经 stash-take.py 观测请求
未发出——此前该端点落静态读库 404 空体，`r.json()` 抛 SyntaxError 掩盖
断言面（入口 reject 语义本已满足）。连带：

- **wrapper 路由序修正**：入口 aborted-reject 收敛到 wrapper 自持端点
  （infinite-slow-response）；generic URL 一律交还引擎（新入口同步取消面
  不被 wrapper 遮蔽——cache-abort 页既有 2 腿零回退，引擎 reject reason
  与 wrapper 一致）。
- **infinite-slow-response 立即返响应**（上游头快到、体慢流；runner 同步
  契约下防 await 悬挂到 harness 超时）+ abort 视为连接关闭落
  `stateKey='closed'`（轮询观测面）——「Underlying connection is closed…」
  http/no-cors 两腿连带翻绿。

## 验证

- probe（终版侧信道重建后，`testharness-net-api abort/general`）：
  general.any.js 29P/32F → **35P/26F**——本片 4 腿 + 连带 2 腿全翻绿
  （rejects immediately / still 'used' / does not make request /
  synchronously cancels / connection-closed http+no-cors）。
- 全通道 A/B（`netapi-m2s6-r1.json` vs **直接前轮**
  `netapi-m2s5-r1.json`，逐 subtest）：**+6 增益 / 1 Pass→Fail（显影，
  见记账 4）/ MISSING 0**（10882 → 10887 pass）；stderr **0 panic**，
  构建 0 warning（`build-m2s6-r1.log`）。
- RT 侧信道单变量 bisect：临时还原旧包装 probe（`/tmp/bisect-build-rt.log`）
  → SRI 腿 Pass 复现——RT 包装为唯一变量，入口同步取消与 fixture 注入排除；
  还原终版重建后 response-null-body 逐腿逐字对齐 `netapi-m2s6-r1.json`
  （9P + HEAD 既存 F + SRI 显影 F）→ 全通道 A/B 数据即终版验证，无需重跑。

### SRI 腿显影机制（记账 4 机械证据）

该腿 `promise_rejects_js(t, TypeError, promise)` 返回值**未被测试体 await**
（response-null-body.any.js:35-38）——断言在 test body 返回后异步触发。
引擎无 SRI 实现（M2-S3c 记账 1 定性，integrity 元数据被忽略）→ fetch 正常
fulfill(204) → `unreached_func` 必然触发（testharness.js:809，消息中
undefined 为缺省 description）。三轮假绿历史：

1. **m2s3b 基线**：status.py 未路由 → 404 → p reject(TypeError) →
   `assert_throws_js_impl(TypeError)` 误中——S3c 记账 1 同款 Failed-to-fetch
   假绿。
2. **m2s3c/m2s5**：status.py 解锁 → p fulfill；旧 RT 包装
   `p.then(emit, emit)` 派生 promise 使 unreached 触发晚于 test `done()` →
   `Test.step` phase 守卫（testharness.js:2838
   `if (this.phase > this.phases.STARTED) return;`）**静默丢弃迟到断言**。
3. **本片侧信道**：去包装 → unreached 提前到 done 之前 → 真缺口显影
   （观测消息逐字吻合）。

排除法封闭：m2s5 Pass + p fulfill（m2s6 实证）⇒ unreached 必触发 ⇒ 唯一
无副作用出口是 phase 丢弃。归类**语义显影**（非行为回归），与 S3c 记账 1
同根因（SRI 未实现）、不同假绿机制。

## 记账（未清偿）

1. **response body 中止族 ~15F + request.any.js 5F**（`response.text()`
   rejects if already aborted / Call text() twice / request consumed 族）：
   响应/请求体消费中 signal abort → body stream error(AbortError)——引擎
   body-abort 机制，入口面之后的第二块。
2. **signal 身份族 4F**（Signal state is cloned / Signal on request object /
   should also have abort reason / Signal removed by setting to null）：
   Request 构造器 signal [SameObject] 克隆语义 + init.signal null 覆盖面。
3. **Fetch aborted & connection closed after body 调用 7F + Stream errors
   once aborted 3F**：引擎 body-abort（记账 1）+ 体读中连接关闭状态机的
   组合面，随记账 1 重排。
4. **SRI 腿显影（本片 A/B 唯一 Pass→Fail）**：response-null-body「Null
   body status with subresource integrity should abort」——与 M2-S3c 记账 1
   （「CORS invalid integrity」）同根因（引擎无 SRI/integrity 校验），
   假绿机制为旧 RT 包装 +1 microtask 使迟到断言被 testharness phase 守卫
   丢弃（机制见验证节）。SRI 切片（goal 小片清单第 2 位）入列时两腿同批
   收敛。
