# R317 Evidence — doctype 品牌化/元数据惰性读 + ParentNode 导航族 `in` 可见性 + 元素子 getter 融合路由（dom/nodes 六文件收口）

**日期**: 2026-08-28
**切片**: M4——R317(a) 剩余 Fail 聚类轻量修复（六文件 +12 subtest）
**改动面**: `part06.js`（doctype IIFE 元数据惰性 getter + childNodes 前导注释融合）+ `part05.js`（has trap 白名单扩 ParentNode/元素导航族）+ `part04.js`（childElementCount/firstElementChild/lastElementChild 的 sel 分支走融合视图）+ `js_dom_bridge.rs`/`callbacks.rs`（`__zw_doc_comments`/`__zw_doc_doctype_json` 两回调）+ `part24.rs`（r317 回归测试）

## 一、四个独立缺口（WPT 残余聚类 → 逐一修复）

1. **主文档 doctype 无 DocumentType 品牌**（`Document-doctype` "Doctype should be a
   DocumentType"）：R128 的 `setPrototypeOf(dt, DocumentType.prototype)` 只落在了
   `implementation.createDocumentType` 产物上；`document.doctype` 的 IIFE 字面量漏配 →
   `instanceof DocumentType` 恒 false。修：IIFE 返回前接线（同 R128 模式）。
2. **doctype 元数据硬编码**（`DocumentType-literal` publicId 期望 "STAFF" 得 ""）：
   `<!DOCTYPE html PUBLIC "STAFF" "staffNS.dtd">` 的真实元数据在 host 解析树
   （`DocumentTypeData`），JS 静态字面量恒空。修：host `__zw_doc_doctype_json` 回调 +
   dt 的 name/publicId/systemId 改 **getter + 一次性缓存**——关键时序坑：IIFE 在
   `execute(shim)` 时求值，而回调注册在其后，快照式读取恒得默认值（首版踩中，探针
   `dtpub=,,html` 实证），惰性化后首次属性访问时回调已就绪。
3. **ParentNode/元素导航族 `in` 不可见**（`Element-childElementCount`
   `"childElementCount" in parentEl` 恒 false）：get trap 提供的 accessor 面
   （children/firstElementChild/lastElementChild/childElementCount/previous/
   nextElementSibling）在 has trap 白名单缺席（target 无 own key）。修：R129 同款白名单
   扩展。
4. **sel 父的元素子 getter 读 stale 快照**（`Element-childElementCount-dynamic-add`
   append 后 count 期望 2 得 1）：childElementCount/firstElementChild/lastElementChild
   的 sel 分支直读 host `__zw_element_children`（快照，同 turn append 不可见）。修：
   优先走 `_childNodeList(sel, null)` 融合视图（含 pending overlay，R140 同路径）过滤
   元素子；融合视图全空时回落 host 快照（零行为漂移）。**连带**：dynamic-remove 同修、
   `Element-firstElementChild-namespace` 转绿。

## 二、附加语义修复：document.childNodes 前导注释

spec 解析树中 doctype/文档元素之前的 comment 是 document 子节点（真浏览器
`document.childNodes = [Comment, Doctype, html]`）。旧合成 `[doctype, html]` 使
`Document-doctype` 的 `childNodes[1] === document.doctype` 断言失败（期望位 1 得 0）。
修：host `__zw_doc_comments` 读解析树根级 Comment（探测缓存防每读一往返），JS 视图合成
`[comments…, dt, html]`——单测 `docKids=8,10,1` 与真浏览器一致。

## 三、A/B

| 套件/文件 | R316 基线 | R317 | Δ |
|---|---|---|---|
| Document-doctype / DocumentType-literal | 1P/1F + 0P/1F | **2P/0F + 1P/0F** | +2 |
| Element-childElementCount 族 ×3 | 1P/3F | **4P/0F** | +3 |
| Element-firstElementChild-namespace | 0P/1F | **1P/0F** | +1 |
| TreeWalker.html / NodeIterator.html | 761/766 P | 766/766 P | TreeWalker +5（childNodes 注释面连带解锁）|
| **全量 dom sweep** | 54128P/67F/25T | **54138P/61F/21T** | **+10P/-6F/-4T，Fail set 恰 -6 零新增** |
| engine --lib（v8 / quickjs）| 2454/1460 | **2455/1460** | +1（r317 回归）|
| webview 658 / integration 781 / e2e 20 / lit 21 | — | 全绿 | 持平 |
| fmt / clippy（v8 guarded + quickjs）| — | 干净/0 警告 | — |

## 四、域状态

dom/nodes 剩余 Fail 全部为既存备档（realm/adoption 深结构族 4 + HTMLCollection live
边缘 3 + MO 备档 4 + HTMLNess 1 + 探针自留件）。本轮六项全部属于「此前未归档的新修复面」。

## 五、教训

1. **一次性求值点不能读回调**：shim 加载期执行的 IIFE（document.doctype 元数据）无法
   访问 register_dom_callbacks 之后才注册的 `__zw_*` 回调——快照式读取恒默认值且无报错。
   惰性 getter + 首访问缓存是标准解。
2. **has trap 白名单与 get trap 方法面要同步维护**：每给 get trap 加属性分支，同轮核对
   has 白名单（`prop in el` 是独立语义面，R129/R184/R317 三次同型缺口）。
3. **聚类的连带面**：元素子 getter 的融合路由为 childElementCount 而做，连带修复
   firstElementChild-namespace 与 dynamic-remove——同一底层路径的文件应一起回归验证。
