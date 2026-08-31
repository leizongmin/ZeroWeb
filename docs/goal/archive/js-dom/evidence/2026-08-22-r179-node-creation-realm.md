# R179 Evidence — node-creation-realm 跨 realm 工厂通道（M4）

**日期**: 2026-08-22
**切片**: M4 轻量——node-creation-realm 13F → 0F（13P/13 全 100%），全量净 +14P/-14F
**改动面**: part03（Document/DOMImplementation prototype 工厂转发 + 节点原型链 + head 查询段）+ part05（iframe win 构造器转发 + 工厂 attachShadow）+ part06（implementation 接原型 + Range.prototype 通道）

## 一、测试形态

WPT node-creation-realm（whatwg/dom#977）：`inner.Document.prototype[name].apply(document, args)` ——用 **iframe realm 的接口方法**在**顶层 realm 的 document** 上创建节点。这要求接口方法在 **prototype** 上可达。沙箱是单 realm（inner 构造器转发主 realm），但 shim 的工厂方法全挂 document **对象自身**——`Xxx.prototype[method]` undefined 直接 `.apply` 崩。

## 二、修复（六件）

| 件 | 内容 |
|----|------|
| **Document.prototype 工厂转发** | createElement(NS)/createTextNode/createComment/createProcessingInstruction/createDocumentFragment/createAttribute(NS)/createCDATASection/createRange/importNode/adoptNode —— prototype 通道转发到 `this` 同名 own 方法（own 优先命中，零既有行为变化） |
| **DOMImplementation 构造器** | 旧 implementation 是匿名对象字面量——建构造器 + prototype 工厂转发（hasFeature/createDocumentType/createDocument/createHTMLDocument），主 document 的 implementation 对象 setPrototypeOf 接入 |
| **iframe win 构造器转发补齐** | DOMImplementation/Range/StaticRange（旧缺——`inner.DOMImplementation` undefined） |
| **Range.prototype 方法通道** | `_makeRange` 产物接 Range.prototype（new Range()/createRange() 双入口）；prototype 方法以模板 own 方法转发 |
| **节点原型链** | `_zwMText` → Text.prototype、`_zwMComment` → Comment.prototype、CDATA 产物 → CDATASection.prototype（instanceof 断言族） |
| **工厂元素 attachShadow** | 轻量 shadow root（innerHTML setter 经 `_zwMBuildBodyTree` 解析，firstChild/lastChild/childNodes）——工厂元素无 sel/handle 不能走 part04 `_attachShadow` |

## 三、过程中的回归与修正

title 查询修复首版把 `<head><title>` 前置到 **bodyHtml**——查询树把 head 段建成
body 子，`DOMImplementation-createHTMLDocument` 的 `body.childNodes.length === 0`
回归 7F（A/B 门当场抓回）。修正：head 段经 `_makeDetachedDocument` 局部变量
`_r179HeadHtml` 并入 **detHtml 包装层**（`<html><head>…</head><body>`——查询可见、
body 视图零变化）。第二版踩字面量求值时序坑（doc 赋值前访问 doc 槽 → undefined
赋值崩整页）——局部变量收口。

## 四、验证

| 门 | 结果 |
|----|------|
| node-creation-realm | 0P/13F → **13P/0F（100%）** |
| DOMImplementation-createHTMLDocument | 回归 7F 全数收回（15P/0F） |
| Document-implementation | 连带转绿 +1P |
| 全量 dom WPT polyfill | **9638P/225F/20T**（R178 9624P/239F——**净 +14P/-14F 零回归**） |
| 全量 dom WPT native | **9638P/225F/20T**，per-file 唯一分歧 = 2 个 crash-flaky 超时互换 |
| `make test` | 66 套件 **18128P/0F** 一次通过 |
| fmt / clippy | 干净 |

## 五、下一步（R180）

- 全量 fail Top 簇：Event-dispatch-single-activation-behavior 14F / node-realm-* adoption 族。
- tree order 2F 记 RFC（identity 归一域）。
- M2/M6 面：S6 高层 API 去字符串 / native dom_bindings 补齐。
