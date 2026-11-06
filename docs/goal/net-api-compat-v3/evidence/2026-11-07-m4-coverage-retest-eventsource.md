# M4：coverage 复测收口（DC-4 三面数值 + eventsource 簇收口，13,256 → 13,275P 零回归）

日期：2026-11-07。前置：M3 收口（13,256P/1,422F/13T，分母 14,690 键）。
M4 两件事：①DC-4 coverage 复测三面数值（wptfyi-cache 10-29 summaries 同
universe 对 coverage-2026-11-02 基线）；②复测分域账显影的 eventsource
未归因簇（10F+4T）收口。终态 **13,275P**/1,412F/9T，**DC-4 落定**。

## 一、coverage 复测（DC-4 数值门三面）

wptfyi_gap.py coverage 工具跑 wptfyi-cache 10-29 summaries（与
coverage-2026-11-02 同 universe、分母逐字节同口径）：

| 面 | 分母 | 基线（11-02） | 复测 | Δ |
|---|---:|---:|---:|---|
| fetch | 13,274 | 1,894P / 14.27% | **2,266P / 17.07%** | +372P |
| xhr | 2,333 | 554P / 23.75% | **977P / 41.88%** | +423P |
| websockets | 2,389 | 102P / 4.27% | **105P / 4.40%** | +3P |

三面均升，头条不回落门过（基线 fetch 14.25% / websockets 4.27% /
xhr 23.75%）。全量头条由 weekly cron 产出，本 goal 只复测三面。

## 二、eventsource 簇归因（M1a 分母扩展显影）

M0 基线 eventsource 35P/0F/0T——M1a「枚举器窗体解锁」把 `.window.js`
变体带入通道分母（corpus 文件 10-27 已在、M0 时枚举器不选 window 形态
→ 71 案含 10F+4T）。DC-4 复测分域账该抓的未归因簇，逐案归因：

| 簇 | 案 | 根因 | 归 knife |
|---|---|---|---|
| constructor-non-same-origin | 4F | 非 http(s) scheme（ftp/about:blank/mailto/javascript:）初始失败走 reestablish（CONNECTING）而非 fail the connection（CLOSED） | engine A |
| cross-origin failCORS | 3F | 初始 CORS 失败同因——fetch rejection 恒 reestablish | engine A |
| request-cancellation | 1F | runner 无 window.stop→在途 fetch 中止管道（见 §五排除记账） | 排除 |
| cross-origin doCORS | 3T 键（挂起） | cors.py run=message/redirect/status-reconnect 委派臂缺失 | fixture B |
| reconnect.window | 2T 键 | status-reconnect.py cookie 态机缺失 | fixture B |
| format-field-id-3 | 3T 键 | last-event-id2.py（`id:` 空/裸 `id` 重置流）缺失 | fixture B |
| request-credentials | 3T 键 | cors-cookie.py（Last-Event-ID 态 + credentials 送达面）缺失 | fixture B |
| close test2 | 1F | reconnect-fail.py cookie 态机缺失 | fixture B |
| reconnect test2 | 1F | status-reconnect.py 同上 | fixture B |

### engine knife A（part05.js/part06.js EventSource）

spec 锚：HTML §server-sent-events processing model——network error →
reestablish（CONNECTING+error），**unless the connection has never been
established** → fail the connection（CLOSED+error）；WPT 三浏览器行为
标尺一致（非 http(s) scheme、初始 CORS 失败、请求取消均 CLOSED）：

1. constructor 增 `_everAnnounced`（连接曾 establish 与否分派面）+
   `_origin = resolved.origin`（MessageEvent.origin = 连接 URL origin
   **序列化**——spec dispatch 步；此前派 `this.url` 全 URL，
   cross-origin e.origin 断言面错值）；
2. connect() scheme 预检：非 http:/https: → `setTimeout(failConnection,
   0)`（constructor-non-same-origin 四 scheme 面）；
3. announce（OPEN）处置 `_everAnnounced = true`；
4. fetch rejection 分派 `everAnnounced ? reestablish : failConnection`
   （初始 CORS 失败/取消 CLOSED，已 announce 后维持 CONNECTING 重连）；
   sync throw 早于任何 announce → 直接 failConnection；
5. fetch init 补 `credentials: withCredentials ? 'include' :
   'same-origin'`（§constructing——credentials mode；链路
   _zwFetchMain → wire arg 9 → FetchRequest.credentials 既有，仅缺传参）。

### fixture knife B（testharness.rs，M3 get-set-cookie.py 先例）

- `es_cookie_stash` 表（cookie name → value）：reconnect-fail.py
  （无态→200 retry:2+data:opened；opened→200 data:reconnected；
  reconnected→204 data:closed 清态）、status-reconnect.py（首请求置态
  +`<status>`+retry:2[+ok_first data:ok]；态命中→200 data:data 清态；
  键 `request<id 缺省 status>`）、cors-cookie.py（Last-Event-ID 缺省→
  `id:1`+retry:200+data:first+置 cookie；`1`→`id:2`+data:second+清
  cookie）三态机共享承载；
- cors-cookie.py 凭证送达判定：cookie 值 = 表含 ident **且**
  `req.credentials == "include"`（knife A credentials 通道）→ COOKIE；
- cors.py run= 扩三臂：message（message.py 缺省形态）/redirect（302 +
  Location ← ?location，ACAO/ACAC 先行）/status-reconnect（共享
  `es_status_reconnect_response` fn）；
- last-event-id2.py 静态三流（type 1/2/3；4+ → invalid_test）。

### 探针归因插曲（preflight × redirect 复合面）

首修后 `redirect use` 仍挂：三腿 bisect 探针（面包屑经故意
assert_equals 失败透传，收口已删）——redirect 无 preflight **过**、
direct+preflight **过**、redirect+preflight **拒**（REJ Failed to
fetch）。根因：cors.py run= 委派臂不分手——OPTIONS preflight 打到
run=redirect 落 **302**（preflight 2xx 门不过）。修：OPTIONS 先于委派
应答早退（200 + ACAM/ACAH + 空 body；cors-cookie.py 同修——preflight
不再消费状态机）。ES shim 无涉，纯 fixture 顺序缺陷。

## 三、request-cancellation 排除记账

window.stop() 中止面（stop at timeout 0 → 在途 ES fetch 中止 → 未
establish → CLOSED）：runner `window.stop` 仅消费 navigation API 进行
中导航（part02.js M2-S4G `_navOngoing` abort），无「文档停止 → 在途
fetch bridge 请求中止」管道——fetch 正常返回 → announce OPEN
（onopen/onmessage unreached 守卫被 `_dispatch` try/catch 吞）→ error
落 CONNECTING → 断言 expected 2 got 0。`net_api_case_skipped` 单名门
（redirect-keepalive 同款），重入条件：window.stop → 在途 fetch 中止
管道落地（AbortController 臂不受影响——abort 信号管道独立既有绿面）。

## 四、验证

- 定向：`testharness-net-api FILTER=eventsource` → **76P/0F/0T**
  （57P/10F/4T → 全绿）。
- 全量：`make` 同参（test-guard --per-proc-mem 4 --total-mem 8
  --time-limit 900）→ **13,256P/1,422F/13T → 13,275P/1,412F/9T**
  （分母 14,690 → 14,695 键）。
- 逐 subtest A/B（对 /tmp/zw-results-m4 基线 JSON）：**F→P 9 /
  F→排除移出 1（request-cancellation）/ 文件级 Timeout 键移出 4 +
  真实子测试入账 10（cross-origin 3 + reconnect 1 + id-3 3 +
  request-credentials 3）；零 P→F、零 T→F、零其他状态变化**。
- clippy `-p zero-wpt-runner` 0 warning；`cargo fmt --check` 干净。
- 日志：基线 `/tmp/zw-results-m4/testharness-net-api.json`（文本
  /tmp/netapi-m4-text.log）；当前 `/tmp/zw-results-m4b/testharness-net-api.json`
  （文本 /tmp/netapi-m4b-text.log）；对照本档 §内联脚本。

## 五、DC-4 判定（M4 收口）

- 批次门：各批导入面 pass ≥ 实跑分母 × 98.5% 已随各批落账（M1a/M1b/
  M2/M3 各证据档）；eventsource 簇系 M1a 批次面未归因尾巴，本批收口
  后 M1a 批次门实质补齐（71 案 76P/0F/0T）。
- DC-3 面门：websockets interfaces ≥98.5% ✓（M3 收口 100%）。
- 零回归门：13,256 → 13,275P 逐 subtest A/B 零 P→F ✓。
- 头条不回落：fetch 17.07% / xhr 41.88% / websockets 4.40%，均不低于
  2026-11-05 基线 ✓（§一）。

**DC-4 落定。** Done Criteria 四项全绿：DC-1（M0 甄别三档）✓、DC-2
（批次四件套常驻通道）✓、DC-3（websockets 入账 + interfaces 门 +
xhr 残余归因档）✓、DC-4（本档）✓。

## 六、挂账（收口后移交/重入口径）

- request-cancellation 重入：window.stop → 在途 fetch 中止管道
  （fetch cancellation 域，46 号 runner-net-infra 邻接）。
- redirect-keepalive 重入：multi-window 管道（M3 档口径不变）。
- cookies 语义面：真 Set-Cookie jar 消费点未接（M3 档口径不变）——
  本批 cors-cookie/reconnect-fail/status-reconnect 态机同为 in-process
  表态；ES credentials include 通道（knife A）已入引擎常驻。
- responsexml 族 19 行 / permissions-policy 2F：M0 甄别档既有口径不动。
