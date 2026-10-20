# M5-S1 — websockets corpus 导入 + runner WS fixture 通道 + 基线（2026-10-13）

**背景**：P6 重入核记（commit `c178abdcc`）——zero-web 流 2026-10-08~09 落地
WebSocket 页面 API + zero-net 底座（`crates/net/src/websocket.rs` tungstenite +
`engine/ws_bridge.rs` + shim part06.js），Mission 约束 2 析取重入条件
「zero-net WS 底座就位」兑现。本切片 = M5 首步：corpus 导入 + 度量通道 + 基线，
纯 runner/test 资产切片，engine/net 产品面零改动（run-rules §9 共享面避让——
WS 语义修齐若触 shim part06.js 须与 zero-web 流碰头核对）。

## 交付物

1. **corpus 拉取**：`20-net-api-compat.sh` DIRS 增 `websockets`；runner 侧
   `NET_API_CORPUS_SUBDIRS` 增 8 项（websockets 顶层 + binary/closing-handshake/
   constructor/cookies/keeping-connection-open/opening-handshake/security 子目录）。
   拉取 134 文件 / 68 `.any.js` 候选 + 老式 `.html` 案。
2. **runner 内嵌 WS fixture 端点**（`tests/wpt-runner/src/ws_fixture.rs`，~660 行）：
   手写 RFC6455 server（sha1 accept 推导 + 帧编解码 + continuation 聚合 +
   ping 自动 pong）。**语义基准 = 上游 `websockets/handlers/*.py` @
   WPT_REV `31597693` 逐文件核读**（20 个 handler 拉取对照），21 端点一一对应
   （/echo 族、/empty-message、/echo-cookie、/echo-query、/origin、/referrer、
   /protocol_array、/protocol、/handshake_no_protocol|no_extensions|protocol、
   /wrong_accept_key、/invalid、/handshake_sleep_2、/set-cookie(_http)、
   /simple_handshake、/receive-many-with-backpressure）。未知 path 404 拒绝握手
   （wptserve 无 handler 语义——opening-handshake/006 断言不干净失败）。
3. **FixtureWsHost**：`ws://*.wpt.test[:port]/...` 权威段 → `127.0.0.1:<同端口>`
   改写后委派 `default_net_ws_host`——zero-net 真 TCP/tungstenite 全栈被测，
   Origin/Cookie 语义不改写（/origin、/echo-cookie 即被测面）。
4. **token 替换扩集**：`apply_wpt_substitutions` 增 `{{ports[ws][0]}}`/
   `{{ports[wss][0]}}`/`{{ports[h2][0]}}`（全映射内嵌 fixture 端口）+
   `{{hosts[alt][www]}}`（www1.alt.wpt.test）。**`.sub.js` 服务端替换补全三条加载
   通道**：script fetcher（M4-S22 已有）、fetch handler 静态回退、
   `inline_local_scripts`（本切片两处——`.html` 案外部脚本走预处理直读，缺此则
   SCHEME_DOMAIN_PORT 带 token 字面量 → Invalid URL 假阴簇）。
5. **websockets 案页面 http origin**：`run_testharness_html_inner` 对
   `websockets/` 前缀 case 用 `http://wpt.test`——上游 `?default` 变体语义即
   http 页面 + ws 连接（constants.sub.js 判 `location.protocol === 'https:'` 翻
   wss 分支）；通道级 scheme 翻转是 P3 挂账另案（两前置审计），此处局部生效。
6. **check.py 合成 fixture**：`websockets/security/check.py`（上游逐字等价——
   收到 `Sec-WebSocket-Key` 头回 FAIL 否则 PASS；该头系 Fetch 规范 forbidden
   header name，XHR setRequestHeader 须被剥离）。security/002 **Pass**（shim
   剥离行为在位）。
7. **筛减记账**：`.https.*` 案（需安全上下文 + TLS wss 端点，两臂均不可测）；
   `multi-globals`/`unload-a-document`（多文档/多页编排，runner 无管道）；
   `.window.js`（bfcache 族，枚举不含）；worker/iframe 通用规则照旧
   （Create-on-worker-shutdown 被自动筛除）。

## 基线（隔离跑，`make testharness-net-api FILTER=websockets`）

| 口径 | 通过率 |
|---|---|
| 子测试 | **334/375 = 89.1%** |
| case | **78/106 = 73.6%**（Fail 26 案 + Timeout 2 案） |

首轮全链路冒烟（Close-1000.any.js）一次通过；两轮 fixture 保真修正后数字收敛
（迭代记录见下）。门禁 exit≠0 为基线轮既有失败如实记录（照 M1 先例，修齐轮推进）。

## 失败聚类（26 Fail + 2 Timeout → 修齐归属）

| 簇 | 案数 | 归属 |
|---|---|---|
| close()/constructor IDL 校验面（close 码域 0/500/NaN/String/null/2^16+1000/1005/2999/仅 reason、binaryType 非法值、无参构造、参数转换、SYNTAX_ERR 族 ×5、TreatNonCallableAsNull ×4、Send-null DOMString 转换 Timeout） | 19 子测试 | **engine shim part06**（zero-web 流共享面——修齐须碰头核对） |
| `ws.protocol` 属性面（服务器选定协议不上抛——010/011 证明客户端已校验协商响应，Open 事件 protocol 未落到属性） | 4 案 | engine shim + ws_bridge Open 事件协议字段（`ws_bridge.rs` t8k 已有 FIXME 记录） |
| WS cookie jar 集成（握手 Cookie 不携带 / Set-Cookie 不落 document.cookie） | 4 案 | zero-net WS ↔ cookie jar（既有挂账族「cookie-jar credentials ~13 腿（storage goal）」邻域） |
| 网络栈面（basic-auth 401 挑战流、ws 301 重定向语义） | 2 案 | zero-net（goal 排除「HTTP 栈本身」，跨域记账） |
| 连接状态机面（Close-delayed CONNECTING 期 close、constructor/014 serialize establish、013 25 并发 Timeout） | 3 案 | engine shim/bridge 状态机 |
| URL 编码面（windows-1252 percent-encoding UTF-8） | 1 案 | url 解析（url crate 深水邻域，P4 残余同族） |
| runner 工件（constructor/017 顶层 `__SCHEME` 引用——inlined constants 与页脚本执行序，预处理工件非产品缺口） | 1 案 | runner 预处理（记工件，不动通道语义） |

**不测面**：`?wss`/`?wpt_flags=h2` 变体（TLS 面——fixture 无 TLS）；`.https.*`
案 5 个（同上，已筛减）；multi-globals/unload-a-document/bfcache（结构面）。

## 迭代记录（fixture 保真修正，各根因已修）

1. **Invalid URL 假阴簇（~30 子测试）**：`.html` 案 constants.sub.js 三条加载
   通道中 fetch handler/inline_local_scripts 未做 `.sub.js` 替换 → token 字面量。
   修：两处读点补 `apply_wpt_substitutions`（上游 wptserve `.sub` 服务端替换语义）。
2. **constructor/012 Timeout**：`/handshake_no_protocol` 服务端 keep-open——上游
   pywebsocket handler return 即走服务端关闭握手。修：单发型 handler 数据发完发
   空 close 帧等回关。
3. **opening-handshake/006 Timeout**：未知 path 我最初兜底 echo（假开连接）——
   上游 wptserve 无 handler 即 404 握手失败。修：KNOWN_PATHS 白名单前置拒绝。
4. **closing-handshake/002/004 wasClean**：`/echo_exit`/`/echo_close_data` 上游
   **不回显** Goodbye 且 handler return 走干净关闭；`/echo` 先回显再关（语义分岔
   即断言面）。修：echo_loop 分叉。

## 门禁

- `cargo fmt --all -- --check` 干净；`cargo clippy -p zero-wpt-runner
  --all-targets -- -D warnings` 零告警；`cargo test -p zero-wpt-runner`
  （test-guard 包裹）216P/0F。
- 产品 crate（engine/net）零改动——无需 workspace 全量门禁/bench-gate。

## 下一步（M5-S2 候选）

1. close()/constructor IDL 校验簇（19 子测试，纯 shim 校验面——与 zero-web 流碰头核对后修齐）
2. `ws.protocol` 属性面（Open 事件 protocol 字段落属性，4 案）
3. WS cookie jar 集成（与 storage goal 挂账族并案评估）
