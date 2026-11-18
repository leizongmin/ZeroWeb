# M2 s434 — 桥套件 7 红全树 bisect 归因 + 清偿（429 断点接手轮）

日期：2026-11-14。前情：s433 收口时发现桥套件（cargo test -p zero-engine --lib
js_dom_bridge）12 红，其中 5 例经他流 94883fbbe 清偿（复跑 1561P/7F），余 7 例
「签名共性=异步事件派发缺」引入点未考。本切片全树 bisect 定谳 7 例引入点、
逐案勘根后清偿全部 7 红。

## 判据与 bisect 链

判据二值化脚本：`bad iff (判据测试 FAILED)`，测试执行经 test-guard 包裹
（`--compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900`），
脚本 compile 失败 exit 125（skip）。五轮串行归因（每轮 good = 上轮 first bad）：

| 轮 | 判据测试 | good 锚 | first bad | 引入提交 |
|---|---|---|---|---|
| 1 | 全 7 例 | 2e260d532（s180 XHR，2026-10-31；锚跑 1370P/1F 唯一红非判据集） | ba03fd7ff | net-api M3 ws interfaces（part06.js 211 行） |
| 2 | 除 ws×2 余 5 例 | ba03fd7ff | f021e11fb | fetch M3 redirect 七簇（part01.js+testharness.rs） |
| 3 | 除 fetch 余 4 例 | f021e11fb | 4615680c0 | xhr M3 auth 五簇（part02/03.js） |
| 4 | 除 sw-iframe 余 3 例 | 4615680c0 | 015172475 | SMIL 文档级时间线 M1（part09.js +630） |
| 5 | 仅 stream | 015172475 | c6dcbce0a | http-cache M2 wire 第 12 参 |

轮 1 锚验证：锚提交整体 101（旧红存在）但判据 7 例全绿——判据二值化不受
非判据集红影响。bisect 工件 /tmp/s434/bisect{,2,3,4,5}-main.log +
bisect-log.txt（git bisect log 全量）。

## 逐案勘根（四案同型：引擎语料定谳 spec 正解、桥测试前提过时；一案引擎真 bug）

### ① ws×2（ba03fd7ff）——part45.rs 测试前提更新

- `test_t8k_websocket_error_nonterminal`：旧契约 err wire 立即派 error
  （readyState=1）。ba03fd7ff 契约翻转：err 只置 `_zwErrSeen` 标记，error 由
  close wire 臂补派且恰一次、readyState 先 CLOSED（upstream events/015 断言
  onerror 时已 CLOSED；close-connecting-async「error 恰一次」面）。旧测试
  emit err→msg 无 close，违反宿主契约「err 后必随终结 close wire」→ error
  永不派发（left "msg:after-error" vs right "error:1|msg:after-error"）。
  修：emit err→msg 断言零派发 + readyState=1 + 泵继续；再 emit close(1006,
  wasClean=false) 断言 error:3 → close:false:1006 序。
  spec：https://html.spec.whatwg.org/#fail-the-websocket-connection
- `test_t8k_websocket_event_flow_open_message_close`（364 行）：close 后 send
  断言 InvalidStateError = 旧版 spec「非 OPEN 即抛」；现行 spec send step 2
  （CLOSING/CLOSED abort these steps）→ 静默返回 undefined（send/007/008
  断言面）。修：断言静默返回 undefined + 宿主 send 命令零新增。
  spec：https://html.spec.whatwg.org/multipage/web-sockets.html#dom-websocket-send

### ② sw-iframe（4615680c0）——part23.rs mock 挑战流化

`test_iframe_service_worker_messages_dispatch_to_iframe_container` 内嵌
iframe 凭证 XHR 断言（URL userinfo → `authorization\x1eBasic YWE6YmI=` wire
头）编码 s180 旧契约「首跳即转 Basic」。4615680c0 改**延迟注入**：首跳无
Authorization、凭据暂存，401 Basic 挑战同源透明重试才携 Basic（upstream
authentication.py 挑战流要求首跳无凭据可发挑战；M3 auth 账本 2(c)）。
恒 200 的旧 mock 下凭据永不上线 → 断言红。修：mock 改两腿（首跳 401 +
`WWW-Authenticate: Basic realm="secure"` → 重试 200 body），重试腿验证
Basic wire 头；401 不透传 JS 断言（readyState DONE + responseText body）
保持。

### ③ fetch_filtered（f021e11fb）——part10.rs mock 补 ACAO

`test_fetch_filtered_response_type_matrix`：Promise.all 4 路全灭
（__filteredTypes undefined）。f021e11fb 步序修正：**cors check 先于
manual→opaqueredirect 转换**（fetch spec HTTP-network-or-cache fetch——无
ACAO 的跨源重定向响应须 network error 而非 opaqueredirect；upstream
redirect-mode 面要求，M3 redirect 账本 2(b)(c)）。mock 302 无 ACAO + 默认
mode=cors → cors check 失败 → reject → Promise.all 全灭。修：mock 302 补
`Access-Control-Allow-Origin: *` 过 cors check 后落 opaqueredirect（保持
测试意图——filtered response 全类型矩阵）。期望串零变化
（opaqueredirect 过滤面 headers/体不可见语义断言保持）。
spec：https://fetch.spec.whatwg.org/#http-network-or-cache-fetch

### ④ stream（c6dcbce0a）——part10.rs 前提更新为 reject TypeError

`test_response_body_readable_stream_r2967` 网络错误腿：旧实现 error wire →
resolve ok:false status-0 伪响应（body=null）。c6dcbce0a 起硬 reject
TypeError（fetch spec network error——promise 恒不 resolve status-0 伪对象；
request-cache 语料 response:"error" 臂 assert_throws_js(TypeError) 判据面）。
旧测试 `.then(onFulfilled)` 无 rejection 臂 → __respErr undefined →
`undefined.body` TypeError。修：断言 reject TypeError（双臂 + instanceof）。
spec：https://fetch.spec.whatwg.org/#concept-network-error

### ⑤ img×2（015172475）——引擎真 bug：SMIL 泵 microtask 自链饿死 _deferBudget

`img_runtime_deferred_settle_r5275`（(c2) load 后续任务派发 0≠1）+
`svg_img_settle_density_r5277`（load 计数 0≠1）。根因链：

1. SMIL 事件泵 `_scheduleTick`（part09b）setTimeout ≤50ms 轮询自 re-arm；
2. 裸 sandbox/桥测试环境无宿主定时器 → shim setTimeout 回落 = `_defer`
   = queueMicrotask → **轮询自链不可终止**；
3. `_deferBudget`（单次脚本执行 microtask 派发上限 256）在首个 checkpoint
   被泵链烧尽；桥测试环境 `__zw_begin_script`（预算重置点）只由生产
   webview.rs 路径调用、裸 execute 从不重置 → 预算归零后**同环境一切后续
   `_defer` 回调永久丢弃**——img/svg load 派发（available-images 命中臂
   `_defer(loadDispatch)`）全灭；
4. 生产 webview/runner 有宿主定时器（`__zw_setTimeout` webview.rs:3090 /
   testharness stub `__zw_test_setTimeout`）→ task 粒度真实任务，不受此
   病——svg 通道 SMIL 面 1,209P 零位移；但「泵 tick 排队早于其他 deferred
   回调时单 checkpoint 内预算耗尽」的饿死面在生产同构存在（泵 tick 链于
   同一 checkpoint drain 内 ×256 自链）。

修（part09b.js `_scheduleTick` 单点）：无宿主定时器（双探针
`__zw_test_setTimeout`/`__zw_setTimeout` 皆缺，与 part01 `_zwHostSetTimeout()`
同判据）不排轮询——轮询语义 = task 粒度（模块头注 ⑦、learnings #30 禁
microtask 链），无任务调度环境下本就不成立；seek 一次性扫描
（`_scheduleSeekScan`）单发自终结不守卫。生产/runner 双探针命中，行为零
变化。

## 验证

- solo 7 案（filter 五连跑，/tmp/s434/verify-ws.log + verify-solo.log 终稿
  复跑）：ws×2 + img×2 + sw-iframe + fetch_filtered + stream 全绿
  3949 filtered out；
- 全桥套件（终稿代码，/tmp/s434/verify-full.log）：**1568 passed /
  0 failed**（前 1561P/7F，12 红中 5 例他流 94883fbbe 已偿）。push 前
  pull --rebase 入他流两笔（css anchor dbef933d3 + engine B1 子文档键
  路由 6f7acb01d，桥面零交集），rebase 终稿复跑（
  /tmp/s434/rebase-suite.log）：**1570P/0F**——+2 系他流新增桥测试，
  零失败维持；
- 25 part 拼装 node --check（/tmp/s434/assembled-s434.js，13 文件式拼装
  顺序同 js_dom_bridge.rs 装配序）：OK；
- `cargo fmt -p zero-engine -- --check`：干净；
- clippy `-p zero-engine --lib --tests -- -D warnings`（test-guard 包裹，
  /tmp/s434/clippy.log）：exit 0 干净。

移交/留账：生产 webview/runner「泵 tick 链单 checkpoint 内 ×256 自链」
同构饿死面（预算未烧至枯、但同 checkpoint 内后到 deferred 回调仍可被
挤占）——行为零变化确认后不施工，留观；learnings #30（禁 microtask
自链）新增实例注记。
