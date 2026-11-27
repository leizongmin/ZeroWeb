# s10k72 刀60：detached 文档工厂 createTextNode ownerDocument own 槽 defineProperty

- 日期：2026-11-28（UTC 23:37 发射）
- 基线：chain71（刀59 后 dom 失败行 229＋Timeout 10；bin d22d10dd9ad8＝172bf3d4e）
- 刀体：9750ed392（直推；part03.js +17/−1，单点 createTextNode 工厂方法）
- 字节绑定：bin 12946e4dfc4a＝9750ed392；发射断言＋终 md5 恒等（批中零重建，rebuild 33.58s Compiling ×6 含 zero-engine+webview）
- 电池：chain72 pid 3108737，17.6min，EXIT=1 与基线同形

## 目标键（4 键，单一根因）

| # | 键 | 根因 | 修复 |
|---|----|------|------|
| 1 | Node-properties :: detachedForeignTextNode.ownerDocument | part03 `_makeDetachedDocument` 工厂表 createTextNode 用 `n.ownerDocument = doc` 赋值——被原型链 Text.prototype 的 get-only accessor（R3254-M1 挂主文档 getter）拦截：spec 赋值语义沿原型链找 setter，无 setter 非 strict 静默丢弃且 own 槽不创建 → detached doc 产物读 ownerDocument 恒走原型 getter 返主文档 | Object.defineProperty 直建 own data 槽（own 槽创建语义不查原型 setter，own 优先于原型 accessor） |
| 2 | Node-properties :: detachedXmlTextNode.ownerDocument | 同上（xmlDoc 形态同根因） | 同上 |
| 3 | Node-removeChild :: "Passing a detached text from a synthetic document to removeChild should not affect it." | 同根因下游：corpus 断言 `text.ownerDocument === doc`（foreignDoc/xmlDoc createTextNode 产物）修前恒 false → removeChild 期望不变性断言败 | 同上（removeChild 语义不变，仅前置 ownerDocument 断言翻绿） |
| 4 | Node-mutation-adoptNode :: "simple append of foreign div with text"（pre-append 段） | 同根因下游：append 前断言 ownerDocument 已是 foreignDoc | 同上（adopt 翻转语义不变） |

createElement 不受累＝Element.prototype 无 ownerDocument accessor；part02 XML 工厂独立路径 own `ownerDocument: d` 字面量本就正确。

## 判定（chain72）

- dom：失败行 229 → **225**（−4 恰 4 目标键、零新增，comm 双向；Timeout 10/10 恒等）
- san 5F／tt 32F 逐字节恒等（comm -3 diff=0）；ref rc=0 Layout 521/521＋Text 204/204 全绿
- solo（发射前）：Node-properties 726P/0F、Node-removeChild 28P/0F、Node-mutation-adoptNode 2P/0F；blast spot DOMImplementation-createDocument 517P/0F＋createHTMLDocument 15P＋Document-adoptNode 4P＋Node-cloneNode 145P 零漂移
- 探针双页（zz-k60-probe5/6，跑后已删）：probe5 定谳 foreignDoc/xmlDoc createTextNode.ownerDocument 修前===主文档、修后===创建文档；probe6 定谳 remove-and-adopt-thcrash 的 window.open() 返 null（runner 无 popup 文档管道）

## 挂账（runner 域，不扩刀面）

- remove-and-adopt-thcrash 1F＝window.open() 返 null——runner 无 popup 文档管道（同 quirks「无 iframe 文档管道」类基建面），需 runner 侧宿主桥支持，单独归因不属 shim 刀面。

## census

84,536 → 84,540（+4）。DC-3 校准线 ≥94,093，差 9,553。
