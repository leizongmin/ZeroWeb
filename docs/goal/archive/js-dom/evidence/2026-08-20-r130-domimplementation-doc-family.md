# R130 — M4 nodes：DOMImplementation doc 族 detached 语义 + crash 用例跑法（+461 净，双路径）

**日期**: 2026-08-20
**Driving WPT**: `dom/nodes/DOMImplementation-createHTMLDocument{,-with-saved-implementation,
-with-null-browsing-context-crash}.html`、`DOMImplementation-createDocument{,-with-null-
browsing-context-crash}.html`、`DOMImplementation-createDocumentType.html`、`Node-baseURI.html`、
crash 族（`Node-cloneNode-on-inactive-document-crash` / `MutationObserver-nested-crash` /
`node-appendchild-crash` / `replaceWith-document-element-crash`）、回归守卫
`dom/events/Event-dispatch-bubbles-{true,false}.html`
**账本**: `tests/wpt-runner/imported-tests.txt`（R130 条目）

## 根因（五类）

1. **createHTMLDocument title 子树缺失**（7F）：spec 步骤 4「create a title element, append
   to head」——detached doc 的 headEl 旧恒空 childNodes；title 参数 `undefined` 时**不建**
   （WPT test 2 else 分支期望 0）、`null` 时文本子 data 为 `'null'`（String 转换）。
2. **原型接线缺失**：docEl/headEl/body 是 plain object——`documentElement instanceof
   HTMLHtmlElement` 等恒 false；createDocument 产物 `Object.getPrototypeOf(doc) ===
   XMLDocument.prototype` 恒 false。
3. **createDocument 语义缺**：WebIDL 必参（namespace/qualifiedName）缺省不抛 TypeError；
   qualifiedName 非空不建 root（documentElement 恒 null 语义错）；doctype 参数非
   DocumentType 不抛。
4. **无 src iframe contentDocument 返 null**：spec「iframe 初始导航到 about:blank，
   contentDocument 是空 Document」——saved-implementation / null-browsing-context-crash
   用例取 `.implementation` 直接 TypeError。
5. **runner 对 crash 用例的 Timeout 伪失败**（8 文件）：`*-crash.html` 不引
   testharness.js（纯脚本页断言「不崩溃」）——runner 的 completion 探针依赖 harness
   全局，无 harness 时永远 `testharness completion callback was not called`。

## 修复面

| 层 | 修复 |
|----|------|
| part03 `_makeDetachedDocument` | title 子树（title 文本/元素节点 + hasChildNodes/firstChild 叶子导航面）+ docEl/head/body 原型接线 + head/body 兄弟 getter + doc 级 appendChild/insertBefore/removeChild 后 `_r130WireSiblings` + documentElement 惰性 getter |
| part03 createElementNS | 校验对齐主文档（R81 期望表：'}'/'<' 非 NameStart 合法、'0:a' 从宽、'f:o:o' 有 ns 合法、XMLNS ns 仅 xmlns） |
| part03 Node 常量 | NodeType 常量幂等补挂静态 Node + Node.prototype（**无 polyfill-chain 守卫**——native 叠加路径 native 构造器常量全缺，diag7 实证） |
| part04 | 无 src iframe contentDocument 返 `_zwMakeIframeDoc('html','')` 空文档 + `baseURI` getter（location.href 回落） |
| part06 createDocument | 必参 TypeError + doctype 类型校验 + XMLDocument.prototype 接线 + Node 常量族挂 XMLDocument.prototype + qualifiedName 非空经 createElementNS 建 root |
| part06 createHTMLDocument | contentType 先于 documentElement append 设置（惰性 getter 的 HTML 回落依赖）+ Document.prototype 保持 |
| part06 createDocumentType | 宽松校验（仅含空白或 '>' 抛 InvalidCharacterError——WPT 期望表实证） |
| part03 `_zwMEl` | A/AREA `href` IDL accessor（getter percent-encode query 非 ASCII；setter 写属性） |
| runner testharness.rs | **crash 用例支持**：无 `/resources/testharness.js` 引用的用例预置 harness 内联（插首部）+ terminal 无注册测试时按「页面脚本未抛错」PASS |

## 关键回归与修复（当轮）

- **Event-dispatch-bubbles "In new Document()" 2F 回归**：documentElement 惰性 getter 首版
  对 proxy 形态子（主文档 `documentElement.cloneNode(true)` append 进 `new Document()`）
  返回克隆 proxy——但 R112 事件面以内部 docEl 为站点（tag registry + bodyHtml 并入），
  身份脱钩使 4 站丢失。**修复**：proxy 子（`__zwSelector`/`__zwHandle` 为 string）保持
  返内部 docEl；plain-object 子（`_zwMEl` 自建 root）走首元素子。
- **native traversal foreignDoc 20F 回归**：title 文本子缺 `hasChildNodes`——oracle
  `nextNode(node)` 统一调 `node.hasChildNodes()` 直接 TypeError。**修复**：叶子导航面
  补齐（hasChildNodes/firstChild/lastChild）。
- **native createDocument 111F**：`assert_equals(doc.nodeType, Node.DOCUMENT_NODE)`——
  native 叠加路径 `globalThis.Node` 是 native 构造器，常量族全缺（diag7 探针实证
  `Node.DOCUMENT_NODE === undefined`）。**修复**：NodeType 常量无守卫幂等补挂（同
  DOCUMENT_POSITION 族 R2815 模式）。

## A/B 验证

- **Driving 用例**：createHTMLDocument 7F→**0F（13P 双路径 100%）**；createDocument
  434P/0F 双路径；createDocumentType 82P/0F；Node-baseURI 9P/0F；crash 族 8 文件
  Timeout→PASS（nodes 5 + events 2 既知 Fail + ranges 1 Unsupported 不变）。
- **dom/nodes 全量**：polyfill 7949→**8410P（+461）** fail 237→**214（零新增）**；
  native 6445→**7627P（+1182）**。
- **回归面**：events 419→420P（fail 集 diff **IDENTICAL** 零新增）；collections 49P
  （native 48→49 +1）；traversal 1589P polyfill / native 37F→**36F（还修好 TreeWalker-realm
  1F，零新增）**。
- **单测**：engine `test_dom_implementation_doc_family_r130`（title 子树/原型/惰性
  documentElement/必参校验/doctype 校验/href percent-encode 断言组）。
- `make test` 66 套件全绿（双矩阵）；fmt 无 diff；clippy 零警告。

## 剩余缺口（记深结构）

- `node-creation-realm` / `node-realm-*` 族（~30F）：跨 realm 语义需 iframe 独立
  window（R117 frame 域深结构）。
- `remove-and-adopt-thcrash` 1F：`contentWindow.document` 链（frame 域）。
- `keypress-dispatch-crash`（NotSupportedError）与 `replace-event-listener-null-
  browsing-context-crash`（contentDocument.adoptNode 缺）为既知 Fail（R130 前同状态）。
- `insertBefore-iframe-crash`：testdriver Actions 未支持（Unsupported，中性）。

## 教训

1. **「首个元素子」的 spec 语义要按子节点形态分派**——polyfill 桥里同名 API 的返回身份
   （内部站点对象 vs 克隆 proxy）决定派发/查询链是否连通；R112 的 tag-registry 派发
   以内部 docEl 为锚，身份换掉 = 链断。A/B 对照门的 events 全量 diff 是抓回此回归的
   唯一手段。
2. **plain-object 叶子节点的最小导航面**（hasChildNodes/firstChild/lastChild）是 oracle
   遍历的硬依赖——新增任何合成节点（title 文本子）都要补齐三件套，否则 traversal
   oracle 直接 TypeError。
3. **crash 用例的 WPT 跑法是「页面脚本不崩」**——没有 test() 注册；runner 若只认
   harness completion 探针，这批用例永远是 Timeout 伪失败。按 has-harness-ref 分派
   （预置 harness + terminal 零注册按 PASS 计）是 runner 侧的语义对齐，非用例豁免。
4. **native 叠加路径的常量缺口是系统性的**——native 构造器上的静态常量
   （Node.DOCUMENT_NODE 族）不随 polyfill 自建链守卫走，须独立幂等补挂（diag7 探针
   `Node.DOCUMENT_NODE === undefined` 一发定位）。
