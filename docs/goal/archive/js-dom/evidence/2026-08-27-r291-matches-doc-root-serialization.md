# R291 Evidence — matches 的 Document 根序列化 + 键回落（部分进展，靶簇归因收窄至 wrapper identity）

**日期**: 2026-08-27
**切片**: M4——R291(a) body type selector 匹配域（6F 簇，**未全解**——归因收窄）
**改动面**: `part03.js`（`_zwMOuterHtml` 的 Document 根合成序列化 + 无 attributes 视图的 id/class 反射 + matches 的 tag+id 主键/documentElement tag-only）

## 一、修复内容（增量，严格新增路径）

1. **`_zwMOuterHtml` 的 Document 根**：nodeType 9 时旧返 ''（查询源空）——
   合成 `<html attrs>{head}{body}</html>`（head/body 从 doc 视图序列化；
   attrs 用 R159 `_r159HtmlAttrs`）。
2. **无 attributes 视图的 id/class 反射**：iframe 子文档 body 等轻量视图无
   `attributes` 数组但有 `getAttribute`——反射进序列化使 self/cand 键可对齐。
3. **matches 键比较重构**：tag+id 主键（唯一候选直接命中）+ outer 决胜
   （多候选才比）；documentElement 的 tag-only 形态（工厂 docEl 无属性反射面）。

## 二、验证（单跑 A/B）

- sandbox 直测：`iframeDoc.body.matches('body'/'#body')` **false→true**、
  `iframeDoc.head.matches('head')` **false→true**（body/head 视图对象直测）。
- WPT 套件：**无数字变化**（Element-matches 674P/1F、webkitMatchesSelector
  668P/1F、querySelector-All 1971P/4F 全同 R290）——根因见下。
- dom 全量：54045P→54047P（+2 全为 flaky `.sub` 超时恢复，F 集**逐条相同**，
  set-diff 0/0）。
- engine 2429 全绿 + fmt/clippy 干净 + closest/contains/escapes/scope/
  MO-childList 全持平（零回归）。

## 三、靶簇归因收窄（未解根因：querySelector 产物 wrapper identity）

WPT Element-matches 的失败链（probe 实证逐步收窄）：
1. 测试经 `root.querySelector('#body')` 取元素——**iframe 子文档的
   querySelector 返 `_zwParseEl` wrapper**（probe：`q === doc.body` false、
   `q.parentNode` null——JSON 往返产物非视图对象）。
2. wrapper 上调 `.matches('body')`：wrapper 无 `_zwRootHtml` 根上下文 →
   自身包裹查询 → body 不在自身子树 → false。
3. ParentNode-querySelector-All 的 tree-order 4F 同源——结果数组的对象
   identity 与视图节点不一致（`expected [object Object] got [object
   Object]` 是两个不同 wrapper 实例）。

**深结构定性**：querySelector 产物归一是 R158/R171/R173 系列的架构域
（per-root wrapper 缓存/JSON 往返归一/`:lang`/`:target` 消费面），改它波及
全套选择器用例（R171 曾实测 902F 依赖）——非轻量切片，转 R292 靶点按
「wrapper→视图对象归一（body/head/html 特例先行）」立项。

## 四、R292 靶点

- **(a) querySelector 产物 wrapper→视图归一（body/head/html 特例先行）**
  （iframe 子文档 doc.querySelector('#body'/'body') 直返 doc.body 视图对象
  ——matches/tree-order 6F 簇的公共根因；R158 缓存键复用）。
- **(b) insertAdjacentText/insert-adjacent 2F**（独立小簇）。
- **(c) MutationObserver 8F**（childList Range 系 3F + document parser 3F +
  inner-outer 2F）。
- **(d) Text/Comment-constructor 跨 globals 2F**（iframe doc childNodes
  计数 1 vs 2——doctype 链入域）。
