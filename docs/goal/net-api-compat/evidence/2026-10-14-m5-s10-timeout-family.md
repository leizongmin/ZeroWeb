# M5-S10 — XHR timeout 族 3 腿销账 + timeout setter 重排（xhr +6P/−4F/−2T）

**日期**: 2026-10-14
**切片**: M5-S10（master.md 下一步计划第 3 项「timeout 族 3 腿」——M4-S21 记账：
timeout-multiple-fetches / thrown-error-in-events / xhr-timeout-longtask）
**原始数据**: 六 corpus 基线/修齐双跑（同树 stash 对照，本文件同日实测）

## 结果（六 corpus 受控 A/B，对照 M5-S9 树）

| corpus | 基线（M5-S9 树） | M5-S10 | Δ |
|---|---|---|---|
| fetch | 1504P/583F/2T | 1504P/583F/2T | 逐字节持平 |
| xhr | 565P/26F/3T | **571P/22F/1T** | **+6P/−4F/−2T** |
| url | 4639P/1426F | 4639P/1426F | 逐字节持平 |
| mimesniff | 1033P/865F | 1033P/865F | 逐字节持平 |
| streams | 905P/108F/5T | 905P/108F/5T | 逐字节持平（5T 先在） |
| eventsource | 35P | 35P | 逐字节持平 |
| websockets | 376P | 376P | 逐字节持平 |

xhr +6 腿逐条（Fail/Timeout → Pass）：

1. `timeout-multiple-fetches.html` :: Redirects should not reset the timer
2. `timeout-multiple-fetches.html` :: CORS preflights should not reset the timer
3. `xhr-timeout-longtask.any.js` :: Long tasks should not trigger load timeout
4. `thrown-error-in-events.html` :: onload 抛 → window.onerror（addEventListener 腿原为
   页级 Timeout 内记账 Fail，随页恢复一并转 Pass）
5. `xmlhttprequest-timeout-overrides.html` :: timeout overrides load after a delay
   （timeout setter 重排附带解锁——原 M4-S20 全绿后漂移回 Fail 的 2 腿之一）
6. `xmlhttprequest-timeout-overridesexpires.html` :: timeout set to expired value
   before load fires（同上）

页级 Timeout 3→1（`xhr/cookies.http.html` 保留——cookies 跨域记账域既有）。残余
22F 集与基线逐条相同（formdata structural 2 + surrogate 1 + 坐标 2 + permissions-policy
2 + template-element 3 + overridemimetype shift-jis 8 + response-body-errors 2 +
event-error 1 + getallresponseheaders 1——既有记账，零新增）。

## 根因与修齐（shim 三处 + runner 两处）

### ① `_zwFetchSimDelay` 共享延迟模型（part01.js）

原 fetch wrapper 只认 `delay.py?ms=N`；XHR send 计时器判定无处复用。提取
`globalThis._zwFetchSimDelay(url)`：`?ms=` 逐字、**bare delay.py 缺省 1000ms**
（上游 wptserve `request.GET.first(b"ms", 1000)`；timeout-multiple-fetches subtest 2
「bare delay.py + timeout 750 → ontimeout」钉死缺省 >750，测试内注释「500+500=1000」
为陈旧注释不采信）、redirect.py 折叠（`delay=N` + location 链到 delay.py 的 ms/缺省
——重定向链为同步 host 契约就地结算（M4-S17），逐跳延迟不可行，「redirects should not
reset the timer」面按总延迟一次计算）。fetch wrapper 与 XHR send 计时器判定共用。

### ② timeout setter 重排（part03.js）

spec timeout setter：send() 进行中改值 → deadline = send 起点 + 新值，剩余 = 新值 −
已流逝；已过 → 立即异步走 request error steps；0 = 禁用 → 解除。原实现为 send 时静态
`setTimeout(T)`，setter 改值不生效（overrides/overridesexpires 2 腿漂移根因）。实施：
`_zwSendEpoch` + `_zwArmTimeout(remaining)`（token 单调递增使旧计时器失活——runner stub
clearTimeout 为 no-op 不依赖移除；fire 闭包 gen/aborted/readyState 守卫兜底迟到旧火）。

### ③ finite 模型静态判定（part03.js）

send 时目标为延迟端点且模拟延迟 D < timeout T → 响应必先至 → 不挂计时器。真实网络
语义中服务端延迟不占主线程预算——xhr-timeout-longtask：busy loop 200ms 把发题推迟到
T=150ms 之后，陈旧计时器伪 ontimeout（ontimeout = assert_unreached → 页级 Timeout）。
D ≥ T 或非延迟端点照常挂计时器（timeout-multiple-fetches 两腿、overrides 族不受影响）。

### ④ on* event handler 抛按 spec report（part05.js R114）

EventTarget.prototype.dispatchEvent 的 on* 属性 handler 调用原 `catch (_e114o) {}` 吞掉
——与上方 addEventListener 路径（R139/R140 经 `_zwReportListenerError` 上报）不同机。
thrown-error-in-events subtest 2（`xhr.onload = () => { throw error; }`）依赖 window
'error' 事件结算 promise_test → 吞掉即永不结算 → 页级 Timeout。补
`_zwReportListenerError(_zwWrapCallError(_e114o, null), null)`（stamp null = 页面同
realm——HTML §report the exception）。

### ⑤ runner：preflight 应答面 + timer 到期序

- **delay.py fixture OPTIONS 分支 + 泛化 OPTIONS 兜底 ACAM/ACAH 回显**：探针页三级
  定位（A: cross-origin YO bare delay.py → `error` 而非 `timeout`；B: 同源 YO /
  C: cross-origin GET 对照均正常）→ 根因 = delay.py fixture 分支（先于泛化兜底返回）
  无 CORS 头，preflight OPTIONS 落入 200 裸应答 → shim ACAM 覆盖判定拒（YO 不在
  safelisted 集）→ fetch reject → onerror。补 OPTIONS preflight 应答（ACAO 回显 Origin
  + ACAM = 固定集 ∪ ACRM 回显 + ACAH = ACRH 回显）——上游 wptserve 自动应答 preflight
  的合规 CORS 服务端语义，M5-S3 eventsource cors.py 同先例；泛化兜底同步补 ACAM/ACAH
  回显（trickle.py/content.py 等无分支端点同类面）。
- **timer 到期序派发**（probe 循环 `due.sort((a,b) => a.at - b.at)`）：HTML event loop
  timers 语义先到期先跑；push 序在「send() 先挂长 timeout 计时器、后挂 0ms 派发/其回调
  再挂 fetch-issue」形态下到期集乱序 → 伪 ontimeout。③ 为主修，此为通用事件循环语义
  兜底（per-task 模式首派发也取最早到期者）。

**探针页已清理**（zz-probe-s10.html 三腿定位后删除，零残留）。

## 碰头核对（run-rules §9）

共享面 last-touch 全为本流自身（part01/03/05、testharness.rs = 625dc53dc M5-S9），
无并行编辑。域外挡门记录（渲染流域，不硬解）：T9 merge（fb3873802）带回
`apps/renderer/src/runtime.rs` fmt 不洁 2 处 + clippy dead-code 1（`build_implicit_
submit_url`/`submit_form_on_enter_at` never used）；R5108（4fc38b7d2）带入
`crates/layout-engine/src/engine/postprocess.rs` fmt 不洁 1 处——本流 fmt/clippy 门禁
以本流文件（zero-wpt-runner 等）为准，渲染流需自清。

## 门禁

- `make test` 全绿 **20664P/0F**（M5-S9 20661 → +3 = 随 pull 并入的 R5108
  r5108_max_height_margin_box_tests 3 测试，本流 ±0）。
- `cargo clippy -p zero-wpt-runner --all-targets -- -D warnings` 干净（workspace 全量被
  上记渲染流域外错误挡门——本流 crate 零涉）；`cargo fmt` 本流文件干净。
- 六 corpus 受控 A/B 同树 stash 对照双跑（上表）。

## 残余（本切片后 xhr 域）

- 账面失败集与基线逐条相同：XML 文档解析 3（template-element——engine XML DOM 深度）、
  formdata 5F（M5-S8 精确记账：iframe 编排 structural 2 + surrogate wire 1 + 坐标跨域
  2）、permissions-policy 2（跨域）、overridemimetype shift-jis 8（encoding 域）、
  response-body-errors 2、event-error 1、getallresponseheaders 1、cookies 页级 Timeout
  1（跨域）。
- fetch T=2（request-upload.h2 HTTP/2 变体 + canvas-remote-read-redirect）与 streams
  T=5 为先在（基线同值），非本切片引入。
