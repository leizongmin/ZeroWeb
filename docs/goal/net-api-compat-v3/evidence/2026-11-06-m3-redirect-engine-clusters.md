# M3（二）：fetch redirect 引擎语义簇收敛——七簇 99F+1T → 363P/0F/1T，+102P 零回归

日期：2026-11-06。前置：M1b redirect 批次挂账（99F+1T 七簇归 M3）+
M3（一）ws interfaces（全量基线 **13,109P**/1,567F/16T，分母 14,692 行 /
14,650 键）。本批改 `crates/engine/src/js_dom_shim/part01.js`（fetch shim
六刀）与 `tests/wpt-runner/src/testharness.rs`（`.headers` sidecar 全量头 +
redirect.py location 字节解码），零 fixture 新增端点、零通道配置变化。

## 一、七簇收敛（引擎改动逐簇）

1. **referrer 簇 63F**（redirect-referrer-override 51 + redirect-referrer
   12）：referrerPolicy 计算抽 `_zwReferrerPolicyOut` helper 供初始请求与
   重定向环共用；重定向链**逐跳**重算 Referer——每跳读重定向响应
   Referrer-Policy 头（逗号分隔取最后有效 token，**替换**当前策略），再按
   determine-request's-referrer 重算外发值并替换线缆 Referer。
   referrerSource 取**链式**语义：上一跳实际外发值（非 spec 字面固定文档
   URL）——期望矩阵 origin-严于-full 且与先后序无关 ⇒ 链式（origin-only 源
   经全 URL 策略重算仍 origin-only；空源链上恒不复活）。浏览器共识定谳：
   wpt.fyi Firefox 162.0a1 / Edge 159.0.4378.0 master 对该案面均 128/128。
   beacon 通道不参与重算（M4-S25 一次性计算零回归）。
   首刀残余 14F（init ∈ {origin, strict-origin} 行送出全 URL）即以固定文档
   URL 为源所致，改链上值后精确清零。
   https://fetch.spec.whatwg.org/#http-redirect-fetch
   https://w3c.github.io/webappsec-referrer-policy/#set-request-referrer-policy-on-redirect
2. **mode 簇 11F**（redirect-mode 全绿 61/61）：三刀——(a) wptserve
   `.headers` sidecar 从仅 Content-Type 扩为**全量响应头**（INI 行解析，
   per-file 语义；静态服务臂追加，Content-Type 替换、其余不覆盖已存在头）
   ——hop2 静态 cors-top.txt 借 sidecar ACAO `*` 过 cors check（探针三形态
   A/B/C 定位：fixture redirect.py 恒回 ACAO 已在，缺口在静态资源）；(b)
   main fetch 步骤 3 mode 分支 no-cors 臂：redirect ≠ follow 且 current
   URL 跨源 → 入口 network error（同源 URL 先被「同源 + tainting basic」
   臂截住——spec 臂排序）；(c) HTTP-network-or-cache fetch 步序：cors check
   移到 manual→opaqueredirect 转换**之前**（manual redirect with a CORS
   error 期望 network error 而非 opaqueredirect）。
   https://fetch.spec.whatwg.org/#main-fetch
   https://fetch.spec.whatwg.org/#http-network-or-cache-fetch
3. **origin 簇 10F**（redirect-origin 40/40，含 upstream 双引擎红 10 行
   crbug.com/1432059）：hop Origin 发射门收敛——**跳不透明**（已污染链再跨
   源，`_zwHopOriginOpaque` sticky）→ 'null'；**跳前已污染或本跳跨源**
   （clean 链首次跨）→ 文档 origin；**clean 同源链同源跳 → 不发射**。
   矩阵覆盖 cors-redirect 20 行 + redirect-origin 40 行全绿（clean same→
   same 期望 null：append-a-request-origin-header cors 臂字面「恒 append」
   与浏览器行为不符，以 corpus 为准；跳前 tainting 快照 `_zwHopWasTainted`
   先于跨源臂置位取值，区分「clean 首跨」与「已污染再跨」）。
4. **method 簇 9F**（redirect-method 全绿 15/15）：(a) hop Content-Length
   门——体 null **或空串**（无体 GET/HEAD 跳：初始 body 变量无体时为 '' 非
   null）不加 CL（浏览器无体 GET 不带 CL；POST 空体跳仍 CL:0 对齐初始请求
   M2-S10 口径）；(b) 303 方法改写扩为**任意非 GET/HEAD** → GET（原仅
   POST——spec 方法改写步「303 and method is not GET or HEAD」；HEAD 不
   改写）。
5. **back-to-original 簇 2F**：response tainting 持久——链上曾跨源（initial
   跨源或任一跳跨源，mode 无关追踪 `_zwChainCrossed`）后终跳回到同源不回
   退：cors 链 type 'cors'、no-cors 链落 opaque 过滤（main fetch mode
   switch 同源臂门在 tainting "basic"；whatwg/fetch#737）。opaqueredirect
   与已 opaque 终跳不重包，SW/img 内部通道豁免。
6. **location-escape 簇 4F**（tentative，whatwg/fetch#883）：两层根因——
   fixture `location` 参数改**字节语义解码**（`wpt_query_value_bytes`，
   逐字节 Latin-1 映射：UTF-8 文本解码把 %E2%98%83 变 U+2603 → JS Headers
   ByteString 校验拒响应、%FF lossy 变 U+FFFD 丢字节）；shim 解析 Location
   前对 0x80..0xFF 字符逐个 `%XX` 单字节转义（`new URL` query 通道会 UTF-8
   重编成 mojibake）。根因与避免方式落
   [learning](../../../learnings/bugs/2026-11/2026-11-06-response-header-values-are-byte-sequences.md)。
7. **keepalive 1T**（redirect-keepalive）：维持挂账（keepalive 语义簇，
   随 M3 余项 xhr 在账失败簇一并收敛）。

## 二、定向 A/B（FILTER=redirect 面，同树基线对照）

fetch/api/redirect 文件集：338P/25F/1T → **363P/0F/1T**（+25P，336 子测试
批次通过率 99.7%，批次门 ≥98.5% 达成）。链外复核：cors-redirect 115P/0F
维持（sidecar/origin 改动零外溢初验）；CHANGE 余面（cors-preflight-redirect
5F / cors-cookies-redirect 2F / xhr send-redirect 族 / response-static-
redirect 1F 等）逐行对照全量基线在账，非本批回归。

## 三、全量 A/B（零回归门）

同树基线 13,109P/1,567F/16T → cur **13,211P**/1,465F/16T（+102P）：分母
14,650 键逐 subtest 不变（14,692 行含 42 重复行两侧一致），**changed 102
全 F→P、零 P→F、零 T 变化**。构成：referrer 63 + mode 11 + origin 10 +
method 9 + escape 4 + back-to-original 2 + 链外搭车 3（fetch/api/basic/
referrer.any.js「origin-when-cross-origin after same-origin redirection」
——referrer helper 同源链路径；xhr send-redirect 两行——303 方法改写外溢
XHR 同构通路），逐行可溯源。

sidecar 全量头对 content-length 面、security-features 共享资源面的外溢
由全量 A/B 覆盖（零 P→F）。本轮 corpus 目录拉取遭遇 GitHub 匿名限流
（403，语料走既有缓存，分母与基线逐键一致）——续拉须带 GITHUB_TOKEN
（既有约定）。

## 四、批次门与挂账

- redirect 批次门（M1b 挂账面）：99F+1T 七簇 → 1T（keepalive），**99/100
  = 99.0%**；fetch/api/redirect 实跑口径 363/364 = 99.7%，均 ≥98.5%。
- M3 余项（下一步）：xhr auth 五簇 12F + `__custom` 排序簇 1F + 重定向终
  跳 sync cors-filter 同规 + xhr 在账失败簇（formdata/submitter 4F +
  overrideMimeType Shift-JIS 1F + cookies 1T）→ M4 coverage 复测收口。
