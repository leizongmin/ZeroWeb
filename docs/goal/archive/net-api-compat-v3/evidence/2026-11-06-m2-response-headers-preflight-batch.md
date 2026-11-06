# M2（三）：getResponseHeader 响应头控制面 + CORS preflight 端点扩表

日期：2026-11-06。前置：M2（二）auth 面批次（基线
13,005P/1,629F/16T；同树重捕基线 13,031P/1,645F/16T——分母 +42 漂移在
非 xhr 面 ws 时序类，xhr 面 928P/314F/5T 与上轮账逐字节一致，本批 A/B
以同树新捕基线为准）。

## 一、改动内容

1. **fixture 套件**（testharness.rs，上游逐字/语义等价，pin
   `315976933870`）：
   - `xhr/resources/headers.py`：getResponseHeader 响应头控制面——同名
     双值 comma join、空值、UTF-8 字节值（上游 `u"…".encode("utf-8")` 的
     ByteString 头值视图 = 三字符 U+00E2 U+0080 U+00A6）、Set-Cookie
     隐藏面；Server/Date 固定合法 IMF-fixdate（wptserve 每响应自动头，
     server-date 案仅断言非 null）。
   - `xhr/resources/chunked.py`：Trailer 声明 + chunked 帧；runner 无传
     输层，体直给解帧拼接串，trailer 头不并入响应头。
   - `xhr/resources/headers.asis`（asis 嵌入 map 第七项）：原始 HTTP 响
     应字节（foo-TEST/FOO-test 同名双值 + `__Custom`/`ALSO-here`/`ewok`）。
   - `xhr/resources/parse-headers.py`：`?my-custom-header=` 回显为响应头，
     头值 OWS trim（wptserve 头解析层行为）；值含 NUL 原样下发由引擎门
     转网络错误。
   - `xhr/resources/access-control-basic-cors-safelisted-{response,
     request}-headers.py`：safelisted 七头 + x-test 过滤断言面 / 四头
     回显 + ACAO 回显 + ACAC true（非 POST 拒 400 = 简单请求不预检）。
   - `xhr/resources/no-custom-header-on-preflight.py`：token stash——
     OPTIONS 携 x-test → 400 FAIL（preflight 不得携带自定义头），否则
     `stash[token]=PASS`；GET 携 x-test 回显 stash（take 语义）。
   - `xhr/resources/access-control-preflight-request-header-{lowercase,
     sorted}.py` / `-headers-origin.py` / `-invalid-status.py`：ACRH
     小写化 / 排序无空格精确串 / 禁含 origin / code 参数状态码四端点。
   - `wpt_status_text` 补 301/501（invalid-status fixture reason
     phrase；400 维持空串不扩——既有 400 案面 statusText 零外溢）。
2. **引擎 E1（part01.js `_makeResponseFromWire`）**：响应头值含 NUL →
   网络错误（fetch spec HTTP-network fetch 头解析 failure；`\x0B`/`\x0C`
   竖直空白合法——headers-some-are-empty 在账通过面，仅 NUL 拒）。错误
   wire tag 经 `_makeResponse` 新挂 `__zwFetchErrorTag`（窄化识别，其余
   错误语义不变）。
3. **引擎 E2（part03.js 同步 preflight）**：preflight 请求头取**最小集**
   （镜像异步 M4-S26——Origin + ACRM + Accept: */* + UA + ACRH）；旧实
   现全转发 syncHeaders 使 no-custom-header-on-preflight OPTIONS 臂探得
   x-test → 400 → 恒 network error。
4. **引擎 E6（part03.js 同步终响应）**：无重定向同步终响应跨源 CORS 门
   （镜像异步终局 `_zwFetchApplyFilteredResponse`——cors check +
   cors-filtered 头过滤；旧实现直填使跨源响应非 safelisted 头暴露、缺
   ACAO 也可读）。重定向终跳面仍待 M3 与 fetch redirect 簇同规。

## 二、定向 A/B（xhr 面，同树基线对照）

基线 928P/314F/5T → cur **961P/281F/5T**：changed 66 = **33 F→P + 零
P→F + 零 T 变化**，分母 1,247 不变。F→P 归因：

- headers-normalize-response ×15（fixture + E1 NUL 门 + E2 同步抛——
  3 error 臂 assert_throws_dom + 12 match 臂 OWS trim）。
- getresponseheader 族 ×5（headers.py：case-insensitive / server-date /
  cookies-and-more / chunked-trailer / special-characters）。
- preflight 端点扩表 ×7（headers-async / -sync / lowercase / sorted /
  headers-origin + safelisted-response / -request）。
- E6 语义暴露 ×6（send-non-same-origin ×2 / access-control-basic-denied
  / basic-get-fail-non-simple / basic-post-with-non-cors-safelisted-
  content-type——跨源无 ACAO 响应此前直填凑对，现正确网络错误）。

getallresponseheaders-cookies ×1（headers.py Set-Cookie 隐藏面）。

**残余 1F（预期）**：`getallresponseheaders.htm` 首子测试——期望序
`also-here, ewok, foo-test, __custom`（wpt.fyi Firefox 7/7 定谳为真实
浏览器行为）vs 引擎 Headers sort-and-combine 字节升序（`_` 0x5F 最先
→ `__custom` 恒第一）。排序面归 M3（共享 fetch Headers 通路，零回归
风险不入本批）。文件内其余 3 子测试（headers-basic/some-are-empty/
www-authenticate 单头 join）P。

invalid-status-{301,400,501} / getallresponseheaders-status /
getresponseheader-unsent-opened-state 五案 vacuous P → fixture 落地后
转**真实 P**（preflight status 门 E 已在 `_zwRunPreflight`；NUL/error
臂真实执行）。

## 三、全量 A/B（零回归门）

同树基线 13,031P/1,645F/16T → cur **13,064P**/1,612F/16T（+33P 精确
对应定向面）：分母 14,692 逐 subtest 不变，changed 33 全部为 xhr 面
目标族 F→P，**零 P→F、零 T 变化**——E1/E2/E6 共享通路改动零外溢
（cors-redirect 115 子测试、sync preflight 既有案、fetch 面逐对一致）。

**批次门**：本批面（18 翻转文件 + invalid-status ×3 + status/
unsent-opened-state ×2 vacuous→真实 P）实跑分母 39、pass 38 = 97.4%，
差 98.5% 门一线（1F = `__custom` 排序簇）。按 ws interfaces 批次先例
挂账随 M3 排序面收敛复验；净收益 +33P 零回归成立。

## 四、后续

- M3：getAllResponseHeaders 排序面（`__custom` 簇 1F）并入总账；重定向
  终跳 sync cors-filter 与 fetch redirect 簇同规；must-not-contain-cookie
  （跨源凭据 cookie jar 基建）随 M3 cookies 面收敛。
