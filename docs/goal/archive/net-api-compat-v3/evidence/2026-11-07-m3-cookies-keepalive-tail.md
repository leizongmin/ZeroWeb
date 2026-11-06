# M3（八）：cookies/keepalive 尾批 2T 收口（cookies.http.html T→P，redirect-keepalive 排除记账，13,255 → 13,256P 零回归）

日期：2026-11-07。前置：M3（七）formdata/submitter 簇（全量基线
**13,255P**/1,422F/15T，分母 14,692 行 / 14,621 键）。M3 在账余项
「cookies.http.html 1T + redirect-keepalive 1T」双 T 探针定谳，一案修复、
一案排除记账，**M3 在账余项清零**：

| # | subtest | 处置 | 根因 |
|---|---|---|---|
| T1 | xhr/cookies.http.html | **修复**（testharness.rs get-set-cookie.py fixture） | POST 撞非 GET 兜底门 handler Err → onerror 挂起（见 §一） |
| T2 | fetch/api/redirect/redirect-keepalive.any.js | **排除记账**（net_api_case_skipped 单名门） | 多窗口编排面，runner 无多窗口管道（见 §三） |

## 一、T1 归因（探针定谳）

临时探针页（breadcrumb 经故意 assert_equals 失败透传，收口前已删）：
XHR POST `resources/get-set-cookie.py?clear=1` →
`sent|onerror:0|onloadend:0|dc=""`——请求**网络错误**而非 404。两层根因：

1. fixture 缺失：`xhr/resources/get-set-cookie.py` 为 wptserve 端点
   （corpus 拉取面排除 .py），runner 无该端点分支。
2. 兜底门顺序：fixture 端点分支全部先于「非 GET 兜底拒绝」门
   （net-api M4-S18——`req.method != "GET" && != "OPTIONS"` →
   `Err(method not supported)`），测试用 POST 撞门 → fetch handler Err →
   shim XHR 判网络错误（status 0）→ onerror。测试 promise 只在
   `req.onload` resolve → pending:1 永挂 → 案级 Timeout。

## 二、T1 修法（fixture cookie 状态机）

testharness.rs 两处，零引擎改动（beacon/link-sheet/alt-no-store stash
同款 per-case 表先例）：

1. handler per-case 表 `get_set_cookie_stash`：
   `Arc<Mutex<BTreeMap<String, (String, bool)>>>`（name → (value, secure)）。
   不接引擎 `zero_net` CookieStore——进程全局 jar 面向真实网络栈，runner
   in-process fetch 通道无 Set-Cookie 消费点；表仅承载案内往返态
   （测试自身首尾 clearCookies 对称，正常路径表尾恒空，跨案泄漏面自清）。
2. `xhr/resources/get-set-cookie.py` 分支（先于非 GET 兜底门——测试用
   POST）：上游 wptserve handler 逐语义（pin 3159769 逐字核对）——
   - 恒 Set-Cookie 双枚：`WK-test=1`（会话）+ `WK-test-secure=1; Secure`；
     `?clear=1` 时均带过期 Expires（上游为昨天，epoch 同为过去时刻，删除
     语义等价）；
   - 响应体 = 请求实际送达的 cookie 回显（逐枚前置空格 ` name=value`，
     无则空体——上游 `content + b" " + cookie + b"=" + value`）；
   - **Secure cookie 不随 http 请求送达**（RFC 6265bis §5.4——runner 页锚
     http://wpt.test，`WK-test-secure` 恒不在回显体，测试核心断言）；
   - 回显体先于本响应 Set-Cookie 生效（上游 `response.content` 取
     request.cookies）。

定向：`make testharness-net-api FILTER=xhr/cookies.http` → 1P
（「Basic non-cross-site cookie handling in XHR」）。

## 三、T2 归因 + 处置（排除记账）

探针定谳：`openType=function|openRet=null|msg=null`——`window.open` 存在
但返回 null（runner 无多窗口管道），`keepalive-helper.js
keepaliveRedirectInUnloadTest` 打开 NOTSAMESITE 子窗后
`getTokenFromMessage` 等 opener postMessage 永不到达 → 六案 setting up 全
挂（runner 报 `pending:6`）→ 案级 Timeout。

处置：`net_api_case_skipped` 单名门（websockets back-forward-cache
name-skip 同款先例——`window.open` 内容标记不可用：该文件的窗口编排全在
META helper `keepalive-helper.js`，case 源本身无标记）：

- 排除口径 = M0 甄别档 multi-globals/unload-a-document「runner 无多文档
  管道」同因（重入条件：multi-window 管道落地）。
- `.https.any.js` 变体走 iframe flow（`keepaliveRedirectTest`）不在此门，
  定向过滤复跑双 P 维持既有绿面。

## 四、验证

- 定向：cookies.http.html 1P；redirect-keepalive 过滤仅余 .https 变体双 P
  （skip 门生效）。
- 全量：`make testharness-net-api`（test-guard 包裹，--per-proc-mem 4
  --total-mem 8 --time-limit 900）对 M3（七）基线逐 subtest A/B：
  **13,255P/1,422F/15T → 13,256P/1,422F/13T**，分母 14,621 → 14,620 键；
  added 1 = cookies 子测试（T→P 面——案级 Timeout 键移出、子测试键入账），
  removed 2 = cookies.http.html / redirect-keepalive.any.js 双案级 Timeout
  键；**零 P→F，零 T 变化，零其他变化**。
- clippy `-p zero-wpt-runner`：0 warning；`cargo fmt --check` 干净。
- 日志：基线 `/tmp/netapi-m3-fd-family-full-cur.log`；当前
  `/tmp/netapi-m3-cookie-family-full-cur.log`；对照脚本 `/tmp/m3-auth-ab.py`。

## 五、M3 收口

在账失败簇/挂起全清：fetch redirect 七簇 0F、xhr auth 五簇 0F、
overridemimetype 族 0F+0T、formdata/submitter 簇 2F 重分类移交他域
（不计门）、cookies/keepalive 2T 本批处置。在账外存量缺口不动
（responsexml 族 19 行、permissions-policy 2F 策略面排除）。

下一站 **M4：coverage 复测收口**（DC-4 数值门——批次门 98.5% / DC-3
interfaces 面门 / 零回归门已随各批落账，复测三族 coverage 头条对
coverage-2026-11-02 基线 67.54% 不回落）。

## 六、挂账

- redirect-keepalive.any.js 重入条件：multi-window 管道（navigation/
  multi-window goal 域）。
- cookies 通道语义面（真 Set-Cookie jar 消费点）未接：本 fixture 为
  in-process 表态；引擎 fetch 通道 Set-Cookie→CookieStore 消费与请求侧
  Cookie 注入（t8k-delta F1 FIXME 同源欠账）留 net 流后续批次。
