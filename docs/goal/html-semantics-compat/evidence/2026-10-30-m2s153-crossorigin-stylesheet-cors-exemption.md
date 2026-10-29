# M2 s153 · 跨源 stylesheet 通道 CORS 豁免——上游 no-cors 语义归位 + @import 链三根连带收口

日期：2026-10-30 · 切片：html-semantics M2 · 前序：s152（静态 link 文档序交错
触发）

## 入场定刀位

s152 下一步候选①：跨源静态资源 ACAO 面影响面盘点（s151 新记——link-load-
error-events https Load/Import ×2 撞 fetch CORS check 静态臂无 ACAO，2+1 案
修复面）。

## 影响面盘点（刀法取舍）

修复面 = link-load-error-events.html 的 https 3 案（`httpsSheet()` =
`https://wpt.test:8444/...good.css` 跨端口 vs 页锚 `https://wpt.test`）——s151
假阳消除 2 案（https Import）+ 非 import 形 1F（https Load）。.https.html 变体
不在 corpus 清单（runner 执行清单 17 文件无它，基线零行实证）。

两条刀法：

1. **runner 静态臂普发 ACAO/ACEH（否决）**：全语料交叉面盘点——html/semantics
   60 文件用跨源 URL，与 stylesheet/.css 交叉仅 5 文件（本刀靶面 +
   scripting-1/the-script-element/css-module 族 4 文件）。css-module
   `crossorigin-import-without-cors.sub.html` 期望跨源 basic.css module import
   **失败**（module 默认 cors 模式无 ACAO 被拒 = 期望语义）——静态臂普发 ACAO
   翻红该负案，且篡改上游 wptserve 行为（上游静态 .css 确实无 ACAO）。
2. **shim stylesheet 通道豁免 CORS check（采纳）**：上游 `<link rel=stylesheet>`
   （无 crossorigin 属性）的样式表取回是 no-cors 请求，不做 CORS check 且跨源
   无 ACAO 照常 load（HTML spec fetching stylesheets；上游 wptserve 静态 .css
   不带 ACAO、跨端口取回成功）。shim 仿真（s150 动态臂 + s151 判定链 + s152
   静态扫描共用 `_zwStylesheetMaybeStart` → 页面 fetch(cors)）引入了上游不存在的
   check——豁免 = 归位上游语义，波及面仅 shim helper 取回链，css-module module
   通道与页面 fetch()/XHR 面零波及。

## 勘根（定向跑三轮勘根转折）

1. **旗标不可页面伪造**：豁免旗标用 part01 IIFE 模块局部 `var`（页面脚本不可
   见），不挂 globalThis——信任边界不放宽（页面代码无法借道绕 CORS check）。
   `_zwFetchMainImpl` 同步段快照进闭包，trickle/delay 等 setTimeout 异步取回臂
   同样生效（link-load-error-events slowResponse 案 trickle(d1) 走闭包快照）。
   豁免 = 跳过 cors check + 跳过 cors 头过滤（全头保留——`_zwStylesheetJudge`
   须读 x-content-type-options，cors 过滤剥非 safelisted 头即破 MIME/nosniff
   判定链）。
2. **一轮定向揭开内层 mixed 缺口（P→F 形）**：data: 外层的 @import http: 内层
   （「Import of http:// stylesheet」案）此前只有外层 href mixed 门覆盖，内层
   跨源 http 取回**碰巧由 fetch CORS check 顶住**（静态臂无 ACAO → reject →
   error ✓ 双错凑巧 Pass）——CORS 豁免后 check 消失 → http: 内层 200 取回成功 →
   load 误 fire（2 案 P→F）。s151 假阳消除同型教训重现。修复 = `_zwStylesheet
   Imports` 展开后逐层 mixed 门（http: URL + https 页 → error）。
3. **二轮定向揭开 @import 提取正则截断（4 案单根）**：`_zwStylesheetImports`
   url() 臂捕获类 `[^'")]` 排除 `)`——`url("...?pipe=status(404)")` 在 404 闭
   括号处截断（URL 丢 `)`）→ runner `wpt_pipe_status` 的 `strip_suffix(')')`
   失败 → 404 退化为 200 good.css → 期望 error 案误 load（nonexistent Import
   ×2 + slow/fast 组合 4 案「load fired when error expected」同根）。外层直
   href 案（不经提取）与 link-style-error 案（css.py URL 无括号）不受影响——
   与基线 Pass/Fail 分布完全吻合。修复 = 带引号形回引边界捕获（引号内 `)`
   合法）、裸形才排除。
4. **三层定向揭开 @import 一层递归边界（3 案）**：s151 一层递归（内层响应体
   不再展开）使双层案第二层不可见——「Import of import of nonexistent/non-CSS/
   http:// stylesheet」3 案期望 error 全误 load。s151 账本「语料无多层形态」
   勘根有误（案 5/6/9/17 即双层形态，当时未被对照）。修复 = 取回段递归化
   （`_ssVisit(url, abs, depth, onDone)`——pending 聚合递归展开，深度上限 3
   兜底环）。案 9 基线 Pass 机制未完全考古（正则截断 + 一层边界的某凑巧组合），
   现状 Fail 期望 error 明确、递归修复语义正确——归档勘根疑点。

## 施工（shim 2 文件净 +42/−13）

1. **part01 `_zwStylesheetChannelDepth` 旗标 + `_zwFetchApplyFilteredResponse`
   第 6 参**：IIFE 局部旗标变量；check 臂与 cors 头过滤臂各前置 `!_zwSsChannel`
   门；`_zwFetchMainImpl` credentials 解析后同步段快照；finishFetch/finishHop/
   settleFetch 三个调用点透传。FIXME(compat)：crossorigin 属性形态 stylesheet
   仍应走 CORS check（语料无该形态，未分型）。
2. **part03 `_zwStylesheetMaybeStart` 取回段递归化**：`_ssVisit` 闭包函数——
   逐层 mixed 门（同步回调标 failed 先于微任务 load 子归零，聚合 error 优先）+
   fetch 调用窗旗标 + judge → text() → `_zwStylesheetImports` 递归展开（深度
   3 上限）+ pending 聚合。`_zwStylesheetImports` 正则带引号臂回引边界修复。

## 验证

- **语法**：shim 规范序拼接 node --check SYNTAX_OK 110270 行（s152 110217，
  净 +53 后重构 −5）。Rust 面零改动（clippy/fmt 零波及，本轮未跑）。
- **定向 the-link-element 全目录（对 s152 基线案级 join）**：**7 案 F→P、
  LOST 0、零 P→F**——https 3 案（主修复面）+ nonexistent Import ×2 + slow/fast
  ×2（正则截断单根）+ link-load-error-events 全文件 17/17（基线 11P/6F）；
  GAINED 1（link-multiple media 案——s151/s152 在册「全量负载少报、定向稳定
  P」flake 形态，非本刀位移）。s152 修复面（fired-before-scripting 2 案）与
  s151 面（link-style-error 74 案）全保持。
- **全量案级 join（对 s152 /tmp/zw-s152-full.log，(key,state) 对 sort -u
  去重）**：8080 对 vs 8081 对（7186P/806F/88T vs 7194P/799F/88T）——**真修复
  7 案 F→P**（link-load-error-events 全族）+ **P→F 0、Timeout 位移 0、LOST 0**
  + GAINED 1（link-multiple media 在册少报键复现 Pass）。数字配平 ΔP=+8
  （7 F→P + 1 GAINED）/ΔF=−7。全语料 fetch API/XHR 面零位移 = 页面 CORS
  check 语义保持实证。
- **css-module 影响面修正**：`crossorigin-import-without-cors.sub.html` 位于
  resources/ 子目录、非 corpus 独立用例——不在 runner 度量面（基线/本轮均零
  行实证）。刀法 A' 否决论据修正为「上游 wptserve 行为篡改 + fetch 负语义面
  风险」，刀法 C 取舍不变（语义归位 + 波及面收敛于 helper 通道）。css-module
  顶层文件（charset 等）状态零位移。

## 挂账（本切片更新）

- **收口：跨源静态资源 ACAO 面**——s151 新记 2+1 案全收口；shim stylesheet
  通道 CORS check/头过滤豁免 + 内层 mixed 门补齐。FIXME 在册：crossorigin
  属性形态未分型（语料无）。
- **收口：@import 内层链残余**——正则截断 4 案 + 一层递归边界 3 案收口
  （「@import 多层递归链」挂账销账；环检测 FIXME 在册）。
- **承袭不变**：adopt/innerHTML 身份双表示深水面（document-without-browsing-
  context 2F）；LinkStyle sheet 属性面 1F；rel=preload fetch 面（link-multiple
  案 5 + 案 6 页级 10s 时序必然）；stylesheet-media-change-no-fetch 1F；meta
  long × corpus 通道放宽失效；details 族 lib flake；promise_test 串行启动面；
  待决三项（css/text 语料资源、green-2x2.png、parser form pointer 缓行）须
  用户拍板。
- **勘根疑点归档**：案 9（Import of import of non-CSS）基线 Pass 机制未完全
  考古——正则截断 + 一层边界的某凑巧组合使 error 误判方向相反；递归修复后
  语义正确（期望 error 真达成），疑点无行为残留。

## 下一步（s154 候选）

①adopt/innerHTML 身份双表示深水面（document-without-browsing-context 2F——
Create a document, adopt the node + Create a stylesheet in innerHTML document，
expected "1" but got "0"）；②rel=preload fetch 面（link-multiple 案 5 + 缓存
复用）；③LinkStyle sheet 属性面 1F；④link-load-event document load 阻塞面
（Check if the stylesheet's load event blocks the document load event 1F）。
