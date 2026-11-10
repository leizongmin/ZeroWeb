# M2 切片 383（s383）——script-crossorigin-network 翻绿（cookie 送达桥 + cross-origin.py 仿真，域 T 1→0）

日期：2026-11-10（承 s382 挂账：基建面 T=1 勘根——script-crossorigin-network cross-origin.py 语料资产缺失盘点 + CORS 强制面核查）
域：html/semantics/scripting-1/the-script-element（script-crossorigin-network.sub.html）
性质：**施工轮——文件级 Timeout→8 子测全 Pass（真案 -1T），the-script-element 域 Timeout 清零；engine shim 双文件 + runner 路由双文件**

## 案面与勘根

案形：8 async_test = same-origin × {use-credentials, invalid(gibberish), missing} + cross-origin × {use-credentials, invalid(gibberish), missing, UsE-cReDenTiAlS(混合大小写), uſe-credentialſ(非 ASCII)}——各 `script.crossOrigin = …`（**IDL property 赋值**）+ `setAttribute('src', …)` 动态插入，`document.cookie = "milk=1"` 后 onload 断言 `included` 真假。期望矩阵：same-origin 全 true；cross-origin use-credentials/missing/混合大小写 true，invalid/非 ASCII false（→anonymous→same-origin credentials 跨源不附 cookie）。solo 红形 = 文件级 Timeout（8 pending）。

**根因分解（三层事实）**：

1. **语料资产缺失**：`cross-origin.py` 整树缺席（wpt-data find 零命中）——8 个脚本 fetch 全 404 → error 事件 → onload 永不触发 → async_test 全 pending 文件 Timeout。
2. **cookie 状态通道断裂**：上游 wptserve 由浏览器真 jar 按 credentials mode + 同源性附 `Cookie` 头（milk=1 → `var included = true`）；runner in-process fetch 通道无 jar（testharness.rs:8255 在册），`document.cookie = "milk=1"` 落 in-JS `_zwCookieJar`（part06 cookies-compat M2，RFC 6265bis 存储模型）——Rust 侧路由不可见。
3. **credentials mode 判据面**：net-api M4 cors-cookie.py 仿真先例（testharness_py_routes/es.rs:251+）——runner 无 jar 时跨源 cookie 送达以 `FetchRequest.credentials == include` 判定（RFC 6265bis §5.4）；WS 握手桥 `_zwCookieHeaderForUrl`（part06:333，M5-S3 globalThis 桥）已实现完整 jar 检索（域/路径匹配、secure、过期、路径长度排序）——状态桥出请求头（而非路由读 stash）的机具现成。

**关键事实盘点**：

- 动态 classic 脚本取回 = s110 臂 `fetch(url)`（part03.js，part05 src set trap 经 globalThis 取用 `_zwPrepareSrcScript`）→ shim fetch wrapper → fetch bridge → py_routes dispatch（host 无关路径匹配——same/cross 双端口同路径命中同路由）。
- shim fetch 默认 credentials = `'same-origin'`（part01.js:5714，fetch spec 现行默认）——裸 `fetch(url)` 恒 same-origin。
- crossOrigin IDL getter 对 SCRIPT 已实现（part04.js:13623 R5008 片 d：missing→null、ASCII CI、invalid→'anonymous'）；AUDIO/VIDEO 分支（:662）另有同形——SCRIPT 元素读 `el.crossOrigin` 即归一值。
- 跨源 cors GET 已自动附 `Origin: <页 origin>`（part01.js:6033 net-api M2-S3c）——路由可上游忠实回显 ACAO；CORS 响应判定面在 shim 侧活跃（`_zwFetchApplyFilteredResponse` :4775）。
- **include 投影回归面**：fetch init 投影 credentials=include 会翻转 `include+'*'` CORS 判定（:4799——include 下 ACAO:* 拒收），而真实浏览器缺省脚本为 no-cors 无 CORS 判定；no-cors 本体不可走（opaque 体空，脚本消费需正文）。s110 跨源动态脚本案面全域恰 2 文件（muted-errors + 本案），投影即 muted-errors 假红风险。
- **漂移面收敛**：document.cookie 写面 corpus 恰 5 文件（credentials 族 5 个 credentials.sub.html/credentials-iframe）；classic s110 车道 jar 非空使用者仅本案。css-module/text-module/json-module 三份 `resources/cross-origin.py` 为**不同上游脚本**（module 载荷 + ?id/?origin 参数）——路由必须 Exact 路径注册，suffix 匹配会劫持兄弟面。
- 上游 handler verbatim（master 拉取）：Origin 头在场 → ACAO=origin 回显 + ACAM=GET + ACAC=true；OPTIONS → 空体（CORS 头照设）；Content-Type: text/javascript；Cookie milk 缺省/≠1 → `var included = false;`，milk=1 → `var included = true;`。

## 施工（三件套）

1. **part03.js s110 臂 credentials 判据 slot**（`var _p110 = fetch(url);` 前置）：`el96.crossOrigin` 归一读（getter 缺席/异常回落 include），anonymous → `'same-origin'`、null/use-credentials → `'include'`，置 `globalThis.__zwScriptCookieCred`。**fetch init 不投影 credentials**（见 include 投影回归面）——mode 语义经 slot 交 fetch wrapper 承载。
2. **part01.js fetch wrapper 双编辑**：①`_zwFetchMainImpl` 头部（credentials 解析后）slot **一次性捕获**——捕获点先于 preflight/ctor 校验等任何提前 return/throw 路径，script 车道外恒 null no-op，不向后续 fetch 泄漏 stale slot（trickle/delay 延迟发起臂消费闭包捕获值）；②Origin 附加块之后（preflight 判定面之后——真实浏览器 Cookie 网络层追加不计 safelist 不触发 preflight）附加臂：判据 = slot include 或请求同源（`_zwReqOrigin === _zwDocOrigin`，RFC 6265bis §5.4 与 s338/net-api M4 仿真同判据式），jar 检索走 `_zwCookieHeaderForUrl`（空 jar 返空串不附），`Cookie:` 头附入 headersWire。
3. **runner 路由**（testharness_py_routes）：`site.rs` 新增 `route_cross_origin_py`（上游 1:1——Origin 回显 CORS 三头/OPTIONS 空体/Cookie milk 扫全部 cookie 对判 included 体/text/javascript，`wpt_add_fetch_metadata` 尾调同惯例）+ `mod.rs` 注册 `PyRouteMatch::Exact("html/semantics/scripting-1/the-script-element/resources/cross-origin.py")`（the-script-element 族聚簇，module-fetch-tester 邻位）。

## 设计取舍（Option F-min）

cookie 送达面三案对照：**A 投影 fetch init credentials**（include+'*' CORS 判定翻转 = muted-errors 假红面 + FetchRequest.credentials 面变宽）；**B 通用 RFC 6265bis 附附**（判据含 fetch init include + 同源对所有 initiator 生效——通用忠实但触发面不可由案面审计）；**C（采纳）slot 收窄**：slot 由 s110 臂前置、wrapper 一次性消费，附加判据内嵌同源臂（same-origin anonymous 子测 test2/3 由同源臂覆盖）——漂移面经构造收敛到本案（jar 非空 × classic 车道 × slot 在场三重门），muted-errors 无 document.cookie 写面零漂移。preflighted script 请求重入臂不重捕（corpus 无此形，注释在册）。

## 验证

- solo（FILTER=script-crossorigin-network）：**8/8 Pass 首跑即中**（Timeout→Pass；invalid/非 ASCII 两案 false 判据正确）
- credentials 族 solo（FILTER=credentials）：**24/24 Pass**（dynamic-imports-credentials ×2 姊妹案等全保持）
- muted-errors 归因：solo 无结果行 exit 1——**stash 基线二进制同形复跑**（无行 + exit 1）= 先在 crash 形非本刀回归；域账两侧均无该案行（先在覆盖缺口，记账不修）
- shim 22-part 拼装 node --check PASS
- 域 A/B join（s382 stash 基线 A vs 本刀 B，(key,state) 口径）：**A=1130P/1F/2T（1137 对）→ B=1150P/1F/0T（1155 对）**；A-only 恰 2 键 = credentials-setTimeout + script-crossorigin-network 两文件级 Timeout 旧形（前者为 s382 修复前基线侧），B-only 恰 20 键 = 12（credentials-setTimeout 子测）+ 8（本案子测）Pass，**零 STATE-DIFF**，F=1（dynamic-imports-script-error 平行流漂移）原样——**the-script-element 域 Timeout 清零（T 1→0）**
- make test（fmt 前置过 + workspace 腿，guard --compile-first）：聚合 **9398P/7F**——7 红（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_sw_messages_dispatch / response_body_readable_stream_r2967 / websocket_error_nonterminal / websocket_event_flow_open_message_close）与 s371 基线名单逐名精确同构零新增红；腿中止形态与 s381/s382 同（后续腿未达）
- clippy：`cargo clippy -p zero-wpt-runner --all-targets -- -D warnings` PASS（首跑 7 错 = 本刀 doc 注释 list-item 缩进，修后过；QUICKJS_CLIPPY_CRATES 不含 runner，targeted 补跑为 Rust 变化轮例行动作）；fmt 全绿
- 落盘后注释级修订（clippy doc 修复 + mod.rs fmt 单行化）零行为差，行为验证（solo/域 join）均以终态源码构建二进制承担

日志 /tmp/s383/（domain-new.raw、a.kv、b.kv、join.sh、make-test.log、make-test.exit、clippy.log）。
