# M2 切片 104：frameset 文档 document.body 语义——body element 按 spec 返 frameset

日期：2026-10-27。范围：`crates/engine/src/js_dom_shim/part06.js`（`document.body` getter 分派）+ `crates/webview/src/webview.rs`（1 单测）。上轮：[M2 切片 103](2026-10-27-m2s103-module-graph-failure-face.md)。

## 1. 入场定谳

- 承袭挂账「116 frameset 文档语义缺口」（s97 记账）：execution-timing/116「scheduler: adding script to head of frameset document」1F——`assert_array_equals` 期望 `['document.body: <FRAMESET>']` 实得 `<BODY>`。
- 判据机制：head 内动态插入外链 find-body.js（testlib.addScript first-in-parent），文档解析完后执行，记录 `document.body.localName`——frameset 文档（`<head>` 后 `<frameset>`，无 body）spec 语义 body element = html 元素第一个 body **或 frameset** 子元素（dom.html#dom-document-body）。
- 根因：解析器（html5ever spec 树构造）frameset 文档无 body 元素（Rust DOM 正确）；shim `document.body: _wrapSelector('body')`（part06:4900）为静态 selector proxy——恒报 BODY（幽灵元素）。
- 候选对比：063（cloneNode deep + runner 脚本记账——body 内联脚本重复执行 pending=1）与 module 尾片（import attributes / .py 执行器基建）均大体量；116 单 getter 分派定刀。

## 2. 接线

- part06 `document.body` 由静态 proxy 改 getter：`__zw_query_match('body')` 未命中且 `__zw_query_match('frameset')` 命中 → `_wrapSelector('frameset')`；否则保持 `_wrapSelector('body')`（普通文档 identity/行为零变化——proxy 按选择器缓存跨访问同对象；均无时保持 body proxy 旧兜底，不新增 null 分派）。
- webview +1 单测 `frameset_document_body_is_frameset_s104`：frameset 文档（探针脚本置 head 内）→ `document.body.localName` === 'frameset'；普通文档 → 'body'；`document.body === document.body` → true（identity 保持）。

## 3. 验证

- 单测：zero-webview lib **807/807**（+1）；zero-engine lib **3533/3533**；fmt clean；clippy 双 crate `-D warnings` clean。
- 定向：**FILTER=116 1F→1P**。
- 全子树终扫（scripting-1 口径 593 行）：**349P/192F/37T + 15 NotRun**（s103：348P/193F/37T+15N）；行级 diff **恰 1 行**（116 F→P），**零 P→非P**（comm 校验）、零旁损。
- 探针记录（取证留档）：①裸 `execute_script` 走轻量 bootstrap（`__zw_query_match` 未定义）——shim 语义取证必须走页面脚本路径；②探针脚本置于 `</html>` 后 → 解析器建隐式 body，`document.body` 合法地是 body——判据文档形状须与 116 同形（脚本在 head 内）。
- 渲染面核对：reftest harness 自有 body/frameset onload 提取（tag 无关，reftest_scripts.rs:523）；layout 读 Rust DOM 不经 shim——本轮 JS-only 面，reftest/product-smoke 未跑。

## 4. 范围与挂账

- 触碰面：part06.js ×1（body getter）+ webview.rs ×1（1 单测）；解析器/语料/runner 零变更；语料不可改维持。
- 残余（execution-timing 族 50F 另立项）：scheduler 语义族（063 cloneNode deep 的 runner 脚本记账重复执行 + pending=1 挂起为其中一页）；module 尾片余项（import attributes / .py 执行器 / 非 Error 抛出值身份 / 动态 import reject 形 / error-type-3 instantiation 优先级）承袭。
- 承袭挂账：063 Timeout、charset 族（Timeout 形）、svg-script-fragment-clone 3F、FIXME 组、接口对象基建面、testharness.rs /tmp/zw-hb*.txt 插桩来源待查。
