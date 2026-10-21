# M3-S7 子文档嵌套 iframe 自主载入 + 子文档 contentDocument 面（2026-10-21）

M3 第七切片：S6 FIXME 记账的第一缺口（子文档元素 contentDocument 面缺）
与第二缺口（script-disabled 文档 lazy 豁免）同轮收口。零既有测试改写；
runner skip 规则零改动（案本就运行——S6 位移后的 Fail）。

## 根因

- 子文档合成树（detached doc + `_zwMEl` 轻量解析节点）的 IFRAME 元素无
  contentDocument/contentWindow 面——读得 undefined。R115 面只接主文档
  元素 proxy（part04），轻量节点无 sel/handle，S1~S6 从未触达。
- 子文档树内 iframe 无人启航——`_zwStartConnectedIframe`（动态插入）与
  S6 `__zwStartParserIframes`（主文档 window load 扫描）都只扫主文档；
  finish 管线对文档内嵌套 iframe 无递归步。
- WPT `iframe-loading-lazy-in-script-disabled-iframe` 断言链三层：外层
  sandbox="allow-same-origin"（无 allow-scripts）→ 中层文档 loading=lazy
  子 iframe → 孙代 subframe.html；断言 `中层 contentDocument.querySelector
  ('iframe').contentDocument.body.textContent === 'Subframe'`。spec
  lazy loading 仅 script-enabled 文档适用——script-disabled 文档的 lazy
  不豁免即不可达（S6 记为 Timeout→Fail 位移）。

## 实现

- **嵌套自主载入**（part01 `_zwStartNestedIframeLoads` + `_zwResolveNestedSrc`，
  挂点 `_zwFinishIframeEntry` 内 URL 槽之后、win 构建之前）：文档解析完成
  即扫描 `entry.doc.querySelectorAll('iframe')`，src/srcdoc 者走共享 finish
  管线（doc/win/scripts/history/load 全链）——srcdoc 合成 text/html wire、
  src 经同步 `__zw_fetch`。递归成立（孙代文档同轮扫描；`_zwNestedDepth` 印
  深度界 5，corpus 最深 3 层）。合成 frameKey（`'::n'` 序号）不入宿主
  注册表；sandbox 旗标整链继承（spec：flags propagate to all descendant
  browsing contexts）——脚本门随 `entry.flags` 在 `_zwRunIframeScripts`
  既有门生效。
- **跨层 src 解析**（`_zwResolveNestedSrc`）：相对 src 按**所在文档** URL
  解析（spec：parse a url against the node document）——主文档版
  `_zwResolveIframeSrc` 恒 globalThis.location base，嵌套 base 不同。
- **lazy 门**：`loading=lazy` 仅 script-enabled 节点文档适用——
  script-disabled（flags.sandboxed && !allowScripts）不跳过、立即载；
  script-enabled 链维持 S6 惰性记账面。
- **子文档 contentDocument/contentWindow 面**（part03 `_zwMEl` IFRAME 分支）：
  轻量解析节点挂惰性 getter 读嵌套扫描盖的 `__zwNestedEntry` 印；未扫描/
  未载 → null（R115「未加载完成可为 null」同语义）；跨源门同 R115
  （沙箱链无 allow-same-origin / data: URL → null）。
- part63 新增 `m3s7_nested_iframe_loads_and_content_document_face`——三层
  链（script-disabled 文档 lazy 照载 + body 文本可达 / script-enabled 链
  lazy 惰性 contentDocument null / 无 loading 常载）+ fetch 日志跨层解析
  断言（subframe 恰两载）。

## FIXME 记账销项与余项

- S6 记账①「子文档元素 contentDocument 面缺」——本轮销项（嵌套已载案面）。
- S6 记账②「script-disabled 文档 lazy 豁免」——本轮销项（spec 方向相反：
  script-disabled **不**豁免 lazy——立即载）。
- 余项：嵌套子文档的元素 load 事件/动态插入嵌套 iframe（`innerHTML` 写入
  后的连接变更路径）未触——corpus 零消费，后续按 F 池密度挂账。

## 验证

- 域内：the-iframe-element 36P/32F/22T→**37P/31F/22T**。
- 全量 corpus：`make testharness-navigation` **473P/68F/43T**（584 案行）
  对 S6 全量（472P/69F/43T）逐路径 diff **恰 1 路径变化**：
  iframe-loading-lazy-in-script-disabled-iframe Fail→Pass（单 subtest
  完整过）；**其余 583 路径逐行恒等零回归**。
- 引擎库：`cargo test -p zero-engine --lib` **3376/3376**（含 m3s7 新案；
  3360→3376 含 rebase 远端新案的增量）。
- 跨套件回归臂：`make testharness-webmessaging` **132P/3F** 对 S6 账本
  135 行逐行恒等。
- fmt 干净；clippy workspace --all-targets -D warnings 零 warning。
- reftest：切片内未跑（变更面零布局/绘制触达，S1~S6 同理由）；M3 收口
  复验全量补跑 **712/712 零不一致**（见
  [M3 收口评估](2026-10-21-m3-closeout-assessment.md)）。

原始输出：[2026-10-21-m3-s7-nested-iframe-contentdocument-family.txt](2026-10-21-m3-s7-nested-iframe-contentdocument-family.txt)
（关键行）、[2026-10-21-m3-s7-nested-iframe-contentdocument-full.txt](2026-10-21-m3-s7-nested-iframe-contentdocument-full.txt)
（全量 584 案行）、[2026-10-21-m3-s7-webmessaging-crosscheck.txt](2026-10-21-m3-s7-webmessaging-crosscheck.txt)
（跨套件回归臂）。
