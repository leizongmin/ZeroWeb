# M3 documents 簇收口（2026-11-05）

## 簇规模与收敛

M1 基线 `html/dom/documents` 域 48 案 93P/139F（控制面挂账「documents 语义簇
~130F」）。本轮（含上一轮 429 中断前的 shim 工作底稿）收口到
**229P/3F**：**0 回归 + 136 F→P**（head `301e45c` 二进制逐子测试对拍）。

## 修复面（spec：HTML `#dom-document-body` / `#the-title-element-2` /
`#dom-document-nameditem`；DOM `#interface-htmlcollection`）

1. **document.body getter/setter**（part03 detached + part06 主文档）——
   strict document element（不走 HTML 回落）、HTML-ns body/frameset 子、
   setter replace/append/HierarchyRequestError/TypeError 全分支；
   Document.body.html 23 子测试全绿。
2. **document.title 三角规则**（part03 `_zwFindTitleElement`，getter/setter
   共享体 `_zwDocGetTitle/_zwDocSetTitle`，主文档/detached 同源）——WPT
   全集归纳：① root SVG-ns `svg` → SVG title 直接子；② root HTML-ns 或
   SVG-ns（raw ≠ 'svg'）→ 全树树序首个 HTML-ns title；③ 其余 root（null-ns
   XML 文档）不建立 title element（getter "" / setter no-op）。子测试
   title-01/03/06/07/09 + title-not-in-html-svg 全绿（title-07
   createHTMLDocument null→"null" 经 headEl 视图 text 兜底读）。
3. **HTMLCollection named property 完备**（part03 `_liveQueryCollection`）——
   空串守卫 + name 侧 HTML ns 门 + miss→undefined（普通属性缺失语义）+
   `has`（'length' 门 + named）+ `ownKeys`（索引 + named 名）+ `gOPD`
   （named enumerable:false——LegacyUnenumerableNamedProperties）。
   document.forms/images/links + nameditem-01..07 双向面。
4. **HTMLCollection prototype enumerable**（part05 `_zwHCPrototype`）——
   接口 prototype 上属性/操作 enumerable:true（WebIDL；document.forms
   iteration 期望 for-in 出 `["item","namedItem","length"]`）；own 层 named
   仍 enumerable:false 不受影响。
5. **document.head getter 化**（part06）——html 元素首个 head 子（live），
   head2 插入前移/赋值面（head-01/02）。
6. **getElementsByName 重写**（part06）——HTML-ns 全元素 name identical +
   live NodeList（旧 `[name="…"]` 选择器委托的 foreign-ns 命中/静态快照/
   class string 三缺口）。
7. **document named access 树内守卫**（part06 `_namedSnapshot`）——
   parentNode 爬链到 #document 判定树内（remove 后不再命中，
   nameditem-01/04/05/06/07 removed 面）。
8. **embeds/plugins [SameObject]**（part06 单例）+ **scripts 等集合 getter
   HTML-ns 快照**（`_zwDocHtmlNsColl`）。
9. **getElementsByClassName DOMString 转换**（part04/06 三处）——null→
   "null"/undefined→"undefined" 命中 class。
10. **detached 文档 readyState/cookie**（part03）——恒 'complete' +
    cookie-averse ''；**DOMParser 产物 readyState**（part02）同语义。
11. **detached 文档 body getter 两级骨架回落**（part03，2026-11-05 补）——
    strict 走查在两类合成文档上落空（探针实证）：① 浏览上下文子文档
    （`_zwMakeIframeDoc` 产物，`_zwMarkup` 印章）doc 级 html 包装 kids=0——
    iframe 内 setupRangeTests `document.body.insertBefore` 抛 null 崩，脚本
    顺序求值中断 → Range-test-iframe 加载链全断（cloneContents 族 186
    NotRun、insertNode/surroundContents 各 1840F，并行流修，
    `_zwMarkup` 印章臂回落骨架 body）；② `document.cloneNode(true)` 产物
    出生 childNodes=0（R112 只注 innerHTML 快照不落 docEl）——Event-dispatch
    clone 派发/R112 注入拿不到 body（`__zwDetRootRemoved` 摘根标记区分
    「Childless document」spec null 面后骨架回落，本流修）。回落臂经
    wip11 定向复验 229P/3F 不动，html 通道 A/B 0 回归。

## 性能（title getter 无缓存全树走的教训）

head 快道先行（head 为 HTML root 首个元素子，head 内 title ≡ 全树树序首，
等价）+ **pendingLen 戳缓存**（仅主文档；键 = doc + 宿主视图戳 + pending
长度；缓存**元素引用**不缓存文本——结构性变化变戳/变 pendingLen 失效，
文本变化实时读元素）。探针 3k 树 200 次 title 读：5761ms → 37ms。两次
教训已写进注释：① 戳不反映 pending 动态子（title-09 detached 回归 3F——
detached 直走 uncached）；② R130 head 槽恒含构造空 title（head 快道仅
root HTML-ns 启用，title-09 t5 setter 写穿修复）。换代失效挂在
`_zw_reset_pending_state`（`_doc_title` 滞留前车之鉴同点）。

## dom 通道连带修复（documents 簇工作底稿触发，逐页对拍定位）

1. **document named-proxy named-miss 二次方扫描**（part06 get/has trap）——
   named access 让 `document.<missing>` 每读走 `_namedSnapshot` 全树扫描
   （~100µs/次）；R262/R350/R351 的 live-range 爬链键读（`document.__zw*`
   内部印记）每行数百次触达 document 包装 → Range-mutations 族 90s 截断
   加剧（分相探针 remMs 732ms/60 行实证）。修复：`__zw` 前缀门跳过 named
   查找（内部命名空间非 nameditem 面）。remMs 732→26.1ms/60 行（28 倍），
   growth B/C 相回 head 持平。
2. **iframe 文档 body getter 回落**（part03 get body，`_zwMarkup` 印章臂，
   修复面 11① 的落地）——严格扫描对 iframe 子文档恒 null（内容在宿主解析
   树，docEl 包装层 childNodes 不物化 body 子）→ iframe 内 common.js:25
   `document.body.insertBefore` 抛 null、**脚本顺序求值中断**（加载期异常
   截断后续顶层声明：ownerDocument/rangeFromEndpoints 缺失而 setupRangeTests
   在）→ Aryeh iframe 页全断（cloneContents 186 NotRun、insertNode/
   surroundContents 1840F、3×90s 截断空转）。印章文档回落静态合成 body
   （修复前语义），detached 文档 strict 面不变。cloneContents 187/187、
   insertNode 1840/1840 恢复，ranges 段 654.78s→469.33s。
3. **document.remove 缺席**（part06 named-proxy get/has trap）——原型接线
   （setPrototypeOf → HTMLDocument.prototype →…→ Node.prototype 含 R238
   泛型 remove）使 `document.remove` 泄漏为 function（head 为 undefined）。
   spec ChildNode mixin 挂 Element/CharacterData/DocumentType，**不含
   Document**（prepend/append/replaceChildren 属 ParentNode mixin，R152 表
   已豁免故不受影响）。get/has 双 trap 拦截真缺席——仅 get 返 undefined
   不够，`'remove' in document` 为真时 with 的 HasBinding 仍吞裸名。
   remove-unscopable 1F → 6/6 Pass。

## 残余挂账（3F，均 runner 管道/共享查询层，不属本簇 shim 面）

1. `document.embeds-document.plugins-01` Two plugins——pending 快照树序
   （insertBefore 后 pending 段尾接非树序；共享查询路径 R120 形态）。
2. `document.scripts` live——runner 注入的 harness script 不进 DOM 视图
   （初始 length 2 ≠ 3）。
3. `document-lastModified`——runner 无 HTTP Last-Modified header 注入
   （file:// mtime 语义）。

## 验证

- 定向（html/dom/documents 48 案）：head 93P/139F → **229P/3F**，
  0 回归 + 136 F→P（逐子测试对拍，test-guard 包裹；终版二进制定向复验
  229P/3F 不动）。
- 全通道 A/B（test-guard 包裹，back-to-back 配对）：

  | 通道 | head `301e45c` | 本轮终版 | 结论 |
  |---|---|---|---|
  | testharness-dom 385 页 | 832.40s / 56274P / 323F / 15T | 786.61s / 56301P / 324F / 15T | 唯一回归 remove-unscopable 1F（终版 6/6 已修）；0 既有 Pass→Fail；+27P 为看门狗边界页多记录子测试 |
  | testharness-html-syntax 273 页 | 61671P / 697F / 7T | 61807P / 561F / 7T | **136 F→P 全落 documents 簇**（Document.body/nameditem-01..07/title-03/07/09/images/forms/no-shadowing）；0 回归、0 新增/丢失键 |

- dom 通道注记：ab7 配对页面集含 battery 遗留探针页 `__m3probe2.html`
  （已清理，1P/2F，不涉语义面）；终版复验跑（836s）与 m3clone battery
  并行重叠，其墙钟不作 perf 证据——语义面与 ab7 一致（仅 Range-mutations
  五页截断深度随执行速度波动：dP -46/-29/-14/-11/-6，Fail 列恒 0；
  remove-unscopable 6/6、探针页 3P 确认）。
- 机时漂移：同日 head ranges 段 442.24s（晨）→ 487.04s（午后），~10%；
  dom 段墙钟对比须同窗配对读数，跨窗绝对值不可比。
- 性能探针见上（title 读 169 倍；remMs 28 倍）。
