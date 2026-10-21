# M3-S2 子文档 location 合流 + 合成树 innerText（2026-10-21）

M3 第二切片：S1 之后 the-iframe-element 域内 Fail 池的 srcdoc 深语义簇——
**16P→25P→27P（+2 案翻绿）**，srcdoc_change_hash Fail→Timeout 为推进位移
（见下）。三处 shim 变更，零 runner 产品面变更。

## 实现

- **doc.location ↔ win.location 同一对象**（part05 `_zwMakeIframeDoc`）：spec
  [dom-document-location](https://html.spec.whatwg.org/multipage/nav-history-apis.html#dom-document-location)
  ——browsing context 内 document.location 与 window.location 返回同一 Location。
  `_zwDocLoc` 扩全 URL 解析读面（protocol/host/hostname/pathname/search/hash/
  origin，同 iframeLocation 的 URL 构造器解析）；doc.location getter 改惰性——
  win 物化后返回 win.location（identity 全等），物化前 `_zwDocLoc` 服务
  （srcdoc_process_attributes / iframe_remove_src 只读 contentDocument.location
  .protocol 不触 contentWindow——S1 的 href-only 单例返 undefined 即此簇根因；
  srcdoc 档案 'about:'、blob: 档案 'blob:'）。成熟度门前 toString 面维持
  （content_document_changes_only_after_load_matures）。
- **子窗 location.hash 写侧**（part05 `iframeLocation`）：同文档片段导航（spec
  [the-location-interface](https://html.spec.whatwg.org/multipage/nav-history-apis.html#the-location-interface)
  hash setter）——'#' 前缀可省、空值剥 hash、同值 no-op 不派；`iframeSetCurrent
  HistoryUrl` 更新 URL 槽 + 子会话条目 push（cursor 后截断 + push）；异步
  （_defer）派 HashChangeEvent（`__zwTrusted` UA 口、oldURL/newURL）到
  `win.onhashchange`。srcdoc_change_hash 前两步断言面。父窗 hash 通道
  （part02 `_setLocationHash`）不动——子窗无 Navigation API 面。
  `FIXME(popstate)`：spec 片段导航还同步派 popstate（子窗 popstate 事件面未建）。
- **合成树 body.innerText**（part03 `_makeDetachedDocument` body 视图）：R3260
  近似语义（≈ textContent getter/setter 委托）——spec
  [the-innertext-idl-attribute](https://html.spec.whatwg.org/#the-innertext-idl-attribute)
  的 CSS 感知步在无渲染信息的合成树上不可达。srcdoc-anchor /
  srcdoc-attribute-reset / srcdoc_change_hash 经
  `contentDocument.body.innerText` 断言（textContent 已通而 innerText undefined
  即此簇根因）。

## 验证

- 域内：`make testharness-navigation FILTER=the-iframe-element`
  （TIME_LIMIT=2700）**27P/32F/28T**；对 S1 账本逐案 diff **恰 2 案翻绿**
  （srcdoc-anchor、srcdoc-attribute-reset）+ 1 案 Fail→Timeout 推进位移：
  srcdoc_change_hash——innerText 门过后推进到 step 3/4 `history.back()`/
  `forward()`，卡**联合 iframe session history** 缺口（父窗 history 列表不含
  子同文档条目——联合史最小面 = 下一切片候选，非行为回归：无 Pass 丢失、
  失败点从 step 0 深入到 step 3）。blob:-src 档案经 host fetch 同步契约
  （R115）+ X-Zero-Final-URL（R360）记账。
- 集成测试：`js_dom_bridge_tests/part63.rs` 三案（location 同一对象 + protocol
  解析面 about:/blob: + 未物化 toString 维持、子 hash 写侧 URL/会话条目/
  异步 trusted hashchange + 同值 no-op、合成 body innerText 读写 + srcdoc 移除
  后空串）3/3 过；全量 `cargo test -p zero-engine --lib` **3353/3353**；
  `make test` 76 suites **22,250P/0F/199I**。
- fmt 干净；clippy -p zero-engine --all-targets -D warnings 零 warning。
- reftest 未跑：变更面（子文档 location/合成树 innerText）零布局/绘制触达
  （S1 同理由）。

## 全量 corpus 零回归 diff

全量 `testharness-navigation`（TIME_LIMIT=2700，对本切片工作树）459P/69F/52T
（580 案行）对 S1 全量（457P/72F/51T）逐路径 diff：428 路径中恰 3 路径变化 =
域内 +2 翻绿 + srcdoc_change_hash Fail→Timeout 推进位移（同上记），**其余
425 路径逐案恒等**（含 navigation-api 5 案维持既有 Fail 面）。位移案无 Pass
丢失、失败点从 step 0（innerText 门）深入到 step 3/4（父窗 history.back/
forward 的联合 session history 缺口）——联合史最小面列为下一切片候选。

原始输出：[2026-10-21-m3-s2-child-location-innertext.txt](2026-10-21-m3-s2-child-location-innertext.txt)
（全量）、[2026-10-21-m3-s2-iframe-filtered.txt](2026-10-21-m3-s2-iframe-filtered.txt)
（域内）。
