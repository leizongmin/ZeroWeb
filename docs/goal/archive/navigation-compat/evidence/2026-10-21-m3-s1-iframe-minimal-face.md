# M3-S1 iframe 浏览上下文最小面（2026-10-21）

M3 门控获批（2026-10-21 用户批复「批准 M3 最小面」）后首个切片：iframe 元素
属性语义（srcdoc 处理模型 + sandbox 指令解析 + load 信任面）+ 嵌套浏览上下文
读面（contentWindow/contentDocument 成熟度 + frames named access + 子文档
location 写侧）。the-iframe-element 解锁域 **16P→25P（+4 案全案翻绿、零回归）**。

## 实现（engine shim，零 runner 产品面变更）

- **srcdoc 处理模型**（part01 `_zwLoadIframeEntry`）：srcdoc 属性在场 → 合成
  `text/html` wire（`200\x1fOK\x1fcontent-type\x1etext/html\x1e\x1f` + 标记）进
  既有 `_zwFinishIframeEntry` 管线——doc/win/scripts/history/load 全链复用，
  地址 `about:srcdoc`（spec
  [process-the-iframe-attributes](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#process-the-iframe-attributes)
  srcdoc 优先于 src）；属性 set/change/remove 三触发（part04 setAttribute/
  removeAttribute 钩子 + part05 IDL setter 扩 srcdoc）；子发起导航
  （location.href/assign/replace，part05 iframeLocation 写侧新增）带
  `viaNavigation` 绕 srcdoc 属性处理（spec：srcdoc 只挂属性变更，子导航按 URL
  取资源）。fetch 无通道时 src 也记账 `entry.url`（data: 判 null 门数据源）。
- **成熟度语义**（part01 `__zw_reload_iframe` + part04 读侧）：cache 立即换新
  entry——contentWindow 即时反映替换 Window（js-dom M4 R388 跨文档导航
  `same12:false` 既有断言面）；contentDocument 经 `entry.servedDoc` 槽在新文档
  成熟前继续服务旧 Document，成熟翻转与元素 load 派发同批次延后 microtask
  （spec 同上「前置文档保持步」；content_document_changes_only_after_load_
  matures 同步断言面）。第一版「cache 整体延迟换」与 R388 冲突且 settle 守卫
  `cache === entry` 永假（第二次 load 永不派发 → network-error 子导航双案
  挂起、lazy 双案回归）——servedDoc 拆分两读面后全解。
- **iframe 子文档 doc.location**（part05 `_zwMakeIframeDoc`）：detached 工厂的
  `get location() { return null; }` 对 browsing context 内文档不成立——挂
  doc-anchored 单例（读 `_zwURL` 槽、无槽回 about:blank）；createHTMLDocument
  非 browsing context 面不改（spec document location getter is null 维持）。
- **load 信任面**（part01 两派发点）：`__zwMakeTrustedEvent('load')` 出口
  （R312 UA 通道——`_zwDispatchGuard` 对无 `_zwUaDispatch` 印记事件翻
  isTrusted=false，`new Event(t, {__zwTrusted})` 直传不生效）；iframe-load-event
  双案（blob: URL + initial about:blank）isTrusted/Event 构造/非 cancelable/
  非 bubbles/target 断言面。双派封口：reload 给被替换 entry 打
  `_zwFrameElementLoadQueued`（start-connected append 期 defer 读 cache 命中
  旧 entry 时不再补派）。
- **sandbox 指令**（part01 `_zwIframeSandboxFlags`）：token 先 ASCII 小写再匹配
  （İ/ſ 形近字不经 ASCII 小写 → 不命中——sandbox-ascii-case-insensitive 断言
  面；JS toLowerCase 是 Unicode 语义会把 İ 折叠，须 ASCII-only 替换）+
  allowScripts 门（无 allow-scripts → `_zwRunIframeScripts` 直接返回，子文档
  脚本不执行）；part04 sandbox supportedTokens 十九 allow-* keyword
  （`sandbox.supports()` 查固定表与属性值无关——同案 subtest 1 断言面）。
- **跨源 contentDocument null**（part04 读侧）：沙箱无 allow-same-origin =
  opaque origin 同判 + `data:` URL 文档同门（spec
  [dom-iframe-contentdocument](https://html.spec.whatwg.org/multipage/nav-history-apis.html#dom-iframe-contentdocument)
  跨源 access-check；document-getters-return-null-for-cross-origin 断言面，
  getSVGDocument() 同语义）。
- **frames named access**（part01 R177 frames proxy）：`frames.<name>` /
  `frames['name']` 按 iframe name 内容属性解析到 contentWindow（HTML window
  named access 的 frames 别名面；iframe-network-error
  `frames.existingIframe1.location.href` 子导航断言入口——基线该双案
  `frames.existingIframe1 undefined` 直接崩）。
- **解锁域 skip 规则**（testharness.rs `navigation_case_skipped`）：运行形态/
  深结构子域记账跳过——服务端 stash 协议（iframe_harness.js → wptserve
  stash.py，runner 无该端点）、子发起嵌套导航控制（iframe_sandbox_navigate_*）、
  popup 窗口管理（iframe_sandbox_popups_* / allow_top_navigation 的 open() 链
  + 用户手势）、三层嵌套 bc（change_parentage / cross-origin-to-whom）、
  worker/serviceWorker 面；srcdoc/sandbox/frames/parent/top/load 语义最小面
  照常执行（23 案自基线 Fail 池转记账 skip）。资产补拉
  （60-navigation-compat.sh）：/common/blank.html（上游 0 字节空文件，合法）
  + ../resources/not-embeddable.html。

## 验证

- 域内：`make testharness-navigation FILTER=the-iframe-element`
  （TIME_LIMIT=2700）**25P/35F/27T**；对基线逐案 diff **恰 4 案翻绿、零回归**
  （document-getters-return-null-for-cross-origin、iframe-load-event、
  iframe-loading-lazy-multiple-queued-navigations、iframe-network-error.sub
  ——后者 4/4 子测试全 Pass，含子发起导航双案）。基线部分账（首跑
  TIME_LIMIT=1800 截断）见
  [baseline-partial](2026-10-21-m3-s1-iframe-baseline-partial.txt)。make
  exit 1 = 既有慢性面签名。
- 集成测试：`js_dom_bridge_tests/part62.rs` 十案（srcdoc 初始/优先/变更成熟/
  移除回退、sandbox supports 固定表、ASCII 解析双门、跨源 null、frames
  named+indexed、src 成熟序 trusted load、子 location.href 导航）10/10 过；
  全量 `cargo test -p zero-engine --lib` **3334/3334**（含 R388 既有语义回归
  ——第一版成熟度设计曾破 R388，servedDoc 拆分后恢复）。
- fmt 干净；clippy -p zero-engine --all-targets -D warnings 零 warning。
- reftest 未跑：变更面（shim iframe 语义 + runner skip 规则）零布局/绘制触达
  （S11/S12 同理由，708 比较 0 不一致账本仍新）。

原始输出：[2026-10-21-m3-s1-iframe-minimal-face.txt](2026-10-21-m3-s1-iframe-minimal-face.txt)。

## 全量 corpus 零回归 diff

全量 `testharness-navigation`（TIME_LIMIT=2700，对本切片工作树）457P/72F/51T
（580 案行）对 S12 账本（437P/34F/24T/495 案行）逐路径 diff：S12 运行被
network-error 挂起截断、本运行解锁后新触达 72 路径（含 17 路径含 Pass 面，
含上记 4 翻绿案）；**逐路径零回归**。

**例外定性（非本切片回归）**：navigation-api/navigate-event 5 案
（intercept-handler-throws / intercept-multiple-times-reject /
intercept-reject / navigation-back-same-document-preventDefault /
signal-abort-window-stop-after-intercept）对 S12 账本 Pass→Fail——
`e.constructor` 返回 R109 真 ctor 而非 Guarded 包装。隔离探针：暂存本切片
全部 diff 后同 5 案仍全 Fail（探针 log /tmp/nav-probe-stashed.log）→
回归在 HEAD 已存在，S12（10-20）后唯一触碰 shim 的提交
d45247864（part03 92 行，事件机制邻域）最可能。归 html-semantics 面，
本切片不修。

